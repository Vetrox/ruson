use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{cell::RefCell, rc::Rc};

static GLOBAL_NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
#[derive(Clone)]
pub struct Node {
    graph: Rc<RefCell<Vec<Option<Node>>>>,
    pub node_kind: NodeKind,
    /// ordered list of def`s this Node is depending on
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    /// unique id that is incremented with every new node
    pub uid: usize,
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
    KeepAlive
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
            NodeKind::KeepAlive => write!(f, "KeepAlive()")?,
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
        let index = find_first_empty_cell(&graph);
        let node = Node { graph: graph.clone(), node_kind, inputs: vec![], outputs: vec![], uid: GLOBAL_NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst) };
        let inputs_c = inputs.clone();
        add_usage_for_deps(graph.clone(), index, &inputs_c)?;
        if index == graph.borrow().len() {
            graph.borrow_mut().push(None);
        }
        graph.borrow_mut()[index] = Some(node);
        add_dependencies(graph.clone(), index, &inputs_c)?;
        Ok(index)
    }

    /// returns whether this node is associated with the control flow graph
    pub fn is_cfg(&self) -> bool {
        match self.node_kind {
            NodeKind::Return
            | NodeKind::Start => true,
            _ => false
        }
    }
}

pub fn find_first_empty_cell(graph: &Rc<RefCell<Vec<Option<Node>>>>) -> usize {
    let g = graph.borrow();
    let index = g.iter().enumerate().find_map(|(i, x)| {
        if x.is_none() {
            Some(i)
        } else {
            None
        }
    }).unwrap_or_else(|| g.len());
    index
}

/// adds the usages for all nodes in input to point to nid
pub fn add_usage_for_deps(
    graph: Rc<RefCell<Vec<Option<Node>>>>,
    nid: usize,
    deps: &Vec<usize>,
) -> Result<(), SoNError> {
    let mut graph_br = graph.borrow_mut();
    for id in deps {
        match graph_br.get_mut(*id) {
            Some(Some(def)) => {
                def.outputs.push(nid);
                def.outputs = def.outputs.clone().into_iter().unique().collect();
            }
            _ => return Err(SoNError::NodeIdNotExisting),
        }
    }
    Ok(())
}

pub fn get_node_mut(
    graph: &mut Vec<Option<Node>>,
    nid: usize,
) -> Result<&mut Node, SoNError> {
    graph
        .get_mut(nid)
        .and_then(|n| n.as_mut())
        .ok_or(SoNError::NodeIdNotExisting)
}

pub fn get_node(
    graph: &Vec<Option<Node>>,
    nid: usize,
) -> Result<&Node, SoNError> {
    graph
        .get(nid)
        .and_then(|n| n.as_ref())
        .ok_or(SoNError::NodeIdNotExisting)
}

pub fn node_exists(
    graph: &Vec<Option<Node>>,
    nid: usize,
) -> bool {
    get_node(graph, nid).is_ok()
}

/// checks that the node with slot nid exists and that the unique id matches
pub fn node_exists_unique(
    graph: &Vec<Option<Node>>,
    nid: usize,
    uid: usize,
) -> bool {
    get_node(graph, nid).is_ok_and(|x| x.uid == uid)
}

/// adds the dependencies for a node
pub fn add_dependencies(
    graph: Rc<RefCell<Vec<Option<Node>>>>,
    nid: usize,
    deps: &Vec<usize>,
) -> Result<(), SoNError> {
    let mut graph_br = graph.borrow_mut();
    match graph_br.get_mut(nid) {
        Some(Some(node)) => {
            node.inputs.extend(deps);
            node.inputs = node.inputs.clone().into_iter().unique().collect();
        },
        _ => return Err(SoNError::NodeIdNotExisting),
    };
    Ok(())
}

/// remove dependency dep_nid from nid so nid doesn't depend on dep_nid anymore.
pub fn remove_dependency(
    graph: Rc<RefCell<Vec<Option<Node>>>>,
    nid: usize,
    dep_nid: usize,
) -> Result<(), SoNError> {
    let mut graph_br = graph.borrow_mut();

    if !node_exists(&mut graph_br, nid) || !node_exists(&mut graph_br, nid) {
        return Err(SoNError::NodeIdNotExisting);
    }

    get_node_mut(&mut graph_br, nid)?.inputs.retain(|&x| x != dep_nid);
    get_node_mut(&mut graph_br, dep_nid)?.outputs.retain(|&x| x != nid);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn should_construct_start_node() {
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

    #[test]
    fn should_construct_return_node_in_empty_slot() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![None]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Return).unwrap();

        // Assert
        assert_eq!(1, graph.borrow().len());
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Return));
    }
}
