//! Fontes de pixel do Windows: `.fnt` (2.0/3.0) e `.fon` (NE ou PE) — porte de
//! `docs/Pixel Art/app/engine/font_win.js`. O `.fnt` fala BYTES: o `dfCharSet` traduz para Unicode.

use std::collections::BTreeMap;

use crate::charsets::charset_to_unicode;
use crate::font::{Glyph, PixelFont};

fn u16_at(b: &[u8], o: usize) -> usize {
    usize::from(b[o]) | usize::from(b[o + 1]) << 8
}
fn u32_at(b: &[u8], o: usize) -> usize {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize
}

const HEADER_V2: usize = 118;
const HEADER_V3: usize = 148;
const DFF_COLOR: usize = 0x10 | 0x20 | 0x40 | 0x80;

fn weight_style(weight: usize, italic: bool) -> String {
    let w = if weight >= 700 {
        "Bold"
    } else if weight > 0 && weight <= 300 {
        "Light"
    } else {
        ""
    };
    [w, if italic { "Italic" } else { "" }]
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Um recurso `.fnt` que começa em `base` e cabe até `end`.
pub fn parse_fnt(b: &[u8], file_name: &str, base: usize, end: usize) -> Result<PixelFont, String> {
    let o = base;
    if end < o || end - o < HEADER_V2 {
        return Err("FNT: curto demais para o cabeçalho".into());
    }
    let version = u16_at(b, o);
    if version != 0x200 && version != 0x300 {
        return Err(format!(
            "FNT: versão 0x{version:04x} (só a 2.0 e a 3.0 existem em fontes de pixel)"
        ));
    }
    if u16_at(b, o + 66) & 1 != 0 {
        return Err("FNT: fonte VETORIAL — só as de pixel (raster) têm células para ler".into());
    }
    let v3 = version == 0x300;
    if v3 && end - o < HEADER_V3 {
        return Err("FNT 3.0: curto demais para o cabeçalho".into());
    }
    if v3 && u32_at(b, o + 118) & DFF_COLOR != 0 {
        return Err("FNT 3.0: fonte COLORIDA — só as de 1 bit são fontes de pixel".into());
    }
    let (ascent, ext_leading, italic, weight) = (
        u16_at(b, o + 74),
        u16_at(b, o + 78),
        b[o + 80] != 0,
        u16_at(b, o + 83),
    );
    let (charset, height) = (b[o + 85], u16_at(b, o + 88));
    let (first, last, default_rel) = (b[o + 95], b[o + 96], b[o + 97]);
    let face_at = u32_at(b, o + 105);
    if last < first {
        return Err(format!(
            "FNT: o último caractere ({last}) vem antes do primeiro ({first})"
        ));
    }
    if ascent > height {
        return Err(format!(
            "FNT: a subida ({ascent}) passa da altura ({height})"
        ));
    }
    let table = o + if v3 { HEADER_V3 } else { HEADER_V2 };
    let entry = if v3 { 6 } else { 4 };
    let n = usize::from(last - first) + 1;
    if table + n * entry > end {
        return Err("FNT: a tabela de caracteres passa do fim do recurso".into());
    }
    let mut glyphs = BTreeMap::new();
    let mut codes = Vec::new();
    for k in 0..n {
        let e = table + k * entry;
        let w = u16_at(b, e);
        let off = if v3 {
            u32_at(b, e + 2)
        } else {
            u16_at(b, e + 2)
        };
        let (row, at) = ((w + 7) >> 3, o + off);
        if at + row * height > end {
            return Err(format!(
                "FNT: o caractere {} aponta para fora do recurso",
                usize::from(first) + k
            ));
        }
        let mut bits = vec![0u8; w * height];
        for y in 0..height {
            for x in 0..w {
                bits[y * w + x] = (b[at + y * row + (x >> 3)] >> (7 - (x & 7))) & 1;
            }
        }
        let byte = first + k as u8;
        let cp = charset_to_unicode(charset, byte);
        glyphs.insert(
            cp,
            Glyph::new(w as u32, height as u32, 0, ascent as i32, w as i32, bits),
        );
        codes.push((byte, cp));
    }
    let mut family = String::new();
    if face_at != 0 && o + face_at < end {
        family = b[o + face_at..end]
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as char)
            .collect();
    }
    if family.is_empty() {
        let base = file_name.rsplit(['/', '\\']).next().unwrap_or("");
        family = base.rsplit_once('.').map_or(base, |(s, _)| s).to_owned();
    }
    let default_cp = Some(charset_to_unicode(charset, first.wrapping_add(default_rel)));
    let mut f = PixelFont::new(
        family,
        weight_style(weight, italic),
        if v3 { "fnt3" } else { "fnt" },
        ascent as i32,
        (height - ascent) as i32,
        ext_leading as i32,
        glyphs,
        default_cp,
        Vec::new(),
    );
    f.codes = codes;
    Ok(f)
}

fn ne_fonts(b: &[u8], ne: usize, file_name: &str) -> Result<Vec<PixelFont>, String> {
    let table = ne + u16_at(b, ne + 0x24);
    let shift = u16_at(b, table);
    let mut fonts = Vec::new();
    let mut p = table + 2;
    while p + 8 <= b.len() {
        let (ty, count) = (u16_at(b, p), u16_at(b, p + 2));
        if ty == 0 {
            break;
        }
        p += 8;
        for _ in 0..count {
            if ty == 0x8008 {
                let (off, len) = (u16_at(b, p) << shift, u16_at(b, p + 2) << shift);
                fonts.push(parse_fnt(b, file_name, off, b.len().min(off + len))?);
            }
            p += 12;
        }
    }
    Ok(fonts)
}

fn pe_fonts(b: &[u8], pe: usize, file_name: &str) -> Result<Vec<PixelFont>, String> {
    let coff = pe + 4;
    let (nsec, opt_size, opt) = (u16_at(b, coff + 2), u16_at(b, coff + 16), coff + 20);
    let dirs = opt + if u16_at(b, opt) == 0x20b { 112 } else { 96 };
    let rsrc = u32_at(b, dirs + 16);
    let sections: Vec<(usize, usize, usize)> = (0..nsec)
        .map(|s| {
            let h = opt + opt_size + s * 40;
            (
                u32_at(b, h + 12),
                u32_at(b, h + 8).max(u32_at(b, h + 16)),
                u32_at(b, h + 20),
            )
        })
        .collect();
    let to_offset = |rva: usize| -> Result<usize, String> {
        sections
            .iter()
            .find(|(va, size, _)| rva >= *va && rva < va + size)
            .map(|(va, _, raw)| raw + rva - va)
            .ok_or_else(|| "FON (PE): um recurso aponta para fora das seções".to_owned())
    };
    if rsrc == 0 {
        return Err("FON (PE): o executável não tem recursos".into());
    }
    let root = to_offset(rsrc)?;
    let entries = |dir: usize| -> Vec<(usize, usize)> {
        let n = u16_at(b, dir + 12) + u16_at(b, dir + 14);
        (0..n)
            .map(|e| (u32_at(b, dir + 16 + e * 8), u32_at(b, dir + 20 + e * 8)))
            .collect()
    };
    let sub = |data: usize| root + (data & 0x7fff_ffff);
    let mut fonts = Vec::new();
    for (id, data) in entries(root) {
        if id != 8 || data & 0x8000_0000 == 0 {
            continue;
        }
        for (_, fdata) in entries(sub(data)) {
            let mut node = fdata;
            while node & 0x8000_0000 != 0 {
                node = entries(sub(node)).first().map_or(0, |e| e.1);
            }
            let leaf = root + node;
            let (at, size) = (to_offset(u32_at(b, leaf))?, u32_at(b, leaf + 4));
            fonts.push(parse_fnt(b, file_name, at, b.len().min(at + size))?);
        }
    }
    Ok(fonts)
}

/// Um `.fon`: todas as fontes que ele carrega, na ordem dos recursos.
pub fn parse_fon(b: &[u8], file_name: &str) -> Result<Vec<PixelFont>, String> {
    if b.len() < 0x40 || b[0] != 0x4d || b[1] != 0x5a {
        return Err("FON: não é um executável (falta o \"MZ\")".into());
    }
    let at = u32_at(b, 0x3c);
    if at + 4 > b.len() {
        return Err("FON: o cabeçalho novo aponta para fora do arquivo".into());
    }
    let fonts = if b[at] == 0x4e && b[at + 1] == 0x45 {
        ne_fonts(b, at, file_name)?
    } else if b[at] == 0x50 && b[at + 1] == 0x45 && b[at + 2] == 0 && b[at + 3] == 0 {
        pe_fonts(b, at, file_name)?
    } else {
        return Err("FON: executável que não é nem NE nem PE".into());
    };
    if fonts.is_empty() {
        return Err("FON: o executável não traz nenhuma fonte".into());
    }
    Ok(fonts)
}
