import type { ToolDef } from '../../../shell/registry'

/** 网页翻译。 */
export const translatorTool: ToolDef = {
  key: 'translator',
  label: '网页翻译',
  description: '在应用内切换使用 Google、百度、Bing、有道、DeepL 和混元翻译',
  category: 'devtool',
  keywords: ['translate', 'translator', 'google translate', 'baidu fanyi', 'bing translator', 'youdao', 'deepl', 'tencent', 'hunyuan', '翻译', '谷歌翻译', '百度翻译', '必应翻译', '有道翻译', '混元翻译'],
  component: () => import('./TranslatorView.vue'),
}
