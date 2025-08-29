extern crate core;

use crate::services::parser::Parser;
use services::dotvis::as_dotfile;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;

fn main() {
    let mut parser = Parser::new("int a=1;int b=2; #showGraph; return a;").unwrap();
    parser.do_optimize = false;
    parser.parse().unwrap();
    fs::write("target/output.dot", as_dotfile(&parser)).expect("Unable to write file");
}
