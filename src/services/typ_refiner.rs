use crate::errors::son_error::SoNError;
use crate::nodes::node::{CompNodeKind, Graph, Node, NodeKind};
use crate::typ::typ::Typ;

impl Graph {
    pub fn compute_refined_typ(&self, node: &Node) -> Result<Typ, SoNError> {
        match &node.node_kind {
            NodeKind::Add => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;
                let rhs = self.get_node(*node.inputs.get(1).unwrap())?;

                if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                    return Ok(Typ::Int { constant: clhs + crhs }); // T_CONSTFLD
                }
                Ok(node.typ())
            }
            NodeKind::Sub => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;
                let rhs = self.get_node(*node.inputs.get(1).unwrap())?;

                if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                    return Ok(Typ::Int { constant: clhs - crhs }); // T_CONSTFLD
                }
                Ok(node.typ())
            }
            NodeKind::Mul => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;
                let rhs = self.get_node(*node.inputs.get(1).unwrap())?;

                if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                    return Ok(Typ::Int { constant: clhs * crhs }); // T_CONSTFLD
                }
                Ok(node.typ())
            }
            NodeKind::Div => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;
                let rhs = self.get_node(*node.inputs.get(1).unwrap())?;

                if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                    return Ok(Typ::Int { constant: clhs / crhs }); // T_CONSTFLD
                }
                Ok(node.typ())
            }
            NodeKind::Minus => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;

                if let Typ::Int { constant: clhs } = lhs.typ() {
                    return Ok(Typ::Int { constant: -clhs }); // T_CONSTFLD
                }
                Ok(node.typ())
            }
            NodeKind::Proj { proj_index, .. } => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;

                if let Typ::Tuple { typs } = lhs.typ() {
                    return Ok(typs.get(*proj_index).unwrap().clone());
                }
                Ok(node.typ())
            }
            NodeKind::Constant
            | NodeKind::Return
            | NodeKind::Start
            | NodeKind::KeepAlive
            | NodeKind::Scope { .. }
            => Ok(node.typ()),
            NodeKind::Comp { kind } => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;
                let rhs = self.get_node(*node.inputs.get(1).unwrap())?;
                match kind {
                    CompNodeKind::LT => {
                        if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs < crhs }); // T_CONSTFLD
                        }
                    }
                    CompNodeKind::LEQ => {
                        if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs <= crhs }); // T_CONSTFLD
                        }
                    }
                    CompNodeKind::EQ => {
                        if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs == crhs }); // T_CONSTFLD
                        }
                        if let Typ::Bool { constant: clhs } = lhs.typ() && let Typ::Bool { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs == crhs }); // T_CONSTFLD
                        }
                    }
                    CompNodeKind::LogAnd => {
                        if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Int { constant: clhs & crhs }); // T_CONSTFLD
                        }
                        if let Typ::Bool { constant: clhs } = lhs.typ() && let Typ::Bool { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs && crhs }); // T_CONSTFLD
                        }
                    }
                    CompNodeKind::LogOr => {
                        if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Int { constant: clhs | crhs }); // T_CONSTFLD
                        }
                        if let Typ::Bool { constant: clhs } = lhs.typ() && let Typ::Bool { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs || crhs }); // T_CONSTFLD
                        }
                    }
                    CompNodeKind::LogXor => {
                        if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Int { constant: clhs ^ crhs }); // T_CONSTFLD
                        }
                        if let Typ::Bool { constant: clhs } = lhs.typ() && let Typ::Bool { constant: crhs } = rhs.typ() {
                            return Ok(Typ::Bool { constant: clhs ^ crhs }); // T_CONSTFLD
                        }
                    }
                }
                Ok(node.typ())
            }
            NodeKind::Not => {
                let lhs = self.get_node(*node.inputs.get(0).unwrap())?;

                if let Typ::Int { constant: clhs } = lhs.typ() {
                    return Ok(Typ::Int { constant: !clhs }); // T_CONSTFLD
                }

                if let Typ::Bool { constant: clhs } = lhs.typ() {
                    return Ok(Typ::Bool { constant: !clhs }); // T_CONSTFLD
                }

                Ok(node.typ())
            }
        }
    }
}