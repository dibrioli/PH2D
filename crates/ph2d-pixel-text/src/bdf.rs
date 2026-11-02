//! Leitor de BDF 2.1 — porte de `docs/Pixel Art/app/engine/font_bdf.js` (conferido lá pela FreeType).
//!
//! Tolera: dígitos a mais/menos na linha de BITMAP; glifo sem DWIDTH (vale o DWIDTH da fonte, senão a
//! LARGURA DA CAIXA — a regra medida da FreeType 2.14); glifo sem BBX (vale a FONTBOUNDINGBOX).
//! Recusa com a linha: dígito não hexadecimal, caixa negativa, glifo sem ENDCHAR. `ENCODING -1` fica fora.

use std::collections::BTreeMap;

use crate::font::{Glyph, PixelFont};

fn hex_nibble(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|v| v as u8)
}

fn is_unicode_registry(reg: &str, enc: &str) -> bool {
    let r = reg.to_ascii_uppercase();
    r.is_empty() || r == "ISO10646" || (r == "ISO8859" && enc == "1") || r == "ASCII"
}

#[derive(Clone, Debug, PartialEq)]
enum Prop {
    Int(i64),
    Str(String),
}

impl Prop {
    fn as_str(&self) -> String {
        match self {
            Prop::Int(i) => i.to_string(),
            Prop::Str(s) => s.clone(),
        }
    }
}

fn split(line: &str) -> (&str, &str) {
    match line.find(char::is_whitespace) {
        None => (line, ""),
        Some(sp) => (&line[..sp], line[sp + 1..].trim()),
    }
}

struct P<'a> {
    lines: Vec<&'a str>,
    i: usize,
}

impl P<'_> {
    fn fail<T>(&self, msg: impl std::fmt::Display) -> Result<T, String> {
        Err(format!("BDF, linha {}: {msg}", self.i + 1))
    }
    fn ints(&self, rest: &str, n: usize) -> Result<Vec<i64>, String> {
        let v: Vec<Option<i64>> = rest.split_whitespace().map(|t| t.parse().ok()).collect();
        if v.len() < n || v[..n].iter().any(Option::is_none) {
            return self.fail(format!("esperado {n} inteiro(s), veio \"{rest}\""));
        }
        Ok(v.into_iter().map_while(|x| x).collect())
    }
}

fn split_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(p) = rest.find(['\r', '\n']) {
        out.push(&rest[..p]);
        let skip = if rest[p..].starts_with("\r\n") { 2 } else { 1 };
        rest = &rest[p + skip..];
    }
    out.push(rest);
    out
}

pub(crate) fn split_text_lines(text: &str) -> Vec<&str> {
    split_lines(text)
}

/// Lê um BDF; `Err` com a linha do problema.
pub fn parse_bdf(text: &str, file_name: &str) -> Result<PixelFont, String> {
    let mut p = P {
        lines: split_lines(text),
        i: 0,
    };
    while p.i < p.lines.len() && p.lines[p.i].trim().is_empty() {
        p.i += 1;
    }
    let first = p.lines.get(p.i).map_or("", |l| l.trim());
    if split(first).0 != "STARTFONT" {
        return p.fail("um BDF começa com STARTFONT");
    }
    let mut props: BTreeMap<String, Prop> = BTreeMap::new();
    let (mut font_name, mut fbb, mut font_dw, mut declared) =
        (String::new(), None::<Vec<i64>>, None::<Vec<i64>>, -1i64);
    let (mut unencoded, mut duplicated) = (0usize, 0usize);
    let mut glyphs: BTreeMap<u32, Glyph> = BTreeMap::new();

    p.i += 1;
    while p.i < p.lines.len() {
        let t = p.lines[p.i].trim();
        if t.is_empty() {
            p.i += 1;
            continue;
        }
        let (key, rest) = split(t);
        match key {
            "FONT" => font_name = rest.to_owned(),
            "FONTBOUNDINGBOX" => fbb = Some(p.ints(rest, 4)?),
            "DWIDTH" => font_dw = Some(p.ints(rest, 1)?),
            "CHARS" => declared = p.ints(rest, 1)?[0],
            "STARTPROPERTIES" => {
                p.i += 1;
                while p.i < p.lines.len() {
                    let pl = p.lines[p.i].trim();
                    if split(pl).0 == "ENDPROPERTIES" {
                        break;
                    }
                    if !pl.is_empty() {
                        let (name, value) = split(pl);
                        let v = if let Some(s) = value.strip_prefix('"') {
                            let s = if value.len() > 1 {
                                s.strip_suffix('"').unwrap_or(s)
                            } else {
                                s
                            };
                            Prop::Str(s.replace("\"\"", "\""))
                        } else if let Ok(i) = value.parse::<i64>() {
                            Prop::Int(i)
                        } else {
                            Prop::Str(value.to_owned())
                        };
                        props.insert(name.to_owned(), v);
                    }
                    p.i += 1;
                }
                if p.i >= p.lines.len() {
                    return p.fail("STARTPROPERTIES sem ENDPROPERTIES");
                }
            }
            "STARTCHAR" => read_char(
                &mut p,
                fbb.as_deref(),
                font_dw.as_deref(),
                &mut glyphs,
                &mut unencoded,
                &mut duplicated,
            )?,
            "ENDFONT" => break,
            _ => {}
        }
        p.i += 1;
    }

    let int_prop = |k: &str| match props.get(k) {
        Some(Prop::Int(i)) => Some(*i),
        _ => None,
    };
    let (ascent, descent) = match (int_prop("FONT_ASCENT"), int_prop("FONT_DESCENT")) {
        (Some(a), Some(d)) => (a, d),
        _ => match &fbb {
            Some(b) => (b[1] + b[3], -b[3]),
            None => return Err("BDF: sem FONT_ASCENT/FONT_DESCENT e sem FONTBOUNDINGBOX — não há onde pôr a linha de base".into()),
        },
    };
    if glyphs.is_empty() {
        return Err("BDF: nenhum glifo com código".into());
    }
    let mut notes = Vec::new();
    if unencoded > 0 {
        notes.push(format!(
            "{unencoded} glifo(s) sem código (ENCODING -1) ficaram de fora"
        ));
    }
    if duplicated > 0 {
        notes.push(format!(
            "{duplicated} código(s) repetido(s): valeu o último"
        ));
    }
    let total = (glyphs.len() + unencoded + duplicated) as i64;
    if declared >= 0 && declared != total {
        notes.push(format!(
            "o arquivo declara {declared} glifos e traz {total}"
        ));
    }
    let reg = props
        .get("CHARSET_REGISTRY")
        .map(Prop::as_str)
        .unwrap_or_default();
    let enc = props
        .get("CHARSET_ENCODING")
        .map(Prop::as_str)
        .unwrap_or_default();
    if !is_unicode_registry(&reg, &enc) {
        notes.push(format!(
            "codificação {reg}-{enc}: os códigos foram lidos como Unicode"
        ));
    }
    let xlfd = if font_name.starts_with('-') {
        font_name.split('-').nth(2).unwrap_or("").to_owned()
    } else {
        String::new()
    };
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(s, _)| s)
        .to_owned();
    let family = ["FAMILY_NAME", "FACE_NAME", "FONT_NAME"]
        .iter()
        .filter_map(|k| props.get(*k).map(Prop::as_str))
        .find(|s| !s.is_empty())
        .or_else(|| (!xlfd.is_empty()).then_some(xlfd))
        .or_else(|| (!font_name.is_empty()).then(|| font_name.clone()))
        .unwrap_or(stem);
    let default_cp = match int_prop("DEFAULT_CHAR") {
        Some(dc) if dc >= 0 && glyphs.contains_key(&(dc as u32)) => Some(dc as u32),
        _ => glyphs.contains_key(&0xfffd).then_some(0xfffd),
    };
    let style = props
        .get("WEIGHT_NAME")
        .map(Prop::as_str)
        .unwrap_or_default();
    Ok(PixelFont::new(
        family,
        style,
        "bdf",
        ascent as i32,
        descent as i32,
        0,
        glyphs,
        default_cp,
        notes,
    ))
}

fn read_char(
    p: &mut P<'_>,
    fbb: Option<&[i64]>,
    font_dw: Option<&[i64]>,
    glyphs: &mut BTreeMap<u32, Glyph>,
    unencoded: &mut usize,
    duplicated: &mut usize,
) -> Result<(), String> {
    let (mut enc, mut dw, mut bbx) = (None::<i64>, None::<i64>, None::<Vec<i64>>);
    p.i += 1;
    while p.i < p.lines.len() {
        let t = p.lines[p.i].trim();
        if t.is_empty() {
            p.i += 1;
            continue;
        }
        let (key, rest) = split(t);
        match key {
            "ENCODING" => enc = Some(p.ints(rest, 1)?[0]),
            "DWIDTH" => dw = Some(p.ints(rest, 1)?[0]),
            "BBX" => bbx = Some(p.ints(rest, 4)?),
            "BITMAP" | "ENDCHAR" => {
                let Some(b) = bbx.as_deref().or(fbb) else {
                    return p.fail("glifo sem BBX, numa fonte sem FONTBOUNDINGBOX");
                };
                let (w, h, xo, yo) = (b[0], b[1], b[2], b[3]);
                if w < 0 || h < 0 {
                    return p.fail(format!("caixa negativa: {w}×{h}"));
                }
                let (w, h) = (w as usize, h as usize);
                let mut bits = vec![0u8; w * h];
                if key == "BITMAP" {
                    let mut row = 0;
                    p.i += 1;
                    while p.i < p.lines.len() {
                        let r = p.lines[p.i].trim();
                        if r == "ENDCHAR" {
                            break;
                        }
                        let k0 = split(r).0;
                        if k0 == "STARTCHAR" || k0 == "ENDFONT" {
                            return p.fail("glifo sem ENDCHAR");
                        }
                        if row < h {
                            let rb = r.as_bytes();
                            for x in 0..w {
                                let at = x >> 2;
                                if at >= rb.len() {
                                    break;
                                }
                                let Some(v) = hex_nibble(rb[at]) else {
                                    let c = r[at..].chars().next().unwrap_or('?');
                                    return p.fail(format!("\"{c}\" não é um dígito hexadecimal"));
                                };
                                bits[row * w + x] = (v >> (3 - (x & 3))) & 1;
                            }
                        }
                        row += 1;
                        p.i += 1;
                    }
                    if p.i >= p.lines.len() {
                        return p.fail("glifo sem ENDCHAR");
                    }
                }
                let Some(cp) = enc.filter(|e| *e >= 0) else {
                    *unencoded += 1;
                    return Ok(());
                };
                let adv = dw.or(font_dw.map(|d| d[0])).unwrap_or(w as i64);
                if glyphs.contains_key(&(cp as u32)) {
                    *duplicated += 1;
                }
                glyphs.insert(
                    cp as u32,
                    Glyph::new(
                        w as u32,
                        h as u32,
                        xo as i32,
                        (yo + h as i64) as i32,
                        adv as i32,
                        bits,
                    ),
                );
                return Ok(());
            }
            "STARTCHAR" | "ENDFONT" => return p.fail("glifo sem ENDCHAR"),
            _ => {}
        }
        p.i += 1;
    }
    p.fail("o arquivo acabou dentro de um glifo")
}
