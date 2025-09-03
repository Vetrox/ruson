use crate::errors::son_error::SoNError;
use crate::nodes::node::CompNodeKind::{LogAnd, LogOr, LogXor};
use crate::nodes::node::NodeKind::{Comp, Constant, Mul};
use crate::nodes::node::{CompNodeKind, NodeKind};
use crate::services::parser::Parser;
use crate::typ::typ::Typ;
use crate::typ::typ::Typ::{Bool, Int};
use CompNodeKind::EQ;
use NodeKind::{Add, Div, KeepAlive, Minus, Proj, Return, Scope, Start, Sub};
use Typ::{BoolBot, BoolTop, IntBot, IntTop};

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
                    return Ok(lhs_nid); // T_ARITH_IDENT
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
                    return Ok(outer); // T_ASSOCIATIVITY
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
                        return Ok(outer); // T_RIGHT_CONST
                    }

                    if lhs_rhs.uid > rhs.uid {
                        let inner = self.add_node_unrefined(vec![lhs_lhs_nid, rhs_nid], Add)?;
                        let outer = self.add_node_unrefined(vec![inner, lhs_rhs_nid], Add)?;
                        return Ok(outer);  // T_CANONIC_INC_NID
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
                    return Ok(lhs_nid); // T_ARITH_IDENT
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
                    return Ok(lhs_nid); // T_ARITH_IDENT
                }
                Ok(nid)
            }
            Minus => Ok(nid),
            Scope { .. } => Ok(nid),
            Proj { .. } => Ok(nid),
            Comp { kind: ref comp_node_kind } => {
                let lhs_nid = node.inputs.get(0).unwrap().clone();
                let lhs = self.graph.get_node(lhs_nid)?;
                let rhs_nid = node.inputs.get(1).unwrap().clone();
                let rhs = self.graph.get_node(rhs_nid)?;

                if matches!(rhs.typ(), Bool { constant: _a @ true }) && matches!(comp_node_kind, LogAnd)
                    || matches!(rhs.typ(), Bool { constant: _a @ false }) && matches!(comp_node_kind, LogOr) {
                    return Ok(lhs_nid); // T_ARITH_IDENT
                }
                if lhs_nid == rhs_nid && matches!(comp_node_kind, LogAnd | LogOr) {
                    return Ok(lhs_nid); // T_ADD_SAME
                }

                if lhs_nid == rhs_nid && matches!(comp_node_kind, LogXor) {
                    if matches!(node.typ(), Int { .. } | IntBot | IntTop ) {
                        return Ok(self.add_node(vec![], Constant, Int { constant: 0 })?); // T_ADD_SAME
                    }
                    if matches!(node.typ(),  Bool { .. } | BoolTop | BoolBot ) {
                        return Ok(self.add_node(vec![], Constant, Bool { constant: false })?); // T_ADD_SAME
                    }
                }

                // Note: T_LEFT_SPINE is only implemented for situations where the operation is commutative.
                if !matches!(&lhs.node_kind, Comp { kind: lhs_comp_node_kind } if lhs_comp_node_kind == comp_node_kind)
                    && matches!(&rhs.node_kind, Comp { kind: rhs_comp_node_kind } if rhs_comp_node_kind == comp_node_kind)
                    && matches!(comp_node_kind, EQ | LogXor | LogAnd | LogOr) {
                    let mut_node = self.graph.get_node_mut(nid)?;
                    mut_node.inputs[0] = rhs_nid;
                    mut_node.inputs[1] = lhs_nid;
                    return Ok(self.peephole(nid)?); // T_LEFT_SPINE
                }

                if matches!(&rhs.node_kind, Comp { kind: rhs_comp_node_kind } if rhs_comp_node_kind == comp_node_kind)
                    && matches!(comp_node_kind, LogXor | LogAnd | LogOr) {
                    let rhs_lhs_nid = rhs.inputs.get(0).unwrap().clone();
                    let rhs_rhs_nid = rhs.inputs.get(1).unwrap().clone();
                    let inner = self.add_node_unrefined(vec![lhs_nid, rhs_lhs_nid], Comp { kind: comp_node_kind.clone() })?;
                    let outer = self.add_node_unrefined(vec![inner, rhs_rhs_nid], Comp { kind: comp_node_kind.clone() })?;
                    return Ok(outer); // T_ASSOCIATIVITY
                }

                if !matches!(&lhs.node_kind, Comp { kind: lhs_comp_node_kind } if lhs_comp_node_kind == comp_node_kind)
                    && !matches!(&rhs.node_kind, Comp { kind: rhs_comp_node_kind } if rhs_comp_node_kind == comp_node_kind)
                    && matches!(comp_node_kind, EQ | LogXor | LogAnd | LogOr) {
                    return if lhs.uid > rhs.uid {
                        let mut_node = self.graph.get_node_mut(nid)?;
                        mut_node.inputs[0] = rhs_nid;
                        mut_node.inputs[1] = lhs_nid;
                        Ok(self.peephole(nid)?) // T_CANONIC_INC_NID
                    } else {
                        Ok(nid)
                    }
                }

                if matches!(&lhs.node_kind, Comp { kind: lhs_comp_node_kind } if lhs_comp_node_kind == comp_node_kind)
                    && matches!(comp_node_kind, LogXor | LogAnd | LogOr) {
                    let lhs_lhs_nid = lhs.inputs.get(0).unwrap().clone();
                    let lhs_rhs_nid = lhs.inputs.get(1).unwrap().clone();
                    let lhs_rhs = self.graph.get_node(lhs_rhs_nid)?;
                    if lhs_rhs.typ().is_constant() && rhs.typ().is_constant() {
                        let inner = self.add_node_unrefined(vec![lhs_rhs_nid, rhs_nid], Comp { kind: comp_node_kind.clone() })?;
                        let outer = self.add_node_unrefined(vec![lhs_lhs_nid, inner], Comp { kind: comp_node_kind.clone() })?;
                        return Ok(self.peephole(outer)?); // T_RIGHT_CONST
                    }
                }

                if matches!(&lhs.node_kind, Comp { kind: lhs_comp_node_kind } if lhs_comp_node_kind == comp_node_kind)
                    && matches!(comp_node_kind, EQ | LogXor | LogAnd | LogOr) {
                    let lhs_lhs_nid = lhs.inputs.get(0).unwrap().clone();
                    let lhs_rhs_nid = lhs.inputs.get(1).unwrap().clone();
                    let lhs_rhs = self.graph.get_node(lhs_rhs_nid)?;
                    if lhs_rhs.uid > rhs.uid {
                        let inner = self.add_node_unrefined(vec![lhs_lhs_nid, rhs_nid], Comp { kind: comp_node_kind.clone() })?;
                        let outer = self.add_node_unrefined(vec![inner, lhs_rhs_nid], Comp { kind: comp_node_kind.clone() })?;
                        return Ok(self.peephole(outer)?);  // T_CANONIC_INC_NID
                    }
                }

                Ok(nid)
            }
            NodeKind::Not => Ok(nid)
        }
    }
}