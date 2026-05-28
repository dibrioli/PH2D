# ADR-0061 — Vector LLM authoring (MCP tools + LLM4SVG semantic tokens + sanitizer)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [HR-10 — MCP first-class](../../SKILL_Stack_PH2D_Definitiva.md), [HR-11 — MCP destructive governance](../../SKILL_Stack_PH2D_Definitiva.md), [ADR-0056 — Vector Network](0056-vector-network-data-model.md), [ADR-0058 — Geometry graph](0058-vector-geometry-graph.md), [ADR-0047 — Painter MCP Stroke Engine](0047-painter-mcp-stroke-engine.md) (precedente).
**Spec normativa:** [`docs/Vector Module/09_scripting_mcp.md`](../../Vector%20Module/09_scripting_mcp.md) §9.5.
**Tags:** vector, wave-0, contract, mcp, llm, security

---

## 1. Contexto

Inovação #4: **LLM-as-graph-node** — primeiro tool vetorial onde LLM emite **strokes editáveis** (não SVG opaco dump). LLM4SVG semantic tokens preservam editability downstream do node; user edita slider em outro node e afeta output do LLM.

Antigravity 3ª iter findings absorvidos: L4F1 sanitizer (token injection) + L6F3 JSON Schema dinamic injection + L7F2 15s timeout fallback.

---

## 2. Decisão

### 2.1 Crate foundational `ph2d-vector-llm`

Consolidação Antigravity 2ª iter (combina MCP tools + node wrapper em single crate):

```
crates/ph2d-vector-llm/
├── src/
│   ├── lib.rs                       MCP tools + sanitizer + cache
│   ├── semantic_tokens.rs           LLM4SVG parser + JSON Schema validation
│   ├── sanitizer.rs                 Bounds enforcement pré-alocação (L4F1)
│   ├── mcp_tools.rs                 Tools + governance HR-11
│   ├── node_wrapper.rs              vector-llm-shape node impl
│   └── cache.rs                     Result cache by (prompt_hash, seed)
├── resources/
│   ├── schemas/
│   │   └── llm4svg-v1.json          JSON Schema versioned (L6F3 injection)
│   └── prompts/
│       └── vector_paint_shape_system.md  System prompt template
└── tests/
    └── security_token_injection.rs  Adversarial sanitizer fixtures
```

### 2.2 MCP tools canônicas

| Tool | Tipo | Função |
|------|------|--------|
| `vector_paint_shape(prompt, seed, style_ref)` | Mutative | LLM emite VectorNetwork via semantic tokens |
| `vector_modify_shape(shape_ref, mod_prompt)` | Mutative | LLM modifica shape existente |
| `vector_query_shape(shape_ref)` | Read-only | Retorna metadata + bbox |
| `vector_inspect_shape(shape_ref)` | Read-only | Retorna semantic tokens |
| `vector_delete_path(handle)` | **Destructive HR-11** | Confirmation token obrigatório |
| `vector_clear_scene(scene_id)` | **Destructive HR-11** | Confirmation token obrigatório |

### 2.3 LLM4SVG semantic tokens (NÃO SVG opaco)

LLM responde com **structured JSON** (não SVG dump):

```json
{
  "shape_type": "spiral",
  "params": {
    "turns": 8,
    "scale_ratio": 1.618,
    "center": [0, 0],
    "initial_radius": 100
  },
  "style": {
    "stroke_width": 2,
    "stroke_color_oklch": [0.5, 0.2, 30],
    "fill": "none"
  }
}
```

Parser converte → `vector-source.spiral` primitive node call + style application. Output é VectorNetwork standard editável downstream.

### 2.4 Token Injection Sanitizer (L4F1 Antigravity 3ª iter)

LLM output **NÃO confiável** (prompt injection OR adversarial response). Sanitizer obrigatório ANTES de alocar SmallVecs:

```rust
const MAX_SPIRAL_TURNS: u32 = 64;
const MAX_POLYGON_SIDES: u32 = 128;
const MAX_COORD: f32 = 1.0e6;
const MAX_VERTICES_PER_LLM_GEN: usize = 1000;

fn sanitize_semantic_tokens(tokens: &SemanticTokens) -> Result<(), SanitizerError> {
    match tokens.shape_type {
        ShapeType::Spiral => {
            let turns = tokens.params.get_u32("turns").unwrap_or(8);
            if turns > MAX_SPIRAL_TURNS {
                return Err(SanitizerError::ExceedsBound { param: "turns", limit: MAX_SPIRAL_TURNS, got: turns });
            }
        }
        // ...
    }
    for v in &tokens.params.get_vec2s("vertices") {
        if v.x.abs() > MAX_COORD || v.y.abs() > MAX_COORD || !v.x.is_finite() || !v.y.is_finite() {
            return Err(SanitizerError::CoordOutOfBounds { v: *v });
        }
    }
    let estimated_vertices = estimate_vertices_from_tokens(tokens);
    if estimated_vertices > MAX_VERTICES_PER_LLM_GEN {
        return Err(SanitizerError::TooManyVertices { estimated: estimated_vertices, limit: MAX_VERTICES_PER_LLM_GEN });
    }
    Ok(())
}
```

Gate `vector_fuzz_llm_semantic_tokens` (cargo-fuzz target) — 10k adversarial inputs daily CI.

### 2.5 JSON Schema dinamic injection (L6F3 Antigravity 3ª iter — long-tail)

LLM4SVG spec evolui (2024 paper → 2031 modelos diferentes). Schema versioned **injected dinamically** em MCP context:

- `resources/schemas/llm4svg-v1.json` — canonical schema.
- `resources/prompts/vector_paint_shape_system.md` — system prompt template com schema embedded.
- MCP server inject `system_prompt + JSON Schema` no LLM context per call.
- LLM responses validados contra schema antes de sanitizer.
- Schema versioned (`llm4svg-v1.json`, `llm4svg-v2.json`); MCP includes version hint.

Audit log salva schema version + LLM model id para reproducibility.

### 2.6 15s hard timeout + graceful fallback (L7F2 Antigravity 3ª iter)

Sem timeout, LLM API outage causa UI block infinito. Pipeline canônico:

```rust
let result = tokio::time::timeout(
    Duration::from_secs(15),
    mcp_client.call_paint_shape(prompt, seed, style_ref),
).await;

match result {
    Ok(Ok(tokens)) => {
        cache.insert(cache_key, tokens.clone());
        sanitize_semantic_tokens(&tokens)?;
        parse_to_network(tokens)
    }
    Ok(Err(e)) => fallback_to_cache_or_empty(cache_key, e),
    Err(_timeout) => {
        log::warn!("LLM timeout after 15s, using cache");
        ui_toast("LLM unavailable — using cached result");
        fallback_to_cache_or_empty(cache_key, Error::Timeout)
    }
}
```

Gate `vector_llm_timeout_graceful`.

### 2.7 HR-11 governance + audit log JSONL

Destructive tools (`vector_delete_path`, `vector_clear_scene`) exigem:
- `confirmation_token` no payload (single-use, valid 5 min).
- OR `--unsafe-mcp` flag (dev/CI mode).

Audit log JSONL append-only em `audit/vector.log` (rotated daily):

```json
{
  "timestamp": "2026-05-29T15:30:00.123Z",
  "session_id": "uuid-v4",
  "agent": "claude-opus-4-7@anthropic-mcp-client",
  "tool": "vector_paint_shape",
  "params": { "prompt": "spiral 8 arms", "seed": 42, "style_hint": "Geometric" },
  "state_hash_before": "blake3:abc123...",
  "state_hash_after": "blake3:def456...",
  "result": "ok",
  "result_summary": { "network_added": "uuid", "vertices": 8, "segments": 8 },
  "schema_version": "llm4svg-v1",
  "llm_model": "claude-opus-4-7"
}
```

### 2.8 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| MCP tools count | **6** (4 mutative + 2 destructive) | Padrão Painter ADR-0047 |
| LLM timeout | **15 seconds** hard | UI responsive |
| Sanitizer MAX_SPIRAL_TURNS | **64** | Adversarial cap |
| Sanitizer MAX_POLYGON_SIDES | **128** | Adversarial cap |
| Sanitizer MAX_VERTICES_PER_LLM_GEN | **1000** | Memory bound |
| Sanitizer MAX_COORD | **1.0e6** px | Adversarial cap (exponential rejection) |
| Audit log rotation | **daily** | Disk size manageable |
| Confirmation token TTL | **5 minutes** | Padrão HR-11 |
| Fuzz cases daily CI | **10_000** | Catch regressions |

---

## 3. Consequências

### 3.1 Positivas

- **Primeira ferramenta vetorial com LLM emitting editable strokes** — diferencial competitivo absoluto vs Inkscape AI plugin (output opaco).
- **Sanitizer bound-rigoroso** elimina OOM/DoS via prompt injection.
- **Schema injection** preserve future LLM compatibility (2031 modelos auto-adapt).
- **15s timeout + cache fallback** garante UI responsive em LLM outage.
- **HR-11 governance herdado** Painter ADR-0047 pattern.

### 3.2 Negativas

- **LLM external API dependency** — Anthropic/OpenAI/Gemini outage afeta feature. Cache fallback mitiga.
- **Sanitizer caps strict** podem rejeitar legitimate complex shapes; user re-prompt com conservative prompt.
- **Schema versioning** adds maintenance cost (each Vector Module release deve revisar `llm4svg-vN.json`).

### 3.3 Neutras

- LLM call latency 2-10s typical; UI spinner durante async.

---

## 4. Alternativas consideradas

### 4.1 LLM emits raw SVG (rejeitada — Inkscape AI plugin pattern)

Inkscape AI Generator (2026 plugin) emits raw SVG; group is opaque. **Por que rejeitada**: editability downstream lost; differencial Vector Module = editability preserved.

### 4.2 Sem sanitizer (rejeitada — L4F1 CRITICAL)

Trust LLM output direto. **Por que rejeitada**: adversarial response com `turns: 1000000` causa OOM. Sanitizer non-negotiable.

### 4.3 Embed ML model offline (rejeitada — V2.0 stretch)

SuperSVG ~50 MB binary embed. **Por que rejeitada**: v1.0 prioriza MCP external; embed model é V2.0 stretch para offline scenarios.

### 4.4 Hardcoded LLM4SVG spec (rejeitada — L6F3 long-tail)

Schema fixo em código. **Por que rejeitada**: 2031 LLMs precisam schema injected dinamically; auto-adaptation via JSON Schema validation.

---

## 5. Implementação (Wave 13)

- **T13.1**: `ph2d-vector-llm` skeleton + MCP tools + semantic tokens parser + JSON Schema.
- **T13.2**: `vector-llm-shape` node wrapper.
- **T13.3**: HR-11 governance + confirmation tokens.
- **T13.4**: Audit + sanitizer fuzz integration.
- **T13.5**: Fuzz cargo-fuzz target daily CI (L3F1 Antigravity 3ª iter).

Gates ativos: `vector_fuzz_llm_semantic_tokens` + `vector_llm_timeout_graceful` + `vector_mcp_governance_bypass_rejected`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/09_scripting_mcp.md §9.5`](../../Vector%20Module/09_scripting_mcp.md).
- LLM4SVG paper + project: <https://ximinng.github.io/LLM4SVGProject/>
- StarVector (arXiv 2312.11556): <https://arxiv.org/html/2312.11556v4>
- SuperSVG (arXiv 2406.09794): <https://arxiv.org/pdf/2406.09794>
- Painter MCP Stroke Engine ADR-0047 (pattern precedente).
- Inkscape AI SVG Generator extension (March 2026): <https://youvenz.github.io/blog/2026-03-05-ai-svg-generator-create-diagrams-in-inkscape-with-llms/>
