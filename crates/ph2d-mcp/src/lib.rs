#![forbid(unsafe_code)]
// ph2d-mcp is presentation/IO (LLM-facing tooling), NOT in the simulation
// path defined by ADR-0022. HashMap iteration here is fine.
#![allow(clippy::disallowed_types)]
//! ph2d-mcp — MCP server skeleton (HR-10, HR-11).
//!
//! JSON-RPC 2.0 dispatcher para tools registradas. Governance (HR-11):
//! tools `destructive: true` exigem `confirmation_token` válido (single-use,
//! 5 min TTL). Audit log JSONL append-only para toda mutação destrutiva.
//!
//! Backend agnóstico: o caller implementa trait `McpHost` que faz a ponte
//! com World/MessageBus/AssetDb concretos. Skeleton inclui implementação
//! mock (`MemoryHost`) para testes e exemplos.
//!
//! Surface deste skeleton (será expandido em S2/S3):
//! - `scene.spawn_entity` (returns u64 entity)
//! - `scene.add_component(entity, name, data)`
//! - `scene.get_component(entity, name) → data?`
//! - `message.send(target, message, payload)`
//! - `scene.delete_entity(entity, confirmation_token)` — destrutivo

pub mod governance;
pub mod host;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub use governance::{ConfirmationStore, ConfirmationToken};
pub use host::{McpHost, MemoryHost};

/// JSON-RPC 2.0 request envelope.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Request {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    /// HR-11: destrutivo sem token. Custom code in PH2D namespace.
    pub const DESTRUCTIVE_REQUIRES_TOKEN: i32 = -32_001;

    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

/// Schema metadata about a tool, attached to each registered tool.
#[derive(Serialize, Clone, Debug)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    /// HR-11: destrutivo exige confirmation_token (gerado por UI humana,
    /// válido por 5 min, single-use) ou flag `--unsafe-mcp` no servidor.
    pub destructive: bool,
}

pub struct Server {
    tools: HashMap<String, ToolSchema>,
    confirmations: ConfirmationStore,
    /// Audit log lines (JSONL) — flushed by the caller when desired.
    audit: Vec<String>,
    /// Bypass governance (CI/dev mode, --unsafe-mcp).
    unsafe_mcp: bool,
}

impl Server {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            confirmations: ConfirmationStore::new(),
            audit: Vec::new(),
            unsafe_mcp: false,
        }
    }

    pub fn with_unsafe(mut self) -> Self {
        self.unsafe_mcp = true;
        self
    }

    pub fn register(&mut self, schema: ToolSchema) {
        self.tools.insert(schema.name.clone(), schema);
    }

    /// Default tool catalog (the 5 tools of c6-prompts.md). Caller wires
    /// these to a concrete McpHost via `dispatch`.
    pub fn register_default_tools(&mut self) {
        for tool in [
            ToolSchema {
                name: "scene.spawn_entity".into(),
                description: "Spawn empty entity, returns Entity handle.".into(),
                destructive: false,
            },
            ToolSchema {
                name: "scene.add_component".into(),
                description: "Attach a component (JSON data) to an entity.".into(),
                destructive: false,
            },
            ToolSchema {
                name: "scene.get_component".into(),
                description: "Read a component by name from an entity.".into(),
                destructive: false,
            },
            ToolSchema {
                name: "message.send".into(),
                description: "Send a Defold-style message to a target entity.".into(),
                destructive: false,
            },
            ToolSchema {
                name: "scene.delete_entity".into(),
                description:
                    "Despawn an entity. Destructive — requires confirmation_token (HR-11).".into(),
                destructive: true,
            },
        ] {
            self.register(tool);
        }
    }

    pub fn list_tools(&self) -> Vec<&ToolSchema> {
        self.tools.values().collect()
    }

    pub fn audit_lines(&self) -> &[String] {
        &self.audit
    }

    pub fn issue_confirmation(&mut self) -> ConfirmationToken {
        self.confirmations.issue()
    }

    /// Dispatch a single JSON-RPC request against a host.
    pub fn dispatch<H: McpHost>(&mut self, host: &mut H, req: &Request) -> Response {
        let response = self.dispatch_inner(host, req);
        if let Some(audit_entry) = self.audit_for(req, &response) {
            self.audit.push(audit_entry);
        }
        response
    }

    fn dispatch_inner<H: McpHost>(&mut self, host: &mut H, req: &Request) -> Response {
        let id = req.id.clone();
        let tool = match self.tools.get(&req.method) {
            Some(t) => t.clone(),
            None => {
                return Response {
                    jsonrpc: "2.0".into(),
                    id,
                    result: None,
                    error: Some(RpcError::new(
                        RpcError::METHOD_NOT_FOUND,
                        format!("unknown tool: {}", req.method),
                    )),
                };
            }
        };

        if tool.destructive && !self.unsafe_mcp {
            // Validate confirmation_token in params.
            let token = req
                .params
                .get("confirmation_token")
                .and_then(|v| v.as_str());
            let ok = match token {
                Some(t) => self.confirmations.consume(t),
                None => false,
            };
            if !ok {
                return Response {
                    jsonrpc: "2.0".into(),
                    id,
                    result: None,
                    error: Some(RpcError::new(
                        RpcError::DESTRUCTIVE_REQUIRES_TOKEN,
                        format!(
                            "tool {} is destructive (HR-11); valid confirmation_token required",
                            req.method
                        ),
                    )),
                };
            }
        }

        let result: Result<Value, RpcError> = match req.method.as_str() {
            "scene.spawn_entity" => {
                let e = host.spawn_entity();
                Ok(serde_json::json!({ "entity": e }))
            }
            "scene.add_component" => (|| {
                let entity = req
                    .params
                    .get("entity")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing entity"))?;
                let name = req
                    .params
                    .get("component")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing component"))?;
                let data = req.params.get("data").cloned().unwrap_or(Value::Null);
                host.add_component(entity, name, data);
                Ok(serde_json::json!({ "ok": true }))
            })(),
            "scene.get_component" => (|| {
                let entity = req
                    .params
                    .get("entity")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing entity"))?;
                let name = req
                    .params
                    .get("component")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing component"))?;
                let data = host.get_component(entity, name);
                Ok(serde_json::json!({ "data": data }))
            })(),
            "message.send" => (|| {
                let target = req
                    .params
                    .get("target")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing target"))?;
                let message = req
                    .params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing message"))?;
                let payload = req.params.get("payload").cloned().unwrap_or(Value::Null);
                host.send_message(target, message, payload);
                Ok(serde_json::json!({ "ok": true }))
            })(),
            "scene.delete_entity" => (|| {
                let entity = req
                    .params
                    .get("entity")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::new(RpcError::INVALID_PARAMS, "missing entity"))?;
                let removed = host.delete_entity(entity);
                Ok(serde_json::json!({ "removed": removed }))
            })(),
            _ => Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("dispatch missing for {}", req.method),
            )),
        };

        match result {
            Ok(value) => Response {
                jsonrpc: "2.0".into(),
                id,
                result: Some(value),
                error: None,
            },
            Err(err) => Response {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(err),
            },
        }
    }

    fn audit_for(&self, req: &Request, resp: &Response) -> Option<String> {
        // Per HR-11: audit_log is JSONL append-only with hashes pre/post + agent + params.
        let tool = self.tools.get(&req.method)?;
        if !tool.destructive {
            return None;
        }
        let entry = serde_json::json!({
            "timestamp_ns": now_ns(),
            "agent": "mcp:claude",
            "tool": &req.method,
            "params": &req.params,
            "ok": resp.error.is_none(),
            "state_hash_after": blake3::hash(&serde_json::to_vec(&resp).ok()?).to_hex().to_string(),
        });
        Some(entry.to_string())
    }
}

impl Default for Server {
    fn default() -> Self {
        let mut s = Self::new();
        s.register_default_tools();
        s
    }
}

fn now_ns() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rpc(method: &str, params: Value) -> Request {
        Request {
            jsonrpc: "2.0".into(),
            id: Some(Value::from(1)),
            method: method.into(),
            params,
        }
    }

    #[test]
    fn spawn_then_add_then_get() {
        let mut server = Server::default();
        let mut host = MemoryHost::default();
        let r = server.dispatch(&mut host, &rpc("scene.spawn_entity", Value::Null));
        let entity = r.result.unwrap()["entity"].as_u64().unwrap();
        let _ = server.dispatch(
            &mut host,
            &rpc(
                "scene.add_component",
                serde_json::json!({
                    "entity": entity, "component": "Health", "data": { "value": 42 }
                }),
            ),
        );
        let r = server.dispatch(
            &mut host,
            &rpc(
                "scene.get_component",
                serde_json::json!({
                    "entity": entity, "component": "Health"
                }),
            ),
        );
        let data = &r.result.unwrap()["data"];
        assert_eq!(data["value"], 42);
    }

    #[test]
    fn destructive_blocked_without_token() {
        let mut server = Server::default();
        let mut host = MemoryHost::default();
        let r = server.dispatch(
            &mut host,
            &rpc("scene.delete_entity", serde_json::json!({ "entity": 1 })),
        );
        assert!(r.error.is_some());
        assert_eq!(r.error.unwrap().code, RpcError::DESTRUCTIVE_REQUIRES_TOKEN);
    }

    #[test]
    fn destructive_allowed_with_token() {
        let mut server = Server::default();
        let mut host = MemoryHost::default();
        let token = server.issue_confirmation();
        let r = server.dispatch(
            &mut host,
            &rpc(
                "scene.delete_entity",
                serde_json::json!({
                    "entity": 1, "confirmation_token": token.value()
                }),
            ),
        );
        assert!(r.error.is_none(), "expected ok, got {:?}", r.error);
        let r = server.dispatch(
            &mut host,
            &rpc(
                "scene.delete_entity",
                serde_json::json!({
                    "entity": 2, "confirmation_token": token.value()
                }),
            ),
        );
        assert!(r.error.is_some(), "single-use token must fail second time");
    }

    #[test]
    fn audit_log_records_destructive() {
        let mut server = Server::default();
        let mut host = MemoryHost::default();
        let token = server.issue_confirmation();
        server.dispatch(
            &mut host,
            &rpc(
                "scene.delete_entity",
                serde_json::json!({
                    "entity": 7, "confirmation_token": token.value()
                }),
            ),
        );
        server.dispatch(&mut host, &rpc("scene.spawn_entity", Value::Null));
        assert_eq!(server.audit_lines().len(), 1);
        assert!(server.audit_lines()[0].contains("\"tool\":\"scene.delete_entity\""));
    }
}
