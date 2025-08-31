extern crate core;
use crate::services::parser::Parser;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;
mod errors;


fn main() {
    let mut parser = Parser::new_noarg("return 0 + $ctrl;").unwrap();
    parser.do_optimize = true;
    let r = parser.parse();

    println!("{}", r.unwrap_err());

    fs::write("target/output.dot", parser.as_dotfile()).expect("Unable to write file");
}
