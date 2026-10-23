import { native } from './native'
import { base64ToBytes } from '../tools/encode/codec/codec'
export function saveBase64File(base64: string, defaultName: string): Promise<boolean> {
  return native.saveBinary(base64ToBytes(base64.trim()), defaultName)
}
export function saveTextFile(text: string, defaultName: string): Promise<boolean> {
  return native.saveText(text, defaultName)
}
