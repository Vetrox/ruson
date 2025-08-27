extern crate core;

use crate::services::parser::Parser;
use services::dotvis::as_dotfile;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;

fn main() {
    let mut parser = Parser::new("return 1+2*3+-5;").unwrap();
    parser.do_optimize = false;
    parser.parse().unwrap();

    let dotfile = as_dotfile(&parser);

    fs::write("target/output.dot", dotfile).expect("Unable to write file");
}
