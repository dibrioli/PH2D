═══════════════════════════════════════════════════════════════════
HANDOFF → PRÓXIMO IMPLEMENTADOR · Vector W3 · T3.3 `ph2d-node-vector-boolean`
Autor: Implementador Vector (sessão W3 — pós-T3.2, pós-ship) · 2026-06-04
Baseline: **`98da9d3`** (origin/main == main, push mais recente do Coord). Sanity antes
de começar (DIRETRIZ §0): `git status` limpo + `cargo check --workspace` verde — a CI
do `98da9d3` pode estar em andamento (refactors em cima do `d81fc8c` já verde).
═══════════════════════════════════════════════════════════════════

## §0 — ⚠️ ANTES DE TUDO: git discipline (aviso crítico do Coord)
- **Branch/rebase a partir de `98da9d3`** (o Coord acabou de pushar; é a baseline viva).
  Trabalho baseado em commit anterior = conflito garantido.
- Commits **SCOPED**: `git add -- <só teus paths>`; `git commit --no-verify -m "msg" -- <paths>`.
  **NUNCA** `-A`/`-a`/`git add .`/`git stash`. `git status` antes de stage; `M`/`??` alheio → não comite, reporte.
- Fast mode (dia): commit local sem push. **Você NÃO pusha nem roda CI** — o Coord absorve PRCI no fim.
- `cargo fmt --all`/`-p` reformata WIP alheio → formate só os SEUS com `rustfmt <arquivos>`.

## §1 — Onde estamos (W3: T3.2 ✅, T3.1 ✅ — T3.3 é a tua)
| Task | Estado | Onde |
|---|---|---|
| **T3.2 `vector.source`** | ✅ shipado | `ph2d-node-vector-source` (5 primitives, snap Q16.16, registrado) |
| **Substrato carrier** | ✅ shipado | ADR-0058-amendment-1: `CookValue::Opaque(Arc<dyn Any>)` + glue `ph2d-vector-graph` (`VectorEvalExt::{emit_network,input_network}` + `VECTOR_PORT`) |
| **T3.1 panel + bridge** | ✅ shipado (Coord) | `ph2d-panel-vector-graph` (8 sliders) + `shells/desktop/src/render_loop/vector_graph_bridge.rs` (cook→render, flag `PH2D_VECTOR_GRAPH=1`) |
| **T3.3 boolean** | ⏳ **TUA PRÓXIMA ETAPA** | `ph2d-node-vector-boolean` (a criar) |

Smoke W3 já anda parcial: com `PH2D_VECTOR_GRAPH=1` o painel mostra os 8 sliders e
o `vector.source` renderiza live. Falta o boolean (Day 12 Linesweeper + Day 16 SDF).

## §2 — 🔥 NOVA DIRETRIZ PERMANENTE: máxima performance em tempo real
**Toda implementação daqui pra frente mira perf real-time.** Para T3.3 isso é literal —
é o coração da task (ADR-0065). Regras concretas:
- **Draft+reconcile (ADR-0059 §2.4 + ADR-0065):** hot-path (slider drag) usa **SDF GPU
  compute draft ≤ 0.5 ms** (silhueta); o **Linesweeper exato roda async off-thread**
  (debounced, on-commit/mouse-up) para topology canônica. NUNCA Linesweeper síncrono no
  hot-path (crítica C — inviável sub-9ms ProMotion).
- **Memoize o cook entre frames.** O `vector_graph_bridge` HOJE reconstrói registry+graph+`Cook`
  **todo frame** (`§3` do bridge admite: "Memoizing across frames é o W3 perf follow-up").
  Persistir `Cook`+`Graph`+`NodeRegistry` no estado do shell faz o cook memoizar (re-cook só
  no param edit). **Isso é parte da tua entrega** (sem isso, o slider real-time é fake).
- **Meça em `--release`.** `dev` = opt-level 0, ~7× mais lento — medir em dev mente
  (memória [project_painter_composite_perf_2026_06_03]). Instrumente, não chute.
- **Cache by hash:** `(input_a_hash, input_b_hash, op)` → result network, LRU 50 MB (ADR-0058 §2.6).

## §3 — O QUE FOI REFATORADO nesta jornada (file map — referências mudaram!)
Refactor "de-god-object" mecânico (move puro, round-trip verificado). **Se teu código
referenciava algum destes, o path mudou** (arquivo → diretório de submódulos):
- **Painter (subsistema FECHADO):** `tool.rs`→`tool/`, `stamp_scheduler.rs`→submódulos,
  `compositor.rs`→`compositor/`, `layers.rs`+`cpu_render.rs`→submódulos, `adjustments.rs`→submódulos,
  `bgremoval/chroma.rs`→submódulos.
- **render (perf-relevante p/ SDF GPU):** `sprite.rs`→`sprite/`, `layer_compositor.rs`→`layer_compositor/`.
  GPU adjustment kernels (filtros real-time) já vivem no layer compositor — **precedente de
  compute pass** útil pro teu `boolean_sdf.wgsl`.
- **editor-core:** `ids.rs`→`ids/` (por domínio), `hero.rs`→`hero/`, `state/mod.rs` WidgetStore→siblings,
  `dispatch/tests.rs`→`tests/`.
- **imageio-ora** `lib.rs`→import/export/blend; **ktx2** `lib.rs`→sibling submódulos.
- Fila de refactor restante + método: [`HANDOFF_refactor_map_coord.md`](HANDOFF_refactor_map_coord.md)
  (Grupo D `render_loop/mod.rs`+`input_dispatch` é **CONTENDED** — coordena antes de tocar).

## §4 — T3.3 spec (plano §6 + ADR-0065 + ADR-0059)
**Crate novo:** `crates/ph2d-node-vector-boolean` (drop-crate fan-out, DIRETRIZ §3.A).

**Contrato do nó:**
- **Inputs:** 2 portas geometria `a`, `b` (`VECTOR_PORT`). **Output:** 1 porta geometria.
  Lê com `VectorEvalExt::input_network(0/1)`, emite com `emit_network` (glue `ph2d-vector-graph`).
- **Effect::Stateful** (resultado cacheado por hash — ADR-0058 §2.2.2).
- **Param `op`** = discriminante f32 (mesmo padrão do `kind` do vector.source): 9 variants
  `union/subtract/intersect/exclude/divide/trim/merge/crop/outline` (paridade Pathfinder).
- Surface de dados **já existe**: `VectorOp::ApplyBoolean { op: BooleanOp, .. }` +
  `BooleanOp` em `ph2d-vector-doc/src/edit_log.rs` ("boolean engine ships W3" — é agora).

**Pipeline draft+reconcile (o núcleo):**
1. **SDF GPU draft** (`crates/ph2d-vector/shaders/boolean_sdf.wgsl` — **dir não existe, criar**):
   rasteriza VectorNetwork→SDF 2D (≤0.2ms) + boolean `min`/`max` compute (≤0.3ms) = **≤0.5ms**
   (ADR-0065 §2.5). Silhueta real-time pro slider. Resolution adaptive em zoom (§2.2).
2. **Linesweeper exato async** (debounced, on-commit): topology canônica editável. SDF é
   só silhueta (NÃO topology — ADR-0065 §2.3). Linesweeper é o output do `NodeOp::eval` (CPU).
3. **Fallback graceful** (ADR-0065 §2.6): sem compute shader → Linesweeper síncrono + warning UI.

**Critérios (DoD):** 9 variants corretas via Linesweeper; SDF draft visível durante slider drag;
commit refaz topology exata; golden cross-OS (snap Q16.16 + fixed SDF res, ADR-0065 §2.4).

**⚠️ Heavy lift:** o Linesweeper exato (edge cases: coincident edges / tangent contact / shared
vertices — onde Clipper falha, plano §6 T3.5 lente A) é a parte difícil. Decida cedo: hand-roll
(Bentley-Ottmann) vs. crate vetada (cheque HR-rules + `cargo machete`/`deny`). Reporte ao Coord
se precisar de dep nova (é decisão de stack).

## §5 — Integração (o que ligar pra fechar o smoke)
- **Estende o `vector_graph_bridge`** (`render_loop/vector_graph_bridge.rs`): hoje cozinha 1 nó
  `vector.source`. Pro smoke do boolean precisa de graph **multi-nó**: `source(a)`+`source(b)`+
  `boolean(a,b)` → render do resultado. Isso + a persistência do `Cook` (§2) andam juntos.
  `render_loop/mod.rs:1123-1125` chama o dispatch — **`render_loop/mod.rs` é CONTENDED (Grupo D)**,
  coordena com o Coord se precisar mexer no call-site (o painel/bridge novo provavelmente é chrome→Coord).
- **Panel:** os sliders extra (op dropdown, 2 inputs) são `ph2d-panel-vector-graph` (chrome → Coord-B,
  igual T3.1). Tu fazes o **nó + lógica + shader**; o Coord plumba painel/bridge se for chrome.
- **Follow-up herdado (Coord, não-bloqueante):** mover downcast `VectorDirectTool` de
  `render_loop/mod.rs` (~L1149) pra um bridge e tirar do `DOWNCAST_ALLOWLIST` (ship-fix `4054138`).

## §6 — Onboarding (ordem de leitura)
1. **CLAUDE.md** §0 (inegociáveis) + §5 (estado Vector) + §6 (contratos: Vector + Nodes).
2. **ADR-0058-amendment-1** (carrier opaco — como nó vetorial emite/consome `VectorNetwork`).
3. **ADR-0065** (SDF Hybrid GPU — o §2.5 budget + §2.6 fallback + §2.4 determinismo) + **ADR-0059** §2.4 (draft+reconcile).
4. Spec [`02_geometry_graph.md`](Vector%20Module/02_geometry_graph.md) §2.2.2 (boolean) — pseudocódigo
   é **ilustrativo**; a API real é a do `vector.source` (memória [project_vector_node_opaque_carrier]).
5. Código de referência: `ph2d-node-vector-source` (template de nó), `ph2d-vector-graph` (glue),
   `vector_graph_bridge.rs` (consumidor), `edit_log.rs` (`BooleanOp`/`ApplyBoolean`).
6. Precedente GPU compute: os adjustment kernels real-time em `ph2d-render/src/layer_compositor/`.

## §7 — Velocidade (inner loop)
- Slot CoW: `bash scripts/slot-seed.sh slot-impl-vector` → prefixe cada cargo com o
  `CARGO_TARGET_DIR` impresso. Inner loop = **`cargo check -p ph2d-node-vector-boolean`** só.
- Teste/clippy/auditoria 1× no fechamento do módulo (não por task). **Perf: meça em `--release`.**
- Golden + audit (T3.5, lente A boolean edge-cases / lente B SDF-vs-Linesweeper / lente C perf 100 paths)
  fecham o W3 depois de T3.3+T3.4.

## §8 — Caps congelados a respeitar (CLAUDE.md §6, gate `architecture_vector_contract_surface`)
`VectorOp≤16` / boolean **9 variants** (frozen) / cache LRU **50 MB** / `NodeOp=2`/`OpResolver=1`/
`NodeManifest=8` **intactos** (o carrier vive nos internos ungated — não bumpar). Gate
`vello_kurbo_only_in_ph2d_vector` ainda **W2-deferred** (cheque se já existe antes de assumir).
═══════════════════════════════════════════════════════════════════
