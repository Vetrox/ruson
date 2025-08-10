use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct Node {
    graph: Rc<RefCell<Vec<Node>>>,
    node_kind: NodeKind,
    /// ordered list of def`s this Node is depending on
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Start,
    Constant { value: i64 },
}

#[derive(Debug)]
pub enum SoNError {
    NodeIdNotExisting,
}

impl Node {
    pub fn new_start(
        graph: Rc<RefCell<Vec<Node>>>,
        inputs: Vec<usize>,
        node_type: NodeKind,
    ) -> Result<usize, SoNError> {
        let index = graph.borrow().len();

        let node = Node {
            graph: graph.clone(),
            node_kind: node_type,
            inputs,
            outputs: vec![],
        };

        add_use(graph.clone(), index, &node.inputs)?;

        graph.borrow_mut().push(node);

        Result::Ok(index)
    }
}

fn add_use(
    graph: Rc<RefCell<Vec<Node>>>,
    index: usize,
    inputs: &Vec<usize>,
) -> Result<(), SoNError> {
    let mut graph_br = graph.borrow_mut();
    for id in inputs {
        match graph_br.get_mut(*id) {
            Some(def) => def.outputs.push(index),
            None => return Result::Err(SoNError::NodeIdNotExisting),
        }
    }
    Result::Ok(())
}

// #[derive(Default)]
// struct Graph {
//     graph: Rc<RefCell<Vec<Node>>>,
// }

// impl Graph {
//     unsafe fn get(&self, index: usize) -> Ref<Option<&Node>> {
//         let vec_ref = self.graph.borrow();
//         Ref::map(vec_ref, |v| v.get(index))
//     }
// }

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn should_run_parse() {
        let graph = Rc::new(RefCell::new(vec![]));

        let nid1 = Node::new_start(graph.clone(), vec![], NodeKind::Start).unwrap();
        let nid2 = Node::new_start(graph.clone(), vec![nid1], NodeKind::Start).unwrap();

        assert_eq!(nid2, graph.borrow_mut().get(nid1).unwrap().outputs[0]);
        assert_eq!(0, graph.borrow_mut().get(nid2).unwrap().outputs.len());
    }

    #[test]
    fn should_construct_constant_node() {
        let graph = Rc::new(RefCell::new(vec![]));

        let nid1 = Node::new_start(graph.clone(), vec![], NodeKind::Constant { value: 42 }).unwrap();
        assert!(matches!(graph.borrow_mut().get(nid1).unwrap().node_kind, NodeKind::Constant { value: 42 }));
    }
}
