use crate::typ::typ::Typ;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{cell::RefCell, rc::Rc};

static GLOBAL_NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
#[derive(Clone)]
pub struct Graph {
    m_graph: Vec<Option<Node>>,
}

impl Deref for Graph {
    type Target = Vec<Option<Node>>;
    fn deref(&self) -> &Self::Target {
        &self.m_graph
    }
}

impl DerefMut for Graph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.m_graph
    }
}

impl Graph {
    pub fn from(g: Vec<Option<Node>>) -> Graph {
        Graph { m_graph: g }
    }

    pub fn new() -> Graph {
        Self::from(vec![])
    }

    /// automatically filters for None elements
    pub fn graph_iter(&self) -> impl Iterator<Item=&Node> {
        self.iter().filter_map(|x| x.as_ref())
    }

    /// automatically filters for None elements
    pub fn graph_iter_mut(&mut self) -> impl Iterator<Item=&Node> {
        self.iter_mut().filter_map(|x| x.as_ref())
    }

    /// remove dependency dep_nid from nid so nid doesn't depend on dep_nid anymore.
    pub fn remove_dependency_br(&mut self, nid: usize, dep_nid: usize) -> Result<(), SoNError> {
        if !self.node_exists(nid) || !self.node_exists(nid) {
            return Err(SoNError::NodeIdNotExisting);
        }

        let node = self.get_node_mut(nid)?;
        if let Some(pos) = node.inputs.iter().rev().position(|&x| x == dep_nid) {
            node.inputs.remove(node.inputs.len() - 1 - pos);
        }
        let dep = self.get_node_mut(dep_nid)?;
        if let Some(pos) = dep.outputs.iter().rev().position(|&x| x == nid) {
            dep.outputs.remove(dep.outputs.len() - 1 - pos);
        }
        Ok(())
    }

    /// make the usages for all nodes in deps to point to nid
    pub fn add_reverse_dependencies_br(&mut self, nid: usize, deps: &Vec<usize>) -> Result<(), SoNError> {
        for id in deps {
            match self.get_mut(*id) {
                Some(Some(def)) => {
                    def.outputs.push(nid);
                    // def.outputs = def.outputs.clone().into_iter().unique().collect();
                }
                _ => return Err(SoNError::NodeIdNotExisting),
            }
        }
        Ok(())
    }

    /// adds the dependencies for a node
    pub fn add_dependencies_br(&mut self, nid: usize, deps: &Vec<usize>) -> Result<(), SoNError> {
        match self.get_mut(nid) {
            Some(Some(node)) => {
                node.inputs.extend(deps);
                // node.inputs = node.inputs.clone().into_iter().unique().collect();
            },
            _ => return Err(SoNError::NodeIdNotExisting),
        };
        Ok(())
    }

    pub fn find_first_empty_cell(&mut self) -> usize {
        let index = self.iter().enumerate().find_map(|(i, x)| {
            if x.is_none() {
                Some(i)
            } else {
                None
            }
        }).unwrap_or_else(|| self.len());
        index
    }

    pub fn get_node_mut(&mut self, nid: usize) -> Result<&mut Node, SoNError> {
        self.get_mut(nid)
            .and_then(|n| n.as_mut())
            .ok_or(SoNError::NodeIdNotExisting)
    }

    pub fn get_node(&self, nid: usize) -> Result<&Node, SoNError> {
        self.get(nid)
            .and_then(|n| n.as_ref())
            .ok_or(SoNError::NodeIdNotExisting)
    }

    pub fn node_exists(&self, nid: usize) -> bool {
        self.get_node(nid).is_ok()
    }

    /// checks that the node in slot nid exists and that the unique id matches
    pub fn node_exists_unique(&self, nid: usize, uid: usize) -> bool {
        self.get_node(nid).is_ok_and(|x| x.uid == uid)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Node {
    /// for Display trait we need to be able to traverse the whole graph.
    graph: Rc<RefCell<Graph>>,
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
        let node = Node { graph: graph.clone(), node_kind, inputs: vec![], outputs: vec![], uid: GLOBAL_NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst), nid: index, typ };
        let inputs_c = inputs.clone();
        graph_br.add_reverse_dependencies_br(index, &inputs_c)?;
        if index == graph_br.len() {
            graph_br.push(None);
        }
        graph_br.m_graph[index] = Some(node.clone());
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
