use crate::nodes::node::{Graph, Node, NodeKind, SoNError};
use crate::typ::typ::Typ;

impl Graph {
    pub fn compute_refined_typ(&self, node: &Node) -> Result<Typ, SoNError> {
        match node.node_kind {
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
            NodeKind::Constant
            | NodeKind::Return
            | NodeKind::Start
            | NodeKind::KeepAlive
            | NodeKind::Scope { .. }
            => Ok(node.typ())
        }
    }
}