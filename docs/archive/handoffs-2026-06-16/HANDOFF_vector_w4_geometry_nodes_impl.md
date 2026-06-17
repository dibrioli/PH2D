═══════════════════════════════════════════════════════════════════
HANDOFF → IMPLEMENTADOR Vector · W4 — fan-out de 12 geometry nodes
Autor: Coordenador (sessão 2026-06-04, pós ADR-0065 SDF fechado) · contexto separado
═══════════════════════════════════════════════════════════════════

## §0 — ⚠️ ANTES DE TUDO

**Baseline = HEAD LOCAL atual (NÃO `origin/main`).** Há **~27 commits locais não-pushados**
(SDF Phases 1-3 + GPU, Painter W4 §3, vector boolean). `origin/main` está MUITO atrás —
**não rebase pra origin, não reset**. Você ramifica do HEAD local; seus commits empilham;
o Coord faz ship 1×/jornada no fim. Sanity (DIRETRIZ §0): `git log --oneline -3` +
`git status` (working tree deve ter só docs/`.vscode` alheios — não toque) + 
`cargo check -p ph2d-node-vector-source` verde.

**Git discipline (crítico — multi-agente):**
- Commits SCOPED: `git add -- <só teus paths>` · `git commit --no-verify -m "msg" -- <paths>`.
  **NUNCA** `-A`/`-a`/`git add .`/`git stash`. `git status` antes de stage; `M`/`??` alheio → reporte, não comite.
- `cargo fmt --all`/`-p` reformata WIP alheio → formate só os SEUS com `rustfmt <arquivos>`.
- Você **NÃO pusha** nem roda CI. Fast mode: commit local sem push. Coord absorve PRCI.

## §1 — Onde estamos (W3 CORE LANDADO — NÃO refaça)

| Peça | Estado | Dono |
|---|---|---|
| `vector.source` (5 primitives) | ✅ shipado | impl Vector |
| `vector.boolean` (9 ops) | ✅ shipado (`9f21db1`) | impl Vector |
| Carrier opaco (`CookValue::Opaque(Arc<VectorNetwork>)` + glue `ph2d-vector-graph`) | ✅ shipado | substrato |
| **SDF draft+reconcile (ADR-0065)** | ✅ **FECHADO (Coord)** — smoke-OK 2026-06-04 | **Coord** |
| Geometry-graph smoke (`vector_graph_bridge`) | ✅ Coord-owned (shell plumbing) | **Coord** |

**⚠️ A SDF NÃO é mais tua task.** O handoff antigo (`HANDOFF_vector_w3_t33_boolean_impl.md`
§4) te mandava criar `crates/ph2d-vector/shaders/boolean_sdf.wgsl` — **isso foi superado.**
A SDF virou crate satélite **`ph2d-vector-sdf`** (CPU core + GPU compute + marching-squares)
e o draft+reconcile já está wireado no `vector_graph_bridge` pelo Coord (draft GPU durante
drag → reconcile exato no settle). **NÃO toque em `ph2d-vector-sdf` nem no bridge** — são do
Coord. Detalhe: [`HANDOFF_vector_sdf_phase3_coord.md`](HANDOFF_vector_sdf_phase3_coord.md).

**Estado real do W3 (§6 do plano):** T3.1 (panel) · T3.2 (source) · T3.3 (boolean+SDF) ·
**T3.4 (`vector.offset`)** = TODOS feitos. Falta só **T3.5 (audit + fechamento W3** — 3
lentes: edge-cases boolean / SDF-vs-Linesweeper / perf), que é **Coord-orchestrado** e
roda **em paralelo** ao teu W4 (deps de W4 = T3.1 + T0.3, já satisfeitas — não te bloqueia).
Não faça o T3.5; é meu. Se tua implementação de W4 expuser um bug de boolean/offset/SDF,
reporta ao Coord (entra na lente do audit).

## §2 — TUA TASK: W4 (plano §7) — 12 geometry nodes, drop-crate fan-out (A)

Caminho **(A) DIRETRIZ §3.A** — cada nó é um crate isolado, zero edit central, wiring gerado.
Os 12 (`ph2d-node-vector-<slug>`):

`outline-stroke` · `roughen` · `twist` · `bend-path` · `pattern-along-path` · `scatter` ·
`width-profile` · `hatch` · `mirror` · `corner-round` · `warp` · `recolor`

**Ordem recomendada (smokes-intra do plano):** Day 5 = **roughen / mirror / corner-round**
(os 3 mais diretos pra primeiro pixel) → Day 10 += twist / scatter / hatch → Day 15 = 12.

### Contrato de cada nó (segue o template `ph2d-node-vector-source`)
- **Input/Output:** quase todos **unários** — 1 porta geometria in (`VECTOR_PORT`) + params →
  1 porta geometria out. Lê com `VectorEvalExt::input_network(0)`, emite com `emit_network`
  (glue `ph2d-vector-graph`). **Exceção:** `pattern-along-path` é binário (pattern + path) e
  reusa `ph2d-painter-brush` no W8 — faça uma versão básica OU reporte pro Coord se quiser adiar.
- **🔥 Effect = `Pure`** (NÃO `Stateful`). O spec/handoffs antigos dizem Stateful, mas nó
  consumido pelo renderer DEVE ser Pure — Stateful é push-side, nunca dirigido pelo Cook → smoke
  morto. O Cook memo já é o "cache by (input,params)". (memória [project_node_effect_pure_for_renderer_consumed].)
- **Params:** lidos via `ctx.param("nome")` no eval (NUNCA `MANIFEST.params[..].default`).
  Alocação capada via `param_as_count(v, MAX)` (ex.: `mirror` count, `scatter` count, `roughen` subdiv).
- **Golden test** bit-idêntico cross-OS: snap Q16.16 + ordem de varredura fixa + seed fixo
  (qualquer ruído — roughen/scatter — usa PRNG determinístico semeado por param, não `rand` global).

### Intenção por nó (detalhe em `Vector Module/02_geometry_graph.md` + `08_*`)
- **roughen** — subdivide arestas + jitter por ruído (amplitude, detail/freq, seed).
- **mirror** — reflete em X / Y / ambos → 2 ou 4 cópias unidas (smoke: Quad → 4 cópias).
- **corner-round** — fillet de cantos agudos por raio (insere arcos/béziers).
- **twist** — rotaciona vértices por ângulo ∝ distância do centro.
- **scatter** — N cópias do input em posições PRNG-determinísticas numa área (count capado).
- **hatch** — preenche região com linhas paralelas (ângulo, espaçamento).
- **outline-stroke** — expande path → região preenchida (offset ±width/2).
- **bend-path** — deforma Y por função de X (arco/ângulo).
- **width-profile** — largura de stroke variável ao longo do path (curva de perfil).
- **warp** — deforma via lattice/envelope.
- **recolor** — transforma `StyleTable` refs (cor de fill/stroke); geometria intacta.

## §3 — Validação por nó (DoD plano §7)
1. `cargo check -p ph2d-node-vector-<slug>` verde (inner loop — só isso por task).
2. `cargo run -p ph2d-node-sync` regenera `register_all_nodes` (diff esperado, NÃO viola isolamento).
3. `cargo test -p ph2d-node-registry-init` (staleness) verde.
4. Golden test do nó verde (bit-idêntico).
5. **"Aparece no panel UI"** (critério 5 do plano): hoje **NÃO há editor de grafo livre** —
   o smoke é hardcoded (`source+source+boolean`). Adicionar nó arbitrário ao grafo é **chrome
   (Coord)**. Pra TI: entregue crate + golden + sync (1-4); pra ver na tela, **reporte ao Coord**
   que plumba uma wiring de smoke (ele estende o bridge). Não toque no bridge você mesmo.

## §4 — O QUE VOCÊ NÃO TOCA (anti-colisão)
- `crates/ph2d-vector-sdf/` + `shells/desktop/src/render_loop/vector_graph_bridge.rs` — **Coord** (SDF).
- `render_loop/mod.rs` + `input_dispatch` — **CONTENDED Grupo D**; coordene antes.
- `ph2d-nodegraph` / `ph2d-expr` / `ph2d-node-registry` / `ph2d-node-registry-init/` (GERADO) —
  contrato congelado (ADR-0039). `ph2d-vector` API pública / `Camera2d` / shells plumbing — foundational.
- Qualquer arquivo fora do teu `crates/ph2d-node-vector-<slug>/`. Precisou? PARE e reporte ao Coord.

## §5 — Caps congelados (gate `architecture_vector_contract_surface` + `architecture_contract_surface`)
`NodeOp=2` / `OpResolver=1` / `NodeManifest=8` **intactos** (carrier vive nos internos ungated —
não bumpar). `VectorOp≤16` (os geometry nodes são nós de grafo, não `VectorOp` novos — não mexem
nesse cap). `MAX_VERTICES_PER_LLM_GEN=1000`. Sem cap-bust ad-hoc; mudança de contrato = Coord + ADR.

## §6 — Onboarding (ordem de leitura)
1. **CLAUDE.md** §0 (inegociáveis) + §6 (contratos Nodes + Vector).
2. **DIRETRIZ §3.A** (receita drop-crate node) + `examples-fan-out.md` (`ph2d-node-shader-blur` end-to-end).
3. **ADR-0058-amendment-1** (carrier opaco — como nó vetorial emite/consome `VectorNetwork`).
4. Código de referência: **`ph2d-node-vector-source`** (template canônico de nó vetorial) +
   `ph2d-vector-graph` (glue `VectorEvalExt`/`VECTOR_PORT`) + `ph2d-node-vector-boolean` (binário, 9 ops).
5. Spec `Vector Module/02_geometry_graph.md` — pseudocódigo é **ilustrativo**; a API real é a do
   `vector.source` (memória [project_vector_node_opaque_carrier]).

## §7 — Velocidade (inner loop) + perf
- Slot CoW: `bash scripts/slot-seed.sh slot-impl-vector` → prefixe cada cargo com o
  `CARGO_TARGET_DIR` impresso. Inner loop = **`cargo check -p ph2d-node-vector-<slug>`** só.
- Teste/clippy/auditoria 1× no fechamento do módulo (não por task). **Meça perf em `--release`**
  (dev = opt0, ~7× mais lento — mente). Nós pesados (warp/scatter/hatch) capam alocação via `param_as_count`.
- T4.13 audit fecha W4 (corretude per-node · perf de grafo 6-nós encadeados · consistency panel+render+edit_log).

QUANDO TERMINAR cada nó (ou batch), reporte ao Coord:
  "vector-<slug> pronto. Commit local <sha>. cargo test -p ph2d-node-vector-<slug> +
   -p ph2d-node-registry-init verdes. Precisa wiring de smoke no bridge? (Coord plumba)."
═══════════════════════════════════════════════════════════════════
