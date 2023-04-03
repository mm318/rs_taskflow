#![allow(dead_code)]

use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;
use rs_taskflow_derive::derive_task;

#[derive_task((i32, u8, Option<bool>), (i64, String))]
struct TestTask;

fn dummy0(_task: &dyn ExecutableTask) -> Option<&i32> {
    unimplemented!()
}

fn dummy1(_task: &dyn ExecutableTask) -> Option<&u8> {
    unimplemented!()
}

fn dummy2(_task: &dyn ExecutableTask) -> Option<&Option<bool>> {
    unimplemented!()
}

#[test]
fn works() {
    let mut test_task =
        TestTask::new(|_x: &i32, _y: &u8, _z: &Option<bool>| (54 as i64, String::from("asdf")));

    test_task.set_input_0(TaskInputHandle::new(0, dummy0));
    test_task.set_input_1(TaskInputHandle::new(0, dummy1));
    test_task.set_input_2(TaskInputHandle::new(0, dummy2));

    // unable to infer test_task type
    // let output0 = TestTask::get_output_0(&test_task);
    // let output1 = TestTask::get_output_1(&test_task);
}
