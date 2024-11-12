use biome_js_syntax::TextRange;

/// Events emitted by the [DependencyGraphEventExtractor] which are then
/// converted into the [super::DependencyGraphModel].
///
/// Events are only responsible for discovering all the import statements that
/// are tracked in the Dependency Graph Model. Import resolution is handled by
/// the Dependency Graph Builder instead.
#[derive(Debug, Eq, PartialEq)]
pub enum DependencyGraphEvent {
    /// Tracks where a dependency is imported.
    /// Generated for both static and dynamic imports.
    ImportFound {
        node: TextRange,

        /// Import identifier, if it can be statically determined.
        ///
        /// For import identifiers that contain variables, this will be `None`.
        identifier: Option<String>,
    },
}
