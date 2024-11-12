use std::path::{Path, PathBuf};

use biome_analyze::{context::RuleContext, declare_lint_rule, Rule, RuleDiagnostic};
use biome_console::markup;
use biome_js_dependency_graph::DependencyGraphModel;
use rustc_hash::FxHashMap;

use crate::services::dependency_graph::DependencyGraph;

declare_lint_rule! {
    /// Disallows import cycles between modules.
    ///
    /// Currently this can only detect cycles between relative paths that start
    /// with a dot. TypeScript aliases are not yet supported.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// **a.js:**
    /// ```js
    /// import "./b.js";
    /// ```
    ///
    /// **b.js:**
    /// ```js
    /// import "./a.js";
    /// ```
    ///
    pub NoImportCycles {
        version: "next",
        name: "noImportCycles",
        language: "js",
        recommended: false,
    }
}

impl Rule for NoImportCycles {
    type Query = DependencyGraph;
    type State = Vec<PathBuf>;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let result = CycleFinder::with_model(ctx.query()).resolve_all_from_path(ctx.file_path());
        if let Err(paths) = result {
            Some(paths)
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, paths: &Self::State) -> Option<RuleDiagnostic> {
        let dependency_graph = ctx.query();
        let import = paths
            .first()
            .and_then(|path| dependency_graph.imports_for_path(path))?;

        let import_node = paths
            .get(1)
            .and_then(|path| import.get_node_for_resolved_import(path))?;

        let mut diagnostic = RuleDiagnostic::new(
            rule_category!(),
            import_node.text_trimmed_range(),
            markup! {
                "Import cycle detected."
            },
        );

        for path in paths.iter().skip(2) {
            diagnostic = diagnostic.note(markup! { "Imported: "{path.display().to_string()} });
        }

        Some(diagnostic)
    }
}

pub struct CycleFinder<'a> {
    dependency_graph: &'a DependencyGraphModel,
    module_state: FxHashMap<&'a Path, ModuleState>,
    current_paths: Vec<&'a Path>,
}

impl<'a> CycleFinder<'a> {
    pub fn with_model(dependency_graph: &'a DependencyGraphModel) -> Self {
        Self {
            dependency_graph,
            module_state: Default::default(),
            current_paths: Default::default(),
        }
    }

    fn resolve_all_from_path(&mut self, path: &'a Path) -> Result<(), Vec<PathBuf>> {
        if let Some(imports) = self.dependency_graph.imports_for_path(path) {
            self.current_paths.push(path);
            self.module_state.insert(path, ModuleState::Resolving);

            for (_, import) in imports.all_analyzable_imports() {
                let Some(resolved_path) = &import.resolved_path else {
                    continue;
                };

                match self.module_state.get(resolved_path.as_path()) {
                    Some(&ModuleState::Resolving) => {
                        // Cycle found.
                        return Err(self
                            .current_paths
                            .iter()
                            .map(|path| path.to_path_buf())
                            .chain([resolved_path.clone()])
                            .collect());
                    }
                    Some(&ModuleState::AllResolved) => continue,
                    None => self.resolve_all_from_path(resolved_path)?,
                }
            }

            self.module_state.insert(path, ModuleState::AllResolved);
            self.current_paths.pop();
        }

        Ok(())
    }
}

enum ModuleState {
    Resolving,
    AllResolved,
}
