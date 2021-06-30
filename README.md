# rs_taskflow
Attempt at recreating https://github.com/cpp-taskflow/cpp-taskflow in Rust. 


## Example:
```rust
let mut flow = Flow::new();

let a = flow.new_task(DefaultTask::new(1));
let b = flow.new_task(DefaultTask::new(2));
let c = flow.new_task(DefaultTask::new(3));

flow.connect(&a, DefaultTask::get_output, &b, DefaultTask::set_input)
flow.connect(&b, DefaultTask::get_output, &c, DefaultTask::set_input)

let flow_arc = Arc::new(flow);
flow_arc.start().await;
```

For a more complete example, see [flow_test.rs](tests/flow_test.rs).


## Usage
### Installation
```bash
git clone https://github.com/mm318/rs_taskflow.git
```

### Build and Test
```bash
cargo test --all-features -- --nocapture
```