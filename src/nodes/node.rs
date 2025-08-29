pub(crate) use crate::nodes::graph::Graph;
use crate::typ::typ::Typ;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{cell::RefCell, rc::Rc};

static GLOBAL_NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
}

#[derive(Debug)]
pub enum SoNError {
    NodeIdNotExisting,
    NumberCannotStartWith0,
    SyntaxExpected { expected: String, actual: String },
    TypTransitionNotAllowed,
    VariableRedefinition { variable: String },
    VariableUndefined { variable: String },
}

impl Node {
    pub fn new(
        graph: Rc<RefCell<Graph>>,
        inputs: Vec<usize>,
        node_kind: NodeKind,
        typ: Typ,
    ) -> Result<usize, SoNError> {
        let mut graph_br = graph.borrow_mut();
        let index = graph_br.find_first_empty_cell();
        let node = Node { node_kind, inputs: vec![], outputs: vec![], uid: GLOBAL_NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst), nid: index, typ };
        let inputs_c = inputs.clone();
        graph_br.add_reverse_dependencies_br(index, &inputs_c)?;
        if index == graph_br.len() {
            graph_br.push(None);
        }
        graph_br[index] = Some(node.clone());
        graph_br.add_dependencies_br(index, &inputs_c)?;

        // refine the node typ immediately. This sets the refined typ but doesn't optimize anything.
        let n = graph_br.get_node(index)?;
        let typ = graph_br.compute_refined_typ(n)?;
        graph_br.get_node_mut(index)?.refine_typ(typ)?;

        Ok(index)
    }

    pub fn typ(&self) -> Typ {
        self.typ.clone()
    }

    /// refines the typ of this node. Typ always moves upwards from BOT to TOP as we optimize.
    pub fn refine_typ(&mut self, typ: Typ) -> Result<(), SoNError> {
        if !self.typ.transition_allowed(&typ) {
            return Err(SoNError::TypTransitionNotAllowed);
        }
        println!("Node {:?} node_kind: {:?}, typ: {:?} -> {:?}", self.nid, self.node_kind, self.typ, typ);
        self.typ = typ;
        Ok(())
    }

    /// returns whether this node is associated with the control flow graph
    pub fn is_cfg(&self) -> bool {
        match self.node_kind {
            NodeKind::Return
            | NodeKind::Start
            => true,
            NodeKind::Constant
            | NodeKind::KeepAlive
            | NodeKind::Add
            | NodeKind::Sub
            | NodeKind::Mul
            | NodeKind::Div
            | NodeKind::Minus
            | NodeKind::Scope { .. }
            => false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn should_construct_start_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(Graph::new()));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Start, Typ::Bot).unwrap();
        let nid2 = Node::new(graph.clone(), vec![nid1], NodeKind::Start, Typ::Bot).unwrap();

        // Assert
        assert_eq!(nid2, graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().outputs[0]);
        assert_eq!(0, graph.borrow_mut().get(nid2).unwrap().as_ref().unwrap().outputs.len());
    }

    #[test]
    fn should_construct_constant_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(Graph::new()));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Constant, Typ::Int { constant: 42 }).unwrap();

        // Assert
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().typ, Typ::Int { constant: 42 }));
    }

    #[test]
    fn should_construct_return_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(Graph::new()));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Return, Typ::Bot).unwrap();

        // Assert
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Return));
    }

    #[test]
    fn should_construct_return_node_in_empty_slot() {
        // Arrange
        let graph = Rc::new(RefCell::new(Graph::from(vec![None])));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Return, Typ::Bot).unwrap();

        // Assert
        assert_eq!(1, graph.borrow().len());
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Return));
    }

    #[test]
    fn should_be_able_to_contain_same_dependency_multiple_times() {
        // Arrange
        let graph = Rc::new(RefCell::new(Graph::from(vec![None])));
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Constant, Typ::Bot).unwrap();

        // Act
        let nid2 = Node::new(graph.clone(), vec![nid1, nid1], NodeKind::Constant, Typ::Bot).unwrap();

        // Assert
        let graph_br = graph.borrow();
        assert_eq!(2, graph_br.get_node(nid2).unwrap().inputs.len());
        assert_eq!(2, graph_br.get_node(nid1).unwrap().outputs.len());
    }

    #[test]
    fn should_remove_dependency_from_the_back() {
        // Arrange
        let graph = Rc::new(RefCell::new(Graph::from(vec![None])));
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Constant, Typ::Bot).unwrap();
        let nid2 = Node::new(graph.clone(), vec![nid1], NodeKind::Constant, Typ::Bot).unwrap();
        let nid3 = Node::new(graph.clone(), vec![nid1], NodeKind::Constant, Typ::Bot).unwrap();
        let mut graph_br = graph.borrow_mut();
        graph_br.add_reverse_dependencies_br(nid2, &vec![nid1]).unwrap();
        graph_br.add_dependencies_br(nid2, &vec![nid1]).unwrap();

        // Act
        graph_br.remove_dependency_br(nid2, nid1).unwrap();

        // Assert
        assert!(matches!(graph_br.get_node( nid2).unwrap().inputs.as_slice(), [i] if i == &nid1));
        assert!(matches!(graph_br.get_node( nid1).unwrap().outputs.as_slice(), [i, j] if i == &nid2 && j == &nid3));
    }
}
