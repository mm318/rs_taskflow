// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cmp::Eq;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::dag::node::{Node, NodeId};
use crate::dag::visit::DagVisitationInfo;

pub struct Dag<T: Eq + Clone> {
    nodes: Vec<RwLock<Node<T>>>,
    dependencies: Vec<HashSet<usize>>,
}

impl<T: Eq + Clone> Dag<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_node(&mut self, value: T) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(RwLock::new(Node::new(id, value)));
        self.dependencies.push(HashSet::new());
        id
    }

    pub fn connect(&mut self, from_node_id: usize, to_node_id: usize) {
        self.dependencies[to_node_id].insert(from_node_id);
    }

    pub fn get_num_nodes(&self) -> usize {
        assert_eq!(self.nodes.len(), self.dependencies.len());
        self.nodes.len()
    }

    pub fn get_node(&self, node_id: NodeId) -> RwLockReadGuard<Node<T>> {
        self.nodes[node_id].read().unwrap()
    }

    pub fn get_mut_node(&self, node_id: NodeId) -> RwLockWriteGuard<Node<T>> {
        self.nodes[node_id].write().unwrap()
    }

    // pub fn iter_nodes(&self) -> Iter<'_, Node<T>> {
    //     self.nodes.iter()
    // }

    pub fn get_dependencies(&self, node_id: NodeId) -> &HashSet<usize> {
        &self.dependencies[node_id]
    }

    // find roots
    pub fn build_bfs(&self) -> Result<DagVisitationInfo<T>, &str> {
        let mut bfs = DagVisitationInfo::new(self);

        for (to_node_id, deps) in self.dependencies.iter().enumerate() {
            for from_node_id in deps {
                bfs.add_relationship(*from_node_id, to_node_id);
            }
        }

        for node in &self.nodes {
            let node_id = node.read().unwrap().get_id();
            if bfs.get_dependencies(node_id).is_empty() {
                bfs.add_root_node(node_id);
            }
        }

        bfs.check()
    }

    fn copy_nodes(source: &Self) -> Vec<RwLock<Node<T>>> {
        let mut vec_copy = Vec::with_capacity(source.nodes.len());
        for node in &source.nodes {
            vec_copy.push(RwLock::new(node.read().unwrap().clone()))
        }
        vec_copy
    }
}

impl<T: Eq + Clone> Clone for Dag<T> {
    fn clone(&self) -> Self {
        Self {
            nodes: Dag::copy_nodes(self),
            dependencies: self.dependencies.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.nodes = Dag::copy_nodes(source);
        self.dependencies = source.dependencies.clone();
    }
}

impl<T: Eq + Clone> Default for Dag<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::dag::Dag;

    #[derive(Hash, Clone, Eq, PartialEq, Debug)]
    struct MockStruct {
        id: char,
    }

    impl MockStruct {
        fn new(id: char) -> MockStruct {
            MockStruct { id }
        }
    }

    #[test]
    fn build_dag() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('a'));
        let c = dag.add_node(MockStruct::new('C'));
        let d = dag.add_node(MockStruct::new('D'));
        let e = dag.add_node(MockStruct::new('E'));
        let f = dag.add_node(MockStruct::new('F'));
        let g = dag.add_node(MockStruct::new('G'));
        let h = dag.add_node(MockStruct::new('H'));

        dag.connect(a, b);
        dag.connect(b, c);
        dag.connect(c, d);
        dag.connect(d, e);
        dag.connect(d, f);
        dag.connect(f, g);
        dag.connect(f, h);

        let bfs = dag.build_bfs();
        assert!(bfs.is_ok());
    }

    #[test]
    fn build_dag_with_circular_dependency() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('a'));
        let c = dag.add_node(MockStruct::new('C'));
        let d = dag.add_node(MockStruct::new('D'));
        let e = dag.add_node(MockStruct::new('E'));
        let f = dag.add_node(MockStruct::new('F'));
        let g = dag.add_node(MockStruct::new('G'));
        let h = dag.add_node(MockStruct::new('H'));

        dag.connect(a, b);
        dag.connect(b, c);
        dag.connect(c, d);
        dag.connect(d, e);
        dag.connect(d, f);
        dag.connect(f, g);
        dag.connect(f, h);
        dag.connect(d, b); // causes circular dependency

        let bfs = dag.build_bfs();
        assert!(bfs.is_err());
    }

    #[test]
    fn remove_nodes() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('B'));

        dag.connect(a, b);

        let bfs = dag.build_bfs().unwrap();

        assert!(
            !bfs.get_dependencies(b).is_empty(),
            "Node was not successfully removed"
        );

        bfs.visited_node(&*dag.get_node(a));

        assert!(
            bfs.get_dependencies(b).is_empty(),
            "Node was not successfully removed"
        );
    }
}
