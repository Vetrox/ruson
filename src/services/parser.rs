use crate::node::node::Node;

pub fn parse() {
    println!("Hello World1");
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_run_parse() {
        parse();
        println!("I didn't panic!")
    }
}
