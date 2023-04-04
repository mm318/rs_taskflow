use std::collections::{HashMap, HashSet};

use crate::dag::dag::Dag;
use crate::dag::node::{Node, NodeId};
use std::cell::{Ref, RefCell};
use std::sync::RwLockReadGuard;

enum CycleCheckStatus {
    Initial,
    Processing,
    Processed,
}

pub struct DagVisitationInfo<'a, T: Eq + Clone> {
    dag: &'a Dag<T>,
    dependencies: RefCell<Vec<HashSet<NodeId>>>, // upstream nodes
    dependants: Vec<HashSet<NodeId>>,            // downstream nodes
    roots: RefCell<HashSet<NodeId>>,
}

impl<'a, T: Eq + Clone> DagVisitationInfo<'a, T> {
    pub(crate) fn new(dag: &'a Dag<T>) -> Self {
        let len = dag.get_num_nodes();

        let mut result = Self {
            dag: dag,
            dependencies: RefCell::new(Vec::with_capacity(len)),
            dependants: Vec::with_capacity(len),
            roots: RefCell::new(HashSet::new()),
        };

        result.dependencies.borrow_mut().resize(len, HashSet::new());
        result.dependants.resize(len, HashSet::new());

        result
    }

    pub(crate) fn check(self) -> Result<Self, &'static str> {
        if self.get_next_root().is_none() {
            return Err("No roots found. DAG is invalid!");
        }

        if self
            .get_roots()
            .iter()
            .all(|root_id| self._check(*root_id, &mut HashMap::new()))
        {
            Ok(self)
        } else {
            Err("Invalid DAG detected")
        }
    }

    fn _check(
        &self,
        curr_node_id: NodeId,
        visited: &mut HashMap<NodeId, CycleCheckStatus>,
    ) -> bool {
        visited.insert(curr_node_id, CycleCheckStatus::Processing);

        for dep in self.get_dependants(curr_node_id).iter() {
            let status = match visited.get(dep) {
                Some(v) => v,
                None => &CycleCheckStatus::Initial,
            };

            match status {
                CycleCheckStatus::Initial => {
                    if !self._check(*dep, visited) {
                        return false;
                    }
                }
                CycleCheckStatus::Processing => return false,
                CycleCheckStatus::Processed => {}
            }
        }

        visited.insert(curr_node_id, CycleCheckStatus::Processed);

        true
    }

    // dependants are downstream
    fn add_dependant(&mut self, from_node_id: NodeId, to_node_id: NodeId) {
        self.dependants[from_node_id].insert(to_node_id);
    }

    // dependencies are upstream
    fn add_dependency(&mut self, from_node_id: NodeId, to_node_id: NodeId) {
        self.dependencies.borrow_mut()[to_node_id].insert(from_node_id);
    }

    pub(crate) fn add_relationship(&mut self, from_node_id: NodeId, to_node_id: NodeId) {
        self.add_dependant(from_node_id, to_node_id);
        self.add_dependency(from_node_id, to_node_id);
    }

    pub(crate) fn add_root_node(&mut self, node_id: NodeId) {
        self.roots.borrow_mut().insert(node_id);
    }

    fn remove_root_node(&self, node_id: NodeId) {
        self.roots.borrow_mut().remove(&node_id);
    }

    fn get_roots(&self) -> Ref<HashSet<NodeId>> {
        self.roots.borrow()
    }

    fn get_next_root(&self) -> Option<NodeId> {
        match self.roots.borrow().iter().next() {
            Some(node_id) => Some(*node_id),
            None => None,
        }
    }

    pub(crate) fn get_dependencies(&self, node_id: NodeId) -> Ref<HashSet<NodeId>> {
        Ref::map(self.dependencies.borrow(), |vec| &vec[node_id])
    }

    // fn remove_dependency(&mut self, from_node_id: NodeId, to_node_id: NodeId) {
    //     self.dependencies[*to_node_id].remove(from_node_id);
    // }

    fn get_dependants(&self, node_id: NodeId) -> &HashSet<NodeId> {
        &self.dependants[node_id]
    }

    pub fn visited_node(&self, node: &Node<T>) {
        for id in self.dependants[node.get_id()].iter() {
            self.dependencies.borrow_mut()[*id].remove(&node.get_id());
        }

        self.remove_root_node(node.get_id());

        for id in self.dependants[node.get_id()].iter() {
            if self.dependencies.borrow_mut()[*id].is_empty() {
                self.roots.borrow_mut().insert(*id);
            }
        }
    }

    pub fn next(&self) -> Option<RwLockReadGuard<Node<T>>> {
        match self.get_next_root() {
            Some(id) => Some(self.dag.get_node(id)),
            None => None,
        }
    }
}
