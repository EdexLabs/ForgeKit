use forge_kit::parser::{AstNode, parse};
use forge_kit::visitor::{AstVisitor, AstVisitorMut, FunctionCollector, NodeCounter};

#[test]
fn test_function_collector_basic() {
    let (ast, _) = parse("code: `$get[foo] $set[bar]`");
    let mut collector = FunctionCollector::new();
    collector.visit(&ast);
    assert_eq!(collector.functions, vec!["get", "set"]);
}

#[test]
fn test_function_collector_nested() {
    let (ast, _) = parse("code: `$outer[$inner[val];$second]`");
    let mut collector = FunctionCollector::new();
    collector.visit(&ast);
    // Visitor should visit in pre-order: outer, then arguments (inner), then inner arg (val), then second arg
    assert_eq!(collector.functions, vec!["outer", "inner", "second"]);
}

#[test]
fn test_node_counter_all_types() {
    let input = "code: `text $func[] ${ 1+1 } $c[esc]`";
    let (ast, _) = parse(input);
    let mut counter = NodeCounter::default();
    counter.visit(&ast);

    // Structure:
    // Text("text ")
    // FunctionCall("func")
    // Text(" ")
    // JavaScript(" 1+1 ")
    // Text(" ")
    // Escaped("esc")

    assert_eq!(counter.text_nodes, 3);
    assert_eq!(counter.function_nodes, 1);
    assert_eq!(counter.javascript_nodes, 1);
    assert_eq!(counter.escaped_nodes, 1);
}

// Custom mutating visitor to uppercase all text content
struct UppercaseVisitor;

impl AstVisitorMut for UppercaseVisitor {
    fn visit_text_mut(&mut self, content: &mut String, _span: forge_kit::parser::Span) {
        *content = content.to_uppercase();
    }
}

#[test]
fn test_mutating_visitor() {
    let (mut ast, _) = parse("code: `hello $func[world]`");
    let mut visitor = UppercaseVisitor;
    visitor.visit_mut(&mut ast);

    if let AstNode::Program { body, .. } = ast {
        // Text("HELLO ")
        if let AstNode::Text { content, .. } = &body[0] {
            assert_eq!(content, "HELLO ");
        } else {
            panic!("Expected text");
        }

        // FunctionCall("func", args: [Text("WORLD")])
        if let AstNode::FunctionCall { args, .. } = &body[1] {
            let args = args.as_ref().unwrap();
            let arg_content = &args[0].parts[0];
            if let AstNode::Text { content, .. } = arg_content {
                assert_eq!(content, "WORLD");
            } else {
                panic!("Expected text in arg");
            }
        }
    }
}

// Visitor to track traversal order
struct OrderVisitor {
    log: Vec<String>,
}

impl AstVisitor for OrderVisitor {
    fn visit_function_call(
        &mut self,
        name: &str,
        args: Option<&Vec<forge_kit::parser::Argument>>,
        _modifiers: &forge_kit::parser::Modifiers,
        _span: forge_kit::parser::Span,
    ) {
        self.log.push(format!("enter:{}", name));
        if let Some(args) = args {
            for arg in args {
                self.visit_argument(arg);
            }
        }
        self.log.push(format!("exit:{}", name));
    }

    fn visit_text(&mut self, content: &str, _span: forge_kit::parser::Span) {
        self.log.push(format!("text:{}", content.trim()));
    }
}

#[test]
fn test_traversal_order() {
    let (ast, _) = parse("code: `$a[$b[]]`");
    let mut visitor = OrderVisitor { log: Vec::new() };
    visitor.visit(&ast);

    // Expected: enter:a -> enter:b -> exit:b -> exit:a
    assert_eq!(visitor.log, vec!["enter:a", "enter:b", "exit:b", "exit:a"]);
}
