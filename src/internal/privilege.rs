use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueA, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::PCSTR;

/// 启用当前进程的调试权限
///
/// 此函数尝试为当前进程启用SeDebugPrivilege权限，
/// 这允许进程访问其他进程的内存空间。
///
/// # 返回值
/// 成功返回Ok(()), 失败返回Err(error_message)
///
/// # 权限要求
/// 需要管理员权限或具备相关权限的用户账户
pub fn enable_debug_privilege() -> std::result::Result<(), String> {
    unsafe {
        // 获取当前进程的访问令牌
        let mut token: HANDLE = HANDLE(std::ptr::null_mut());
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )
        .is_err()
        {
            return Err("无法打开进程令牌".into());
        }

        // 查找SeDebugPrivilege权限的LUID值
        let mut luid = LUID::default();
        let name = PCSTR::from_raw("SeDebugPrivilege\0".as_ptr());
        if LookupPrivilegeValueA(PCSTR::null(), name, &mut luid).is_err() {
            let _ = CloseHandle(token);
            return Err("无法查找Debug权限值".into());
        }

        // 构建权限结构体
        let mut tkp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            ..Default::default()
        };
        tkp.Privileges[0].Luid = luid;
        tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        // 调整令牌权限
        let res = AdjustTokenPrivileges(token, false, Some(&tkp), 0, None, None);
        let _ = CloseHandle(token);

        if res.is_err() {
            return Err("AdjustTokenPrivileges失败".into());
        }

        // 检查是否有部分权限未分配
        if GetLastError().is_err() {
            return Err("提权部分失败 (ERROR_NOT_ALL_ASSIGNED)".into());
        }
        Ok(())
    }
}
