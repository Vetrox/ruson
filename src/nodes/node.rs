use crate::nodes::typ_refiner::compute_refined_typ;
use crate::typ::typ::Typ;
use std::collections::HashMap;
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

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.clone().node_kind {
            NodeKind::Constant => {
                match self.typ() {
                    Typ::Int { constant } => write!(f, "{}", constant)?,
                    _ => panic!("Type {:?} for NodeKind::Constant unsupported", self.typ()),
                }
            }
            NodeKind::Return => {
                let data_nid = self.inputs.get(1).unwrap();
                let node = self.graph.borrow_mut().get(*data_nid).unwrap().as_ref().unwrap().clone();
                write!(f, "return {};", format!("{}", node))?
            }
            NodeKind::Start => write!(f, "Start()")?,
            NodeKind::KeepAlive => write!(f, "KeepAlive()")?,
            NodeKind::Add => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}+{})", format!("{}", node_lhs), format!("{}", node_rhs))?
            }
            NodeKind::Sub => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}-{})", format!("{}", node_lhs), format!("{}", node_rhs))?
            }
            NodeKind::Mul => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}*{})", format!("{}", node_lhs), format!("{}", node_rhs))?
            }
            NodeKind::Div => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}/{})", format!("{}", node_lhs), format!("{}", node_rhs))?
            }
            NodeKind::Minus => {
                let lhs = self.inputs.get(0).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                write!(f, "(-{})", format!("{}", node_lhs))?
            },
            NodeKind::Scope { scopes } => {
                write!(f, "Scope(")?;
                for scope in scopes {
                    let mut entries: Vec<_> = scope.iter().collect();
                    entries.sort_by_key(|&(k, _)| k);

                    write!(f, "[")?;
                    let mut first = true;
                    for (k, v) in entries {
                        if !(first) {
                            write!(f, ", ")?;
                        }
                        first = false;
                        let node_lhs = self.graph.borrow().get(*v).unwrap().as_ref().unwrap().clone();
                        write!(f, "{}: {}", k, format!("{}", node_lhs))?;
                    }
                    write!(f, "]")?;
                }
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SoNError {
    NodeIdNotExisting,
    NumberCannotStartWith0,
    SyntaxExpected { expected: String, actual: String },
    TypTransitionNotAllowed,
}

impl Node {
    pub fn new(
        graph: Rc<RefCell<Vec<Option<Node>>>>,
        inputs: Vec<usize>,
        node_kind: NodeKind,
        typ: Typ,
    ) -> Result<usize, SoNError> {
        let index = find_first_empty_cell(&graph);
        let node = Node { graph: graph.clone(), node_kind, inputs: vec![], outputs: vec![], uid: GLOBAL_NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst), nid: index, typ };
        let inputs_c = inputs.clone();
        add_usage_for_deps(graph.clone(), index, &inputs_c)?;
        if index == graph.borrow().len() {
            graph.borrow_mut().push(None);
        }
        graph.borrow_mut()[index] = Some(node.clone());
        add_dependencies(graph.clone(), index, &inputs_c)?;

        // refine the node typ immediately. This sets the refined typ but doesn't optimize anything.
        let graph_br = graph.borrow();
        let n = get_node(graph_br.as_ref(), index)?;
        let typ = compute_refined_typ(&*graph.borrow(), n)?;
        drop(graph_br);
        let mut graph_br = graph.borrow_mut();
        get_node_mut(graph_br.as_mut(), index)?.refine_typ(typ)?;

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
            NodeKind::Constant => {}
            NodeKind::Return => {}
            NodeKind::Start => {}
            NodeKind::KeepAlive => {}
            NodeKind::Add => {}
            NodeKind::Sub => {}
            NodeKind::Mul => {}
            NodeKind::Div => {}
            NodeKind::Minus => {}
            NodeKind::Scope { .. } => {}
        }
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
                // def.outputs = def.outputs.clone().into_iter().unique().collect();
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
            // node.inputs = node.inputs.clone().into_iter().unique().collect();
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

    if let Some(pos) = get_node_mut(&mut graph_br, nid)?.inputs.iter().position(|&x| x == dep_nid) {
        get_node_mut(&mut graph_br, nid)?.inputs.remove(pos);
    }
    if let Some(pos) = get_node_mut(&mut graph_br, dep_nid)?.outputs.iter().position(|&x| x == nid) {
        get_node_mut(&mut graph_br, dep_nid)?.outputs.remove(pos);
    }
    Ok(())
}

pub fn iter_graph(
    graph: &Vec<Option<Node>>) -> impl Iterator<Item=&Node> {
    graph.iter().filter_map(|x| x.as_ref())
}

pub fn iter_graph_mut(
    graph: &mut Vec<Option<Node>>) -> impl Iterator<Item=&Node> {
    graph.iter_mut().filter_map(|x| x.as_ref())
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
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Start, Typ::Bot).unwrap();
        let nid2 = Node::new(graph.clone(), vec![nid1], NodeKind::Start, Typ::Bot).unwrap();

        // Assert
        assert_eq!(nid2, graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().outputs[0]);
        assert_eq!(0, graph.borrow_mut().get(nid2).unwrap().as_ref().unwrap().outputs.len());
    }

    #[test]
    fn should_construct_constant_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Constant, Typ::Int { constant: 42 }).unwrap();

        // Assert
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().typ, Typ::Int { constant: 42 }));
    }

    #[test]
    fn should_construct_return_node() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Return, Typ::Bot).unwrap();

        // Assert
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Return));
    }

    #[test]
    fn should_construct_return_node_in_empty_slot() {
        // Arrange
        let graph = Rc::new(RefCell::new(vec![None]));

        // Act
        let nid1 = Node::new(graph.clone(), vec![], NodeKind::Return, Typ::Bot).unwrap();

        // Assert
        assert_eq!(1, graph.borrow().len());
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().as_ref().unwrap().node_kind, NodeKind::Return));
    }
}
