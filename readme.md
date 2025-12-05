# Memory Tool

一个基于 Rust 和 N-API 的 Windows 平台内存操作库，提供高性能的进程内存读写、指针链解析、模块信息获取等功能。

## 功能特性

- 🚀 **高性能**: 基于 Rust 开发，提供卓越的性能和内存安全性
- 🪟 **Windows 专用**: 仅支持 Windows 平台（Windows 10/11），兼容 32 位和 64 位进程
- 📝 **TypeScript 友好**: 完整的 TypeScript 类型定义和类型安全
- 🛡️ **安全操作**: 完善的权限检查、错误处理和内存保护机制
- 📚 **完整 API**: 支持多种数据类型读写（整数、浮点数、字符串、缓冲区）
- 🔍 **进程管理**: 进程枚举、模块查询、指针链解析等功能

## 系统要求

- **操作系统**: Windows 10/11（仅支持 Windows）
- **Node.js**: 16.0.0 或更高版本
- **权限**: 某些操作需要管理员权限

## 安装

```bash
npm install memory-tool
# 或
yarn add memory-tool
```

## 快速开始

```typescript
import { MemoryTool } from 'memory-tool';

// 通过进程名创建实例（推荐）
const memoryTool = MemoryTool.create_from_name(
    'notepad.exe',  // 进程名
    true,           // 是否 64 位进程
    false           // 调试模式（可选）
);

// 或通过 PID 创建
const memoryTool2 = MemoryTool.create_from_pid(
    1234,   // 进程 ID
    true,   // 是否 64 位进程
    false   // 调试模式（可选）
);

// 读取内存
const value = memoryTool.read_i32(0x400000n);
console.log('读取的值:', value);

// 写入内存
memoryTool.write_i32(0x400000n, 1234);
```

## API 文档

### 创建实例

#### `MemoryTool.create_from_name(processName, archIsX64, debugMode?)`

通过进程名创建 MemoryTool 实例。

**参数:**
- `processName` (string): 进程名称（如 `notepad.exe`）
- `archIsX64` (boolean): 目标进程是否为 64 位
- `debugMode?` (boolean): 启用调试模式，默认 `false`

**返回值:** `MemoryTool`

**示例:**
```typescript
const tool = MemoryTool.create_from_name('game.exe', true, true);
```

#### `MemoryTool.create_from_pid(pid, archIsX64, debugMode?)`

通过进程 ID 创建 MemoryTool 实例。

**参数:**
- `pid` (number): 进程 ID
- `archIsX64` (boolean): 目标进程是否为 64 位
- `debugMode?` (boolean): 启用调试模式，默认 `false`

**返回值:** `MemoryTool`

**示例:**
```typescript
const tool = MemoryTool.create_from_pid(1234, true);
```

### 进程和模块信息

#### `MemoryTool.get_modules()`

获取目标进程加载的所有模块。

**返回值:** `ProcessInfo[]`

**示例:**
```typescript
const modules = tool.get_modules();
modules.forEach(m => console.log(`${m.name} - PID: ${m.pid}`));
```

#### `MemoryTool.get_all_processes()`

获取系统所有运行中的进程列表（静态方法）。

**返回值:** `ProcessInfo[]`

**示例:**
```typescript
const processes = MemoryTool.get_all_processes();
const notepad = processes.find(p => p.name === 'notepad.exe');
```

**类型定义:**
```typescript
interface ProcessInfo {
    pid: number;      // 进程 ID 或模块基地址
    name: string;     // 进程名或模块名
}
```

### 模块地址操作

#### `MemoryTool.get_module_start_address(moduleName)`

获取指定模块的起始地址。

**参数:**
- `moduleName` (string): 模块名称（如 `kernel32.dll`）

**返回值:** `BigInt`

**示例:**
```typescript
const addr = tool.get_module_start_address('kernel32.dll');
console.log('0x' + addr.toString(16));
```

#### `MemoryTool.get_module_end_address(moduleName)`

获取指定模块的结束地址。

**参数:**
- `moduleName` (string): 模块名称

**返回值:** `BigInt`

**示例:**
```typescript
const start = tool.get_module_start_address('game.dll');
const end = tool.get_module_end_address('game.dll');
console.log(`模块范围: 0x${start.toString(16)} - 0x${end.toString(16)}`);
```

### 指针链解析

#### `MemoryTool.resolve_pointer_chain(baseAddr, offsets)`

解析多层指针链，获取最终地址。

**参数:**
- `baseAddr` (BigInt): 基地址
- `offsets` (number[]): 偏移量数组

**返回值:** `BigInt`

**说明:** 逐层读取指针，每层应用对应的偏移量。如果任何中间指针为空，将抛出错误。

**示例:**
```typescript
// 基地址 + 0x10 -> 指针A + 0x24 -> 指针B + 0x8 -> 目标值
const offsets = [0x10, 0x24, 0x8];
const finalAddr = tool.resolve_pointer_chain(0x400000n, offsets);
console.log('最终地址: 0x' + finalAddr.toString(16));
```

### 内存读写操作

#### 整数类型

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `read_u8(addr)` | 读取 8 位无符号整数 | `BigInt` | `number` |
| `write_u8(addr, value)` | 写入 8 位无符号整数 | `BigInt, number` | `void` |
| `read_i8(addr)` | 读取 8 位有符号整数 | `BigInt` | `number` |
| `write_i8(addr, value)` | 写入 8 位有符号整数 | `BigInt, number` | `void` |
| `read_u16(addr)` | 读取 16 位无符号整数 | `BigInt` | `number` |
| `write_u16(addr, value)` | 写入 16 位无符号整数 | `BigInt, number` | `void` |
| `read_i16(addr)` | 读取 16 位有符号整数 | `BigInt` | `number` |
| `write_i16(addr, value)` | 写入 16 位有符号整数 | `BigInt, number` | `void` |
| `read_u32(addr)` | 读取 32 位无符号整数 | `BigInt` | `number` |
| `write_u32(addr, value)` | 写入 32 位无符号整数 | `BigInt, number` | `void` |
| `read_i32(addr)` | 读取 32 位有符号整数 | `BigInt` | `number` |
| `write_i32(addr, value)` | 写入 32 位有符号整数 | `BigInt, number` | `void` |
| `read_u64(addr)` | 读取 64 位无符号整数 | `BigInt` | `BigInt` |
| `write_u64(addr, value)` | 写入 64 位无符号整数 | `BigInt, BigInt` | `void` |
| `read_i64(addr)` | 读取 64 位有符号整数 | `BigInt` | `BigInt` |
| `write_i64(addr, value)` | 写入 64 位有符号整数 | `BigInt, BigInt` | `void` |

#### 浮点数类型

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `read_float(addr)` | 读取 32 位浮点数 | `BigInt` | `number` |
| `write_float(addr, value)` | 写入 32 位浮点数 | `BigInt, number` | `void` |
| `read_double(addr)` | 读取 64 位浮点数 | `BigInt` | `number` |
| `write_double(addr, value)` | 写入 64 位浮点数 | `BigInt, number` | `void` |

#### 缓冲区和字符串

##### `MemoryTool.read_buffer(addr, length)`

读取指定长度的字节缓冲区。

**参数:**
- `addr` (BigInt): 起始地址
- `length` (number): 要读取的字节数

**返回值:** `Buffer`

**示例:**
```typescript
const buffer = tool.read_buffer(0x400000n, 16);
console.log(buffer.toString('hex'));
```

##### `MemoryTool.write_buffer(addr, buffer)`

写入字节缓冲区。

**参数:**
- `addr` (BigInt): 起始地址
- `buffer` (Buffer): 要写入的数据

**返回值:** `void`

**示例:**
```typescript
const data = Buffer.from([0x90, 0x90, 0x90]); // NOP 指令
tool.write_buffer(0x400000n, data);
```

##### `MemoryTool.read_string(addr, maxLength?)`

读取 null 终止的字符串。

**参数:**
- `addr` (BigInt): 字符串起始地址
- `maxLength?` (number): 最大读取长度，默认 `256`

**返回值:** `string`

**示例:**
```typescript
const str = tool.read_string(0x400000n, 128);
console.log('字符串:', str);
```

## 使用示例

### 基础内存读写

```typescript
import { MemoryTool } from 'memory-tool';

// 创建实例
const tool = MemoryTool.create_from_name('game.exe', true, false);

// 读取 32 位整数
const hp = tool.read_i32(0x400000n);
console.log('HP:', hp);

// 写入 32 位整数
tool.write_i32(0x400000n, 9999);

// 读取浮点数
const speed = tool.read_float(0x400004n);
console.log('速度:', speed);

// 读取字符串
const playerName = tool.read_string(0x400100n, 32);
console.log('玩家名:', playerName);
```

### 指针链解析

```typescript
// 场景：基地址 + 0x10 -> 指针A + 0x24 -> 指针B + 0x8 -> 目标值
const baseAddr = tool.get_module_start_address('game.dll');
const offsets = [0x10, 0x24, 0x8];

try {
    const finalAddr = tool.resolve_pointer_chain(baseAddr, offsets);
    const value = tool.read_i32(finalAddr);
    console.log('目标值:', value);
} catch (error) {
    console.error('指针链解析失败:', error.message);
}
```

### 进程和模块操作

```typescript
// 列出所有进程
const processes = MemoryTool.get_all_processes();
console.log('运行中的进程:');
processes.forEach(p => console.log(`  ${p.name} (PID: ${p.pid})`));

// 获取目标进程的模块
const tool = MemoryTool.create_from_name('game.exe', true);
const modules = tool.get_modules();
console.log('游戏模块:');
modules.forEach(m => console.log(`  ${m.name}`));

// 获取特定模块的地址范围
const gameStart = tool.get_module_start_address('game.dll');
const gameEnd = tool.get_module_end_address('game.dll');
console.log(`game.dll: 0x${gameStart.toString(16)} - 0x${gameEnd.toString(16)}`);
```

### 缓冲区操作

```typescript
// 读取原始字节
const buffer = tool.read_buffer(0x400000n, 32);
console.log('十六进制:', buffer.toString('hex'));

// 写入字节（例如 NOP 指令）
const nops = Buffer.alloc(10, 0x90);
tool.write_buffer(0x400000n, nops);
```

## 调试模式

启用调试模式可输出详细的操作日志，便于排查问题：

```typescript
const tool = MemoryTool.create_from_name('game.exe', true, true);
```

调试输出示例：
```
[DEBUG] MemoryTool初始化 - PID: 1234, 架构: X64
[INFO] 调试权限启用成功
[DEBUG] 进程句柄: HANDLE(0x123)
[DEBUG] 获取模块起始地址: game.dll
[DEBUG] 模块 game.dll 起始地址: 0x140000000
```

## 错误处理

所有操作都应使用 try-catch 进行错误处理：

```typescript
try {
    const value = tool.read_i32(0x400000n);
    console.log('读取成功:', value);
} catch (error) {
    console.error('读取失败:', error.message);
}
```

常见错误：
- **InvalidArg**: 参数无效（如负数地址、精度丢失）
- **GenericFailure**: 操作失败（权限不足、内存访问失败、进程不存在等）

**权限提示:** 某些操作需要管理员权限。如果遇到权限错误，请以管理员身份运行 Node.js。

## 开发

### 构建要求

- **Rust**: 1.70 或更高版本
- **Node.js**: 16.0.0 或更高版本
- **操作系统**: Windows 10/11

### 构建项目

```bash
# 安装依赖
npm install

# 发布版本构建（优化）
npm run build

# 开发版本构建
npm run dev

# 代码格式化
npm run format

# 清理构建产物
npm run clean
```

### 测试

```bash
npm test
```

## 架构设计

### 模块结构

```
src/
├── lib.rs                 # N-API 主接口
├── internal/
│   ├── mod.rs            # 模块导出
│   ├── arch.rs           # 架构定义和错误处理
│   ├── memory.rs         # 内存读写底层实现
│   ├── privilege.rs      # 权限提升（SeDebugPrivilege）
│   └── process.rs        # 进程和模块枚举
```

### 设计特点

- **模块化**: 清晰的职责分离，便于维护和扩展
- **类型安全**: 充分利用 Rust 的类型系统
- **错误处理**: 完善的错误传播和报告机制
- **内存保护**: 自动处理内存保护属性修改
- **架构兼容**: 支持 32 位和 64 位进程

## 安全说明

⚠️ **重要**: 此库具有系统级权限，能够访问和修改其他进程的内存。

**使用责任:**
- 仅用于合法目的，符合当地法律法规
- 修改内存可能导致程序崩溃或系统不稳定
- 需要管理员权限才能访问其他进程
- 某些杀毒软件可能误报，这是正常现象

**最佳实践:**
- 始终使用 try-catch 处理异常
- 在修改内存前备份原始数据
- 在测试环境中验证代码
- 避免修改系统关键进程

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 更新日志

### v1.0.0
- 完整的内存读写 API
- 指针链解析功能
- 进程和模块枚举
- 完整的 TypeScript 类型定义
- Windows 平台优化

## 相关资源

- [Windows API 文档](https://docs.microsoft.com/en-us/windows/win32/)
- [Rust Windows Crate](https://github.com/microsoft/windows-rs)
- [N-API 文档](https://nodejs.org/api/n_api.html)

## 反馈和支持

如有问题或建议，请通过 [GitHub Issues](https://github.com/PromiseAll/memory-tool/issues) 联系我们。