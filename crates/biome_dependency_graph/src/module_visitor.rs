use biome_js_syntax::{
    AnyJsBinding, AnyJsBindingPattern, AnyJsDeclarationClause, AnyJsExportClause, AnyJsImportLike,
    AnyJsModuleSource, AnyJsRoot, AnyTsIdentifierBinding, AnyTsModuleName, JsExport,
    JsExportFromClause, JsExportNamedFromClause, JsExportNamedSpecifierList,
    JsVariableDeclaratorList,
};
use biome_rowan::{AstNode, TriviaPieceKind, WalkEvent};
use camino::{Utf8Path, Utf8PathBuf};
use oxc_resolver::{ResolveError, ResolverGeneric};

use crate::{
    dependency_graph::{Export, Import, ModuleDependencyData, OwnExport, ReexportAll},
    resolver_cache::ResolverCache,
};

pub(crate) struct ModuleVisitor<'a> {
    root: AnyJsRoot,
    directory: &'a Utf8Path,
    resolver: &'a ResolverGeneric<ResolverCache<'a>>,
    data: ModuleDependencyData,
}

impl<'a> ModuleVisitor<'a> {
    pub fn new(
        root: AnyJsRoot,
        directory: &'a Utf8Path,
        resolver: &'a ResolverGeneric<ResolverCache<'a>>,
    ) -> Self {
        Self {
            root,
            directory,
            resolver,
            data: Default::default(),
        }
    }

    pub fn collect_data(mut self) -> ModuleDependencyData {
        let iter = self.root.syntax().preorder();
        for event in iter {
            match event {
                WalkEvent::Enter(node) => {
                    if let Some(import) = AnyJsImportLike::cast_ref(&node) {
                        self.visit_import(import);
                    } else if let Some(export) = JsExport::cast_ref(&node) {
                        self.visit_export(export);
                    }
                }
                WalkEvent::Leave(_) => {}
            }
        }

        self.data
    }

    fn visit_import(&mut self, node: AnyJsImportLike) {
        let Some(specifier) = node.inner_string_text() else {
            return;
        };

        let import = self.import_from_specifier(specifier.text());

        match node {
            AnyJsImportLike::JsModuleSource(_) => {
                self.data
                    .static_imports
                    .insert(specifier.to_string(), import);
            }
            AnyJsImportLike::JsCallExpression(_) | AnyJsImportLike::JsImportCallExpression(_) => {
                self.data
                    .dynamic_imports
                    .insert(specifier.to_string(), import);
            }
        }
    }

    fn visit_export(&mut self, node: JsExport) -> Option<()> {
        let jsdoc_comment = node.export_token().ok().and_then(|token| {
            token
                .leading_trivia()
                .pieces()
                .rev()
                .find_map(|trivia| match trivia.kind() {
                    TriviaPieceKind::MultiLineComment | TriviaPieceKind::SingleLineComment => {
                        (trivia.text().starts_with("/**") && trivia.text().ends_with("*/"))
                            .then(|| normalize_jsdoc_comment(trivia.text()))
                    }
                    _ => None,
                })
        });

        match node.export_clause().ok()? {
            AnyJsExportClause::AnyJsDeclarationClause(node) => {
                self.visit_export_declaration_clause(node, jsdoc_comment)
            }
            AnyJsExportClause::JsExportDefaultDeclarationClause(_) => {
                self.register_export_with_name_and_jsdoc_comment("default", jsdoc_comment)
            }
            AnyJsExportClause::JsExportDefaultExpressionClause(_) => {
                self.register_export_with_name_and_jsdoc_comment("default", jsdoc_comment)
            }
            AnyJsExportClause::JsExportFromClause(node) => self.visit_export_from_clause(node),
            AnyJsExportClause::JsExportNamedClause(node) => {
                self.visit_export_named_specifiers(node.specifiers(), jsdoc_comment)
            }
            AnyJsExportClause::JsExportNamedFromClause(node) => {
                self.visit_export_named_from_clause(node)
            }
            AnyJsExportClause::TsExportAsNamespaceClause(node) => {
                let token = node.name().ok()?.value_token().ok()?;
                let name = token.text_trimmed();
                self.register_export_with_name_and_jsdoc_comment(name, jsdoc_comment)
            }
            AnyJsExportClause::TsExportAssignmentClause(_) => {
                // This is somewhat misleading, since the `export =` syntax is
                // used for CommonJS exports rather than ES6 `default` exports.
                // Thankfully, bundlers are responsible for normalising this,
                // which isn't really Biome's concern.
                self.register_export_with_name_and_jsdoc_comment("default", jsdoc_comment)
            }
            AnyJsExportClause::TsExportDeclareClause(node) => {
                self.visit_export_declaration_clause(node.declaration().ok()?, jsdoc_comment)
            }
        }
    }

    fn visit_export_declaration_clause(
        &mut self,
        node: AnyJsDeclarationClause,
        jsdoc_comment: Option<String>,
    ) -> Option<()> {
        let name = match node {
            AnyJsDeclarationClause::JsClassDeclaration(node) => {
                node.id().ok().and_then(get_name)?
            }
            AnyJsDeclarationClause::JsFunctionDeclaration(node) => {
                node.id().ok().and_then(get_name)?
            }
            AnyJsDeclarationClause::JsVariableDeclarationClause(node) => {
                return self.visit_export_variable_declarations(
                    node.declaration().ok()?.declarators(),
                    jsdoc_comment,
                );
            }
            AnyJsDeclarationClause::TsDeclareFunctionDeclaration(node) => {
                node.id().ok().and_then(get_name)?
            }
            AnyJsDeclarationClause::TsEnumDeclaration(node) => node.id().ok().and_then(get_name)?,
            AnyJsDeclarationClause::TsExternalModuleDeclaration(_)
            | AnyJsDeclarationClause::TsGlobalDeclaration(_)
            | AnyJsDeclarationClause::TsImportEqualsDeclaration(_) => return None,
            AnyJsDeclarationClause::TsInterfaceDeclaration(node) => {
                node.id().ok().and_then(get_ts_name)?
            }
            AnyJsDeclarationClause::TsModuleDeclaration(node) => match node.name().ok()? {
                AnyTsModuleName::AnyTsIdentifierBinding(node) => get_ts_name(node)?,
                AnyTsModuleName::TsQualifiedModuleName(_) => return None,
            },
            AnyJsDeclarationClause::TsTypeAliasDeclaration(node) => {
                node.binding_identifier().ok().and_then(get_ts_name)?
            }
        };

        self.register_export_with_name_and_jsdoc_comment(name, jsdoc_comment)
    }

    fn visit_export_from_clause(&mut self, node: JsExportFromClause) -> Option<()> {
        let module_source = node.source().ok()?;
        let import = self.import_from_module_source(module_source)?;

        self.data.blanket_reexports.push(ReexportAll { import });

        Some(())
    }

    fn visit_export_named_from_clause(&mut self, node: JsExportNamedFromClause) -> Option<()> {
        let module_source = node.source().ok()?;
        let import = self.import_from_module_source(module_source)?;

        for specifier in node.specifiers() {
            let Ok(specifier) = specifier else {
                continue;
            };

            let name = specifier
                .export_as()
                .and_then(|export_as| export_as.exported_name().ok())
                .or_else(|| specifier.source_name().ok())?;
            self.data
                .exports
                .insert(name.to_trimmed_string(), Export::Reexport(import.clone()));
        }

        Some(())
    }

    fn visit_export_named_specifiers(
        &mut self,
        specifiers: JsExportNamedSpecifierList,
        jsdoc_comment: Option<String>,
    ) -> Option<()> {
        Some(())
    }

    fn visit_export_variable_declarations(
        &mut self,
        declarators: JsVariableDeclaratorList,
        jsdoc_comment: Option<String>,
    ) -> Option<()> {
        for declarator in declarators {
            let Ok(declarator) = declarator else {
                continue;
            };

            match declarator.id().ok()? {
                AnyJsBindingPattern::AnyJsBinding(node) => todo!(),
                AnyJsBindingPattern::JsArrayBindingPattern(node) => todo!(),
                AnyJsBindingPattern::JsObjectBindingPattern(node) => todo!(),
            }
        }

        Some(())
    }

    fn register_export_with_name_and_jsdoc_comment(
        &mut self,
        name: impl Into<String>,
        jsdoc_comment: Option<String>,
    ) -> Option<()> {
        self.data
            .exports
            .insert(name.into(), Export::Own(OwnExport { jsdoc_comment }));

        Some(())
    }

    fn import_from_module_source(&self, module_source: AnyJsModuleSource) -> Option<Import> {
        let specifier = module_source.as_js_module_source()?.value_token().ok()?;
        Some(self.import_from_specifier(specifier.text_trimmed()))
    }

    fn import_from_specifier(&self, specifier: &str) -> Import {
        let resolved_path =
            self.resolver
                .resolve(self.directory, specifier)
                .and_then(|resolution| {
                    Utf8PathBuf::from_path_buf(resolution.into_path_buf())
                        .map_err(|path| ResolveError::NotFound(path.to_string_lossy().to_string()))
                });
        Import { resolved_path }
    }
}

fn get_name(binding_result: AnyJsBinding) -> Option<String> {
    let name = binding_result
        .as_js_identifier_binding()?
        .name_token()
        .ok()?
        .text_trimmed()
        .to_string();
    Some(name)
}

fn get_ts_name(binding_result: AnyTsIdentifierBinding) -> Option<String> {
    let name = binding_result
        .as_ts_identifier_binding()?
        .name_token()
        .ok()?
        .text_trimmed()
        .to_string();
    Some(name)
}

/// Normalizes the text in a JSDoc comment by stripping the opening and trailing
/// markers, and trimming the leading asterisk from every line.
fn normalize_jsdoc_comment(text: &str) -> String {
    debug_assert!(text.starts_with("/**") && text.ends_with("*/"));

    text[3..text.len() - 2]
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix('*').map_or(trimmed, str::trim_start)
        })
        .fold(String::new(), |mut result, line| {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            result
        })
}
