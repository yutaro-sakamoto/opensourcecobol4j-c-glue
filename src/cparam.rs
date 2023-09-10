use crate::java_type::PossibleJavaType;
use tree_sitter::{Node, Parser, Query, QueryCursor};

#[derive(Clone, Debug)]
pub struct CParameter {
    pub var_name: String,
    pub type_name: String,
    pub pointer_depth: u32,
    pub type_size: u32,
}

impl CParameter {
    pub fn new() -> Self {
        Self {
            var_name: String::new(),
            type_name: String::new(),
            pointer_depth: 0,
            type_size: 0,
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

    pub fn convert_to_java_type(&self) -> PossibleJavaType {
        match &*self.type_name {
            "int" => PossibleJavaType::Int,
            "unsigned int" => PossibleJavaType::Int,
            "char" => PossibleJavaType::Byte,
            "unsigned char" => PossibleJavaType::Byte,
            "short" => PossibleJavaType::Short,
            "unsigned short" => PossibleJavaType::Short,
            _ => PossibleJavaType::ByteArray,
        }
    }
}
