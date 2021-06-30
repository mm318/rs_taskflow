// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Eq)]
pub struct Node<T: Eq + Debug> {
    id: usize,
    value: T,
}

impl<T: Eq + Debug> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.value == other.value
    }
}

impl<T: Eq + Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Node {}: {:?})", self.id, self.value)
    }
}

impl<T: Eq + Debug> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Eq + Debug> Node<T> {
    pub fn new(i: usize, v: T) -> Node<T> {
        Node { id: i, value: v }
    }

    pub fn get_id(&self) -> &usize {
        return &self.id;
    }

    pub fn get_value(&self) -> &T {
        return &self.value;
    }

    pub fn get_mut_value(&mut self) -> &mut T {
        return &mut self.value;
    }
}
