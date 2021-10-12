fn main() {
    windows::build! {
        Windows::Win32::Foundation::{HANDLE, HWND},
        Windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId},
        Windows::Win32::System::Threading::OpenProcess,
        Windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory, GetLastError},
    };
}