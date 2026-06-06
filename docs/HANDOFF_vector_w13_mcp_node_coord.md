═══════════════════════════════════════════════════════════════════
HANDOFF → Impl Vector + Shell/Host · W13/P4 — 6 MCP tools + vector.llm-shape node PRONTOS
Autor: Coordenador (jornada 2026-06-06) · continua HANDOFF_vector_w13_llm_client_coord.md §3
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
As duas fatias concretas do P4 que faltavam landaram (local, sem push):
1. **6 MCP tools `vector.*`** no `ph2d-mcp` (`c7a634e`) — o agente externo (Claude via MCP) é o LLM.
2. **Nó `vector.llm-shape` (slot #15)** crate `ph2d-node-vector-llm-shape` (`fe190fe`) — caminho in-engine.

O **contrato deferido** ("como a resposta chega ao nó") está **DECIDIDO + implementado**, fundamentado
no substrato real (não no pseudocódigo do spec). Falta só o **wiring do host** (editor/shell) — §3.

## §1 — 6 MCP tools (`c7a634e`, `crates/ph2d-mcp`)
`vector.paint_shape` / `vector.modify` / `vector.query` / `vector.inspect` / `vector.delete_path` /
`vector.clear_scene`. O **trust boundary é o dispatch**: toda blob passa por
`ph2d_vector_llm::build_network_from_json` (sanitizer bounds-antes-de-alocar) **no servidor** — o host
nunca recebe blob não-sanitizada. `delete_path`/`clear_scene` = destrutivas HR-11 (confirmation_token).
- Trait **`VectorSceneHost`** (`add_path`/`replace_path`/`list_paths`/`inspect_path`/`delete_path`/
  `clear_scene`) — o host (editor) implementa sobre o doc vetorial vivo; `MemoryVectorScene` é a ref.
- `Server::dispatch_vector(host, req)` (paralelo ao `dispatch` ECS; HR-11 + audit compartilhados via
  `precheck()`). Catálogo nas tools (bindgen regenerou `runtime/mcp/schema.json` + luau; paridade OK).
- 9 gates (incl. `adversarial_blob_rejected_not_materialized`, HR-11 token-gating, audit em destrutivas).
- **Falta (host):** implementar `VectorSceneHost` sobre a cena real do editor (hoje só `MemoryVectorScene`).

## §2 — Nó `vector.llm-shape` (`fe190fe`, `crates/ph2d-node-vector-llm-shape`)
`Effect::Pure` + `Clock::Static`, 1 output `VECTOR_PORT`, **1 param `seed` (f32)**, sem inputs.
`eval`: lê `seed` → busca a resposta cacheada → `build_network_from_json` → `emit_network`. Blob
ausente/rejeitada → `VectorNetwork::empty()` (total, nunca paniqueia). 5 cook-tests verdes; node-sync
regenerou registry-init (staleness gate verde).

### §2.1 — O CONTRATO DECIDIDO (e PORQUÊ, fundamentado no substrato)
O spec sugeria "input `cached_response: String`". **Não dá no substrato real:**
- params de nó são **`f32` only** (`ParamSpec{name,default:f32}`; `Graph::set_param(id,name,f32)`);
- **não há host-set graph input** — o cook só puxa de outputs de nós conectados (`Cook::cook(g,ops,target,t)`);
- ops são resolvidos **por tipo** (`OpResolver::resolve(NodeTypeId)->Option<&dyn NodeOp>`), stateless.

⇒ O **único seam** por onde dado per-instância não-`f32` chega a um op `Pure` é o **`OpResolver` do host**.
Por isso: `seed` (f32) identifica QUAL resposta; o host injeta o cache via op no seu próprio resolver.

### §2.2 — WIRING DO HOST (a tua próxima ação p/ ligar o nó)
```rust
// 1. Implementa a ponte ao ResultCache (populado off-thread pelo ph2d-vector-llm-client):
struct EditorLlmCache { /* Arc<...> ao ResultCache + um mapa seed→(prompt,seed) */ }
impl ph2d_node_vector_llm_shape::LlmResponseSource for EditorLlmCache {
    fn response_for(&self, seed: u64) -> Option<&str> { /* devolve o JSON cacheado p/ esse seed */ }
}
// 2. Resolver em camadas: vector.llm-shape → op com cache; resto → registry base.
struct HostResolver<'r> { base: &'r NodeRegistry, llm: LlmShapeOp<EditorLlmCache> }
impl OpResolver for HostResolver<'_> {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == ph2d_node_vector_llm_shape::MANIFEST.id { Some(&self.llm) } else { self.base.resolve(ty) }
    }
}
// 3. Antes de cozinhar: ph2d-vector-llm-client faz o fetch (15s+fallback) e popula o cache p/ o seed.
//    O nó registrado por node-sync usa NoCache (emite vazio) até esse wiring existir — seguro headless.
```
**Atribui um `seed` distinto por nó llm-shape** (o cache key). Resposta nova = seed novo ⇒ memoização do
cook continua correta (Pure preservado: output = f(seed, snapshot do cache no cook)).

## §3 — O QUE FICA
- **Host/Shell:** (a) `VectorSceneHost` sobre a cena real (liga as 6 MCP tools); (b) o `HostResolver` +
  `EditorLlmCache` acima (liga o nó); (c) disparar o `ph2d-vector-llm-client` off-thread p/ popular o cache.
- **Coord (meu, I/O — menor):** audit-log JSONL sink em disco (as linhas JSONL já saem de `Server::audit_lines()`;
  falta só o append-to-file do host) + templates de prompt em `resources/` (hoje o system-prompt + schema
  LLM4SVG vivem inline no `ph2d-vector-llm-client`). Marginal — não bloqueia nada.

## §4 — POSSE / GIT
Crates lidas read-only (contratos congelados: `NodeOp=2`/`NodeManifest=8`/caps/`VECTOR_PORT`/
`build_network_from_json`) — **nenhum tocado**. `ph2d-mcp` ganhou deps `ph2d-vector-{llm,doc}` (só o
dev-tool `ph2d-bindgen` depende de `ph2d-mcp` ⇒ não entra no build do engine/shell). Crate nova é glob
member (zero edit no Cargo.toml raiz). registry-init regenerado por node-sync (codegen determinístico).
Commits locais `c7a634e` + `fe190fe`, sem push (Coord shipa). `git status` conferido: WIP alheio
(Painter compute.rs/spatial.rs) não tocado.
═══════════════════════════════════════════════════════════════════
