import { native } from '../../../platform/native'

export type TranslationProvider = 'google' | 'baidu' | 'bing' | 'youdao' | 'deepl' | 'tencent'

export interface TranslationWebviewBounds {
  x: number
  y: number
  width: number
  height: number
}

export async function openTranslator(provider: TranslationProvider, bounds: TranslationWebviewBounds): Promise<void> {
  return native.tools.translator.openTranslator(provider, bounds)
}

export async function navigateTranslator(provider: TranslationProvider): Promise<void> {
  return native.tools.translator.navigateTranslator(provider)
}

export async function setTranslatorBounds(bounds: TranslationWebviewBounds): Promise<void> {
  return native.tools.translator.setTranslatorBounds(bounds)
}

export async function reloadTranslator(): Promise<void> {
  return native.tools.translator.reloadTranslator()
}

export async function closeTranslator(): Promise<void> {
  return native.tools.translator.closeTranslator()
}
