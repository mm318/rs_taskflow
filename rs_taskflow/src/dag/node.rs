// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub type NodeId = usize;

#[derive(Eq, Clone)]
pub struct Node<T: Eq + Clone + Debug> {
    id: NodeId,
    value: T,
}

impl<T: Eq + Clone + Debug> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.value == other.value
    }
}

impl<T: Eq + Clone + Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Node {}: {:?})", self.id, self.value)
    }
}

impl<T: Eq + Clone + Debug> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Eq + Clone + Debug> Node<T> {
    pub fn new(i: NodeId, v: T) -> Node<T> {
        Node { id: i, value: v }
    }

    pub fn get_id(&self) -> NodeId {
        self.id
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn get_mut_value(&mut self) -> &mut T {
        &mut self.value
    }
}
