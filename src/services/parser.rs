use crate::nodes::node::{Node, NodeKind, SoNError};
use crate::services::lexer::Lexer;
use std::cell::RefCell;
use std::rc::Rc;

struct Parser {
    lexer: Lexer,
    graph: Rc<RefCell<Vec<Option<Node>>>>,
}

const START_NODE: usize = 0;

impl Parser {
    pub fn new(input: &str) -> Result<Parser, SoNError> {
        let ctx = Parser { lexer: Lexer::from_str(input), graph: Rc::new(RefCell::new(vec![])) };
        Node::new(ctx.graph.clone(), vec![], NodeKind::Start)?;
        Ok(ctx)
    }

    fn add_node(&mut self, inputs: Vec<usize>, node_kind: NodeKind) -> Result<usize, SoNError> {
        Node::new(self.graph.clone(), inputs, node_kind)
    }

    pub fn parse(&mut self) -> Result<usize, SoNError> {
        let node = self.parse_statement()?;
        if !self.lexer.is_eof() {
            return Err(SoNError::SyntaxExpected { expected: "End of file".to_string(), actual: self.lexer.dbg_get_any_next_token() })
        }
        Ok(node)
    }

    fn parse_statement(&mut self) -> Result<usize, SoNError> {
        if self.lexer.matschx("return") {
            return self.parse_return();
        }
        Err(SoNError::SyntaxExpected { expected: "Statement".to_string(), actual: self.lexer.dbg_get_any_next_token() })
    }

    fn parse_return(&mut self) -> Result<usize, SoNError> {
        let primary = self.parse_expression()?;
        self.require(";")?;
        self.add_node(vec![START_NODE, primary], NodeKind::Return)
    }

    fn parse_expression(&mut self) -> Result<usize, SoNError> {
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<usize, SoNError> {
        self.lexer.skip_whitespace();
        if self.lexer.peek_is_number() {
            return self.parse_number_literal()
        }
        Err(SoNError::SyntaxExpected { expected: "Primary expression".to_string(), actual: self.lexer.dbg_get_any_next_token() })
    }

    fn parse_number_literal(&mut self) -> Result<usize, SoNError> {
        let value = self.lexer.parse_number()?;
        Ok(Node::new(self.graph.clone(), vec![START_NODE], NodeKind::Constant { value })?)
    }

    /// require this syntax
    fn require(&mut self, syntax: &str) -> Result<(), SoNError> {
        if self.lexer.matsch(syntax) {
            Ok(())
        } else {
            Err(SoNError::SyntaxExpected {
                expected: syntax.to_string(),
                actual: self.lexer.dbg_get_any_next_token(),
            })
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::nodes::node::NodeKind;
    use crate::services::parser::{Parser, START_NODE};

    #[test]
    fn should_be_able_to_create_new_parser() {
        // Arrange & Act
        let parser = Parser::new("return 1;").unwrap();

        // Assert
        assert_eq!(1, parser.graph.borrow().len());
        assert!(matches!( parser.graph.borrow_mut().get(START_NODE).unwrap().as_ref().unwrap().node_kind, NodeKind::Start))
    }

    #[test]
    fn should_parse_return() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let g = parser.graph.borrow_mut();
        let node = g.get(result).unwrap().as_ref().unwrap();
        assert!(matches!(node.node_kind, NodeKind::Return));
        assert!(matches!(node.outputs.as_slice(), []));
        match node.inputs.as_slice() {
            [START_NODE, x] => {
                let dnode = g.get(*x).unwrap().as_ref().unwrap();
                assert!(matches!(dnode.node_kind, NodeKind::Constant { value: 1 }));
                assert!(matches!(dnode.outputs.as_slice(), [y] if y.eq(&result) ));
            }
            _ => assert!(false)
        }
    }
}
