use std::fmt::{Display, Formatter};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
#[derive(Clone)]
pub struct Node {
    graph: Rc<RefCell<Vec<Option<Node>>>>,
    pub node_kind: NodeKind,
    /// ordered list of def`s this Node is depending on
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Constant { value: i64 },
    Return,
    Start,
}


impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.node_kind {
            NodeKind::Constant { value } => write!(f, "Constant({})", value)?,
            NodeKind::Return => {
                let data_nid = self.inputs.get(1).unwrap();
                let node = self.graph.borrow_mut().get(*data_nid).unwrap().as_ref().unwrap().clone();
                write!(f, "Return({})", format!("{}", node))?
            }
            NodeKind::Start => write!(f, "Start()")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SoNError {
    NodeIdNotExisting,
    NumberCannotStartWith0,
    SyntaxExpected { expected: String, actual: String },
}

impl Node {
    pub fn new(
        graph: Rc<RefCell<Vec<Option<Node>>>>,
        inputs: Vec<usize>,
        node_kind: NodeKind,
    ) -> Result<usize, SoNError> {
        let index = graph.borrow().len();
        let node = Node {
            graph: graph.clone(),
            node_kind,
            inputs,
            outputs: vec![],
        };
        add_use(graph.clone(), index, &node.inputs)?;
        graph.borrow_mut().push(Some(node));
        Ok(index)
    }
}

fn add_use(
    graph: Rc<RefCell<Vec<Option<Node>>>>,
    index: usize,
    inputs: &Vec<usize>,
) -> Result<(), SoNError> {
    let mut graph_br = graph.borrow_mut();
    for id in inputs {
        match graph_br.get_mut(*id) {
            Some(Some(def)) => def.outputs.push(index),
            _ => return Err(SoNError::NodeIdNotExisting),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn should_run_parse() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Start).unwrap();
        let nid2 = Node::new(graph.clone(), vec![nid1], NodeKind::Start).unwrap();

        // Assert
        assert_eq!(nid2, graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().outputs[0]);
        assert_eq!(0, graph.borrow_mut().get(nid2).unwrap().as_ref().unwrap().outputs.len());
    }

    #[test]
    fn should_construct_constant_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Constant { value: 42 }).unwrap();

        // Assert
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Constant { value: 42 }));
    }

    #[test]
    fn should_construct_return_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Return).unwrap();

        // Assert
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Return));
    }
}
