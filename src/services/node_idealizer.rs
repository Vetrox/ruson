use crate::nodes::node::NodeKind::{Constant, Mul};
use crate::nodes::node::{NodeKind, SoNError};
use crate::services::parser::Parser;
use crate::typ::typ::Typ::Int;
use NodeKind::{Add, Div, KeepAlive, Minus, Proj, Return, Scope, Start, Sub};

impl Parser {
    pub(crate) fn idealize_node(&mut self, nid: usize) -> Result<usize, SoNError> {
        let node = self.graph.get_node(nid)?.clone();
        match node.node_kind {
            Constant => Ok(nid),
            Return => Ok(nid),
            Start => Ok(nid),
            KeepAlive => Ok(nid),
            Add => {
                let lhs_nid = node.inputs.get(0).unwrap().clone();
                let lhs = self.graph.get_node(lhs_nid)?;
                let rhs_nid = node.inputs.get(1).unwrap().clone();
                let rhs = self.graph.get_node(rhs_nid)?;
                assert!(!lhs.typ().is_constant() || !rhs.typ().is_constant(), "Already handled by peephole constant folding");

                if let Int { constant } = rhs.typ() && constant == 0 {
                    return Ok(self.peephole(lhs_nid)?); // T_ARITH_IDENT
                }

                if lhs_nid == rhs_nid {
                    let two = self.add_node(vec![], Constant, Int { constant: 2 })?;
                    return Ok(self.add_node_unrefined(vec![lhs_nid, two], Mul)?); // T_ADD_SAME
                }

                let is_lhs_add = matches!(&lhs.node_kind, Add);
                let is_rhs_add = matches!(&rhs.node_kind, Add);
                if !is_lhs_add && is_rhs_add {
                    let mut_node = self.graph.get_node_mut(nid)?;
                    mut_node.inputs[0] = rhs_nid;
                    mut_node.inputs[1] = lhs_nid;
                    return Ok(self.peephole(nid)?); // T_LEFT_SPINE
                }

                if is_rhs_add {
                    let rhs_lhs_nid = rhs.inputs.get(0).unwrap().clone();
                    let rhs_rhs_nid = rhs.inputs.get(1).unwrap().clone();
                    let inner = self.add_node_unrefined(vec![lhs_nid, rhs_lhs_nid], Add)?;
                    let outer = self.add_node_unrefined(vec![inner, rhs_rhs_nid], Add)?;
                    return Ok(self.peephole(outer)?); // T_ASSOCIATIVITY
                }

                if !is_lhs_add && !is_rhs_add {
                    return if lhs.uid > rhs.uid {
                        let mut_node = self.graph.get_node_mut(nid)?;
                        mut_node.inputs[0] = rhs_nid;
                        mut_node.inputs[1] = lhs_nid;
                        Ok(self.peephole(nid)?) // T_CANONIC_INC_NID
                    } else {
                        Ok(nid)
                    }
                }

                if is_lhs_add {
                    let lhs_lhs_nid = lhs.inputs.get(0).unwrap().clone();
                    let lhs_rhs_nid = lhs.inputs.get(1).unwrap().clone();
                    let lhs_rhs = self.graph.get_node(lhs_rhs_nid)?;
                    if lhs_rhs.typ().is_constant() && rhs.typ().is_constant() {
                        let inner = self.add_node_unrefined(vec![lhs_rhs_nid, rhs_nid], Add)?;
                        let outer = self.add_node_unrefined(vec![lhs_lhs_nid, inner], Add)?;
                        return Ok(self.peephole(outer)?); // T_RIGHT_CONST
                    }

                    if lhs_rhs.uid > rhs.uid {
                        let inner = self.add_node_unrefined(vec![lhs_lhs_nid, rhs_nid], Add)?;
                        let outer = self.add_node_unrefined(vec![inner, lhs_rhs_nid], Add)?;
                        return Ok(self.peephole(outer)?);  // T_CANONIC_INC_NID
                    }
                }
                Ok(nid)
            }
            Sub => Ok(nid),
            Mul => {
                let lhs_nid = node.inputs.get(0).unwrap().clone();
                let lhs = self.graph.get_node(lhs_nid)?;
                let rhs_nid = node.inputs.get(1).unwrap().clone();
                let rhs = self.graph.get_node(rhs_nid)?;

                if let Int { constant } = rhs.typ() && constant == 1 {
                    return Ok(self.peephole(lhs_nid)?); // T_ARITH_IDENT
                }

                if lhs.typ().is_constant() && !rhs.typ().is_constant() {
                    let mut_node = self.graph.get_node_mut(nid)?;
                    mut_node.inputs[0] = rhs_nid;
                    mut_node.inputs[1] = lhs_nid;
                    return Ok(self.peephole(nid)?)  // T_RIGHT_CONST
                }

                Ok(nid)
            }
            Div => {
                let lhs_nid = node.inputs.get(0).unwrap().clone();
                let rhs_nid = node.inputs.get(1).unwrap().clone();
                let rhs = self.graph.get_node(rhs_nid)?;

                if let Int { constant } = rhs.typ() && constant == 1 {
                    return Ok(self.peephole(lhs_nid)?); // T_ARITH_IDENT
                }
                Ok(nid)
            }
            Minus => {
                Ok(nid)
            }
            Scope { .. } => Ok(nid),
            Proj { .. } => Ok(nid)
        }
    }
}