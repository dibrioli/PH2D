//! Leitor do `.hex` do GNU Unifont/Unscii — porte de `docs/Pixel Art/app/engine/font_hex.js`.
//!
//! A altura sai do comprimento MAIS COMUM (16 dígitos → 8, senão 16); a largura de cada letra, dos dígitos
//! dela. Linha de base: 14/2 para 16 de altura (Unifont), 8/0 para 8 (Unscii).

use std::collections::BTreeMap;

use crate::bdf::split_text_lines;
use crate::font::{Glyph, PixelFont};

fn parse_line(t: &str) -> Option<(u32, &str)> {
    let (code, bits) = t.split_once(':')?;
    let hex = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit());
    if !(2..=8).contains(&code.len()) || !hex(code) || !hex(bits) {
        return None;
    }
    Some((u32::from_str_radix(code, 16).ok()?, bits))
}

pub fn parse_hex(text: &str, file_name: &str) -> Result<PixelFont, String> {
    let mut entries = Vec::new();
    let mut lengths: BTreeMap<usize, usize> = BTreeMap::new();
    for (n, raw) in split_text_lines(text).into_iter().enumerate() {
        let t = raw.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some((cp, digits)) = parse_line(t) else {
            let head: String = t.chars().take(40).collect();
            return Err(format!(
                "HEX, linha {}: esperado CÓDIGO:BITS, veio \"{head}\"",
                n + 1
            ));
        };
        entries.push((cp, digits, n + 1));
        *lengths.entry(digits.len()).or_default() += 1;
    }
    if entries.is_empty() {
        return Err("HEX: nenhuma letra no arquivo".into());
    }
    // A moda; empate fica com o comprimento menor (o BTreeMap itera em ordem crescente).
    let (mut mode, mut best) = (0usize, 0usize);
    for (&len, &count) in &lengths {
        if count > best {
            mode = len;
            best = count;
        }
    }
    let h: usize = if mode == 16 { 8 } else { 16 };
    let ascent: i32 = if h == 16 { 14 } else { 8 };
    let mut glyphs = BTreeMap::new();
    let mut duplicated = 0;
    for (cp, digits, line_no) in entries {
        let nbits = digits.len() * 4;
        if nbits % h != 0 {
            return Err(format!(
                "HEX, linha {line_no}: {} dígitos não fecham uma altura de {h}",
                digits.len()
            ));
        }
        let w = nbits / h;
        let db = digits.as_bytes();
        let bits: Vec<u8> = (0..nbits)
            .map(|k| {
                let v = (db[k >> 2] as char).to_digit(16).unwrap_or(0) as u8;
                (v >> (3 - (k & 3))) & 1
            })
            .collect();
        if glyphs.contains_key(&cp) {
            duplicated += 1;
        }
        glyphs.insert(
            cp,
            Glyph::new(w as u32, h as u32, 0, ascent, w as i32, bits),
        );
    }
    let notes = if duplicated > 0 {
        vec![format!(
            "{duplicated} código(s) repetido(s): valeu o último"
        )]
    } else {
        Vec::new()
    };
    let base = file_name.rsplit(['/', '\\']).next().unwrap_or("");
    let stem = base
        .strip_suffix(".hex")
        .or_else(|| base.strip_suffix(".HEX"))
        .unwrap_or(base);
    let family = if stem.is_empty() {
        "hex".to_owned()
    } else {
        stem.to_owned()
    };
    let default_cp = glyphs.contains_key(&0xfffd).then_some(0xfffd);
    Ok(PixelFont::new(
        family,
        String::new(),
        "hex",
        ascent,
        h as i32 - ascent,
        0,
        glyphs,
        default_cp,
        notes,
    ))
}
