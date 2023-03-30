mod example_tasks;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::TaskOutput0;

use crate::example_tasks::{AddValuesTask, ForwardDataTask, SettableOutputTask, TaskParamReqs};

impl TaskParamReqs for u8 {}
impl TaskParamReqs for i32 {}
impl TaskParamReqs for i64 {}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    type SimpleAdditionTask = AddValuesTask<i32, u8, i64>;

    //
    // declare system
    //
    let mut flow = Flow::new();

    //
    // create system components
    //
    let input_task_handle = flow.add_new_task(SettableOutputTask::new(42 as i32, 8 as u8));
    let task1_handle = flow.add_new_task(ForwardDataTask::new(1 as i32));
    let task2_handle = flow.add_new_task(ForwardDataTask::new(2 as u8));
    let last_task_handle = flow.add_new_task(SimpleAdditionTask::new(0));

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
    {
        let mut write_handle = flow.get_mut_task(&input_task_handle);
        write_handle.borrow_concrete().set_value0(20);
        write_handle.borrow_concrete().set_value1(10);
    }

    if cfg!(debug_assertions) {
        println!("Executing model with updated parameters");
    }
    let flow_exec2_future = flow.execute();

    //
    // get the results of the systems
    //
    let flow_exec = flow_exec1_future.await;
    let read_handle = flow_exec.get_task(&last_task_handle);
    let result = SimpleAdditionTask::get_output_0(read_handle.borrow());
    println!("first execution result: {}", result);
    assert_eq!(*result, 50);

    let flow_exec = flow_exec2_future.await;
    let read_handle = flow_exec.get_task(&last_task_handle);
    let result = SimpleAdditionTask::get_output_0(read_handle.borrow());
    println!("second execution result: {}", result);
    assert_eq!(*result, 30);
}
