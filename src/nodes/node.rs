use crate::errors::son_error::SoNError;
use crate::nodes::bound_node::BoundNode;
pub(crate) use crate::nodes::graph::Graph;
use crate::nodes::node::NodeKind::{Add, Comp, Constant, Div, KeepAlive, Minus, Mul, Proj, Return, Scope, Start, Sub};
use crate::typ::typ::Typ;
use std::collections::HashMap;
use NodeKind::Not;

#[derive(Debug)]
#[derive(Clone)]
pub struct Node {
    pub node_kind: NodeKind,
    /// ordered list of def`s this Node is depending on
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    /// unique id that is incremented with every new node
    pub uid: usize,
    pub nid: usize,
    typ: Typ,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Constant,
    Return,
    Start,
    KeepAlive,
    Add,
    Sub,
    Mul,
    Div,
    Minus,
    Scope { scopes: Vec<HashMap<String, usize>> },
    Proj { proj_index: usize, _dbg_proj_label: String },
    Comp { kind: CompNodeKind },
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompNodeKind {
    LT,
    LEQ,
    EQ,
    LogAnd,
    LogOr,
    LogXor,
}

impl NodeKind {
    pub fn arity(&self) -> usize {
        match self {
            Start | KeepAlive | Scope { .. } | Constant => 0,
            Minus | Proj { .. } | Not => 1,
            Return | Add | Sub | Mul | Div | Comp { .. } => 2,
        }
    }
}

impl Node {
    pub fn new(node_kind: NodeKind, uid: usize, nid: usize, typ: Typ) -> Node {
        Node { node_kind, inputs: vec![], outputs: vec![], uid, nid, typ }
    }

    pub fn typ(&self) -> Typ {
        self.typ.clone()
    }

    /// refines the typ of this node. Typ always moves upwards from BOT to TOP as we optimize.
    pub fn refine_typ(&mut self, typ: Typ) -> Result<(), SoNError> {
        if !self.typ.transition_allowed(&typ) {
            return Err(SoNError::TypTransitionNotAllowed);
        }
        println!("Node {:?} ({:?}) node_kind: {:?}, typ: {:?} -> {:?}", self.nid, self.uid, self.node_kind, self.typ, typ);
        self.typ = typ;
        Ok(())
    }

    pub fn bind<'a>(&'a self, graph: &'a Graph) -> BoundNode<'a> {
        BoundNode::new(&self, &graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_construct_constant_node() {
        // Arrange
        let mut graph = Graph::new();

        // Act
        let nid1 = graph.new_node(vec![], NodeKind::Constant, Typ::Int { constant: 42 }).unwrap();

        // Assert
        assert!(matches!(graph.get(nid1).unwrap().as_ref().unwrap().typ, Typ::Int { constant: 42 }));
    }

    #[test]
    fn should_construct_constant_node_in_empty_slot() {
        // Arrange
        let mut graph = Graph::from(vec![None]);

        // Act
        let nid1 = graph.new_node(vec![], Constant, Typ::Bot).unwrap();

        // Assert
        assert_eq!(1, graph.len());
        assert!(matches!(graph.get(nid1).unwrap().as_ref().unwrap().node_kind, Constant));
    }

    #[test]
    fn should_be_able_to_contain_same_dependency_multiple_times() {
        // Arrange
        let mut graph = Graph::from(vec![None]);
        let nid1 = graph.new_node(vec![], Constant, Typ::Bot).unwrap();

        // Act
        let nid2 = graph.new_node(vec![nid1, nid1], Add, Typ::Bot).unwrap();

        // Assert
        let graph_br = graph;
        assert_eq!(2, graph_br.get_node(nid2).unwrap().inputs.len());
        assert_eq!(2, graph_br.get_node(nid1).unwrap().outputs.len());
    }

    #[test]
    fn should_remove_dependency_from_the_back() {
        // Arrange
        let mut graph = Graph::from(vec![None]);
        let nid1 = graph.new_node(vec![], NodeKind::Constant, Typ::Bot).unwrap();
        let nid2 = graph.new_node(vec![nid1], NodeKind::Minus, Typ::Bot).unwrap();
        let nid3 = graph.new_node(vec![nid1], NodeKind::Minus, Typ::Bot).unwrap();
        let mut graph_br = graph;
        graph_br.add_reverse_dependencies_br(nid2, &vec![nid1]).unwrap();
        graph_br.add_dependencies_br(nid2, &vec![nid1]).unwrap();

        // Act
        graph_br.remove_dependency_br(nid2, nid1).unwrap();

        // Assert
        assert!(matches!(graph_br.get_node( nid2).unwrap().inputs.as_slice(), [i] if i == &nid1));
        assert!(matches!(graph_br.get_node( nid1).unwrap().outputs.as_slice(), [i, j] if i == &nid2 && j == &nid3));
    }
}
