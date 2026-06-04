═══════════════════════════════════════════════════════════════════
HANDOFF → Coord (ou Vector impl) · ADR-0065 SDF Phase 3 — wire o draft no app
Autor: Coordenador (sessão 2026-06-04, pós SDF Phases 1+2) · contexto separado
═══════════════════════════════════════════════════════════════════

## §0 — Estado: SDF compute FECHADO E PROVADO (Phases 1+2)

`ph2d-vector-sdf` (crate satélite novo) entrega o **draft** do draft+reconcile
do ADR-0065 — commits locais `5f68dbb` (núcleo CPU) + `4c24aa6` (GPU + parity).
ADR-0065 **amendment-1** registra placement + algoritmo + faseamento. **NÃO há
nada a refazer no compute** — Phase 3 é só consumir + renderizar.

## §1 — A API a consumir (já pronta, testada)

```rust
use ph2d_vector_sdf::{Bounds, SdfGrid, SdfOp, network_sdf, boolean_sdf};
use ph2d_vector_sdf::gpu::GpuSdf;

// CPU (usável em draft-res; é a fonte-da-verdade + fallback sem GPU):
let bounds = Bounds::of_network(&net_a, PAD);        // ou min/max manual co-locado p/ a+b
let sdf_a = network_sdf(&net_a, RES, bounds);        // SdfGrid: data[y*res+x], NEGATIVO dentro
let sdf_b = network_sdf(&net_b, RES, bounds);
let draft = boolean_sdf(&sdf_a, &sdf_b, SdfOp::Union).unwrap();  // min/max combine

// GPU (real-time; mesma assinatura, paridade sub-pixel provada no Metal):
let pipe = GpuSdf::new(gpu);                          // construir 1× (cachear)
let sdf_a = pipe.network_sdf(gpu, &net_a, RES, bounds);
// boolean_sdf (min/max) é CPU-trivial; combine os dois grids GPU-readback.
```
- **`SdfOp`** = os 5 ops SDF-representáveis: `Union`(min) `Subtract`(max(a,-b))
  `Intersect`(max) `Exclude`(sym-diff) `Outline{radius}`. Os **outros 4**
  (Merge/Crop/Divide/Trim) **NÃO** têm forma SDF → caia no motor exato sempre.
- **`SdfGrid { res, bounds, data: Vec<f32> }`** — co-locar `bounds`+`res` entre A
  e B (mesma janela) senão `boolean_sdf` retorna `None`.

## §2 — Phase 3: o que falta (2 peças)

### A. SDF → visual (a peça com algoritmo)
O `vector_graph_bridge` desenha um `VectorNetwork` via `draw_vector_network`. O
SDF é um **grid**, não um network. Opções:

**Recomendado — marching-squares (contorno zero):** extrai o iso-contorno
`sdf == 0` do grid → segmentos de linha, e **stroke** direto na cena vello
(`vector_scene.inner_mut().stroke(...)`) p/ a silhueta-outline; OU monte um
`VectorNetwork` (vertices+segments+1 region) do loop fechado e reuse
`draw_vector_network` p/ silhueta-fill. Algoritmo (em `ph2d-vector-sdf`, novo
`marching.rs`, testável puro):
```text
para cada célula 2×2 do grid (corners d00,d10,d11,d01 = SDF):
  case = (d00<0)<<0 | (d10<0)<<1 | (d11<0)<<2 | (d01<0)<<3   // 16 casos
  p/ cada aresta com troca de sinal: cross = lerp pela razão d0/(d0-d1) (em world)
  emita o(s) segmento(s) do caso (tabela marching-squares 2D padrão)
```
Determinismo: grid fixo + ordem de varredura fixa (§2.4). Teste: square → o
contorno é um quadrado ~no boundary (±meia-célula). **Coloque `marching_contour`
no crate `ph2d-vector-sdf`** (perto do SDF, testável sem GPU).

*Alternativa (coverage-texture):* render o grid como textura e fill onde `<0` —
mais rápido p/ res alta mas precisa de um path de textura na cena (o bridge hoje
é vello/path-based). Marching-squares encaixa no bridge atual sem novo path.

### B. Máquina draft/reconcile (estado no bridge)
Hoje o bridge cozinha o **exato** todo frame. O draft+reconcile (ADR-0065):
- **Durante o drag** (params mudando frame-a-frame) → mostre o **SDF draft**
  (barato, GPU): `boolean_sdf(network_sdf(a), network_sdf(b), op)` → contorno.
- **No settle** (params estáveis por ≥1 frame / drag-end) → mostre o **motor
  exato** (`cook_boolean_smoke` já existe no bridge).
- Detecção: `vector_graph_bridge` constrói inline cada frame (sem estado). Adicione
  um `thread_local!`/`OnceLock<Mutex>` com `last_params` + um settle-counter:
  `if params != last { draft } else { exact }`. (Persistir o Cook é o perf
  follow-up separado; aqui só o gate draft-vs-exact.)
- Só p/ os 5 ops SDF; nos outros 4 → sempre exato (sem draft).

### C. Integração (onde, sem colisão)
- **`shells/desktop/src/render_loop/vector_graph_bridge.rs`** (Coord/T3.1, meu) —
  o `dispatch` ganha o branch draft. **NÃO mude a assinatura** `dispatch(visible,
  camera, window_size, vector_scene)` → o call-site em `render_loop/mod.rs:1125`
  fica intacto (mod.rs é CONTENDED-Vector). Adicione o dep
  `ph2d-vector-sdf` no `shells/desktop/Cargo.toml`.
- `GpuSdf` precisa de `&GpuContext`. O bridge hoje NÃO recebe gpu — pegue do
  `renderer.gpu()` se acessível no call-site, OU use o **CPU `network_sdf`** em
  draft-res (64²–128² é ~1ms, suficiente p/ smoke) e deixe o GPU p/ quando o
  bridge tiver o GpuContext threaded. (CPU-draft é o caminho de menor fricção p/
  o smoke; o GPU é a escala.)

## §3 — Pegadinhas
- **Co-localizar bounds+res** entre A e B (mesma janela) senão `boolean_sdf`=None.
- **Só 5 ops** têm SDF; Merge/Crop/Divide/Trim → exato sempre (sem draft).
- **Winding GPU = NonZero global** (união) — bate com o CPU p/ region única
  NonZero (o caso dos sources). Multi-region/EvenOdd no GPU é aproximação (draft).
- **Determinismo** (§2.4): marching-squares com varredura ordenada + grid fixo.
- O draft é **silhueta, não topologia** — não tente derivar vertices/regions
  exatos do SDF; o reconcile (motor exato, §1 do boolean) é quem produz o network.

## §4 — Testes a adicionar
- `marching_contour`: square SDF → contorno ~no boundary (±meia-célula); círculo
  (se tiver) → contorno fechado.
- bridge: draft-vs-exact gate (params mudam → draft path; estáveis → exact).

## §5 — Referências
- ADR-0065 + **amendment-1** (placement + faseamento): `docs/architecture/decisions/0065-vector-sdf-hybrid-gpu.md`.
- API: `crates/ph2d-vector-sdf/src/lib.rs` (`network_sdf`/`boolean_sdf`/`SdfOp`/`network_edges`) + `src/gpu.rs` (`GpuSdf`) + `shaders/network_sdf.wgsl`.
- Bridge a estender: `shells/desktop/src/render_loop/vector_graph_bridge.rs` (`cook_boolean_smoke` = o reconcile exato já lá).
═══════════════════════════════════════════════════════════════════

## §6 — STATUS: CLOSED (2026-06-04, smoke-OK do Enio)

Phase 3 ENTREGUE + validado visualmente. Commits locais (não-pushados):
- `58ee181` — `marching.rs` (marching_contour, determinístico) + draft/reconcile
  gate no bridge (5 ops SDF; 4 topológicos → exato). CPU 96² (não GPU — bridge
  não recebe `GpuContext`; ver follow-up). Testes: marching 3/3 + bridge 5/5.
- `87aa7ec` — auto-frame: as fontes (~100 wu) estouravam a câmera default
  (`height_world` 10) → blob de tela cheia. Local `Camera2d` enquadra o resultado.
- `e2156d5` — `VGRAPH_PANEL` na lista fallback do z-order (editor-core paint.rs):
  o painel de sliders nunca pintava (gap pré-existente do smoke; só o cook fora
  validado). Agora doca sobre o inspector quando `PH2D_VECTOR_GRAPH=1`.

**Smoke (Enio 2026-06-04):** painel + 8 sliders OK, cria Rect/Ellipse/Polygon/
Star/Spiral, draft+reconcile funciona. Quirks aceitos como artefatos do scaffold
(união-de-2-cópias-rotacionadas = demo de booleana; Height ignorado em radiais =
contrato do `vector.source`).

**FOLLOW-UP (deferido, BAIXO valor):** GPU SDF no bridge. A ADR §amendment-1 §195
previa "GPU silhouette during drag", mas p/ o caso vetorial (fontes pequenas,
poucas arestas) o CPU 96² é ~1ms — não é gargalo. GPU só rende em escala (4K/
muitas arestas) que o smoke não exercita. Threadear `GpuContext` no bridge
(assinatura `dispatch` muda → toca `mod.rs` CONTENDED-Vector) **só quando houver
necessidade de perf real medida** (não otimizar prematuro). Não bloqueia nada.
═══════════════════════════════════════════════════════════════════
