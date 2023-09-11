use rustop::opts;
use std::error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use tree_sitter::{Parser, Query, QueryCursor};
use unwrap_or::*;
use yaml_rust::Yaml;
use yaml_rust::{YamlEmitter, YamlLoader};

mod cfunc;
mod cparam;
mod java_type;

use cfunc::CFunction;
use cparam::CParameter;
use java_type::PossibleJavaType;

impl fmt::Display for PossibleJavaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PossibleJavaType::Byte => write!(f, "byte"),
            PossibleJavaType::Short => write!(f, "short"),
            PossibleJavaType::Int => write!(f, "int"),
            PossibleJavaType::ByteArray => write!(f, "byte[]"),
        }
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
                c_function.parameters.clear();
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
                            CParameter::get_pointer_depth_and_var_name(
                                source_code,
                                parameter_var_node,
                            );
                        if pointer_depth > 1 {
                            return None;
                        }
                        c_function.parameters.push(CParameter::new(
                            &parameter_var_text,
                            parameter_type_text,
                            pointer_depth,
                            0,
                        ));
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
        for each_parameter in each_function.parameters.iter() {
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
    GenerateC,
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
    UnableToWriteFile(String),
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
            GlueError::UnableToWriteFile(file_path) => {
                write!(f, "Unable to write file: {}", file_path)
            }
            GlueError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl error::Error for GlueError {}

/// Convert a yaml object to a vector of CFunctions
fn yml_to_c_function(yml: &Yaml) -> Option<Vec<CFunction>> {
    let mut c_functions = Vec::new();
    let root_hash = yml.as_hash()?.get(&Yaml::String("functions".to_string()))?;
    for yml_function in root_hash.as_vec()?.iter() {
        let mut c_function = CFunction::new();
        let hash1 = yml_function.as_hash()?;
        c_function.name = hash1
            .get(&Yaml::String("func_name".to_string()))?
            .as_str()?
            .to_string();
        c_function.return_type = hash1
            .get(&Yaml::String("return_type".to_string()))?
            .as_str()?
            .to_string();
        let yml_parameter_types = hash1
            .get(&Yaml::String("parameters".to_string()))?
            .as_vec()?;
        for yml_parameter in yml_parameter_types.iter() {
            let hash2 = yml_parameter.as_hash()?;
            let var_name = hash2
                .get(&Yaml::String("var_name".to_string()))?
                .as_str()?
                .to_string();
            let type_name = hash2
                .get(&Yaml::String("type_name".to_string()))?
                .as_str()?
                .to_string();
            let pointer_depth = hash2
                .get(&Yaml::String("pointer_depth".to_string()))?
                .as_i64()?
                .try_into()
                .ok()?;
            let type_size = hash2
                .get(&Yaml::String("type_size".to_string()))?
                .as_i64()?
                .try_into()
                .ok()?;
            c_function.parameters.push(CParameter::new(
                &var_name,
                &type_name,
                pointer_depth,
                type_size,
            ));
        }
        c_functions.push(c_function);
    }
    Some(c_functions)
}

fn get_java_file_content(c_function: &CFunction) -> String {
    let mut s = "".to_string();
    s += "import jp.osscons.opensourcecobol.libcobj.data.CobolDataStorage;\n";

    s += &format!(
        "public class {} extends CobolRunnableCGlue {{\n",
        c_function.name
    );

    s += &format!("  public native void {}(", c_function.name);
    let num_of_parameters = c_function.parameters.len();
    for (i, parameter_type) in c_function.parameters.iter().enumerate() {
        s += &format!("{} {}", parameter_type.java_type, parameter_type.var_name);
        if i != num_of_parameters - 1 {
            s += ", ";
        }
    }
    s += ");\n";

    s += &format!(
        "  static {{ System.loadLibrary(\"{}\"); }}\n",
        c_function.name
    );

    s += "  @Override\n";
    s += "  public int run(CobolDataStorage... argStorages) {\n";

    for (i, parameter_type) in c_function.parameters.iter().enumerate() {
        match parameter_type.java_type {
            PossibleJavaType::ByteArray => {
                s += &format!(
                    "    byte[] {} = storageToByteArray(argStorages[{}], {});\n",
                    parameter_type.var_name, i, parameter_type.type_size
                );
            }
            _ => {}
        }
    }
    s += &format!("    {}(", c_function.name);
    for (i, parameter_type) in c_function.parameters.iter().enumerate() {
        match parameter_type.java_type {
            PossibleJavaType::Byte => {
                s += &format!("storageToByte(argStorages[{}])", i);
            }
            PossibleJavaType::Short => {
                s += &format!("storageToShort(argStorages[{}])", i);
            }
            PossibleJavaType::Int => {
                s += &format!("storageToInt(argStorages[{}])", i);
            }
            PossibleJavaType::ByteArray => {
                s += &format!("{}", parameter_type.var_name);
            }
        };
        if i != num_of_parameters - 1 {
            s += ", ";
        }
    }
    s += ");\n";
    for (i, parameter_type) in c_function.parameters.iter().enumerate() {
        match parameter_type.java_type {
            PossibleJavaType::ByteArray => {
                s += &format!(
                    "    bytesToStorage(argStorages[{}], {});\n",
                    i, parameter_type.var_name
                );
            }
            _ => {}
        }
    }
    s += "    return 0;\n";
    s += "  }\n";
    s += "}\n";
    s
}

static C_LOCAL_PARAM_PREFIX: &'static str = "oc4j_glue_";

fn get_c_file_content(c_function: &CFunction) -> String {
    let mut s = "".to_string();
    s += &format!("#include \"{}.h\"\n", c_function.name);

    let num_of_params = c_function.parameters.len();

    s += &format!("extern {} {}(", c_function.return_type, c_function.name);
    for (index, param) in c_function.parameters.iter().enumerate() {
        s += &format!("{}", param.type_name);
        if param.pointer_depth > 0 {
            s += "*";
        }
        if index < num_of_params - 1 {
            s += ", ";
        }
    }
    s += ");\n";

    s += &format!(
        "JNIEXPORT void JNICALL Java_{}_{}\n",
        c_function.name, c_function.name
    );
    s += &format!("(JNIEnv *env , jobject object");

    for param in c_function.parameters.iter() {
        match param.java_type {
            PossibleJavaType::Byte => {
                s += &format!(", jbyte {}", param.var_name);
            }
            PossibleJavaType::Short => {
                s += &format!(", jshort {}", param.var_name);
            }
            PossibleJavaType::Int => {
                s += &format!(", jint {}", param.var_name);
            }
            PossibleJavaType::ByteArray => {
                s += &format!(", jbyteArray {}", param.var_name);
            }
        }
    }
    s += ")\n{\n";
    for param in c_function.parameters.iter() {
        match param.java_type {
            PossibleJavaType::Byte => {
                s += &format!(
                    "  char {}{} = {};\n",
                    C_LOCAL_PARAM_PREFIX, param.var_name, param.var_name
                );
            }
            PossibleJavaType::Short => {
                s += &format!(
                    "  short {}{} = {};\n",
                    C_LOCAL_PARAM_PREFIX, param.var_name, param.var_name
                );
            }
            PossibleJavaType::Int => {
                s += &format!(
                    "  int {}{} = {};\n",
                    C_LOCAL_PARAM_PREFIX, param.var_name, param.var_name
                );
            }
            PossibleJavaType::ByteArray => {
                s += &format!("// Generatoin code for Byte Array not implemented\n");
            }
        }
    }

    s += &format!("  {}(", c_function.name);
    for (index, param) in c_function.parameters.iter().enumerate() {
        match param.java_type {
            PossibleJavaType::Byte | PossibleJavaType::Short | PossibleJavaType::Int => {
                if param.pointer_depth == 1 {
                    s += &format!("&");
                }
                s += &format!("{}{}", C_LOCAL_PARAM_PREFIX, param.var_name);
            }
            _ => {}
        }
        if index < num_of_params - 1 {
            s += ", ";
        }
    }
    s += ");\n";
    s += "}\n";
    s
}

fn write_file(file: &mut File, content: String) -> Result<(), Box<std::io::Error>> {
    file.write_all(content.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn read_c_functions_from_yml(rest: &Vec<String>) -> Result<Vec<CFunction>, GlueError> {
    let yml_file_path = unwrap_some_or!(rest.get(0), return Err(GlueError::MissingFilePath));

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
    let c_functions = unwrap_some_or!(
        yml_to_c_function(&yml_docs[0]),
        return Err(GlueError::InvalidYamlFormat(yml_file_path.to_string()))
    );
    Ok(c_functions)
}

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
            "generate_c" => RunningMode::GenerateC,
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
            let c_functions = read_c_functions_from_yml(&rest)?;
            for c_function in c_functions.iter() {
                println!("func_name: {}", c_function.name);
                println!("return_type: {}", c_function.return_type);
                println!("parameters:");
                for c_parameter_type in c_function.parameters.iter() {
                    println!("  ---");
                    println!("  var_name: {}", c_parameter_type.var_name);
                    println!("  type_name: {}", c_parameter_type.type_name);
                    println!("  pointer_depth: {}", c_parameter_type.pointer_depth);
                    println!("  type_size: {}", c_parameter_type.type_size);
                }
                println!("==========");
            }

            for c_function in c_functions.iter() {
                let java_file_path = &format!("{}.java", c_function.name);
                let mut java_file = unwrap_ok_or! {
                    File::create(java_file_path),
                    _,
                    return Err(GlueError::UnableToWriteFile(java_file_path.to_string()))
                };
                let java_file_content = get_java_file_content(c_function);
                unwrap_ok_or! {
                    write_file(&mut java_file, java_file_content),
                    _,
                    return Err(GlueError::UnableToWriteFile(java_file_path.to_string()))
                };
            }
        }
        RunningMode::GenerateC => {
            let c_functions = read_c_functions_from_yml(&rest)?;

            for c_function in c_functions.iter() {
                let c_file_path = &format!("{}.c", c_function.name);
                let mut c_file = unwrap_ok_or! {
                    File::create(c_file_path),
                    _,
                    return Err(GlueError::UnableToWriteFile(c_file_path.to_string()))
                };
                let java_file_content = get_c_file_content(c_function);
                unwrap_ok_or! {
                    write_file(&mut c_file, java_file_content),
                    _,
                    return Err(GlueError::UnableToWriteFile(c_file_path.to_string()))
                };
            }
        }
    }
    Ok(())
}
