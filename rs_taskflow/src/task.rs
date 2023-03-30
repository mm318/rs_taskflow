use std::fmt::Debug;
use std::marker::Send;

use crate::flow::Flow;
use crate::task::private::AsAny;

mod private {
    use std::any::Any;

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
}

pub trait ExecutableTask: AsAny + Sync + Send + Debug {
    // type TaskType = Self;
    fn exec(&mut self, flow: &Flow);
}

impl PartialEq for dyn ExecutableTask {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for dyn ExecutableTask {}

#[cfg(not(feature = "manual_task_ifaces"))]
rs_taskflow_derive::generate_task_input_iface_traits!(TaskInput, set_input, 10);
#[cfg(not(feature = "manual_task_ifaces"))]
rs_taskflow_derive::generate_task_output_iface_traits!(TaskOutput, get_output, 10);
#[cfg(feature = "manual_task_ifaces")]
pub trait TaskInput0<I0>: ExecutableTask {
    fn set_input_0(&mut self, task_input: TaskInputHandle<I0>);
}
#[cfg(feature = "manual_task_ifaces")]
pub trait TaskInput1<I0, I1>: TaskInput0<I0> {
    fn set_input_1(&mut self, task_input: TaskInputHandle<I1>);
}
#[cfg(feature = "manual_task_ifaces")]
pub trait TaskOutput0<O0>: ExecutableTask {
    fn get_output_0(task: &dyn ExecutableTask) -> &O0;
}
#[cfg(feature = "manual_task_ifaces")]
pub trait TaskOutput1<O0, O1>: TaskOutput0<O0> {
    fn get_output_1(task: &dyn ExecutableTask) -> &O1;
}

pub struct TaskInputHandle<T> {
    source_task_id: usize,
    value_func: fn(&dyn ExecutableTask) -> &T,
}

impl<T> TaskInputHandle<T> {
    pub fn new(id: usize, func: fn(&dyn ExecutableTask) -> &T) -> Self {
        TaskInputHandle {
            source_task_id: id,
            value_func: func,
        }
    }

    pub fn set(&mut self, id: usize, func: fn(&dyn ExecutableTask) -> &T) {
        self.source_task_id = id;
        self.value_func = func;
    }

    pub fn get_value<'a, 'b>(&'a self, flow: &'b Flow) -> &'b T {
        let task_handle = flow.get_task_by_id(self.source_task_id);
        let val_ref = (self.value_func)(task_handle.borrow());
        let val_ptr: *const T = val_ref;
        unsafe {
            &*val_ptr
        }
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
