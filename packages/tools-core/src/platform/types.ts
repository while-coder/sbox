/**
 * 平台适配层接口 —— 工具组件唯一与宿主（Web / Tauri）耦合的部分：文件落盘。
 * Web 宿主用浏览器下载实现；Tauri 宿主用「保存对话框 + Rust 落盘」实现。
 * 工具内部通过 getPlatform() 取用，不直接依赖任何宿主 API。
 */

/** 批量保存的单个条目。 */
export interface SaveItem {
  bytes: Uint8Array
  /** 文件名（含扩展名） */
  name: string
  mime: string
}

export interface Platform {
  /**
   * 保存单个二进制文件。
   * @param bytes       原始字节
   * @param defaultName 建议文件名（含扩展名）
   * @param mime        MIME 类型（Web 端用于 Blob）
   * @returns false 表示用户取消（Web 端恒为 true）
   */
  saveBinary(bytes: Uint8Array, defaultName: string, mime: string): Promise<boolean>

  /**
   * 保存文本内容（UTF-8）。
   * @returns false 表示用户取消
   */
  saveText(text: string, defaultName: string): Promise<boolean>

  /**
   * 批量保存。Web 端逐个触发下载；桌面端弹一次目录选择后全部落盘。
   * @returns 实际保存成功的数量（0 表示用户取消）
   */
  saveBatch(items: SaveItem[]): Promise<number>
}
