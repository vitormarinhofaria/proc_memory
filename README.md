# proc_memory
### Basic rust crate for accessing another process's memory.
Only reads memory for now, will implement writing as soon as I can.

### Usage examples

```rust
use proc_memory::ProcT;

if let Some(proc) = proc_memory::Proc::get("Other Proccess"){
    let addr = 0x7FF49E8720A8;
    if let Some(number) = proc.read_valid(addr, |data: &i64| *data > 0){
        println!("{:08X} - {}", addr, number);
    }
}
```

```rust
use proc_memory::ProcT;

struct TwoNum {
    num1: u64,
    num2: i64,
}

let proc = proc_memory::Proc::get("Other Proccess").unwrap();
let two_num = proc.read::<TwoNum>(0x7FF49E8720A8).unwrap();
println!("{} + {} = {}", two_num.num1, two_num.num2, two_num.num1 + two_num.num2);
```

```rust
use proc_memory::ProcT;

let proc = proc_memory::Proc::get("Other Proccess").expect("Failed to get proccess");
let vec = proc.read_vec(0x7FF49E8720A8, 2, || 0i64).unwrap();
println!("{} + {} = {}", vec[0], vec[1], vec[0] + vec[1]);
```
