use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::{Task, TaskInput};

#[derive(Debug)]
pub struct DefaultTask {
    input_getter: Option<TaskInput<i32>>,
    output: AtomicI32,
}

impl DefaultTask {
    pub fn new(initial_value: i32) -> Self {
        return Self {
            input_getter: Option::None,
            output: AtomicI32::new(initial_value),
        };
    }

    fn set_input(&mut self, task_input: TaskInput<i32>) {
        self.input_getter = Option::Some(task_input);
    }

    fn get_output(task: &dyn Task) -> i32 {
        return task
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output
            .load(Ordering::Relaxed);
    }
}

impl Task for DefaultTask {
    fn exec(&self, flow: &Flow) {
        match &self.input_getter {
            Option::Some(get_input) => {
                let input = get_input.get_value(flow);
                self.output.store(input, Ordering::Relaxed);
            }
            Option::None => {
                // no-op
            }
        }
    }
}

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let mut flow = Flow::new();

    //
    // create system components
    //
    let task1_handle = flow.new_task(DefaultTask::new(1));
    let task2_handle = flow.new_task(DefaultTask::new(2));
    let input_task_handle = flow.new_task(DefaultTask::new(42));

    //
    // hook up system components
    //
    if cfg!(debug_assertions) {
        println!("Connecting dependent tasks");
    }
    flow.connect(
        &input_task_handle,
        DefaultTask::get_output,
        &task1_handle,
        DefaultTask::set_input,
    );
    flow.connect(
        &task1_handle,
        DefaultTask::get_output,
        &task2_handle,
        DefaultTask::set_input,
    );

    //
    // run the system for 3 time steps
    //
    if cfg!(debug_assertions) {
        println!("Executing model");
    }
    let flow_arc = Arc::new(flow);
    flow_arc.clone().start().await;

    //
    // get the result of the system
    //
    let result = DefaultTask::get_output(flow_arc.get_task(&task2_handle));
    println!("result: {}", result);
    assert_eq!(result, 42);
}
