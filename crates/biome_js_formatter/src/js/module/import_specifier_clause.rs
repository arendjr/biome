use crate::prelude::*;

use biome_formatter::write;
use biome_js_syntax::{JsImportSpecifierClause, JsImportSpecifierClauseFields};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatJsImportSpecifierClause;
impl FormatNodeRule<JsImportSpecifierClause> for FormatJsImportSpecifierClause {
    fn fmt_fields(&self, node: &JsImportSpecifierClause, f: &mut JsFormatter) -> FormatResult<()> {
        let JsImportSpecifierClauseFields {
            from_token,
            import_clause,
        } = node.as_fields();

        write!(f, [import_clause.format(), space(), from_token.format()])
    }
}
