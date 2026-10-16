//! JSON-RPC base protocol framing: `Content-Length: N\r\n\r\n<N bytes of JSON>`.

use std::io::{self, BufRead, Write};

use serde_json::Value;

/// Writes one framed message and flushes.
pub fn write_message(w: &mut impl Write, body: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(body).map_err(io::Error::other)?;
    write!(w, "Content-Length: {}\r\n\r\n", bytes.len())?;
    w.write_all(&bytes)?;
    w.flush()
}

/// Reads one framed message. `Ok(None)` = clean end of stream before a header; anything else that
/// is not a well-formed message is an error (`InvalidData` / `UnexpectedEof`).
pub fn read_message(r: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length: Option<usize> = None;
    let mut line = String::new();
    let mut first = true;
    loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            if first {
                return Ok(None);
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "eof inside headers",
            ));
        }
        first = false;
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        let (name, value) = header
            .split_once(':')
            .ok_or_else(|| invalid(format!("malformed header {header:?}")))?;
        if name.trim().eq_ignore_ascii_case("content-length") {
            let n = value
                .trim()
                .parse()
                .map_err(|_| invalid(format!("bad Content-Length {value:?}")))?;
            length = Some(n);
        }
    }
    let n = length.ok_or_else(|| invalid("missing Content-Length".into()))?;
    let mut body = vec![0u8; n];
    r.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| invalid(e.to_string()))
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}
