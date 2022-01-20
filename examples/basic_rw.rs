use proc_memory::*;

const ADDR: usize = 0x00007FF49E872000;
fn main() {
    let proc = proc_memory::Proc::get("target_program").expect("Error opening program");
    
    let mut read_val = proc.read::<u64>(ADDR).expect("Failed to read value");
    println!("Read {} from {:X}", read_val, ADDR);
    
    let write_val = 180;
    
    let (write, write_count) = proc.write(ADDR, &write_val);
    if write && write_count > 0 {
        read_val = proc.read::<u64>(ADDR).expect("Failed to read value");
        println!("Read {} from {:X}", read_val, ADDR);
    }else{
        println!("Could not write");
    }
}