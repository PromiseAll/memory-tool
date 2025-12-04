use napi::bindgen_prelude::{BigInt, Buffer, Result};
use napi::{Error, Status};
use napi_derive::napi;
use std::ffi::c_void;

// Windows API 依赖
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtectEx,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

// 使用内部模块化导入
mod internal;

// 重新导出模块类型
use crate::internal::{
    Arch, ProcessInfo, enable_debug_privilege, find_module_info, find_process_id,
    get_all_processes, read_memory_raw, write_memory_raw,
};

// N-API 核心实现

#[napi]
pub struct MemoryTool {
    handle: HANDLE,
    pid: u32,
    arch: Arch,
    debug: bool,
}

impl MemoryTool {
    /// 将BigInt转换为内存地址
    fn bigint_to_addr(&self, val: BigInt) -> Result<usize> {
        let (signed, val_u64, lossless) = val.get_u64();
        if signed {
            return Err(Error::new(
                Status::InvalidArg,
                "内存地址不能为负数".to_string(),
            ));
        }
        if !lossless {
            return Err(Error::new(
                Status::InvalidArg,
                "内存地址过大，精度丢失".to_string(),
            ));
        }
        Ok(val_u64 as usize)
    }

    /// 内部构造函数，处理初始化逻辑
    fn new_internal(pid: u32, arch: Arch, debug: bool) -> Result<Self> {
        if debug {
            println!("[DEBUG] MemoryTool初始化 - PID: {}, 架构: {:?}", pid, arch);
        }

        // 启用调试权限
        if let Err(e) = enable_debug_privilege() {
            if debug {
                println!("[WARN] 启用调试权限失败: {}", e);
            }
        } else if debug {
            println!("[INFO] 调试权限启用成功");
        }

        // 打开进程（使用兼容WoW64的权限）
        let access_flags =
            PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION;

        let handle = unsafe {
            OpenProcess(access_flags, false, pid).map_err(|e| {
                Error::new(Status::GenericFailure, format!("OpenProcess 失败: {}", e))
            })?
        };

        if debug {
            println!("[DEBUG] 进程句柄: {:?}", handle);
        }

        Ok(MemoryTool {
            handle,
            pid,
            arch,
            debug,
        })
    }
}

#[napi]
impl MemoryTool {
    /// 通过进程名创建MemoryTool实例
    #[napi]
    pub fn create_from_name(
        process_name: String,
        arch_is_x64: bool,
        debug_mode: Option<bool>,
    ) -> Result<Self> {
        let debug = debug_mode.unwrap_or(false);
        let arch = if arch_is_x64 { Arch::X64 } else { Arch::X86 };

        // 查找进程ID
        let pid = find_process_id(&process_name).ok_or_else(|| {
            Error::new(
                Status::GenericFailure,
                format!("未找到进程: {}", process_name),
            )
        })?;

        Self::new_internal(pid, arch, debug)
    }

    /// 通过PID创建MemoryTool实例
    #[napi]
    pub fn create_from_pid(pid: u32, arch_is_x64: bool, debug_mode: Option<bool>) -> Result<Self> {
        let debug = debug_mode.unwrap_or(false);
        let arch = if arch_is_x64 { Arch::X64 } else { Arch::X86 };

        Self::new_internal(pid, arch, debug)
    }

    /// 获取所有运行中的进程列表
    #[napi]
    pub fn get_all_processes() -> Vec<ProcessInfo> {
        get_all_processes()
    }

    /// 获取当前进程的所有模块
    #[napi]
    pub fn get_modules(&self) -> Vec<ProcessInfo> {
        crate::internal::get_process_modules(self.pid)
    }

    /// 获取指定模块的起始地址
    #[napi]
    pub fn get_module_start_address(&self, module_name: String) -> Result<BigInt> {
        if self.debug {
            println!("[DEBUG] 获取模块起始地址: {}", module_name);
        }

        let module_info: internal::ModuleAddressInfo = find_module_info(self.pid, &module_name).ok_or_else(|| {
            Error::new(
                Status::GenericFailure,
                format!("模块未找到: {}", module_name),
            )
        })?;
        let addr = module_info.start_address;

        if self.debug {
            println!("[DEBUG] 模块 {} 起始地址: {:#X}", module_name, addr);
        }
        Ok(BigInt::from(addr as u64))
    }

    /// 获取指定模块的结束地址
    #[napi]
    pub fn get_module_end_address(&self, module_name: String) -> Result<BigInt> {
        if self.debug {
            println!("[DEBUG] 获取模块结束地址: {}", module_name);
        }

        let module_info = find_module_info(self.pid, &module_name).ok_or_else(|| {
            Error::new(
                Status::GenericFailure,
                format!("模块未找到: {}", module_name),
            )
        })?;
        let addr = module_info.end_address;

        if self.debug {
            println!("[DEBUG] 模块 {} 结束地址: {:#X}", module_name, addr);
        }
        Ok(BigInt::from(addr as u64))
    }

    /// 解析指针链，获取最终地址
    #[napi]
    pub fn resolve_pointer_chain(&self, base_addr: BigInt, offsets: Vec<u32>) -> Result<BigInt> {
        let mut current_addr = self.bigint_to_addr(base_addr)?;

        if offsets.is_empty() {
            return Ok(BigInt::from(current_addr as u64));
        }

        for (i, &offset) in offsets.iter().take(offsets.len() - 1).enumerate() {
            let ptr_location = current_addr + offset as usize;

            let next_addr_res = match self.arch {
                Arch::X86 => read_memory_raw::<u32>(self.handle, ptr_location).map(|v| v as usize),
                Arch::X64 => read_memory_raw::<u64>(self.handle, ptr_location).map(|v| v as usize),
            };

            match next_addr_res {
                Ok(val) => {
                    if val == 0 {
                        return Err(Error::new(
                            Status::GenericFailure,
                            format!("第 {} 层指针为空", i + 1),
                        ));
                    }
                    current_addr = val;
                }
                Err(e) => {
                    return Err(Error::new(
                        Status::GenericFailure,
                        format!("第 {} 层读取失败: {}", i + 1, e),
                    ));
                }
            }
        }

        if let Some(&last_offset) = offsets.last() {
            let final_addr = current_addr + last_offset as usize;
            Ok(BigInt::from(final_addr as u64))
        } else {
            Ok(BigInt::from(current_addr as u64))
        }
    }

    /// 读取指定地址的字节缓冲区
    #[napi]
    pub fn read_buffer(&self, addr: BigInt, length: u32) -> Result<Buffer> {
        let addr_val = self.bigint_to_addr(addr)?;
        let mut buffer = vec![0u8; length as usize];
        let mut bytes_read = 0;

        let success = unsafe {
            ReadProcessMemory(
                self.handle,
                addr_val as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                length as usize,
                Some(&mut bytes_read),
            )
        };

        if success.is_ok() && bytes_read == length as usize {
            Ok(Buffer::from(buffer))
        } else {
            Err(Error::new(
                Status::GenericFailure,
                format!(
                    "读取缓冲区失败: {}",
                    crate::internal::get_last_error_string()
                ),
            ))
        }
    }

    /// 写入字节缓冲区到指定地址
    #[napi]
    pub fn write_buffer(&self, addr: BigInt, buffer: Buffer) -> Result<()> {
        let addr_val = self.bigint_to_addr(addr)?;
        let data: &[u8] = &buffer;
        let mut bytes_written = 0;

        // 直接写入尝试
        let success = unsafe {
            WriteProcessMemory(
                self.handle,
                addr_val as *const c_void,
                data.as_ptr() as *const c_void,
                data.len(),
                Some(&mut bytes_written),
            )
        };

        if success.is_ok() {
            return Ok(());
        }

        // 尝试修改内存保护后写入
        let mut old_protect = PAGE_PROTECTION_FLAGS(0);
        let protect_res = unsafe {
            VirtualProtectEx(
                self.handle,
                addr_val as *const c_void,
                data.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        };

        if protect_res.is_err() {
            return Err(Error::new(
                Status::GenericFailure,
                format!(
                    "修改内存保护失败: {}",
                    crate::internal::get_last_error_string()
                ),
            ));
        }

        let success_retry = unsafe {
            WriteProcessMemory(
                self.handle,
                addr_val as *const c_void,
                data.as_ptr() as *const c_void,
                data.len(),
                Some(&mut bytes_written),
            )
        };

        // 恢复原始内存保护
        unsafe {
            let _ = VirtualProtectEx(
                self.handle,
                addr_val as *const c_void,
                data.len(),
                old_protect,
                &mut old_protect,
            );
        }

        match success_retry {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(
                Status::GenericFailure,
                format!(
                    "写入缓冲区失败: {}",
                    crate::internal::get_last_error_string()
                ),
            )),
        }
    }

    /// 读取指定地址的字符串（以null结尾）
    #[napi]
    pub fn read_string(&self, addr: BigInt, max_length: Option<u32>) -> Result<String> {
        let addr_val = self.bigint_to_addr(addr)?;
        let limit = max_length.unwrap_or(256) as usize;
        let mut bytes = Vec::new();
        let mut current_offset = 0;

        loop {
            if current_offset >= limit {
                break;
            }
            // 逐字节读取，直到遇到null终止符
            match read_memory_raw::<u8>(self.handle, addr_val + current_offset) {
                Ok(byte) => {
                    if byte == 0 {
                        break;
                    }
                    bytes.push(byte);
                    current_offset += 1;
                }
                Err(e) => {
                    return Err(Error::new(
                        Status::GenericFailure,
                        format!("读取字符串失败 (偏移 {}): {}", current_offset, e),
                    ));
                }
            }
        }

        String::from_utf8(bytes)
            .map_err(|e| Error::new(Status::GenericFailure, format!("UTF-8解码错误: {}", e)))
    }
}

// 宏定义与批量实现

/// 宏：实现数值类型的读写操作
macro_rules! impl_number_rw {
    ($read_name:ident, $write_name:ident, $rust_type:ty, $js_type:ty) => {
        #[napi]
        impl MemoryTool {
            #[napi]
            pub fn $read_name(&self, addr: BigInt) -> Result<$js_type> {
                let addr_val = self.bigint_to_addr(addr)?;
                let val = read_memory_raw::<$rust_type>(self.handle, addr_val)
                    .map_err(|e| Error::new(Status::GenericFailure, e))?;

                Ok(val as $js_type)
            }

            #[napi]
            pub fn $write_name(&self, addr: BigInt, value: $js_type) -> Result<()> {
                let addr_val = self.bigint_to_addr(addr)?;
                let val_converted = value as $rust_type;

                write_memory_raw::<$rust_type>(self.handle, addr_val, &val_converted)
                    .map_err(|e| Error::new(Status::GenericFailure, e))
            }
        }
    };
}

/// 宏：实现大整数类型的读写操作
macro_rules! impl_bigint_rw {
    ($read_name:ident, $write_name:ident, $rust_type:ty) => {
        #[napi]
        impl MemoryTool {
            #[napi]
            pub fn $read_name(&self, addr: BigInt) -> Result<BigInt> {
                let addr_val = self.bigint_to_addr(addr)?;
                let val = read_memory_raw::<$rust_type>(self.handle, addr_val)
                    .map_err(|e| Error::new(Status::GenericFailure, e))?;

                Ok(BigInt::from(val))
            }

            #[napi]
            pub fn $write_name(&self, addr: BigInt, value: BigInt) -> Result<()> {
                let addr_val = self.bigint_to_addr(addr)?;
                let (_, val_u64, _) = value.get_u64();
                let val_converted = val_u64 as $rust_type;

                write_memory_raw::<$rust_type>(self.handle, addr_val, &val_converted)
                    .map_err(|e| Error::new(Status::GenericFailure, e))
            }
        }
    };
}

// 批量生成数值类型读写方法
// 8位整数
impl_number_rw!(read_u8, write_u8, u8, u32);
impl_number_rw!(read_i8, write_i8, i8, i32);
// 16位整数
impl_number_rw!(read_u16, write_u16, u16, u32);
impl_number_rw!(read_i16, write_i16, i16, i32);
// 32位整数和浮点数
impl_number_rw!(read_u32, write_u32, u32, u32);
impl_number_rw!(read_i32, write_i32, i32, i32);
impl_number_rw!(read_float, write_float, f32, f64);
impl_number_rw!(read_double, write_double, f64, f64);
// 64位整数（使用BigInt）
impl_bigint_rw!(read_u64, write_u64, u64);
impl_bigint_rw!(read_i64, write_i64, i64);

// 资源清理
impl Drop for MemoryTool {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_invalid() {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}
