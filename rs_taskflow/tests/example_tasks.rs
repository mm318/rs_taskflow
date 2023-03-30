use num::{cast, NumCast};
use std::fmt::Debug;
use std::ops::Add;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;

pub trait TaskParamReqs: 'static + Clone + Send + Sync + Debug {}

//
// Task that just simply outputs constant data
//
#[derive(Clone, Debug)]
pub struct SettableOutputTask<O1, O2> {
    output0: O1,
    output1: O2,
}

impl<O1, O2> SettableOutputTask<O1, O2> {
    pub fn new(initial_value0: O1, initial_value1: O2) -> SettableOutputTask<O1, O2> {
        Self {
            output0: initial_value0,
            output1: initial_value1,
        }
    }

    pub fn set_value0(&mut self, value: O1) {
        self.output0 = value;
    }

    pub fn set_value1(&mut self, value: O2) {
        self.output1 = value;
    }
}

impl<O1: TaskParamReqs, O2: TaskParamReqs> TaskOutput0<O1> for SettableOutputTask<O1, O2> {
    fn get_output_0(task: &dyn ExecutableTask) -> &O1 {
        &task.as_any().downcast_ref::<Self>().unwrap().output0
    }
}

impl<O1: TaskParamReqs, O2: TaskParamReqs> TaskOutput1<O1, O2> for SettableOutputTask<O1, O2> {
    fn get_output_1(task: &dyn ExecutableTask) -> &O2 {
        &task.as_any().downcast_ref::<Self>().unwrap().output1
    }
}

impl<O1: TaskParamReqs, O2: TaskParamReqs> ExecutableTask for SettableOutputTask<O1, O2> {
    fn exec(&mut self, _flow: &Flow) {
        // no-op
    }
}

//
// Task that just simply forwards data
//
#[derive(Clone, Debug)]
pub struct ForwardDataTask<T> {
    input_handle: Option<TaskInputHandle<T>>,
    output: T,
}

impl<T: Clone> ForwardDataTask<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            input_handle: None,
            output: initial_value,
        }
    }

    fn perform_task(&mut self, input: &T) {
        self.output = input.clone();
    }
}

impl<T: TaskParamReqs + Clone> TaskInput0<T> for ForwardDataTask<T> {
    fn set_input_0(&mut self, task_input: TaskInputHandle<T>) {
        self.input_handle = Some(task_input);
    }
}

impl<T: TaskParamReqs + Clone> TaskOutput0<T> for ForwardDataTask<T> {
    fn get_output_0(task: &dyn ExecutableTask) -> &T {
        &task.as_any().downcast_ref::<Self>().unwrap().output
    }
}

impl<T: TaskParamReqs + Clone> ExecutableTask for ForwardDataTask<T> {
    fn exec(&mut self, flow: &Flow) {
        match &self.input_handle {
            Some(input) => {
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
#[derive(Clone, Debug)]
pub struct AddValuesTask<I1, I2, O> {
    input0_handle: Option<TaskInputHandle<I1>>,
    input1_handle: Option<TaskInputHandle<I2>>,
    output: O,
}

impl<I1: Clone + NumCast, I2: Clone + NumCast, O: NumCast + Add<Output = O>>
    AddValuesTask<I1, I2, O>
{
    pub fn new(initial_value: O) -> Self {
        Self {
            input0_handle: None,
            input1_handle: None,
            output: initial_value,
        }
    }

    fn perform_task(&mut self, input0: &I1, input1: &I2) {
        self.output =
            cast::<I1, O>(input0.clone()).unwrap() + cast::<I2, O>(input1.clone()).unwrap();
    }
}

impl<
        I1: TaskParamReqs + NumCast,
        I2: TaskParamReqs + NumCast,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > TaskInput0<I1> for AddValuesTask<I1, I2, O>
{
    fn set_input_0(&mut self, task_input: TaskInputHandle<I1>) {
        self.input0_handle = Some(task_input);
    }
}

impl<
        I1: TaskParamReqs + NumCast,
        I2: TaskParamReqs + NumCast,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > TaskInput1<I1, I2> for AddValuesTask<I1, I2, O>
{
    fn set_input_1(&mut self, task_input: TaskInputHandle<I2>) {
        self.input1_handle = Some(task_input);
    }
}

impl<
        I1: TaskParamReqs + NumCast,
        I2: TaskParamReqs + NumCast,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > TaskOutput0<O> for AddValuesTask<I1, I2, O>
{
    fn get_output_0(task: &dyn ExecutableTask) -> &O {
        &task.as_any().downcast_ref::<Self>().unwrap().output
    }
}

impl<
        I1: TaskParamReqs + NumCast,
        I2: TaskParamReqs + NumCast,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > ExecutableTask for AddValuesTask<I1, I2, O>
{
    fn exec(&mut self, flow: &Flow) {
        match (&self.input0_handle, &self.input1_handle) {
            (Some(input0), Some(input1)) => {
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
