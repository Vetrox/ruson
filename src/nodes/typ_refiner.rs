use crate::nodes::node::{get_node, Node, NodeKind, SoNError};
use crate::typ::typ::Typ;

pub fn compute_refined_typ(graph: &Vec<Option<Node>>, node: &Node) -> Result<Typ, SoNError> {
    match node.node_kind {
        NodeKind::Add => {
            let lhs = get_node(graph, *node.inputs.get(0).unwrap())?;
            let rhs = get_node(graph, *node.inputs.get(1).unwrap())?;

            if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                return Ok(Typ::Int { constant: clhs + crhs });
            }
            Ok(node.typ())
        }
        NodeKind::Sub => {
            let lhs = get_node(graph, *node.inputs.get(0).unwrap())?;
            let rhs = get_node(graph, *node.inputs.get(1).unwrap())?;

            if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                return Ok(Typ::Int { constant: clhs - crhs });
            }
            Ok(node.typ())
        }
        NodeKind::Mul => {
            let lhs = get_node(graph, *node.inputs.get(0).unwrap())?;
            let rhs = get_node(graph, *node.inputs.get(1).unwrap())?;

            if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                return Ok(Typ::Int { constant: clhs * crhs });
            }
            Ok(node.typ())
        }
        NodeKind::Div => {
            let lhs = get_node(graph, *node.inputs.get(0).unwrap())?;
            let rhs = get_node(graph, *node.inputs.get(1).unwrap())?;

            if let Typ::Int { constant: clhs } = lhs.typ() && let Typ::Int { constant: crhs } = rhs.typ() {
                return Ok(Typ::Int { constant: clhs / crhs });
            }
            Ok(node.typ())
        }
        NodeKind::Minus => {
            let lhs = get_node(graph, *node.inputs.get(0).unwrap())?;

            if let Typ::Int { constant: clhs } = lhs.typ() {
                return Ok(Typ::Int { constant: -clhs });
            }
            Ok(node.typ())
        }
        _ => Ok(node.typ())
    }
}