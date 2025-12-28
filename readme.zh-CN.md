# Memory Tool

[English](./readme.md)

基于 Rust 和 N-API 的高性能 Windows 内存操作库，提供进程内存读写、指针链解析、模块信息获取和指令 Patch 功能。

## 功能特性

- 🚀 **高性能**: 基于 Rust 开发，卓越的性能和内存安全性
- 🪟 **Windows 专用**: 支持 Windows 10/11，兼容 32/64 位进程
- 🔍 **自动架构检测**: 自动检测目标进程是 32 位还是 64 位
- 📝 **TypeScript 友好**: 完整的类型定义
- 🛡️ **安全操作**: 完善的权限检查和错误处理
- 🔧 **指令 Patch**: 读写机器码，支持代码注入

## 系统要求

- **操作系统**: Windows 10/11
- **Node.js**: 16.0.0+
- **权限**: 部分操作需要管理员权限

## 安装

```bash
npm install memory-tool
# 或
pnpm add memory-tool
```

## 快速开始

```javascript
const { MemoryTool } = require('memory-tool');

// 推荐：具名参数
const tool = MemoryTool.create({
  processName: 'game.exe',  // 或使用 `pid: 1234`
  // archX64: true,         // 可选，默认自动检测
  debug: true,              // 可选
});

// 备选：位置参数
const tool2 = MemoryTool.createFromName('game.exe', true);
const tool3 = MemoryTool.createFromPid(1234, true);

console.log(`架构: ${tool.getArch()}, PID: ${tool.getPid()}`);

// 读写内存
const value = tool.readI32(0x400000n);
tool.writeI32(0x400000n, 9999);
```

## API 参考

### 创建实例

```typescript
interface CreateOptions {
  processName?: string;  // 进程名（与 pid 二选一）
  pid?: number;          // 进程 ID（与 processName 二选一）
  archX64?: boolean;     // 可选，默认自动检测
  debug?: boolean;       // 调试模式
}
```

| 方法 | 说明 |
|------|------|
| `create(options)` | 具名参数创建（推荐） |
| `createFromName(name, debug?)` | 通过进程名创建（自动检测架构） |
| `createFromNameWithArch(name, isX64, debug?)` | 通过进程名创建（手动指定架构） |
| `createFromPid(pid, debug?)` | 通过 PID 创建（自动检测架构） |
| `createFromPidWithArch(pid, isX64, debug?)` | 通过 PID 创建（手动指定架构） |

### 进程与模块

| 方法 | 说明 | 返回值 |
|------|------|--------|
| `MemoryTool.getAllProcesses()` | 获取所有运行中的进程 | `ProcessInfo[]` |
| `getModules()` | 获取所有已加载模块 | `ModuleInfo[]` |
| `getModule(name)` | 获取指定模块信息 | `ModuleInfo` |
| `getArch()` | 获取检测到的架构 | `string` |
| `getPid()` | 获取进程 ID | `number` |

```typescript
interface ProcessInfo {
  pid: number;
  name: string;
}

interface ModuleInfo {
  name: string;
  baseAddress: BigInt;
  size: number;
  endAddress: BigInt;
}
```

```javascript
// 获取模块信息（推荐）
const module = tool.getModule('game.exe');
console.log(`基址: ${module.baseAddress}, 结束: ${module.endAddress}, 大小: ${module.size}`);

// 从模块基址解析指针链
const addr = tool.resolvePointerChain(module.baseAddress, [0x10, 0x24]);
```

### 内存读写

| 方法 | 类型 | 读取返回 | 写入参数 |
|------|------|----------|----------|
| `readU8` / `writeU8` | uint8 | `number` | `number` |
| `readI8` / `writeI8` | int8 | `number` | `number` |
| `readU16` / `writeU16` | uint16 | `number` | `number` |
| `readI16` / `writeI16` | int16 | `number` | `number` |
| `readU32` / `writeU32` | uint32 | `number` | `number` |
| `readI32` / `writeI32` | int32 | `number` | `number` |
| `readU64` / `writeU64` | uint64 | `BigInt` | `BigInt` |
| `readI64` / `writeI64` | int64 | `BigInt` | `BigInt` |
| `readFloat` / `writeFloat` | float32 | `number` | `number` |
| `readDouble` / `writeDouble` | float64 | `number` | `number` |

### 缓冲区与字符串

| 方法 | 说明 |
|------|------|
| `readBuffer(addr, length)` | 读取字节缓冲区 |
| `writeBuffer(addr, buffer)` | 写入字节缓冲区 |
| `readString(addr, maxLength?)` | 读取 null 结尾字符串（默认 256） |

### 指针链解析

```javascript
// 解析: 基址 + 0x10 -> 指针 + 0x24 -> 指针 + 0x8 -> 最终地址
const baseAddr = tool.getModuleStartAddress('game.exe');
const finalAddr = tool.resolvePointerChain(baseAddr, [0x10, 0x24, 0x8]);
const hp = tool.readI32(finalAddr);
```

### 指令 Patch

| 方法 | 说明 |
|------|------|
| `readInstruction(addr, length?)` | 读取机器码，返回十六进制字符串 |
| `writeInstruction(addr, hexBytes)` | 写入机器码（如 `"90 90"` 或 `"9090"`） |
| `nopInstruction(addr, length)` | 用 NOP (0x90) 填充 |

```javascript
// 读取原始指令
const original = tool.readInstruction(patchAddr, 8);
console.log(`原始指令: ${original}`); // 如 "29 C8 ..." (sub eax, ecx)

// Patch: SUB -> ADD
tool.writeInstruction(patchAddr, "01 C8"); // add eax, ecx

// 或者 NOP 掉指令
tool.nopInstruction(patchAddr, 2);
```

常见 Patch：
- `29` (sub) → `01` (add)
- `2B` (sub) → `03` (add)
- `74 XX` (je) → `90 90` (nop)
- `75 XX` (jnz) → `EB XX` (jmp)

## 错误处理

```javascript
try {
  const tool = MemoryTool.createFromName('game.exe');
  const value = tool.readI32(addr);
} catch (e) {
  if (e.message.includes('未找到进程')) {
    console.error('进程未运行，请先启动目标程序');
  } else if (e.message.includes('OpenProcess')) {
    console.error('权限不足，请以管理员身份运行');
  }
}
```

## 构建

```bash
pnpm install
pnpm build        # 发布构建
pnpm build:debug  # 调试构建
```

## License

MIT
