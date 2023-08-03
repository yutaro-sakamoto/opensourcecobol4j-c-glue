use std::fs;
use tree_sitter::{Language, Parser};
fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading C grammar");
    let source_code = fs::read_to_string("tests/basic/basic.c").unwrap();
    let tree = parser.parse(&source_code, None).unwrap();
    let root_node = tree.root_node();
    println!("{}", root_node.to_sexp());
}
