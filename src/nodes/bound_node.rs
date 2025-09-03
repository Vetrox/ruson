use crate::nodes::graph::Graph;
use crate::nodes::node::{CompNodeKind, Node, NodeKind};
use crate::typ::typ::Typ;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use NodeKind::{Add, Comp, Constant, Div, KeepAlive, Minus, Mul, Not, Proj, Return, Scope, Start, Sub};

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
            Return
            | Start
            | Comp { .. }
            | Not
            => true,
            Constant
            | KeepAlive
            | Add
            | Sub
            | Mul
            | Div
            | Minus
            | Scope { .. }
            => false,
            Proj { proj_index, _dbg_proj_label: _ } => proj_index == 0 /*&& matches!(self.graph.get_node(*self.inputs.get(proj_index).unwrap()).unwrap().node_kind, NodeKind::If)*/,
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
            Constant => {
                match self.typ() {
                    Typ::Int { constant } => write!(f, "{}", constant)?,
                    _ => panic!("Type {:?} for NodeKind::Constant unsupported", self.typ()),
                }
            }
            Return => {
                let lhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                write!(f, "return {};", format!("{}", self.from(node_lhs)))?
            }
            Start => write!(f, "Start()")?,
            KeepAlive => write!(f, "KeepAlive()")?,
            Add => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}+{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            Sub => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}-{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            Mul => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}*{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            Div => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                write!(f, "({}/{})", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
            }
            Minus => {
                let lhs = self.inputs.get(0).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                write!(f, "(-{})", format!("{}", self.from(&node_lhs)))?
            }
            Scope { scopes } => {
                write!(f, "Scope(")?;
                for scope in scopes {
                    let mut entries: Vec<_> = scope.iter().collect();
                    entries.sort_by_key(|&(k, _)| k);

                    write!(f, "[")?;
                    let mut first = true;
                    for (k, v) in entries {
                        if !first {
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
            Proj { _dbg_proj_label, .. } => {
                write!(f, "{}", _dbg_proj_label)?
            },
            Comp { kind } => {
                let lhs = self.inputs.get(0).unwrap();
                let rhs = self.inputs.get(1).unwrap();
                let node_lhs = self.graph.get_node(*lhs).unwrap();
                let node_rhs = self.graph.get_node(*rhs).unwrap();
                match kind {
                    CompNodeKind::LT => {
                        write!(f, "{} < {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
                    }
                    CompNodeKind::LEQ => {
                        write!(f, "{} <= {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
                    }
                    CompNodeKind::EQ => {
                        write!(f, "{} == {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
                    },
                    CompNodeKind::LogAnd => {
                        match self.typ() {
                            Typ::Int { .. } | Typ::IntTop | Typ::IntBot => write!(f, "{} & {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?,
                            Typ::Bool { .. } | Typ::BoolTop | Typ::BoolBot => write!(f, "{} && {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?,
                            _ => write!(f, "Unsupported LogAnd comparison Typ")?
                        }
                    },
                    CompNodeKind::LogOr => {
                        match self.typ() {
                            Typ::Int { .. } | Typ::IntTop | Typ::IntBot => write!(f, "{} | {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?,
                            Typ::Bool { .. } | Typ::BoolTop | Typ::BoolBot => write!(f, "{} || {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?,
                            _ => write!(f, "Unsupported LogOr comparison Typ")?
                        }
                    },
                    CompNodeKind::LogXor => {
                        write!(f, "{} ^ {}", format!("{}", self.from(&node_lhs)), format!("{}", self.from(&node_rhs)))?
                    },
                }
            }
            Not => {}
        }
        Ok(())
    }
}