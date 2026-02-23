use forge_kit::parser::{AstNode, parse};
use forge_kit::utils::{calculate_stats, contains_javascript, count_nodes};

#[cfg(test)]
mod tests {
    use super::{AstNode, calculate_stats, contains_javascript, count_nodes, parse};
    use forge_kit::visitor::{AstVisitor, FunctionCollector, NodeCounter};

    #[test]
    fn test_simple_text() {
        let (ast, errors) = parse("Hello, world!");
        assert!(errors.is_empty());

        match ast {
            AstNode::Program { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    AstNode::Text { content, .. } => {
                        assert_eq!(content, "Hello, world!");
                    }
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected program node"),
        }
    }

    #[test]
    fn test_simple_function() {
        let (ast, errors) = parse("code: `$userName`");
        assert!(errors.is_empty());

        match ast {
            AstNode::Program { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    AstNode::FunctionCall { name, args, .. } => {
                        assert_eq!(name, "userName");
                        assert!(args.is_none());
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected program node"),
        }
    }

    #[test]
    fn test_function_with_args() {
        let (ast, errors) = parse("code: `$get[coins]`");
        assert!(errors.is_empty());

        match ast {
            AstNode::Program { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    AstNode::FunctionCall { name, args, .. } => {
                        assert_eq!(name, "get");
                        assert!(args.is_some());
                        let args = args.as_ref().unwrap();
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected program node"),
        }
    }

    #[test]
    fn test_nested_functions() {
        let (ast, errors) = parse("code: `$get[$getUserVar[coins;$authorID]]`");
        assert!(errors.is_empty());
        assert_eq!(count_nodes(&ast), 5);
    }

    #[test]
    fn test_modifiers() {
        let (ast, errors) = parse("code: `$!silent[]`");
        assert!(errors.is_empty());

        match ast {
            AstNode::Program { body, .. } => match &body[0] {
                AstNode::FunctionCall { modifiers, .. } => {
                    assert!(modifiers.silent);
                    assert!(!modifiers.negated);
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected program node"),
        }
    }

    #[test]
    fn test_javascript() {
        let (ast, errors) = parse("code: `Result: ${ 1 + 1 }`");
        assert!(errors.is_empty());
        assert!(contains_javascript(&ast));
    }

    /// \` escapes a backtick — the backtick is emitted as literal text and does
    /// NOT close the surrounding context.
    #[test]
    fn test_escape_backtick() {
        // Outside a code block: \` should be a literal backtick in plain text.
        let (ast, errors) = parse("before \\` after");
        assert!(errors.is_empty());
        if let AstNode::Program { body, .. } = ast {
            // Depending on how text nodes are merged, there may be 1–3 text nodes;
            // the important thing is that the backtick appears in the output and
            // no errors are raised.
            let text: String = body
                .iter()
                .filter_map(|n| {
                    if let AstNode::Text { content, .. } = n {
                        Some(content.clone())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(text.contains('`'), "Expected a literal backtick in output");
        }
    }

    /// \` inside a code block does NOT terminate the block.
    #[test]
    fn test_escape_backtick_in_code_block() {
        // The backtick after \\ is escaped; the second backtick closes the block.
        let (ast, errors) = parse("code: `hello\\`world`");
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        if let AstNode::Program { body, .. } = ast {
            let texts: Vec<&str> = body
                .iter()
                .filter_map(|n| {
                    if let AstNode::Text { content, .. } = n {
                        Some(content.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let combined = texts.join("");
            assert!(
                combined.contains('`'),
                "Escaped backtick should appear as literal content"
            );
        }
    }

    #[test]
    fn test_escaped_function() {
        let (ast, errors) = parse("code: `$c[escaped content]`");
        assert!(errors.is_empty());

        match ast {
            AstNode::Program { body, .. } => match &body[0] {
                AstNode::Escaped { content, .. } => {
                    assert_eq!(content, "escaped content");
                }
                _ => panic!("Expected escaped node"),
            },
            _ => panic!("Expected program node"),
        }
    }

    #[test]
    fn test_stats() {
        let code = "code: `$if[$authorID==$ownerID]$get[role]$endif`";
        let (_ast, _) = parse(code);

        let stats = calculate_stats(&_ast);
        assert!(stats.function_calls >= 3);
        assert!(stats.total_nodes > stats.function_calls);
    }

    #[test]
    fn test_visitor_pattern() {
        let (ast, errors) = parse("code: `$userName $get[role]`");
        assert!(errors.is_empty());

        let mut collector = FunctionCollector::new();
        collector.visit(&ast);
        assert_eq!(collector.functions, vec!["userName", "get"]);
    }

    #[test]
    fn test_node_counter() {
        let (ast, errors) = parse("code: `$userName $get[role]`");
        assert!(errors.is_empty());

        let mut counter = NodeCounter::default();
        counter.visit(&ast);
        assert_eq!(counter.function_nodes, 2);
        assert_eq!(counter.text_nodes, 2);
    }

    /// `\\$` suppresses the dollar sign — it becomes literal `$` text rather than
    /// beginning a function call.
    #[test]
    fn test_escaped_dollar_is_literal() {
        // Rust `"\\\\$userName"` inside a code block = chars `\\$userName`.
        // `\\$` (3 source bytes) → literal `$`.
        // `userName` is then plain text (no leading `$`).
        let (ast, errors) = parse("code: `\\\\$userName`");
        assert!(errors.is_empty(), "Errors: {:?}", errors);

        // The block contains `\\$userName`:
        //   \\  → Text("\\")   (first, via the `\\` → `\` rule)
        // Wait — `\\$` is 3-byte rule that wins. Let's be precise.
        // escape_sequence_len sees `\`, then `\`, then `$` → returns 3 → emits `$`.
        // So `\\$userName` → Text("$") + Text("userName") = effectively "$userName" text.
        //
        // But `\\$userName` Rust string is actually two chars `\\` then `$userName`.
        // We need `\\` then `$` then `userName` which is Rust `"\\\\$userName"`.
        // That IS our input. inner = `\\$userName` (2 backslashes, then dollar, then name).
        // escape_sequence_len at 0: bytes[0]=`\`, bytes[1]=`\`, bytes[2]=`$` → len=3, emit `$`.
        // Remaining: `userName` → plain text.
        // So: Text("$") + Text("userName") or merged into Text("$userName").
        // Either way NO FunctionCall node.

        if let AstNode::Program { body, .. } = ast {
            let has_function_call = body
                .iter()
                .any(|n| matches!(n, AstNode::FunctionCall { .. }));
            assert!(
                !has_function_call,
                "\\\\$ should suppress the function call — expected no FunctionCall node"
            );
            let full_text: String = body
                .iter()
                .filter_map(|n| {
                    if let AstNode::Text { content, .. } = n {
                        Some(content.clone())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(
                full_text.contains('$'),
                "Escaped dollar should appear as literal `$` in output"
            );
        }
    }

    /// Single `\` followed by `$func` — the backslash is emitted as-is (lone
    /// backslash rule) and `$func` is parsed as a normal function call.
    #[test]
    fn test_lone_backslash_then_function() {
        // Rust `"\\$func"` inside the block = `\$func` (one backslash, then `$func`).
        // escape_sequence_len at 0: bytes[0]=`\`, bytes[1]=`$` → not `\` → len=1.
        // Emits `\` as text, then `$func` is a function call.
        let (ast, errors) = parse("code: `\\$func`");
        assert!(errors.is_empty(), "Errors: {:?}", errors);

        if let AstNode::Program { body, .. } = ast {
            assert_eq!(body.len(), 2, "Expected a text node and a function call");
            assert!(matches!(body[0], AstNode::Text { .. }));
            if let AstNode::Text { content, .. } = &body[0] {
                assert_eq!(content, "\\");
            }
            assert!(matches!(body[1], AstNode::FunctionCall { .. }));
        }
    }

    /// `\\]` inside function arguments produces a literal `]` without closing
    /// the argument list.
    #[test]
    fn test_escaped_closing_bracket_in_args() {
        // Source: $func[hello\\]world]
        // `\\]` → literal `]`, then `world` is still inside the arg, then `]` closes.
        let (ast, errors) = parse("code: `$func[hello\\\\]world]`");
        assert!(errors.is_empty(), "Errors: {:?}", errors);

        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall { args, .. } = &body[0] {
                let args = args.as_ref().expect("Expected arguments");
                assert_eq!(args.len(), 1, "Should be one argument");
                let text = args[0].as_text().unwrap_or_default();
                assert!(
                    text.contains(']'),
                    "Escaped ] should appear as literal `]` in the argument"
                );
            } else {
                panic!("Expected FunctionCall");
            }
        }
    }

    #[test]
    fn test_semicolons_in_nested_args() {
        let (ast, errors) = parse("code: `$parent[$inner[a;b];outer_second]`");
        assert!(errors.is_empty());

        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall { args, .. } = &body[0] {
                let args = args.as_ref().unwrap();
                assert_eq!(args.len(), 2);
            }
        }
    }

    #[test]
    fn test_empty_and_whitespace_args() {
        let (ast, _) = parse("code: `$func[; ;last]`");
        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall { args, .. } = &body[0] {
                let args = args.as_ref().unwrap();
                assert_eq!(args.len(), 3);
            }
        }
    }

    #[test]
    fn test_modifier_complex_count() {
        let (ast, errors) = parse("code: `$@[ 100 ]funcName`");
        assert!(errors.is_empty());

        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall {
                name, modifiers, ..
            } = &body[0]
            {
                assert_eq!(name, "funcName");
                assert_eq!(modifiers.count.as_deref(), Some(" 100 "));
            }
        }
    }

    #[test]
    fn test_unicode_safety() {
        let input = "Stars: ⭐ and 宝箱";
        let (ast, errors) = parse(input);
        assert!(errors.is_empty());

        match ast {
            AstNode::Program { body, span } => {
                assert_eq!(span.end, input.len());
                if let AstNode::Text { content, .. } = &body[0] {
                    assert!(content.contains("宝箱"));
                }
            }
            _ => panic!("Expected program"),
        }
    }

    #[test]
    fn test_deep_nesting_stack() {
        let mut deep = String::from("code: `$a[");
        for _ in 0..50 {
            deep.push_str("$a[");
        }
        for _ in 0..51 {
            deep.push(']');
        }
        deep.push('`');

        let (_, errors) = parse(&deep);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mixed_text_and_code() {
        let (ast, errors) = parse("Prefix code: `$foo` Suffix");
        assert!(errors.is_empty());

        if let AstNode::Program { body, .. } = ast {
            assert_eq!(body.len(), 3);
            assert!(matches!(body[0], AstNode::Text { .. }));
            assert!(matches!(body[1], AstNode::FunctionCall { .. }));
            assert!(matches!(body[2], AstNode::Text { .. }));
        }
    }

    #[test]
    fn test_code_literal_in_text() {
        let (ast, errors) = parse("code: not a block");
        assert!(errors.is_empty());
        if let AstNode::Program { body, .. } = ast {
            assert_eq!(body.len(), 1);
            if let AstNode::Text { content, .. } = &body[0] {
                assert_eq!(content, "code: not a block");
            }
        }
    }

    #[test]
    fn test_code_block_span_accuracy() {
        let input = "prefix code: `$func[]` suffix";
        let (ast, _) = parse(input);

        if let AstNode::Program { body, .. } = ast {
            let func_node = body
                .iter()
                .find(|n| matches!(n, AstNode::FunctionCall { .. }))
                .unwrap();
            let span = func_node.span();
            assert_eq!(&input[span.start..span.end], "$func[]");
        }
    }

    // =========================================================================
    // Bare-bracket tests (Fix #2)
    // =========================================================================

    /// A bare `[` inside a function argument (not attached to `$identifier`)
    /// must NOT be counted as an open bracket.  The argument list should parse
    /// successfully without any "Unclosed function arguments" error.
    #[test]
    fn test_bare_bracket_in_args_no_error() {
        // $ban[user; 1m; [some reason here]
        // The `[` before "some reason" is bare (no `$` before it), so it is
        // literal text and does not require a matching `]`.
        let (ast, errors) = parse("code: `$ban[user; 1m; []`");
        assert!(
            errors.is_empty(),
            "Bare `[` in args should not cause an error: {:?}",
            errors
        );

        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall { args, name, .. } = &body[0] {
                assert_eq!(name, "ban");
                let args = args.as_ref().expect("Expected arguments");
                assert_eq!(args.len(), 3, "Expected 3 arguments");
                // Third argument contains the literal `[`
                let third_text = args[2].as_text().unwrap_or_default();
                assert!(
                    third_text.contains('['),
                    "Third arg should contain bare `[`"
                );
            } else {
                panic!("Expected FunctionCall node");
            }
        }
    }

    /// The motivating example from the issue: an escaped `$get` inside an arg
    /// (the `]` after `hello` is NOT a function bracket) should not trigger
    /// "Unclosed function arguments".
    #[test]
    fn test_escaped_dollar_in_args_no_unclosed_error() {
        // $attachment[$get[hello] \\$get[hello\\];hello]
        //
        // Breakdown of the first argument:
        //   $get[hello]      — real nested call, brackets balanced.
        //   ` `              — space (literal text).
        //   \\$get[hello\\]  — \\$ escapes the dollar; `get[hello\\]` is plain text.
        //                      The `]` at the end of `hello\\]` is a bare `]` at
        //                      depth 0 inside the argument, which is just literal text.
        // Second argument: `hello`.
        let source = "code: `$attachment[$get[hello] \\\\$get[hello\\\\];hello]`";
        let (ast, errors) = parse(source);
        assert!(
            errors.is_empty(),
            "Escaped $ in args must not cause unclosed-bracket errors: {:?}",
            errors
        );

        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall { args, name, .. } = &body[0] {
                assert_eq!(name, "attachment");
                let args = args.as_ref().expect("Expected arguments");
                assert_eq!(args.len(), 2, "Expected 2 arguments");
            } else {
                panic!("Expected FunctionCall");
            }
        }
    }
    /// `\\;` inside function arguments produces a literal `;` without splitting
    /// into a new argument.
    ///
    /// Source: `$let[agent;Whee 1.0.1 rv:10102 (iPhone\\; iOS 16.2\\; en_US) Cronet]`
    ///
    /// The two `\\;` sequences each collapse to a literal `;`, so the entire
    /// second token is one argument — not three.
    #[test]
    fn test_escaped_semicolon_in_args() {
        // Rust string: "code: `$let[agent;Whee 1.0.1 rv:10102 (iPhone\\\\; iOS 16.2\\\\; en_US) Cronet]`"
        // Inside the code block: $let[agent;Whee 1.0.1 rv:10102 (iPhone\\; iOS 16.2\\; en_US) Cronet]
        // arg 0: "agent"
        // arg 1: "Whee 1.0.1 rv:10102 (iPhone; iOS 16.2; en_US) Cronet"  (two \\; → literal ;)
        let source =
            "code: `$let[agent;Whee 1.0.1 rv:10102 (iPhone\\\\; iOS 16.2\\\\; en_US) Cronet]`";
        let (ast, errors) = parse(source);
        assert!(
            errors.is_empty(),
            "\\\\; should not cause parse errors: {:?}",
            errors
        );

        if let AstNode::Program { body, .. } = ast {
            if let AstNode::FunctionCall { name, args, .. } = &body[0] {
                assert_eq!(name, "let");
                let args = args.as_ref().expect("Expected arguments");
                assert_eq!(
                    args.len(),
                    2,
                    "\\\\; must not split into a new argument — expected 2 args, got {}",
                    args.len()
                );

                // First arg is plain "agent"
                assert_eq!(args[0].as_text().as_deref(), Some("agent"));

                // Second arg contains the literal semicolons produced by \\;
                let second = args[1].as_text().unwrap_or_default();
                assert!(
                    second.contains(';'),
                    "Escaped semicolons should appear as literal `;` in the argument text"
                );
                assert!(
                    second.contains("iPhone"),
                    "Argument should preserve surrounding text"
                );
                assert!(
                    second.contains("en_US"),
                    "Argument should preserve text after second escaped semicolon"
                );
            } else {
                panic!("Expected FunctionCall node, got {:?}", body);
            }
        }
    }
}

#[cfg(feature = "validation")]
mod validation_tests {
    use forge_kit::metadata::{MetadataCache, MetadataManager};
    use forge_kit::parser::{AstNode, ErrorKind, Parser, ValidationConfig};
    use forge_kit::types::{Arg, Function};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_mock_metadata() -> Arc<MetadataManager> {
        let manager = MetadataManager::new();
        let valid_func = Function {
            name: "$validFunc".to_string(),
            args: Some(vec![
                Arg {
                    name: "arg1".to_string(),
                    required: Some(true),
                    ..Default::default()
                },
                Arg {
                    name: "arg2".to_string(),
                    required: Some(false),
                    ..Default::default()
                },
            ]),
            brackets: Some(true),
            ..Default::default()
        };

        let enum_func = Function {
            name: "$enumFunc".to_string(),
            args: Some(vec![Arg {
                name: "option".to_string(),
                required: Some(true),
                arg_enum: Some(vec!["yes".to_string(), "no".to_string()]),
                ..Default::default()
            }]),
            brackets: Some(true),
            ..Default::default()
        };

        let forbidden_brackets_func = Function {
            name: "$forbidden".to_string(),
            brackets: None,
            ..Default::default()
        };

        let cache = MetadataCache::new(
            vec![valid_func, enum_func, forbidden_brackets_func],
            HashMap::new(),
            vec![],
        );
        manager.import_cache(cache).unwrap();
        Arc::new(manager)
    }

    #[test]
    fn test_unclosed_bracket() {
        let (_ast, errors) =
            Parser::with_config("code: `$get[unclosed`", ValidationConfig::syntax_only()).parse();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_unclosed_js() {
        let (_ast, errors) =
            Parser::with_config("code: `${ unclosed`", ValidationConfig::syntax_only()).parse();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_unclosed_block() {
        let (ast, errors) =
            Parser::with_config("code: `$foo", ValidationConfig::syntax_only()).parse();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Unclosed code block");
        if let AstNode::Program { body, .. } = ast {
            if let AstNode::Text { content, .. } = &body[0] {
                assert_eq!(content, "code: `$foo");
            }
        }
    }

    /// A \` inside a code block should NOT trigger an "Unclosed code block" error
    /// because the backtick is escaped.
    #[test]
    fn test_escaped_backtick_does_not_close_block() {
        // code: `hello\`still_inside` — the \` is escaped, block closes at the final `.
        let (_, errors) = Parser::with_config(
            "code: `hello\\`still_inside`",
            ValidationConfig::syntax_only(),
        )
        .parse();
        assert!(
            errors.is_empty(),
            "Escaped backtick must not close the code block: {:?}",
            errors
        );
    }

    #[test]
    fn test_validation_argument_count() {
        let metadata = create_mock_metadata();
        let config = ValidationConfig {
            validate_arguments: true,
            validate_functions: true,
            ..Default::default()
        };
        let (_ast, errors) =
            Parser::with_validation("code: `$validFunc[]`", config.clone(), metadata.clone())
                .parse();
        assert_eq!(errors[0].kind, ErrorKind::ArgumentCount);
    }

    #[test]
    fn test_validation_enum_values() {
        let metadata = create_mock_metadata();
        let config = ValidationConfig {
            validate_enums: true,
            validate_functions: true,
            ..Default::default()
        };
        let (_ast, errors) =
            Parser::with_validation("code: `$enumFunc[maybe]`", config.clone(), metadata.clone())
                .parse();
        assert_eq!(errors[0].kind, ErrorKind::EnumValue);
    }

    #[test]
    fn test_validation_brackets() {
        let metadata = create_mock_metadata();
        let config = ValidationConfig {
            validate_brackets: true,
            validate_functions: true,
            ..Default::default()
        };
        let (_ast, errors) =
            Parser::with_validation("code: `$validFunc`", config.clone(), metadata.clone()).parse();
        assert_eq!(errors[0].kind, ErrorKind::BracketUsage);
    }

    #[test]
    fn test_validation_unknown_function() {
        let metadata = create_mock_metadata();
        let config = ValidationConfig {
            validate_functions: true,
            ..Default::default()
        };
        let (_ast, errors) =
            Parser::with_validation("code: `$unknown[]`", config.clone(), metadata.clone()).parse();
        assert_eq!(errors[0].kind, ErrorKind::UnknownFunction);
    }

    /// Bare brackets in args must not cause false positives in syntax-only mode.
    #[test]
    fn test_bare_brackets_no_syntax_error() {
        let (_, errors) =
            Parser::with_config("code: `$ban[user; 1m; []`", ValidationConfig::syntax_only())
                .parse();
        assert!(
            errors.is_empty(),
            "Bare `[` in args should not produce a syntax error: {:?}",
            errors
        );
    }
}
