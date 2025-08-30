use crate::nodes::graph::Graph;
use crate::nodes::node::{Node, NodeKind};
use crate::typ::typ::Typ;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

pub struct BoundNode<'a> {
    node: &'a Node,
    graph: &'a Graph,
}

impl<'a> BoundNode<'a> {
    pub fn new(node: &'a Node, graph: &'a Graph) -> BoundNode<'a> {
        BoundNode { node, graph }
    }

    pub fn from(&self, other: &'a Node) -> BoundNode<'a> {
        Self::new(other, self.graph)
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
            => false,
            NodeKind::Proj { proj_index, _dbg_proj_label: _ } => proj_index == 0 /*&& matches!(self.graph.get_node(*self.inputs.get(proj_index).unwrap()).unwrap().node_kind, NodeKind::If)*/,
        }
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
                let lhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                write!(f, "return {};", format!("{}", self.from(node_lhs)))?
            }
            NodeKind::Start => write!(f, "Start()")?,
            NodeKind::KeepAlive => write!(f, "KeepAlive()")?,
            NodeKind::Add => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}+{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Sub => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}-{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Mul => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}*{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Div => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}/{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            NodeKind::Minus => {
                let lhs = self.inputs.get(0).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
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
                        let node_lhs = self.graph.get_node(*v).unwrap().clone();
                        write!(f, "{}: {}", k, format!("{}", self.from(&node_lhs)))?;
                    }
                    write!(f, "]")?;
                }
                write!(f, ")")?;
            }
            NodeKind::Proj { _dbg_proj_label, .. } => {
                write!(f, "{}", _dbg_proj_label)?
            }
        }
        Ok(())
    }
}