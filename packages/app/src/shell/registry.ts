/**
 * 桌面端工具注册表 = 共享工具（@sbox/tools-core 的 WEB_TOOLS）+ 仅桌面可用的 native 工具。
 * 各 native 工具的定义（ToolDef）放在 tools/<key>/index.ts 中，此处只做汇总。
 * native 工具依赖本机能力（全屏截图 / 本机 Java / 绕 CORS 的 HTTP / OAuth 本地回调），无法上 Web。
 */
import { WEB_TOOLS, type ToolDef } from '@sbox/tools-core'
import { systemInfoTool } from '../tools/account/system-info'
import { xiaoaiLoginTool } from '../tools/account/xiaoai-login'
import { gdriveLoginTool } from '../tools/account/gdrive-login'
import { fileLocksTool } from '../tools/devtool/file-locks'
import { batchRenameTool } from '../tools/file/batch-rename'
import { portCheckTool } from '../tools/devtool/port-check'
import { envVarsTool } from '../tools/devtool/env-vars'
import { translatorTool } from '../tools/devtool/translator'
import { keystoreTool } from '../tools/genverify/keystore'
import { sshKeygenTool } from '../tools/genverify/ssh-keygen'
import { screenshotTool } from '../tools/media/screenshot'

/** 仅桌面（Tauri）可用的工具。 */
export const NATIVE_TOOLS: ToolDef[] = [
  systemInfoTool,
  fileLocksTool,
  batchRenameTool,
  portCheckTool,
  envVarsTool,
  screenshotTool,
  keystoreTool,
  sshKeygenTool,
  translatorTool,
  xiaoaiLoginTool,
  gdriveLoginTool,
]

/** 桌面端完整工具列表。 */
export const ALL_TOOLS: ToolDef[] = [...WEB_TOOLS, ...NATIVE_TOOLS]
