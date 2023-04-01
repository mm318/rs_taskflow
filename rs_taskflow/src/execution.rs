use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::task::{Context, Poll};

use tokio::task;
use tokio::task::JoinHandle;

use crate::dag::node::NodeId;
use crate::flow::{Flow, TaskHandle, TaskReadHandle};

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
        Self {
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
    node_id: NodeId,
    futures: Arc<Vec<ExecTask>>,
}

impl Future for ExecTaskFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for from_node_id in self.flow.get_flow_graph().get_dependencies(self.node_id) {
            if cfg!(debug_assertions) && self.futures[*from_node_id].lock().is_none() {
                println!("Node {} is missing a future / join handle!", *from_node_id)
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
            .get_flow_graph()
            .get_mut_node(self.node_id)
            .get_mut_value()
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

pub struct Execution {
    flow: Arc<Flow>, // parent Flow object
}

impl Execution {
    pub(crate) fn new(flow: Arc<Flow>) -> Execution {
        Execution { flow: flow }
    }

    fn spawn_exec_task(&self, node_id: usize, futures: Arc<Vec<ExecTask>>) {
        // if cfg!(debug_assertions) {
        //     println!("Adding future for node {}", node_id);
        // }

        let task_future = task::spawn(ExecTaskFuture {
            flow: self.flow.clone(),
            node_id,
            futures: futures.clone(),
        });
        *futures[node_id].lock() = Option::Some(ExecTaskJoinHandle {
            join_handle: task_future,
            completed: false,
        });

        // if cfg!(debug_assertions) {
        //     match *futures.clone()[node_id].lock() {
        //         Option::Some(_) => println!("Node {} has a future!", node_id),
        //         Option::None => println!("Node {} has no future!", node_id),
        //     }
        // }
    }

    pub async fn start_and_finish(self) -> Self {
        let len = self.flow.get_num_tasks();

        let mut futures_vec = Vec::<ExecTask>::with_capacity(len);
        for _ in 0..len {
            futures_vec.push(ExecTask::new());
        }

        // each future will have a copy of this Arc
        let futures_vec_arc = Arc::new(futures_vec);

        let bfs = self.flow.get_flow_graph().build_bfs().unwrap();
        while let Some(node) = bfs.next() {
            if cfg!(debug_assertions) {
                println!("  Visiting node id {}", node.get_id());
            }

            bfs.visited_node(&*node);

            let node_id = node.get_id();
            let futures_vec_copy = futures_vec_arc.clone();
            self.spawn_exec_task(node_id, futures_vec_copy);
        }

        for future_iter in futures_vec_arc.iter() {
            let mut locked_guard = future_iter.lock();
            while !locked_guard.as_ref().unwrap().completed {
                locked_guard = future_iter.condvar.wait(locked_guard).unwrap();
            }
        }

        self
    }

    pub fn get_task<T>(&self, task_handle: &TaskHandle<T>) -> TaskReadHandle<T> {
        self.flow.get_task(task_handle)
    }
}
