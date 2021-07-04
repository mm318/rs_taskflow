use rs_taskflow::flow::Flow;
use rs_taskflow::task::{ExecutableTask, TaskInputHandle};

rs_taskflow_derive::generate_task_input_iface_traits!(TaskInput, set_input, 4);
rs_taskflow_derive::generate_task_output_iface_traits!(TaskOutput, get_output, 4);

#[derive(Debug)]
struct TestTask {}

impl ExecutableTask for TestTask {
    fn exec(&self, _flow: &Flow) {
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
    fn get_output_0(_task: &dyn ExecutableTask) -> i32 {
        unimplemented!();
    }
}

impl TaskOutput1<i32, u32> for TestTask {
    fn get_output_1(_task: &dyn ExecutableTask) -> u32 {
        unimplemented!();
    }
}

impl TaskOutput2<i32, u32, String> for TestTask {
    fn get_output_2(_task: &dyn ExecutableTask) -> String {
        unimplemented!();
    }
}

impl TaskOutput3<i32, u32, String, Option<bool>> for TestTask {
    fn get_output_3(_task: &dyn ExecutableTask) -> Option<bool> {
        unimplemented!();
    }
}

#[test]
fn works() {
    let _task = TestTask {};
}
