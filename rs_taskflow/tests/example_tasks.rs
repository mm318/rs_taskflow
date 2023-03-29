use num::{cast, NumCast};
use std::fmt::Debug;
use std::ops::Add;
use std::sync::{Arc, RwLock, Weak};

use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;

pub trait TaskParamReqs: 'static + Send + Sync + Debug {}

//
// Task that just simply outputs constant data
//
#[derive(Debug)]
pub struct ConstTask<O1, O2> {
    output0: Arc<RwLock<O1>>,
    output1: Arc<RwLock<O2>>,
}

impl<O1, O2> ConstTask<O1, O2> {
    pub fn new(initial_value0: O1, initial_value1: O2) -> ConstTask<O1, O2> {
        Self {
            output0: Arc::new(RwLock::new(initial_value0)),
            output1: Arc::new(RwLock::new(initial_value1)),
        }
    }
}

impl<O1: TaskParamReqs, O2: TaskParamReqs> TaskOutput0<O1> for ConstTask<O1, O2> {
    fn get_output_0(task: &dyn ExecutableTask) -> Weak<RwLock<O1>> {
        Arc::downgrade(&task.as_any().downcast_ref::<Self>().unwrap().output0)
    }
}

impl<O1: TaskParamReqs, O2: TaskParamReqs> TaskOutput1<O1, O2> for ConstTask<O1, O2> {
    fn get_output_1(task: &dyn ExecutableTask) -> Weak<RwLock<O2>> {
        Arc::downgrade(&task.as_any().downcast_ref::<Self>().unwrap().output1)
    }
}

impl<O1: TaskParamReqs, O2: TaskParamReqs> ExecutableTask for ConstTask<O1, O2> {
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
    output: Arc<RwLock<T>>,
}

impl<T: Clone> ForwardDataTask<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            input_handle: None,
            output: Arc::new(RwLock::new(initial_value)),
        }
    }

    fn perform_task(&self, input: Weak<RwLock<T>>) {
        let input_binding = input.upgrade().unwrap();
        let input_ref = input_binding.read().unwrap();
        *self.output.write().unwrap() = (*input_ref).clone();
    }
}

impl<T: TaskParamReqs + Clone> TaskInput0<T> for ForwardDataTask<T> {
    fn set_input_0(&mut self, task_input: TaskInputHandle<T>) {
        self.input_handle = Some(task_input);
    }
}

impl<T: TaskParamReqs + Clone> TaskOutput0<T> for ForwardDataTask<T> {
    fn get_output_0(task: &dyn ExecutableTask) -> Weak<RwLock<T>> {
        Arc::downgrade(&task.as_any().downcast_ref::<Self>().unwrap().output)
    }
}

impl<T: TaskParamReqs + Clone> ExecutableTask for ForwardDataTask<T> {
    fn exec(&self, flow: &Flow) {
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
#[derive(Debug)]
pub struct AdderTask<I1, I2, O> {
    input0_handle: Option<TaskInputHandle<I1>>,
    input1_handle: Option<TaskInputHandle<I2>>,
    output: Arc<RwLock<O>>,
}

impl<I1: NumCast + Clone, I2: NumCast + Clone, O: NumCast + Add<Output = O>> AdderTask<I1, I2, O> {
    pub fn new(initial_value: O) -> Self {
        Self {
            input0_handle: None,
            input1_handle: None,
            output: Arc::new(RwLock::new(initial_value)),
        }
    }

    fn perform_task(&self, input0: Weak<RwLock<I1>>, input1: Weak<RwLock<I2>>) {
        let input0_binding = input0.upgrade().unwrap();
        let input1_binding = input1.upgrade().unwrap();
        let input0_ref = input0_binding.read().unwrap();
        let input1_ref = input1_binding.read().unwrap();
        *self.output.write().unwrap() = cast::<I1, O>((*input0_ref).clone()).unwrap()
            + cast::<I2, O>((*input1_ref).clone()).unwrap();
    }
}

impl<
        I1: TaskParamReqs + NumCast + Clone,
        I2: TaskParamReqs + NumCast + Clone,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > TaskInput0<I1> for AdderTask<I1, I2, O>
{
    fn set_input_0(&mut self, task_input: TaskInputHandle<I1>) {
        self.input0_handle = Some(task_input);
    }
}

impl<
        I1: TaskParamReqs + NumCast + Clone,
        I2: TaskParamReqs + NumCast + Clone,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > TaskInput1<I1, I2> for AdderTask<I1, I2, O>
{
    fn set_input_1(&mut self, task_input: TaskInputHandle<I2>) {
        self.input1_handle = Some(task_input);
    }
}

impl<
        I1: TaskParamReqs + NumCast + Clone,
        I2: TaskParamReqs + NumCast + Clone,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > TaskOutput0<O> for AdderTask<I1, I2, O>
{
    fn get_output_0(task: &dyn ExecutableTask) -> Weak<RwLock<O>> {
        Arc::downgrade(&task.as_any().downcast_ref::<Self>().unwrap().output)
    }
}

impl<
        I1: TaskParamReqs + NumCast + Clone,
        I2: TaskParamReqs + NumCast + Clone,
        O: TaskParamReqs + NumCast + Add<Output = O>,
    > ExecutableTask for AdderTask<I1, I2, O>
{
    fn exec(&self, flow: &Flow) {
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
