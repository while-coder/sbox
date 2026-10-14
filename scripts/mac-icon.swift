#!/usr/bin/env swift
// Regenerate the macOS icon without changing the shared source or Windows/Linux icons.
import Foundation
import CoreGraphics
import ImageIO

let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
let icons = root.appendingPathComponent("packages/app/src-tauri/icons")

guard let source = CGImageSourceCreateWithURL(icons.appendingPathComponent("icon.png") as CFURL, nil),
      let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
    fatalError("Cannot read icons/icon.png")
}

// Keep the rounded-square artwork inside an 856 px square on a 1024 px canvas.
// The transparent inset prevents it from looking oversized in the Dock.
let representations = [
    (16, 1, "icp4"), (16, 2, "ic11"),
    (32, 1, "icp5"), (32, 2, "ic12"),
    (128, 1, "ic07"), (128, 2, "ic13"),
    (256, 1, "ic08"), (256, 2, "ic14"),
    (512, 1, "ic09"), (512, 2, "ic10"),
]

func chunkHeader(_ type: String, _ length: Int) -> Data {
    var size = UInt32(length).bigEndian
    var header = Data(type.utf8)
    withUnsafeBytes(of: &size) { header.append(contentsOf: $0) }
    return header
}

var chunks = Data()
var tableOfContents = Data()
for (points, scale, type) in representations {
    let pixels = points * scale
    guard let context = CGContext(
        data: nil, width: pixels, height: pixels, bitsPerComponent: 8, bytesPerRow: pixels * 4,
        space: CGColorSpace(name: CGColorSpace.sRGB)!,
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
    ) else { fatalError("Cannot create \(pixels) px canvas") }
    let canvas = CGFloat(pixels)
    let artwork = (canvas * 856 / 1024).rounded()
    let inset = (canvas - artwork) / 2
    context.clear(CGRect(x: 0, y: 0, width: canvas, height: canvas))
    context.interpolationQuality = .high
    context.draw(image, in: CGRect(x: inset, y: inset, width: artwork, height: artwork))

    let png = NSMutableData()
    guard let result = context.makeImage(),
          let destination = CGImageDestinationCreateWithData(png, "public.png" as CFString, 1, nil) else {
        fatalError("Cannot encode \(pixels) px icon")
    }
    CGImageDestinationAddImage(destination, result, nil)
    guard CGImageDestinationFinalize(destination) else { fatalError("Cannot write \(pixels) px icon") }
    let header = chunkHeader(type, png.length + 8)
    tableOfContents.append(header)
    chunks.append(header)
    chunks.append(png as Data)
}

// ICNS stores PNG representations in typed chunks with big-endian lengths.
var output = chunkHeader("icns", 8 + 8 + tableOfContents.count + chunks.count)
output.append(chunkHeader("TOC ", tableOfContents.count + 8))
output.append(tableOfContents)
output.append(chunks)
try output.write(to: icons.appendingPathComponent("icon.icns"), options: .atomic)
print("Generated icons/icon.icns (856/1024 artwork, transparent padding)")
