use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

use tokio::task;
use tokio::task::JoinHandle;

use crate::dag::node::NodeId;
use crate::flow::{Flow, TaskHandle};
use crate::task::*;

struct ExecTask {
    waker: Mutex<Option<Waker>>,
    completed: AtomicBool,
}

impl ExecTask {
    fn new() -> Self {
        Self {
            waker: Mutex::new(None),
            completed: AtomicBool::new(false),
        }
    }

    fn get_waker(&self) -> MutexGuard<Option<Waker>> {
        self.waker.lock().unwrap()
    }

    fn is_completed(&self) -> bool {
        self.completed.load(Relaxed)
    }

    fn set_completed(&self) {
        self.completed.store(true, Relaxed)
    }
}

struct ExecTaskFuture {
    flow: Arc<Flow>,
    node_id: NodeId,
    task_execs: Arc<Vec<ExecTask>>,
}

impl Future for ExecTaskFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if cfg!(debug_assertions) {
            println!(
                "{:?} Visiting node id {} (waker: {:?})",
                thread::current().id(),
                self.node_id,
                cx.waker()
            );
        }
        *self.task_execs[self.node_id].get_waker() = Some(cx.waker().clone());

        for dep_node_id in self.flow.get_flow_graph().get_dependencies(self.node_id) {
            if self.task_execs[*dep_node_id].is_completed() {
                continue;
            } else {
                return Poll::Pending;
            }
        }

        self.flow
            .get_flow_graph()
            .get_mut_node(self.node_id)
            .get_mut_value()
            .exec(self.flow.as_ref());

        self.task_execs[self.node_id].set_completed();

        for dep_node_id in self.flow.get_flow_graph().get_dependants(self.node_id) {
            if let Some(waker) = self.task_execs[*dep_node_id].get_waker().take() {
                if cfg!(debug_assertions) {
                    println!(
                        "{:?} Visited node id {} (waking: {:?})",
                        thread::current().id(),
                        self.node_id,
                        waker
                    );
                }
                waker.wake();
            }
        }

        Poll::Ready(())
    }
}

pub struct Execution {
    flow: Arc<Flow>, // parent Flow object
}

impl Execution {
    pub(crate) fn new(flow: Arc<Flow>) -> Execution {
        Execution { flow: flow }
    }

    fn spawn_exec_task(
        &self,
        node_id: NodeId,
        task_execs_ref: &Arc<Vec<ExecTask>>,
    ) -> JoinHandle<()> {
        if cfg!(debug_assertions) {
            println!("Spawning task for node id {}", node_id);
        }

        task::spawn(ExecTaskFuture {
            flow: self.flow.clone(),
            node_id,
            task_execs: task_execs_ref.clone(),
        })
    }

    pub async fn start_and_finish(self) -> Self {
        let len = self.flow.get_num_tasks();
        let mut task_execs_vec = Vec::<ExecTask>::with_capacity(len);
        for _ in 0..len {
            task_execs_vec.push(ExecTask::new());
        }
        let task_execs = Arc::new(task_execs_vec);

        let mut join_handles = Vec::<JoinHandle<()>>::with_capacity(len);
        let bfs = self.flow.get_flow_graph().build_bfs().unwrap();
        while let Some(node) = bfs.next() {
            bfs.visited_node(&*node);
            let join_handle = self.spawn_exec_task(node.get_id(), &task_execs);
            join_handles.push(join_handle);
        }

        for (node_id, join_handle) in join_handles.into_iter().enumerate() {
            if !task_execs[node_id].is_completed() {
                let result = join_handle.await;
                assert!(result.is_ok());
            }
        }

        self
    }

    #[cfg(not(feature = "manual_task_ifaces"))]
    rs_taskflow_derive::generate_get_task_output_funcs!(10);
    #[cfg(feature = "manual_task_ifaces")]
    pub fn get_task_output0<O: 'static, T: TaskOutput0<O>>(
        &self,
        task_handle: &TaskHandle<T>,
    ) -> Option<&O> {
        let read_handle = self.flow.get_task(task_handle);
        let val_ref = T::get_output_0(read_handle.borrow());
        let val_ptr: *const O = val_ref.unwrap();
        unsafe { Some(&*val_ptr) }
    }
    #[cfg(feature = "manual_task_ifaces")]
    pub fn get_task_output1<O0, O: 'static, T: TaskOutput1<O0, O>>(
        &self,
        task_handle: &TaskHandle<T>,
    ) -> Option<&O> {
        let read_handle = self.flow.get_task(task_handle);
        let val_ref = T::get_output_1(read_handle.borrow());
        let val_ptr: *const O = val_ref.unwrap();
        unsafe { Some(&*val_ptr) }
    }
}
