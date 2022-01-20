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
    fn get(proc_name: &str) -> Option<ProcWindows>;

    ///Read a certain type T from specified memory address
    fn read<T>(&self, proc_address: usize) -> Option<T>;

    ///Read a certain type T from specified memory address and only return the value if 'validator' function returns 'true'
    fn read_valid<T>(&self, proc_address: usize, validator: impl Fn(&T) -> bool) -> Option<T>;

    ///Read a vector of type T with specified 'len' number of elements, fills the array with value returned by 'default_provider'
    fn read_vec<T>(
        &self,
        proc_address: usize,
        len: usize,
        default_provider: impl Fn() -> T,
    ) -> Option<Vec<T>>;

    ///Write the value of T to the specified address
    fn write<T>(&self, proc_address: usize, data: &T) -> (bool, usize);

    ///Get the opened process id
    fn pid(&self) -> isize;
}
#[cfg(target_os = "windows")]
pub type Proc = ProcWindows;
#[cfg(target_os = "linux")]
pub type Proc = ProcLinux;
#[cfg(target_os = "windows")]
#[allow(clippy::needless_return)]
pub mod implementation {
    use std::ffi::c_void;
    use std::process::Output;

    use windows::Win32::Foundation::{GetLastError, HANDLE, HWND, PWSTR};
    use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_ALL_ACCESS
    };
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId};

    #[derive(Clone, Copy, Debug, Default)]
    pub struct ProcWindows {
        win_handle: HANDLE,
    }

    fn parse_tlist_output(plist: Output) -> u32 {
        let plist = plist;
        let stdout = String::from_utf8(plist.stdout);

        if stdout.is_err() {
            return 0;
        }
        let stdout = stdout.unwrap();
        let args: Vec<&str> = stdout.split(',').collect();

        let pids = args[1].trim_matches('"');

        return pids.parse().unwrap();
    }

    impl crate::ProcT for ProcWindows {
        fn get(proc_name: &str) -> Option<ProcWindows> {
            unsafe {
                let mut pid = 0;

                let mut proc_name_w: Vec<u16> = proc_name.encode_utf16().collect();
                let window = FindWindowW(None, PWSTR(proc_name_w.as_mut_ptr()));

                if window == HWND(0) {
                    let arg = format!("IMAGENAME eq {}.exe", proc_name);

                    let plist = std::process::Command::new("cmd")
                        .args(["/C", "tasklist", "/FI", &arg, "/FO", "CSV", "/NH"])
                        .output()
                        .expect("Failed to get process name - lib.bs 97");

                    pid = parse_tlist_output(plist);
                    if pid == 0 {
                        return None;
                    }
                }

                let _ = GetWindowThreadProcessId(window, &mut pid);

                if pid == 0 {}

                let handle = OpenProcess(PROCESS_ALL_ACCESS, None, pid);
                if handle == HANDLE(0) {
                    return None;
                }

                return Some(ProcWindows { win_handle: handle });
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

        fn pid(&self) -> isize {
            self.win_handle.0
        }

        fn write<T>(&self, proc_address: usize, data: &T) -> (bool, usize) {
            unsafe {
                let mut write = 0;
                let result = WriteProcessMemory(
                    self.win_handle,
                    proc_address as *const c_void,
                    std::ptr::addr_of!(*data) as *const c_void,
                    std::mem::size_of::<T>(),
                    &mut write,
                );
                if !result.as_bool() {
                    println!("Erro {:?}", GetLastError());
                }
                return (result.as_bool(), write);
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

    #[derive(Clone, Copy, Debug, Default)]
    pub struct ProcLinux {
        handle: libc::pid_t,
    }

    impl crate::ProcT for ProcLinux {
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

                Some(ProcLinux {
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

        fn pid(&self) -> isize {
            self.handle as isize
        }

        fn write<T>(&self, proc_address: usize, data: &T) -> (bool, usize) {
            todo!()
        }
    }
}
