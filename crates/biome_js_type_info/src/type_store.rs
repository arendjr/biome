use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use hashbrown::{HashTable, hash_table::Entry};
use rustc_hash::FxHasher;

use crate::{Resolvable, ResolvedTypeId, TypeData, TypeId, TypeReference, TypeResolverLevel};

/// Type store with efficient lookup mechanism.
///
/// Type resolvers are responsible for storing types. The simplest resolvers can
/// store their types in a `Vec`, but this makes registration expensive when
/// they try to avoid registering duplicate types. For this reason, the
/// `TypeStore` exists.
///
/// `TypeStore` offers extremely fast lookups by `TypeId`, as fast as when using
/// a `Vec`, but offers the performance of a hash set when looking up a
/// `TypeData` reference.
///
/// Unlike a hash set, `TypeStore` allows duplicate types to be stored, even if
/// some amount of deduplication is preferred. The reason for this is that type
/// flattening can reduce disjoint types to the same type, at which point we
/// store both to preserve their indices.
#[derive(Debug, Default)]
pub struct TypeStore {
    types: Vec<TypeData>,
    table: HashTable<usize>,
}

impl TypeStore {
    pub fn from_types(types: Vec<TypeData>) -> Self {
        let mut table = HashTable::new();
        for (i, data) in types.iter().enumerate() {
            let hash = hash_data(data);

            table.insert_unique(hash, i, |i| hash_data(&types[*i]));
        }

        Self { types, table }
    }

    pub fn as_slice(&self) -> &[TypeData] {
        &self.types
    }

    /// Returns a deduplicated version of this type store.
    ///
    /// Also returns the vectors necessary for mapping references.
    pub fn deduplicated(self, level: TypeResolverLevel) -> (Self, Vec<Option<u32>>, Vec<u32>) {
        // First find all duplicates and create a map where the "original" can
        // be found. Whichever index the hash table points at is considered the
        // original.
        let mut map = Vec::with_capacity(self.types.len());

        // While populating the map, we also track how many types have been
        // removed at a given index. We need this for compensating the resolved
        // IDs.
        let mut num_removed_types = 0;
        let mut num_removed_types_at_index = Vec::with_capacity(self.types.len());

        for (i, data) in self.types.iter().enumerate() {
            let hash = hash_data(data);
            let expected_index = self
                .table
                .find(hash, |i| self.types[*i] == *data)
                .map(|index| *index as u32);

            if expected_index.is_none_or(|index| index == i as u32) {
                map.push(None);
            } else {
                map.push(expected_index);
                num_removed_types += 1;
            }

            num_removed_types_at_index.push(num_removed_types)
        }

        if num_removed_types == 0 {
            return (self, map, num_removed_types_at_index); // Nothing to do.
        }

        // Move all originals into a new vector, mapping resolved IDs as we go.
        let types = self
            .types
            .into_iter()
            .enumerate()
            .filter(|(i, _)| map[*i].is_none())
            .map(|(_, mut data)| {
                data.update_all_references(|reference| match reference {
                    TypeReference::Resolved(resolved_id) if resolved_id.level() == level => {
                        let old_index = resolved_id.index();
                        let new_index = map[old_index].unwrap_or(old_index as u32);
                        let new_index = new_index - num_removed_types_at_index[new_index as usize];
                        *resolved_id = ResolvedTypeId::new(level, TypeId::new(new_index as usize));
                    }
                    _ => {}
                });
                data
            })
            .collect();

        (Self::from_types(types), map, num_removed_types_at_index)
    }

    /// Returns the `TypeId` of the given `data`, if it is registered.
    pub fn find_type(&self, data: &TypeData) -> Option<TypeId> {
        let hash = hash_data(data);

        self.table
            .find(hash, |i| self.types[*i] == *data)
            .map(|index| TypeId::new(*index))
    }

    pub fn get_by_id(&self, id: TypeId) -> &TypeData {
        &self.types[id.index()]
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Registers the given `data` if it is not registered yet.
    ///
    /// Returns the `TypeId` of the newly registered data, or the `TypeId` of
    /// an already registered equivalent type.
    pub fn register_type(&mut self, data: Cow<TypeData>) -> TypeId {
        let entry = self.table.entry(
            hash_data(&data),
            |i| &self.types[*i] == data.as_ref(),
            |i| hash_data(&self.types[*i]),
        );
        match entry {
            Entry::Occupied(entry) => TypeId::new(*entry.get()),
            Entry::Vacant(entry) => {
                let index = self.types.len();
                self.types.push(data.into_owned());
                entry.insert(index);
                TypeId::new(index)
            }
        }
    }

    /// Takes the type at the given `index` and swaps it for
    /// `TypeData::Unknown`.
    ///
    /// The returned type is correctly removed from the store's hash table, but
    /// the `TypeData::Unknown` that takes its place is never registered. This
    /// is done on the assumption that an updated type will be reinserted before
    /// any other lookups are performed. This allows us to avoid unnecessarily
    /// update the hash table twice, but makes the API slightly unsafe to use.
    ///
    /// # Safety
    ///
    /// Callers must promise to reinsert the (updated) value before new lookups
    /// are performed. They must use
    /// [`Self::reinsert_temporarily_taken_data()`] for this.
    pub unsafe fn take_from_index_temporarily(&mut self, index: usize) -> TypeData {
        let data = std::mem::take(&mut self.types[index]);

        let hash = hash_data(&data);

        if let Ok(occupied) = self.table.find_entry(hash, |i| self.types[*i] == data) {
            occupied.remove();
        }

        data
    }

    /// Reinserts an (updated) value that was taken using
    /// [`Self::take_from_index_temporarily()`].
    ///
    /// # Safety
    ///
    /// Callers must only call this once after calling
    /// [`Self::take_from_index_temporarily()`] with the same index, before any
    /// lookups are performed.
    pub unsafe fn reinsert_temporarily_taken_data(&mut self, index: usize, data: TypeData) {
        let entry = self.table.entry(
            hash_data(&data),
            |i| self.types[*i] == data,
            |i| hash_data(&self.types[*i]),
        );
        if let Entry::Vacant(entry) = entry {
            entry.insert(index);
        }

        self.types[index] = data;
    }
}

impl From<TypeStore> for Box<[TypeData]> {
    fn from(store: TypeStore) -> Self {
        store.types.into()
    }
}

#[inline(always)]
fn hash_data(data: &TypeData) -> u64 {
    let mut hash = FxHasher::default();
    data.hash(&mut hash);
    hash.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let store = TypeStore::from_types(vec![
            TypeData::String,
            TypeData::Number,
            TypeData::String,
            TypeData::Reference(TypeReference::Resolved(ResolvedTypeId::new(
                TypeResolverLevel::Thin,
                TypeId::new(2),
            ))),
        ]);
        let expected = &[
            TypeData::String,
            TypeData::Number,
            TypeData::Reference(TypeReference::Resolved(ResolvedTypeId::new(
                TypeResolverLevel::Thin,
                TypeId::new(0),
            ))),
        ];
        assert_eq!(
            store.deduplicated(TypeResolverLevel::Thin).0.as_slice(),
            expected
        );

        let store = TypeStore::from_types(vec![
            TypeData::String,
            TypeData::Number,
            TypeData::String,
            TypeData::Reference(TypeReference::Resolved(ResolvedTypeId::new(
                TypeResolverLevel::Thin,
                TypeId::new(2),
            ))),
            TypeData::Reference(TypeReference::Resolved(ResolvedTypeId::new(
                TypeResolverLevel::Thin,
                TypeId::new(6),
            ))),
            TypeData::Number,
            TypeData::Null,
        ]);
        let expected = &[
            TypeData::String,
            TypeData::Number,
            TypeData::Reference(TypeReference::Resolved(ResolvedTypeId::new(
                TypeResolverLevel::Thin,
                TypeId::new(0),
            ))),
            TypeData::Reference(TypeReference::Resolved(ResolvedTypeId::new(
                TypeResolverLevel::Thin,
                TypeId::new(4),
            ))),
            TypeData::Null,
        ];
        assert_eq!(
            store.deduplicated(TypeResolverLevel::Thin).0.as_slice(),
            expected
        );
    }
}
