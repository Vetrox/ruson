use crate::nodes::graph::Graph;
use crate::nodes::node::{Node, NodeKind};
use crate::typ::typ::Typ;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;

pub struct BoundNode<'a> {
    pub node: &'a Node,
    graph: Rc<RefCell<Graph>>,
}

impl<'a> BoundNode<'a> {
    pub fn new(node: &'a Node, graph: Rc<RefCell<Graph>>) -> BoundNode<'a> {
        BoundNode { node, graph }
    }

    pub fn from(&self, other: &'a Node) -> BoundNode<'a> {
        Self::new(other, self.graph.clone())
    }
}

impl Deref for BoundNode<'_> {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        self.node
    }
}

impl Display for BoundNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.node.clone().node_kind {
            NodeKind::Constant => {
                match self.typ() {
                    Typ::Int { constant } => write!(f, "{}", constant)?,
                    _ => panic!("Type {:?} for NodeKind::Constant unsupported", self.typ()),
                }
            }
            NodeKind::Return => {
                let data_nid = self.inputs.get(1).unwrap();
                let node = self.graph.borrow_mut().get(*data_nid).unwrap().as_ref().unwrap().clone();
                write!(f, "return {};", format!("{}", self.from(&node)))?
            }
            NodeKind::Start => write!(f, "Start()")?,
            NodeKind::KeepAlive => write!(f, "KeepAlive()")?,
            NodeKind::Add => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}+{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Sub => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}-{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Mul => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}*{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Div => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                let node_rhs = self.graph.borrow_mut().get(*rhs).unwrap().as_ref().unwrap().clone();
                write!(f, "({}/{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Minus => {
                let lhs = self.inputs.get(0).unwrap();
                let node_lhs = self.graph.borrow_mut().get(*lhs).unwrap().as_ref().unwrap().clone();
                write!(f, "(-{})", format!("{}", self.from(&node_lhs)))?
            }
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
                        write!(f, "{}: {}", k, format!("{}", self.from(&node_lhs)))?;
                    }
                    write!(f, "]")?;
                }
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}