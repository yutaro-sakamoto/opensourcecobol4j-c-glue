use tree_sitter::{Language, Parser};

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .expect("Error loading C grammar");
    println!("Hello, world!");
}
