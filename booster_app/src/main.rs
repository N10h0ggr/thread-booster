use std::env;
use std::mem::size_of;
use windows::core::PCWSTR;
use windows_sys::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{CreateFileW, OPEN_EXISTING, FILE_SHARE_WRITE, FILE_FLAGS_AND_ATTRIBUTES};
use windows::Win32::System::IO::DeviceIoControl;

// Bring in the shared definitions.
use priority_booster_common::{IOCTL_PRIORITY_BOOSTER_SET_PRIORITY, ThreadData};

fn main() {
    // Read command-line arguments.
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: Booster.exe <threadid> <priority>");
        return;
    }

    // Parse the thread id and priority from the arguments.
    let thread_id: u32 = args[1].parse().expect("Invalid thread id");
    let priority: i32 = args[2].parse().expect("Invalid priority");

    // Use the w! macro from the windows crate to create a wide string.
    let device_name:*const u16 = w!("\\\\.\\PriorityBooster");

    // Open a handle to the device.
    let handle: HANDLE = unsafe {
        CreateFileW(
            PCWSTR(device_name),
            1073741824u32, // GENERIC_WRITE
            FILE_SHARE_WRITE,
            None,         // security attributes
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),     // flags and attributes cast to u32
            None,         // template file
        )
        .expect("Failed to open device")
    };

    // Prepare the ThreadData structure.
    let thread_data = ThreadData {
        thread_id,
        priority,
    };

    let mut bytes_returned = 0u32;

    // Issue the IOCTL call to set the priority.
    let success = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_PRIORITY_BOOSTER_SET_PRIORITY,
            Some(&thread_data as *const _ as *const _), // pointer to input buffer
            size_of::<ThreadData>() as u32,
            None, // no output buffer
            0,
            Some(&mut bytes_returned),
            None, // no OVERLAPPED
        )
    };

    match success {
        Ok(()) => println!("Priority change succeeded!"),
        Err(e) => eprintln!("Priority change failed! Error: {:?}", e),
    }

    // Close the device handle.
    unsafe {
        CloseHandle(handle);
    }
}
