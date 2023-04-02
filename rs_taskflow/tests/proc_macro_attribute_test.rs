#![allow(dead_code)]

use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;
use rs_taskflow_derive::derive_task;

#[derive_task((i32, u8, Option<bool>), (i64, String))]
struct TestTask;

#[test]
fn works() {
    let _task =
        TestTask::new(|_x: &i32, _y: &u8, _z: &Option<bool>| (54 as i64, String::from("asdf")));
}
