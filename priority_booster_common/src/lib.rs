#![no_std]

/// This structure corresponds to the C struct `ThreadData`.
/// We use `#[repr(C)]` to ensure the same memory layout as in C.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ThreadData {
    pub thread_id: u32, // C's ULONG is typically a 32-bit unsigned integer.
    pub priority: i32,
}

/// Device type for the Priority Booster.
/// In the C code, this was defined as 0x8000.
pub const PRIORITY_BOOSTER_DEVICE: u32 = 0x8000;

/// Generate the IOCTL code using the CTL_CODE formula from Windows.
///
/// The CTL_CODE macro is defined as:
/// (DeviceType << 16) | (Access << 14) | (Function << 2) | (Method)
///
/// For PriorityBooster:
/// - DeviceType: PRIORITY_BOOSTER_DEVICE (0x8000)
/// - Function: 0x800
/// - Method: METHOD_NEITHER (which is 3)
/// - Access: FILE_ANY_ACCESS (which is 0)
///
/// This yields:
///   IOCTL_PRIORITY_BOOSTER_SET_PRIORITY = (0x8000 << 16) | (0 << 14) | (0x800 << 2) | 3
pub const IOCTL_PRIORITY_BOOSTER_SET_PRIORITY: u32 =
    (PRIORITY_BOOSTER_DEVICE << 16) | (0 << 14) | (0x800 << 2) | 3;
