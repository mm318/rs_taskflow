mod example_tasks;

use std::sync::Arc;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::TaskOutput0;

use crate::example_tasks::{AdderTask, ConstTask, ForwardDataTask};

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let mut flow = Flow::new();

    //
    // create system components
    //
    type TestAdder = AdderTask<i32, u8, i64>;
    let task1_handle = flow.new_task(ForwardDataTask::new(1 as i32));
    let task2_handle = flow.new_task(ForwardDataTask::new(2 as u8));
    let input_task_handle = flow.new_task(ConstTask::new(42 as i32, 8 as u8));
    let last_task_handle = flow.new_task(TestAdder::new(0));

    //
    // hook up system components
    //
    if cfg!(debug_assertions) {
        println!("Connecting dependent tasks");
    }
    flow.connect_output0_to_input0(&input_task_handle, &task1_handle);
    flow.connect_output1_to_input0(&input_task_handle, &task2_handle);
    flow.connect_output0_to_input0(&task1_handle, &last_task_handle);
    flow.connect_output0_to_input1(&task2_handle, &last_task_handle);

    //
    // starting running the system
    //
    if cfg!(debug_assertions) {
        println!("Executing model");
    }
    let flow_arc = Arc::new(flow);
    flow_arc.clone().start();

    //
    // get the result of the system
    //
    let result = TestAdder::get_output_0(flow_arc.get_task(&last_task_handle));
    println!("result: {}", result);
    assert_eq!(result, 50);
}
