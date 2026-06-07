//! `ph2d-vector-llm-client` — the network I/O half of LLM-as-graph-node (W13
//! Coord, Inovação P4, ADR-0061).
//!
//! The impl shipped the PURE, deterministic core in [`ph2d_vector_llm`]
//! (`build_from_json` = size-guard → parse → sanitize → lower; plus the
//! `ResultCache` and HR-11 governance). This crate adds the only thing that
//! crate deliberately excludes — the **untrusted network call**:
//!
//! - [`LlmTransport`] abstracts "ask the model for an LLM4SVG blob". The real
//!   [`AnthropicTransport`] calls the Messages API over HTTPS with a **15 s
//!   per-call timeout** ([`LLM_TIMEOUT_SECS`]); a [`MockTransport`] drives the
//!   deterministic gates with no network and no API key.
//! - [`LlmClient::generate_shape`] runs the call, and on ANY transport failure
//!   (timeout, network, HTTP error) falls back **gracefully** to the last cached
//!   result for `(prompt, seed)` — so a slow or down model degrades to the last
//!   good shape instead of an error or a hang. The bad-input path stays the
//!   impl's: a structurally-valid but out-of-bounds blob is REJECTED by the
//!   sanitizer inside `build_from_json` and never materialised.
//!
//! **Sync by design.** The engine and `ph2d-mcp` are synchronous, so this is a
//! blocking call with the timeout inside the transport (`ureq`) rather than a
//! tokio task — ~10× lighter than dragging an async runtime into a sync engine.
//! The host runs `generate_shape` on a worker thread (the ≤15 s block must not be
//! on the UI thread); the [`LlmTransport`] trait keeps an async transport a
//! drop-in swap if a future async MCP server ever needs one.

use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_llm::{CacheKey, LlmError, ResultCache, build_from_json, tokens_to_network};

/// The LLM call's hard wall-clock budget. A response that takes longer is
/// abandoned and the client falls back to the cache (Inovação P4 §2.6). 45 s
/// (was 15) leaves margin for an Opus structured-output round-trip under load;
/// the request disables thinking (a trivial shape spec needs none) so the call
/// is normally a few seconds.
pub const LLM_TIMEOUT_SECS: u64 = 45;

/// The model the shape generator targets — the latest, most capable Claude
/// (the project default; see CLAUDE.md / claude-api skill).
pub const MODEL: &str = "claude-opus-4-8";

/// Why a single LLM fetch did not return a usable LLM4SVG blob. Every variant
/// triggers the cache fallback in [`LlmClient::generate_shape`].
#[derive(Debug)]
pub enum FetchError {
    /// The call exceeded [`LLM_TIMEOUT_SECS`].
    Timeout,
    /// A non-2xx HTTP status.
    Http { status: u16, body: String },
    /// A transport/connection failure (DNS, TLS, reset, …).
    Network(String),
    /// The response had no text content block to read the blob from.
    NoContent,
    /// `ANTHROPIC_API_KEY` was not set (real transport only).
    MissingApiKey,
}

impl core::fmt::Display for FetchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FetchError::Timeout => write!(f, "LLM call timed out after {LLM_TIMEOUT_SECS}s"),
            FetchError::Http { status, .. } => write!(f, "LLM HTTP error {status}"),
            FetchError::Network(e) => write!(f, "LLM network error: {e}"),
            FetchError::NoContent => write!(f, "LLM response had no text content"),
            FetchError::MissingApiKey => write!(f, "ANTHROPIC_API_KEY not set"),
        }
    }
}

impl std::error::Error for FetchError {}

/// Why [`LlmClient::generate_shape`] could not produce a network.
#[derive(Debug)]
pub enum GenError {
    /// The fetch failed AND no cached fallback existed for `(prompt, seed)`.
    Fetch(FetchError),
    /// The fetched blob was structurally valid but failed validation/bounds —
    /// the sanitizer rejected it (it was never materialised). The caller decides
    /// whether to retry; the bad shape is never produced.
    Build(LlmError),
}

impl core::fmt::Display for GenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GenError::Fetch(e) => write!(f, "{e} (no cached fallback)"),
            GenError::Build(e) => write!(f, "LLM blob rejected: {e}"),
        }
    }
}

impl std::error::Error for GenError {}

/// "Ask the model for an LLM4SVG blob given `system` + `user`, constrained to
/// `schema`." Returns the LLM4SVG JSON text (what [`build_from_json`] consumes).
/// The real impl does the HTTPS round-trip + timeout; a mock drives the gates.
pub trait LlmTransport {
    fn complete(
        &self,
        system: &str,
        user: &str,
        schema: &serde_json::Value,
    ) -> Result<String, FetchError>;
}

/// Drives the LLM4SVG pipeline: fetch → validate → cache, with a graceful cache
/// fallback when the fetch fails. Generic over the transport so the gates can
/// inject a [`MockTransport`].
pub struct LlmClient<T> {
    transport: T,
    system: String,
    schema: serde_json::Value,
}

impl<T: LlmTransport> LlmClient<T> {
    /// Build a client over `transport`, with the canonical LLM4SVG system prompt
    /// + output schema (the §2.5 "schema injection" — host's job, here).
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            system: llm4svg_system_prompt(),
            schema: llm4svg_schema(),
        }
    }

    /// Generate a [`VectorNetwork`] for `prompt` (deterministic per `seed`).
    ///
    /// On a successful fetch the blob is validated by `build_from_json` (parse +
    /// sanitize + lower), cached for `(prompt, seed)`, and returned. On ANY fetch
    /// failure (timeout / network / HTTP) the client falls back to the cached
    /// result for that key — the graceful-degradation path the timeout gate
    /// proves. A fetch that succeeds but yields an out-of-bounds blob returns
    /// [`GenError::Build`] (rejected, never materialised) rather than silently
    /// serving stale geometry.
    pub fn generate_shape(
        &self,
        prompt: &str,
        seed: u64,
        cache: &mut ResultCache,
    ) -> Result<VectorNetwork, GenError> {
        self.generate_shape_with_blob(prompt, seed, cache)
            .map(|(network, _blob)| network)
    }

    /// As [`generate_shape`](Self::generate_shape), but also surfaces the **raw
    /// LLM4SVG JSON blob** the model produced.
    ///
    /// This is the seam for the `vector.llm-shape` graph node (ADR-0061 §2.1,
    /// the "path A" of `HANDOFF_vector_w13_host_wiring_spec_impl.md`): a host
    /// driving that node populates its `seed → raw_json`
    /// [`LlmResponseSource`](https://docs.rs/ph2d-node-vector-llm-shape) from the
    /// returned blob, and the node re-runs `build_network_from_json` so it stays
    /// `Effect::Pure` + self-contained — **without** a second API round-trip or a
    /// tokens→JSON re-serializer. The blob is `Some` on a live fetch and `None` on
    /// a cache-fallback (the [`ResultCache`] holds sanitized *tokens*, not the
    /// original text); on `None` the host keeps the seed's previously-stored blob.
    ///
    /// Hosts that inject the [`VectorNetwork`] straight into the document
    /// ("path B", the MVP) can keep calling [`generate_shape`](Self::generate_shape)
    /// and ignore the blob.
    pub fn generate_shape_with_blob(
        &self,
        prompt: &str,
        seed: u64,
        cache: &mut ResultCache,
    ) -> Result<(VectorNetwork, Option<String>), GenError> {
        let key = CacheKey::new(prompt, seed);
        match self.transport.complete(&self.system, prompt, &self.schema) {
            Ok(json) => {
                let (tokens, network) = build_from_json(&json).map_err(GenError::Build)?;
                cache.insert(key, tokens); // refresh the fallback for next time
                Ok((network, Some(json)))
            }
            Err(fetch_err) => match cache.get(&key) {
                // Cached tokens are already sanitized → lowering is safe. No raw
                // blob survives the cache (it stores tokens), so the node keeps
                // whatever blob it last held for this seed.
                Some(tokens) => Ok((tokens_to_network(tokens), None)),
                None => Err(GenError::Fetch(fetch_err)),
            },
        }
    }
}

// ─────────────────────────── LLM4SVG prompt + schema ──────────────────────────

/// The system prompt injected for shape generation — instructs the model to emit
/// ONLY an LLM4SVG blob in the `{shape_type, params, style}` shape `parse` reads.
#[must_use]
pub fn llm4svg_system_prompt() -> String {
    "You are a vector-shape generator for the PH2D editor. Given a natural-language \
     request, respond with a single LLM4SVG object describing ONE shape. \
     Use this exact shape: {\"shape_type\": <one of spiral|polygon|star|ellipse|rect|path>, \
     \"params\": { shape-specific keys }, \"style\": {\"stroke_width\": <number>, \
     \"stroke_color_oklch\": [L, C, H], \"fill\": \"none\"|\"solid\"}}. \
     Coordinates are canvas pixels; centre points are [x, y] arrays. \
     Keep shapes within reasonable bounds (the editor clamps unsafe values). \
     Param keys by shape: spiral{center,inner_radius,outer_radius,turns,samples_per_turn,rotation}; \
     polygon{center,radius,sides,rotation}; star{center,outer_radius,inner_radius,points,rotation}; \
     ellipse{center,radii}; rect{corner_a,corner_b}; path{vertices,closed}."
        .to_string()
}

/// The JSON Schema constraining the LLM's output (Anthropic `output_config.format`).
/// `params` enumerates the union of all shape keys with `additionalProperties:false`
/// (the typed `parse` reads the subset each shape needs; the sanitizer enforces
/// the frozen caps). Avoids numeric/array constraints (unsupported by structured
/// outputs) — bounds are the sanitizer's job, not the schema's.
#[must_use]
pub fn llm4svg_schema() -> serde_json::Value {
    use serde_json::json;
    let num = json!({ "type": "number" });
    let int = json!({ "type": "integer" });
    let arr = json!({ "type": "array" });
    json!({
        "type": "object",
        "properties": {
            "shape_type": {
                "type": "string",
                "enum": ["spiral", "polygon", "star", "ellipse", "rect", "path"],
            },
            "params": {
                "type": "object",
                "properties": {
                    "center": arr, "radius": num, "radii": arr, "sides": int,
                    "points": int, "turns": num, "inner_radius": num,
                    "outer_radius": num, "rotation": num, "samples_per_turn": int,
                    "corner_a": arr, "corner_b": arr, "vertices": arr,
                    "closed": { "type": "boolean" },
                },
                "additionalProperties": false,
            },
            "style": {
                "type": "object",
                "properties": {
                    "stroke_width": num,
                    "stroke_color_oklch": arr,
                    "fill": { "type": "string", "enum": ["none", "solid"] },
                },
                "additionalProperties": false,
            },
        },
        "required": ["shape_type"],
        "additionalProperties": false,
    })
}

// ─────────────────────────── real Anthropic transport ─────────────────────────

/// The production [`LlmTransport`] — Claude Messages API over HTTPS (`ureq`,
/// rustls), with the [`LLM_TIMEOUT_SECS`] per-call timeout and the API key read
/// from `ANTHROPIC_API_KEY`. Structured output (`output_config.format`) pins the
/// response to the LLM4SVG schema, so `content[0].text` is the blob verbatim.
pub struct AnthropicTransport {
    agent: ureq::Agent,
    api_key: String,
}

impl AnthropicTransport {
    /// Build the transport, reading `ANTHROPIC_API_KEY` from the environment
    /// (never hardcode a key). Errors if it is absent.
    pub fn from_env() -> Result<Self, FetchError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| FetchError::MissingApiKey)?;
        Ok(Self::with_key(api_key))
    }

    /// Build over an explicit key (e.g. from the host's secret store).
    #[must_use]
    pub fn with_key(api_key: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(LLM_TIMEOUT_SECS))
            .build();
        Self { agent, api_key }
    }
}

impl LlmTransport for AnthropicTransport {
    fn complete(
        &self,
        system: &str,
        user: &str,
        schema: &serde_json::Value,
    ) -> Result<String, FetchError> {
        let body = serde_json::json!({
            "model": MODEL,
            "max_tokens": 4096,
            "system": system,
            // No thinking: lowering one prompt to a tiny `{shape_type, params,
            // style}` object is a trivial mapping, and disabling it keeps the
            // interactive round-trip to a few seconds (the response is
            // schema-constrained, so there's no stray reasoning to leak).
            "thinking": { "type": "disabled" },
            // Structured output → the response IS the LLM4SVG JSON (no prose).
            "output_config": { "format": { "type": "json_schema", "schema": schema } },
            "messages": [{ "role": "user", "content": user }],
        });
        let resp = self
            .agent
            .post("https://api.anthropic.com/v1/messages")
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_string(&body.to_string());
        let text = match resp {
            Ok(r) => r
                .into_string()
                .map_err(|e| FetchError::Network(e.to_string()))?,
            Err(ureq::Error::Status(status, r)) => {
                return Err(FetchError::Http {
                    status,
                    body: r.into_string().unwrap_or_default(),
                });
            }
            Err(ureq::Error::Transport(t)) => {
                // ureq surfaces a per-call timeout as an Io transport error.
                let msg = t.to_string();
                let timed_out = matches!(t.kind(), ureq::ErrorKind::Io)
                    && (msg.contains("timed out") || msg.contains("timeout"));
                return Err(if timed_out {
                    FetchError::Timeout
                } else {
                    FetchError::Network(msg)
                });
            }
        };
        // Extract the first text content block — with output_config.format it is
        // the validated LLM4SVG JSON.
        let v: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| FetchError::Network(e.to_string()))?;
        v.get("content")
            .and_then(|c| c.as_array())
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
            })
            .and_then(|b| b.get("text"))
            .and_then(|t| t.as_str())
            .map(str::to_string)
            .ok_or(FetchError::NoContent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// A transport whose response is fully scripted — no network, no key. `outcome`
    /// is what its single `complete` returns.
    struct MockTransport {
        outcome: Cell<Option<Result<String, FetchError>>>,
    }
    impl MockTransport {
        fn ok(json: &str) -> Self {
            Self {
                outcome: Cell::new(Some(Ok(json.to_string()))),
            }
        }
        fn err(e: FetchError) -> Self {
            Self {
                outcome: Cell::new(Some(Err(e))),
            }
        }
    }
    impl LlmTransport for MockTransport {
        fn complete(&self, _: &str, _: &str, _: &serde_json::Value) -> Result<String, FetchError> {
            self.outcome.take().expect("mock called once")
        }
    }

    const HEXAGON: &str = r#"{"shape_type":"polygon","params":{"center":[0.0,0.0],"radius":100.0,"sides":6,"rotation":0.0},"style":{"stroke_width":2.0,"stroke_color_oklch":[0.6,0.1,40.0],"fill":"solid"}}"#;

    #[test]
    fn generate_shape_success_validates_and_caches() {
        let client = LlmClient::new(MockTransport::ok(HEXAGON));
        let mut cache = ResultCache::new(8);
        let net = client
            .generate_shape("a hexagon", 0, &mut cache)
            .expect("built");
        assert!(net.validate().is_ok(), "the built network is valid");
        assert_eq!(cache.len(), 1, "the success is cached for fallback");
    }

    #[test]
    fn generate_shape_with_blob_surfaces_raw_json_then_none_on_fallback() {
        // Path A (graph node): a live fetch returns the raw blob verbatim, so the
        // host can populate the node's seed→raw_json source without re-fetching.
        let mut cache = ResultCache::new(8);
        let live = LlmClient::new(MockTransport::ok(HEXAGON));
        let (net, blob) = live
            .generate_shape_with_blob("a hexagon", 3, &mut cache)
            .expect("built");
        assert!(net.validate().is_ok());
        assert_eq!(
            blob.as_deref(),
            Some(HEXAGON),
            "live fetch surfaces the blob"
        );

        // A cache-fallback yields no raw blob (the cache holds tokens) → None, and
        // the host keeps the seed's prior blob.
        let down = LlmClient::new(MockTransport::err(FetchError::Timeout));
        let (net, blob) = down
            .generate_shape_with_blob("a hexagon", 3, &mut cache)
            .expect("fallback");
        assert!(net.validate().is_ok(), "fallback network is valid");
        assert!(blob.is_none(), "fallback has no raw blob to surface");
    }

    /// THE gate (`vector_llm_timeout_graceful`): a timed-out fetch with a cached
    /// prior result returns that result — graceful degradation, no error, no hang.
    #[test]
    fn vector_llm_timeout_graceful() {
        let mut cache = ResultCache::new(8);
        // Seed the cache with a prior successful generation.
        {
            let warm = LlmClient::new(MockTransport::ok(HEXAGON));
            warm.generate_shape("a hexagon", 7, &mut cache)
                .expect("warm");
        }
        // Now the model times out — the client must fall back to the cached shape.
        let client = LlmClient::new(MockTransport::err(FetchError::Timeout));
        let net = client
            .generate_shape("a hexagon", 7, &mut cache)
            .expect("timeout falls back to the cached result");
        assert!(net.validate().is_ok(), "fallback network is valid");
    }

    #[test]
    fn timeout_without_cache_surfaces_the_error() {
        let client = LlmClient::new(MockTransport::err(FetchError::Timeout));
        let mut cache = ResultCache::new(8);
        let err = client
            .generate_shape("never fetched", 1, &mut cache)
            .unwrap_err();
        assert!(
            matches!(err, GenError::Fetch(FetchError::Timeout)),
            "no fallback → error"
        );
    }

    #[test]
    fn out_of_bounds_blob_is_rejected_not_materialised() {
        // A structurally-valid blob with 1e9 turns — the sanitizer (inside
        // build_from_json) must reject it; we surface Build, never a giant network.
        let evil = r#"{"shape_type":"spiral","params":{"turns":1000000000.0},"style":{}}"#;
        let client = LlmClient::new(MockTransport::ok(evil));
        let mut cache = ResultCache::new(8);
        let err = client
            .generate_shape("huge spiral", 0, &mut cache)
            .unwrap_err();
        assert!(
            matches!(err, GenError::Build(_)),
            "out-of-bounds blob rejected"
        );
        assert_eq!(cache.len(), 0, "rejected blob is never cached");
    }
}
