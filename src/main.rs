use std::fs;
use tree_sitter::{Node, Parser, Query, QueryCursor};
#[derive(Clone, Debug)]
struct CFunction {
    pub return_type: String,
    pub name: String,
    pub parameter_types: Vec<CParameterType>,
}

#[derive(Clone, Debug)]
struct CParameterType {
    pub var_name: String,
    pub type_name: String,
    pub pointer_depth: u32,
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

impl CParameterType {
    pub fn new() -> Self {
        Self {
            var_name: String::new(),
            type_name: String::new(),
            pointer_depth: 0,
        }
    }

    pub fn get_pointer_depth_and_var_name<'a>(
        source_code: &'a str,
        pointer_node: Node<'a>,
    ) -> (u32, String) {
        let mut pointer_depth = 0;
        let mut current_node = pointer_node;
        while let Some(child_node) = current_node.child_by_field_name("declarator") {
            pointer_depth += 1;
            current_node = child_node
        }
        (
            pointer_depth,
            source_code[current_node.range().start_byte..current_node.range().end_byte].to_string(),
        )
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
                            declarator: (_)
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
                        let parameter_type_node =
                            parameter_node.child_by_field_name("type").unwrap();
                        let parameter_var_node =
                            parameter_node.child_by_field_name("declarator").unwrap();
                        let parameter_type_text =
                            &source_code[parameter_type_node.range().start_byte
                                ..parameter_type_node.range().end_byte];
                        let (pointer_depth, parameter_var_text) =
                            CParameterType::get_pointer_depth_and_var_name(
                                source_code,
                                parameter_var_node,
                            );
                        c_function.parameter_types.push(CParameterType {
                            var_name: parameter_var_text,
                            type_name: parameter_type_text.to_string(),
                            pointer_depth: pointer_depth,
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
    let mut s = "#include <stdio.h>\n".to_string();
    s += "int main() {\n";
    s += "  printf(\"functions:\\n\");\n";
    for each_function in c_functions.iter() {
        s += &format!("  printf(\"  - func_name: {}\\n\");\n", each_function.name);
        s += &format!(
            "  printf(\"    return_type: {}\\n\");\n",
            each_function.return_type
        );
        s += &format!("  printf(\"    parameters:\\n\");\n");
        for each_parameter in each_function.parameter_types.iter() {
            s += &format!(
                "  printf(\"      - var_name: {}\\n\");\n",
                each_parameter.var_name
            );
            s += &format!(
                "  printf(\"        type_name: {}\\n\");\n",
                each_parameter.type_name
            );
            s += &format!(
                "  printf(\"        pointer_depth: {}\\n\");\n",
                each_parameter.pointer_depth
            );
            let mut type_name = each_parameter.type_name.to_string();
            for _ in 0..each_parameter.pointer_depth {
                type_name += &format!("*");
            }
            s += &format!(
                "  printf(\"        type_size: %lu\\n\", sizeof({}));\n",
                type_name
            );
        }
    }
    s += "  return 0;\n";
    s += "}\n";
    s
}

fn main() {
    let mut c_lang_parser = Parser::new();
    c_lang_parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading C grammar");
    let c_file_path = std::env::args().nth(1).expect("Missing C file path");
    let source_code = fs::read_to_string(c_file_path).unwrap();
    let c_functions = extract_function_declarators(&mut c_lang_parser, &source_code);

    println!("{}", c_info_source(&c_functions));
}
