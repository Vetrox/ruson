use crate::nodes::node::{Node, SoNError};
use std::ops::{Deref, DerefMut};

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
            }
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