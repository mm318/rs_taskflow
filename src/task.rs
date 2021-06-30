use std::any::Any;
use std::fmt::Debug;
use std::marker::Send;

use crate::flow::Flow;

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Task: AsAny + Sync + Send + Debug {
    fn exec(&self, flow: &Flow);
}

impl PartialEq for dyn Task {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for dyn Task {}

pub struct TaskInput<T> {
    task_id: usize,
    value_func: fn(&dyn Task) -> T,
}

impl<T> TaskInput<T> {
    pub fn new(id: usize, func: fn(&dyn Task) -> T) -> Self {
        TaskInput {
            task_id: id,
            value_func: func,
        }
    }

    pub fn set(&mut self, id: usize, func: fn(&dyn Task) -> T) {
        self.task_id = id;
        self.value_func = func;
    }

    pub fn get_value(&self, flow: &Flow) -> T {
        return (self.value_func)(flow.get_task_by_id(self.task_id));
    }
}

impl<T> Debug for TaskInput<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskInput")
            .field("task_id", &self.task_id)
            .field(
                "value_func",
                &format_args!("{:p}", self.value_func as *const ()),
            )
            .finish()
    }
}
