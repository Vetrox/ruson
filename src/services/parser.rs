pub struct ParseContext {}

pub fn parse() {
    println!("Hello World1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_run_parse() {
        parse();
    }
}
