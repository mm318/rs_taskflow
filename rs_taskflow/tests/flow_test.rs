mod example_tasks;

use std::sync::Arc;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::TaskOutput0;

use crate::example_tasks::{AdderTask, ConstTask, ForwardDataTask, TaskParamReqs};

impl TaskParamReqs for u8 {}
impl TaskParamReqs for i32 {}
impl TaskParamReqs for i64 {}

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    type TestAdder = AdderTask<i32, u8, i64>;

    //
    // declare system
    //
    let mut flow = Flow::new();

    //
    // create system components
    //
    let input_task_handle = flow.add_new_task(ConstTask::new(42 as i32, 8 as u8));
    let task1_handle = flow.add_new_task(ForwardDataTask::new(1 as i32));
    let task2_handle = flow.add_new_task(ForwardDataTask::new(2 as u8));
    let last_task_handle = flow.add_new_task(TestAdder::new(0));

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
    let flow_exec = flow_arc.clone().new_execution();
    flow_exec.start();

    //
    // get the result of the system
    //
    let read_handle = flow_arc.get_task(&last_task_handle);
    let result = TestAdder::get_output_0(read_handle.borrow());
    println!("result: {}", result);
    assert_eq!(*result, 50);
}
