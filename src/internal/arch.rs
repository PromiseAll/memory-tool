use windows::Win32::Foundation::GetLastError;

/// 处理器架构枚举
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Arch {
    /// 32位x86架构
    X86,
    /// 64位x64架构
    X64,
}

/// 获取最后一次Win32错误的字符串描述
///
/// # 返回值
/// 格式化的错误描述字符串，格式为"Win32 Error: {error_code}"
pub fn get_last_error_string() -> String {
    let err = unsafe { GetLastError() };
    format!("Win32 Error: {:?}", err)
}

/// 将i8数组转换为String（处理null终止符）
///
/// 此函数用于将Windows API返回的i8数组转换为Rust字符串，
/// 自动处理null终止符并支持UTF-8编码。
///
/// # 参数
/// * `arr` - 输入的i8数组
///
/// # 返回值
/// 转换后的String，如果编码无效则返回替换字符
pub fn i8_to_string(arr: &[i8]) -> String {
    unsafe {
        // 查找null终止符位置
        let len = arr.iter().position(|&x| x == 0).unwrap_or(arr.len());
        // 将i8数组转换为u8数组并创建字符串切片
        let slice = std::slice::from_raw_parts(arr.as_ptr() as *const u8, len);
        // 使用lossy转换，处理无效UTF-8序列
        String::from_utf8_lossy(slice).into_owned()
    }
}
