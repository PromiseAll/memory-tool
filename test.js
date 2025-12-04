const { MemoryTool } = require('./');

const CONFIG = {
  PROCESS_NAME: 'PlantsVsZombies.exe',
  MODULE_NAME: 'PlantsVsZombies.exe',
  OFFSETS: [0xfa428, 0x18, 0x34, 0x1c, 0x5560],
  ARCH_IS_X64: false,
};

async function main() {
  try {
    const tool = await MemoryTool.createFromName(CONFIG.PROCESS_NAME, CONFIG.ARCH_IS_X64, true);
    const moduleStartAddress = tool.getModuleStartAddress(CONFIG.MODULE_NAME);
    const moduleEndAddress = tool.getModuleEndAddress(CONFIG.MODULE_NAME);
    console.log(`>>> 模块起始地址: ${moduleStartAddress.toString(16)}`);
    console.log(`>>> 模块结束地址: ${moduleEndAddress.toString(16)}`);
    const finalAddr = tool.resolvePointerChain(moduleStartAddress, CONFIG.OFFSETS);

    // 读取
    const val = tool.readU32(finalAddr);
    console.log(`>>> 当前阳光: ${val}`);

    // 写入
    tool.writeU32(finalAddr, 9999);
    console.log('>>> 写入完成');
  } catch (e) {
    console.error('JS Error:', e);
  }
}

main();
