mod example_tasks;

use num::cast;
use rs_taskflow::flow::Flow;

use crate::example_tasks::{OneInputOneOutputTask, TwoInputOneOutputTask, ZeroInputTwoOutputTask};

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    //
    // declare system
    //
    let mut flow = Flow::new();

    //
    // create system components
    //
    let input_task_handle = flow.add_new_task(ZeroInputTwoOutputTask::new(|| (42 as i32, 8 as u8)));
    let task1_handle = flow.add_new_task(OneInputOneOutputTask::new(|x: &i32| x.clone()));
    let task2_handle = flow.add_new_task(OneInputOneOutputTask::new(|x: &u8| x.clone()));
    let last_task_handle = flow.add_new_task(TwoInputOneOutputTask::new(|x: &i32, y: &u8| {
        cast::<i32, i64>(x.clone()).unwrap() + cast::<u8, i64>(y.clone()).unwrap()
    }));

    if cfg!(debug_assertions) {
        println!("Connecting dependent tasks");
    }
    flow.connect_output0_to_input0(&input_task_handle, &task1_handle);
    flow.connect_output1_to_input0(&input_task_handle, &task2_handle);
    flow.connect_output0_to_input0(&task1_handle, &last_task_handle);
    flow.connect_output0_to_input1(&task2_handle, &last_task_handle);

    if cfg!(debug_assertions) {
        println!("Executing model with initial parameters");
    }
    let flow_exec1_future = flow.execute();

    if cfg!(debug_assertions) {
        println!("Updating model parameters");
    }
    // {
    //     let mut write_handle = flow.get_mut_task(&input_task_handle);
    //     write_handle.borrow_concrete().set_value0(20);
    //     write_handle.borrow_concrete().set_value1(10);
    // }

    if cfg!(debug_assertions) {
        println!("Executing model with updated parameters");
    }
    let flow_exec2_future = flow.execute();

    //
    // get the results of the systems
    //
    let flow_exec = flow_exec1_future.await;
    let result = flow_exec.get_task_output0(&last_task_handle);
    println!("first execution result: {}", result.unwrap());
    assert_eq!(*result.unwrap(), 50);

    let flow_exec = flow_exec2_future.await;
    let result = flow_exec.get_task_output0(&last_task_handle);
    println!("second execution result: {}", result.unwrap());
    assert_eq!(*result.unwrap(), 30);
}
