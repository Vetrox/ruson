extern crate core;

use crate::services::parser::Parser;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;

fn main() {
    let mut parser = Parser::new("{ int x0=arg; int y0=2; int x1=3; int y1=4; return (x0-x1)*(x0-x1) + (y0-y1)*(y0-y1); } #showGraph; ", 0).unwrap();
    parser.do_optimize = false;
    parser.parse().unwrap();

    fs::write("target/output.dot", parser.as_dotfile()).expect("Unable to write file");
}
