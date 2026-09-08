/**
 * 二维码生成与识别（纯前端）。
 * 生成：qrcode；识别：jsQR（在 canvas 上取像素后解码）。
 */
import QRCode from 'qrcode'
import jsQR from 'jsqr'

export type ErrorLevel = 'L' | 'M' | 'Q' | 'H'

export interface QrGenOptions {
  errorCorrectionLevel: ErrorLevel
  margin: number
  width: number
}

/** 生成二维码，返回 PNG 的 data URL。 */
export async function generateQrDataUrl(text: string, opts: Partial<QrGenOptions> = {}): Promise<string> {
  return QRCode.toDataURL(text, {
    errorCorrectionLevel: opts.errorCorrectionLevel ?? 'M',
    margin: opts.margin ?? 2,
    width: opts.width ?? 320,
  })
}

/** 加载图片（Blob 或 URL）为 HTMLImageElement。 */
export function loadImage(blobOrUrl: Blob | string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const url = typeof blobOrUrl === 'string' ? blobOrUrl : URL.createObjectURL(blobOrUrl)
    const img = new Image()
    img.onload = () => { resolve(img); if (typeof blobOrUrl !== 'string') URL.revokeObjectURL(url) }
    img.onerror = (e) => { if (typeof blobOrUrl !== 'string') URL.revokeObjectURL(url); reject(e) }
    img.src = url
  })
}

/** 在 canvas 上取像素并用 jsQR 解码，返回文本或 null。 */
export function decodeQrFromImage(img: HTMLImageElement): string | null {
  const w = img.naturalWidth, h = img.naturalHeight
  if (!w || !h) return null
  const canvas = document.createElement('canvas')
  canvas.width = w
  canvas.height = h
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  if (!ctx) return null
  ctx.drawImage(img, 0, 0, w, h)
  const data = ctx.getImageData(0, 0, w, h)
  const res = jsQR(data.data, w, h)
  return res?.data ?? null
}

/** 从 Blob 解码，同时给出可预览的 data URL。 */
export async function decodeQrFromBlob(blob: Blob): Promise<{ text: string | null; dataUrl: string }> {
  const dataUrl = await blobToDataUrl(blob)
  const img = await loadImage(dataUrl)
  return { text: decodeQrFromImage(img), dataUrl }
}

export function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const r = new FileReader()
    r.onload = () => resolve(r.result as string)
    r.onerror = () => reject(r.error)
    r.readAsDataURL(blob)
  })
}

/** 尝试从系统剪贴板读取图片（需 WebView 支持 navigator.clipboard.read）。 */
export async function readClipboardImage(): Promise<Blob | null> {
  if (!navigator.clipboard?.read) return null
  const items = await navigator.clipboard.read()
  for (const item of items) {
    const type = item.types.find(t => t.startsWith('image/'))
    if (type) return await item.getType(type)
  }
  return null
}
