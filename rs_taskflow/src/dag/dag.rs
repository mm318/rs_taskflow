// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cmp::Eq;
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::Send;
use std::slice::Iter;

use crate::dag::node::{Node, NodeId};
use crate::dag::visit::DagVisitationInfo;

#[derive(Eq, PartialEq, Debug)]
pub struct Dag<T: Eq + Debug> {
    nodes: Vec<Node<T>>,
    dependencies: Vec<HashSet<usize>>,
}

impl<T: Eq + Debug> Dag<T> {
    pub fn new() -> Self {
        Dag {
            nodes: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_node(&mut self, value: T) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node::new(id, value));
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

    pub fn get_node(&self, node_id: NodeId) -> &Node<T> {
        &self.nodes[node_id]
    }

    pub fn get_mut_node(&mut self, node_id: NodeId) -> &mut Node<T> {
        &mut self.nodes[node_id]
    }

    pub fn iter_nodes(&self) -> Iter<'_, Node<T>> {
        self.nodes.iter()
    }

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
            if bfs.get_dependencies(node.get_id()).is_empty() {
                bfs.add_root_node(node.get_id());
            }
        }

        bfs.check()
    }
}

impl<T: Eq + Debug> Default for Dag<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Eq + Debug> Send for Dag<T> {}

#[cfg(test)]
mod tests {
    use crate::dag::node::Node;
    use crate::dag::Dag;
    use std::collections::HashSet;

    #[derive(Hash, Eq, PartialEq, Debug)]
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

        bfs.visited_node(dag.get_node(a));

        assert!(
            bfs.get_dependencies(b).is_empty(),
            "Node was not successfully removed"
        );
    }

    #[test]
    fn node_hash() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('a'));
        let c = dag.add_node(MockStruct::new('C'));
        let d = dag.add_node(MockStruct::new('D'));
        let e = dag.add_node(MockStruct::new('E'));
        let f = dag.add_node(MockStruct::new('F'));
        let g = dag.add_node(MockStruct::new('G'));
        let h = dag.add_node(MockStruct::new('H'));

        let mut hash: HashSet<&Node<MockStruct>> = HashSet::new();

        hash.insert(dag.get_node(a));
        hash.insert(dag.get_node(b));
        hash.insert(dag.get_node(c));
        hash.insert(dag.get_node(d));
        hash.insert(dag.get_node(e));
        hash.insert(dag.get_node(f));
        hash.insert(dag.get_node(g));
        hash.insert(dag.get_node(h));

        assert!(hash.contains(dag.get_node(a)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(b)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(c)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(d)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(e)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(f)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(g)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(h)), "Node did not hash properly");
    }
}
