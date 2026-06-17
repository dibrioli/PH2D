═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Vector W4 — fan-out de geometry nodes (11/12 PRONTOS)
Autor: Implementador Vector (sessão W4) · 2026-06-04 · baseline = HEAD local
Commits locais (não pushados): `03c28b5` `30fc1d8` `07a81e4` `70fe276` `c1e4b5e` `4db8408`
═══════════════════════════════════════════════════════════════════

## §1 — ENTREGUE (verde, isolado, drop-crate fan-out A)

**11 dos 12 geometry nodes** — cada um crate `ph2d-node-vector-<slug>` isolado, `Effect::Pure`,
param via `ctx.param`, alocação capada via `param_as_count`, golden bit-idêntico (snap Q16.16 +
PRNG inteiro semeado p/ ruído), cook-path test e2e. Registrados via `ph2d-node-sync` (staleness
verde, **19 node crates**). Caps congelados intactos (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`,
`VectorOp≤16` — geometry nodes não criam VectorOp).

| Nó | Resumo | Notas v1 |
|---|---|---|
| **mirror** | reflete X/Y/Both + combina cópias | combine (não boolean-union) |
| **twist** | rotaciona vértices ∝ dist do centro | trig f64, radius-preserving |
| **roughen** | subdivide + jitter na normal | PRNG inteiro (seed,seg_id,sample); shared edges consistentes |
| **corner-round** | fillet de cantos por raio | v1 polígono (curvas polygonizadas) |
| **bend-path** | bend em arco circular por x | trig f64 |
| **scatter** | N cópias PRNG numa área | count capado |
| **recolor** | reassign fill ref (geom intacta) | carrier só tem refs, cor é asset-side |
| **outline-stroke** | stroke→região via kurbo bridge | open-chains por adjacência + region loops |
| **hatch** | linhas paralelas (scanline) | emite open segments; região=polígono v1 |
| **warp** | envelope sine-wave Y | v1 onda; lattice/mesh = follow-up |
| **width-profile** | taper linear → band preenchida | v1 linear; profile-curve = follow-up |

**~80 testes** (unit + cook + golden) + **gates** todos verdes: clippy `--all-targets`, machete
zero unused, **rustfmt pinned 1.95**, contract gate, staleness.

## §2 — T4.13 audit (perf + chain-consistency) ✅ parcial

Crate `ph2d-vector-fanout-audit` (**NÃO é nó** — fora do glob `ph2d-node-*`, node-sync ignora):
cozinha `source(poly)→corner-round→mirror→twist→bend→warp` pela **registry real**.
- Correção: chain cozinha p/ network válido + determinístico + reproduzível e2e.
- **Perf (`--release`):** cold cook **0.054 ms** (56v/56s/4r); re-cook memoizado **0.001 ms**
  (Cook memo = "cache by (input,params)" provado no chain). Folga enorme vs frame budget.
- `cargo run --release -p ph2d-vector-fanout-audit --example chain_perf`.

Falta da lente de audit (Coord-orquestrável, como T3.5): consistency **panel+render+edit_log**
(precisa do panel/bridge chrome) e per-node visual. Per-node correctness já coberto por testes.

## §3 — DEFERIDO p/ Coord (handoff §2 sancionou)

- **`pattern-along-path`** (o 12º): é **binário** (pattern + path) e reusa `ph2d-painter-brush`
  no W8. Fora do escopo unário limpo + dep cross-módulo. **Pré-req: decidir API com o owner do
  painter-brush.** Não comecei (isolamento).

## §4 — Pra VER no app (chrome → Coord)

Não há editor de grafo livre; o smoke é hardcoded (`source+source+boolean` no
`vector_graph_bridge`). Pra qualquer um destes 11 nós aparecer na tela, o **bridge precisa de
wiring de smoke** (estender o grafo cozido) — é teu (não toquei `vector_graph_bridge` nem
`render_loop`). Sugiro um smoke `source→<transform>→render` atrás de flag pra cada nó, ou um
mini-editor. Geometria já validada por testes + o harness de audit.

## §5 — Gotcha registrado (memória)

Crate helper na área de nós **NÃO pode** começar com `ph2d-node-` — o `ph2d-node-sync` registra
todo `crates/ph2d-node-*` (exceto registry/registry-init) e geraria um `::register` inexistente,
quebrando `registry-init`. Por isso o audit é `ph2d-vector-fanout-audit`, não `ph2d-node-*`.
═══════════════════════════════════════════════════════════════════

## RESPOSTA DO COORDENADOR (2026-06-04)

- **§4 smoke wiring — DONE (`f0ca76d`).** `PH2D_VECTOR_NODE=<slug>` cozinha
  `source(sliders) → vector.<slug> → render` (auto-framed, exato). Slugs: `roughen`,
  `mirror`, `twist`, `corner-round`, `bend-path`, `scatter`, `recolor`, `outline-stroke`,
  `hatch`, `warp`, `width-profile`. Via `register_all_nodes` (dep nova `ph2d-node-registry-init`
  no shell) — qualquer type-id resolve sem dep per-node. Assinatura `dispatch` intacta,
  `render_loop/mod.rs` não tocado. Shell compila limpo (os 19 node crates registram). **Aguarda
  smoke visual do Enio.**
- **§3 `pattern-along-path` — DEFERIDO p/ W8 (correto).** É o item da tabela de smokes do W8
  ("Pattern Along Path + Painter brush reuse"), não gap do W4. Binário + dep cross-módulo em
  `ph2d-painter-brush` → API decidida com o owner do painter-brush quando W8 abrir. Não-bloqueante.
- **§2 T4.13 audit — desbloqueado.** Per-node correctness ✓ (teus ~80 testes), perf ✓ (0.054ms
  cold / 0.001ms memo no chain harness). A lente visual/consistency agora roda (smoke §4). Fecho
  o T4.13 (doc, espelho do T3.5) assim que o Enio confirmar visual de alguns nós.
═══════════════════════════════════════════════════════════════════
