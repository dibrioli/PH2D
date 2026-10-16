//! The JSON-RPC framing: round trip, back-to-back messages, case-insensitive headers, clean EOF vs
//! truncation.

use std::io::{BufReader, Cursor, ErrorKind};

use ph2d_code_lsp::framing::{read_message, write_message};
use serde_json::json;

#[test]
fn framing_round_trips_back_to_back_messages() {
    let a = json!({ "jsonrpc": "2.0", "id": 1, "method": "x", "params": { "s": "ação 😀" } });
    let b = json!({ "jsonrpc": "2.0", "method": "y" });
    let mut buf = Vec::new();
    write_message(&mut buf, &a).unwrap();
    write_message(&mut buf, &b).unwrap();
    let mut r = BufReader::new(Cursor::new(buf));
    assert_eq!(read_message(&mut r).unwrap(), Some(a));
    assert_eq!(read_message(&mut r).unwrap(), Some(b));
    assert_eq!(read_message(&mut r).unwrap(), None);
}

#[test]
fn framing_length_counts_bytes_and_headers_are_case_insensitive() {
    let body = r#"{"m":"é"}"#;
    let raw = format!(
        "content-length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{body}",
        body.len()
    );
    let mut r = BufReader::new(Cursor::new(raw.into_bytes()));
    assert_eq!(read_message(&mut r).unwrap(), Some(json!({ "m": "é" })));
}

#[test]
fn framing_truncation_is_an_error_not_a_clean_end() {
    let mut r = BufReader::new(Cursor::new(b"Content-Length: 50\r\n\r\n{\"a\":".to_vec()));
    assert_eq!(
        read_message(&mut r).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
    let mut r = BufReader::new(Cursor::new(b"Content-Length: 5\r\n".to_vec()));
    assert_eq!(
        read_message(&mut r).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
    let mut r = BufReader::new(Cursor::new(b"X: 1\r\n\r\n{}".to_vec()));
    assert_eq!(
        read_message(&mut r).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
}
