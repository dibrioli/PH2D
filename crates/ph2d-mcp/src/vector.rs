//! Vector-authoring MCP surface (ADR-0061, Inovação #4 — LLM4SVG).
//!
//! Six tools that let an **external** agent (Claude over MCP) author *editable*
//! vector geometry. The agent itself is the LLM: it emits an LLM4SVG semantic
//! token blob (a `spiral` with `turns`, a `polygon` with `sides`…) and calls
//! [`Server::dispatch_vector`](crate::Server::dispatch_vector). The server runs
//! every blob through [`ph2d_vector_llm::build_network_from_json`] — the
//! security-critical, bounds-*before*-allocation sanitizer — so a host can never
//! be handed an unsanitized blob. **The trust boundary is the dispatch layer.**
//!
//! The scene backend is abstracted by [`VectorSceneHost`]; the editor bridges it
//! to the live vector document, while [`MemoryVectorScene`] drives the gates.

use ph2d_vector_doc::VectorNetwork;
use std::collections::BTreeMap;

/// A compact, host-computed summary of one path — the `vector.inspect` reply.
///
/// Counts plus the axis-aligned bounds over vertex positions. Deliberately
/// small (no geometry) so the MCP reply stays cheap regardless of path size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathSummary {
    pub vertices: usize,
    pub segments: usize,
    pub regions: usize,
    /// `[min_x, min_y, max_x, max_y]`; all-zero for an empty (vertex-less) path.
    pub bounds: [f32; 4],
}

impl PathSummary {
    /// Summarize a network: element counts + AABB over vertex positions.
    #[must_use]
    pub fn of(net: &VectorNetwork) -> Self {
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for v in &net.vertices {
            min[0] = min[0].min(v.pos.x);
            min[1] = min[1].min(v.pos.y);
            max[0] = max[0].max(v.pos.x);
            max[1] = max[1].max(v.pos.y);
        }
        let bounds = if net.vertices.is_empty() {
            [0.0; 4]
        } else {
            [min[0], min[1], max[0], max[1]]
        };
        Self {
            vertices: net.vertices.len(),
            segments: net.segments.len(),
            regions: net.regions.len(),
            bounds,
        }
    }
}

/// Backend for the six `vector.*` MCP tools.
///
/// The real implementation bridges to the editor's vector document (or the ECS
/// scene); [`MemoryVectorScene`] is the reference in-memory backend used by the
/// gates and the server bin. Networks handed to [`add_path`](Self::add_path) /
/// [`replace_path`](Self::replace_path) are **already sanitized** by the
/// dispatch layer — the host stores them as-is.
pub trait VectorSceneHost {
    /// Insert a freshly-built (already-sanitized) network; returns its new id.
    fn add_path(&mut self, network: VectorNetwork) -> u64;
    /// Replace the geometry of an existing path; `false` if `path_id` is unknown.
    fn replace_path(&mut self, path_id: u64, network: VectorNetwork) -> bool;
    /// Ids of all paths currently in the scene, ascending.
    fn list_paths(&self) -> Vec<u64>;
    /// Inspect one path; `None` if `path_id` is unknown.
    fn inspect_path(&self, path_id: u64) -> Option<PathSummary>;
    /// Delete one path; `false` if `path_id` is unknown. **Destructive (HR-11).**
    fn delete_path(&mut self, path_id: u64) -> bool;
    /// Remove every path; returns how many were removed. **Destructive (HR-11).**
    fn clear_scene(&mut self) -> usize;
}

/// Reference in-memory [`VectorSceneHost`]: monotonic ids, `BTreeMap`-backed so
/// [`list_paths`](VectorSceneHost::list_paths) is sorted for free.
#[derive(Default)]
pub struct MemoryVectorScene {
    next: u64,
    paths: BTreeMap<u64, VectorNetwork>,
}

impl MemoryVectorScene {
    /// Borrow a stored network (test/inspection aid; not part of the trait).
    #[must_use]
    pub fn path(&self, path_id: u64) -> Option<&VectorNetwork> {
        self.paths.get(&path_id)
    }

    /// Number of paths currently held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// True when the scene holds no paths.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

impl VectorSceneHost for MemoryVectorScene {
    fn add_path(&mut self, network: VectorNetwork) -> u64 {
        self.next += 1;
        self.paths.insert(self.next, network);
        self.next
    }

    fn replace_path(&mut self, path_id: u64, network: VectorNetwork) -> bool {
        match self.paths.get_mut(&path_id) {
            Some(slot) => {
                *slot = network;
                true
            }
            None => false,
        }
    }

    fn list_paths(&self) -> Vec<u64> {
        self.paths.keys().copied().collect()
    }

    fn inspect_path(&self, path_id: u64) -> Option<PathSummary> {
        self.paths.get(&path_id).map(PathSummary::of)
    }

    fn delete_path(&mut self, path_id: u64) -> bool {
        self.paths.remove(&path_id).is_some()
    }

    fn clear_scene(&mut self) -> usize {
        let n = self.paths.len();
        self.paths.clear();
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Request, RpcError, Server};
    use serde_json::{Value, json};

    const HEXAGON: &str = r#"{ "shape_type": "polygon", "params": { "sides": 6, "radius": 80 } }"#;

    fn rpc(method: &str, params: Value) -> Request {
        Request {
            jsonrpc: "2.0".into(),
            id: Some(Value::from(1)),
            method: method.into(),
            params,
        }
    }

    #[test]
    fn paint_then_query_then_inspect() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();

        let r = server.dispatch_vector(
            &mut scene,
            &rpc("vector.paint_shape", json!({ "blob": HEXAGON })),
        );
        let res = r.result.expect("paint ok");
        let path_id = res["path_id"].as_u64().unwrap();
        assert_eq!(res["vertices"].as_u64().unwrap(), 6);

        let r = server.dispatch_vector(&mut scene, &rpc("vector.query", Value::Null));
        let paths = r.result.unwrap()["paths"].as_array().unwrap().clone();
        assert_eq!(paths, vec![Value::from(path_id)]);

        let r = server.dispatch_vector(
            &mut scene,
            &rpc("vector.inspect", json!({ "path_id": path_id })),
        );
        let s = r.result.unwrap();
        assert_eq!(s["found"], json!(true));
        assert_eq!(s["vertices"].as_u64().unwrap(), 6);
        assert_eq!(s["bounds"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn modify_replaces_geometry() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        let path_id = server
            .dispatch_vector(
                &mut scene,
                &rpc("vector.paint_shape", json!({ "blob": HEXAGON })),
            )
            .result
            .unwrap()["path_id"]
            .as_u64()
            .unwrap();

        let octagon = r#"{ "shape_type": "polygon", "params": { "sides": 8, "radius": 40 } }"#;
        let r = server.dispatch_vector(
            &mut scene,
            &rpc(
                "vector.modify",
                json!({ "path_id": path_id, "blob": octagon }),
            ),
        );
        assert_eq!(r.result.unwrap()["replaced"], json!(true));
        assert_eq!(scene.inspect_path(path_id).unwrap().vertices, 8);

        // Unknown id → replaced=false, no error.
        let r = server.dispatch_vector(
            &mut scene,
            &rpc("vector.modify", json!({ "path_id": 999, "blob": octagon })),
        );
        assert_eq!(r.result.unwrap()["replaced"], json!(false));
    }

    #[test]
    fn adversarial_blob_rejected_not_materialized() {
        // turns: 1e9 → the sanitizer rejects before any geometry exists.
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        let evil = r#"{ "shape_type": "spiral", "params": { "turns": 1000000000 } }"#;
        let r = server.dispatch_vector(
            &mut scene,
            &rpc("vector.paint_shape", json!({ "blob": evil })),
        );
        assert_eq!(r.error.unwrap().code, RpcError::INVALID_PARAMS);
        assert!(scene.is_empty(), "rejected blob must not enter the scene");
    }

    #[test]
    fn delete_path_blocked_without_token_then_allowed() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        let path_id = server
            .dispatch_vector(
                &mut scene,
                &rpc("vector.paint_shape", json!({ "blob": HEXAGON })),
            )
            .result
            .unwrap()["path_id"]
            .as_u64()
            .unwrap();

        // HR-11: destructive without a token is rejected.
        let r = server.dispatch_vector(
            &mut scene,
            &rpc("vector.delete_path", json!({ "path_id": path_id })),
        );
        assert_eq!(r.error.unwrap().code, RpcError::DESTRUCTIVE_REQUIRES_TOKEN);
        assert_eq!(scene.len(), 1, "blocked delete must not mutate the scene");

        // With a valid token it succeeds and the path is gone.
        let token = server.issue_confirmation();
        let r = server.dispatch_vector(
            &mut scene,
            &rpc(
                "vector.delete_path",
                json!({ "path_id": path_id, "confirmation_token": token.value() }),
            ),
        );
        assert_eq!(r.result.unwrap()["removed"], json!(true));
        assert!(scene.is_empty());
    }

    #[test]
    fn clear_scene_is_destructive_and_counts() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        for _ in 0..3 {
            server.dispatch_vector(
                &mut scene,
                &rpc("vector.paint_shape", json!({ "blob": HEXAGON })),
            );
        }
        // Without a token → rejected, scene untouched.
        let r = server.dispatch_vector(&mut scene, &rpc("vector.clear_scene", Value::Null));
        assert_eq!(r.error.unwrap().code, RpcError::DESTRUCTIVE_REQUIRES_TOKEN);
        assert_eq!(scene.len(), 3);

        let token = server.issue_confirmation();
        let r = server.dispatch_vector(
            &mut scene,
            &rpc(
                "vector.clear_scene",
                json!({ "confirmation_token": token.value() }),
            ),
        );
        assert_eq!(r.result.unwrap()["removed_count"].as_u64().unwrap(), 3);
        assert!(scene.is_empty());
    }

    #[test]
    fn destructive_vector_ops_are_audited() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        let path_id = server
            .dispatch_vector(
                &mut scene,
                &rpc("vector.paint_shape", json!({ "blob": HEXAGON })),
            )
            .result
            .unwrap()["path_id"]
            .as_u64()
            .unwrap();
        let token = server.issue_confirmation();
        server.dispatch_vector(
            &mut scene,
            &rpc(
                "vector.delete_path",
                json!({ "path_id": path_id, "confirmation_token": token.value() }),
            ),
        );
        // paint_shape (non-destructive) is NOT audited; delete_path is.
        assert_eq!(server.audit_lines().len(), 1);
        assert!(server.audit_lines()[0].contains("\"tool\":\"vector.delete_path\""));
    }

    #[test]
    fn inspect_unknown_path_reports_not_found() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        let r =
            server.dispatch_vector(&mut scene, &rpc("vector.inspect", json!({ "path_id": 42 })));
        assert_eq!(r.result.unwrap()["found"], json!(false));
    }

    #[test]
    fn missing_blob_is_invalid_params() {
        let mut server = Server::default();
        let mut scene = MemoryVectorScene::default();
        let r = server.dispatch_vector(&mut scene, &rpc("vector.paint_shape", json!({})));
        assert_eq!(r.error.unwrap().code, RpcError::INVALID_PARAMS);
    }
}
