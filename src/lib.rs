use napi::bindgen_prelude::{BigInt, Buffer, Result};
use napi::{Error, Status};
use napi_derive::napi;
use std::ffi::c_void;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtectEx,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

mod internal;

use crate::internal::{
    Arch, ModuleInfo, ProcessInfo, enable_debug_privilege, find_module_info, find_process_id,
    get_all_processes, get_last_error_string, is_process_x64, read_memory_raw, write_memory_raw,
};

/// 创建选项（具名参数）
#[napi(object)]
pub struct CreateOptions {
    /// 进程名称（与 pid 二选一）
    pub process_name: Option<String>,
    /// 进程 ID（与 process_name 二选一）
    pub pid: Option<u32>,
    /// 是否为 64 位进程（可选，默认自动检测）
    pub arch_x64: Option<bool>,
    /// 调试模式
    pub debug: Option<bool>,
}

/// 内存读写工具主类
#[napi]
pub struct MemoryTool {
    handle: HANDLE,
    pid: u32,
    arch: Arch,
    debug: bool,
}

impl MemoryTool {
    /// BigInt 转内存地址
    fn bigint_to_addr(&self, val: BigInt) -> Result<usize> {
        let (signed, val_u64, lossless) = val.get_u64();
        if signed {
            return Err(Error::new(Status::InvalidArg, "内存地址不能为负数"));
        }
        if !lossless {
            return Err(Error::new(Status::InvalidArg, "内存地址精度丢失"));
        }
        Ok(val_u64 as usize)
    }

    /// 内部构造
    fn new_internal(pid: u32, arch: Arch, debug: bool) -> Result<Self> {
        if debug {
            println!("[DEBUG] 初始化 PID: {}, 架构: {:?}", pid, arch);
        }

        if let Err(e) = enable_debug_privilege() {
            if debug {
                println!("[WARN] 启用调试权限失败: {}", e);
            }
        }

        let access_flags =
            PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION;

        let handle = unsafe {
            OpenProcess(access_flags, false, pid)
                .map_err(|e| Error::new(Status::GenericFailure, format!("OpenProcess 失败: {}", e)))?
        };

        Ok(MemoryTool { handle, pid, arch, debug })
    }
}

#[napi]
impl MemoryTool {
    /// 通过配置对象创建（推荐，具名参数）
    #[napi]
    pub fn create(options: CreateOptions) -> Result<Self> {
        let debug = options.debug.unwrap_or(false);

        // 获取 PID
        let pid = match (options.process_name, options.pid) {
            (Some(name), _) => find_process_id(&name).ok_or_else(|| {
                Error::new(Status::GenericFailure, format!("未找到进程: {}", name))
            })?,
            (None, Some(pid)) => pid,
            (None, None) => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "必须提供 processName 或 pid",
                ));
            }
        };

        // 确定架构
        let arch = match options.arch_x64 {
            Some(true) => Arch::X64,
            Some(false) => Arch::X86,
            None => match is_process_x64(pid) {
                Some(true) => Arch::X64,
                Some(false) => Arch::X86,
                None => {
                    if debug {
                        println!("[WARN] 无法检测架构，默认使用 X64");
                    }
                    Arch::X64
                }
            },
        };

        if debug {
            println!("[INFO] 检测到进程架构: {:?}", arch);
        }

        Self::new_internal(pid, arch, debug)
    }

    /// 通过进程名创建（自动检测架构）
    #[napi]
    pub fn create_from_name(process_name: String, debug_mode: Option<bool>) -> Result<Self> {
        Self::create(CreateOptions {
            process_name: Some(process_name),
            pid: None,
            arch_x64: None,
            debug: debug_mode,
        })
    }

    /// 通过进程名创建（手动指定架构）
    #[napi]
    pub fn create_from_name_with_arch(
        process_name: String,
        arch_is_x64: bool,
        debug_mode: Option<bool>,
    ) -> Result<Self> {
        Self::create(CreateOptions {
            process_name: Some(process_name),
            pid: None,
            arch_x64: Some(arch_is_x64),
            debug: debug_mode,
        })
    }

    /// 通过 PID 创建（自动检测架构）
    #[napi]
    pub fn create_from_pid(pid: u32, debug_mode: Option<bool>) -> Result<Self> {
        Self::create(CreateOptions {
            process_name: None,
            pid: Some(pid),
            arch_x64: None,
            debug: debug_mode,
        })
    }

    /// 通过 PID 创建（手动指定架构）
    #[napi]
    pub fn create_from_pid_with_arch(pid: u32, arch_is_x64: bool, debug_mode: Option<bool>) -> Result<Self> {
        Self::create(CreateOptions {
            process_name: None,
            pid: Some(pid),
            arch_x64: Some(arch_is_x64),
            debug: debug_mode,
        })
    }

    /// 获取所有进程
    #[napi]
    pub fn get_all_processes() -> Vec<ProcessInfo> {
        get_all_processes()
    }

    /// 获取当前进程的所有模块
    #[napi]
    pub fn get_modules(&self) -> Vec<ModuleInfo> {
        crate::internal::get_process_modules(self.pid)
    }

    /// 获取指定模块信息
    #[napi]
    pub fn get_module(&self, module_name: String) -> Result<ModuleInfo> {
        let info = find_module_info(self.pid, &module_name).ok_or_else(|| {
            Error::new(Status::GenericFailure, format!("模块未找到: {}", module_name))
        })?;

        if self.debug {
            println!("[DEBUG] {} 地址: {:#X} - {:#X}", module_name, info.start_address, info.end_address);
        }

        Ok(ModuleInfo {
            name: module_name,
            base_address: BigInt::from(info.start_address as u64),
            size: (info.end_address - info.start_address) as u32,
            end_address: BigInt::from(info.end_address as u64),
        })
    }

    /// 获取模块起始地址（兼容旧 API）
    #[napi]
    pub fn get_module_start_address(&self, module_name: String) -> Result<BigInt> {
        let info = find_module_info(self.pid, &module_name).ok_or_else(|| {
            Error::new(Status::GenericFailure, format!("模块未找到: {}", module_name))
        })?;
        Ok(BigInt::from(info.start_address as u64))
    }

    /// 获取模块结束地址（兼容旧 API）
    #[napi]
    pub fn get_module_end_address(&self, module_name: String) -> Result<BigInt> {
        let info = find_module_info(self.pid, &module_name).ok_or_else(|| {
            Error::new(Status::GenericFailure, format!("模块未找到: {}", module_name))
        })?;
        Ok(BigInt::from(info.end_address as u64))
    }

    /// 解析指针链
    #[napi]
    pub fn resolve_pointer_chain(&self, base_addr: BigInt, offsets: Vec<u32>) -> Result<BigInt> {
        let mut current = self.bigint_to_addr(base_addr)?;

        if offsets.is_empty() {
            return Ok(BigInt::from(current as u64));
        }

        for (i, &offset) in offsets.iter().take(offsets.len() - 1).enumerate() {
            let ptr_loc = current + offset as usize;

            let next = match self.arch {
                Arch::X86 => read_memory_raw::<u32>(self.handle, ptr_loc).map(|v| v as usize),
                Arch::X64 => read_memory_raw::<u64>(self.handle, ptr_loc).map(|v| v as usize),
            };

            match next {
                Ok(0) => {
                    return Err(Error::new(
                        Status::GenericFailure,
                        format!("第 {} 层指针为空", i + 1),
                    ));
                }
                Ok(val) => current = val,
                Err(e) => {
                    return Err(Error::new(
                        Status::GenericFailure,
                        format!("第 {} 层读取失败: {}", i + 1, e),
                    ));
                }
            }
        }

        if let Some(&last) = offsets.last() {
            Ok(BigInt::from((current + last as usize) as u64))
        } else {
            Ok(BigInt::from(current as u64))
        }
    }

    /// 读取缓冲区
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
                format!("读取缓冲区失败: {}", get_last_error_string()),
            ))
        }
    }

    /// 写入缓冲区
    #[napi]
    pub fn write_buffer(&self, addr: BigInt, buffer: Buffer) -> Result<()> {
        let addr_val = self.bigint_to_addr(addr)?;
        let data: &[u8] = &buffer;
        let mut bytes_written = 0;

        // 先尝试直接写入
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

        // 修改内存保护后重试
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
                format!("修改内存保护失败: {}", get_last_error_string()),
            ));
        }

        let retry = unsafe {
            WriteProcessMemory(
                self.handle,
                addr_val as *const c_void,
                data.as_ptr() as *const c_void,
                data.len(),
                Some(&mut bytes_written),
            )
        };

        // 恢复保护
        unsafe {
            let _ = VirtualProtectEx(
                self.handle,
                addr_val as *const c_void,
                data.len(),
                old_protect,
                &mut old_protect,
            );
        }

        retry.map_err(|_| {
            Error::new(
                Status::GenericFailure,
                format!("写入缓冲区失败: {}", get_last_error_string()),
            )
        })
    }

    /// 读取字符串（优化：批量读取）
    #[napi]
    pub fn read_string(&self, addr: BigInt, max_length: Option<u32>) -> Result<String> {
        let limit = max_length.unwrap_or(256) as usize;

        // 批量读取，避免逐字节跨进程调用
        let buffer = self.read_buffer(addr, limit as u32)?;
        let bytes: &[u8] = &buffer;

        // 查找 null 终止符
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());

        String::from_utf8(bytes[..end].to_vec())
            .map_err(|e| Error::new(Status::GenericFailure, format!("UTF-8 解码错误: {}", e)))
    }

    /// 获取当前进程架构
    #[napi]
    pub fn get_arch(&self) -> String {
        match self.arch {
            Arch::X64 => "x64".to_string(),
            Arch::X86 => "x86".to_string(),
        }
    }

    /// 获取当前进程 PID
    #[napi]
    pub fn get_pid(&self) -> u32 {
        self.pid
    }

    /// 读取指令字节（用于分析汇编）
    #[napi]
    pub fn read_instruction(&self, addr: BigInt, length: Option<u32>) -> Result<String> {
        let len = length.unwrap_or(16);
        let buffer = self.read_buffer(addr, len)?;
        let bytes: &[u8] = &buffer;
        
        // 返回十六进制格式，便于分析
        let hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
        Ok(hex.join(" "))
    }

    /// 写入指令字节（Patch 代码）
    #[napi]
    pub fn write_instruction(&self, addr: BigInt, hex_bytes: String) -> Result<()> {
        // 解析十六进制字符串，支持 "90 90 90" 或 "909090" 格式
        let cleaned: String = hex_bytes.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        
        if cleaned.len() % 2 != 0 {
            return Err(Error::new(Status::InvalidArg, "无效的十六进制字符串"));
        }

        let bytes: Result<Vec<u8>> = (0..cleaned.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&cleaned[i..i + 2], 16)
                    .map_err(|_| Error::new(Status::InvalidArg, "十六进制解析失败"))
            })
            .collect();

        self.write_buffer(addr, Buffer::from(bytes?))
    }

    /// NOP 填充指定地址（用空指令覆盖）
    #[napi]
    pub fn nop_instruction(&self, addr: BigInt, length: u32) -> Result<()> {
        let nops = vec![0x90u8; length as usize]; // 0x90 = NOP
        self.write_buffer(addr, Buffer::from(nops))
    }
}

// 宏：数值类型读写
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
                write_memory_raw::<$rust_type>(self.handle, addr_val, &(value as $rust_type))
                    .map_err(|e| Error::new(Status::GenericFailure, e))
            }
        }
    };
}

// 宏：BigInt 类型读写
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
                write_memory_raw::<$rust_type>(self.handle, addr_val, &(val_u64 as $rust_type))
                    .map_err(|e| Error::new(Status::GenericFailure, e))
            }
        }
    };
}

// 批量生成读写方法
impl_number_rw!(read_u8, write_u8, u8, u32);
impl_number_rw!(read_i8, write_i8, i8, i32);
impl_number_rw!(read_u16, write_u16, u16, u32);
impl_number_rw!(read_i16, write_i16, i16, i32);
impl_number_rw!(read_u32, write_u32, u32, u32);
impl_number_rw!(read_i32, write_i32, i32, i32);
impl_number_rw!(read_float, write_float, f32, f64);
impl_number_rw!(read_double, write_double, f64, f64);
impl_bigint_rw!(read_u64, write_u64, u64);
impl_bigint_rw!(read_i64, write_i64, i64);

impl Drop for MemoryTool {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_invalid() {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}
