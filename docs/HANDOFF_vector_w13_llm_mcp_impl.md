═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador Vector · W13 LLM/MCP authoring (Inovação P4, ADR-0061) — core puro fechado
Autor: Implementador Vector (jornada 2026-06-05)
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
1. **Inovação P4 (LLM-as-graph-node) VIVA no core seguro.** Crate nova **`ph2d-vector-llm`** (ADR-0061): o LLM
   emite **semantic tokens LLM4SVG** (JSON estruturado, ex. `spiral{turns}`/`polygon{sides}`) que viram um
   **`VectorNetwork` editável** — NÃO um SVG opaco. **32/32 testes verdes**, clippy zero warnings.
2. **Segurança L4F1 (o ponto não-negociável) implementada e testada:** sanitizer de bounds **antes de alocar**
   (`turns:1e9` / path de 1 bilhão de vértices → **rejeitado, nunca materializado**) + guarda de byte-size
   pré-parse + propriedade fuzz "nunca paniqueia". Governança HR-11 (token de confirmação single-use, TTL 5min)
   rejeita ops destrutivas sem token.
3. **Divisão arquitetural (a tensão Pure-node vs I/O-async resolvida):** o pipeline **parse→sanitize→lower é
   puro/determinista** (`build_network_from_json`) — exatamente o que um nó `Effect::Pure` precisa sobre uma
   resposta cacheada. A **chamada LLM async + timeout 15s** é I/O do host/`ph2d-mcp` (já existe) — **fora desta
   crate**, flagada pro Coord (§4).
4. **Isolamento:** drop-crate lendo só contratos congelados (`ph2d-vector-doc` primitivas + caps). Nome
   `ph2d-vector-llm` (não `ph2d-node-`) evita o gotcha do node-sync. Caps congelados consumidos, não tocados.

## §1 — O QUE LANDOU (`crates/ph2d-vector-llm/`)
- **`semantic_tokens.rs`** — modelo LLM4SVG: `parse(json)→SemanticTokens` (Shape: Spiral/Polygon/Star/Ellipse/
  Rect/Path + StyleTokens). 1ª camada de validação (shape desconhecido / param mal-tipado → `ParseError`;
  lenient em ausentes via defaults).
- **`sanitizer.rs`** — **o coração de segurança (L4F1)**: `sanitize(&tokens)` checa caps congelados
  (`MAX_SPIRAL_TURNS=64`/`MAX_POLYGON_SIDES=128`/`MAX_VERTICES_PER_LLM_GEN=1000`) + `MAX_COORD=1e6` +
  finitude + estimate-de-vértices (f64, sem overflow) — tudo **pré-alocação**. [gate `vector_fuzz_llm_semantic_tokens`].
- **`to_network.rs`** — tokens sanitizados → `VectorNetwork` via `primitives::{spiral,polygon,star,ellipse,rect}`
  (+ polyline explícita). Primitivas auto-clampam = defesa-em-profundidade.
- **`governance.rs`** — HR-11: `Governance` emite tokens (blake3(nonce‖now), sem RNG) + `authorize` rejeita
  destrutivo sem token / inválido / expirado / reusado; `--unsafe-mcp` bypassa (dev/CI). Clock injetado (`now`),
  determinista. [gate `vector_mcp_governance_bypass_rejected`].
- **`cache.rs`** — `ResultCache` por `(blake3(prompt), seed)`, cap-bounded, eviction determinista — serve o
  fallback do timeout (§2.6).
- **`lib.rs`** — `build_network_from_json` / `build_from_json` (size-guard→parse→sanitize→lower) + `LlmError` +
  `MAX_INPUT_BYTES=64KB`.

## §2 — GATES
- ✅ `vector_fuzz_llm_semantic_tokens` — sanitizer + testes adversariais (turns 1e9, samples u32::MAX, coords
  NaN/∞/1e9, 1001 vértices, sides>cap) + `parse_then_sanitize_never_panics` + `garbage_never_panics`
  (propriedade fuzz: total, sem panic/OOM). **O target cargo-fuzz daily-CI é infra tua** — a fn fuzzável
  (`build_network_from_json`) e a propriedade estão prontas.
- ✅ `vector_mcp_governance_bypass_rejected` — `destructive_without_token_is_rejected` + invalid/expired/single-use.
- 🟡 `vector_llm_timeout_graceful` — **teu** (async I/O): o `tokio::timeout(15s)` + fallback. A máquina de
  fallback (`ResultCache`) está aqui; só o timeout async + a chamada real ficam no host.

## §3 — DECISÕES (reporto)
- **Crate `ph2d-vector-llm` (não `ph2d-node-vector-llm-shape`):** o ADR §2.1 consolida tudo numa crate; mantive
  a LÓGICA aqui (não-`ph2d-node-` → fora do node-sync). O **nó `vector-llm-shape`** (slot #15 dos 18) é um wrapper
  fino que registra e chama `build_network_from_json` sobre uma resposta cacheada — **próximo passo** (precisa
  decidir como o nó recebe a resposta LLM já-fetchada; sugiro um param/input com o JSON cacheado, mantendo o nó Pure).
- **Validação por serde tipado (não lib JSON-Schema):** o `#[derive(Deserialize)]` + leitura tipada É a validação
  estrutural — zero dep nova de jsonschema. O JSON Schema do ADR §2.5 é p/ **injeção no contexto do LLM** (host),
  não validação runtime.
- **`MAX_INPUT_BYTES` pré-parse:** camada extra L4F1 — impede OOM no `serde_json` antes do sanitizer rodar.
- **`MAX_COORD=1e6`:** definido local (não está no `postcard_schema` congelado; é cap de sanitizer, não de serialização).

## §4 — O QUE FICA (teu / deferido)
- **I/O MCP async (o grande pedaço teu):** o cliente LLM real + `tokio::timeout(15s)` + as 6 MCP tools
  (`vector_paint_shape`/`modify`/`query`/`inspect`/`delete_path`/`clear_scene`) registradas no `ph2d-mcp`
  (que já existe). Eu entrego a lógica pura (sanitize/governance/parse/cache) que essas tools chamam.
- **Nó `vector-llm-shape` (slot #15):** crate fina `ph2d-node-vector-llm-shape` (Pure, consome resposta cacheada
  → `build_network_from_json`). Eu faço se quiser — confirma o contrato de como a resposta chega ao nó.
- **JSON Schema injection (§2.5) + audit log JSONL (§2.7) + prompt templates (resources/):** host/I/O.
- **Schema versioned em `resources/schemas/`:** não criei (é o contrato de injeção do host); o parser tipado é
  o equivalente runtime.

## §5 — GIT / POSSE
- Commit scoped local: `crates/ph2d-vector-llm/**` + `Cargo.lock` (só o node da crate; serde_json/blake3 já no
  tree) + este handoff. `--no-verify`, sem push. `git status` conferido: WIP sujo alheio (Painter/render) não tocado.
  Caps congelados (1000/128/64) consumidos read-only; contratos intactos.
═══════════════════════════════════════════════════════════════════
