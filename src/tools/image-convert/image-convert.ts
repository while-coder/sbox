/**
 * 图片格式转换 —— 前端实现。
 * 解码/缩放/编码全部由 @imagemagick/magick-wasm(WASM 版 ImageMagick)在前端完成，
 * 覆盖常规位图、SVG 矢量以及 HEIC/HEIF/AVIF 等；落盘复用通用命令 save_base64_file。
 */
import { invoke } from '@tauri-apps/api/core'
import {
  ImageMagick, initializeImageMagick, MagickFormat, MagickGeometry,
  MagickColors, AlphaAction,
} from '@imagemagick/magick-wasm'
// Vite：以 URL 引用包内 .wasm，运行时再加载（不打进 JS bundle）。
import wasmUrl from '@imagemagick/magick-wasm/magick.wasm?url'

/** WASM 懒加载单例：首次转换时初始化，之后复用。 */
let initPromise: Promise<unknown> | null = null
function ensureMagick(): Promise<unknown> {
  return (initPromise ??= initializeImageMagick(new URL(wasmUrl, import.meta.url)))
}

/** 可输出（可编码）的目标格式。 */
export interface OutputFormat {
  /** 目标 key（UI 取值） */
  value: string
  label: string
  /** 保存时的扩展名 */
  ext: string
  /** ImageMagick 编码格式 */
  magick: MagickFormat
  mime: string
  /** 是否有损（决定是否显示质量滑块） */
  lossy: boolean
}

export const OUTPUT_FORMATS: OutputFormat[] = [
  { value: 'png', label: 'PNG', ext: 'png', magick: MagickFormat.Png, mime: 'image/png', lossy: false },
  { value: 'jpeg', label: 'JPEG', ext: 'jpg', magick: MagickFormat.Jpeg, mime: 'image/jpeg', lossy: true },
  { value: 'webp', label: 'WebP', ext: 'webp', magick: MagickFormat.WebP, mime: 'image/webp', lossy: true },
  { value: 'heic', label: 'HEIC', ext: 'heic', magick: MagickFormat.Heic, mime: 'image/heic', lossy: true },
  { value: 'avif', label: 'AVIF', ext: 'avif', magick: MagickFormat.Avif, mime: 'image/avif', lossy: true },
  { value: 'gif', label: 'GIF', ext: 'gif', magick: MagickFormat.Gif, mime: 'image/gif', lossy: false },
  { value: 'bmp', label: 'BMP', ext: 'bmp', magick: MagickFormat.Bmp, mime: 'image/bmp', lossy: false },
  { value: 'ico', label: 'ICO（图标，≤256px）', ext: 'ico', magick: MagickFormat.Ico, mime: 'image/x-icon', lossy: false },
  { value: 'tiff', label: 'TIFF', ext: 'tiff', magick: MagickFormat.Tiff, mime: 'image/tiff', lossy: false },
  { value: 'tga', label: 'TGA', ext: 'tga', magick: MagickFormat.Tga, mime: 'image/x-tga', lossy: false },
  { value: 'qoi', label: 'QOI', ext: 'qoi', magick: MagickFormat.Qoi, mime: 'image/qoi', lossy: false },
]

/** 输入侧支持的格式说明（含 HEIC/HEIF/AVIF 等，由 ImageMagick 解码）。 */
export const INPUT_HINT =
  'HEIC · HEIF · AVIF · PNG · JPEG · WebP · GIF · BMP · ICO · TIFF · TGA · QOI · PNM · DDS · HDR · EXR · SVG'

export interface ConvertOptions {
  target: string
  quality?: number
  width?: number
  height?: number
  keepAspect?: boolean
}

export interface ConvertResult {
  bytes: Uint8Array
  width: number
  height: number
  mime: string
}

export interface ConvertFileResult {
  width: number
  height: number
  mime: string
  /** 编码后字节数 */
  bytes: number
  /** 输出文件路径 */
  path: string
}

/** 从文件名取小写扩展名（无点）。 */
export function extOf(name: string): string {
  const i = name.lastIndexOf('.')
  return i >= 0 ? name.slice(i + 1).toLowerCase() : ''
}

/**
 * 依据宽/高与 keepAspect 生成 ImageMagick geometry 字符串；都为空返回 null（不缩放）。
 * 单维（仅宽或仅高）天然按比例缩放；双维时 keepAspect=false 用 `!` 强制精确尺寸。
 */
function geometryStr(opts: ConvertOptions): string | null {
  const w = opts.width && opts.width > 0 ? opts.width : undefined
  const h = opts.height && opts.height > 0 ? opts.height : undefined
  if (w && h) return opts.keepAspect === false ? `${w}x${h}!` : `${w}x${h}`
  if (w) return `${w}x`
  if (h) return `x${h}`
  return null
}

/** 前端转换：输入字节 → 目标格式字节。 */
export async function convert(input: Uint8Array, options: ConvertOptions): Promise<ConvertResult> {
  await ensureMagick()
  const fmt = OUTPUT_FORMATS.find(f => f.value === options.target)
  if (!fmt) throw new Error(`不支持的目标格式: ${options.target}`)

  return ImageMagick.read(input, (image): ConvertResult => {
    const geo = geometryStr(options)
    if (geo) image.resize(new MagickGeometry(geo))

    // ICO 上限 256×256，超出时再 fit 一次。
    if (fmt.value === 'ico' && (image.width > 256 || image.height > 256)) {
      image.resize(new MagickGeometry('256x256'))
    }

    // JPEG 不支持透明，铺白底（沿用旧行为）。
    if (fmt.value === 'jpeg') {
      image.backgroundColor = MagickColors.White
      image.alpha(AlphaAction.Remove)
    }

    if (fmt.lossy && options.quality != null) {
      image.quality = Math.min(100, Math.max(1, options.quality))
    }

    const width = image.width
    const height = image.height
    return image.write(fmt.magick, (data): ConvertResult => ({
      bytes: new Uint8Array(data), // 拷出，离开回调后 magick 内存失效
      width,
      height,
      mime: fmt.mime,
    }))
  })
}

/** 转换并写入指定路径（复用 Rust 通用命令 save_base64_file）。 */
export async function convertToFile(
  input: Uint8Array,
  outputPath: string,
  options: ConvertOptions,
): Promise<ConvertFileResult> {
  const res = await convert(input, options)
  await invoke('save_base64_file', { path: outputPath, base64: bytesToBase64(res.bytes) })
  return { width: res.width, height: res.height, mime: res.mime, bytes: res.bytes.length, path: outputPath }
}

/** Uint8Array → base64（分块避免大数组爆栈）。 */
export function bytesToBase64(bytes: Uint8Array): string {
  let binary = ''
  const chunk = 0x8000
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk))
  }
  return btoa(binary)
}

/** 人类可读字节数。 */
export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(2)} MB`
}
