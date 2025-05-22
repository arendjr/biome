use crate::prelude::*;

use biome_formatter::write;
use biome_js_syntax::JsImportCombinedClause;
use biome_js_syntax::JsImportCombinedClauseFields;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatJsImportCombinedClause;

impl FormatNodeRule<JsImportCombinedClause> for FormatJsImportCombinedClause {
    fn fmt_fields(&self, node: &JsImportCombinedClause, f: &mut JsFormatter) -> FormatResult<()> {
        let JsImportCombinedClauseFields {
            default_specifier,
            comma_token,
            specifier,
        } = node.as_fields();
        write![
            f,
            [
                default_specifier.format(),
                comma_token.format(),
                space(),
                specifier.format(),
            ]
        ]
    }
}
