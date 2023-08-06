use std::fs;
use tree_sitter::{Language, Parser, Query, QueryCursor};
#[derive(Clone, Debug)]
struct CFunction {
    pub return_type: String,
    pub name: String,
    pub parameter_types: Vec<String>,
}

impl CFunction {
    pub fn new() -> Self {
        Self {
            return_type: String::new(),
            name: String::new(),
            parameter_types: Vec::new(),
        }
    }
}

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading C grammar");
    let c_file_path = std::env::args().nth(1).expect("Missing C file path");
    let source_code = fs::read_to_string(c_file_path).unwrap();
    let tree = parser.parse(&source_code, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
    let query = Query::new(
        tree_sitter_c::language(),
        r#"(declaration
            type: (_) @return_type
            declarator: (function_declarator
                declarator: (_) @declarator
                parameters:
                    (parameter_list
                        (parameter_declaration
                            type: (_)
                        )*
                    ) @parameters
            )
        )"#,
    )
    .unwrap();
    let mut query_cursor = QueryCursor::new();
    let all_matches = query_cursor.matches(&query, tree.root_node(), source_code.as_bytes());
    let return_type_index = query.capture_index_for_name("return_type").unwrap();
    let declarator_index = query.capture_index_for_name("declarator").unwrap();
    let parameters_index = query.capture_index_for_name("parameters").unwrap();
    let c_function = &mut CFunction::new();
    let mut c_functions = Vec::new();
    let mut first_return_type = true;
    for each_match in all_matches {
        for capture in each_match.captures.iter().filter(|c| {
            c.index == return_type_index
                || c.index == declarator_index
                || c.index == parameters_index
        }) {
            let range = capture.node.range();
            let text = &source_code[range.start_byte..range.end_byte];
            let line = range.start_point.row;
            let node_type = capture.node.kind();
            let col = range.start_point.column;
            println!("[Line: {}, Col: {}] {}: `{}`", line, col, node_type, text);
            if capture.index == return_type_index {
                if !first_return_type {
                    c_functions.push(c_function.clone());
                }
                c_function.return_type = text.to_string();
                c_function.parameter_types.clear();
                first_return_type = false;
            } else if capture.index == declarator_index {
                c_function.name = text.to_string();
            } else if capture.index == parameters_index {
                c_function.parameter_types.push(text.to_string());
            }
        }
        if first_return_type {
            c_functions.push(c_function.clone());
        }
    }

    for each_function in c_functions.iter() {
        println!("Function: {}", each_function.name);
        println!("Return Type: {}", each_function.return_type);
        println!("Parameters: {:?}", each_function.parameter_types);
    }
}
