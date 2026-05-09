#![forbid(unsafe_code)]
//! `ph2d-mcp-server` — JSON-RPC 2.0 over stdin / stdout.
//!
//! Reads one JSON object per line from stdin, dispatches against an
//! in-memory [`ph2d_mcp::MemoryHost`], writes the response as one
//! JSON object per line to stdout. End on EOF.
//!
//! Why one-per-line: real MCP transports use Content-Length headers
//! (LSP-style) but for M9 the line-delimited form is good enough for
//! local testing + future test fixtures. Switching to LSP framing is
//! a swap of the framing layer; dispatch / governance / catalog stays.
//!
//! Usage:
//! ```text
//! cargo run -p ph2d-mcp --bin ph2d-mcp-server -- [--unsafe-mcp]
//! ```
//!
//! The `--unsafe-mcp` flag bypasses HR-11 confirmation tokens; meant
//! for CI fixtures only. Production use requires the human-issued
//! token via the (future) editor UI.
//!
//! ECS connection: M9 ships only the in-memory backend (`MemoryHost`).
//! A `SimWorldHost` impl that wraps a live `bevy_ecs::World` lands
//! when the desktop shell exposes an `--mcp-listen` flag (M10+).

use ph2d_mcp::{MemoryHost, Request, Response, Server};
use std::io::{BufRead, BufReader, Write};

fn main() -> std::io::Result<()> {
    let unsafe_mcp = std::env::args().any(|a| a == "--unsafe-mcp");
    let server = build_server(unsafe_mcp);
    let host = MemoryHost::default();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut out = stdout.lock();

    run_loop(&mut reader, &mut out, server, host)?;
    Ok(())
}

fn build_server(unsafe_mcp: bool) -> Server {
    let mut s = Server::new();
    s.register_default_tools();
    if unsafe_mcp {
        eprintln!("ph2d-mcp-server: --unsafe-mcp flag set; HR-11 destructive guard bypassed.");
        s = s.with_unsafe();
    }
    s
}

fn run_loop<R: BufRead, W: Write>(
    reader: &mut R,
    out: &mut W,
    mut server: Server,
    mut host: MemoryHost,
) -> std::io::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line)? {
            0 => break Ok(()), // EOF
            _ => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let response = handle_one(&mut server, &mut host, trimmed);
                writeln!(out, "{}", serde_json::to_string(&response).unwrap())?;
                out.flush()?;
            }
        }
    }
}

fn handle_one(server: &mut Server, host: &mut MemoryHost, raw: &str) -> Response {
    match serde_json::from_str::<Request>(raw) {
        Ok(req) => server.dispatch(host, &req),
        Err(e) => Response {
            jsonrpc: "2.0".into(),
            id: None,
            result: None,
            error: Some(ph2d_mcp::RpcError::new(
                ph2d_mcp::RpcError::PARSE_ERROR,
                format!("invalid JSON-RPC request: {e}"),
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn end_to_end_spawn_then_get_via_stdio() {
        // Two requests on stdin → two responses on stdout. Verifies
        // the binary's framing + dispatch + serialization pipeline.
        let stdin = b"\
{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"scene.spawn_entity\",\"params\":{}}\n\
{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"scene.add_component\",\"params\":{\"entity\":1,\"component\":\"Health\",\"data\":{\"value\":42}}}\n\
{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"scene.get_component\",\"params\":{\"entity\":1,\"component\":\"Health\"}}\n\
";
        let mut reader = Cursor::new(stdin);
        let mut out: Vec<u8> = Vec::new();

        let server = build_server(false);
        let host = MemoryHost::default();
        run_loop(&mut reader, &mut out, server, host).unwrap();

        let lines: Vec<&str> = std::str::from_utf8(&out).unwrap().lines().collect();
        assert_eq!(lines.len(), 3, "one response per request");

        // First response carries the new entity id.
        let r1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert!(
            r1["result"]["entity"].is_u64(),
            "spawn_entity must return integer entity id, got: {r1}"
        );

        // Third response carries the Health component we wrote.
        let r3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(
            r3["result"]["data"]["value"], 42,
            "get_component must echo the value we set, got: {r3}"
        );
    }

    #[test]
    fn parse_error_returns_jsonrpc_error_envelope() {
        let stdin = b"not json at all\n";
        let mut reader = Cursor::new(stdin);
        let mut out: Vec<u8> = Vec::new();
        let server = build_server(false);
        let host = MemoryHost::default();
        run_loop(&mut reader, &mut out, server, host).unwrap();

        let r: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(r["error"]["code"], -32700, "must report PARSE_ERROR");
    }

    #[test]
    fn destructive_blocked_in_safe_mode() {
        let stdin = b"\
{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"scene.delete_entity\",\"params\":{\"entity\":1,\"confirmation_token\":\"\"}}\n\
";
        let mut reader = Cursor::new(stdin);
        let mut out: Vec<u8> = Vec::new();
        let server = build_server(false);
        let host = MemoryHost::default();
        run_loop(&mut reader, &mut out, server, host).unwrap();

        let r: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            r["error"]["code"], -32_001,
            "must report DESTRUCTIVE_REQUIRES_TOKEN (HR-11)"
        );
    }

    #[test]
    fn unsafe_mode_allows_destructive_without_token() {
        let stdin = b"\
{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"scene.delete_entity\",\"params\":{\"entity\":1}}\n\
";
        let mut reader = Cursor::new(stdin);
        let mut out: Vec<u8> = Vec::new();
        let server = build_server(true);
        let host = MemoryHost::default();
        run_loop(&mut reader, &mut out, server, host).unwrap();

        let r: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert!(
            r["error"].is_null(),
            "unsafe mode must bypass HR-11, got error: {r}"
        );
    }
}
