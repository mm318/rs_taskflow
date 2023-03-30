# rs_taskflow
Attempt at recreating https://github.com/cpp-taskflow/cpp-taskflow in Rust. 

![build](https://github.com/mm318/rs_taskflow/actions/workflows/build.yml/badge.svg)


## Example:
```rust
let mut flow = Flow::new();

let a = flow.new_task(DefaultTask::new(1));
let b = flow.new_task(DefaultTask::new(2));
let c = flow.new_task(DefaultTask::new(3));

flow.connect_output0_to_input0(&a, &b);
flow.connect_output0_to_input0(&b, &c);

let flow_exec = flow.execute().await;
let handle = flow_exec.get_task(&c);
let result = DefaultTask::get_output_0(handle.borrow());
```

For a more complete example, see [flow_test.rs](rs_taskflow/tests/flow_test.rs).


## Usage

### Installation
```bash
git clone https://github.com/mm318/rs_taskflow.git
```

### Build and Test
```bash
cargo test --release -- --nocapture
```


## Requirements

Developed using Ubuntu 20.04 and Rust 1.64.  
Tested nightly using `ubuntu-latest` and latest stable Rust (as fetched by `actions-rs/toolchain@v1`). 
