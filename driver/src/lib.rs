#![no_std]
#![no_main]

extern crate alloc;
#[cfg(not(test))]
extern crate wdk_panic;

use wdk_alloc::WdkAllocator;
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

mod helpers;
use helpers::{init_unicode_string, io_get_current_irp_stack_location};

use core::ptr;
use wdk::println;
use wdk_sys::{
    DRIVER_OBJECT,
    DEVICE_OBJECT,
    IRP,
    NTSTATUS,
    PCUNICODE_STRING,
    STATUS_SUCCESS,
    STATUS_BUFFER_TOO_SMALL,
    STATUS_INVALID_PARAMETER,
    STATUS_INVALID_DEVICE_REQUEST,
    IO_NO_INCREMENT,
    FILE_DEVICE_UNKNOWN,
    IRP_MJ_CREATE,
    IRP_MJ_CLOSE,
    IRP_MJ_DEVICE_CONTROL,
};

use wdk_sys::ntddk::{
    IoCreateDevice,
    IoCreateSymbolicLink,
    IoDeleteSymbolicLink,
    IoDeleteDevice,
    IofCompleteRequest,
};

// Import shared definitions from your common library.
use priority_booster_common::{ThreadData, IOCTL_PRIORITY_BOOSTER_SET_PRIORITY};

extern "C" {
    // FFI declarations for Windows kernel functions.
    fn PsLookupThreadByThreadId(thread_id: usize, thread: *mut *mut core::ffi::c_void) -> NTSTATUS;
    fn KeSetPriorityThread(thread: *mut core::ffi::c_void, priority: i32);
    fn ObDereferenceObject(object: *mut core::ffi::c_void);
}

#[export_name = "DriverEntry"]
pub unsafe extern "C" fn driver_entry(driver_object: *mut DRIVER_OBJECT, _registry_path: PCUNICODE_STRING) -> NTSTATUS {
    println!("PriorityBooster DriverEntry started (println)");

    // Set the unload routine.
    (*driver_object).DriverUnload = Some(driver_unload);

    // Set up the IRP dispatch functions.
    (*driver_object).MajorFunction[IRP_MJ_CREATE as usize] = Some(create_close);
    (*driver_object).MajorFunction[IRP_MJ_CLOSE as usize] = Some(create_close);
    (*driver_object).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(device_control);

    // Use helper function to initialize Unicode strings.
    let mut nt_name = init_unicode_string("\\Device\\PriorityBooster");
    let mut dos_name = init_unicode_string("\\??\\PriorityBooster");

    // Create the device.
    let mut device_object: *mut DEVICE_OBJECT = ptr::null_mut();
    let status = IoCreateDevice(
        driver_object,
        0,
        &mut nt_name, // Passing a mutable pointer as required.
        FILE_DEVICE_UNKNOWN,
        0,
        0,
        &mut device_object,
    );
    if status < 0 {
        println!("Failed to create device: {:#x}", status);
        return status;
    }

    // Create the symbolic link.
    let status = IoCreateSymbolicLink(&mut dos_name, &mut nt_name);
    if status < 0 {
        println!("Failed to create symbolic link: {:#x}", status);
        IoDeleteDevice(device_object);
        return status;
    }

    println!("PriorityBooster DriverEntry completed successfully");
    STATUS_SUCCESS
}

unsafe extern "C" fn driver_unload(driver_object: *mut DRIVER_OBJECT) {
    let mut dos_name = init_unicode_string("\\??\\PriorityBooster");
    IoDeleteSymbolicLink(&mut dos_name);
    IoDeleteDevice((*driver_object).DeviceObject);
    println!("PriorityBooster unloaded");
}

unsafe extern "C" fn create_close(_device_object: *mut DEVICE_OBJECT, irp: *mut IRP) -> NTSTATUS {
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*irp).IoStatus.Information = 0;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

unsafe extern "C" fn device_control(_device_object: *mut DEVICE_OBJECT, irp: *mut IRP) -> NTSTATUS {
    // Use our helper function to retrieve the current IRP stack location.
    let stack = match io_get_current_irp_stack_location(irp) {
        Ok(loc) => loc,
        Err(err) => {
            (*irp).IoStatus.__bindgen_anon_1.Status = err;
            (*irp).IoStatus.Information = 0;
            IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
            return err;
        }
    };

    let mut status = STATUS_SUCCESS;

    // Check if the IOCTL code matches our defined IOCTL.
    if (*stack).Parameters.DeviceIoControl.IoControlCode == IOCTL_PRIORITY_BOOSTER_SET_PRIORITY {
        if (*stack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<ThreadData>() as u32 {
            status = STATUS_BUFFER_TOO_SMALL;
        } else {
            let data_ptr = (*stack).Parameters.DeviceIoControl.Type3InputBuffer as *const ThreadData;
            if data_ptr.is_null() {
                status = STATUS_INVALID_PARAMETER;
            } else {
                let data = *data_ptr;
                if data.priority < 1 || data.priority > 31 {
                    status = STATUS_INVALID_PARAMETER;
                } else {
                    let mut thread: *mut core::ffi::c_void = ptr::null_mut();
                    status = PsLookupThreadByThreadId(data.thread_id as usize, &mut thread);
                    if status >= 0 {
                        KeSetPriorityThread(thread, data.priority);
                        ObDereferenceObject(thread);
                        println!("Thread Priority change for {} to {} succeeded!", data.thread_id, data.priority);
                    }
                }
            }
        }
    } else {
        status = STATUS_INVALID_DEVICE_REQUEST;
    }

    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    (*irp).IoStatus.Information = 0;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    status
}
