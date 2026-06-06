═══════════════════════════════════════════════════════════════════
HANDOFF → Shell/Host · W13/P4 wiring spec (ancorado nas APIs reais; pronto p/ aplicar)
Autor: Implementador Vector (jornada 2026-06-06) · responde HANDOFF_vector_w13_mcp_node_coord.md §2.2/§3
═══════════════════════════════════════════════════════════════════

## §0 — Estado verificado (na minha máquina, não "claimed")
Cadeia P4 toda verde: `ph2d-vector-llm` (core, 32) + `ph2d-node-vector-llm-shape` (nó, 5) +
`ph2d-mcp` (16 lib + 4 server + 2 schema). Os 3 seams que o host pluga **já existem e compilam**:
- **Client** (`ph2d-vector-llm-client`): `LlmClient::new(transport)` + `generate_shape(prompt, seed,
  &mut ResultCache) -> Result<VectorNetwork, GenError>` (fetch→validate→cache + fallback gracioso).
  System prompt + JSON Schema JÁ existem (`llm4svg_system_prompt()` / `llm4svg_schema()`) — o §3
  "templates de prompt" está, de fato, resolvido como funções.
- **Nó** (`ph2d-node-vector-llm-shape`): `LlmShapeOp::new(source: impl LlmResponseSource)`;
  `LlmResponseSource::response_for(&self, seed: u64) -> Option<&str>` (**JSON cru**).
- **MCP** (`ph2d-mcp::vector`): `VectorSceneHost` trait + `Server::dispatch_vector(host, req)`;
  `MemoryVectorScene` é a ref.

## §1 — A SUTILEZA que o pseudocódigo do coord-handoff não captura (corrige antes de codar)
`ResultCache` guarda **`SemanticTokens`** (não JSON cru) e o `LlmClient` retorna o `VectorNetwork`.
Mas o nó (`response_for`) quer **`&str` JSON cru**. ⇒ Há DOIS caminhos, escolha um:
- **(A) caminho-nó** (grafo-cêntrico): o `EditorLlmCache` precisa de um mapa **`seed → String (JSON cru)`**
  à parte, populado quando o client busca. `ResultCache` sozinho NÃO serve `response_for` (tem tokens, não JSON).
- **(B) caminho-direto** (mais simples): pular o nó; o editor chama `client.generate_shape(prompt, seed, &mut cache)`
  e injeta o `VectorNetwork` resultante direto no doc/cena. Use o nó só quando a geometria precisa ser
  re-cozida no grafo (downstream modifiers). Recomendo (B) p/ o MVP, (A) quando o grafo vetorial existir.

## §2 — Wiring concreto (caminho-nó, quando houver cook-site)
```rust
use std::collections::BTreeMap;
use ph2d_node_vector_llm_shape::{LlmResponseSource, LlmShapeOp, MANIFEST as LLM_MAN};
use ph2d_nodegraph::cook::{NodeOp, OpResolver};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_node_registry::NodeRegistry;

/// seed → JSON cru (populado off-thread pelo client; ver §3).
#[derive(Default)]
struct EditorLlmCache { by_seed: BTreeMap<u64, String> }
impl LlmResponseSource for EditorLlmCache {
    fn response_for(&self, seed: u64) -> Option<&str> { self.by_seed.get(&seed).map(String::as_str) }
}

/// Resolver em camadas: vector.llm-shape → op com cache; resto → registry base.
struct HostResolver<'r> { base: &'r NodeRegistry, llm: LlmShapeOp<EditorLlmCache> }
impl OpResolver for HostResolver<'_> {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == LLM_MAN.id { Some(&self.llm) } else { self.base.resolve(ty) }
    }
}
```
- **Seed distinto por nó llm-shape** (é o cache key; Pure preservado: output = f(seed, snapshot do cache no cook)).
- O nó registrado por node-sync usa `NoCache` (emite vazio) até este resolver existir — **seguro headless**.

## §3 — Driver off-thread (o client é SYNC-blocking de propósito)
`LlmClient` NÃO arrasta tokio (memória: ~10× mais leve). Roda numa **thread** dedicada:
```rust
// thread worker: recebe (prompt, seed) por channel, busca, devolve o JSON cru p/ a UI thread popular o EditorLlmCache.
let client = LlmClient::new(AnthropicTransport::from_env()?);
// em generate_shape o blob já passou por build_from_json (sanitizer) no client; p/ o caminho-nó
// guarde TAMBÉM o JSON cru (re-fetch do transport, ou exponha-o — ver nota §5) → by_seed.insert(seed, json).
```
Fallback de timeout (15s) já está DENTRO de `generate_shape` (cache hit on fetch-fail). UI: spinner durante.

## §4 — As 6 MCP tools (caminho do agente externo)
Implementar **`VectorSceneHost`** sobre a cena vetorial viva do editor (hoje só `MemoryVectorScene`):
`add_path`/`replace_path`/`list_paths`/`inspect_path`/`delete_path`/`clear_scene`. O trust boundary
(sanitizer + HR-11 + audit) JÁ está no `dispatch_vector` do servidor — o host só liga a cena real +
o sink do audit-log JSONL em disco (`Server::audit_lines()` já emite as linhas; falta o append-to-file).

## §5 — POSSE / por que isto é um doc e não um commit meu
O wiring vive em `ph2d-editor`/`ph2d-host` — **fora da posse Vector** (§0 regra 2). Além disso o
**cook-site do grafo vetorial ainda não existe no editor** (grep vazio), então isto é uma *feature de editor*,
não um "wiring" de drop-in. Entreguei o blueprint ancorado nas APIs reais; a implementação no shell precisa
de um dono Shell/Host (ou autorização explícita p/ eu vestir esse chapéu).
- **Nota p/ o Coord:** se o caminho-nó (A) for o alvo, considere expor o JSON cru no `ResultCache`/client
  (hoje ele guarda só `SemanticTokens`), senão o host re-serializa ou re-fetcha — pequeno ajuste de contrato teu.
═══════════════════════════════════════════════════════════════════
