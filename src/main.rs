use rustop::opts;
use std::error;
use std::fmt;
use std::fs;
use tree_sitter::{Node, Parser, Query, QueryCursor};
use unwrap_or::*;
use yaml_rust::Yaml;
use yaml_rust::{YamlEmitter, YamlLoader};

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
) -> Option<Vec<CFunction>> {
    let tree = c_lang_parser.parse(&source_code, None)?;
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
    let return_type_index = query.capture_index_for_name("return_type")?;
    let declarator_index = query.capture_index_for_name("declarator")?;
    let parameters_index = query.capture_index_for_name("parameters")?;
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
                    let parameter_node = capture.node.child(index)?;
                    if parameter_node.kind() == "parameter_declaration" {
                        let parameter_type_node = parameter_node.child_by_field_name("type")?;
                        let parameter_var_node =
                            parameter_node.child_by_field_name("declarator")?;
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

    Some(c_functions)
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

#[derive(Clone, Debug)]
enum RunningMode {
    ParseC,
    GenerateJava,
}

#[derive(Debug, Clone)]
enum GlueError {
    InvalidCommandlineArguments,
    InvalidRunningMode(String),
    MissingRunningMode,
    MissingFilePath,
    UnableToReadFile(String),
    InvalidYamlFormat(String),
    InvalidCFormat(String),
    Other(String),
}

impl fmt::Display for GlueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GlueError::InvalidRunningMode(s) => write!(f, "Invalid running mode: {}", s),
            GlueError::MissingRunningMode => write!(f, "Missing running mode"),
            GlueError::MissingFilePath => write!(f, "Missing file path"),
            GlueError::UnableToReadFile(file_path) => {
                write!(f, "Unable to read file: {}", file_path)
            }
            GlueError::InvalidYamlFormat(file_path) => {
                write!(f, "Invalid yaml format: {}", file_path)
            }
            GlueError::InvalidCommandlineArguments => write!(f, "Invalid commandline arguments"),
            GlueError::InvalidCFormat(file_path) => write!(f, "Invalid C format: {}", file_path),
            GlueError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl error::Error for GlueError {}

fn main() -> Result<(), GlueError> {
    let (args, rest) = unwrap_ok_or! {opts! {
        synopsis "Generate glue code for C functions and opensource COBOL 4J";
        param mode:Option<String>, desc:"Specify running mode.";
    }.parse(),
    _,
    return Err(GlueError::InvalidCommandlineArguments)};

    let running_mode = match args.mode {
        Some(mode) => match mode.as_str() {
            "parse_c" => RunningMode::ParseC,
            "generate_java" => RunningMode::GenerateJava,
            _ => {
                return Err(GlueError::InvalidRunningMode(
                    "Invalid running mode".to_string(),
                ))
            }
        },
        None => return Err(GlueError::MissingRunningMode),
    };

    match running_mode {
        RunningMode::ParseC => {
            let mut c_lang_parser = Parser::new();

            unwrap_ok_or! {c_lang_parser
            .set_language(tree_sitter_c::language()),
            _,
            return Err(GlueError::Other("Error loading C grammar".to_string()))};

            let c_file_path = unwrap_some_or! {rest.get(0), return Err(GlueError::MissingFilePath)};
            let source_code = unwrap_ok_or! {fs::read_to_string(c_file_path), _, return Err(GlueError::UnableToReadFile(c_file_path.to_string()))};
            let c_functions = unwrap_some_or! {
                extract_function_declarators(&mut c_lang_parser, &source_code),
                return Err(GlueError::InvalidCFormat(c_file_path.to_string()))
            };
            println!("{}", c_info_source(&c_functions));
        }
        RunningMode::GenerateJava => {
            let yml_file_path =
                unwrap_some_or!(rest.get(0), return Err(GlueError::MissingFilePath));

            let yml_content = unwrap_ok_or!(
                fs::read_to_string(yml_file_path),
                _,
                return Err(GlueError::UnableToReadFile(yml_file_path.to_string()))
            );

            let yml_docs = unwrap_ok_or!(
                YamlLoader::load_from_str(&yml_content),
                _,
                return Err(GlueError::InvalidYamlFormat(yml_file_path.to_string()))
            );
        }
    }
    Ok(())
}
