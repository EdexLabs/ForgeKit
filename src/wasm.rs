#![cfg(feature = "wasm")]

use crate::parser::Parser;
use crate::utils::format_ast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    let (ast, errors) = Parser::new(source).parse();

    let errors_json: Vec<serde_json::Value> = errors
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "message": e.message,
                "span": { "start": e.span.start, "end": e.span.end },
                "kind": format!("{:?}", e.kind),
            })
        })
        .collect();

    serde_json::json!({
        "ast": format_ast(&ast),
        "errors": errors_json,
    })
    .to_string()
}
