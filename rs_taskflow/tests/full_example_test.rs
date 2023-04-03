use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitXor};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::sync::Mutex;

use rs_taskflow::flow::Flow;
use rs_taskflow::task::*;

/*
Create a TaskFlow graph like the following:

                A     B
               / \    |
              C   D   E
              |\ / \ /
              | F   G
              |/   / \
              H   I   J
               \  |  /
                \ | /
                 \|/
                  K
*/

struct Data {
    x: &'static str,
    y: AtomicUsize,
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: AtomicUsize::new(self.y.load(Relaxed)),
        }
    }
}

#[rs_taskflow_derive::derive_task((), (i64,))]
struct TaskA;

fn func_a() -> i64 {
    492
}

#[rs_taskflow_derive::derive_task((), (String,))]
struct TaskB;

fn func_b() -> String {
    String::from("From Task B")
}

#[rs_taskflow_derive::derive_task((i64,), (bool, Box<u8>))]
struct TaskC;

fn func_c(x: &i64) -> (bool, Box<u8>) {
    (x % 2 == 1, Box::new(*x as u8))
}

#[rs_taskflow_derive::derive_task((i64,), (Arc<Mutex<bool>>, Data))]
struct TaskD;

fn func_d(x: &i64) -> (Arc<Mutex<bool>>, Data) {
    (
        Arc::new(Mutex::new(x % 2 == 0)),
        Data {
            x: "From Task D",
            y: AtomicUsize::new(*x as usize),
        },
    )
}

#[rs_taskflow_derive::derive_task((String,), (String,))]
struct TaskE;

fn func_e(x: &String) -> String {
    x.clone().add("\nFrom Task E")
}

#[rs_taskflow_derive::derive_task((Box<u8>, Arc<Mutex<bool>>), (u32,))]
struct TaskF;

fn func_f(x: &Box<u8>, y: &Arc<Mutex<bool>>) -> u32 {
    let result = if *y.lock().unwrap() {
        x.bitxor(0b00000000)
    } else {
        x.bitxor(0b11111111)
    };
    result as u32
}

#[rs_taskflow_derive::derive_task((Data, String), (u32, bool))]
struct TaskG;

fn func_g(x: &Data, y: &String) -> (u32, bool) {
    let mut s = DefaultHasher::new();
    x.x.hash(&mut s);
    y.hash(&mut s);
    (
        s.finish() as u32,
        y.eq_ignore_ascii_case(x.x) || x.y.load(Relaxed) % 2 == 0,
    )
}

#[rs_taskflow_derive::derive_task((bool, u32), (Data, Vec<bool>))]
struct TaskH;

fn func_h(x: &bool, y: &u32) -> (Data, Vec<bool>) {
    let new_data = Data {
        x: "From Task H",
        y: AtomicUsize::new(*y as usize),
    };
    (new_data, vec![*x; 15])
}

#[rs_taskflow_derive::derive_task((bool, u32), ([u16; 5],))]
struct TaskI;

fn func_i(x: &bool, y: &u32) -> [u16; 5] {
    let bytes = y.to_ne_bytes();
    let mut result: [u16; 5] = [0; 5];
    for (i, b) in bytes.iter().enumerate() {
        result[i] = *b as u16;
    }
    *result.last_mut().unwrap() = *x as u16;
    result
}

#[rs_taskflow_derive::derive_task((bool, u32), ([i8; 10],))]
struct TaskJ;

fn func_j(x: &bool, y: &u32) -> [i8; 10] {
    let bytes = y.to_ne_bytes();
    let mut result: [i8; 10] = if *x { [0; 10] } else { [2; 10] };
    for (i, b) in bytes.iter().enumerate() {
        result[i] += *b as i8;
    }
    for (i, b) in bytes.iter().enumerate() {
        result[10 - i - 1] -= *b as i8;
    }
    result
}

#[rs_taskflow_derive::derive_task(([i8; 10], [u16; 5], Vec<bool>, Data), (String,))]
struct TaskK;

fn func_k(w: &[i8; 10], x: &[u16; 5], y: &Vec<bool>, z: &Data) -> String {
    format!("Task K result: {:?} {:?} {:?} {} {:?}", w, x, y, z.x, z.y)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    //
    // declare system
    //
    let mut flow = Flow::new();

    let task_a = flow.add_new_task(TaskA::new(func_a));
    let task_b = flow.add_new_task(TaskB::new(func_b));
    let task_c = flow.add_new_task(TaskC::new(func_c));
    let task_d = flow.add_new_task(TaskD::new(func_d));
    let task_e = flow.add_new_task(TaskE::new(func_e));
    let task_f = flow.add_new_task(TaskF::new(func_f));
    let task_g = flow.add_new_task(TaskG::new(func_g));
    let task_h = flow.add_new_task(TaskH::new(func_h));
    let task_i = flow.add_new_task(TaskI::new(func_i));
    let task_j = flow.add_new_task(TaskJ::new(func_j));
    let task_k = flow.add_new_task(TaskK::new(func_k));

    flow.connect_output0_to_input0(&task_a, &task_c);
    flow.connect_output0_to_input0(&task_a, &task_d);
    flow.connect_output0_to_input0(&task_b, &task_e);
    flow.connect_output0_to_input0(&task_c, &task_h);
    flow.connect_output1_to_input0(&task_c, &task_f);
    flow.connect_output0_to_input1(&task_d, &task_f);
    flow.connect_output1_to_input0(&task_d, &task_g);
    flow.connect_output0_to_input1(&task_e, &task_g);
    flow.connect_output0_to_input1(&task_f, &task_h);
    flow.connect_output0_to_input1(&task_g, &task_i);
    flow.connect_output1_to_input0(&task_g, &task_i);
    flow.connect_output0_to_input1(&task_g, &task_j);
    flow.connect_output1_to_input0(&task_g, &task_j);
    flow.connect_output0_to_input3(&task_h, &task_k);
    flow.connect_output1_to_input2(&task_h, &task_k);
    flow.connect_output0_to_input1(&task_i, &task_k);
    flow.connect_output0_to_input0(&task_j, &task_k);

    let flow_exec = flow.execute().await;

    let result = flow_exec.get_task_output0(&task_k);
    println!("result: {}", result.unwrap());
}
