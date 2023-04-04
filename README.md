# rs_taskflow
Attempt at recreating https://github.com/cpp-taskflow/cpp-taskflow in Rust. 

![build](https://github.com/mm318/rs_taskflow/actions/workflows/build.yml/badge.svg)


## Example:
```rust
#[derive_task((i32,), (i32,))]
struct FlowTask;

let mut flow = Flow::new();

let a = flow.new_task(FlowTask::new(|x: &i32| *x + 1));
let b = flow.new_task(FlowTask::new(|x: &i32| *x - 2));
let c = flow.new_task(FlowTask::new(|x: &i32| *x + 3));

flow.connect_output0_to_input0(&a, &b);
flow.connect_output0_to_input0(&b, &c);

let flow_exec = flow.execute().await;
let result = flow_exec.get_output_0(&c);
```

For a more complete example, see [full_example_test.rs](rs_taskflow/tests/full_example_test.rs).


## Usage

### Installation
```bash
git clone https://github.com/mm318/rs_taskflow.git
```

### Build
For development iterations
```bash
# disables use of the macros in rs_taskflow_derive
cargo test --no-default-features -- --nocapture
```

For testing build with more debug messages
```bash
cargo test -- --nocapture
```

For testing release build
```bash
cargo test --release -- --nocapture
```


## Requirements

Developed using Ubuntu 20.04 and Rust 1.64.  
Tested nightly using `ubuntu-latest` and latest stable Rust (as fetched by `actions-rs/toolchain@v1`). 
