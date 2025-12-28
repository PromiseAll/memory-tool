use crate::internal::arch::i8_to_string;
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;
use std::mem::size_of;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32, Module32First, Module32Next, PROCESSENTRY32,
    Process32First, Process32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};

/// 进程信息结构体
#[napi(object)]
#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// 模块信息结构体（完整信息）
#[napi(object)]
#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub name: String,
    pub base_address: BigInt,
    pub size: u32,
    pub end_address: BigInt,
}

/// 模块地址信息（内部使用）
pub struct ModuleAddressInfo {
    pub start_address: usize,
    pub end_address: usize,
}

/// 根据进程名查找进程ID
pub fn find_process_id(name: &str) -> Option<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()? };
    let mut entry = PROCESSENTRY32 {
        dwSize: size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    let result = unsafe {
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                if i8_to_string(&entry.szExeFile).eq_ignore_ascii_case(name) {
                    let _ = CloseHandle(snapshot);
                    return Some(entry.th32ProcessID);
                }
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        None
    };

    unsafe {
        let _ = CloseHandle(snapshot);
    }
    result
}

/// 根据进程ID和模块名查找模块地址信息
pub fn find_module_info(pid: u32, mod_name: &str) -> Option<ModuleAddressInfo> {
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid).ok()? };
    let mut entry = MODULEENTRY32 {
        dwSize: size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };

    let result = unsafe {
        if Module32First(snapshot, &mut entry).is_ok() {
            loop {
                if i8_to_string(&entry.szModule).eq_ignore_ascii_case(mod_name) {
                    let start_address = entry.modBaseAddr as usize;
                    let module_size = entry.modBaseSize as usize;
                    let _ = CloseHandle(snapshot);
                    return Some(ModuleAddressInfo {
                        start_address,
                        end_address: start_address + module_size,
                    });
                }
                if Module32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        None
    };

    unsafe {
        let _ = CloseHandle(snapshot);
    }
    result
}

/// 获取所有运行中的进程
pub fn get_all_processes() -> Vec<ProcessInfo> {
    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    let mut entry = PROCESSENTRY32 {
        dwSize: size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    let mut processes = Vec::new();

    unsafe {
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                processes.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    name: i8_to_string(&entry.szExeFile),
                });
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    processes
}

/// 获取指定进程的所有模块（修复：返回正确的 ModuleInfo）
pub fn get_process_modules(pid: u32) -> Vec<ModuleInfo> {
    let snapshot =
        match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) } {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };

    let mut entry = MODULEENTRY32 {
        dwSize: size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };

    let mut modules = Vec::new();

    unsafe {
        if Module32First(snapshot, &mut entry).is_ok() {
            loop {
                let base = entry.modBaseAddr as u64;
                let size = entry.modBaseSize;
                modules.push(ModuleInfo {
                    name: i8_to_string(&entry.szModule),
                    base_address: BigInt::from(base),
                    size,
                    end_address: BigInt::from(base + size as u64),
                });
                if Module32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    modules
}

/// 检测目标进程是否为64位
#[cfg(target_pointer_width = "64")]
pub fn is_process_x64(pid: u32) -> Option<bool> {
    use windows::Win32::System::Threading::{
        IsWow64Process, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::core::BOOL;

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    let mut is_wow64 = BOOL::default();
    let result = unsafe { IsWow64Process(handle, &mut is_wow64) };
    unsafe {
        let _ = CloseHandle(handle);
    }

    if result.is_ok() {
        // WoW64 = 32位进程运行在64位系统上
        // is_wow64 == true 表示是32位进程
        Some(!is_wow64.as_bool())
    } else {
        None
    }
}

#[cfg(target_pointer_width = "32")]
pub fn is_process_x64(_pid: u32) -> Option<bool> {
    // 32位系统上所有进程都是32位
    Some(false)
}
