//! The JSON side of the client: the answers to the server's own requests, the parsing of its
//! notifications and the request bodies (moved out of `client.rs` by the workspace LOC cap).

use super::{Diagnostic, Notification, PublishedDiagnostics, ServerMessage, ph2d_settings};
use crate::position::{Position, Range};
use serde_json::{Value, json};
use std::time::Instant;

pub(super) fn answer_server_request(method: &str, params: Option<&Value>) -> Value {
    match method {
        "workspace/configuration" => {
            let items = params
                .and_then(|p| p.get("items"))
                .and_then(Value::as_array);
            Value::Array(
                items
                    .into_iter()
                    .flatten()
                    .map(|it| match it.get("section").and_then(Value::as_str) {
                        Some("luau-lsp") => ph2d_settings(),
                        _ => Value::Null,
                    })
                    .collect(),
            )
        }
        _ => Value::Null,
    }
}

pub(super) fn parse_notification(method: &str, params: Value) -> Notification {
    match method {
        "textDocument/publishDiagnostics" => Notification::Diagnostics(PublishedDiagnostics {
            uri: params
                .get("uri")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            version: params.get("version").and_then(Value::as_i64),
            diagnostics: params
                .get("diagnostics")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(parse_diagnostic)
                .collect(),
            received: Instant::now(),
        }),
        "window/logMessage" | "window/showMessage" => Notification::Message(ServerMessage {
            shown: method == "window/showMessage",
            level: params.get("type").and_then(Value::as_u64).unwrap_or(4) as u8,
            text: params
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        }),
        _ => Notification::Other {
            method: method.to_owned(),
            params,
        },
    }
}

pub(super) fn parse_diagnostic(d: &Value) -> Diagnostic {
    Diagnostic {
        range: d.get("range").map(parse_range).unwrap_or_default(),
        severity: d.get("severity").and_then(Value::as_u64).map(|s| s as u8),
        code: d.get("code").and_then(|c| match c {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        }),
        message: d
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    }
}

pub(super) fn parse_position(p: &Value) -> Position {
    let n = |k| p.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
    Position::new(n("line"), n("character"))
}

pub(super) fn parse_range(r: &Value) -> Range {
    Range {
        start: r.get("start").map(parse_position).unwrap_or_default(),
        end: r.get("end").map(parse_position).unwrap_or_default(),
    }
}

pub(super) fn range_json(r: Range) -> Value {
    json!({ "start": { "line": r.start.line, "character": r.start.character },
            "end": { "line": r.end.line, "character": r.end.character } })
}

pub(super) fn text_position(uri: &str, at: Position) -> Value {
    json!({ "textDocument": { "uri": uri }, "position": { "line": at.line, "character": at.character } })
}

pub(super) fn hover_text(contents: &Value) -> String {
    match contents {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        Value::Array(a) => a.iter().map(hover_text).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

pub(super) fn initialize_params() -> Value {
    json!({
        "processId": std::process::id(),
        "rootUri": null,
        "capabilities": {
            "general": { "positionEncodings": ["utf-16"] },
            "workspace": { "configuration": true },
            "textDocument": {
                "synchronization": { "didSave": false },
                "publishDiagnostics": {},
                "completion": { "completionItem": { "snippetSupport": false } },
                "hover": { "contentFormat": ["markdown", "plaintext"] }
            }
        },
        "initializationOptions": {}
    })
}
