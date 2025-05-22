use crate::prelude::*;
use biome_formatter::write;

use crate::utils::FormatStatementSemicolon;

use biome_js_syntax::JsImport;
use biome_js_syntax::JsImportFields;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatJsImport;

impl FormatNodeRule<JsImport> for FormatJsImport {
    fn fmt_fields(&self, node: &JsImport, f: &mut JsFormatter) -> FormatResult<()> {
        let JsImportFields {
            import_token,
            specifier,
            source,
            assertion,
            semicolon_token,
        } = node.as_fields();

        write!(
            f,
            [
                import_token.format(),
                space(),
                specifier.format(),
                space(),
                source.format(),
            ]
        )?;

        if let Some(assertion) = assertion {
            write!(f, [assertion.format()])?;
        }

        write!(f, [FormatStatementSemicolon::new(semicolon_token.as_ref())])
    }
}
