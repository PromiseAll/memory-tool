use crate::internal::arch::i8_to_string;
use napi_derive::napi;
use std::mem::size_of;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32, Module32First, Module32Next, PROCESSENTRY32,
    Process32First, Process32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};

/// 进程信息结构体，用于返回进程基本信息
#[napi]
#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// 模块地址信息结构体，包含模块在内存中的起始和结束地址
pub struct ModuleAddressInfo {
    pub start_address: usize,
    pub end_address: usize,
}

/// 根据进程名查找对应的进程ID
///
/// # 参数
/// * `name` - 进程名称（不含路径）
///
/// # 返回值
/// 找到进程返回Some(pid)，未找到返回None
pub fn find_process_id(name: &str) -> Option<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()? };
    let mut entry = PROCESSENTRY32 {
        dwSize: size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    if unsafe { Process32First(snapshot, &mut entry).is_ok() } {
        loop {
            if i8_to_string(&entry.szExeFile).eq_ignore_ascii_case(name) {
                unsafe {
                    let _ = CloseHandle(snapshot);
                }
                return Some(entry.th32ProcessID);
            }
            if unsafe { Process32Next(snapshot, &mut entry).is_err() } {
                break;
            }
        }
    }
    unsafe {
        let _ = CloseHandle(snapshot);
    }
    None
}

/// 根据进程ID和模块名查找模块地址信息
///
/// # 参数
/// * `pid` - 进程ID
/// * `mod_name` - 模块名称
///
/// # 返回值
/// 找到模块返回Some(ModuleAddressInfo)，未找到返回None
pub fn find_module_info(pid: u32, mod_name: &str) -> Option<ModuleAddressInfo> {
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid).ok()? };
    let mut entry = MODULEENTRY32 {
        dwSize: size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };

    if unsafe { Module32First(snapshot, &mut entry).is_ok() } {
        loop {
            if i8_to_string(&entry.szModule).eq_ignore_ascii_case(mod_name) {
                unsafe {
                    let _ = CloseHandle(snapshot);
                }
                let start_address = entry.modBaseAddr as usize;
                let module_size = entry.modBaseSize as usize;
                let end_address = start_address + module_size;

                return Some(ModuleAddressInfo {
                    start_address,
                    end_address,
                });
            }
            if unsafe { Module32Next(snapshot, &mut entry).is_err() } {
                break;
            }
        }
    }
    unsafe {
        let _ = CloseHandle(snapshot);
    }
    None
}

/// 获取当前系统所有运行中的进程列表
///
/// # 返回值
/// 返回包含所有进程信息的Vec<ProcessInfo>
pub fn get_all_processes() -> Vec<ProcessInfo> {
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .ok()
            .unwrap_or_default()
    };

    // 检查句柄是否有效
    if snapshot.is_invalid() {
        return Vec::new();
    }

    let mut entry = PROCESSENTRY32 {
        dwSize: size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    let mut processes = Vec::new();

    if unsafe { Process32First(snapshot, &mut entry).is_ok() } {
        loop {
            let process_name = i8_to_string(&entry.szExeFile);
            processes.push(ProcessInfo {
                pid: entry.th32ProcessID,
                name: process_name,
            });

            if unsafe { Process32Next(snapshot, &mut entry).is_err() } {
                break;
            }
        }
    }

    unsafe {
        let _ = CloseHandle(snapshot);
    }

    processes
}

/// 获取指定进程的所有模块信息
///
/// # 参数
/// * `pid` - 目标进程ID
///
/// # 返回值
/// 返回该进程加载的所有模块信息
pub fn get_process_modules(pid: u32) -> Vec<ProcessInfo> {
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
            .ok()
            .unwrap_or_default()
    };

    // 检查句柄是否有效
    if snapshot.is_invalid() {
        return Vec::new();
    }

    let mut entry = MODULEENTRY32 {
        dwSize: size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };

    let mut modules = Vec::new();

    if unsafe { Module32First(snapshot, &mut entry).is_ok() } {
        loop {
            let module_name = i8_to_string(&entry.szModule);
            modules.push(ProcessInfo {
                pid: entry.modBaseAddr as u32, // 使用模块基地址作为标识
                name: module_name,
            });

            if unsafe { Module32Next(snapshot, &mut entry).is_err() } {
                break;
            }
        }
    }

    unsafe {
        let _ = CloseHandle(snapshot);
    }

    modules
}
