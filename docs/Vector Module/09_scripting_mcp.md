# 09 — Scripting + MCP (Luau + LLM-as-graph-node)

> Spec de **scripting Luau** + **MCP toolset** + **LLM-as-graph-node** para Vector Module. Cada node + tool param exposto via `#[lua_export]` (HR-10). Custom modifiers em Luau (`vector-luau-script` node). MCP tools `vector_*` (read-only / mutative / destructive HR-11). LLM-driven authoring via `vector-llm-shape` (LLM4SVG semantic tokens, output editable).
>
> **ADR ratificador:** ADR-0061 (Vector LLM authoring + MCP tools + LLM4SVG).
> **Inovação #4 (vide [`14_inovacoes_extraordinarias.md §14.5`](14_inovacoes_extraordinarias.md)).**

## 9.1 Cada node `#[lua_export]` (HR-10)

### 9.1.1 Princípio

HR-10 (SKILL_Stack §9): toda API exposta a Luau é exposta a MCP. Se LLM não consegue fazer X, humano com Luau também não consegue. `ph2d-bindgen` gera schema MCP a partir das mesmas anotações `#[lua_export]`.

Para Vector Module:
- Cada node param exposto via `#[lua_export]`.
- Cada tool action exposto via `#[lua_export]`.
- Cada VectorOp exposto via `#[lua_export]`.

### 9.1.2 Exemplo

```rust
#[lua_export]
impl VectorBooleanNode {
    /// Apply boolean op to two paths.
    pub fn apply(&self, a: &VectorNetwork, b: &VectorNetwork) -> VectorNetwork {
        // ...
    }
    
    #[lua_export(setter)]
    pub fn set_op(&mut self, op: &str) -> Result<()> {
        // ...
    }
}
```

Bindgen gera:
- Luau `.d.luau` types: `function ph2d.vector.boolean.apply(a: VectorNetwork, b: VectorNetwork): VectorNetwork`.
- MCP schema: `{"name": "vector_boolean_apply", "input_schema": {...}, "output_schema": {...}}`.

### 9.1.3 Coverage gate

CI gate `bindgen_check`: cada `#[lua_export]` tem schema MCP correspondente. Falha se mismatch.

---

## 9.2 Custom modifier em Luau (`vector-luau-script` node)

### 9.2.1 Conceito

Node especial que permite custom modifier em Luau (vide [`02_geometry_graph.md §2.7`](02_geometry_graph.md)). Power user escreve modifier que não existe nos 17 canon nodes.

### 9.2.2 API Lua exposta

```lua
-- Available em modify():
network.vertices    -- array of {id, pos: vec2, kind}
network.segments    -- array of {id, start, end, tangents}
network.regions     -- array of {id, segments, winding}

-- Mutation
ph2d.vector.add_vertex(network, pos, kind)
ph2d.vector.move_vertex(network, id, new_pos)
ph2d.vector.add_segment(network, start, end, tangents)
ph2d.vector.remove_segment(network, id)
ph2d.vector.add_region(network, segment_ids, winding)

-- Sampling
ph2d.vector.compute_bbox(network)
ph2d.vector.sample_at(network, t)  -- t in [0..1]
ph2d.vector.tangent_at(network, t)
ph2d.vector.length(network)

-- Math helpers
ph2d.vec2(x, y)
ph2d.lerp(a, b, t)
```

### 9.2.3 Example script

```lua
function modify(network, params, dt)
    -- Custom modifier: rotate each vertex around centroid by N degrees
    local centroid = ph2d.vec2(0, 0)
    for _, v in ipairs(network.vertices) do
        centroid = centroid + v.pos
    end
    centroid = centroid / #network.vertices
    
    local angle = params.angle  -- degrees
    local rad = angle * math.pi / 180
    
    for _, v in ipairs(network.vertices) do
        local rel = v.pos - centroid
        local rotated = ph2d.vec2(
            rel.x * math.cos(rad) - rel.y * math.sin(rad),
            rel.x * math.sin(rad) + rel.y * math.cos(rad)
        )
        ph2d.vector.move_vertex(network, v.id, centroid + rotated)
    end
    
    return network
end
```

### 9.2.4 Sandbox (HR-8)

- Luau strict mode (ADR-0019).
- No `os.execute`, `io.*`.
- Timeout 5 segundos (kill se exceeds).
- Memory cap (Luau heap budget; HR-13).

### 9.2.5 Determinismo

Quando network deterministic, Luau runs em fixed-point mode (ADR-0019 §8 + HR-5 + HR-16). `pairs_sorted()` obrigatório vs `pairs()` em pipeline determinístico.

---

## 9.3 MCP tools `vector_*` (read-only / mutative / destructive)

### 9.3.1 Read-only tools

```
vector_query_network(handle): {vertex_count, segment_count, region_count, bbox}
vector_inspect_vertex(handle, vertex_id): {pos, kind, tangents}
vector_inspect_segment(handle, segment_id): {start, end, tangents, style}
vector_inspect_region(handle, region_id): {segments, winding, fill}
vector_query_node_graph(handle): {nodes: [...], connections: [...]}
vector_inspect_node_params(node_id): {params: [...]}
vector_query_assets(): {asset_paths: [...]}
```

### 9.3.2 Mutative tools (non-destructive)

```
vector_create_path(scene_id, path_spec): handle
vector_apply_node(network_handle, node_kind, params): network_handle (new)
vector_set_node_param(node_id, param_name, value): ()
vector_add_layer(scene_id, layer_kind): layer_id
vector_set_layer_fill(layer_id, fill_ref): ()
vector_set_layer_stroke(layer_id, stroke_ref): ()
```

### 9.3.3 Destructive tools (HR-11 governance)

```
vector_delete_path(handle): () [destructive=true]
vector_clear_scene(scene_id): () [destructive=true]
vector_flatten_boolean(node_id): () [destructive=true; baked result, sem undo]
vector_delete_layer(layer_id): () [destructive=true]
```

### 9.3.4 HR-11 governance

Toda destructive tool requer:
1. `confirmation_token` no payload (single-use, valid 5 min).
2. OR `--unsafe-mcp` flag (dev / CI mode only).

Audit log JSONL em `audit/vector.log`:
```json
{"timestamp": "2026-05-27T15:23:00Z", "agent": "claude-opus-4", "tool": "vector_delete_path", "params": {...}, "state_hash_before": "...", "state_hash_after": "..."}
```

### 9.3.5 Schema generated via `ph2d-bindgen`

```bash
cargo run -p ph2d-bindgen -- check
```

CI gate `bindgen_check_vector` verifica todos `#[lua_export]` em vector crates têm MCP schema.

---

## 9.4 HR-11 governance (confirmation tokens)

### 9.4.1 Token generation

UI gera token quando user clica button destructive:
1. User clica "Delete Path" no Inspector.
2. Modal "Confirm? [Cancel] [Delete]".
3. Click "Delete" → UI gera token random (UUID v4) + stores em `ph2d-mcp::tokens` com 5 min TTL.
4. Tool call passa token.
5. `Guard::validate(token)` checa válido + single-use.

### 9.4.2 Audit log

JSONL append-only em `audit/vector.log`. Rotated diário (`vector.log.2026-05-27`).

Campos:
- `timestamp` (ISO 8601 UTC).
- `agent` (MCP client identifier).
- `tool` (tool name).
- `params` (input JSON).
- `state_hash_before` (blake3 do scene state before op).
- `state_hash_after` (blake3 after op).
- `result` ("ok" | "error: ...").

### 9.4.3 LLM agent precedent

Mesma governance Painter MCP (ADR-0047). Consistent UX across modules.

---

## 9.5 LLM-as-graph-node (`vector-llm-shape`) — Inovação #4

### 9.5.1 Conceito

Node especial onde **LLM emite vector network estruturado editável** (não SVG opaque dump).

```rust
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.llm_shape"),
    name: "LLM Shape",
    inputs: &[Input::optional_image("style_ref")],
    outputs: &[Output::path("network")],
    effect: Effect::Stateful,  // result cached; re-prompt = re-bake
    params: &[
        Param::string("prompt", "spiral with 8 arms golden ratio"),
        Param::u64("seed", 42),
        Param::enum_var("style_hint", &[
            "Geometric", "Organic", "Hand-drawn", "Logo", "Letterform",
        ], "Geometric"),
    ],
};
```

### 9.5.2 Workflow

1. User adiciona `vector-llm-shape` node no Geometry Graph.
2. User types prompt: "spiral with 8 arms, golden ratio scaling".
3. Hit "Generate" → MCP call to LLM.
4. LLM responds em LLM4SVG semantic tokens.
5. Parser converte tokens → VectorNetwork.
6. Output usable downstream (slider em outro modifier afecta output!).
7. Re-prompt anytime → re-bake.

### 9.5.2-bis Token Injection Sanitizer (Antigravity 3ª iteração L4F1 2026-05-29)

LLM-emitted semantic tokens são **input não confiável** (LLM pode ser fooled via prompt injection OR responder adversarial). Sanitizer obrigatório ANTES de alocação:

```rust
fn sanitize_semantic_tokens(tokens: &SemanticTokens) -> Result<(), SanitizerError> {
    // Verifica caps explícitos antes de alocar SmallVecs
    if tokens.shape_type == ShapeType::Spiral {
        let turns = tokens.params.get_u32("turns").unwrap_or(8);
        if turns > MAX_SPIRAL_TURNS {  // const = 64
            return Err(SanitizerError::ExceedsBound { param: "turns", limit: MAX_SPIRAL_TURNS, got: turns });
        }
    }
    if tokens.shape_type == ShapeType::Polygon {
        let sides = tokens.params.get_u32("sides").unwrap_or(6);
        if sides > MAX_POLYGON_SIDES {  // const = 128
            return Err(SanitizerError::ExceedsBound { param: "sides", limit: MAX_POLYGON_SIDES, got: sides });
        }
    }
    // Coordinate bounds — reject exponential gigantesco
    for v in &tokens.params.get_vec2s("vertices") {
        if v.x.abs() > MAX_COORD || v.y.abs() > MAX_COORD || !v.x.is_finite() || !v.y.is_finite() {
            return Err(SanitizerError::CoordOutOfBounds { v: *v });
        }
    }
    // Total vertex count cap — reject brutalist requests
    let estimated_vertices = estimate_vertices_from_tokens(tokens);
    if estimated_vertices > MAX_VERTICES_PER_LLM_GEN {  // const = 1000
        return Err(SanitizerError::TooManyVertices { estimated: estimated_vertices, limit: MAX_VERTICES_PER_LLM_GEN });
    }
    Ok(())
}

const MAX_SPIRAL_TURNS: u32 = 64;
const MAX_POLYGON_SIDES: u32 = 128;
const MAX_COORD: f32 = 1.0e6;
const MAX_VERTICES_PER_LLM_GEN: usize = 1000;
```

Sanitizer rejeição → MCP tool returns `Error::Validation` com detalhe; LLM pode re-tentar com prompt mais conservador. Audit log registra rejected attempts. Gate CI `vector_fuzz_llm_semantic_tokens` (T13.5) valida sanitizer com 10k random adversarial inputs.

### 9.5.3 LLM4SVG semantic tokens

[ximinng.github.io/LLM4SVGProject](https://ximinng.github.io/LLM4SVGProject/) — LLM emits **structured tokens** (não raw SVG):

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
    "stroke_color": "#000000"
  }
}
```

Parser converts to `vector-source.spiral` primitive call + style application — **fully editable**.

### 9.5.4 Editability preserved

Output VectorNetwork é standard. User pode mover vertex, add modifier downstream, apply boolean. **Não é SVG opaco**.

Re-prompt re-bakes (new output replaces old), but downstream modifications em other nodes preserved (cached graph topology).

### 9.5.5 Example: prompt → editable

```
Prompt: "spiral with 8 arms golden ratio"
↓
LLM emits: {shape_type: "spiral", params: {turns: 8, scale_ratio: 1.618, ...}}
↓
vector-llm-shape node outputs VectorNetwork (8-turn spiral, golden-ratio scaling)
↓
Downstream: vector-roughen(amplitude=5)
   → spiral com roughen visible
↓
Downstream: vector-recolor(harmony="Complementary")
   → spiral colorida
↓
User edits "turns" param em LLM node OR re-prompts "now 12 arms"
   → re-bake LLM output; downstream modifiers re-apply automatically.
```

### 9.5.6 HR-11 governance

`vector_paint_shape` é **mutative** (não destructive). Sem confirmation token.

`vector_delete_llm_history` (clear all LLM-generated outputs) seria **destructive**.

### 9.5.7 Performance + timeout policy (revisado Antigravity 3ª iteração L7F2 2026-05-29)

- LLM call: 2-10 seconds typical (depende do model).
- Parse + bake: ≤ 100 ms.
- Async com spinner UI.
- **Hard timeout 15 seconds**: se LLM API não responde em 15s, MCP client aborta call + toast UI "LLM unavailable, using cached" + fallback graceful:
  1. Se node tem result cached da última call bem-sucedida (same prompt + seed) → use cache.
  2. Senão, node entra em "stale" state, output = last valid OR empty network; UI marca node visualmente com warning.
  3. User pode re-tentar manualmente OR adjust prompt.
- Sem spinners infinitos. Sem UI thread block. Memory `feedback_pipeline_inject_dont_cap` aplica.

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

Gate CI `vector_llm_timeout_graceful` simula LLM network failure, valida zero UI hang.

### 9.5.8 Cache

Output cached by (prompt_hash, seed). Re-prompt with same prompt + seed = use cache.

### 9.5.8-bis JSON Schema dinamic injection (Antigravity 3ª iteração L6F3 2026-05-29)

LLM4SVG spec atual baseia-se em paper acadêmico 2024. Em 2031, novos LLMs podem ignorar tokens hardcoded a menos que receivam **schema vivo** no contexto.

**Pipeline canônico**:
- `crates/ph2d-vector-llm/resources/schemas/llm4svg-v1.json` — JSON Schema completa dos semantic tokens aceitos pelo parser.
- `crates/ph2d-vector-llm/resources/prompts/vector_paint_shape_system.md` — system prompt para LLM com schema embedded.
- MCP server injeta `system_prompt + JSON Schema` no LLM context dinamicamente em cada `vector_paint_shape` call.
- LLM responses validados contra schema antes do sanitizer (§9.5.2-bis).
- Schema versioned (`llm4svg-v1.json`, `llm4svg-v2.json`); MCP includes version hint; LLM emite conforme version.

Benefits:
- Schema é canonical; código não tem hardcoded token names.
- Update schema = LLM auto-adapts.
- Audit log salva schema version used + LLM model id; reproducibility long-tail.

### 9.5.9 LLM backends

- Anthropic Claude (Opus 4.X, Sonnet 4.X) via MCP.
- OpenAI GPT-5 / GPT-4o via MCP.
- Gemini Pro via MCP.
- Local model embed opcional (stretch goal W13+ — SuperSVG embed).

Failover: se primary LLM unavailable, secondary fallback (configurável).

---

## 9.6 Editability preserved downstream

### 9.6.1 Princípio

LLM-generated VectorNetwork **vive como any other node output** — editable downstream sem restrictions:
- User pode adicionar `vector-roughen` downstream → afeta LLM output.
- User pode adicionar `vector-recolor` downstream → afeta LLM output colors.
- User pode mexer manually em vertex (via Direct Select) — overrides LLM output em commit.

### 9.6.2 Re-prompt sem perder downstream

Quando user re-prompts LLM node:
- LLM output muda.
- Downstream modifiers re-apply automatically (graph re-eval).
- Manual vertex edits (via Direct Select) overridos pela re-prompt — UI warns "manual edits will be lost; continue? [Yes] [Cancel]".

### 9.6.3 Diferencial vs current AI vector tools

| Tool | LLM output editável downstream? | Re-promptable? | Memory of session? |
|------|----------------------------------|----------------|---------------------|
| Inkscape AI SVG Generator (2026) | ❌ opaque group | ✓ (regen) | ❌ |
| StarVector / SuperSVG | partial | ❌ | ❌ |
| Adobe Firefly Vector | ❌ rasterizes on export | ✓ | ✓ project |
| **PH2D vector-llm-shape** | **✓ fully** | **✓ + cache** | **✓ via MCP session** |

---

## 9.7 Audit log format

JSONL append-only `audit/vector.log` (rotated daily).

```json
{
  "timestamp": "2026-05-27T15:30:00.123Z",
  "session_id": "uuid-v4",
  "agent": "claude-opus-4-1@anthropic-mcp-client",
  "tool": "vector_paint_shape",
  "params": {
    "prompt": "spiral 8 arms",
    "seed": 42,
    "style_hint": "Geometric"
  },
  "state_hash_before": "blake3:abc123...",
  "state_hash_after": "blake3:def456...",
  "result": "ok",
  "result_summary": {
    "network_added": "uuid-v4",
    "vertices": 8,
    "segments": 8,
    "regions": 1
  }
}
```

Used para:
- Debugging (replay LLM session).
- Audit compliance (who did what when).
- Training data (LLM4SVG model improvement).
- Determinism testing (replay deterministic LLM output if model versioned).

---

## 9.8 Examples canônicos (HR-17)

Per HR-17, examples Luau compilam em CI strict mode.

`docs/scripting/vector-examples/`:

```
docs/scripting/vector-examples/
├── 01_create_rect.luau
├── 02_boolean_union.luau
├── 03_apply_modifier.luau
├── 04_animate_param.luau
├── 05_custom_modifier.luau (vector-luau-script)
├── 06_llm_generate_shape.luau (MCP call)
├── 07_state_machine.luau (runtime)
└── 08_export_svg.luau
```

Cada example:
- Carrega em `luau-analyze --strict`.
- Roda em fixture sintético.
- CI gate `vector_examples_compile` falha na primeira falha.

---

## Fim do scripting + MCP spec

Luau + MCP + LLM-as-graph-node consistent com Painter pattern (ADR-0047). HR-10 / HR-11 enforced. Examples canônicos via HR-17.

**Next:** [`10_runtime_gameplay.md`](10_runtime_gameplay.md) (já criado) + [`11_pencil_pipeline.md`](11_pencil_pipeline.md).
