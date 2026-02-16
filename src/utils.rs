//! Utility functions for working with the ForgeScript AST

use crate::parser::{AstNode, Span};

/// Pretty-print the AST to a string
pub fn format_ast(node: &AstNode) -> String {
    let mut output = String::new();
    format_ast_impl(node, &mut output, 0);
    output
}

fn format_ast_impl(node: &AstNode, output: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);

    match node {
        AstNode::Program { body, span } => {
            output.push_str(&format!(
                "{}Program ({}..{})\n",
                indent, span.start, span.end
            ));
            for child in body {
                format_ast_impl(child, output, depth + 1);
            }
        }
        AstNode::Text { content, span } => {
            output.push_str(&format!(
                "{}Text ({}..{}): {:?}\n",
                indent, span.start, span.end, content
            ));
        }
        AstNode::FunctionCall {
            name,
            args,
            modifiers,
            span,
        } => {
            output.push_str(&format!(
                "{}FunctionCall ({}..{}): ${}{}{}{}\n",
                indent,
                span.start,
                span.end,
                name,
                if modifiers.silent { " [silent]" } else { "" },
                if modifiers.negated { " [negated]" } else { "" },
                if let Some(count) = &modifiers.count {
                    format!(" [count: {}]", count)
                } else {
                    String::new()
                }
            ));
            if let Some(args) = args {
                for (i, arg) in args.iter().enumerate() {
                    output.push_str(&format!(
                        "{}  Arg {} ({}..{}):\n",
                        indent, i, arg.span.start, arg.span.end
                    ));
                    for part in &arg.parts {
                        format_ast_impl(part, output, depth + 2);
                    }
                }
            }
        }
        AstNode::JavaScript { code, span } => {
            output.push_str(&format!(
                "{}JavaScript ({}..{}): {:?}\n",
                indent, span.start, span.end, code
            ));
        }
        AstNode::Escaped { content, span } => {
            output.push_str(&format!(
                "{}Escaped ({}..{}): {:?}\n",
                indent, span.start, span.end, content
            ));
        }
    }
}

/// Extract all function names from the AST
pub fn extract_function_names(node: &AstNode) -> Vec<String> {
    let mut names = Vec::new();
    extract_function_names_impl(node, &mut names);
    names
}

fn extract_function_names_impl(node: &AstNode, names: &mut Vec<String>) {
    match node {
        AstNode::Program { body, .. } => {
            for child in body {
                extract_function_names_impl(child, names);
            }
        }
        AstNode::FunctionCall { name, args, .. } => {
            names.push(name.clone());
            if let Some(args) = args {
                for arg in args {
                    for part in &arg.parts {
                        extract_function_names_impl(part, names);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Count the number of nodes in the AST
pub fn count_nodes(node: &AstNode) -> usize {
    match node {
        AstNode::Program { body, .. } => 1 + body.iter().map(count_nodes).sum::<usize>(),
        AstNode::FunctionCall { args, .. } => {
            1 + args
                .as_ref()
                .map(|args| {
                    args.iter()
                        .map(|arg| arg.parts.iter().map(count_nodes).sum::<usize>())
                        .sum()
                })
                .unwrap_or(0)
        }
        _ => 1,
    }
}

/// Get all text nodes from the AST
pub fn extract_text_nodes(node: &AstNode) -> Vec<(String, Span)> {
    let mut texts = Vec::new();
    extract_text_nodes_impl(node, &mut texts);
    texts
}

fn extract_text_nodes_impl(node: &AstNode, texts: &mut Vec<(String, Span)>) {
    match node {
        AstNode::Program { body, .. } => {
            for child in body {
                extract_text_nodes_impl(child, texts);
            }
        }
        AstNode::Text { content, span } => {
            texts.push((content.clone(), *span));
        }
        AstNode::FunctionCall { args, .. } => {
            if let Some(args) = args {
                for arg in args {
                    for part in &arg.parts {
                        extract_text_nodes_impl(part, texts);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Find the deepest nesting level in the AST
pub fn max_nesting_depth(node: &AstNode) -> usize {
    max_nesting_depth_impl(node, 0)
}

fn max_nesting_depth_impl(node: &AstNode, current: usize) -> usize {
    match node {
        AstNode::Program { body, .. } => body
            .iter()
            .map(|n| max_nesting_depth_impl(n, current))
            .max()
            .unwrap_or(current),
        AstNode::FunctionCall { args, .. } => {
            let next = current + 1;
            args.as_ref()
                .map(|args| {
                    args.iter()
                        .flat_map(|arg| arg.parts.iter())
                        .map(|part| max_nesting_depth_impl(part, next))
                        .max()
                        .unwrap_or(next)
                })
                .unwrap_or(next)
        }
        _ => current,
    }
}

/// Check if the AST contains any JavaScript expressions
pub fn contains_javascript(node: &AstNode) -> bool {
    match node {
        AstNode::Program { body, .. } => body.iter().any(contains_javascript),
        AstNode::JavaScript { .. } => true,
        AstNode::FunctionCall { args, .. } => args
            .as_ref()
            .map(|args| {
                args.iter()
                    .flat_map(|arg| arg.parts.iter())
                    .any(contains_javascript)
            })
            .unwrap_or(false),
        _ => false,
    }
}

/// Get a slice of the source code for a given span
pub fn get_source_slice<'a>(source: &'a str, span: Span) -> &'a str {
    &source[span.start..span.end.min(source.len())]
}

/// Calculate statistics about the AST
#[derive(Debug, Clone)]
pub struct AstStats {
    pub total_nodes: usize,
    pub text_nodes: usize,
    pub function_calls: usize,
    pub javascript_nodes: usize,
    pub escaped_nodes: usize,
    pub max_depth: usize,
    pub unique_functions: usize,
}

pub fn calculate_stats(node: &AstNode) -> AstStats {
    let mut text_nodes = 0;
    let mut function_calls = 0;
    let mut javascript_nodes = 0;
    let mut escaped_nodes = 0;

    count_node_types(
        node,
        &mut text_nodes,
        &mut function_calls,
        &mut javascript_nodes,
        &mut escaped_nodes,
    );

    let function_names = extract_function_names(node);
    let mut unique = function_names.clone();
    unique.sort();
    unique.dedup();

    AstStats {
        total_nodes: count_nodes(node),
        text_nodes,
        function_calls,
        javascript_nodes,
        escaped_nodes,
        max_depth: max_nesting_depth(node),
        unique_functions: unique.len(),
    }
}

fn count_node_types(
    node: &AstNode,
    text: &mut usize,
    funcs: &mut usize,
    js: &mut usize,
    esc: &mut usize,
) {
    match node {
        AstNode::Program { body, .. } => {
            for child in body {
                count_node_types(child, text, funcs, js, esc);
            }
        }
        AstNode::Text { .. } => *text += 1,
        AstNode::FunctionCall { args, .. } => {
            *funcs += 1;
            if let Some(args) = args {
                for arg in args {
                    for part in &arg.parts {
                        count_node_types(part, text, funcs, js, esc);
                    }
                }
            }
        }
        AstNode::JavaScript { .. } => *js += 1,
        AstNode::Escaped { .. } => *esc += 1,
    }
}

/// Flatten the AST into a linear sequence of nodes (depth-first)
pub fn flatten_ast(node: &AstNode) -> Vec<AstNode> {
    let mut nodes = Vec::new();
    flatten_ast_impl(node, &mut nodes);
    nodes
}

fn flatten_ast_impl(node: &AstNode, nodes: &mut Vec<AstNode>) {
    nodes.push(node.clone());
    match node {
        AstNode::Program { body, .. } => {
            for child in body {
                flatten_ast_impl(child, nodes);
            }
        }
        AstNode::FunctionCall { args, .. } => {
            if let Some(args) = args {
                for arg in args {
                    for part in &arg.parts {
                        flatten_ast_impl(part, nodes);
                    }
                }
            }
        }
        _ => {}
    }
}
