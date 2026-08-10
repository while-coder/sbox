import { invoke } from '@tauri-apps/api/core'

export type TranslationProvider = 'google' | 'baidu' | 'bing' | 'youdao' | 'deepl' | 'tencent'

export interface TranslationWebviewBounds {
  x: number
  y: number
  width: number
  height: number
}

export async function openTranslator(provider: TranslationProvider, bounds: TranslationWebviewBounds): Promise<void> {
  await invoke('translator_open', { provider, bounds })
}

export async function navigateTranslator(provider: TranslationProvider): Promise<void> {
  await invoke('translator_navigate', { provider })
}

export async function setTranslatorBounds(bounds: TranslationWebviewBounds): Promise<void> {
  await invoke('translator_set_bounds', { bounds })
}

export async function reloadTranslator(): Promise<void> {
  await invoke('translator_reload')
}

export async function closeTranslator(): Promise<void> {
  await invoke('translator_close')
}
