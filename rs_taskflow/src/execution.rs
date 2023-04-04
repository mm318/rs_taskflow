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
    join_handle: Mutex<Option<JoinHandle<()>>>,
    waker: Mutex<Option<Waker>>,
    completed: AtomicBool,
}

impl ExecTask {
    fn new() -> Self {
        Self {
            join_handle: Mutex::new(None),
            waker: Mutex::new(None),
            completed: AtomicBool::new(false)
        }
    }

    fn get_join_handle(&self) -> MutexGuard<Option<JoinHandle<()>>> {
        self.join_handle.lock().unwrap()
    }

    fn get_waker(&self) -> MutexGuard<Option<Waker>> {
        self.waker.lock().unwrap()
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
        for dep_node_id in self.flow.get_flow_graph().get_dependencies(self.node_id) {
            if self.task_execs[*dep_node_id].completed.load(Relaxed) {
                continue;
            }

            if cfg!(debug_assertions) {
                println!(
                    "{:?} Visiting node id {} (waker: {:?}) checking dependent node id {}",
                    thread::current().id(),
                    self.node_id,
                    cx.waker(),
                    *dep_node_id,
                );
            }
            match Pin::new(
                self.task_execs[*dep_node_id]
                    .get_join_handle()
                    .as_mut()
                    .unwrap(),
            )
            .poll(cx)
            {
                Poll::Ready(_) => {
                    continue;
                }
                Poll::Pending => {
                    *self.task_execs[*dep_node_id].get_waker() = Some(cx.waker().clone());
                    return Poll::Pending;
                }
            }
        }

        self.flow
            .get_flow_graph()
            .get_mut_node(self.node_id)
            .get_mut_value()
            .exec(self.flow.as_ref());

        self.task_execs[self.node_id].completed.store(true, Relaxed);

        if let Some(waker) = self.task_execs[self.node_id].get_waker().take() {
            if cfg!(debug_assertions) {
                println!(
                    "{:?} Visited node id {} (waking: {:?})",
                    thread::current().id(),
                    self.node_id,
                    waker
                );
            }
            waker.wake();
        } else {
            if cfg!(debug_assertions) {
                println!(
                    "{:?} Visited node id {}",
                    thread::current().id(),
                    self.node_id,
                );
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

    fn spawn_exec_task(&self, node_id: usize, task_execs_ref: &Arc<Vec<ExecTask>>) {
        if cfg!(debug_assertions) {
            println!("Spawning task for node id {}", node_id);
        }

        let join_handle = task::spawn(ExecTaskFuture {
            flow: self.flow.clone(),
            node_id,
            task_execs: task_execs_ref.clone(),
        });

        *task_execs_ref[node_id].get_join_handle() = Some(join_handle);
    }

    pub async fn start_and_finish(self) -> Self {
        let len = self.flow.get_num_tasks();
        let mut task_execs_vec = Vec::<ExecTask>::with_capacity(len);
        for _ in 0..len {
            task_execs_vec.push(ExecTask::new());
        }
        let task_execs = Arc::new(task_execs_vec);

        let bfs = self.flow.get_flow_graph().build_bfs().unwrap();
        while let Some(node) = bfs.next() {
            bfs.visited_node(&*node);
            self.spawn_exec_task(node.get_id(), &task_execs);
        }

        let bfs = self.flow.get_flow_graph().build_bfs().unwrap();
        while let Some(node) = bfs.next() {
            bfs.visited_node(&*node);
            let id = node.get_id();
            while !task_execs[id].completed.load(Relaxed) {
                if cfg!(debug_assertions) {
                    println!("{:?} Waiting on node id {}", thread::current().id(), id);
                }

                // let result = task_exec
                //     .condvar
                //     .wait_timeout(task_exec.lock(), Duration::from_secs(1))
                //     .unwrap();
                // if result.1.timed_out() {
                //     continue;
                // } else {
                //     break;
                // }

                thread::sleep(Duration::from_secs(1));
            }
            if cfg!(debug_assertions) {
                println!("{:?} Node id {} is complete", thread::current().id(), id);
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
