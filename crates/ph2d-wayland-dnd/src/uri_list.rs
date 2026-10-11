//! A `text/uri-list` (RFC 2483) de um arrasto: só os `file://` viram caminho.
//!
//! As quebras são `\r\n` pelo RFC, mas o Qt/Dolphin manda `\n` (Blender rB727cc426bc72); `#`
//! abre um comentário; o host do `file://` larga-se.

use std::path::PathBuf;

use percent_encoding::percent_decode;

/// O que uma lista largou: os ficheiros locais e quantas entradas NÃO eram ficheiro (o
/// `https://` de um navegador) — a app diz que as recusou, nunca as ignora calada.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UriList {
    pub paths: Vec<PathBuf>,
    pub not_files: usize,
}

pub fn parse(bytes: &[u8]) -> UriList {
    let mut out = UriList::default();
    for line in bytes.split(|&b| b == b'\n') {
        let line = trim_end(line);
        if line.is_empty() || line[0] == b'#' {
            continue;
        }
        match file_path(line) {
            Some(p) => out.paths.push(p),
            None => out.not_files += 1,
        }
    }
    out
}

/// O `\r` do CRLF e o NUL com que algumas fontes acabam o buffer.
fn trim_end(mut line: &[u8]) -> &[u8] {
    while let [rest @ .., b'\r' | b'\0'] = line {
        line = rest;
    }
    line
}

fn file_path(uri: &[u8]) -> Option<PathBuf> {
    let scheme = uri.get(..5)?;
    if !scheme.eq_ignore_ascii_case(b"file:") {
        return None;
    }
    let rest = &uri[5..];
    let path = match rest.strip_prefix(b"//") {
        Some(authority_and_path) => {
            let slash = authority_and_path.iter().position(|&b| b == b'/')?;
            &authority_and_path[slash..]
        }
        None => rest,
    };
    if path.first() != Some(&b'/') {
        return None;
    }
    Some(to_path(percent_decode(path).collect()))
}

#[cfg(unix)]
fn to_path(bytes: Vec<u8>) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    PathBuf::from(std::ffi::OsString::from_vec(bytes))
}

#[cfg(not(unix))]
fn to_path(bytes: Vec<u8>) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
#[path = "uri_list_tests.rs"]
mod tests;
