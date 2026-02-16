//! Visitor pattern for traversing the ForgeScript AST
//!
//! This module provides a clean way to traverse and analyze the AST
//! without modifying the core parser code.

use crate::parser::{Argument, AstNode, Modifiers, Span};

/// Trait for visiting AST nodes
pub trait AstVisitor {
    /// Visit a program node
    fn visit_program(&mut self, body: &[AstNode], _span: Span) {
        for node in body {
            self.visit(node);
        }
    }

    /// Visit a text node
    fn visit_text(&mut self, content: &str, span: Span) {
        let _ = (content, span);
    }

    /// Visit a function call node
    fn visit_function_call(
        &mut self,
        name: &str,
        args: Option<&Vec<Argument>>,
        modifiers: &Modifiers,
        span: Span,
    ) {
        let _ = (name, modifiers, span);
        if let Some(args) = args {
            for arg in args {
                self.visit_argument(arg);
            }
        }
    }

    /// Visit an argument
    fn visit_argument(&mut self, arg: &Argument) {
        for part in &arg.parts {
            self.visit(part);
        }
    }

    /// Visit a JavaScript expression node
    fn visit_javascript(&mut self, code: &str, span: Span) {
        let _ = (code, span);
    }

    /// Visit an escaped content node
    fn visit_escaped(&mut self, content: &str, span: Span) {
        let _ = (content, span);
    }

    /// Dispatch to the appropriate visit method
    fn visit(&mut self, node: &AstNode) {
        match node {
            AstNode::Program { body, span } => self.visit_program(body, *span),
            AstNode::Text { content, span } => self.visit_text(content, *span),
            AstNode::FunctionCall {
                name,
                args,
                modifiers,
                span,
            } => self.visit_function_call(name, args.as_ref(), modifiers, *span),
            AstNode::JavaScript { code, span } => self.visit_javascript(code, *span),
            AstNode::Escaped { content, span } => self.visit_escaped(content, *span),
        }
    }
}

/// Example visitor that collects all function names
pub struct FunctionCollector {
    pub functions: Vec<String>,
}

impl FunctionCollector {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }
}

impl AstVisitor for FunctionCollector {
    fn visit_function_call(
        &mut self,
        name: &str,
        args: Option<&Vec<Argument>>,
        modifiers: &Modifiers,
        span: Span,
    ) {
        self.functions.push(name.to_string());

        // Continue visiting arguments
        let _ = (modifiers, span);
        if let Some(args) = args {
            for arg in args {
                self.visit_argument(arg);
            }
        }
    }
}

/// Example visitor that counts node types
#[derive(Default)]
pub struct NodeCounter {
    pub text_nodes: usize,
    pub function_nodes: usize,
    pub javascript_nodes: usize,
    pub escaped_nodes: usize,
}

impl AstVisitor for NodeCounter {
    fn visit_text(&mut self, _content: &str, _span: Span) {
        self.text_nodes += 1;
    }

    fn visit_function_call(
        &mut self,
        _name: &str,
        args: Option<&Vec<Argument>>,
        _modifiers: &Modifiers,
        _span: Span,
    ) {
        self.function_nodes += 1;
        if let Some(args) = args {
            for arg in args {
                self.visit_argument(arg);
            }
        }
    }

    fn visit_javascript(&mut self, _code: &str, _span: Span) {
        self.javascript_nodes += 1;
    }

    fn visit_escaped(&mut self, _content: &str, _span: Span) {
        self.escaped_nodes += 1;
    }
}

/// Mutable visitor trait for transforming AST
pub trait AstVisitorMut {
    /// Visit and possibly transform a node
    fn visit_mut(&mut self, node: &mut AstNode) {
        match node {
            AstNode::Program { body, span } => self.visit_program_mut(body, *span),
            AstNode::Text { content, span } => self.visit_text_mut(content, *span),
            AstNode::FunctionCall {
                name,
                args,
                modifiers,
                span,
            } => self.visit_function_call_mut(name, args, modifiers, *span),
            AstNode::JavaScript { code, span } => self.visit_javascript_mut(code, *span),
            AstNode::Escaped { content, span } => self.visit_escaped_mut(content, *span),
        }
    }

    fn visit_program_mut(&mut self, body: &mut [AstNode], span: Span) {
        let _ = span;
        for node in body {
            self.visit_mut(node);
        }
    }

    fn visit_text_mut(&mut self, content: &mut String, span: Span) {
        let _ = (content, span);
    }

    fn visit_function_call_mut(
        &mut self,
        name: &mut String,
        args: &mut Option<Vec<Argument>>,
        modifiers: &mut Modifiers,
        span: Span,
    ) {
        let _ = (name, modifiers, span);
        if let Some(args) = args {
            for arg in args {
                for part in &mut arg.parts {
                    self.visit_mut(part);
                }
            }
        }
    }

    fn visit_javascript_mut(&mut self, code: &mut String, span: Span) {
        let _ = (code, span);
    }

    fn visit_escaped_mut(&mut self, content: &mut String, span: Span) {
        let _ = (content, span);
    }
}
