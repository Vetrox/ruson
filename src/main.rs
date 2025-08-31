extern crate core;

use crate::services::parser::Parser;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;

fn main() {
    let mut parser = Parser::new_noarg("return 0 + arg;").unwrap();
    parser.do_optimize = true;
    parser.parse().unwrap();

    fs::write("target/output.dot", parser.as_dotfile()).expect("Unable to write file");
}
