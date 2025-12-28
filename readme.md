# Memory Tool

[中文文档](./readme.zh-CN.md)

A high-performance Windows memory manipulation library built with Rust and N-API, providing process memory read/write, pointer chain resolution, module information retrieval, and instruction patching capabilities.

## Features

- 🚀 **High Performance**: Built with Rust for excellent performance and memory safety
- 🪟 **Windows Only**: Supports Windows 10/11, compatible with 32-bit and 64-bit processes
- 🔍 **Auto Architecture Detection**: Automatically detects target process architecture
- 📝 **TypeScript Friendly**: Complete TypeScript type definitions
- 🛡️ **Safe Operations**: Comprehensive permission checks and error handling
- � **HInstruction Patching**: Read/write machine code for code injection

## Requirements

- **OS**: Windows 10/11
- **Node.js**: 16.0.0+
- **Permissions**: Administrator privileges required for some operations

## Installation

```bash
npm install memory-tool
# or
pnpm add memory-tool
```

## Quick Start

```javascript
const { MemoryTool } = require('memory-tool');

// Recommended: Named parameters
const tool = MemoryTool.create({
  processName: 'game.exe',  // or use `pid: 1234`
  // archX64: true,         // optional, auto-detect by default
  debug: true,              // optional
});

// Alternative: Positional parameters
const tool2 = MemoryTool.createFromName('game.exe', true);
const tool3 = MemoryTool.createFromPid(1234, true);

console.log(`Arch: ${tool.getArch()}, PID: ${tool.getPid()}`);

// Read/Write memory
const value = tool.readI32(0x400000n);
tool.writeI32(0x400000n, 9999);
```

## API Reference

### Instance Creation

```typescript
interface CreateOptions {
  processName?: string;  // Process name (either this or pid)
  pid?: number;          // Process ID (either this or processName)
  archX64?: boolean;     // Optional, auto-detect by default
  debug?: boolean;       // Debug mode
}
```

| Method | Description |
|--------|-------------|
| `create(options)` | Create with named parameters (recommended) |
| `createFromName(name, debug?)` | Create by process name (auto-detect arch) |
| `createFromNameWithArch(name, isX64, debug?)` | Create by process name (manual arch) |
| `createFromPid(pid, debug?)` | Create by PID (auto-detect arch) |
| `createFromPidWithArch(pid, isX64, debug?)` | Create by PID (manual arch) |

### Process & Module

| Method | Description | Returns |
|--------|-------------|---------|
| `MemoryTool.getAllProcesses()` | List all running processes | `ProcessInfo[]` |
| `getModules()` | Get all loaded modules | `ModuleInfo[]` |
| `getModule(name)` | Get specific module info | `ModuleInfo` |
| `getArch()` | Get detected architecture | `string` |
| `getPid()` | Get process ID | `number` |

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
// Get module info (recommended)
const module = tool.getModule('game.exe');
console.log(`Base: ${module.baseAddress}, End: ${module.endAddress}, Size: ${module.size}`);

// Resolve pointer chain from module base
const addr = tool.resolvePointerChain(module.baseAddress, [0x10, 0x24]);
```

### Memory Read/Write

| Method | Type | Read Return | Write Param |
|--------|------|-------------|-------------|
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

### Buffer & String

| Method | Description |
|--------|-------------|
| `readBuffer(addr, length)` | Read byte buffer |
| `writeBuffer(addr, buffer)` | Write byte buffer |
| `readString(addr, maxLength?)` | Read null-terminated string (default 256) |

### Pointer Chain

```javascript
// Resolve: base + 0x10 -> ptr + 0x24 -> ptr + 0x8 -> final address
const baseAddr = tool.getModuleStartAddress('game.exe');
const finalAddr = tool.resolvePointerChain(baseAddr, [0x10, 0x24, 0x8]);
const hp = tool.readI32(finalAddr);
```

### Instruction Patching

| Method | Description |
|--------|-------------|
| `readInstruction(addr, length?)` | Read machine code as hex string |
| `writeInstruction(addr, hexBytes)` | Write machine code (e.g., `"90 90"` or `"9090"`) |
| `nopInstruction(addr, length)` | Fill with NOP (0x90) |

```javascript
// Read original instruction
const original = tool.readInstruction(patchAddr, 8);
console.log(`Original: ${original}`); // e.g., "29 C8 ..." (sub eax, ecx)

// Patch: SUB -> ADD
tool.writeInstruction(patchAddr, "01 C8"); // add eax, ecx

// Or NOP out the instruction
tool.nopInstruction(patchAddr, 2);
```

Common patches:
- `29` (sub) → `01` (add)
- `2B` (sub) → `03` (add)
- `74 XX` (je) → `90 90` (nop)
- `75 XX` (jnz) → `EB XX` (jmp)

## Error Handling

```javascript
try {
  const tool = MemoryTool.createFromName('game.exe');
  const value = tool.readI32(addr);
} catch (e) {
  if (e.message.includes('未找到进程')) {
    console.error('Process not found');
  } else if (e.message.includes('OpenProcess')) {
    console.error('Permission denied, run as administrator');
  }
}
```

## Build

```bash
pnpm install
pnpm build        # Release build
pnpm build:debug  # Debug build
```

## License

MIT
