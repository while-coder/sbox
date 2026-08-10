/**
 * 从 paste 事件中取出文件。
 * Windows 资源管理器复制 HEIC 时 MIME 可能为空或 application/octet-stream，
 * 因此不能用 `image/*` 过滤，应依据 file kind / files 列表读取。
 */
export function filesFromClipboard(event: ClipboardEvent): File[] {
  const data = event.clipboardData
  if (!data) return []

  const candidates = [
    ...Array.from(data.files),
    ...Array.from(data.items)
      .filter(item => item.kind === 'file')
      .map(item => item.getAsFile())
      .filter((file): file is File => !!file),
  ]

  const seen = new Set<string>()
  return candidates.filter((file) => {
    const key = `${file.name}\0${file.size}\0${file.lastModified}\0${file.type}`
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

