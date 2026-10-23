export interface Capture {
  width: number
  height: number
  previewWidth: number
  previewHeight: number
  x: number
  y: number
  scale: number
}

export type CapturePixels = ArrayBuffer | Uint8Array | number[]

export interface SelectionRect {
  x: number
  y: number
  width: number
  height: number
}

export interface RectMark {
  x: number
  y: number
  width: number
  height: number
  lineWidth: number
  color: number[]
}

/** 自由涂抹马赛克笔迹，点列在选区裁剪坐标系内（原图像素）。 */
export interface MosaicMark {
  points: Array<[number, number]>
  lineWidth: number
  /** 色块边长（原图像素） */
  block: number
}

/** 自由画笔笔迹，点列在选区裁剪坐标系内（原图像素）。 */
export interface BrushMark {
  points: Array<[number, number]>
  lineWidth: number
  color: number[]
}

/** 覆盖层提交的全部标注，Rust 端按马赛克 → 画笔 → 描边矩形的顺序合成。 */
export interface Annotation {
  rectMarks: RectMark[]
  mosaics: MosaicMark[]
  brushStrokes: BrushMark[]
}

export const EMPTY_ANNOTATION: Annotation = {
  rectMarks: [],
  mosaics: [],
  brushStrokes: [],
}

