use std::future::Future;
use std::marker::PhantomData;

use crate::dag::node::Node;
use crate::dag::Dag;
use crate::execution::Execution;
use crate::task::*;
use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};

type NodeDataBaseType = Box<dyn ExecutableTask>;

pub struct TaskHandle<T> {
    task_id: usize,
    data_type: PhantomData<T>,
}

impl<T> TaskHandle<T> {
    pub fn id(&self) -> usize {
        self.task_id
    }
}

pub(crate) struct TaskReadHandle<'a, T> {
    guard: RwLockReadGuard<'a, Node<NodeDataBaseType>>,
    data_type: PhantomData<T>,
}

impl<'a, T: 'static> TaskReadHandle<'a, T> {
    pub(crate) fn borrow(&self) -> &dyn ExecutableTask {
        self.guard.get_value().as_ref()
    }

    // pub(crate) fn borrow_concrete(&self) -> &T {
    //     (*self.guard)
    //         .get_value()
    //         .as_any()
    //         .downcast_ref::<T>()
    //         .unwrap()
    // }
}

pub(crate) struct TaskWriteHandle<'a, T> {
    guard: RwLockWriteGuard<'a, Node<NodeDataBaseType>>,
    data_type: PhantomData<T>,
}

impl<'a, T: 'static> TaskWriteHandle<'a, T> {
    pub(crate) fn borrow_concrete(&mut self) -> &mut T {
        (*self.guard)
            .get_mut_value()
            .as_mut_any()
            .downcast_mut::<T>()
            .unwrap()
    }
}

#[derive(Clone)]
pub struct Flow {
    dag: Dag<NodeDataBaseType>,
}

impl Flow {
    pub fn new() -> Self {
        Self { dag: Dag::new() }
    }

    pub fn add_new_task<O, T: TaskOutput0<O>>(&mut self, new_task: T) -> TaskHandle<T> {
        let id = self.dag.add_node(Box::new(new_task));
        TaskHandle {
            task_id: id,
            data_type: PhantomData,
        }
    }

    pub(crate) fn get_task_by_id<T>(&self, task_id: usize) -> TaskReadHandle<T> {
        TaskReadHandle {
            guard: self.dag.get_node(task_id),
            data_type: PhantomData,
        }
    }

    pub(crate) fn get_task<T>(&self, task_handle: &TaskHandle<T>) -> TaskReadHandle<T> {
        self.get_task_by_id(task_handle.id())
    }

    pub(crate) fn get_mut_task<T>(&self, task_handle: &TaskHandle<T>) -> TaskWriteHandle<T> {
        TaskWriteHandle {
            guard: self.dag.get_mut_node(task_handle.id()),
            data_type: PhantomData,
        }
    }

    fn connect<I, O, A: TaskOutput0<O>, B: TaskInput0<I>, T: 'static>(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task1_output: fn(&dyn ExecutableTask) -> Option<&T>,
        task2_handle: &TaskHandle<B>,
        task2_input: fn(&mut B, TaskInputHandle<T>),
    ) {
        (task2_input)(
            self.get_mut_task(task2_handle).borrow_concrete(),
            TaskInputHandle::new(task1_handle.id(), task1_output),
        );
        self.dag.connect(task1_handle.id(), task2_handle.id());
    }

    #[cfg(not(feature = "manual_task_ifaces"))]
    rs_taskflow_derive::generate_connect_tasks_funcs!(10);
    #[cfg(feature = "manual_task_ifaces")]
    pub fn connect_output0_to_input0<T: 'static, A: TaskOutput0<T>, B: TaskInput0<T>>(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task2_handle: &TaskHandle<B>,
    ) {
        self.connect(task1_handle, A::get_output_0, task2_handle, B::set_input_0);
    }
    #[cfg(feature = "manual_task_ifaces")]
    pub fn connect_output0_to_input1<I0, T: 'static, A: TaskOutput0<T>, B: TaskInput1<I0, T>>(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task2_handle: &TaskHandle<B>,
    ) {
        self.connect(task1_handle, A::get_output_0, task2_handle, B::set_input_1);
    }
    #[cfg(feature = "manual_task_ifaces")]
    pub fn connect_output1_to_input0<O0, T: 'static, A: TaskOutput1<O0, T>, B: TaskInput0<T>>(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task2_handle: &TaskHandle<B>,
    ) {
        self.connect(task1_handle, A::get_output_1, task2_handle, B::set_input_0);
    }
    #[cfg(feature = "manual_task_ifaces")]
    pub fn connect_output1_to_input1<
        O0,
        I0,
        T: 'static,
        A: TaskOutput1<O0, T>,
        B: TaskInput1<I0, T>,
    >(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task2_handle: &TaskHandle<B>,
    ) {
        self.connect(task1_handle, A::get_output_1, task2_handle, B::set_input_1);
    }

    pub fn get_num_tasks(&self) -> usize {
        self.dag.get_num_nodes()
    }

    pub(crate) fn get_flow_graph(&self) -> &Dag<NodeDataBaseType> {
        &self.dag
    }

    pub fn execute(&self) -> impl Future<Output = Execution> {
        let flow_copy = Arc::new(self.clone());
        let flow_exec = Execution::new(flow_copy);
        flow_exec.start_and_finish()
    }
}

impl Default for Flow {
    fn default() -> Self {
        Self::new()
    }
}
