//! De um ARQUIVO para as fontes que ele traz — porte de `docs/Pixel Art/app/engine/font_io.js`.
//! Quem decide o formato é o CONTEÚDO, não a extensão.

use crate::bdf::{parse_bdf, split_text_lines};
use crate::font::PixelFont;
use crate::hex::parse_hex;
use crate::win::{parse_fnt, parse_fon};

/// As extensões que o seletor de arquivo oferece.
pub const FONT_EXTENSIONS: [&str; 4] = [".bdf", ".hex", ".fnt", ".fon"];

/// UTF-8 quando o texto é UTF-8 válido; senão Latin-1 (os BDF antigos).
pub fn decode_text(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => latin1(bytes),
    }
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

fn first_line(bytes: &[u8]) -> String {
    let head = latin1(&bytes[..bytes.len().min(4096)]);
    split_text_lines(&head)
        .into_iter()
        .map(str::trim)
        .find(|t| !t.is_empty() && !t.starts_with('#'))
        .unwrap_or("")
        .to_owned()
}

fn is_hex_line(t: &str) -> bool {
    t.split_once(':').is_some_and(|(c, b)| {
        (2..=8).contains(&c.len())
            && c.bytes().all(|x| x.is_ascii_hexdigit())
            && !b.is_empty()
            && b.bytes().all(|x| x.is_ascii_hexdigit())
    })
}

fn first_word(t: &str) -> &str {
    t.split(char::is_whitespace).next().unwrap_or("")
}

/// As fontes do arquivo (um `.fon` traz várias); `Err` com o motivo.
pub fn read_font_file(file_name: &str, bytes: &[u8]) -> Result<Vec<PixelFont>, String> {
    if bytes.is_empty() {
        return Err("arquivo vazio".into());
    }
    if bytes.len() >= 2 && bytes[0] == 0x4d && bytes[1] == 0x5a {
        return parse_fon(bytes, file_name);
    }
    let line = first_line(bytes);
    if first_word(&line) == "STARTFONT" {
        return Ok(vec![parse_bdf(&decode_text(bytes), file_name)?]);
    }
    if is_hex_line(&line) {
        return Ok(vec![parse_hex(&latin1(bytes), file_name)?]);
    }
    let version = if bytes.len() >= 2 {
        usize::from(bytes[0]) | usize::from(bytes[1]) << 8
    } else {
        0
    };
    if version == 0x200 || version == 0x300 {
        return Ok(vec![parse_fnt(bytes, file_name, 0, bytes.len())?]);
    }
    if line.starts_with("info") && line[4..].starts_with(char::is_whitespace)
        || line.starts_with("<?xml")
    {
        return Err(
            "este .fnt é do BMFont (letras em imagem), não uma fonte de pixel do Windows".into(),
        );
    }
    Err(
        "formato não reconhecido — as fontes de pixel que entram são .bdf, .hex, .fnt e .fon"
            .into(),
    )
}
