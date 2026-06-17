═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Vector W3 · T3.3 `vector.boolean` — integração + ratificação
Autor: Implementador Vector (sessão T3.3) · 2026-06-04 · baseline `98da9d3`
Commit local: **`42422d1`** (não pushado — fast mode). 6 arquivos, +878 linhas.
═══════════════════════════════════════════════════════════════════

## §1 — O QUE ESTÁ PRONTO (verde, commitado `42422d1`)

**Crate novo `crates/ph2d-node-vector-boolean`** — o nó + o **motor exato** (a metade
*reconcile* do draft+reconcile, ADR-0059 §2.4 / ADR-0065). Drop-crate fan-out, isolado.
- `engine.rs`: `VectorNetwork ⇄ kurbo::BezPath ⇄ linesweeper::binary_op ⇄ VectorNetwork`,
  mesma convenção de control-points do renderer; **9 variants** `BooleanOp` mapeadas;
  snap Q16.16 + flag `deterministic` cascateada (cross-OS golden, ADR-0065 §2.4).
- `lib.rs`: `MANIFEST` (`vector.boolean`, 2 inputs `a`/`b` + 1 output, param `op` f32
  discriminante 0..=8), `NodeOp`, `register`. Registrado via `ph2d-node-sync` (gate de
  staleness verde).
- **15 testes** passam (geometria por bbox: Intersect=overlap, Subtract=A∖B, Divide=3 faces;
  9-variants válidas; determinismo reproduz byte-a-byte; cook path e2e). Contract gate
  `architecture_vector_contract_surface` verde (nenhum cap congelado bumpado). clippy
  `--all-targets` limpo. `forbid(unsafe_code)`.

## §2 — DECISÕES PARA RATIFICAR (divergem do spec/handoff — rationale forte)

1. **Dep nova: `linesweeper` 0.3.0** (decisão de stack — reportando, não renegociei).
   - É o motor que o spec **nomeia canonicamente** (`16_referencias.md`: jneem/linesweeper).
     MIT/Apache (deny-allowed). kurbo-native — usa **kurbo 0.13, já no lock via `peniko`**.
     Footprint: só +`linesweeper`+`polycool` (deps de polycool — arbitrary/arrayvec/libm — já
     in-tree). `deny` multiple-versions=`warn` (kurbo 0.13 do engine vs 0.12 do vello 0.8 —
     skew contido: o nó não vaza tipos kurbo, só os usa internamente p/ falar com o engine).
   - ⚠️ **CAVEAT**: linesweeper se auto-descreve "early beta / minimal maintenance". T3.5
     lente A (coincident edges / tangent contact / shared vertices) é o stress real — pode
     achar bugs no upstream. Mitigação se necessário: fuzz + (pior caso) vendor/patch.
     **Se o Enio quiser hand-roll Bentley-Ottmann em vez do crate, é semanas de trabalho +
     risco de robustez numérica — recomendo fortemente o crate.**

2. **`Effect::Pure`, NÃO `Effect::Stateful`** (spec §2.2.2 e handoff §4 dizem Stateful).
   - Neste substrato `Stateful` = "escreve SimWorld, push-side" e **nunca é dirigido pelo
     Cook de apresentação** (a membrana, ADR-0030 — vide `effect.rs`/`cook.rs`). Um boolean
     Stateful seria **invisível ao renderer** → o smoke `source→boolean→render` morto
     (memória `feedback_tool_unit_green_integration_dead`).
   - O Cook **já memoiza** um nó Pure por `(input revisions + param hash)` — isso É o
     "cache by (input_a_hash, input_b_hash, op)" do ADR-0058 §2.2.2. O pseudocódigo do spec é
     aspiracional (memória `project_vector_node_opaque_carrier`). **Decisão substrate-correta.**

3. **Semântica dos 9 ops** (4 diretos via linesweeper; 5 compostos/simplificados):
   - Union/Subtract→Difference/Intersect/Exclude→Xor: **diretos, exatos**.
   - Merge≡Union, Crop≡Intersection: exatos p/ o carrier single-fill do W3 (a distinção
     Pathfinder é cor/bias de operando, não geometria — documentado).
   - Divide = `{A∖B} ∪ {A∩B} ∪ {B∖A}` (3 faces separadas). Trim = `{A∖B} ∪ {B∖A}` (peças
     abutidas, sem merge — vs Exclude que funde em 1 face even-odd).
   - Outline = geometria da Union (boundary). True width-expansion stroke-outlining é
     `vector-outline-stroke` (T3.4+) — documentado como limite W3.

## §3 — O QUE FALTA (cross-crate / CONTENDED / chrome → Coord ou renderer-owner)

Não toquei — são fora da minha pasta (isolamento). Especificados aqui p/ pickup turnkey:

### A. SDF draft shader + wiring no renderer (a metade *draft*, ADR-0065 §2.1/§2.5)
- **Arquivo a criar:** `crates/ph2d-vector/shaders/boolean_sdf.wgsl` (dir não existe).
  Kernel `min/max` é trivial (drop-in abaixo); o **hard part é a rasterização
  VectorNetwork→SDF 2D** (≤0.2ms) + setup do compute pass (buffers, dispatch) **dentro de
  `ph2d-vector`** (crate do renderer — precedente: adjustment kernels em
  `ph2d-render/src/layer_compositor/`). Cross-crate → renderer-owner/Coord.
- WGSL boolean (ADR-0065 §2.1, **5 ops em SDF**; os outros 4 ficam Linesweeper-only §3.3):
  ```wgsl
  @compute @workgroup_size(8, 8)
  fn boolean_main(@builtin(global_invocation_id) gid: vec3<u32>) {
      let pos = vec2<f32>(gid.xy);
      let d_a = sample_sdf(input_a, pos);
      let d_b = sample_sdf(input_b, pos);
      var d: f32;
      switch params.op_kind {
          case 0u: { d = min(d_a, d_b); }                          // Union
          case 1u: { d = max(d_a, -d_b); }                         // Subtract
          case 2u: { d = max(d_a, d_b); }                          // Intersect
          case 3u: { d = max(-min(d_a,d_b), min(d_a,-d_b)); }      // Exclude
          case 8u: { d = abs(d_a) - params.round_radius; }         // Outline
          default: { d = d_a; }
      }
      output_sdf[gid.xy] = d;
  }
  ```
  Determinismo (§2.4): fixed res, ordered reductions, `#pragma fma_off`, sem atomics.
- O motor exato (§1) é o output do `eval`; o SDF é só silhueta real-time pro slider drag
  (NÃO topology — ADR-0065 §2.3). Ligam-se via draft+reconcile no consumidor (bridge).

### B. Bridge multi-nó + memoização do Cook (`render_loop/vector_graph_bridge.rs`)
- Hoje cozinha **1 nó** `vector.source` e **reconstrói registry+graph+Cook todo frame**
  (o próprio §3 do bridge admite: memoização é o W3 perf follow-up).
- Pro smoke do boolean: graph **multi-nó** `source(a)+source(b)+boolean(a,b)` → render do
  resultado. + **persistir `Cook`+`Graph`+`NodeRegistry`** no estado do shell (re-cook só no
  param edit — sem isso o "real-time" é fake). `render_loop/mod.rs` é **CONTENDED (Grupo D)**
  — coordenar antes de mexer no call-site. Provavelmente chrome → Coord-B.

### C. Panel chrome (`ph2d-panel-vector-graph`, Coord-B como T3.1)
- Dropdown do `op` (9 variants) + plumbing dos 2 inputs geométricos. Eu fiz nó+lógica; o
  painel/bridge é chrome → Coord.

### D. Content-LRU 50MB (ADR-0058 §2.6) — DEFERIDO p/ T5.2
- O memo per-instância do Cook já dá "re-cook só no edit" (suficiente p/ W3). O cache
  content-addressed cross-instância (LRU 50MB) casa com T5.2 (SDF Hybrid full). Documentado.

## §4 — T3.4 `vector.offset` TAMBÉM PRONTO (commit `6957d21`)

Segui pro T3.4 na mesma sessão. Dois entregáveis (verdes, scoped):
- **NOVO crate satélite `ph2d-vector-kurbo`** (bridge): a **única** crate que fala
  kurbo+linesweeper — `network_to_bezpath` / `contours_to_network` (snap Q16.16) /
  `boolean_paths` (wrapper Linesweeper) / `fill_rule_of` + re-exports. Extraí a conversão
  do boolean pra cá → o fan-out (boolean, offset, e os próximos outline-stroke/roughen)
  compartilha UMA conversão testada, sem divergência, e kurbo fica confinado (alvo do
  futuro gate `vello_kurbo_only_in_ph2d_vector`). **Refatorei `ph2d-node-vector-boolean`
  pra usar o bridge** (engine.rs agora é só o vocabulário de 9 ops; 12 testes seguem verdes).
- **NOVO `ph2d-node-vector-offset` (`vector.offset`)**: offset CPU de regiões fechadas via
  **stroke-band + boolean** (o fallback kurbo `Offset` do spec) — stroke do boundary em
  largura 2·|d|, Union (outward) / Difference (inward). Params offset / join
  (0=Round/1=Bevel/2=Miter) / miter_limit. `Effect::Pure`. 9 testes (bbox cresce/encolhe por
  distância, over-erosão→empty, zero=identity, joins, determinismo, cook path). Registrado
  via node-sync. Open-path→region offset fica pro `vector.outline-stroke` (§2.2.4).
- Gates: clippy limpo, machete zero unused, staleness verde, contract gate intacto.
- **Falta pro offset (mesma natureza do boolean §3)**: GPU Euler-spiral real-time (Levien+
  Uguray, opt later/T5) é a versão GPU; o CPU entregue é o suficiente p/ W3. Painel/bridge
  multi-nó idem §3.B/§3.C (chrome → Coord).

## §5 — Próximos na wave
- **T3.5** audit + fechamento W3: lente A (boolean edge-cases — stress do linesweeper),
  lente B (SDF vs Linesweeper consistency), lente C (perf 100 paths). Golden cross-OS.
  (Próximos nós do fan-out — outline-stroke/roughen/etc — já têm o bridge `ph2d-vector-kurbo`
  pronto pra reusar.)

## §6 — COORD: RATIFICAÇÃO (§2) + §3.B LANDADO + §3.A/C teed-up (2026-06-04)

**Ratificado (Enio aprovou via "faça tudo"):**
- **§2.1 `linesweeper` 0.3.0** — RATIFICADO. Motor spec-named, MIT/Apache, kurbo-native,
  passou machete/deny/contract-gate. Ressalva "early beta" registrada: T3.5 lente A é o
  stress; mitigação fuzz→vendor/patch se achar bug upstream. Hand-roll = semanas + risco.
- **§2.2 `Effect::Pure`** — RATIFICADO. Substrate-correto (Stateful = invisível ao Cook de
  apresentação → smoke morto). O memo do Cook por `(input revs + param hash)` É o cache do ADR.
- **§2.3 semântica dos 9 ops** — RATIFICADO. Diretos exatos; compostos/equivalências documentadas.

**§3.B — LANDADO (commit Coord local):** `vector_graph_bridge` agora cozinha multi-nó
`source(a) + source(b = a rotacionado 45°) + boolean(op)` → render, sob `PH2D_VECTOR_GRAPH=1`.
`op` via `PH2D_VECTOR_BOOL_OP` (0..=8). Helper puro `cook_boolean_smoke` + 2 testes (fan-in
cozinha a VectorNetwork; 9 ops cozinham). **NÃO toquei `render_loop/mod.rs`** (assinatura do
dispatch intacta → call-site contendido preservado). source_b é cópia rotada pq `vector.source`
não tem param de posição — 2-source independente é o §3.C.

**§3.A SDF — DEFERIDO p/ ADR (não é drop-in):** `ph2d-vector` **não tem nenhuma camada GPU**
(sem `shaders/`, sem compute, sem `GpuContext`; renderiza via vello). O §3.A é **construir um
subsistema GPU de compute do zero** (+ rasterização VectorNetwork→SDF, "a hard part") — adição
arquitetural que pede ADR + design (wgpu dep, GpuContext em ph2d-vector, determinismo fixed-res).
**O eval exato (§1) está correto sem ele** — o SDF é só silhueta real-time pro slider drag. Padrão-
ouro = ADR-first, não rush. Próximo: ADR-0065-amendment "SDF compute layer em ph2d-vector".

**§3.C op dropdown no painel — POLISH (segue o §3.B):** adicionar `op` ao `VectorGraphParams`
ripplaria editor-core `ids/chrome.rs` (VGRAPH_OP) + o gate `node_id_collisions` (lista hand-
maintained) + `[ParamSpec; 8]`→9 + teste. O smoke (op via env) já prova a integração; o dropdown
interativo + 2-source authoring são a próxima passada focada (chrome → Coord).
═══════════════════════════════════════════════════════════════════
