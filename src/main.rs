use std::fs;
use tree_sitter::{Node, Parser, Query, QueryCursor};
#[derive(Clone, Debug)]
struct CFunction {
    pub return_type: String,
    pub name: String,
    pub parameter_types: Vec<CType>,
}

#[derive(Clone, Debug)]
struct CType {
    pub name: String,
    pub is_pointer: u32,
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

impl CType {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            is_pointer: 0,
        }
    }

    pub fn pointer_depth<'a>(pointer_node: Node<'a>) -> u32 {
        let mut pointer_depth = 0;
        let mut current_node = pointer_node;
        while let Some(child_node) = current_node.child_by_field_name("declarator") {
            pointer_depth += 1;
            current_node = child_node
        }
        pointer_depth
    }
}

fn extract_function_declarators<'a>(
    c_lang_parser: &mut Parser,
    source_code: &'a str,
) -> Vec<CFunction> {
    let tree = c_lang_parser.parse(&source_code, None).unwrap();
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
                for index in 0..capture.node.child_count() {
                    let parameter_node = capture.node.child(index).unwrap();
                    if parameter_node.kind() == "parameter_declaration" {
                        let parameter_type_text = &source_code
                            [parameter_node.range().start_byte..parameter_node.range().end_byte];
                        c_function.parameter_types.push(CType {
                            name: parameter_type_text.to_string(),
                            is_pointer: CType::pointer_depth(parameter_node),
                        });
                    }
                }
            }
        }
    }

    if !first_return_type {
        c_functions.push(c_function.clone());
    }
    c_functions
}

fn c_info_source(c_functions: &Vec<CFunction>) -> String {
    "int main() { return 0; }".to_string()
}

fn main() {
    let mut c_lang_parser = Parser::new();
    c_lang_parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading C grammar");
    let c_file_path = std::env::args().nth(1).expect("Missing C file path");
    let source_code = fs::read_to_string(c_file_path).unwrap();
    let c_functions = extract_function_declarators(&mut c_lang_parser, &source_code);

    //for each_function in c_functions.iter() {
    //    println!("Function: {}", each_function.name);
    //    println!("Return Type: {}", each_function.return_type);
    //    println!("Parameters: {:?}", each_function.parameter_types);
    //    println!("======");
    //}

    println!("{}", c_info_source(&c_functions));
}
