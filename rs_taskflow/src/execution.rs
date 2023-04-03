use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

use tokio::task;
use tokio::task::JoinHandle;

use crate::dag::node::NodeId;
use crate::flow::{Flow, TaskHandle};
use crate::task::*;

struct ExecTask {
    join_handle: Mutex<Option<JoinHandle<()>>>,
    completed: AtomicBool,
    condvar: Condvar,
}

impl ExecTask {
    fn new() -> Self {
        Self {
            join_handle: Mutex::new(None),
            completed: AtomicBool::new(false),
            condvar: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<Option<JoinHandle<()>>> {
        return self.join_handle.lock().unwrap();
    }

    fn notify(&self) {
        self.condvar.notify_all();
    }
}

struct ExecTaskFuture {
    flow: Arc<Flow>,
    node_id: NodeId,
    futures: Arc<Vec<ExecTask>>,
}

impl Future for ExecTaskFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for from_node_id in self.flow.get_flow_graph().get_dependencies(self.node_id) {
            if self.futures[*from_node_id].completed.load(Relaxed) {
                continue;
            }

            if cfg!(debug_assertions) {
                println!("{:?} Visiting node id {} checking dependent node id {}", thread::current().id(), self.node_id, *from_node_id);
            }
            match Pin::new(&mut self.futures[*from_node_id].lock().as_mut().unwrap()).poll(cx) {
                Poll::Ready(_) => {
                    continue;
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }

        if cfg!(debug_assertions) {
            println!("{:?} Visiting node id {} ready", thread::current().id(), self.node_id);
        }

        self.flow
            .get_flow_graph()
            .get_mut_node(self.node_id)
            .get_mut_value()
            .exec(self.flow.as_ref());

        self.futures[self.node_id].completed.store(true, Relaxed);
        self.futures[self.node_id].notify();

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

    fn spawn_exec_task(&self, node_id: usize, futures: &Arc<Vec<ExecTask>>) {
        // if cfg!(debug_assertions) {
        //     println!("Adding future for node {}", node_id);
        // }

        let join_handle = task::spawn(ExecTaskFuture {
            flow: self.flow.clone(),
            node_id,
            futures: futures.clone(),
        });
        *futures[node_id].lock() = Some(join_handle);

        // if cfg!(debug_assertions) {
        //     match *futures.clone()[node_id].lock() {
        //         Some(_) => println!("Node {} has a future!", node_id),
        //         None => println!("Node {} has no future!", node_id),
        //     }
        // }
    }

    pub async fn start_and_finish(self) -> Self {
        let len = self.flow.get_num_tasks();

        let mut task_execs = Vec::<ExecTask>::with_capacity(len);
        for _ in 0..len {
            task_execs.push(ExecTask::new());
        }

        // each future will have a copy of this Arc
        let task_execs_arc = Arc::new(task_execs);

        let bfs = self.flow.get_flow_graph().build_bfs().unwrap();
        while let Some(node) = bfs.next() {
            bfs.visited_node(&*node);
            self.spawn_exec_task(node.get_id(), &task_execs_arc);
        }

        for (id, task_exec) in task_execs_arc.iter().enumerate() {
            while !task_exec.completed.load(Relaxed) {
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
