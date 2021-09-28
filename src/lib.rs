//! A crate for reading another process raw memory.
//!
//!Usage examples
//!
//!```no_run
//!use proc_memory::ProcT;
//!
//!if let Some(proc) = proc_memory::Proc::get("Other Proccess"){
//!    let addr = 0x7FF49E8720A8;
//!    if let Some(number) = proc.read_valid(addr, |data: &i64| *data > 0){
//!        println!("{:08X} - {}", addr, number);
//!    }
//!}
//!```
//!
//!```no_run
//!use proc_memory::ProcT;
//!
//!struct TwoNum {
//!    num1: u64,
//!    num2: i64,
//!}
//!
//!let proc = proc_memory::Proc::get("Other Proccess").unwrap();
//!let two_num = proc.read::<TwoNum>(0x7FF49E8720A8).unwrap();
//!println!("{} + {} = {}", two_num.num1, two_num.num2, two_num.num1 + two_num.num2);
//!```
//!
//!```no_run
//!use proc_memory::ProcT;
//!
//!let proc = proc_memory::Proc::get("Other Proccess").expect("Failed to get proccess");
//!let vec = proc.read_vec(0x7FF49E8720A8, 2, || 0i64).unwrap();
//!println!("{} + {} = {}", vec[0], vec[1], vec[0] + vec[1]);
//!```

pub use implementation::*;

pub trait ProcT {
    ///Get a handle to a process with specified title
    fn get(proc_name: &str) -> Option<Proc>;

    ///Read a certain type '<T>' from specified memory address
    fn read<T>(&self, proc_address: usize) -> Option<T>;

    ///Read a certain type '<T>' from specified memory address and only return the value if 'validator' function returns 'true'
    fn read_valid<T>(&self, proc_address: usize, validator: impl Fn(&T) -> bool) -> Option<T>;

    ///Read a vector of type T with specified 'len' number of elements, fills the array with 'default_provider' of type <<T>>
    fn read_vec<T>(
        &self,
        proc_address: usize,
        len: usize,
        default_provider: impl Fn() -> T,
    ) -> Option<Vec<T>>;
}

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

    impl crate::ProcT for Proc {
        fn get(proc_name: &str) -> Option<Proc> {
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

        fn read<T>(&self, proc_address: usize) -> Option<T> {
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

        fn read_valid<T>(&self, proc_address: usize, validator: impl Fn(&T) -> bool) -> Option<T> {
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

        fn read_vec<T>(
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

#[cfg(target_os = "linux")]
#[allow(clippy::needless_return)]
pub mod implementation {
    use std::{
        ffi::CString,
        fs::File,
        io::{Read, Seek, SeekFrom},
    };

    use libc::pid_t;

    pub struct Proc {
        handle: libc::pid_t,
    }

    impl crate::ProcT for Proc {
        fn get(proc_name: &str) -> Option<Proc> {
            unsafe {
                let pid_cmd = CString::new(format!("pidof -s {}", proc_name)).unwrap();
                let mode_c = CString::new("r").unwrap();
                let pid_pipe = libc::popen(pid_cmd.as_c_str().as_ptr(), mode_c.as_c_str().as_ptr());

                let mut buff = [0i8; 512];
                libc::fgets(&mut buff[0], 512, pid_pipe);

                let pid = libc::strtol(&buff[0], std::ptr::null_mut::<*mut i8>(), 10);

                libc::pclose(pid_pipe);

                if pid == 0 {
                    return None;
                } else {
                    println!("PID: {}", pid);
                }

                Some(Proc {
                    handle: pid as pid_t,
                })
            }
        }

        fn read<T>(&self, proc_address: usize) -> Option<T> {
            unsafe {
                let mut temp: T = std::mem::zeroed();
                let proc_file = format!("/proc/{}/mem", self.handle);
                let mem_f = File::open(proc_file);

                if let Ok(mut mem) = mem_f {
                    let _ = mem.seek(SeekFrom::Start(proc_address as u64)).unwrap();

                    let dst_ptr = &mut temp as *mut T as *mut u8;
                    let mut buffer =
                        std::slice::from_raw_parts_mut(dst_ptr, std::mem::size_of::<T>());

                    if let Ok(()) = mem.read_exact(&mut buffer) {
                        return Some(temp);
                    } else {
                        return None;
                    }
                };
                return None;
            }
        }

        fn read_valid<T>(&self, proc_address: usize, validator: impl Fn(&T) -> bool) -> Option<T> {
            unsafe {
                let mut temp: T = std::mem::zeroed();
                let proc_file = format!("/proc/{}/mem", self.handle);
                let mem_f = File::open(proc_file);

                if let Ok(mut mem) = mem_f {
                    let _ = mem.seek(SeekFrom::Start(proc_address as u64)).unwrap();

                    let dst_ptr = &mut temp as *mut T as *mut u8;
                    let mut buffer =
                        std::slice::from_raw_parts_mut(dst_ptr, std::mem::size_of::<T>());

                    if let Ok(()) = mem.read_exact(&mut buffer) {
                        if validator(&temp) {
                            return Some(temp);
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }

                return None;
            }
        }

        fn read_vec<T>(
            &self,
            proc_address: usize,
            len: usize,
            default_provider: impl Fn() -> T,
        ) -> Option<Vec<T>> {
            unsafe {
                let mut temp = Vec::<T>::new();
                temp.resize_with(len, default_provider);

                let proc_file = format!("/proc/{}/mem", self.handle);
                let mem_f = File::open(proc_file);

                if let Ok(mut mem) = mem_f {
                    let _ = mem.seek(SeekFrom::Start(proc_address as u64)).unwrap();

                    let dst_ptr = &mut temp[0] as *mut T as *mut u8;
                    let mut buffer =
                        std::slice::from_raw_parts_mut(dst_ptr, std::mem::size_of::<T>() * len);

                    if let Ok(()) = mem.read_exact(&mut buffer) {
                        return Some(temp);
                    } else {
                        return None;
                    }
                };
                return None;
            }
        }
    }
}
