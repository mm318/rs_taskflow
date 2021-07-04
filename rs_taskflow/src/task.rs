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

pub trait ExecutableTask: AsAny + Sync + Send + Debug {
    fn exec(&self, flow: &Flow);
}

impl PartialEq for dyn ExecutableTask {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for dyn ExecutableTask {}

rs_taskflow_derive::generate_task_input_iface_traits!(TaskInput, set_input, 1);
rs_taskflow_derive::generate_task_output_iface_traits!(TaskOutput, get_output, 1);

pub struct TaskInputHandle<T> {
    source_task_id: usize,
    value_func: fn(&dyn ExecutableTask) -> T,
}

impl<T> TaskInputHandle<T> {
    pub fn new(id: usize, func: fn(&dyn ExecutableTask) -> T) -> Self {
        TaskInputHandle {
            source_task_id: id,
            value_func: func,
        }
    }

    pub fn set(&mut self, id: usize, func: fn(&dyn ExecutableTask) -> T) {
        self.source_task_id = id;
        self.value_func = func;
    }

    pub fn get_value(&self, flow: &Flow) -> T {
        return (self.value_func)(flow.get_task_by_id(self.source_task_id));
    }
}

impl<T> Debug for TaskInputHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskInputHandle")
            .field("source_task_id", &self.source_task_id)
            .field(
                "value_func",
                &format_args!("{:p}", self.value_func as *const ()),
            )
            .finish()
    }
}
