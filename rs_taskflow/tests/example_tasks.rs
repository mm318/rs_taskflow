use num::{cast, NumCast};
use std::ops::Add;
use std::fmt::Debug;
use std::sync::Mutex;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;

//
// Task that just simply outputs constant data
//
#[derive(Debug)]
pub struct ConstTask<O1, O2> {
    output0: Mutex<O1>,
    output1: Mutex<O2>,
}

impl<O1: 'static + Send + Debug, O2: 'static + Send + Debug> ConstTask<O1, O2> {
    pub fn new(initial_value0: O1, initial_value1: O2) -> ConstTask<O1, O2> {
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
    fn exec(&self, _flow: &Flow) {
        // no-op
    }
}

//
// Task that just simply forwards data
//
#[derive(Debug)]
pub struct ForwardDataTask<T> {
    input_handle: Option<TaskInputHandle<T>>,
    output: Mutex<T>,
}

impl<T: 'static + Send + Debug> ForwardDataTask<T> {
    pub fn new(initial_value: T) -> Self {
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
                unreachable!();
            }
        }
    }
}

//
// Task that adds two numbers
//
#[derive(Debug)]
pub struct AdderTask<I1, I2, O> {
    input0_handle: Option<TaskInputHandle<I1>>,
    input1_handle: Option<TaskInputHandle<I2>>,
    output: Mutex<O>,
}

impl<I1: NumCast, I2: NumCast, O: NumCast + Add<Output = O>> AdderTask<I1, I2, O> {
    pub fn new(initial_value: O) -> Self {
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
                unreachable!();
            }
        }
    }
}
