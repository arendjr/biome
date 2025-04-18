mod utils;

use biome_js_formatter::context::JsFormatOptions;
use biome_js_formatter::format_node;
use biome_js_parser::{JsParserOptions, parse};
use biome_js_syntax::AnyJsModuleItem;
use biome_js_syntax::AnyJsRoot;
use biome_js_syntax::AnyJsStatement;
use biome_js_syntax::JsFileSource;
use biome_js_syntax::JsVariableDeclaration;
use biome_js_type_info::{Type, TypeInner};
use biome_rowan::AstNode;
use biome_rowan::Text;

use utils::{assert_type_snapshot, get_function_declaration, parse_ts};

#[test]
fn infer_type_of_promise_returning_function() {
    const CODE: &str = r#"function returnsPromise(): Promise<number> {
    return Promise.resolved(true);
}"#;

    let root = parse_ts(CODE);
    let decl = get_function_declaration(&root);
    let ty = Type::from_js_function_declaration(&decl);
    assert_type_snapshot(CODE, ty, "infer_type_of_promise_returning_function");
}

#[test]
fn infer_type_of_async_function() {
    const CODE: &str = r#"async function returnsPromise(): Promise<string> {
	return "value";
}"#;

    let root = parse_ts(CODE);
    let decl = get_function_declaration(&root);
    let ty = Type::from_js_function_declaration(&decl);
    assert_type_snapshot(CODE, ty, "infer_type_of_async_function");
}

#[test]
fn infer_type_of_array_element() {
    const CODE: &str = r#"const array: Array<string> = [];"#;

    let root = parse_ts(CODE);
    let decl = get_variable_declaration(&root);
    let bindings = Type::typed_bindings_from_js_variable_declaration(&decl);
    assert_typed_bindings_snapshot(CODE, &bindings, "infer_type_of_array_element");
}

#[test]
fn infer_type_of_destructured_array_element() {
    const CODE: &str = r#"const [a]: Array<string> = [];"#;

    let root = parse_ts(CODE);
    let decl = get_variable_declaration(&root);
    let bindings = Type::typed_bindings_from_js_variable_declaration(&decl);
    assert_typed_bindings_snapshot(CODE, &bindings, "infer_type_of_destructured_array_element");
}

#[test]
#[cfg(target_pointer_width = "64")]
fn verify_type_sizes() {
    assert_eq!(
        std::mem::size_of::<Type>(),
        8,
        "`Type` should not be bigger than 8 bytes"
    );
    assert_eq!(
        std::mem::size_of::<TypeInner>(),
        16,
        "`TypeInner` should not be bigger than 16 bytes"
    );
}

fn assert_typed_bindings_snapshot(
    source_code: &str,
    typed_bindings: &[(Text, Type)],
    test_name: &str,
) {
    let mut content = String::new();

    let source_type = JsFileSource::ts();
    let tree = parse(source_code, source_type, JsParserOptions::default());
    let formatted = format_node(JsFormatOptions::default(), tree.tree().syntax())
        .unwrap()
        .print()
        .unwrap();

    content.push_str("```");
    content.push_str("ts");
    content.push('\n');
    content.push_str(formatted.as_code());
    content.push_str("\n```");

    content.push_str("\n\n");
    content.push_str("```\n");
    for (name, ty) in typed_bindings {
        content.push_str(&format!("{name} => {ty}\n"));
    }
    content.push_str("\n```\n\n");

    insta::with_settings!({
        snapshot_path => "../snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(test_name, content);
    });
}

fn get_variable_declaration(root: &AnyJsRoot) -> JsVariableDeclaration {
    let module = root.as_js_module().unwrap();
    module
        .items()
        .into_iter()
        .filter_map(|item| match item {
            AnyJsModuleItem::AnyJsStatement(statement) => Some(statement),
            _ => None,
        })
        .find_map(|statement| match statement {
            AnyJsStatement::JsVariableStatement(statement) => statement.declaration().ok(),
            _ => None,
        })
        .expect("cannot find variable declaration")
}
