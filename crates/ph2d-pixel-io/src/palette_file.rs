//! **Ficheiros de paleta** (Load/Save Palette): `.gpl` (GIMP, com o `Channels: RGBA` do Aseprite),
//! `.pal` (JASC-PAL em texto e RIFF PAL da Microsoft), `.hex` (Lospec: uma cor por linha), `.act`
//! (Adobe Color Table) e o JSON da API do Lospec (só leitura).
//!
//! Leitura TOLERANTE (CRLF, tabs, comentários, nome de cor na cauda) e escrita canónica; uma
//! paleta sem cores é ERRO, nunca uma paleta vazia em silêncio.

use ph2d_pixel_core::palette::{MAX_COLORS, parse_hex};

/// Uma paleta lida de um ficheiro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteFile {
    /// O nome que o ficheiro declara; vazio quando não tem (o rótulo é da tela).
    pub name: String,
    pub colors: Vec<[u8; 4]>,
    /// O índice transparente que o ficheiro declara (`.act` com cauda).
    pub transparent: Option<u8>,
}

/// O formato de um ficheiro de paleta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteFormat {
    Gpl,
    JascPal,
    RiffPal,
    Hex,
    Act,
    LospecJson,
}

impl PaletteFormat {
    /// Os formatos que se ESCREVEM, com a extensão (o filtro do diálogo Save Palette).
    pub const WRITABLE: [(PaletteFormat, &'static str); 4] = [
        (PaletteFormat::Gpl, "gpl"),
        (PaletteFormat::JascPal, "pal"),
        (PaletteFormat::Hex, "hex"),
        (PaletteFormat::Act, "act"),
    ];

    /// As extensões que se LÊEM.
    pub const READABLE: [&'static str; 5] = ["gpl", "pal", "hex", "act", "json"];

    /// O formato de uma extensão (sem ponto, qualquer caixa).
    #[must_use]
    pub fn of_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "gpl" => Some(PaletteFormat::Gpl),
            "pal" => Some(PaletteFormat::JascPal),
            "hex" | "txt" => Some(PaletteFormat::Hex),
            "act" => Some(PaletteFormat::Act),
            "json" => Some(PaletteFormat::LospecJson),
            _ => None,
        }
    }
}

/// Porque um ficheiro de paleta foi recusado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteError {
    /// Nada no ficheiro.
    Empty,
    /// O formato pedido não reconhece o conteúdo (cabeçalho ausente, tamanho errado).
    NotThisFormat(PaletteFormat),
    /// O formato bate mas não há uma única cor legível.
    NoColors,
    /// Mais cores do que um índice alcança.
    TooManyColors(usize),
    /// JSON partido ou sem `colors`.
    Json,
    /// Uma cor que não se lê (`"zz"` num JSON).
    BadColor(String),
}

/// Lê pelo CONTEÚDO, com a extensão como desempate: RIFF e JASC pelo cabeçalho, `.act` pelo
/// tamanho, `{`/`[` = JSON, `GIMP Palette` = `.gpl`, o resto é lista de hex.
pub fn read(bytes: &[u8], extension: Option<&str>) -> Result<PaletteFile, PaletteError> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(PaletteError::Empty);
    }
    if bytes.starts_with(b"RIFF") {
        return read_as(bytes, PaletteFormat::RiffPal);
    }
    let ext = extension.and_then(PaletteFormat::of_extension);
    if ext == Some(PaletteFormat::Act) || (ext.is_none() && matches!(bytes.len(), 768 | 772)) {
        return read_as(bytes, PaletteFormat::Act);
    }
    let text = String::from_utf8_lossy(bytes);
    let t = text.trim_start();
    let fmt = if t.starts_with("JASC-PAL") {
        PaletteFormat::JascPal
    } else if t.len() >= 12 && t[..12].eq_ignore_ascii_case("GIMP Palette") {
        PaletteFormat::Gpl
    } else if t.starts_with('{') || t.starts_with('[') {
        PaletteFormat::LospecJson
    } else {
        PaletteFormat::Hex
    };
    read_as(bytes, fmt)
}

/// Lê no formato dado.
pub fn read_as(bytes: &[u8], format: PaletteFormat) -> Result<PaletteFile, PaletteError> {
    let out = match format {
        PaletteFormat::Act => read_act(bytes)?,
        PaletteFormat::RiffPal => read_riff(bytes)?,
        _ => {
            let text = String::from_utf8_lossy(bytes);
            match format {
                PaletteFormat::Gpl => read_gpl(&text)?,
                PaletteFormat::JascPal => read_jasc(&text)?,
                PaletteFormat::Hex => read_hex(&text)?,
                _ => read_lospec(&text)?,
            }
        }
    };
    if out.colors.is_empty() {
        return Err(PaletteError::NoColors);
    }
    if out.colors.len() > MAX_COLORS {
        return Err(PaletteError::TooManyColors(out.colors.len()));
    }
    Ok(out)
}

fn file(name: &str, colors: Vec<[u8; 4]>) -> PaletteFile {
    PaletteFile {
        name: name.to_owned(),
        colors,
        transparent: None,
    }
}

/// Os primeiros `n` inteiros de uma linha (o resto é o nome da cor); valores acima de 255 ficam
/// em 255 (o MVP fazia o mesmo com os ficheiros reais).
fn leading_ints(line: &str, n: usize) -> Option<Vec<u8>> {
    let v: Vec<u8> = line
        .split_whitespace()
        .take(n)
        .map_while(|w| w.parse::<u32>().ok().map(|x| x.min(255) as u8))
        .collect();
    (v.len() == n).then_some(v)
}

fn read_gpl(text: &str) -> Result<PaletteFile, PaletteError> {
    let mut lines = text.lines();
    let head = lines.next().unwrap_or("").trim();
    if !(head.len() >= 12 && head[..12].eq_ignore_ascii_case("GIMP Palette")) {
        return Err(PaletteError::NotThisFormat(PaletteFormat::Gpl));
    }
    let mut out = file("", Vec::new());
    let mut rgba = false;
    for line in lines {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = t.split_once(':') {
            match k.trim().to_ascii_lowercase().as_str() {
                "name" => out.name = v.trim().to_owned(),
                "channels" => rgba = v.trim().eq_ignore_ascii_case("RGBA"),
                _ => {}
            }
            continue;
        }
        if let Some(v) = leading_ints(t, if rgba { 4 } else { 3 }) {
            out.colors
                .push([v[0], v[1], v[2], v.get(3).copied().unwrap_or(255)]);
        }
    }
    Ok(out)
}

fn read_jasc(text: &str) -> Result<PaletteFile, PaletteError> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    if lines.next() != Some("JASC-PAL") {
        return Err(PaletteError::NotThisFormat(PaletteFormat::JascPal));
    }
    let _version = lines.next();
    let count: usize = lines
        .next()
        .and_then(|l| l.parse().ok())
        .ok_or(PaletteError::NotThisFormat(PaletteFormat::JascPal))?;
    let colors: Vec<[u8; 4]> = lines
        .filter_map(|l| leading_ints(l, 3))
        .take(count)
        .map(|v| [v[0], v[1], v[2], 255])
        .collect();
    Ok(file("", colors))
}

fn read_riff(b: &[u8]) -> Result<PaletteFile, PaletteError> {
    let bad = PaletteError::NotThisFormat(PaletteFormat::RiffPal);
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"PAL " {
        return Err(bad);
    }
    let mut at = 12;
    while at + 8 <= b.len() {
        let id = &b[at..at + 4];
        let size = u32::from_le_bytes([b[at + 4], b[at + 5], b[at + 6], b[at + 7]]) as usize;
        let body = b.get(at + 8..at + 8 + size).ok_or(bad.clone())?;
        if id == b"data" {
            let n = usize::from(u16::from_le_bytes([
                *body.get(2).ok_or(bad.clone())?,
                *body.get(3).ok_or(bad.clone())?,
            ]));
            let colors = body[4..]
                .as_chunks::<4>()
                .0
                .iter()
                .take(n)
                .map(|e| [e[0], e[1], e[2], 255])
                .collect();
            return Ok(file("", colors));
        }
        at += 8 + size + (size & 1);
    }
    Err(bad)
}

/// Uma cor por token (`#rrggbb`, `rrggbb`, `#rgb`; separadores de espaço, vírgula ou `;`). ⛔ Um
/// token de 3 dígitos SEM `#` não é cor: o MVP lia o `255` de um JASC como `#225555`.
fn read_hex(text: &str) -> Result<PaletteFile, PaletteError> {
    let colors = text
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter(|t| t.starts_with('#') || t.len() == 6)
        .filter_map(parse_hex)
        .collect();
    Ok(file("", colors))
}

fn read_act(b: &[u8]) -> Result<PaletteFile, PaletteError> {
    if b.len() != 768 && b.len() != 772 {
        return Err(PaletteError::NotThisFormat(PaletteFormat::Act));
    }
    let (mut n, mut transparent) = (256, None);
    if b.len() == 772 {
        n = usize::from(u16::from_be_bytes([b[768], b[769]])).clamp(1, 256);
        let t = u16::from_be_bytes([b[770], b[771]]);
        transparent = u8::try_from(t).ok().filter(|&t| usize::from(t) < n);
    }
    let colors = b[..768]
        .as_chunks::<3>()
        .0
        .iter()
        .take(n)
        .map(|c| [c[0], c[1], c[2], 255])
        .collect();
    Ok(PaletteFile {
        name: String::new(),
        colors,
        transparent,
    })
}

fn read_lospec(text: &str) -> Result<PaletteFile, PaletteError> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|_| PaletteError::Json)?;
    let list = match &v {
        serde_json::Value::Array(a) => a,
        _ => v["colors"].as_array().ok_or(PaletteError::Json)?,
    };
    let mut colors = Vec::with_capacity(list.len());
    for c in list {
        let rgb = match c {
            serde_json::Value::String(s) => parse_hex(s),
            serde_json::Value::Array(a) if a.len() >= 3 => {
                let ch = |i: usize| a[i].as_u64().map(|x| x.min(255) as u8);
                Some([
                    ch(0).unwrap_or(0),
                    ch(1).unwrap_or(0),
                    ch(2).unwrap_or(0),
                    255,
                ])
            }
            _ => None,
        };
        colors.push(rgb.ok_or_else(|| PaletteError::BadColor(c.to_string()))?);
    }
    let name = v["name"].as_str().unwrap_or_default();
    Ok(file(name, colors))
}

/// Escreve `colors` no formato dado. `transparent` só se grava onde o formato tem onde (`.act`).
#[must_use]
pub fn write(
    format: PaletteFormat,
    name: &str,
    colors: &[[u8; 4]],
    transparent: Option<u8>,
) -> Vec<u8> {
    match format {
        PaletteFormat::Act => {
            let mut out = vec![0u8; 772];
            for (i, c) in colors.iter().take(256).enumerate() {
                out[i * 3..i * 3 + 3].copy_from_slice(&c[..3]);
            }
            out[768..770].copy_from_slice(&(colors.len().min(256) as u16).to_be_bytes());
            out[770..772].copy_from_slice(&transparent.map_or(0xffff, u16::from).to_be_bytes());
            out
        }
        PaletteFormat::JascPal => {
            let mut s = format!("JASC-PAL\r\n0100\r\n{}\r\n", colors.len());
            for c in colors {
                s.push_str(&format!("{} {} {}\r\n", c[0], c[1], c[2]));
            }
            s.into_bytes()
        }
        PaletteFormat::Hex => colors
            .iter()
            .map(|c| format!("{:02x}{:02x}{:02x}\n", c[0], c[1], c[2]))
            .collect::<String>()
            .into_bytes(),
        PaletteFormat::RiffPal => {
            let n = colors.len().min(256);
            let data = 4 + n * 4;
            let mut out = Vec::with_capacity(20 + data);
            out.extend_from_slice(b"RIFF");
            out.extend_from_slice(&((12 + data) as u32).to_le_bytes());
            out.extend_from_slice(b"PAL data");
            out.extend_from_slice(&(data as u32).to_le_bytes());
            out.extend_from_slice(&0x0300u16.to_le_bytes());
            out.extend_from_slice(&(n as u16).to_le_bytes());
            for c in &colors[..n] {
                out.extend_from_slice(&[c[0], c[1], c[2], 0]);
            }
            out
        }
        PaletteFormat::Gpl | PaletteFormat::LospecJson => {
            let rgba = colors.iter().any(|c| c[3] != 255);
            let mut s = format!("GIMP Palette\nName: {name}\n");
            if rgba {
                s.push_str("Channels: RGBA\n");
            }
            s.push_str("Columns: 8\n#\n");
            for c in colors {
                s.push_str(&format!("{:3} {:3} {:3}", c[0], c[1], c[2]));
                if rgba {
                    s.push_str(&format!(" {:3}", c[3]));
                }
                s.push_str(&format!("\t{:02x}{:02x}{:02x}\n", c[0], c[1], c[2]));
            }
            s.into_bytes()
        }
    }
}
