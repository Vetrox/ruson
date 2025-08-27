use crate::nodes::node::{add_dependencies, add_usage_for_deps, get_node, remove_dependency, Node, NodeKind, SoNError};
use crate::services::lexer::Lexer;
use crate::typ::typ::Typ;
use crate::typ::typ::Typ::Bot;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Parser {
    lexer: Lexer,
    pub graph: Rc<RefCell<Vec<Option<Node>>>>,
    /// peephole optimization
    pub do_optimize: bool,
}

pub(crate) const KEEP_ALIVE_NID: usize = 0;
pub(crate) const CTRL_NID: usize = 1; // TODO: Introduce ScopeNode for this

impl Parser {
    pub fn new(input: &str) -> Result<Parser, SoNError> {
        let mut ctx = Parser { lexer: Lexer::from_str(input), graph: Rc::new(RefCell::new(vec![])), do_optimize: true };
        Node::new(ctx.graph.clone(), vec![], NodeKind::KeepAlive, Bot)?;
        let ctrl = Node::new(ctx.graph.clone(), vec![], NodeKind::Start, Bot)?;
        assert_eq!(CTRL_NID, ctrl);
        ctx.keep_node(ctrl)?; // TODO: Introduce ScopeNode for this
        Ok(ctx)
    }

    pub fn src(&self) -> String {
        self.lexer.input.clone()
    }

    /// a.k.a. garbage collect for the java stans
    fn drop_unused_nodes_cap(&mut self, cap: usize) {
        let len = self.graph.borrow().len();
        for nid in 0..len {
            self.attempt_drop_node(nid, cap);
        }
    }

    fn attempt_drop_node(&mut self, nid: usize, cap: usize) -> usize {
        if nid == KEEP_ALIVE_NID {
            return 0;
        }
        if cap <= 0 {
            return 0;
        }
        let inputs = self.graph.borrow_mut().get(nid).map(|n| match n.as_ref() {
            Some(node) if node.outputs.is_empty() => node.inputs.clone(),
            _ => vec![]
        });
        let mut c = cap;
        if let Some(inputs) = inputs {
            for neigh in inputs.into_iter() {
                if let Some(Some(n)) = self.graph.borrow_mut().get_mut(neigh) {
                    n.outputs.retain(|&k| k != nid);
                }
                c -= self.attempt_drop_node(neigh, c);
            }
        }
        if c > 0 {
            if matches!(self.graph.borrow_mut().get_mut(nid), Some(Some(n)) if n.outputs.is_empty()) {
                c -= 1;
                *self.graph.borrow_mut().get_mut(nid).unwrap() = None;
            };
        }
        cap - c
    }

    fn drop_unused_nodes(&mut self) {
        self.drop_unused_nodes_cap(100);
    }
    fn add_node(&mut self, inputs: Vec<usize>, node_kind: NodeKind, typ: Typ) -> Result<usize, SoNError> {
        let pr = format!("add_node inputs: {:?}, node_kind: {:?}, typ: {:?}", inputs, node_kind, typ);
        println!("{}", pr);
        for input in inputs.iter() {
            self.keep_node(*input)?;
        }
        self.drop_unused_nodes();
        for input in inputs.iter() {
            self.unkeep_node(*input)?;
        }
        let mut nid = Node::new(self.graph.clone(), inputs, node_kind, typ)?;
        if self.do_optimize {
            nid = self.peephole(nid)?;
            self.keep_node(nid)?;
            self.drop_unused_nodes();
            self.unkeep_node(nid)?;
        }
        Ok(nid)
    }

    /// Possibly creates a new node that this node needs to be replaced with.
    /// The caller can just use the returned nid instead of the input nid.
    fn peephole(&mut self,
                mut nid: usize) -> Result<usize, SoNError> {
        let node = get_node(self.graph.borrow().as_ref(), nid)?.clone();
        if node.typ().is_constant() && !matches!(node.node_kind, NodeKind::Constant) {
            assert!(node.outputs.is_empty()); // otherwise it won't get gc-collected
            nid = self.add_node(vec![], NodeKind::Constant, node.typ())?;
        }
        Ok(nid)
    }


    fn add_node_unrefined(&mut self, inputs: Vec<usize>, node_kind: NodeKind) -> Result<usize, SoNError> {
        self.add_node(inputs, node_kind, Bot)
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
        let ret = self.add_node_unrefined(vec![CTRL_NID, primary], NodeKind::Return);
        ret
    }

    fn with_kept_node<F, R>(&mut self, node: usize, f: F) -> Result<R, SoNError>
    where
        F: FnOnce(&mut Self) -> Result<R, SoNError>,
    {
        self.keep_node(node)?;
        let result = f(self);
        self.unkeep_node(node)?;
        result
    }

    fn keep_node(&mut self, nid: usize) -> Result<(), SoNError> {
        add_usage_for_deps(self.graph.clone(), KEEP_ALIVE_NID, &vec![nid])?;
        add_dependencies(self.graph.clone(), KEEP_ALIVE_NID, &vec![nid])
    }

    fn unkeep_node(&mut self, nid: usize) -> Result<(), SoNError> {
        remove_dependency(self.graph.clone(), KEEP_ALIVE_NID, nid)
    }

    fn parse_expression(&mut self) -> Result<usize, SoNError> {
        self.parse_addition()
    }

    /// <pre>
    /// additiveExpr : multiplicativeExpr (('+' | '-') additiveExpr)*
    /// </pre>
    fn parse_addition(&mut self) -> Result<usize, SoNError> {
        let lhs = self.parse_multiplication()?;
        if self.lexer.matsch("+") {
            return self.with_kept_node(lhs, |parser| {
                let rhs = parser.parse_addition()?;
                parser.add_node_unrefined(vec![lhs, rhs], NodeKind::Add)
            });
        }
        if self.lexer.matsch("-") {
            return self.with_kept_node(lhs, |parser| {
                let rhs = parser.parse_addition()?;
                parser.add_node_unrefined(vec![lhs, rhs], NodeKind::Sub)
            });
        }
        Ok(lhs)
    }


    /// <pre>
    /// multiplicativeExpr : unaryExpr (('*' | '/') multiplicativeExpr)*
    /// </pre>
    fn parse_multiplication(&mut self) -> Result<usize, SoNError> {
        let lhs = self.parse_unary()?;
        if self.lexer.matsch("*") {
            return self.with_kept_node(lhs, |parser| {
                let rhs = parser.parse_multiplication()?;
                parser.add_node_unrefined(vec![lhs, rhs], NodeKind::Mul)
            });
        }
        if self.lexer.matsch("/") {
            return self.with_kept_node(lhs, |parser| {
                let rhs = parser.parse_multiplication()?;
                parser.add_node_unrefined(vec![lhs, rhs], NodeKind::Div)
            });
        }
        Ok(lhs)
    }

    /// <pre>
    /// unaryExpr : ('-') unaryExpr | primaryExpr
    /// </pre>
    fn parse_unary(&mut self) -> Result<usize, SoNError> {
        if self.lexer.matsch("-") {
            let unary = self.parse_unary()?;
            self.add_node_unrefined(vec![unary], NodeKind::Minus)
        } else {
            self.parse_primary()
        }
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
        self.add_node(vec![], NodeKind::Constant, Typ::Int { constant: value })
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
    use crate::nodes::node::{node_exists_unique, NodeKind, SoNError};
    use crate::services::parser::{Parser, CTRL_NID, KEEP_ALIVE_NID};
    use crate::typ::typ::Typ;

    #[test]
    fn should_be_able_to_create_new_parser() {
        // Arrange & Act
        let parser = Parser::new("return 1;").unwrap();

        // Assert
        assert_eq!(2, parser.graph.borrow().len());
        assert!(matches!( parser.graph.borrow_mut().get(CTRL_NID).unwrap().as_ref().unwrap().node_kind, NodeKind::Start))
    }

    #[test]
    fn should_parse_return() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let g = parser.graph.borrow_mut();
        let node = g.get(result).unwrap().as_ref().unwrap();
        assert!(matches!(node.node_kind, NodeKind::Return));
        assert!(matches!(node.outputs.as_slice(), []));
        match node.inputs.as_slice() {
            [CTRL_NID, x] => {
                let dnode = g.get(*x).unwrap().as_ref().unwrap();
                assert!(matches!(dnode.typ(), Typ::Int { constant: 1 }));
                assert!(matches!(dnode.outputs.as_slice(), [y] if y.eq(&result) ));
            }
            _ => assert!(false)
        }
        let my_node = node.clone();
        drop(g);
        println!("Parsing result is: {}", my_node);
    }

    #[test]
    fn should_drop_unused_nodes_but_never_the_keepalive_node() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let _result = parser.parse().unwrap();
        parser.drop_unused_nodes();

        // Assert
        assert_eq!(2, parser.graph.borrow().iter().filter(|n| n.is_some()).count());
        assert!(matches!( parser.graph.borrow_mut().get(KEEP_ALIVE_NID).unwrap().as_ref().unwrap().node_kind, NodeKind::KeepAlive))
    }

    #[test]
    fn should_not_drop_any_node_when_cap_is_0() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let _result = parser.parse().unwrap();
        parser.drop_unused_nodes_cap(0);

        // Assert
        assert_eq!(4, parser.graph.borrow().iter().filter(|n| n.is_some()).count());
    }

    #[test]
    fn should_only_drop_one_node_when_cap_is_1() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let _result = parser.parse().unwrap();
        parser.drop_unused_nodes_cap(1);

        // Assert
        assert_eq!(3, parser.graph.borrow().iter().filter(|n| n.is_some()).count());
        assert!(matches!( parser.graph.borrow_mut().get(CTRL_NID).unwrap().as_ref().unwrap().node_kind, NodeKind::Start))
    }

    #[test]
    fn should_fail_when_invalid_syntax_is_used() {
        // Arrange
        let mut parser = Parser::new("ret 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse();

        // Assert
        assert!(matches!(result, Err(SoNError::SyntaxExpected {expected, ..}) if expected == "Statement"));
    }

    #[test]
    fn should_check_for_semicolon() {
        // Arrange
        let mut parser = Parser::new("return 1").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse();

        // Assert
        assert!(matches!(result, Err(SoNError::SyntaxExpected {expected, ..}) if expected == ";"));
    }

    #[test]
    fn should_fail_at_brace() {
        // Arrange
        let mut parser = Parser::new("return 1;}").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse();

        // Assert
        assert!(matches!(result, Err(SoNError::SyntaxExpected {expected, ..}) if expected == "End of file"));
    }

    #[test]
    fn should_delete_nodes_that_arent_kept_alive() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;
        let nid = parser.add_node_unrefined(vec![], NodeKind::Start).unwrap(); // this node is not kept

        // Act
        let _result = parser.parse();

        // Assert
        assert!(!node_exists_unique(&parser.graph.borrow(), nid, nid));
    }

    #[test]
    fn should_parse_one_plus_one() {
        // Arrange
        let mut parser = Parser::new("return 1+1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return (1+1);", format!("{:}", node));
    }

    #[test]
    fn should_parse_one_minus_one() {
        // Arrange
        let mut parser = Parser::new("return 1-1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return (1-1);", format!("{:}", node));
    }

    #[test]
    fn should_parse_one_times_one() {
        // Arrange
        let mut parser = Parser::new("return 1*1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return (1*1);", format!("{:}", node));
    }

    #[test]
    fn should_parse_one_div_one() {
        // Arrange
        let mut parser = Parser::new("return 1/1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return (1/1);", format!("{:}", node));
    }

    #[test]
    fn should_parse_mul_and_add() {
        // Arrange
        let mut parser = Parser::new("return 1*2+3;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return ((1*2)+3);", format!("{:}", node));
    }

    #[test]
    fn should_parse_mul_and_mul() {
        // Arrange
        let mut parser = Parser::new("return 1*2*3;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return (1*(2*3));", format!("{:}", node));
    }

    #[test]
    fn should_parse_complex_expression() {
        // Arrange
        let mut parser = Parser::new("return 1+2*3+-5;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return (1+((2*3)+(-5)));", format!("{:}", node));
    }

    #[test]
    fn should_peephole_computed_types() {
        // Arrange
        let mut parser = Parser::new("return 1+2*3+-5;").unwrap();

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.borrow_mut().get(result).unwrap().as_ref().unwrap().clone();
        assert_eq!("return 2;", format!("{:}", node));
    }
}
