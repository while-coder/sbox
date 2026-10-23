import { native } from '../../../platform/native'
import { EMPTY_ANNOTATION } from '../../../platform/screenshot-types'
import type { Annotation, Capture, CapturePixels, SelectionRect } from '../../../platform/screenshot-types'
export * from '../../../platform/screenshot-types'
export const CAPTURE_READY = 'sbox://capture-ready'

export function getLatestCapture(): Promise<Capture | null> { return native.screenshot.getLatestCapture() }
export function getLatestCapturePixels(): Promise<CapturePixels> { return native.screenshot.getLatestCapturePixels() }
export function cropSelectionPixels(selection: SelectionRect, annotation: Annotation = EMPTY_ANNOTATION): Promise<CapturePixels> {
  return native.screenshot.cropSelectionPixels(selection, annotation)
}
export function saveSelection(path: string, selection: SelectionRect, annotation: Annotation = EMPTY_ANNOTATION): Promise<void> {
  return native.screenshot.saveSelection(path, selection, annotation)
}
export function clearCapture(): Promise<void> { return native.screenshot.clearCapture() }
export function ensureOverlay(): Promise<void> { return native.screenshot.ensureOverlay() }
export function startScreenshot(): Promise<void> { return native.screenshot.startScreenshot() }
export function finishScreenshot(restoreMain: boolean): Promise<void> { return native.screenshot.finishScreenshot(restoreMain) }
