import { ref } from 'vue'
import { native } from '../platform/native'
const appVersion = ref('')
let versionInitialized = false
async function initUpdaterVersion(): Promise<void> {
  if (versionInitialized) return
  versionInitialized = true
  appVersion.value = await native.getAppVersion()
}
export function useUpdater() {
  return { ...native.updater, appVersion, initUpdaterVersion }
}
