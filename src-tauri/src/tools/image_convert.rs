//! 图片格式转换：解码（image / resvg）→ 可选缩放 → 编码目标格式。
//! SVG 为输入专用（栅格化）；输出走 image / webp 可编码的标准格式。
//! HEIC / AVIF 暂不支持（需 libheif / dav1d 原生库）。
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    /// 目标格式：png|jpeg|webp|gif|bmp|ico|tiff|tga|qoi
    pub target: String,
    /// 有损格式（jpeg/webp）质量 1-100
    pub quality: Option<u8>,
    /// 缩放目标宽（像素），留空表示不限制
    pub width: Option<u32>,
    /// 缩放目标高（像素），留空表示不限制
    pub height: Option<u32>,
    /// 是否保持纵横比（默认 true）
    pub keep_aspect: Option<bool>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResult {
    /// 编码后内容的 base64
    pub base64: String,
    pub width: u32,
    pub height: u32,
    pub mime: String,
    /// 编码后字节数
    pub bytes: usize,
}

/// 把 SVG 字节栅格化为 RGBA 图。width/height 给定时按其渲染，否则用 SVG 固有尺寸。
fn decode_svg(bytes: &[u8], width: Option<u32>, height: Option<u32>) -> Result<DynamicImage, String> {
    use resvg::tiny_skia;
    use resvg::usvg;

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(bytes, &opt).map_err(|e| format!("SVG 解析失败: {e}"))?;
    let size = tree.size();
    let (sw, sh) = (size.width(), size.height());
    if sw <= 0.0 || sh <= 0.0 {
        return Err("SVG 尺寸无效".into());
    }

    // 依据期望宽/高推导渲染缩放（保持比例），未指定则原始尺寸。
    let scale = match (width, height) {
        (Some(w), _) => w as f32 / sw,
        (None, Some(h)) => h as f32 / sh,
        (None, None) => 1.0,
    };
    let pw = (sw * scale).round().max(1.0) as u32;
    let ph = (sh * scale).round().max(1.0) as u32;

    let mut pixmap = tiny_skia::Pixmap::new(pw, ph).ok_or("无法分配 SVG 画布")?;
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let img = RgbaImage::from_raw(pw, ph, pixmap.take()).ok_or("SVG 像素转换失败")?;
    Ok(DynamicImage::ImageRgba8(img))
}

/// 判断是否 SVG（按扩展名或内容嗅探）。
fn is_svg(ext: &str, bytes: &[u8]) -> bool {
    if ext == "svg" || ext == "svgz" {
        return true;
    }
    let head = &bytes[..bytes.len().min(512)];
    let text = String::from_utf8_lossy(head);
    text.contains("<svg")
}

/// 解码任意输入为 DynamicImage。HEIC/AVIF 暂不支持（需原生库）。
fn decode(bytes: &[u8], ext: &str, width: Option<u32>, height: Option<u32>) -> Result<DynamicImage, String> {
    if is_svg(ext, bytes) {
        decode_svg(bytes, width, height)
    } else {
        image::load_from_memory(bytes).map_err(|e| format!("图片解码失败: {e}"))
    }
}

/// 按宽/高与 keep_aspect 缩放。两者都为空则不缩放。SVG 已在解码阶段按尺寸渲染，这里再兜底一次。
fn maybe_resize(img: DynamicImage, opts: &ConvertOptions) -> DynamicImage {
    let (ow, oh) = (img.width(), img.height());
    let keep = opts.keep_aspect.unwrap_or(true);
    match (opts.width, opts.height) {
        (None, None) => img,
        _ if opts.width == Some(ow) && opts.height == Some(oh) => img,
        (w, h) => {
            let filter = image::imageops::FilterType::Lanczos3;
            if keep {
                // resize 在 keep-aspect 下需要一个边界框；缺省的边用大值占位。
                let bw = w.unwrap_or(u32::MAX);
                let bh = h.unwrap_or(u32::MAX);
                img.resize(bw.max(1), bh.max(1), filter)
            } else {
                let tw = w.unwrap_or(ow).max(1);
                let th = h.unwrap_or(oh).max(1);
                img.resize_exact(tw, th, filter)
            }
        }
    }
}

/// 编码到目标格式，返回 (bytes, mime)。
fn encode(img: &DynamicImage, target: &str, quality: Option<u8>) -> Result<(Vec<u8>, String), String> {
    let q = quality.unwrap_or(85).clamp(1, 100);
    match target {
        "jpeg" | "jpg" => {
            let mut buf = Vec::new();
            // JPEG 不支持透明，铺白底。
            let rgb = DynamicImage::ImageRgb8(img.to_rgb8());
            JpegEncoder::new_with_quality(&mut buf, q)
                .encode_image(&rgb)
                .map_err(|e| format!("JPEG 编码失败: {e}"))?;
            Ok((buf, "image/jpeg".into()))
        }
        "webp" => {
            // image crate 仅支持无损 webp；用 webp crate 支持有损质量。
            let rgba = img.to_rgba8();
            let encoder = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
            let mem = encoder.encode(q as f32);
            Ok((mem.to_vec(), "image/webp".into()))
        }
        other => {
            let fmt = match other {
                "png" => ImageFormat::Png,
                "gif" => ImageFormat::Gif,
                "bmp" => ImageFormat::Bmp,
                "ico" => ImageFormat::Ico,
                "tiff" | "tif" => ImageFormat::Tiff,
                "tga" => ImageFormat::Tga,
                "qoi" => ImageFormat::Qoi,
                _ => return Err(format!("不支持的目标格式: {other}")),
            };
            let mut buf = Vec::new();
            // ICO 上限 256×256，超出时先缩放。
            let src;
            let img_ref: &DynamicImage = if fmt == ImageFormat::Ico && (img.width() > 256 || img.height() > 256) {
                src = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
                &src
            } else {
                img
            };
            img_ref
                .write_to(&mut Cursor::new(&mut buf), fmt)
                .map_err(|e| format!("编码失败: {e}"))?;
            let mime = match fmt {
                ImageFormat::Png => "image/png",
                ImageFormat::Gif => "image/gif",
                ImageFormat::Bmp => "image/bmp",
                ImageFormat::Ico => "image/x-icon",
                ImageFormat::Tiff => "image/tiff",
                ImageFormat::Tga => "image/x-tga",
                ImageFormat::Qoi => "image/qoi",
                _ => "application/octet-stream",
            };
            Ok((buf, mime.into()))
        }
    }
}

/// 转换单张图片：base64 入，base64 出。
#[tauri::command]
pub fn image_convert(
    input_base64: String,
    input_ext: Option<String>,
    options: ConvertOptions,
) -> Result<ConvertResult, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(input_base64.trim())
        .map_err(|e| format!("base64 解码失败: {e}"))?;
    let ext = input_ext.unwrap_or_default().to_ascii_lowercase();

    let img = decode(&bytes, &ext, options.width, options.height)?;
    let img = maybe_resize(img, &options);

    let (out, mime) = encode(&img, &options.target.to_ascii_lowercase(), options.quality)?;
    Ok(ConvertResult {
        width: img.width(),
        height: img.height(),
        bytes: out.len(),
        base64: base64::engine::general_purpose::STANDARD.encode(&out),
        mime,
    })
}
