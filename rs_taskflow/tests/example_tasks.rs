use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;

//
// Task that just simply outputs constant data
//
#[derive(Clone)]
pub struct ZeroInputTwoOutputTask<O1, O2, F> {
    output0: Option<O1>,
    output1: Option<O2>,
    func: F,
}

impl<O1, O2, F> ZeroInputTwoOutputTask<O1, O2, F> {
    pub fn new(task_func: F) -> ZeroInputTwoOutputTask<O1, O2, F> {
        Self {
            output0: None,
            output1: None,
            func: task_func,
        }
    }
}

impl<
        O1: 'static + Clone + Send + Sync,
        O2: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn() -> (O1, O2),
    > TaskOutput0<O1> for ZeroInputTwoOutputTask<O1, O2, F>
{
    fn get_output_0(task: &dyn ExecutableTask) -> Option<&O1> {
        task.as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output0
            .as_ref()
    }
}

impl<
        O1: 'static + Clone + Send + Sync,
        O2: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn() -> (O1, O2),
    > TaskOutput1<O1, O2> for ZeroInputTwoOutputTask<O1, O2, F>
{
    fn get_output_1(task: &dyn ExecutableTask) -> Option<&O2> {
        task.as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output1
            .as_ref()
    }
}

impl<
        O1: 'static + Clone + Send + Sync,
        O2: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn() -> (O1, O2),
    > ExecutableTask for ZeroInputTwoOutputTask<O1, O2, F>
{
    fn exec(&mut self, _flow: &Flow) {
        let (o1, o2) = (self.func)();
        self.output0 = Some(o1);
        self.output1 = Some(o2);
    }
}

//
// Task that just simply forwards data
//
#[derive(Clone)]
pub struct OneInputOneOutputTask<I, O, F> {
    input_handle: Option<TaskInputHandle<I>>,
    output: Option<O>,
    func: F,
}

impl<I, O, F> OneInputOneOutputTask<I, O, F> {
    pub fn new(task_func: F) -> Self {
        Self {
            input_handle: None,
            output: None,
            func: task_func,
        }
    }
}

impl<
        I: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I) -> O,
    > TaskInput0<I> for OneInputOneOutputTask<I, O, F>
{
    fn set_input_0(&mut self, task_input: TaskInputHandle<I>) {
        self.input_handle = Some(task_input);
    }
}

impl<
        I: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I) -> O,
    > TaskOutput0<O> for OneInputOneOutputTask<I, O, F>
{
    fn get_output_0(task: &dyn ExecutableTask) -> Option<&O> {
        task.as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output
            .as_ref()
    }
}

impl<
        I: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I) -> O,
    > ExecutableTask for OneInputOneOutputTask<I, O, F>
{
    fn exec(&mut self, flow: &Flow) {
        match &self.input_handle {
            Some(input) => {
                let input_val = input.get_value(flow);
                let o1 = (self.func)(input_val.unwrap());
                self.output = Some(o1);
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
#[derive(Clone)]
pub struct TwoInputOneOutputTask<I1, I2, O, F> {
    input0_handle: Option<TaskInputHandle<I1>>,
    input1_handle: Option<TaskInputHandle<I2>>,
    output: Option<O>,
    func: F,
}

impl<I1, I2, O, F> TwoInputOneOutputTask<I1, I2, O, F> {
    pub fn new(task_func: F) -> Self {
        Self {
            input0_handle: None,
            input1_handle: None,
            output: None,
            func: task_func,
        }
    }
}

impl<
        I1: 'static + Clone,
        I2: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I1, &I2) -> O,
    > TaskInput0<I1> for TwoInputOneOutputTask<I1, I2, O, F>
{
    fn set_input_0(&mut self, task_input: TaskInputHandle<I1>) {
        self.input0_handle = Some(task_input);
    }
}

impl<
        I1: 'static + Clone,
        I2: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I1, &I2) -> O,
    > TaskInput1<I1, I2> for TwoInputOneOutputTask<I1, I2, O, F>
{
    fn set_input_1(&mut self, task_input: TaskInputHandle<I2>) {
        self.input1_handle = Some(task_input);
    }
}

impl<
        I1: 'static + Clone,
        I2: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I1, &I2) -> O,
    > TaskOutput0<O> for TwoInputOneOutputTask<I1, I2, O, F>
{
    fn get_output_0(task: &dyn ExecutableTask) -> Option<&O> {
        task.as_any()
            .downcast_ref::<Self>()
            .unwrap()
            .output
            .as_ref()
    }
}

impl<
        I1: 'static + Clone,
        I2: 'static + Clone,
        O: 'static + Clone + Send + Sync,
        F: 'static + Clone + Send + Sync + Fn(&I1, &I2) -> O,
    > ExecutableTask for TwoInputOneOutputTask<I1, I2, O, F>
{
    fn exec(&mut self, flow: &Flow) {
        match (&self.input0_handle, &self.input1_handle) {
            (Some(input0), Some(input1)) => {
                let input0_val = input0.get_value(flow);
                let input1_val = input1.get_value(flow);
                let o1 = (self.func)(input0_val.unwrap(), input1_val.unwrap());
                self.output = Some(o1);
            }
            _ => {
                unreachable!();
            }
        }
    }
}
