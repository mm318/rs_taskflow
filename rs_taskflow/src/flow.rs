use std::any::Any;
use std::marker::PhantomData;

use crate::dag::Dag;
use crate::execution::Execution;
use crate::task::*;
use std::sync::Arc;

// #[derive(Copy, Clone)]
pub struct TaskHandle<T> {
    task_id: usize,
    data_type: PhantomData<T>,
}

impl<T> TaskHandle<T> {
    pub fn id(&self) -> usize {
        self.task_id
    }
}

pub struct Flow {
    dag: Dag<Box<dyn ExecutableTask>>,
    // connections_setup: Vec<Box<dyn Fn(&mut Flow)>>
}

impl Flow {
    pub fn new() -> Self {
        Flow {
            dag: Dag::new(),
            // connections_setup: Vec::new(),
        }
    }

    pub fn add_new_task<O, T: TaskOutput0<O>>(&mut self, new_task: T) -> TaskHandle<T> {
        let id = self.dag.add_node(Box::new(new_task));
        TaskHandle {
            task_id: id,
            data_type: PhantomData,
        }
    }

    pub fn get_task_by_id(&self, task_id: usize) -> &dyn ExecutableTask {
        return &**self.dag.get_node(task_id).get_value();
    }

    pub fn get_task<T>(&self, task_handle: &TaskHandle<T>) -> &dyn ExecutableTask {
        self.get_task_by_id(task_handle.id())
    }

    // fn get_concrete_task<B: Any>(&self, task_handle: &TaskHandle<B>) -> &B {
    //     return self
    //         .get_task(task_handle)
    //         .as_any()
    //         .downcast_ref::<B>()
    //         .unwrap();
    // }

    fn get_mut_concrete_task<T: Any>(&mut self, task_handle: &TaskHandle<T>) -> &mut T {
        return self
            .dag
            .get_mut_node(task_handle.id())
            .get_mut_value()
            .as_mut_any()
            .downcast_mut::<T>()
            .unwrap();
    }

    fn connect<I, O, A: TaskOutput0<O>, B: TaskInput0<I>, T: 'static>(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task1_output: fn(&dyn ExecutableTask) -> T,
        task2_handle: &TaskHandle<B>,
        task2_input: fn(&mut B, TaskInputHandle<T>),
    ) {
        (task2_input)(
            self.get_mut_concrete_task(task2_handle),
            TaskInputHandle::new(task1_handle.id(), task1_output),
        );
        self.dag.connect(task1_handle.id(), task2_handle.id());

        // let task1_handle_copy = *task1_handle;
        // let task2_handle_copy = *task2_handle;
        // let connection_setup = move |flow: &mut Flow| (task2_input)(
        //     flow.get_mut_concrete_task(&task2_handle_copy),
        //     TaskInputHandle::new(task1_handle_copy.id(), task1_output),
        // );
        // self.connections_setup.push(Box::new(connection_setup));
    }

    rs_taskflow_derive::generate_connect_tasks_funcs!(10);

    pub fn get_num_tasks(&self) -> usize {
        self.dag.get_num_nodes()
    }

    pub(crate) fn get_flow_graph(&self) -> &Dag<Box<dyn ExecutableTask>> {
        &self.dag
    }

    pub fn new_execution(self: Arc<Flow>) -> Execution {
        Execution::new(self)
    }
}

impl Default for Flow {
    fn default() -> Self {
        Self::new()
    }
}
