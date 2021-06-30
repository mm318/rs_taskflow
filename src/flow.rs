use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::task::{Context, Poll};

use tokio::task;
use tokio::task::JoinHandle;

use crate::dag::Dag;
use crate::task::{ExecutableTask, TaskInput};

pub trait TaskType {
    type TaskType;
}

pub struct TaskHandle<T> {
    task_id: usize,
    data_type: PhantomData<T>,
}

impl<T> TaskType for TaskHandle<T> {
    type TaskType = T;
}

impl<T> TaskHandle<T> {
    pub fn id(&self) -> usize {
        self.task_id
    }
}

pub struct Flow {
    dag: Dag<Box<dyn ExecutableTask>>,
}

struct ExecTaskJoinHandle {
    join_handle: JoinHandle<()>,
    completed: bool,
}

struct ExecTask {
    lock: Mutex<Option<ExecTaskJoinHandle>>,
    condvar: Condvar,
}

impl ExecTask {
    fn new() -> Self {
        ExecTask {
            lock: Mutex::new(Option::None),
            condvar: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<Option<ExecTaskJoinHandle>> {
        return self.lock.lock().unwrap();
    }

    fn notify(&self) {
        self.condvar.notify_all();
    }
}

struct ExecTaskFuture {
    flow: Arc<Flow>,
    node_id: usize,
    futures: Arc<Vec<ExecTask>>,
}

impl Future for ExecTaskFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for from_node_id in self.flow.dag.get_dependencies(&self.node_id) {
            if cfg!(debug_assertions) {
                match *self.futures[*from_node_id].lock() {
                    Option::Some(_) => {}
                    Option::None => {
                        println!("Node {} is missing a future / join handle!", *from_node_id)
                    }
                }
            }

            let mut locked_guard = self.futures[*from_node_id].lock();
            let join_handle = locked_guard.as_mut().unwrap();
            if join_handle.completed {
                continue;
            } else {
                match Pin::new(&mut join_handle.join_handle).poll(cx) {
                    Poll::Ready(_) => continue,
                    Poll::Pending => return Poll::Pending,
                }
            }
        }

        self.flow
            .dag
            .get_node(self.node_id)
            .get_value()
            .exec(self.flow.as_ref());

        self.futures[self.node_id]
            .lock()
            .as_mut()
            .unwrap()
            .completed = true;
        self.futures[self.node_id].notify();

        Poll::Ready(())
    }
}

impl Flow {
    pub fn new() -> Self {
        Flow { dag: Dag::new() }
    }

    pub fn new_task<B: ExecutableTask>(&mut self, new_task: B) -> TaskHandle<B> {
        let id = self.dag.add_node(Box::new(new_task));
        TaskHandle {
            task_id: id,
            data_type: PhantomData,
        }
    }

    pub fn get_task_by_id(&self, task_id: usize) -> &dyn ExecutableTask {
        return &**self.dag.get_node(task_id).get_value();
    }

    pub fn get_task<B>(&self, task_handle: &TaskHandle<B>) -> &dyn ExecutableTask {
        self.get_task_by_id(task_handle.id())
    }

    // fn get_concrete_task<B: Any>(&self, task_handle: &TaskHandle<B>) -> &B {
    //     return self
    //         .get_task(task_handle)
    //         .as_any()
    //         .downcast_ref::<B>()
    //         .unwrap();
    // }

    fn get_mut_concrete_task<B: Any>(&mut self, task_handle: &TaskHandle<B>) -> &mut B {
        return self
            .dag
            .get_mut_node(task_handle.id())
            .get_mut_value()
            .as_mut_any()
            .downcast_mut::<B>()
            .unwrap();
    }

    pub fn connect<A: ExecutableTask, B: ExecutableTask, T: Default>(
        &mut self,
        task1_handle: &TaskHandle<A>,
        task1_output: fn(&dyn ExecutableTask) -> T,
        task2_handle: &TaskHandle<B>,
        task2_input: fn(&mut B, TaskInput<T>),
    ) {
        (task2_input)(
            self.get_mut_concrete_task(task2_handle),
            TaskInput::new(task1_handle.id(), task1_output),
        );
        self.dag.connect(task1_handle.id(), task2_handle.id());
    }

    fn spawn_exec_task(self: Arc<Flow>, node_id: usize, futures: Arc<Vec<ExecTask>>) {
        // if cfg!(debug_assertions) {
        //     println!("Adding future for node {}", node_id);
        // }

        let future_task = task::spawn(ExecTaskFuture {
            flow: self,
            node_id,
            futures: futures.clone(),
        });
        *futures[node_id].lock() = Option::Some(ExecTaskJoinHandle {
            join_handle: future_task,
            completed: false,
        });

        // if cfg!(debug_assertions) {
        //     match *futures.clone()[node_id].lock() {
        //         Option::Some(_) => println!("Node {} has a future!", node_id),
        //         Option::None => println!("Node {} has no future!", node_id),
        //     }
        // }
    }

    pub async fn start(self: Arc<Flow>) {
        let mut futures_vec = Vec::<ExecTask>::with_capacity(self.dag.get_num_nodes());

        // futures_vec.resize(self.dag.get_num_nodes(), Mutex::new(Option::None));
        for _ in 0..self.dag.get_num_nodes() {
            futures_vec.push(ExecTask::new());
        }

        // each future will have a copy of this Arc
        let futures_vec_arc = Arc::new(futures_vec);

        let mut bfs = self.dag.build_bfs().unwrap();
        while let Some(ref node) = self.dag.next_in_bfs(&bfs) {
            if cfg!(debug_assertions) {
                println!("  Visiting {:?}", node);
            }

            self.dag.visited_in_bfs(&mut bfs, node);

            let self_copy = self.clone();
            let node_id = *node.get_id();
            let futures_vec_copy = futures_vec_arc.clone();
            self_copy.spawn_exec_task(node_id, futures_vec_copy);
        }

        for future_iter in futures_vec_arc.iter() {
            let mut locked_guard = future_iter.lock();
            while !locked_guard.as_ref().unwrap().completed {
                locked_guard = future_iter.condvar.wait(locked_guard).unwrap();
            }
        }
    }
}

impl Default for Flow {
    fn default() -> Self {
        Self::new()
    }
}
