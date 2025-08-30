extern crate core;

use crate::services::parser::Parser;
use std::fs;

pub mod nodes;
pub mod services;
pub mod typ;

fn main() {
    let mut parser = Parser::new_noarg("{ int x0=1; int y0=2; int x1=3; int y1=4; return (x0-x1)*(x0-x1) + (y0-y1)*(y0-y1); } #showGraph;").unwrap();
    parser.do_optimize = false;
    parser.parse().unwrap();

    fs::write("target/output.dot", parser.as_dotfile()).expect("Unable to write file");
}
