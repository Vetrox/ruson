extern crate core;

use crate::services::parser::Parser;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;

fn main() {
    let mut parser = Parser::new("int a=1;int b=2; #showGraph; return a;").unwrap();
    parser.do_optimize = false;
    parser.parse().unwrap();
    fs::write("target/output.dot", parser.as_dotfile()).expect("Unable to write file");
}
