/**
 * 图片格式转换 —— 前端工具函数。
 * 解码/编码全部在 Rust 端（src-tauri/src/tools/image_convert.rs）完成，
 * 这里只负责文件读取、调用命令、以及格式元数据。
 */
import { invoke } from '@tauri-apps/api/core'

/** 可输出（可编码）的目标格式。 */
export interface OutputFormat {
  /** 目标 key，与 Rust encode 分派一致 */
  value: string
  label: string
  /** 保存时的扩展名 */
  ext: string
  /** 是否有损（决定是否显示质量滑块） */
  lossy: boolean
}

export const OUTPUT_FORMATS: OutputFormat[] = [
  { value: 'png', label: 'PNG', ext: 'png', lossy: false },
  { value: 'jpeg', label: 'JPEG', ext: 'jpg', lossy: true },
  { value: 'webp', label: 'WebP', ext: 'webp', lossy: true },
  { value: 'gif', label: 'GIF', ext: 'gif', lossy: false },
  { value: 'bmp', label: 'BMP', ext: 'bmp', lossy: false },
  { value: 'ico', label: 'ICO（图标，≤256px）', ext: 'ico', lossy: false },
  { value: 'tiff', label: 'TIFF', ext: 'tiff', lossy: false },
  { value: 'tga', label: 'TGA', ext: 'tga', lossy: false },
  { value: 'qoi', label: 'QOI', ext: 'qoi', lossy: false },
]

/** 输入侧支持的格式说明（SVG 矢量与常规位图；HEIC/AVIF 暂不支持）。 */
export const INPUT_HINT =
  'PNG · JPEG · WebP · GIF · BMP · ICO · TIFF · TGA · QOI · PNM · DDS · HDR · EXR · SVG'

export interface ConvertOptions {
  target: string
  quality?: number
  width?: number
  height?: number
  keepAspect?: boolean
}

export interface ConvertResult {
  base64: string
  width: number
  height: number
  mime: string
  bytes: number
}

export interface ConvertFileResult {
  width: number
  height: number
  mime: string
  bytes: number
  path: string
}

/** 读取文件为纯 base64（不含 data: 前缀）。 */
export function fileToBase64(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const s = reader.result as string
      resolve(s.split(',', 2)[1] ?? '')
    }
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

/** 从文件名取小写扩展名（无点）。 */
export function extOf(name: string): string {
  const i = name.lastIndexOf('.')
  return i >= 0 ? name.slice(i + 1).toLowerCase() : ''
}

/** 调用后端转换。 */
export function convert(
  inputBase64: string,
  inputExt: string,
  options: ConvertOptions,
): Promise<ConvertResult> {
  return invoke<ConvertResult>('image_convert', { inputBase64, inputExt, options })
}

/** 调用后端转换并直接写入文件。 */
export function convertToFile(
  inputBase64: string,
  inputExt: string,
  outputPath: string,
  options: ConvertOptions,
): Promise<ConvertFileResult> {
  return invoke<ConvertFileResult>('image_convert_to_file', { inputBase64, inputExt, outputPath, options })
}

/** 人类可读字节数。 */
export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(2)} MB`
}
