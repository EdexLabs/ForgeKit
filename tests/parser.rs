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

    #[test]
    fn test_escape_sequence() {
        let (_ast, errors) = parse("\\`escaped\\`");
        assert!(errors.is_empty());
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

    #[test]
    fn test_recursive_escapes() {
        let (ast, errors) = parse("code: `\\\\$userName`");
        assert!(errors.is_empty());

        if let AstNode::Program { body, .. } = ast {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], AstNode::Text { .. }));
            assert!(matches!(body[1], AstNode::FunctionCall { .. }));
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
}
