/**
 * Generates the 1024x1024 source icon for `tauri icon`.
 *
 * Hand-rolled rather than exported from a design tool or produced with an image
 * library: this project ships no binary assets and no image dependency, and
 * this keeps the icon reproducible from source. Node's own zlib does the only
 * hard part.
 *
 *   node scripts/make-icon.mjs         # writes icon-source.png
 *   npx tauri icon icon-source.png     # fans out to src-tauri/icons/
 */

import { writeFileSync } from 'node:fs'
import { deflateSync } from 'node:zlib'

const SIZE = 1024
/** Samples per axis. 3 gives clean edges on the almond curve without fuss. */
const SS = 3

// --- geometry --------------------------------------------------------------

const clamp01 = (n) => (n < 0 ? 0 : n > 1 ? 1 : n)
const mix = (a, b, t) => a.map((channel, i) => channel + (b[i] - channel) * t)

/** Rounded-square tile test, in -1..1 space. */
function insideSquircle(x, y, half, radius) {
  const dx = Math.abs(x) - (half - radius)
  const dy = Math.abs(y) - (half - radius)
  if (dx <= 0 || dy <= 0) return Math.abs(x) <= half && Math.abs(y) <= half
  return Math.hypot(dx, dy) <= radius
}

/**
 * The almond. Height tapers as a power of horizontal distance, which produces
 * the pointed corners a real eye has — a plain ellipse reads as a lens.
 */
function insideAlmond(u, v) {
  if (Math.abs(u) >= 1) return false
  const halfHeight = 0.62 * Math.pow(1 - u * u, 0.72)
  return Math.abs(v) <= halfHeight
}

// --- palette (matches --iris tokens in src-vue/styles/theme.css) -----------

const BG_TOP = [18, 84, 90]
const BG_BOTTOM = [8, 42, 47]
const SCLERA = [234, 247, 246]
const IRIS_LIGHT = [127, 227, 222]
const IRIS_MID = [53, 179, 174]
const IRIS_DEEP = [31, 127, 124]
const PUPIL = [11, 20, 23]
const GLINT = [255, 255, 255]

/** Colour for one sample point, or null for transparent. */
function sample(x, y) {
  if (!insideSquircle(x, y, 0.94, 0.42)) return null

  // Background gradient, lit from the top.
  let rgb = mix(BG_TOP, BG_BOTTOM, clamp01((y + 0.94) / 1.88))

  // The eye occupies the middle band of the tile.
  const u = x / 0.66
  const v = y / 0.66

  if (insideAlmond(u, v)) {
    rgb = SCLERA

    const r = Math.hypot(u, v)

    if (r <= 0.42) {
      // Radial iris, brightest towards the top-left.
      const shade = clamp01(Math.hypot(u + 0.06, v + 0.08) / 0.42)
      rgb =
        shade < 0.55
          ? mix(IRIS_LIGHT, IRIS_MID, shade / 0.55)
          : mix(IRIS_MID, IRIS_DEEP, (shade - 0.55) / 0.45)
    }

    if (r <= 0.18) rgb = PUPIL

    // Specular highlight — the detail that makes this read as an eye and not
    // as a target, even at 32px.
    if (Math.hypot(u + 0.14, v + 0.16) <= 0.075) rgb = GLINT
  }

  return rgb
}

// --- rasterise -------------------------------------------------------------

// One filter byte per scanline, then RGBA.
const raw = Buffer.alloc(SIZE * (SIZE * 4 + 1))
const step = 2 / SIZE

for (let py = 0; py < SIZE; py++) {
  const rowStart = py * (SIZE * 4 + 1)
  raw[rowStart] = 0 // filter type: none

  for (let px = 0; px < SIZE; px++) {
    let r = 0
    let g = 0
    let b = 0
    let hits = 0

    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const x = -1 + (px + (sx + 0.5) / SS) * step
        const y = -1 + (py + (sy + 0.5) / SS) * step
        const found = sample(x, y)
        if (found) {
          r += found[0]
          g += found[1]
          b += found[2]
          hits++
        }
      }
    }

    const at = rowStart + 1 + px * 4
    if (hits === 0) {
      raw.writeUInt32BE(0, at)
    } else {
      raw[at] = Math.round(r / hits)
      raw[at + 1] = Math.round(g / hits)
      raw[at + 2] = Math.round(b / hits)
      // Coverage becomes alpha, which is what anti-aliases the tile edge.
      raw[at + 3] = Math.round((hits / (SS * SS)) * 255)
    }
  }
}

// --- PNG container ---------------------------------------------------------

const CRC_TABLE = Array.from({ length: 256 }, (_, n) => {
  let c = n
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
  return c >>> 0
})

function crc32(buf) {
  let c = 0xffffffff
  for (const byte of buf) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8)
  return (c ^ 0xffffffff) >>> 0
}

function chunk(type, data) {
  const length = Buffer.alloc(4)
  length.writeUInt32BE(data.length)
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data])
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(body))
  return Buffer.concat([length, body, crc])
}

const ihdr = Buffer.alloc(13)
ihdr.writeUInt32BE(SIZE, 0)
ihdr.writeUInt32BE(SIZE, 4)
ihdr[8] = 8 // bit depth
ihdr[9] = 6 // colour type: RGBA
ihdr[10] = 0 // compression: deflate
ihdr[11] = 0 // filter method: adaptive
ihdr[12] = 0 // no interlace

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
])

writeFileSync('icon-source.png', png)
console.log(
  `wrote icon-source.png (${SIZE}x${SIZE}, ${(png.length / 1024).toFixed(1)} KiB)`,
)
