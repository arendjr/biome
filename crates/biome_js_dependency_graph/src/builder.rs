use std::{collections::BTreeMap, path::PathBuf};

/// Builds the [DependencyGraphModel] consuming [DependencyGraphEvent] and
/// [JsSyntaxNode]. For a good example on how to use it see [dependency_graph].
///
/// [DependencyGraphModelBuilder] consumes all the [DependencyGraphEvent] and
/// builds all the data necessary to build a semantic model, that is allocated
/// with an [std::sync::Arc] and stored inside the [DependencyGraphModel].
pub struct DependencyGraphModelBuilder {
    modules: BTreeMap<PathBuf, ModuleDependencies>,
}

impl DependencyGraphModelBuilder {
    pub fn new() -> Self {
        Self {
            modules: Default::default(),
        }
    }
}

struct ModuleDependencies {}
