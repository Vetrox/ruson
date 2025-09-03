extern crate core;
use crate::services::parser::Parser;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;
mod errors;


fn main() {
    let mut parser = Parser::new_noarg("return 1 ^ 1;").unwrap();
    parser.do_optimize = true;
    let r = parser.parse().unwrap();

    fs::write("target/output.dot", parser.as_dotfile()).expect("Unable to write file");
}
