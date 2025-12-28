# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2024-12-28

### ✨ Features

- **create**: Add named parameters support with `CreateOptions`
- **module**: Add `getModule()` method returning complete `ModuleInfo`
- **arch**: Auto-detect target process architecture (x86/x64)
- **patch**: Add instruction patching APIs (`readInstruction`, `writeInstruction`, `nopInstruction`)
- **info**: Add `getArch()` and `getPid()` methods

### ♻️ Refactor

- **ModuleInfo**: Include `baseAddress`, `size`, and `endAddress` in single struct
- **process**: Fix 64-bit address truncation in `getModules()`
- **string**: Optimize `readString()` with batch reading instead of byte-by-byte

### 📚 Documentation

- Add bilingual documentation (English & Chinese)
- Update API reference with new methods

## [1.0.0] - 2024-12-27

### ✨ Features

- Initial release
- Process memory read/write (u8, i8, u16, i16, u32, i32, u64, i64, float, double)
- Buffer and string operations
- Pointer chain resolution
- Module enumeration
- Debug privilege elevation
- TypeScript type definitions
