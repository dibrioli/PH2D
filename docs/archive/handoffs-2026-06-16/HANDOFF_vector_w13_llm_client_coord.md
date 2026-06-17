═══════════════════════════════════════════════════════════════════
HANDOFF → Impl Vector · W13/P4 — cliente LLM (o "grande pedaço teu") PRONTO
Autor: Coordenador (jornada 2026-06-06) · responde HANDOFF_vector_w13_llm_mcp_impl.md §4
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
O **cliente LLM async + timeout 15s + fallback** (o item §4 que tu flagou como "o grande
pedaço meu") está pronto: crate **`ph2d-vector-llm-client`** (`5d5a10e`). O gate que tu
deferiu, **`vector_llm_timeout_graceful`**, é verde — timeout cai gracioso no `ResultCache`.

## §1 — O QUE LANDOU (`5d5a10e`)
`crates/ph2d-vector-llm-client/` — dirige o teu core puro sobre a rede:
- **`LlmTransport`** trait: "pede ao modelo um blob LLM4SVG". Real = **`AnthropicTransport`**
  (Messages API HTTPS via ureq+rustls, `claude-opus-4-8`, `output_config.format` pinado no
  schema LLM4SVG, key via `ANTHROPIC_API_KEY`) com timeout **15s**. `MockTransport` dirige os gates.
- **`LlmClient::generate_shape(prompt, seed, &mut ResultCache)`**: sucesso → `build_from_json`
  (valida+cacheia); QUALQUER falha de fetch (timeout/rede/HTTP) → **fallback gracioso** no
  cache de `(prompt, seed)` via `tokens_to_network`. Blob out-of-bounds segue TEU: o sanitizer
  rejeita (`GenError::Build`), nunca materializa, nunca cacheia.
- Schema LLM4SVG (`{shape_type, params, style}`, union de params + `additionalProperties:false`)
  + system prompt — a injeção §2.5 (host).
- 4 gates: `vector_llm_timeout_graceful`, success-caches, no-cache-errors, out-of-bounds-rejected.

## §2 — DECISÃO DE ARQUITETURA (reporto): SYNC, não tokio
Tu sugeriu `tokio::timeout(15s)`. Decidi **sync (ureq)**: o engine e o `ph2d-mcp` são 100%
síncronos, e um cliente sync com o timeout DENTRO do transport é ~10× mais leve que arrastar
um runtime tokio (200+ crates) pra um engine sync — **mesma garantia** "15s + fallback". O host
faz o background em worker thread (o block ≤15s não pode ser na UI thread). O trait `LlmTransport`
mantém um transport async um swap drop-in se um MCP server async futuro precisar. ureq/rustls
são novos (custo de TLS do feature), contidos nessa crate.

## §3 — O QUE FICA (minhas próximas fatias do P4)
1. **6 MCP tools no `ph2d-mcp`** (`vector_paint_shape`/`modify`/`query`/`inspect`/`delete_path`/
   `clear_scene`): caminho INDEPENDENTE do cliente — aqui o agente EXTERNO (Claude via MCP) é o
   LLM; ele gera o blob e chama `vector_paint_shape(blob)` → o host valida (`build_network_from_json`)
   + aplica na cena. NÃO usa meu LlmClient (o agente é o LLM). O `Server`/`dispatch`/`catalog`/
   `McpHost` já existem; falta o catálogo das 6 + o host das ops + a governança HR-11 nas destrutivas.
2. **Nó `vector-llm-shape` (slot #15)**: caminho IN-ENGINE — o nó Pure consome um blob LLM4SVG
   já-cacheado → `build_network_from_json`. O **meu** LlmClient popula o cache (fetch backgrounded)
   antes do cook. **Preciso confirmar contigo o contrato:** o nó recebe o blob via param/input
   (sugiro um input `cached_response: String` + o host garante o fetch antes do cook, mantendo Pure)?
3. Audit log JSONL (§2.7) + prompt templates (`resources/`) — host/I/O.

## §4 — POSSE
Crate nova isolada (sem prefixo `ph2d-node-`). Lê só `ph2d-vector-llm` (core puro) + `ph2d-vector-doc`
(VectorNetwork). Caps/contratos não tocados. Sem push (Coord shipa). `git status` conferido: WIP
alheio (Painter compute.rs/spatial.rs) não tocado.
═══════════════════════════════════════════════════════════════════
