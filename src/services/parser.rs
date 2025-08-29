use crate::nodes::node::SoNError::{DebugPropagateControlFlowUpward, VariableUndefined};
use crate::nodes::node::{Graph, NodeKind, SoNError};
use crate::services::lexer::Lexer;
use crate::typ::typ::Typ;
use crate::typ::typ::Typ::Bot;
use once_cell::sync::Lazy;
use std::collections::hash_map::Values;
use std::collections::{HashMap, HashSet};
use SoNError::{SyntaxExpected, VariableRedefinition};

pub static KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    HashSet::from(["int".into(), "return".into()])
});

pub struct Parser {
    lexer: Lexer,
    pub graph: Graph,
    /// peephole optimization
    pub do_optimize: bool,
    pub _dbg_output: String,
}

pub(crate) const KEEP_ALIVE_NID: usize = 0;
pub(crate) const CTRL_NID: usize = 1; // TODO: Introduce ScopeNode for this
pub(crate) const SCOPE_NID: usize = 2;

impl Parser {
    pub fn new(input: &str) -> Result<Parser, SoNError> {
        let mut ctx = Parser { lexer: Lexer::from_string(format!("{{{}}}", input)), graph: Graph::new(), do_optimize: true, _dbg_output: "".into() };
        ctx.add_node_unrefined(vec![], NodeKind::KeepAlive)?;
        let ctrl = ctx.add_node_unrefined(vec![], NodeKind::Start)?;
        assert_eq!(CTRL_NID, ctrl);
        ctx.keep_node(ctrl)?; // TODO: Introduce ScopeNode for this
        let scope_nid = ctx.add_node_unrefined(vec![], NodeKind::Scope { scopes: vec![] })?;
        assert_eq!(SCOPE_NID, scope_nid);
        ctx.keep_node(scope_nid)?;

        Ok(ctx)
    }

    fn get_var(&mut self, name: &str) -> Result<Option<usize>, SoNError> {
        if let NodeKind::Scope { scopes } = &mut self.graph.get_node_mut(SCOPE_NID)?.node_kind {
            if let Some(scope) = scopes.last_mut() {
                return Ok(scope.get(name.into()).copied());
            }
            panic!("Tried to access scope, but none was there.")
        }
        panic!("Scope node was not scope kind.")
    }

    fn define_var(&mut self, name: &str, nid: usize) -> Result<(), SoNError> {
        self.graph.add_reverse_dependencies_br(SCOPE_NID, &vec![nid])?;
        self.graph.add_dependencies_br(SCOPE_NID, &vec![nid])?;

        if let NodeKind::Scope { scopes } = &mut self.graph.get_node_mut(SCOPE_NID)?.node_kind {
            if let Some(scope) = scopes.last_mut() {
                if scope.insert(name.into(), nid).is_some() {
                    panic!("Variable was already defined. Undefine it first.");
                }
                return Ok(());
            }
            panic!("Tried to access scope, but none was there.")
        }
        panic!("Scope node was not scope kind.")
    }

    fn undefine_var(&mut self, name: &str) -> Result<usize, SoNError> {
        if let NodeKind::Scope { scopes } = &mut self.graph.get_node_mut(SCOPE_NID)?.node_kind {
            if let Some(scope) = scopes.last_mut() {
                if let Some(nid) = scope.remove(name.into()) {
                    self.graph.remove_dependency_br(SCOPE_NID, nid)?;
                    return Ok(nid);
                }
                panic!("Tried to undefine not-defined var.")
            }
            panic!("Tried to access scope, but none was there.")
        }
        panic!("Scope node was not scope kind.")
    }

    pub fn src(&self) -> String {
        self.lexer.input.clone()
    }

    /// a.k.a. garbage collect for the java stans.
    /// Returns the number of deleted nodes
    fn drop_unused_nodes_cap(&mut self, mut cap: usize) -> usize {
        let original_cap = cap;
        let len = self.graph.len();
        for nid in 0..len {
            cap -= self.attempt_drop_node(nid, cap);
        }
        original_cap - cap
    }

    /// returns cap - number_of_deleted_nodes
    fn attempt_drop_node(&mut self, nid: usize, cap: usize) -> usize {
        if nid == KEEP_ALIVE_NID {
            return 0;
        }
        if cap <= 0 {
            return 0;
        }
        let inputs = self.graph.get(nid).map(|n| match n.as_ref() {
            Some(node) if node.outputs.is_empty() => node.inputs.clone(),
            _ => vec![]
        });
        let mut c = cap;
        if let Some(inputs) = inputs {
            for neigh in inputs.into_iter() {
                if let Some(Some(n)) = self.graph.get_mut(neigh) {
                    n.outputs.retain(|&k| k != nid);
                }
                c -= self.attempt_drop_node(neigh, c);
            }
        }
        if c > 0 {
            if matches!(self.graph.get_mut(nid), Some(Some(n)) if n.outputs.is_empty()) {
                c -= 1;
                *self.graph.get_mut(nid).unwrap() = None;
            };
        }
        cap - c
    }

    fn drop_unused_nodes(&mut self) -> usize {
        self.drop_unused_nodes_cap(100)
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
        let mut nid = self.graph.new_node(inputs, node_kind, typ)?;
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
    fn peephole(&mut self, mut nid: usize) -> Result<usize, SoNError> {
        let node = self.graph.get_node(nid)?;
        if node.typ().is_constant() && !matches!(node.node_kind, NodeKind::Constant) {
            assert!(node.outputs.is_empty()); // otherwise it won't get gc-collected
            nid = self.add_node(vec![], NodeKind::Constant, node.typ())?; // T_CONSTPROP
        }
        Ok(nid)
    }


    fn add_node_unrefined(&mut self, inputs: Vec<usize>, node_kind: NodeKind) -> Result<usize, SoNError> {
        self.add_node(inputs, node_kind, Bot)
    }

    fn push_scope(&mut self) -> Result<(), SoNError> {
        if let NodeKind::Scope { scopes } = &mut self.graph.get_node_mut(SCOPE_NID)?.node_kind {
            scopes.push(HashMap::new());
            return Ok(())
        }
        panic!("Scope node was not scope kind.")
    }

    fn pop_scope(&mut self) -> Result<(), SoNError> {
        let node = self.graph.get_node_mut(SCOPE_NID)?;
        if let NodeKind::Scope { scopes } = &mut node.node_kind {
            if let Some(scope) = scopes.pop() {
                let defined_nids: Values<String, usize> = scope.values();
                for &dep_nid in defined_nids {
                    self.graph.remove_dependency_br(SCOPE_NID, dep_nid)?;
                }
                return Ok(());
            }
            panic!("Tried to pop scope, but none was there.")
        }
        panic!("Scope node was not scope kind.")
    }

    pub fn parse(&mut self) -> Result<usize, SoNError> {
        let nid = self.parse_block()?;
        if !self.lexer.is_eof() {
            return Err(SyntaxExpected { expected: "End of file".to_string(), actual: self.lexer.dbg_get_any_next_token() })
        }
        self.keep_node(nid)?;
        while self.drop_unused_nodes() > 0 {
            println!("Dropping unused nodes...");
        }
        self.unkeep_node(nid)?;
        Ok(nid)
    }

    /// <pre>
    /// block: '{' statement+ '}'
    /// </pre>
    fn parse_block(&mut self) -> Result<usize, SoNError> {
        assert!(self.lexer.matsch("{"));
        self.push_scope()?;
        let mut node = self.parse_statement()?;
        while !self.lexer.is_eof() && !self.lexer.peek_matsch("}") {
            let new_node = self.parse_statement();
            if matches!(new_node, Err(DebugPropagateControlFlowUpward)) {
                continue;
            }
            node = new_node?;
        }
        self.require("}")?;
        self.pop_scope()?;
        Ok(node)
    }

    /// <pre>
    /// returnStatement: 'return' returnStatement ';'
    ///   declStatement: 'int' identifier '=' expression ';'
    ///  blockStatement: '{' statement+ '}'
    ///   exprStatement: identifier '=' expression ';'
    /// </pre>
    fn parse_statement(&mut self) -> Result<usize, SoNError> {
        if self.lexer.matsch("#showGraph;") {
            let out = format!("#showGraph@{}\n{}", self.lexer.dbg_position(), self.as_dotfile());
            self._dbg_output.push_str(&out.as_str());
            println!("{}", out);
            return Err(DebugPropagateControlFlowUpward)
        }
        if self.lexer.peek_matschx("return") {
            return self.parse_return_stmnt();
        }
        if self.lexer.peek_matschx("int") {
            return self.parse_decl_stmnt();
        }
        if self.lexer.peek_matsch("{") {
            return self.parse_block();
        }
        self.parse_expression_stmnt()
    }

    /// <pre>
    /// declStatement: 'int' identifier '=' expression ';'
    /// </pre>
    fn parse_decl_stmnt(&mut self) -> Result<usize, SoNError> {
        assert!(self.lexer.matschx("int"));
        let name = self.require_and_get_identifier()?;
        self.require("=")?;
        let expression = self.parse_expression()?;
        self.require(";")?;
        if let Some(_) = self.get_var(&name)? {
            return Err(VariableRedefinition { variable: name });
        }
        self.define_var(&name, expression)?;
        Ok(expression)
    }

    /// <pre>
    /// exprStatement: identifier '=' expression ';'
    /// </pre>
    fn parse_expression_stmnt(&mut self) -> Result<usize, SoNError> {
        let name = self.require_and_get_identifier()?;
        self.require("=")?;
        let expression = self.parse_expression()?;
        self.require(";")?;
        if let Some(nid) = self.get_var(&name)? {
            let nid1 = self.undefine_var(&name)?;
            assert_eq!(nid, nid1);
            self.define_var(&name, expression)?;
        } else {
            return Err(VariableUndefined { variable: name });
        }
        Ok(expression)
    }

    fn parse_return_stmnt(&mut self) -> Result<usize, SoNError> {
        assert!(self.lexer.matschx("return"));
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
        self.graph.add_reverse_dependencies_br(KEEP_ALIVE_NID, &vec![nid])?;
        self.graph.add_dependencies_br(KEEP_ALIVE_NID, &vec![nid])
    }

    fn unkeep_node(&mut self, nid: usize) -> Result<(), SoNError> {
        self.graph.remove_dependency_br(KEEP_ALIVE_NID, nid)
    }

    /// <pre>
    /// expression : additiveExpr
    /// </pre>
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

    /// <pre>
    /// primaryExpr : integerLiteral | identifier | '(' expression ')'
    /// </pre>
    fn parse_primary(&mut self) -> Result<usize, SoNError> {
        self.lexer.skip_whitespace();
        if self.lexer.peek_is_number() {
            return self.parse_number_literal()
        }
        if self.lexer.matsch("(") {
            let node = self.parse_expression()?;
            self.require(")")?;
            return Ok(node);
        }
        let name = self.require_and_get_identifier()?;
        if let Some(nid) = self.get_var(&name)? {
            Ok(nid)
        } else {
            Err(VariableUndefined { variable: name })
        }
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
            Err(SyntaxExpected {
                expected: syntax.to_string(),
                actual: self.lexer.dbg_get_any_next_token(),
            })
        }
    }

    fn require_and_get_identifier(&mut self) -> Result<String, SoNError> {
        self.lexer.skip_whitespace();
        if let Some(c) = self.lexer.peek() && Lexer::is_id_start(&c)
            && let name = self.lexer.parse_id()
            && !KEYWORDS.contains(&name) {
            Ok(name)
        } else {
            Err(SyntaxExpected { expected: "Identifier".to_string(), actual: self.lexer.dbg_get_any_next_token() })
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::nodes::bound_node::BoundNode;
    use crate::nodes::node::{NodeKind, SoNError};
    use crate::services::parser::{Parser, CTRL_NID, KEEP_ALIVE_NID, SCOPE_NID};
    use crate::typ::typ::Typ;

    #[test]
    fn should_be_able_to_create_new_parser() {
        // Arrange & Act
        let parser = Parser::new("return 1;").unwrap();

        // Assert
        assert_eq!(3, parser.graph.len());
        assert!(matches!( parser.graph.get(CTRL_NID).unwrap().as_ref().unwrap().node_kind, NodeKind::Start))
    }

    #[test]
    fn should_parse_return() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert!(matches!(node.node_kind, NodeKind::Return));
        assert!(matches!(node.outputs.as_slice(), []));
        match node.inputs.as_slice() {
            [CTRL_NID, x] => {
                let dnode = parser.graph.get(*x).unwrap().as_ref().unwrap();
                assert!(matches!(dnode.typ(), Typ::Int { constant: 1 }));
                assert!(matches!(dnode.outputs.as_slice(), [y] if y.eq(&result) ));
            }
            _ => assert!(false)
        }
        println!("Parsing result is: {}", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_drop_unused_nodes_but_never_the_keepalive_node() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let _result = parser.parse().unwrap();

        // Assert
        assert_eq!(5, parser.graph.iter().filter(|n| n.is_some()).count());
        assert!(matches!( parser.graph.get(KEEP_ALIVE_NID).unwrap().as_ref().unwrap().node_kind, NodeKind::KeepAlive))
    }

    #[test]
    fn should_not_drop_any_node_when_cap_is_0() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let _result = parser.parse().unwrap();
        let dropped_nodes = parser.drop_unused_nodes_cap(0);

        // Assert
        assert_eq!(0, dropped_nodes);
        assert_eq!(5, parser.graph.iter().filter(|n| n.is_some()).count());
    }

    #[test]
    fn should_only_drop_one_node_when_cap_is_1() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let _result = parser.parse().unwrap();
        let dropped_nodes = parser.drop_unused_nodes_cap(1);

        // Assert
        assert_eq!(1, dropped_nodes);
        assert_eq!(4, parser.graph.iter().filter(|n| n.is_some()).count());
        assert!(matches!( parser.graph.get(CTRL_NID).unwrap().as_ref().unwrap().node_kind, NodeKind::Start))
    }

    #[test]
    fn should_fail_when_invalid_syntax_is_used() {
        // Arrange
        let mut parser = Parser::new("ret 1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse();

        // Assert
        assert!(matches!(result, Err(SoNError::SyntaxExpected {expected, ..}) if expected == "="));
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
        assert!(!parser.graph.node_exists_unique(nid, nid));
    }

    #[test]
    fn should_parse_one_plus_one() {
        // Arrange
        let mut parser = Parser::new("return 1+1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return (1+1);", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_parse_one_minus_one() {
        // Arrange
        let mut parser = Parser::new("return 1-1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return (1-1);", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_parse_one_times_one() {
        // Arrange
        let mut parser = Parser::new("return 1*1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return (1*1);", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_parse_one_div_one() {
        // Arrange
        let mut parser = Parser::new("return 1/1;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return (1/1);", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_parse_mul_and_add() {
        // Arrange
        let mut parser = Parser::new("return 1*2+3;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return ((1*2)+3);", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_parse_mul_and_mul() {
        // Arrange
        let mut parser = Parser::new("return 1*2*3;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return (1*(2*3));", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_parse_complex_expression() {
        // Arrange
        let mut parser = Parser::new("return 1+2*3+-5;").unwrap();
        parser.do_optimize = false;

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get_node(result).unwrap();
        assert_eq!("return (1+((2*3)+(-5)));", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_peephole_computed_types() {
        // Arrange
        let mut parser = Parser::new("return 1+2*3+-5;").unwrap();

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get_node(result).unwrap();
        assert_eq!("return 2;", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }

    #[test]
    fn should_define_var() {
        // Arrange
        let mut parser = Parser::new("").unwrap();
        parser.push_scope().unwrap();
        let nid = parser.add_node_unrefined(vec![], NodeKind::Constant).unwrap();

        // Act
        parser.define_var("x", nid).unwrap();

        // Assert
        assert!(matches!(parser.graph.get_node(nid).unwrap().outputs.as_slice(), [a] if a == &SCOPE_NID));
        assert!(matches!(parser.graph.get_node(SCOPE_NID).unwrap().inputs.as_slice(), [a] if a == &nid));
        if let NodeKind::Scope { scopes } = &parser.graph.get_node(SCOPE_NID).unwrap().node_kind {
            if let [ map ] = scopes.as_slice() && let Some(a) = map.get("x") && a == &nid {
                return;
            }
        }
        panic!();
    }

    #[test]
    fn should_define_var_in_program() {
        // Arrange
        let mut parser = Parser::new("int a=1; return a;").unwrap();

        // Act
        let result = parser.parse().unwrap();

        // Assert
        let node = parser.graph.get(result).unwrap().as_ref().unwrap();
        assert_eq!("return 1;", format!("{:}", BoundNode::new(&node, &parser.graph)));
    }
}
