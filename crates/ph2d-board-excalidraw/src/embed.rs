//! A cena EMBEBIDA (X3): o «embed scene» do Excalidraw guarda o ficheiro dentro do `.png` (um `tEXt`
//! de chave [`crate::MIME`]) e do `.svg` (base64 entre `payload-start` e `payload-end`), num invólucro
//! `{"encoding":"bstring","compressed":true,"encoded":<zlib em bytes latin-1>}` (oráculo, §1.3).

use std::io::Read as _;

use base64::Engine as _;
use serde_json::Value;

/// O invólucro → o texto JSON da cena.
fn unwrap(wrapper: &str) -> Result<String, String> {
    let w: Value = serde_json::from_str(wrapper).map_err(|e| e.to_string())?;
    let encoded = w["encoded"].as_str().ok_or("no scene inside")?;
    let bytes: Vec<u8> = encoded.chars().map(|c| c as u32 as u8).collect();
    if !w["compressed"].as_bool().unwrap_or(false) {
        return String::from_utf8(bytes).map_err(|e| e.to_string());
    }
    let mut out = String::new();
    flate2::read::ZlibDecoder::new(&bytes[..])
        .read_to_string(&mut out)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// ⭐ Os bytes de um ficheiro (`.excalidraw`/`.json`, ou um `.png`/`.svg` com a cena embebida) → o
/// texto JSON da cena.
///
/// # Errors
/// Uma imagem sem cena embebida, ou bytes que não são nenhum dos três.
pub fn scene_from_bytes(bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(b"\x89PNG") {
        // O `tEXt` dele vem DEPOIS dos dados da imagem: lê-se a imagem e fecha-se o leitor antes de
        // perguntar pelos textos (o `read_info` só lê o cabeçalho).
        let dec = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = dec.read_info().map_err(|e| e.to_string())?;
        let mut buf = vec![0; reader.output_buffer_size().ok_or("png too large")?];
        reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
        reader.finish().map_err(|e| e.to_string())?;
        let info = reader.info();
        let text = info
            .uncompressed_latin1_text
            .iter()
            .find(|t| t.keyword == crate::MIME)
            .map(|t| t.text.clone())
            .ok_or("this image has no Excalidraw scene")?;
        return unwrap(&text);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "not an Excalidraw file")?;
    if text.trim_start().starts_with('{') {
        return Ok(text.to_owned());
    }
    let start = text.find("payload-start -->").ok_or("this image has no Excalidraw scene")?;
    let rest = &text[start + "payload-start -->".len()..];
    let end = rest.find("<!--").ok_or("this image has no Excalidraw scene")?;
    let b64: String = rest[..end].chars().filter(|c| !c.is_whitespace()).collect();
    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| e.to_string())?;
    // O base64 dele é de uma string BINÁRIA (latin-1, o `btoa`), não de UTF-8.
    unwrap(&raw.iter().map(|&b| char::from(b)).collect::<String>())
}
