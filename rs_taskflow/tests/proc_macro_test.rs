#![allow(unused_imports)]
#![allow(dead_code)]

use rs_taskflow::flow::{Flow, TaskHandle};
use rs_taskflow::task::{ExecutableTask, TaskInputHandle};
use rs_taskflow_derive::{
    generate_connect_tasks_funcs, generate_task_input_iface_traits,
    generate_task_output_iface_traits,
};

generate_task_input_iface_traits!(TaskInput, set_input, 4);
generate_task_output_iface_traits!(TaskOutput, get_output, 4);

#[derive(Clone, Debug)]
struct TestTask {}

impl ExecutableTask for TestTask {
    fn exec(&mut self, _flow: &Flow) {
        unimplemented!();
    }
}

impl TaskInput0<i32> for TestTask {
    fn set_input_0(&mut self, _task_input: TaskInputHandle<i32>) {
        unimplemented!();
    }
}

impl TaskInput1<i32, u32> for TestTask {
    fn set_input_1(&mut self, _task_input: TaskInputHandle<u32>) {
        unimplemented!();
    }
}

impl TaskInput2<i32, u32, String> for TestTask {
    fn set_input_2(&mut self, _task_input: TaskInputHandle<String>) {
        unimplemented!();
    }
}

impl TaskInput3<i32, u32, String, Option<bool>> for TestTask {
    fn set_input_3(&mut self, _task_input: TaskInputHandle<Option<bool>>) {
        unimplemented!();
    }
}

impl TaskOutput0<i32> for TestTask {
    fn get_output_0(_task: &dyn ExecutableTask) -> &i32 {
        unimplemented!();
    }
}

impl TaskOutput1<i32, u32> for TestTask {
    fn get_output_1(_task: &dyn ExecutableTask) -> &u32 {
        unimplemented!();
    }
}

impl TaskOutput2<i32, u32, String> for TestTask {
    fn get_output_2(_task: &dyn ExecutableTask) -> &String {
        unimplemented!();
    }
}

impl TaskOutput3<i32, u32, String, Option<bool>> for TestTask {
    fn get_output_3(_task: &dyn ExecutableTask) -> &Option<bool> {
        unimplemented!();
    }
}

struct FakeFlow {}

impl FakeFlow {
    fn connect<I, O, A: TaskOutput0<O>, B: TaskInput0<I>, T>(
        &mut self,
        _task1_handle: &TaskHandle<A>,
        _task1_output: fn(&dyn ExecutableTask) -> &T,
        _task2_handle: &TaskHandle<B>,
        _task2_input: fn(&mut B, TaskInputHandle<T>),
    ) {
        unimplemented!()
    }

    generate_connect_tasks_funcs!(4);
}

#[test]
fn works() {
    let _task = TestTask {};
    let _flow = FakeFlow {};
}
