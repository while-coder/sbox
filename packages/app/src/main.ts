import { createApp } from 'vue'
import { configureNative } from '@sbox/ui/native'
import { NativeWeb } from '@sbox/ui/native-web'
import '@sbox/ui/styles.css'

async function start(): Promise<void> {
  const desktop = import.meta.env.MODE !== 'web' && '__TAURI_INTERNALS__' in window
  if (desktop) {
    const { NativeRust } = await import('./platform/native_rust')
    const { configureDesktopNative } = await import('./platform/native')
    configureDesktopNative(new NativeRust())
    const { setupLogger } = await import('./logger')
    setupLogger()
  } else configureNative(new NativeWeb())

  const applyColorScheme = () => {
    document.documentElement.dataset.theme
      = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  applyColorScheme()
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyColorScheme)

  const [{ default: App }, { createSboxRouter }] = await Promise.all([
    desktop ? import('./App.vue') : import('@sbox/ui/app'),
    import('@sbox/ui/router'),
  ])
  const options = desktop ? await import('./shell/registry').then(({ APP_TOOLS, APP_ROUTES }) => ({ tools: APP_TOOLS, routes: APP_ROUTES })) : {}
  createApp(App).use(createSboxRouter(options)).mount('#app')
}
void start()
