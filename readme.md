# Memory Tool

一个基于Rust和N-API的Windows平台内存操作库，支持读写进程内存、解析指针链、操作进程模块等功能。

## 功能特性

- 🚀 **高性能**: 基于Rust语言开发，具有卓越的性能和内存安全性
- 🔧 **跨平台**: 支持Windows平台，兼容32位和64位进程
- 📝 **TypeScript友好**: 完整的TypeScript类型定义
- 🛡️ **安全操作**: 安全的内存读写，带权限检查和错误处理
- 📚 **完整API**: 支持多种数据类型读写（整数、浮点数、字符串、缓冲区）

## 安装

```bash
npm install memory_tool
# 或者
yarn add memory_tool
```

## 快速开始

```typescript
import { MemoryTool } from 'memory_tool';

// 通过进程名创建实例（推荐）
const memoryTool = MemoryTool.create_from_name(
    'notepad.exe',    // 进程名
    true,             // 是否64位进程
    false             // 调试模式
);

// 或者通过PID创建
const memoryTool2 = MemoryTool.create_from_pid(
    1234,    // 进程ID
    true,    // 是否64位进程
    false    // 调试模式
);
```

## API文档

### 创建MemoryTool实例

#### `MemoryTool.create_from_name(processName, archIsX64, debugMode)`

通过进程名创建MemoryTool实例。

**参数:**
- `processName` (string): 进程名称（不含路径）
- `archIsX64` (boolean): 目标进程是否为64位
- `debugMode` (boolean, 可选): 是否启用调试模式，默认false

**返回值:** MemoryTool

#### `MemoryTool.create_from_pid(pid, archIsX64, debugMode)`

通过进程ID创建MemoryTool实例。

**参数:**
- `pid` (number): 进程ID
- `archIsX64` (boolean): 目标进程是否为64位
- `debugMode` (boolean, 可选): 是否启用调试模式，默认false

**返回值:** MemoryTool

### 进程信息

#### `MemoryTool.get_modules()`

获取当前进程的所有模块列表。

**返回值:** ProcessInfo[]

```typescript
interface ProcessInfo {
    pid: number;
    name: string;
}
```

#### `MemoryTool.get_all_processes()`

静态方法，获取系统所有运行中的进程列表。

**返回值:** ProcessInfo[]

### 模块地址操作

#### `MemoryTool.get_module_start_address(moduleName)`

获取指定模块的起始地址。

**参数:**
- `moduleName` (string): 模块名称

**返回值:** BigInt

#### `MemoryTool.get_module_end_address(moduleName)`

获取指定模块的结束地址。

**参数:**
- `moduleName` (string): 模块名称

**返回值:** BigInt

### 指针链解析

#### `MemoryTool.resolve_pointer_chain(moduleBase, offsets)`

解析多层指针链，获取最终地址。

**参数:**
- `moduleBase` (BigInt): 模块基地址
- `offsets` (number[]): 偏移量数组

**返回值:** BigInt

### 内存读写操作

#### 基础数值类型

| 方法名 | 描述 | 参数 | 返回值 |
|--------|------|------|--------|
| `read_u8(addr)` | 读取8位无符号整数 | BigInt | number |
| `write_u8(addr, value)` | 写入8位无符号整数 | BigInt, number | void |
| `read_i8(addr)` | 读取8位有符号整数 | BigInt | number |
| `write_i8(addr, value)` | 写入8位有符号整数 | BigInt, number | void |
| `read_u16(addr)` | 读取16位无符号整数 | BigInt | number |
| `write_u16(addr, value)` | 写入16位无符号整数 | BigInt, number | void |
| `read_i16(addr)` | 读取16位有符号整数 | BigInt | number |
| `write_i16(addr, value)` | 写入16位有符号整数 | BigInt, number | void |
| `read_u32(addr)` | 读取32位无符号整数 | BigInt | number |
| `write_u32(addr, value)` | 写入32位无符号整数 | BigInt, number | void |
| `read_i32(addr)` | 读取32位有符号整数 | BigInt | number |
| `write_i32(addr, value)` | 写入32位有符号整数 | BigInt, number | void |

#### 浮点数类型

| 方法名 | 描述 | 参数 | 返回值 |
|--------|------|------|--------|
| `read_float(addr)` | 读取32位浮点数 | BigInt | number |
| `write_float(addr, value)` | 写入32位浮点数 | BigInt, number | void |
| `read_double(addr)` | 读取64位浮点数 | BigInt | number |
| `write_double(addr, value)` | 写入64位浮点数 | BigInt, number | void |

#### 大整数类型

| 方法名 | 描述 | 参数 | 返回值 |
|--------|------|------|--------|
| `read_u64(addr)` | 读取64位无符号整数 | BigInt | BigInt |
| `write_u64(addr, value)` | 写入64位无符号整数 | BigInt, BigInt | void |
| `read_i64(addr)` | 读取64位有符号整数 | BigInt | BigInt |
| `write_i64(addr, value)` | 写入64位有符号整数 | BigInt, BigInt | void |

#### 特殊数据类型

##### `MemoryTool.read_buffer(addr, length)`

读取指定长度的字节缓冲区。

**参数:**
- `addr` (BigInt): 起始地址
- `length` (number): 要读取的字节数

**返回值:** Buffer

##### `MemoryTool.write_buffer(addr, buffer)`

写入字节缓冲区。

**参数:**
- `addr` (BigInt): 起始地址
- `buffer` (Buffer): 要写入的字节数据

**返回值:** void

##### `MemoryTool.read_string(addr, maxLength?)`

读取null终止的字符串。

**参数:**
- `addr` (BigInt): 字符串起始地址
- `maxLength` (number, 可选): 最大读取长度，默认256

**返回值:** string

## 使用示例

### 基础内存读写

```typescript
import { MemoryTool } from 'memory_tool';

const memoryTool = MemoryTool.create_from_name('notepad.exe', true, true);

// 读取整数
const value = memoryTool.read_i32(0x400000n);
console.log('读取的值:', value);

// 写入整数
memoryTool.write_i32(0x400000n, 1234);

// 读取字符串
const str = memoryTool.read_string(0x400000n);
console.log('读取的字符串:', str);
```

### 指针链解析

```typescript
// 假设有这样的指针链：基地址 + 0x10 -> 指针A + 0x24 -> 指针B + 0x8 -> 目标值
const offsets = [0x10, 0x24, 0x8];
const finalAddr = memoryTool.resolve_pointer_chain(moduleBase, offsets);
console.log('最终地址:', finalAddr.toString(16));
```

### 模块操作

```typescript
// 获取模块信息
const modules = memoryTool.get_modules();
console.log('进程模块:', modules);

// 获取特定模块地址
const kernel32Start = memoryTool.get_module_start_address('kernel32.dll');
const kernel32End = memoryTool.get_module_end_address('kernel32.dll');
console.log('kernel32.dll地址范围:', kernel32Start, '-', kernel32End);
```

## 调试模式

启用调试模式后，库会输出详细的调试信息：

```typescript
const memoryTool = MemoryTool.create_from_name('notepad.exe', true, true);
```

调试输出示例：
```
[DEBUG] MemoryTool初始化 - PID: 1234, 架构: X64
[INFO] 调试权限启用成功
[DEBUG] 进程句柄: HANDLE(0x123)
[DEBUG] 获取模块起始地址: kernel32.dll
[DEBUG] 模块 kernel32.dll 起始地址: 0x7FFE1234
```

## 错误处理

库提供了完善的错误处理机制：

```typescript
try {
    const value = memoryTool.read_i32(0x0n);
} catch (error) {
    console.error('内存读取失败:', error.message);
    // 处理错误...
}
```

常见错误类型：
- `InvalidArg`: 参数无效（如负数地址）
- `GenericFailure`: 操作失败（如进程权限不足、内存访问失败）

## 开发

### 系统要求

- Rust 1.70+
- Node.js 16+
- Windows 10/11

### 构建

```bash
# 安装依赖
npm install

# 构建Rust库
npm run build

# 开发模式构建
npm run dev
```

### 测试

```bash
npm test
```

## 安全说明

⚠️ **重要提醒**: 此库具有系统级权限，能够访问和修改其他进程的内存。使用时请注意：

1. **仅用于合法目的**: 请确保你的使用符合当地法律法规
2. **权限要求**: 某些操作需要管理员权限
3. **数据安全**: 修改内存可能导致程序崩溃，请谨慎操作
4. **病毒扫描**: 某些杀毒软件可能会误报，这是正常现象

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 贡献

欢迎提交Issue和Pull Request！

## 更新日志

### v0.1.0
- 初始版本发布
- 支持基础内存读写操作
- 支持指针链解析
- 支持模块信息获取
- 完整的TypeScript类型定义

## 联系方式

如有问题或建议，请通过GitHub Issues联系我们。