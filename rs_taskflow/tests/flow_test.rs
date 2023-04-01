mod example_tasks;

use num::cast;
use rs_taskflow::flow::Flow;

use crate::example_tasks::{AddValuesTask, ForwardDataTask, SettableOutputTask, TaskParamReqs};

impl TaskParamReqs for u8 {}
impl TaskParamReqs for i32 {}
impl TaskParamReqs for i64 {}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    //
    // declare system
    //
    let mut flow = Flow::new();

    //
    // create system components
    //
    let input_task_handle = flow.add_new_task(SettableOutputTask::new(|| (42 as i32, 8 as u8)));
    let task1_handle = flow.add_new_task(ForwardDataTask::new(|x: &i32| x.clone()));
    let task2_handle = flow.add_new_task(ForwardDataTask::new(|x: &u8| x.clone()));
    let last_task_handle = flow.add_new_task(AddValuesTask::new(|x: &i32, y: &u8| cast::<i32, i64>(x.clone()).unwrap() + cast::<u8, i64>(y.clone()).unwrap()));

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
    let flow_exec = flow.execute().await;

    //
    // get the result of the system
    //
    let result = flow_exec.get_task_output0(&last_task_handle);
    println!("result: {}", result.unwrap());
    assert_eq!(*result.unwrap(), 50);
}
