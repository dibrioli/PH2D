//! Adobe binary palette formats: `.ase` (Swatch Exchange) and `.aco` (Color). Both are big-endian.
//!
//! `.ase`: `ASEF` + version(u16,u16) + block_count(u32), then blocks `[type u16][len u32][data]`. A
//! colour block (type `0x0001`) is `[name_len u16][name UTF-16BE incl NUL][model 4B][values f32…]
//! [colour_type u16]`. Group blocks (`0xC001`/`0xC002`) are skipped — we flatten to a single list.
//!
//! `.aco` v1: `version(u16=1) + count(u16)`, then per colour `space(u16) + 4×u16`. v2 (`version=2`)
//! repeats that plus a UTF-16BE name; Photoshop writes v1 THEN v2. We parse v1 (every colour is
//! there) and write v1+v2 (v2 adds names → the widest app support).

use super::{PaletteData, PaletteError};

/// Big-endian forward cursor; every read is bounds-checked (`None` past the end).
struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self { b, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.pos..self.pos.checked_add(n)?)?;
        self.pos += n;
        Some(s)
    }
    fn u16(&mut self) -> Option<u16> {
        let b = self.take(2)?;
        Some(u16::from_be_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Option<u32> {
        let b = self.take(4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn f32(&mut self) -> Option<f32> {
        let b = self.take(4)?;
        Some(f32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    /// `units` UTF-16BE code units → lossy String, dropping one trailing NUL (Adobe lengths count it).
    fn utf16(&mut self, units: usize) -> Option<String> {
        let b = self.take(units.checked_mul(2)?)?;
        let u: Vec<u16> = b
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        Some(String::from_utf16_lossy(u.strip_suffix(&[0]).unwrap_or(&u)))
    }
}

/// `0..1` linear-byte → u8 (round, clamp).
fn to8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Subtractive CMYK (`0..1`, `1` = full ink) → RGBA8, the simple non-profiled conversion.
fn cmyk_to_rgb(c: f32, m: f32, y: f32, k: f32) -> [u8; 4] {
    let ch = |x: f32| to8((1.0 - x) * (1.0 - k));
    [ch(c), ch(m), ch(y), 255]
}

/// Parse an `.ase` file, flattening any colour groups into one list.
///
/// # Errors
/// [`PaletteError`] when the `ASEF` signature is missing or the block table is truncated.
pub fn parse_ase(bytes: &[u8]) -> Result<PaletteData, PaletteError> {
    let mut r = Reader::new(bytes);
    if r.take(4) != Some(b"ASEF") {
        return Err(PaletteError("not an ASE file (bad signature)"));
    }
    let (_maj, _min) = (r.u16(), r.u16());
    let blocks = r.u32().ok_or(PaletteError("truncated ASE header"))?;
    let mut name = String::new();
    let mut colors = Vec::new();
    for _ in 0..blocks {
        let Some(kind) = r.u16() else { break };
        let len = r.u32().ok_or(PaletteError("truncated ASE block"))? as usize;
        let Some(body) = r.take(len) else { break };
        if kind != 0x0001 {
            continue; // group start / end — flattened away
        }
        let mut b = Reader::new(body);
        let name_units = b.u16().unwrap_or(0) as usize;
        let swatch_name = b.utf16(name_units).unwrap_or_default();
        let Some(model) = b.take(4) else { continue };
        let rgba = match model {
            b"RGB " => match (b.f32(), b.f32(), b.f32()) {
                (Some(r), Some(g), Some(bl)) => [to8(r), to8(g), to8(bl), 255],
                _ => continue,
            },
            b"Gray" => match b.f32() {
                Some(v) => [to8(v), to8(v), to8(v), 255],
                None => continue,
            },
            b"CMYK" => match (b.f32(), b.f32(), b.f32(), b.f32()) {
                (Some(c), Some(m), Some(y), Some(k)) => cmyk_to_rgb(c, m, y, k),
                _ => continue,
            },
            _ => continue, // LAB and any future model — skip rather than guess
        };
        // ASE has no palette-level name; the first swatch's name is the best display label we get.
        if name.is_empty() {
            name = swatch_name;
        }
        colors.push(rgba);
    }
    Ok(PaletteData { name, colors })
}

/// Serialize an `.ase`: one RGB colour block per swatch (alpha dropped — ASE has no alpha channel).
#[must_use]
pub fn write_ase(p: &PaletteData) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"ASEF");
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&(p.colors.len() as u32).to_be_bytes());
    for (i, c) in p.colors.iter().enumerate() {
        let nm: Vec<u16> = format!("Colour {}", i + 1)
            .encode_utf16()
            .chain([0])
            .collect();
        let mut block = Vec::new();
        block.extend_from_slice(&(nm.len() as u16).to_be_bytes());
        for u in &nm {
            block.extend_from_slice(&u.to_be_bytes());
        }
        block.extend_from_slice(b"RGB ");
        for ch in &c[..3] {
            block.extend_from_slice(&(f32::from(*ch) / 255.0).to_be_bytes());
        }
        block.extend_from_slice(&2u16.to_be_bytes()); // colour type: normal
        out.extend_from_slice(&0x0001u16.to_be_bytes());
        out.extend_from_slice(&(block.len() as u32).to_be_bytes());
        out.extend_from_slice(&block);
    }
    out
}

/// Parse the v1 section of an `.aco` (RGB / Grayscale; other spaces are skipped). The optional v2
/// section that follows only adds names, so we ignore it.
///
/// # Errors
/// [`PaletteError`] when the version isn't 1 or the count/colours are truncated, or none parse.
pub fn parse_aco(bytes: &[u8]) -> Result<PaletteData, PaletteError> {
    let mut r = Reader::new(bytes);
    if r.u16() != Some(1) {
        return Err(PaletteError("unsupported ACO version (expected v1 header)"));
    }
    let count = r.u16().ok_or(PaletteError("truncated ACO count"))?;
    let mut colors = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let space = r.u16().ok_or(PaletteError("truncated ACO colour"))?;
        let w = [
            r.u16().ok_or(PaletteError("truncated ACO colour"))?,
            r.u16().ok_or(PaletteError("truncated ACO colour"))?,
            r.u16().ok_or(PaletteError("truncated ACO colour"))?,
            r.u16().ok_or(PaletteError("truncated ACO colour"))?,
        ];
        if let Some(c) = aco_color(space, w) {
            colors.push(c);
        }
    }
    if colors.is_empty() {
        return Err(PaletteError("no RGB/Gray colours in ACO"));
    }
    Ok(PaletteData {
        name: String::new(),
        colors,
    })
}

/// One `.aco` colour record → RGBA8. RGB components are `0..=65535` (`v/257`); Grayscale is `0..=10000`.
fn aco_color(space: u16, w: [u16; 4]) -> Option<[u8; 4]> {
    let to8 = |v: u16| (u32::from(v) * 255 / 65535) as u8;
    match space {
        0 => Some([to8(w[0]), to8(w[1]), to8(w[2]), 255]), // RGB
        8 => {
            let g = (u32::from(w[0]) * 255 / 10000).min(255) as u8; // Grayscale 0..10000
            Some([g, g, g, 255])
        }
        _ => None, // HSB / CMYK / Lab — skip (kept correct over a guessed conversion)
    }
}

/// Serialize an `.aco` with BOTH a v1 and a v2 section (v2 names → widest reader support). RGB only.
#[must_use]
pub fn write_aco(p: &PaletteData) -> Vec<u8> {
    let n = p.colors.len() as u16;
    let mut out = Vec::new();
    let put_rgb = |out: &mut Vec<u8>, c: &[u8; 4]| {
        out.extend_from_slice(&0u16.to_be_bytes()); // RGB space
        for ch in &c[..3] {
            out.extend_from_slice(&(u16::from(*ch) * 257).to_be_bytes());
        }
        out.extend_from_slice(&0u16.to_be_bytes()); // 4th component unused for RGB
    };
    // v1
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    for c in &p.colors {
        put_rgb(&mut out, c);
    }
    // v2 (count repeated, each colour followed by a UTF-16BE name)
    out.extend_from_slice(&2u16.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    for (i, c) in p.colors.iter().enumerate() {
        put_rgb(&mut out, c);
        let nm: Vec<u16> = format!("Colour {}", i + 1)
            .encode_utf16()
            .chain([0])
            .collect();
        out.extend_from_slice(&(nm.len() as u32).to_be_bytes());
        for u in &nm {
            out.extend_from_slice(&u.to_be_bytes());
        }
    }
    out
}
