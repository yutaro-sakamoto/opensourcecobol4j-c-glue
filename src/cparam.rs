use crate::java_type::PossibleJavaType;
use tree_sitter::Node;

#[derive(Clone, Debug)]
pub struct CParameter {
    pub var_name: String,
    pub type_name: String,
    pub pointer_depth: u32,
    pub type_size: u32,
    pub java_type: PossibleJavaType,
}

impl CParameter {
    pub fn new(var_name: &str, type_name: &str, pointer_depth: u32, type_size: u32) -> Self {
        let java_type = Self::convert_to_java_type(type_name);
        Self {
            var_name: var_name.to_string(),
            type_name: type_name.to_string(),
            pointer_depth,
            type_size,
            java_type,
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

    pub fn is_primitive_type(&self) -> bool {
        match self.type_name.as_str() {
            "int" => true,
            "unsigned int" => true,
            "char" => true,
            "unsigned char" => true,
            "short" => true,
            "unsigned short" => true,
            _ => false,
        }
    }

    fn convert_to_java_type(type_name: &str) -> PossibleJavaType {
        match type_name {
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
