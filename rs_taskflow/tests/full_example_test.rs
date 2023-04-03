use std::sync::Arc;
use std::cell::RefCell;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
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
            y: AtomicUsize::new(self.y.load(Relaxed))
        }
    }
}

#[rs_taskflow_derive::derive_task((), (i64,))]
struct A;

#[rs_taskflow_derive::derive_task((), (String,))]
struct B;

#[rs_taskflow_derive::derive_task((i64,), (bool, Box<u8>))]
struct C;

#[rs_taskflow_derive::derive_task((i64,), (Arc<Mutex<bool>>, Data))]
struct D;

#[rs_taskflow_derive::derive_task((String,), (String,))]
struct E;

#[rs_taskflow_derive::derive_task((Box<u8>, Arc<Mutex<bool>>), (u32,))]
struct F;

#[rs_taskflow_derive::derive_task((Data, String), (u32, bool))]
struct G;

#[rs_taskflow_derive::derive_task((bool, u32), (Data, Vec<bool>))]
struct H;

#[rs_taskflow_derive::derive_task((bool, u32), ([u16; 5],))]
struct I;

#[rs_taskflow_derive::derive_task((bool, u32), ([i8; 10],))]
struct J;

#[rs_taskflow_derive::derive_task(([i8; 10], [u16; 5], Vec<bool>, Data), (String,))]
struct K;

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    //
    // declare system
    //
    let mut flow = Flow::new();

}
