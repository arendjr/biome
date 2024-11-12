use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use biome_js_syntax::JsSyntaxNode;

/// Model that holds all dependency graph information.
#[derive(Clone, Debug, Default)]
pub struct DependencyGraphModel {
    /// Map containing all the modules within the project's scope and
    /// collected information about all the imports from those modules.
    pub(crate) modules: BTreeMap<PathBuf, ModuleImports>,
}

impl DependencyGraphModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn imports_for_path(&self, path: &Path) -> Option<&ModuleImports> {
        self.modules.get(path)
    }
}

#[derive(Clone, Debug)]
pub struct ModuleImports {
    /// Map of all static imports found in the module.
    ///
    /// Maps from the identifier found in the import statement to the absolute
    /// path it resolves to. The resolved path may be looked up as key in the
    /// [DependencyGraphModel::modules] map, although it is not required to
    /// be there (for instance, if the path is outside the project's scope).
    pub static_imports: BTreeMap<String, Import>,

    /// Map of all dynamic imports found in the module for which the import
    /// identifier could be statically determined.
    ///
    /// Dynamic import statements for which the identifier cannot be statically
    /// determined (for instance, because a template string with variables is
    /// used) will be omitted from this map and are be placed in
    /// [Self::unresolved_dynamic_imports] instead.
    ///
    /// Maps from the identifier found in the import statement to the absolute
    /// path it resolves to. The resolved path may be looked up as key in the
    /// [DependencyGraphModel::modules] map, although it is not required to
    /// be there (for instance, if the path is outside the project's scope).
    pub dynamic_imports: BTreeMap<String, Import>,

    /// Collection of dynamic import statements that cannot be statically
    /// analyzed due to the presence of variables in the import identifier.
    pub variable_dynamic_imports: BTreeSet<JsSyntaxNode>,
}

impl ModuleImports {
    pub fn all_analyzable_imports(&self) -> impl Iterator<Item = (&str, &Import)> {
        self.static_imports
            .iter()
            .chain(self.dynamic_imports.iter())
            .map(|(path, import)| (path.as_str(), import))
    }

    pub fn get_node_for_resolved_import(&self, path: &Path) -> Option<JsSyntaxNode> {
        self.all_analyzable_imports()
            .find(|(_, import)| {
                import
                    .resolved_path
                    .as_ref()
                    .is_some_and(|resolved| resolved == path)
            })
            .map(|(_, import)| import.node.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Import {
    /// The node with the import statement.
    pub node: JsSyntaxNode,

    /// Absolute path of the resource being imported, if it can be resolved.
    ///
    /// If the import statement referred to a package dependency, the path will
    /// point towards the resolved entry point of the package.
    ///
    /// If `None`, import resolution failed.
    pub resolved_path: Option<PathBuf>,
}
