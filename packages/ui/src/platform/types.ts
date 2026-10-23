/**
 * 桌面平台接口：工具内部通过 getPlatform() 使用原生文件拖放和保存。
 */

export interface Platform {
  /**
   * 监听原生文件拖放。
   * Tauri Windows 默认会拦截 HTML5 drop，必须通过此入口读取系统拖入的路径。
   */
  listenFileDrops?(
    onFiles: (files: File[]) => void,
    onError: (message: string) => void,
  ): Promise<() => void>

  /**
   * 保存单个二进制文件。
   * @param bytes       原始字节
   * @param defaultName 建议文件名（含扩展名）
   * @returns false 表示用户取消
   */
  saveBinary(bytes: Uint8Array, defaultName: string): Promise<boolean>

  /**
   * 保存文本内容（UTF-8）。
   * @returns false 表示用户取消
   */
  saveText(text: string, defaultName: string): Promise<boolean>

}
