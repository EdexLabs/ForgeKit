//! High-performance AST parser for ForgeScript with optional validation
//!
//! This module provides a fast, single-pass parser that builds a proper Abstract Syntax Tree
//! with extensive optimizations for speed and memory efficiency, plus optional validation.

use smallvec::SmallVec;

// Optional validation support
#[cfg(feature = "validation")]
use crate::metadata::MetadataManager;
#[cfg(feature = "validation")]
use crate::types::{Arg, Function};
#[cfg(feature = "validation")]
use std::sync::Arc;

// ============================================================================
// Utility: Escape Detection
// ============================================================================

/// Determines if a character at a given byte index is escaped by backslashes.
/// This checks for an odd number of preceding backslashes.
#[inline]
pub fn is_escaped(code: &str, byte_idx: usize) -> bool {
    if byte_idx == 0 || !code.is_char_boundary(byte_idx) {
        return false;
    }
    let bytes = code.as_bytes();
    let mut count = 0;
    let mut i = byte_idx;
    while i > 0 && bytes[i - 1] == b'\\' {
        count += 1;
        i -= 1;
    }
    count % 2 != 0
}

// ============================================================================
// Validation Configuration
// ============================================================================

/// Configuration for parser validation
#[derive(Debug, Clone, Default)]
pub struct ValidationConfig {
    /// Validate argument counts against function metadata
    pub validate_arguments: bool,
    /// Validate enum values against defined enums
    pub validate_enums: bool,
    /// Validate that all functions exist in metadata
    pub validate_functions: bool,
    /// Validate bracket usage (required/optional/forbidden)
    pub validate_brackets: bool,
}

impl ValidationConfig {
    /// Enable all validations
    pub fn strict() -> Self {
        Self {
            validate_arguments: true,
            validate_enums: true,
            validate_functions: true,
            validate_brackets: true,
        }
    }

    /// Enable only syntax validations (no metadata required)
    pub fn syntax_only() -> Self {
        Self {
            validate_arguments: false,
            validate_enums: false,
            validate_functions: false,
            validate_brackets: true,
        }
    }

    /// Check if any validation is enabled
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.validate_arguments
            || self.validate_enums
            || self.validate_functions
            || self.validate_brackets
    }
}

// ============================================================================
// AST Node Definitions
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline(always)]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub fn offset(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[derive(Debug, Clone, Default)]
pub struct Modifiers {
    pub silent: bool,
    pub negated: bool,
    pub count: Option<String>,
    /// Span covering all modifier characters (e.g. `!#@[n]` before the name).
    /// `None` if no modifiers were present.
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub parts: SmallVec<[AstNode; 4]>,
    pub span: Span,
}

impl Argument {
    /// Check if argument is effectively empty (only whitespace/empty text nodes)
    pub fn is_empty(&self) -> bool {
        self.parts.iter().all(|part| match part {
            AstNode::Text { content, .. } => content.trim().is_empty(),
            _ => false,
        })
    }

    /// Get literal text value if argument is purely text
    pub fn as_text(&self) -> Option<String> {
        if self.parts.len() == 1 {
            if let AstNode::Text { content, .. } = &self.parts[0] {
                return Some(content.clone());
            }
        }

        // Try to concatenate if all parts are text
        if self.parts.iter().all(|p| matches!(p, AstNode::Text { .. })) {
            let mut result = String::new();
            for part in &self.parts {
                if let AstNode::Text { content, .. } = part {
                    result.push_str(&content);
                }
            }
            return Some(result);
        }

        None
    }
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Program {
        body: Vec<AstNode>,
        span: Span,
    },
    Text {
        content: String,
        span: Span,
    },
    FunctionCall {
        name: String,
        /// Span of the function name identifier including any modifier characters (excludes `$`).
        name_span: Span,
        /// Span of the modifier characters between `$` and the name (e.g. `!#@[2]`).
        /// `None` when no modifiers are present.
        modifier_span: Option<Span>,
        /// Span of the argument list including the surrounding `[` and `]`.
        /// `None` when the function was called without brackets.
        args_span: Option<Span>,
        args: Option<Vec<Argument>>,
        modifiers: Modifiers,
        /// Full span from the start of modifiers to the closing `]` (or end of name when no args).
        /// This is the function call without the leading `$`.
        full_span: Span,
        /// Full span from `$` to the closing `]` (or end of name when no args).
        span: Span,
    },
    JavaScript {
        code: String,
        span: Span,
    },
    Escaped {
        content: String,
        span: Span,
    },
}

impl AstNode {
    pub fn span(&self) -> Span {
        match self {
            AstNode::Program { span, .. }
            | AstNode::Text { span, .. }
            | AstNode::FunctionCall { span, .. }
            | AstNode::JavaScript { span, .. }
            | AstNode::Escaped { span, .. } => *span,
        }
    }

    pub fn offset_spans(&mut self, offset: usize) {
        match self {
            AstNode::Program { body, span } => {
                span.offset(offset);
                for node in body {
                    node.offset_spans(offset);
                }
            }
            AstNode::Text { span, .. }
            | AstNode::JavaScript { span, .. }
            | AstNode::Escaped { span, .. } => {
                span.offset(offset);
            }
            AstNode::FunctionCall {
                args,
                span,
                name_span,
                modifier_span,
                args_span,
                full_span,
                ..
            } => {
                span.offset(offset);
                name_span.offset(offset);
                full_span.offset(offset);
                if let Some(ms) = modifier_span {
                    ms.offset(offset);
                }
                if let Some(as_) = args_span {
                    as_.offset(offset);
                }
                if let Some(args) = args {
                    for arg in args {
                        arg.span.offset(offset);
                        for part in &mut arg.parts {
                            part.offset_spans(offset);
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Parse Errors
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    ArgumentCount,
    EnumValue,
    UnknownFunction,
    BracketUsage,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub kind: ErrorKind,
}

impl ParseError {
    #[inline]
    pub fn new(message: impl Into<String>, span: Span, kind: ErrorKind) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }

    #[inline]
    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self::new(message, span, ErrorKind::Syntax)
    }
}

// ============================================================================
// Parser
// ============================================================================

pub struct Parser<'src> {
    source: &'src str,
    bytes: &'src [u8],
    pos: usize,
    errors: Vec<ParseError>,
    config: ValidationConfig,
    #[cfg(feature = "validation")]
    metadata: Option<Arc<MetadataManager>>,
}

impl<'src> Parser<'src> {
    #[inline]
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            errors: Vec::new(),
            config: ValidationConfig::default(),
            #[cfg(feature = "validation")]
            metadata: None,
        }
    }

    /// Create parser with validation configuration (requires "validation" feature)
    #[cfg(feature = "validation")]
    #[inline]
    pub fn with_config(source: &'src str, config: ValidationConfig) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            errors: Vec::new(),
            config,
            metadata: None,
        }
    }

    /// Create parser with validation and metadata (requires "validation" feature)
    #[cfg(feature = "validation")]
    #[inline]
    pub fn with_validation(
        source: &'src str,
        config: ValidationConfig,
        metadata: Arc<MetadataManager>,
    ) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            errors: Vec::new(),
            config,
            metadata: Some(metadata),
        }
    }

    pub fn parse(mut self) -> (AstNode, Vec<ParseError>) {
        let start = self.pos;
        let mut body = Vec::new();

        while !self.is_eof() {
            // Find start of "code: `" block
            if let Some(block_start) = self.find_code_block_start() {
                // Add text before block
                if block_start > self.pos {
                    body.push(AstNode::Text {
                        content: self.slice(self.pos, block_start).to_string(),
                        span: Span::new(self.pos, block_start),
                    });
                }

                // Move pos to start of content (after "code: `")
                let content_start = block_start + 7; // len("code: `")
                self.pos = content_start;

                // Find end of block (unescaped `)
                if let Some(block_end) = self.find_code_block_end() {
                    let content_len = block_end - content_start;

                    if content_len > 0 {
                        // Parse content inside block
                        let inner_source = self.slice(content_start, block_end);

                        #[cfg(feature = "validation")]
                        let inner_parser = if self.config.is_enabled() {
                            if let Some(ref metadata) = self.metadata {
                                Parser::with_validation(
                                    inner_source,
                                    self.config.clone(),
                                    metadata.clone(),
                                )
                            } else {
                                Parser::with_config(inner_source, self.config.clone())
                            }
                        } else {
                            Parser::new(inner_source)
                        };

                        #[cfg(not(feature = "validation"))]
                        let inner_parser = Parser::new(inner_source);

                        let (mut inner_ast, inner_errors) = inner_parser.parse_forge_script();

                        inner_ast.offset_spans(content_start);

                        match inner_ast {
                            AstNode::Program {
                                body: inner_body, ..
                            } => {
                                body.extend(inner_body);
                            }
                            _ => body.push(inner_ast),
                        }

                        for mut error in inner_errors {
                            error.span.offset(content_start);
                            self.errors.push(error);
                        }
                    }

                    // Move past closing backtick
                    self.pos = block_end + 1;
                } else {
                    // Unclosed block
                    if self.config.validate_brackets {
                        self.errors.push(ParseError::syntax(
                            "Unclosed code block",
                            Span::new(block_start, self.source.len()),
                        ));
                    }
                    body.push(AstNode::Text {
                        content: self.slice(block_start, self.source.len()).to_string(),
                        span: Span::new(block_start, self.source.len()),
                    });
                    self.pos = self.source.len();
                }
            } else {
                // No more blocks, rest is text
                if self.pos < self.source.len() {
                    body.push(AstNode::Text {
                        content: self.slice(self.pos, self.source.len()).to_string(),
                        span: Span::new(self.pos, self.source.len()),
                    });
                }
                self.pos = self.source.len();
            }
        }

        let span = Span::new(start, self.source.len());
        (AstNode::Program { body, span }, self.errors)
    }

    fn parse_forge_script(mut self) -> (AstNode, Vec<ParseError>) {
        let start = self.pos;
        let mut body = Vec::new();

        while !self.is_eof() {
            if let Some(node) = self.parse_forge_node() {
                body.push(node);
            }
        }

        let span = Span::new(start, self.source.len());
        (AstNode::Program { body, span }, self.errors)
    }

    // ========================================================================
    // Character/Position Utilities
    // ========================================================================

    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    #[inline(always)]
    fn current_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    #[inline(always)]
    fn peek_byte(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    #[inline(always)]
    fn advance(&mut self) -> Option<u8> {
        let byte = self.current_byte()?;
        self.pos += 1;
        Some(byte)
    }

    #[inline]
    fn slice(&self, start: usize, end: usize) -> &'src str {
        &self.source[start..end.min(self.source.len())]
    }

    #[inline]
    fn is_escaped_at(&self, pos: usize) -> bool {
        is_escaped(self.source, pos)
    }

    fn find_code_block_start(&self) -> Option<usize> {
        let mut p = self.pos;
        while p + 7 <= self.bytes.len() {
            // Check for "code: `"
            if &self.bytes[p..p + 7] == b"code: `" {
                let preceded_by_valid = p == 0
                    || self.bytes[p - 1].is_ascii_whitespace()
                    || self.bytes[p - 1] == b'{'
                    || self.bytes[p - 1] == b',';
                if preceded_by_valid && !is_escaped(self.source, p + 6) {
                    return Some(p);
                }
            }
            p += 1;
        }
        None
    }

    fn find_code_block_end(&self) -> Option<usize> {
        let mut p = self.pos;
        while p < self.bytes.len() {
            if self.bytes[p] == b'`' && !is_escaped(self.source, p) {
                return Some(p);
            }
            p += 1;
        }
        None
    }

    // ========================================================================
    // High-Level Parsing
    // ========================================================================

    fn parse_forge_node(&mut self) -> Option<AstNode> {
        // Handle backslash escapes explicitly
        if self.current_byte() == Some(b'\\') {
            return self.parse_escape_sequence();
        }

        // Handle $ sequences
        if self.current_byte() == Some(b'$') && !self.is_escaped_at(self.pos) {
            if self.peek_byte(1) == Some(b'{') {
                return Some(self.parse_javascript());
            }
            return Some(self.parse_function_call());
        }

        self.parse_text()
    }

    fn parse_text(&mut self) -> Option<AstNode> {
        let start = self.pos;
        while !self.is_eof() {
            if self.current_byte() == Some(b'\\') {
                break;
            }
            if self.current_byte() == Some(b'$') && !self.is_escaped_at(self.pos) {
                break;
            }
            self.advance();
        }

        if self.pos > start {
            Some(AstNode::Text {
                content: self.slice(start, self.pos).to_string(),
                span: Span::new(start, self.pos),
            })
        } else {
            None
        }
    }

    fn parse_escape_sequence(&mut self) -> Option<AstNode> {
        let start = self.pos;
        self.advance(); // consume '\'

        if let Some(next) = self.current_byte() {
            // Escaped backslash: \\ -> single \
            if next == b'\\' {
                self.advance();
                return Some(AstNode::Text {
                    content: "\\".to_string(),
                    span: Span::new(start, self.pos),
                });
            }
            // Escaped special character
            if matches!(next, b'$' | b'[' | b']' | b';' | b'`') {
                self.advance();
                return Some(AstNode::Text {
                    content: self.slice(start + 1, self.pos).to_string(),
                    span: Span::new(start, self.pos),
                });
            }
        }

        // Lone backslash or unrecognised escape — emit as-is with no error
        Some(AstNode::Text {
            content: "\\".to_string(),
            span: Span::new(start, start + 1),
        })
    }

    fn parse_javascript(&mut self) -> AstNode {
        let start = self.pos;
        self.advance(); // '$'
        self.advance(); // '{'
        let brace_start = self.pos - 1;

        if let Some(end) = self.find_matching_brace(brace_start) {
            let code = self.slice(brace_start + 1, end).to_string();
            self.pos = end + 1;
            AstNode::JavaScript {
                code,
                span: Span::new(start, self.pos),
            }
        } else {
            if self.config.validate_brackets {
                self.errors.push(ParseError::syntax(
                    "Unclosed JavaScript expression",
                    Span::new(start, self.source.len()),
                ));
            }
            self.pos = self.source.len();
            AstNode::JavaScript {
                code: String::new(),
                span: Span::new(start, self.pos),
            }
        }
    }

    fn parse_function_call(&mut self) -> AstNode {
        let start = self.pos;
        self.advance(); // '$'

        // Record where modifiers start (right after '$')
        let modifier_start = self.pos;
        let modifiers = self.parse_modifiers();
        let modifier_end = self.pos;

        // modifier_span is Some only when modifier characters were actually consumed
        let modifier_span = if modifier_end > modifier_start {
            Some(Span::new(modifier_start, modifier_end))
        } else {
            None
        };

        // Record where the name begins and ends
        let name = self.parse_identifier();
        let name_end = self.pos;

        if name.is_empty() {
            return AstNode::Text {
                content: "$".to_string(),
                span: Span::new(start, start + 1),
            };
        }

        // name_span now includes modifiers but excludes '$'
        let name_span = Span::new(modifier_start, name_end);

        if self.is_escape_function(&name) {
            return self.parse_escape_function(start, name, name_span);
        }

        // Record bracket/args span
        let has_brackets = self.current_byte() == Some(b'[');
        let bracket_open = self.pos;

        let args = if has_brackets {
            self.parse_function_arguments()
        } else {
            None
        };

        let args_span = if has_brackets {
            // self.pos now points just past the closing ']'
            Some(Span::new(bracket_open, self.pos))
        } else {
            None
        };

        let full_span = Span::new(modifier_start, self.pos);
        let span = Span::new(start, self.pos);

        // Validate with metadata if available
        #[cfg(feature = "validation")]
        if self.config.is_enabled() {
            let full_name = if name.starts_with('$') {
                name.clone()
            } else {
                format!("${}", name)
            };

            if let Some(ref metadata) = self.metadata {
                if let Some(func) = metadata.get(&full_name) {
                    self.validate_function_call(
                        &full_name,
                        &func,
                        args.as_ref(),
                        has_brackets,
                        name_span,
                    );
                } else if self.config.validate_functions {
                    self.errors.push(ParseError::new(
                        format!("Unknown function: {}", full_name),
                        name_span,
                        ErrorKind::UnknownFunction,
                    ));
                }
            } else if self.config.validate_functions {
                self.errors.push(ParseError::new(
                    format!(
                        "Cannot validate function {}: no metadata available",
                        full_name
                    ),
                    name_span,
                    ErrorKind::UnknownFunction,
                ));
            }
        }

        AstNode::FunctionCall {
            name,
            name_span,
            modifier_span,
            args_span,
            args,
            modifiers,
            full_span,
            span,
        }
    }

    // ========================================================================
    // Validation
    // ========================================================================

    #[cfg(feature = "validation")]
    fn validate_function_call(
        &mut self,
        name: &str,
        func: &Function,
        args: Option<&Vec<Argument>>,
        has_brackets: bool,
        name_span: Span,
    ) {
        // Validate brackets usage
        if self.config.validate_brackets {
            match func.brackets {
                Some(true) => {
                    if !has_brackets {
                        self.errors.push(ParseError::new(
                            format!("{} requires brackets", name),
                            name_span,
                            ErrorKind::BracketUsage,
                        ));
                    }
                }
                Some(false) => {
                    // Brackets optional — no error either way
                }
                None => {
                    if has_brackets {
                        self.errors.push(ParseError::new(
                            format!("{} does not accept brackets", name),
                            name_span,
                            ErrorKind::BracketUsage,
                        ));
                    }
                }
            }
        }

        // Validate argument count and enums
        if (self.config.validate_arguments || self.config.validate_enums) && has_brackets {
            if let (Some(args), Some(func_args)) = (args, &func.args) {
                self.validate_arguments(name, args, func_args, name_span);
            }
        }
    }

    #[cfg(feature = "validation")]
    fn validate_arguments(
        &mut self,
        func_name: &str,
        provided_args: &[Argument],
        func_args: &[Arg],
        name_span: Span,
    ) {
        let provided_count = provided_args.len();

        let has_rest = func_args.iter().any(|a| a.rest);
        let required_count = func_args
            .iter()
            .filter(|a| a.required.unwrap_or(false) && !a.rest)
            .count();
        let max_count = if has_rest {
            usize::MAX
        } else {
            func_args.len()
        };

        if self.config.validate_arguments {
            if provided_count < required_count {
                self.errors.push(ParseError::new(
                    format!(
                        "{} requires at least {} argument(s), got {}",
                        func_name, required_count, provided_count
                    ),
                    name_span,
                    ErrorKind::ArgumentCount,
                ));
            } else if !has_rest && provided_count > max_count {
                self.errors.push(ParseError::new(
                    format!(
                        "{} accepts at most {} argument(s), got {}",
                        func_name, max_count, provided_count
                    ),
                    name_span,
                    ErrorKind::ArgumentCount,
                ));
            }
        }

        if self.config.validate_enums {
            for (i, provided_arg) in provided_args.iter().enumerate() {
                let func_arg = if i < func_args.len() {
                    &func_args[i]
                } else if has_rest {
                    func_args.last().unwrap()
                } else {
                    continue;
                };

                self.validate_enum_value(func_name, provided_arg, func_arg, name_span);
            }
        }
    }

    #[cfg(feature = "validation")]
    fn validate_enum_value(
        &mut self,
        func_name: &str,
        arg: &Argument,
        func_arg: &Arg,
        name_span: Span,
    ) {
        if !func_arg.required.unwrap_or(false) && arg.is_empty() {
            return;
        }

        let enum_values = if let Some(enum_name) = &func_arg.enum_name {
            if let Some(ref metadata) = self.metadata {
                metadata.get_enum(enum_name)
            } else {
                None
            }
        } else {
            func_arg.arg_enum.clone()
        };

        if let Some(valid_values) = enum_values {
            if let Some(text_value) = arg.as_text() {
                let trimmed = text_value.trim();
                if !trimmed.is_empty() && !valid_values.contains(&trimmed.to_string()) {
                    self.errors.push(ParseError::new(
                        format!(
                            "Invalid value for {} argument {}: expected one of {:?}",
                            func_name, func_arg.name, valid_values
                        ),
                        name_span,
                        ErrorKind::EnumValue,
                    ));
                }
            }
        }
    }

    // ========================================================================
    // Parsing Helpers
    // ========================================================================

    fn parse_modifiers(&mut self) -> Modifiers {
        let mut modifiers = Modifiers::default();
        let start = self.pos;

        loop {
            match self.current_byte() {
                Some(b'!') => {
                    modifiers.silent = true;
                    self.advance();
                }
                Some(b'#') => {
                    modifiers.negated = true;
                    self.advance();
                }
                Some(b'@') if self.peek_byte(1) == Some(b'[') => {
                    self.advance(); // '@'
                    let bracket_start = self.pos;
                    self.advance(); // '['
                    if let Some(end) = self.find_matching_bracket(bracket_start) {
                        modifiers.count = Some(self.slice(bracket_start + 1, end).to_string());
                        self.pos = end + 1;
                    } else if self.config.validate_brackets {
                        self.errors.push(ParseError::syntax(
                            "Unclosed modifier bracket",
                            Span::new(bracket_start, bracket_start + 1),
                        ));
                        break;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        let end = self.pos;
        if end > start {
            modifiers.span = Some(Span::new(start, end));
        }

        modifiers
    }

    #[inline]
    fn parse_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(b) = self.current_byte() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
        self.slice(start, self.pos).to_string()
    }

    fn is_escape_function(&self, name: &str) -> bool {
        matches!(name, "c" | "C" | "escape")
    }

    fn parse_escape_function(&mut self, start: usize, name: String, name_span: Span) -> AstNode {
        if self.current_byte() != Some(b'[') {
            if self.config.validate_brackets {
                self.errors.push(ParseError::new(
                    format!("${} requires brackets", name),
                    name_span,
                    ErrorKind::BracketUsage,
                ));
            }
            return AstNode::Text {
                content: self.slice(start, self.pos).to_string(),
                span: Span::new(start, self.pos),
            };
        }

        let bracket_start = self.pos;
        self.advance();
        if let Some(end) = self.find_matching_bracket(bracket_start) {
            let content = self.slice(bracket_start + 1, end).to_string();
            self.pos = end + 1;
            AstNode::Escaped {
                content,
                span: Span::new(start, self.pos),
            }
        } else {
            if self.config.validate_brackets {
                self.errors.push(ParseError::syntax(
                    format!("Unclosed '[' for ${}", name),
                    name_span,
                ));
            }
            self.pos = self.source.len();
            AstNode::Escaped {
                content: String::new(),
                span: Span::new(start, self.pos),
            }
        }
    }

    fn parse_function_arguments(&mut self) -> Option<Vec<Argument>> {
        let bracket_start = self.pos;
        self.advance();
        if let Some(end) = self.find_matching_bracket(bracket_start) {
            let args_content = self.slice(bracket_start + 1, end);
            let parsed_args = self.parse_arguments(args_content, bracket_start + 1);
            self.pos = end + 1;
            Some(parsed_args)
        } else {
            if self.config.validate_brackets {
                self.errors.push(ParseError::syntax(
                    "Unclosed function arguments",
                    Span::new(bracket_start, bracket_start + 1),
                ));
            }
            None
        }
    }

    fn parse_arguments(&mut self, content: &str, base_offset: usize) -> Vec<Argument> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let bytes = content.as_bytes();
        let mut i = 0;
        let mut arg_start = 0;

        while i < bytes.len() {
            if bytes[i] == b'$' && depth == 0 {
                if let Some(esc_end) = self.find_escape_function_end(content, i) {
                    current.push_str(&content[i..=esc_end]);
                    i = esc_end + 1;
                    continue;
                }
            }

            if bytes[i] == b'\\' {
                if let Some(next) = bytes.get(i + 1) {
                    if matches!(*next, b'`' | b'$' | b'[' | b']' | b';' | b'\\') {
                        current.push_str(&content[i..i + 2]);
                        i += 2;
                        continue;
                    }
                }
                current.push('\\');
                i += 1;
                continue;
            }

            match bytes[i] {
                b'[' if !is_escaped(content, i) && self.is_function_bracket(content, i) => {
                    depth += 1;
                    current.push('[');
                }
                b']' if depth > 0 && !is_escaped(content, i) => {
                    depth -= 1;
                    current.push(']');
                }
                b';' if depth == 0 => {
                    let arg_offset = base_offset + arg_start;
                    let parts = self.parse_argument_parts(&current, arg_offset);
                    args.push(Argument {
                        parts,
                        span: Span::new(arg_offset, arg_offset + current.len()),
                    });
                    current.clear();
                    arg_start = i + 1;
                }
                _ => current.push(bytes[i] as char),
            }
            i += 1;
        }

        if !current.is_empty() || !args.is_empty() {
            let arg_offset = base_offset + arg_start;
            let parts = self.parse_argument_parts(&current, arg_offset);
            args.push(Argument {
                parts,
                span: Span::new(arg_offset, arg_offset + current.len()),
            });
        }
        args
    }

    fn parse_argument_parts(&mut self, content: &str, offset: usize) -> SmallVec<[AstNode; 4]> {
        if content.is_empty() {
            let mut parts = SmallVec::new();
            parts.push(AstNode::Text {
                content: String::new(),
                span: Span::new(offset, offset),
            });
            return parts;
        }

        #[cfg(feature = "validation")]
        let inner_parser = if self.config.is_enabled() {
            if let Some(ref metadata) = self.metadata {
                Parser::with_validation(content, self.config.clone(), metadata.clone())
            } else {
                Parser::with_config(content, self.config.clone())
            }
        } else {
            Parser::new(content)
        };

        #[cfg(not(feature = "validation"))]
        let inner_parser = Parser::new(content);

        let (ast, errors) = inner_parser.parse_forge_script();

        let nodes = if let AstNode::Program { mut body, .. } = ast {
            for node in &mut body {
                node.offset_spans(offset);
            }
            body
        } else {
            vec![ast]
        };

        for mut error in errors {
            error.span.offset(offset);
            self.errors.push(error);
        }

        let mut parts = SmallVec::new();
        for node in nodes {
            parts.push(node);
        }
        parts
    }

    // ========================================================================
    // Matching Utilities
    // ========================================================================

    fn find_matching_bracket(&self, open_pos: usize) -> Option<usize> {
        let mut depth = 1;
        let mut p = open_pos + 1;
        while p < self.bytes.len() {
            if self.bytes[p] == b'\\' {
                p += 2;
                continue;
            }
            if self.bytes[p] == b'[' && !is_escaped(self.source, p) {
                depth += 1;
            } else if self.bytes[p] == b']' && !is_escaped(self.source, p) {
                depth -= 1;
                if depth == 0 {
                    return Some(p);
                }
            }
            p += 1;
        }
        None
    }

    fn find_matching_brace(&self, open_pos: usize) -> Option<usize> {
        let mut depth = 1;
        let mut p = open_pos + 1;
        while p < self.bytes.len() {
            match self.bytes[p] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(p);
                    }
                }
                _ => {}
            }
            p += 1;
        }
        None
    }

    fn is_function_bracket(&self, content: &str, idx: usize) -> bool {
        if idx == 0 || content.as_bytes().get(idx) != Some(&b'[') {
            return false;
        }
        let bytes = content.as_bytes();
        let mut i = idx;
        while i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
            i -= 1;
        }
        while i > 0 && matches!(bytes[i - 1], b'!' | b'#' | b']') {
            if bytes[i - 1] == b']' {
                let mut d = 1;
                while i > 1 && d > 0 {
                    i -= 1;
                    if bytes[i - 1] == b']' {
                        d += 1;
                    } else if bytes[i - 1] == b'[' {
                        d -= 1;
                    }
                }
                if i < 2 || bytes[i - 2] != b'@' {
                    return false;
                }
                i -= 2;
            } else {
                i -= 1;
            }
        }
        i > 0 && bytes[i - 1] == b'$' && (i == 1 || bytes[i - 2] != b'\\')
    }

    fn find_escape_function_end(&self, content: &str, start: usize) -> Option<usize> {
        let bytes = content.as_bytes();
        let mut p = start + 1;
        while p < bytes.len() && matches!(bytes[p], b'!' | b'#') {
            p += 1;
        }
        let name_start = p;
        while p < bytes.len() && (bytes[p].is_ascii_alphanumeric() || bytes[p] == b'_') {
            p += 1;
        }
        if !self.is_escape_function(&content[name_start..p]) || bytes.get(p) != Some(&b'[') {
            return None;
        }
        let mut depth = 1;
        p += 1;
        while p < bytes.len() {
            if bytes[p] == b'\\' {
                p += 2;
                continue;
            }
            if bytes[p] == b'[' && !is_escaped(content, p) {
                depth += 1;
            } else if bytes[p] == b']' && !is_escaped(content, p) {
                depth -= 1;
                if depth == 0 {
                    return Some(p);
                }
            }
            p += 1;
        }
        None
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Parse ForgeScript source code into an AST (no validation)
pub fn parse(source: &str) -> (AstNode, Vec<ParseError>) {
    Parser::new(source).parse()
}

/// Parse with error handling
pub fn parse_with_errors(source: &str) -> Result<AstNode, Vec<ParseError>> {
    let (ast, errors) = parse(source);
    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}

/// Parse with validation configuration (requires "validation" feature)
#[cfg(feature = "validation")]
pub fn parse_with_config(source: &str, config: ValidationConfig) -> (AstNode, Vec<ParseError>) {
    Parser::with_config(source, config).parse()
}

/// Parse with validation and metadata (requires "validation" feature)
#[cfg(feature = "validation")]
pub fn parse_with_validation(
    source: &str,
    config: ValidationConfig,
    metadata: Arc<MetadataManager>,
) -> (AstNode, Vec<ParseError>) {
    Parser::with_validation(source, config, metadata).parse()
}

/// Parse ForgeScript directly (no wrapper) with validation
#[cfg(feature = "validation")]
pub fn parse_forge_script_with_validation(
    source: &str,
    config: ValidationConfig,
    metadata: Arc<MetadataManager>,
) -> (AstNode, Vec<ParseError>) {
    Parser::with_validation(source, config, metadata).parse_forge_script()
}

/// Parse with strict validation (requires "validation" feature)
#[cfg(feature = "validation")]
pub fn parse_strict(source: &str, metadata: Arc<MetadataManager>) -> (AstNode, Vec<ParseError>) {
    Parser::with_validation(source, ValidationConfig::strict(), metadata).parse()
}
