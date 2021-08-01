use std::cell::Cell;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use num::{cast, NumCast};
use std::ops::Add;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::{
    ExecutableTask, TaskInput0, TaskInput1, TaskInputHandle, TaskOutput0, TaskOutput1,
};

//
// Task that just simply outputs constant data
//
#[derive(Debug)]
struct ConstTask<O1, O2> {
    output0: Mutex<O1>,
    output1: Mutex<O2>,
}

impl<O1: 'static + Send + Debug, O2: 'static + Send + Debug> ConstTask<O1, O2> {
    fn new(initial_value0: O1, initial_value1: O2) -> ConstTask<O1, O2> {
        Self {
            output0: Mutex::new(initial_value0),
            output1: Mutex::new(initial_value1),
        }
    }
}

impl<O1: 'static + Copy + Send + Debug, O2: 'static + Copy + Send + Debug> TaskOutput0<O1>
    for ConstTask<O1, O2>
{
    fn get_output_0(task: &dyn ExecutableTask) -> O1 {
        *task
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output0
            .lock()
            .unwrap()
    }
}

impl<O1: 'static + Copy + Send + Debug, O2: 'static + Copy + Send + Debug> TaskOutput1<O1, O2>
    for ConstTask<O1, O2>
{
    fn get_output_1(task: &dyn ExecutableTask) -> O2 {
        *task
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output1
            .lock()
            .unwrap()
    }
}

impl<O1: 'static + Send + Debug, O2: 'static + Send + Debug> ExecutableTask for ConstTask<O1, O2> {
    fn exec(&self, flow: &Flow) {
        // no-op
    }
}

//
// Task that just simply forwards data
//
#[derive(Debug)]
struct ForwardDataTask<T> {
    input_handle: Option<TaskInputHandle<T>>,
    output: Mutex<T>,
}

impl<T: 'static + Send + Debug> ForwardDataTask<T> {
    fn new(initial_value: T) -> Self {
        Self {
            input_handle: Option::None,
            output: Mutex::new(initial_value),
        }
    }

    fn perform_task(&self, input: T) {
        *self.output.lock().unwrap() = input;
    }
}

impl<T: 'static + Send + Debug> TaskInput0<T> for ForwardDataTask<T> {
    fn set_input_0(&mut self, task_input: TaskInputHandle<T>) {
        self.input_handle = Option::Some(task_input);
    }
}

impl<T: 'static + Copy + Send + Debug> TaskOutput0<T> for ForwardDataTask<T> {
    fn get_output_0(task: &dyn ExecutableTask) -> T {
        *task
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output
            .lock()
            .unwrap()
    }
}

impl<T: 'static + Send + Debug> ExecutableTask for ForwardDataTask<T> {
    fn exec(&self, flow: &Flow) {
        match &self.input_handle {
            Option::Some(input) => {
                let input_val = input.get_value(flow);
                self.perform_task(input_val);
            }
            _ => {
                // no-op
            }
        }
    }
}

//
// Task that adds two numbers
//
#[derive(Debug)]
struct AdderTask<I1, I2, O> {
    input0_handle: Option<TaskInputHandle<I1>>,
    input1_handle: Option<TaskInputHandle<I2>>,
    output: Mutex<O>,
}

impl<I1: NumCast, I2: NumCast, O: NumCast + Add<Output = O>> AdderTask<I1, I2, O> {
    fn new(initial_value: O) -> Self {
        Self {
            input0_handle: Option::None,
            input1_handle: Option::None,
            output: Mutex::new(initial_value),
        }
    }

    fn perform_task(&self, input0: I1, input1: I2) {
        *self.output.lock().unwrap() =
            cast::<I1, O>(input0).unwrap() + cast::<I2, O>(input1).unwrap();
    }
}

impl<
        I1: 'static + NumCast + Debug,
        I2: 'static + NumCast + Debug,
        O: 'static + Send + NumCast + Add<Output = O> + Debug,
    > TaskInput0<I1> for AdderTask<I1, I2, O>
{
    fn set_input_0(&mut self, task_input: TaskInputHandle<I1>) {
        self.input0_handle = Option::Some(task_input);
    }
}

impl<
        I1: 'static + NumCast + Debug,
        I2: 'static + NumCast + Debug,
        O: 'static + Send + NumCast + Add<Output = O> + Debug,
    > TaskInput1<I1, I2> for AdderTask<I1, I2, O>
{
    fn set_input_1(&mut self, task_input: TaskInputHandle<I2>) {
        self.input1_handle = Option::Some(task_input);
    }
}

impl<
        I1: 'static + NumCast + Debug,
        I2: 'static + NumCast + Debug,
        O: 'static + Copy + Send + NumCast + Add<Output = O> + Debug,
    > TaskOutput0<O> for AdderTask<I1, I2, O>
{
    fn get_output_0(task: &dyn ExecutableTask) -> O {
        *task
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output
            .lock()
            .unwrap()
    }
}

impl<
        I1: 'static + NumCast + Debug,
        I2: 'static + NumCast + Debug,
        O: 'static + Send + NumCast + Add<Output = O> + Debug,
    > ExecutableTask for AdderTask<I1, I2, O>
{
    fn exec(&self, flow: &Flow) {
        match (&self.input0_handle, &self.input1_handle) {
            (Option::Some(input0), Option::Some(input1)) => {
                let input0_val = input0.get_value(flow);
                let input1_val = input1.get_value(flow);
                self.perform_task(input0_val, input1_val);
            }
            _ => {
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
    // run the system for 3 time steps
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
