//! A crate for reading another process raw memory.
//!
//!Usage examples
//!
//!```no_run
//!if let Some(proc) = proc_memory::Proc::get("Other Proccess"){
//!    let addr = 0x7FF49E8720A8;
//!    if let Some(number) = proc.read_valid(addr, |data: &i64| *data > 0){
//!        println!("{:08X} - {}", addr, number);
//!    }
//!}
//!```
//!
//!```no_run
//!struct TwoNum {
//!    num1: u64,
//!    num2: i64,
//!}
//!let proc = proc_memory::Proc::get("Other Proccess").unwrap();
//!let two_num = proc.read::<TwoNum>(0x7FF49E8720A8).unwrap();
//!println!("{} + {} = {}", two_num.num1, two_num.num2, two_num.num1 + two_num.num2);
//!```
//!
//!```no_run
//!let proc = proc_memory::Proc::get("Other Proccess").expect("Failed to get proccess");
//!let vec = proc.read_vec(0x7FF49E8720A8, 2, || 0i64).unwrap_or_default();
//!println!("{} + {} = {}", vec[0], vec[1], vec[0] + vec[1]);
//!```

pub use implementation::*;

#[cfg(windows)]
#[allow(clippy::needless_return)]
pub mod implementation {
    use std::ffi::c_void;

    use wbindings::Windows::Win32::Foundation::{HANDLE, HWND};
    use wbindings::Windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use wbindings::Windows::Win32::System::Threading::{
        OpenProcess, PROCESS_VM_READ, PROCESS_VM_WRITE,
    };
    use wbindings::Windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowThreadProcessId,
    };

    pub struct Proc {
        win_handle: HANDLE,
    }

    impl Proc {
        ///Get a handle to a process with specified title
        pub fn get(proc_name: &str) -> Option<Proc> {
            unsafe {
                let window = FindWindowW(None, proc_name);
                if window == HWND(0) {
                    return None;
                }

                let mut pid = 0;
                let _ = GetWindowThreadProcessId(window, &mut pid);
                if pid == 0 {
                    return None;
                }

                let handle = OpenProcess(PROCESS_VM_READ | PROCESS_VM_WRITE, None, pid);
                if handle == HANDLE(0) {
                    return None;
                }

                return Some(Proc { win_handle: handle });
            }
        }

        ///Read a certain type '<T>' from specified memory address
        pub fn read<T>(&self, proc_address: usize) -> Option<T> {
            unsafe {
                let mut t: T = std::mem::zeroed();
                let mut read_bytes = 0;

                let result = ReadProcessMemory(
                    self.win_handle,
                    proc_address as *const c_void,
                    std::ptr::addr_of_mut!(t) as *mut c_void,
                    std::mem::size_of::<T>(),
                    &mut read_bytes,
                );

                if !result.as_bool() {
                    return None;
                }

                return Some(t);
            }
        }

        ///Read a certain type '<T>' from specified memory address and only return the value if 'validator' function returns 'true'
        pub fn read_valid<T>(
            &self,
            proc_address: usize,
            validator: impl Fn(&T) -> bool,
        ) -> Option<T> {
            unsafe {
                let mut t: T = std::mem::zeroed();
                let mut read_bytes = 0;

                let result = ReadProcessMemory(
                    self.win_handle,
                    proc_address as *const c_void,
                    std::ptr::addr_of_mut!(t) as *mut c_void,
                    std::mem::size_of::<T>(),
                    &mut read_bytes,
                );
                if !result.as_bool() {
                    return None;
                }
                if !validator(&t) {
                    return None;
                }
                return Some(t);
            }
        }

        ///Read a vector of type T with specified 'len' number of elements, fills the array with 'default_provider' of type <<T>>
        pub fn read_vec<T>(
            &self,
            proc_address: usize,
            len: usize,
            default_provider: impl Fn() -> T,
        ) -> Option<Vec<T>> {
            unsafe {
                let mut vec = Vec::<T>::new();
                vec.resize_with(len, default_provider);
                let mut read_bytes = 0;

                let result = ReadProcessMemory(
                    self.win_handle,
                    proc_address as *const c_void,
                    std::ptr::addr_of_mut!(vec[0]) as *mut c_void,
                    std::mem::size_of::<T>() * len,
                    &mut read_bytes,
                );
                if !result.as_bool() {
                    return None;
                }
                return Some(vec);
            }
        }
    }
}
