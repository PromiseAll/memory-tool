use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtectEx,
};

use crate::internal::arch::get_last_error_string;

/// 泛型内存读取函数（底层实现）
///
/// # 类型参数
/// * `T` - 要读取的数据类型，必须实现Copy trait
///
/// # 参数
/// * `handle` - 进程句柄
/// * `addr` - 要读取的内存地址
///
/// # 返回值
/// 成功返回Ok(data)，失败返回Err(error_message)
pub fn read_memory_raw<T: Copy>(handle: HANDLE, addr: usize) -> std::result::Result<T, String> {
    let mut buffer: T = unsafe { std::mem::zeroed() };
    let mut bytes_read = 0;
    let success = unsafe {
        ReadProcessMemory(
            handle,
            addr as *const c_void,
            &mut buffer as *mut T as *mut c_void,
            size_of::<T>(),
            Some(&mut bytes_read),
        )
    };
    if success.is_ok() && bytes_read == size_of::<T>() {
        Ok(buffer)
    } else {
        Err(get_last_error_string())
    }
}

/// 泛型内存写入函数（底层实现，带内存保护修改）
///
/// # 类型参数
/// * `T` - 要写入的数据类型，必须实现Copy trait
///
/// # 参数
/// * `handle` - 进程句柄
/// * `addr` - 要写入的内存地址
/// * `value` - 要写入的数据值
///
/// # 返回值
/// 成功返回Ok(()), 失败返回Err(error_message)
///
/// # 实现说明
/// 1. 首先尝试直接写入
/// 2. 如果失败，尝试修改内存保护为可读写
/// 3. 写入完成后恢复原始内存保护
pub fn write_memory_raw<T: Copy>(
    handle: HANDLE,
    addr: usize,
    value: &T,
) -> std::result::Result<(), String> {
    let mut bytes_written = 0;

    // 步骤1：尝试直接写入
    let success = unsafe {
        WriteProcessMemory(
            handle,
            addr as *const c_void,
            value as *const T as *const c_void,
            size_of::<T>(),
            Some(&mut bytes_written),
        )
    };
    if success.is_ok() {
        return Ok(());
    }

    // 步骤2：失败则尝试修改内存保护为可读写
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    let protect_res = unsafe {
        VirtualProtectEx(
            handle,
            addr as *const c_void,
            size_of::<T>(),
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
    };
    if protect_res.is_err() {
        return Err(format!("VirtualProtectEx失败: {}", get_last_error_string()));
    }

    // 步骤3：再次尝试写入
    let success_retry = unsafe {
        WriteProcessMemory(
            handle,
            addr as *const c_void,
            value as *const T as *const c_void,
            size_of::<T>(),
            Some(&mut bytes_written),
        )
    };

    // 步骤4：恢复原始内存保护
    unsafe {
        let _ = VirtualProtectEx(
            handle,
            addr as *const c_void,
            size_of::<T>(),
            old_protect,
            &mut old_protect,
        );
    }

    match success_retry {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("强制写入失败: {}", get_last_error_string())),
    }
}
