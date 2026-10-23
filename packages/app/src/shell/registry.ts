import { ALL_TOOLS, type ToolDef } from '@sbox/ui/registry'
import type { RouteRecordRaw } from 'vue-router'
import { AppWindow, FileSearch2, FolderKey, KeyRound, MonitorUp, ScanText } from 'lucide-vue-next'
import { systemInfoTool } from '../tools/account/system-info'
import { xiaoaiLoginTool } from '../tools/account/xiaoai-login'
import { gdriveLoginTool } from '../tools/account/gdrive-login'
import { fileLocksTool } from '../tools/devtool/file-locks'
import { batchRenameTool } from '../tools/file/batch-rename'
import { portCheckTool } from '../tools/devtool/port-check'
import { envVarsTool } from '../tools/devtool/env-vars'
import { screenshotTool } from '../tools/media/screenshot'
import { keystoreTool } from '../tools/genverify/keystore'
import { sshKeygenTool } from '../tools/genverify/ssh-keygen'
import { translatorTool } from '../tools/devtool/translator'

export const APP_TOOLS: ToolDef[] = [
  ...ALL_TOOLS,
  { ...systemInfoTool },
  { ...fileLocksTool, icon: FileSearch2 },
  batchRenameTool, portCheckTool, envVarsTool,
  { ...screenshotTool, icon: MonitorUp },
  { ...keystoreTool, icon: FolderKey },
  { ...sshKeygenTool, icon: KeyRound },
  { ...translatorTool, icon: ScanText },
  { ...xiaoaiLoginTool, icon: AppWindow },
  { ...gdriveLoginTool, icon: AppWindow },
]
export const APP_ROUTES: RouteRecordRaw[] = [
  { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  { path: '/screenshot-overlay', name: 'screenshot-overlay', meta: { bare: true }, component: () => import('../tools/media/screenshot/ScreenshotOverlay.vue') },
]
