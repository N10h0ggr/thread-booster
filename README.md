# Priority Booster

Priority Booster is a Rust-based project that demonstrates how to adjust thread priorities on Windows by communicating with a custom kernel driver. The project is split into two main components: a driver that handles IOCTL requests and a userland application that sends these requests.

The userland application takes a thread ID and a priority value as command-line arguments and then uses Windows API functions to open a handle to the driver device (using the device path `\\.\PriorityBooster`). It then prepares a simple data structure and sends it via an IOCTL call. This mechanism allows the driver to update the thread’s priority accordingly. For example, if you have a low-priority thread that you need to boost temporarily, this application can be used to increase its priority.

## Overview

- **Driver Component:**  
    A Windows kernel driver that listens for IOCTL commands to change thread priorities. It serves as the bridge between userland commands and the actual thread scheduling mechanism in the Windows kernel.
    
- **Userland Application:**  
    A Rust program that uses Windows API calls (such as `CreateFileW` and `DeviceIoControl`) to communicate with the driver. It parses command-line arguments to obtain the target thread ID and the new priority level, and then issues an IOCTL command to request the priority change.
    
- **Shared Library:**  
    The project includes a shared library (often found in a crate like `priority_booster_common`) which contains the common definitions used by both the driver and the userland application. This includes the IOCTL codes and data structures (such as `ThreadData`). By centralizing these definitions, the project ensures consistency between the two components and reduces code duplication. Imagine having a single source of truth for data structures—if you need to make a change, you update it once rather than in multiple places.
    
- **Helpers File:**  
    The helpers file contains utility functions, macros, and any other common functionality that both parts of the project might need. For example, it can simplify tasks like string conversions, error handling, or other repetitive operations. This keeps the main codebase cleaner and makes maintenance easier. Think of it as a toolbox that both the driver and the userland application can share, making sure they both speak the same language when it comes to common tasks.
    

## Getting Started

### Prerequisites

- **Debugee Virtual Machine:** Windows 10 or later.
- **Rust Toolchain:** Install via [rustup](https://rustup.rs/).
- **Environment Setup:** I recommend following the GitHub [Windows WDK repository](https://github.com/microsoft/windows-drivers-rs/tree/main) guide to install the build requirements. This project is configured to have this repository locally.

### Building the Project

1. **Build the Driver:**
    
    Navigate to the driver directory and build it:
    
    ```sh
    cd driver
    cargo make
    ```
    
    _Note:_ Make sure you follow Windows driver signing and installation guidelines.
    
2. **Build the Userland Application:**
    
    Navigate to the userland_app directory and build it:
    
    ```sh
    cd ../booster_app
    cargo build --release
    ```
    

### Usage Example

You can install and start the driver as in the [Windows example](https://github.com/microsoft/windows-drivers-rs/tree/main/examples/sample-wdm-driver) or this way:

```cmd
sc create booster type= kernel binPath= C:\Users\debugee\Desktop\Booster\thread_booster.sys
sc start booster
```

Now, you can change a thread’s priority by running:

```cmd
Booster.exe 944 25
```

In this example, the application sends an IOCTL request to the driver to change the priority of the thread with ID `944` to a new level `25`.

![[./before_boost.png]] ![[./after_boost.png]]

## References

- Thanks to the developers of the [windows crate](https://github.com/microsoft/windows-rs) for simplifying Windows API integration in Rust.

This README explains the purpose of Priority Booster and its key components, including the shared library and helpers file. The shared library ensures that both the driver and the userland application use consistent definitions for data structures and IOCTL codes, while the helpers file provides a common set of utility functions to keep the codebase clean and maintainable. Enjoy experimenting with thread priority changes using Rust on Windows!
