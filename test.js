const { MemoryTool } = require('./');

const CONFIG = {
  PROCESS_NAME: 'game_demo.exe',
  MODULE_NAME: 'game_demo.exe',
  OFFSETS: [0x00009080, 0x4],
  DEBUG_MODE: true,
};

/**
 * 常用 x86/x64 指令对照表
 * 
 * SUB 指令:
 *   29 C8    - sub eax, ecx     (32位)
 *   48 29 C8 - sub rax, rcx     (64位)
 *   83 E8 XX - sub eax, XX      (立即数)
 *   
 * ADD 指令:
 *   01 C8    - add eax, ecx     (32位)
 *   48 01 C8 - add rax, rcx     (64位)
 *   83 C0 XX - add eax, XX      (立即数)
 *   
 * NOP: 90
 * 
 * 常见 Patch 场景:
 *   sub -> add: 29 -> 01, 2B -> 03
 *   jnz -> jmp: 75 -> EB (短跳转)
 *   je  -> nop: 74 XX -> 90 90
 */

function handleError(e) {
  const msg = e.message || '';
  
  if (msg.includes('未找到进程')) {
    console.error('[错误] 进程未运行，请先启动目标程序');
  } else if (msg.includes('OpenProcess')) {
    console.error('[错误] 权限不足，请以管理员身份运行');
  } else if (msg.includes('模块未找到')) {
    console.error('[错误] 模块未加载，目标程序可能还在初始化');
  } else if (msg.includes('指针为空')) {
    console.error('[错误] 指针链断裂，偏移可能已失效');
  } else {
    console.error('[错误]', msg);
  }
}

function formatAddr(addr) {
  return `0x${addr.toString(16).toUpperCase()}`;
}

function main() {
  let tool = null;

  try {
    // 使用具名参数创建（推荐）
    tool = MemoryTool.create({
      processName: CONFIG.PROCESS_NAME,
      debug: CONFIG.DEBUG_MODE,
    });
    console.log(`[信息] 进程架构: ${tool.getArch()}, PID: ${tool.getPid()}`);

    // 获取模块信息（新 API，一次获取完整信息）
    const module = tool.getModule(CONFIG.MODULE_NAME);
    console.log(`[信息] 模块地址: ${formatAddr(module.baseAddress)} - ${formatAddr(module.endAddress)}`);

    // ========== 数值修改示例 ==========
    const targetAddr = tool.resolvePointerChain(module.baseAddress, CONFIG.OFFSETS);
    console.log(`[信息] 目标地址: ${formatAddr(targetAddr)}`);

    const oldVal = tool.readU32(targetAddr);
    console.log(`[信息] 当前血量: ${oldVal}`);

    tool.writeU32(targetAddr, 9999);
    const verifyVal = tool.readU32(targetAddr);
    console.log(`[成功] 血量修改: ${oldVal} -> ${verifyVal}`);

    // ========== 汇编 Patch 示例 ==========
    // 假设扣血指令在 模块基址 + 0x1234 位置
    // 你需要用 CE 或 x64dbg 找到实际地址
    
    // const patchAddr = moduleStart + 0x1234n; // 替换为实际偏移
    
    // 1. 读取原始指令
    // const originalBytes = tool.readInstruction(patchAddr, 8);
    // console.log(`[信息] 原始指令: ${originalBytes}`);
    
    // 2. 方案A: SUB -> ADD (修改操作码)
    // 如果原始是 "29 C8" (sub eax, ecx)，改成 "01 C8" (add eax, ecx)
    // tool.writeInstruction(patchAddr, "01 C8");
    
    // 3. 方案B: NOP 掉整条指令（跳过扣血逻辑）
    // tool.nopInstruction(patchAddr, 2); // NOP 2字节
    
    // 4. 验证修改
    // const patchedBytes = tool.readInstruction(patchAddr, 8);
    // console.log(`[信息] 修改后: ${patchedBytes}`);

  } catch (e) {
    handleError(e);
  } finally {
    tool = null;
  }
}

main();
