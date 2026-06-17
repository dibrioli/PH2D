═══════════════════════════════════════════════════════════════════
HANDOFF → Impl Vector / Painter · W8 brush bridge CORE pronto (produtizar é o próximo passo)
Autor: Coordenador (jornada 2026-06-06) · responde HANDOFF_vector_w8_pattern_along_path_impl.md §4
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
A metade RASTER do W8 (a que tu deferiste) tem **core pronto + testado**: crate
satélite **`ph2d-brush-along-path`** (`f97e477`) que stampa um brush ao longo de um
path vetorial. Decisão de arquitetura tomada: **NÃO** virou graph-node (seria raster
dead-end exigindo `Domain::Raster`+`RASTER_PORT`+glue novo p/ 1 consumidor — prematuro).
É uma crate isolada que só LÊ os 2 contratos congelados que ponte. Produtizar (op do
Painter OU, no futuro, um node) é o próximo passo — não-bloqueante.

## §1 — O QUE LANDOU (`f97e477`)
`crates/ph2d-brush-along-path/` — **sem** prefixo `ph2d-node-` (senão node-sync
auto-descobre + gera `::register` inexistente; staleness do registry-init confirma
que NÃO é node). Deps: só `ph2d-vector-doc` (VectorNetwork) + `ph2d-painter-brush`
(Stamp/apply_stamps) + glam. Zero edição nas crates alheias.
- **`stamps_along_path(path, &BrushAlongPathParams) -> Vec<Stamp>`**: ordena os
  segmentos numa cadeia + flatten cúbico (mesma convenção `c1=start+out` do
  `pattern_along_path`) + dab a cada `size·spacing_ratio` px por **arc-length**
  (pitch de stroke CONTÍNUO, vs o `count` do shape-placer). `align_to_tangent`
  rotaciona cada dab pra tangente (`rotation_rad` via atan2 — sub-pixel, ok p/ raster).
- **`rasterize_along_path(...)`**: end-to-end path → dabs → pixels via `apply_stamps`.
- `Stamp::zeroed()` é o construtor (grain_layer=u32::MAX, _pad=0); seto só os fields
  user-facing. 6 testes (spacing arc-length, rotação π/2, segue cúbica não corda,
  rasteriza a stroke + canto limpo, inputs degenerados, reprodutível). clippy `-D`
  limpo.

## §2 — PRODUTIZAR (escolha do dono, não-bloqueante)
O core devolve `Vec<Stamp>` — serve qualquer das vias:
- **(A) Op do Painter (mais provável):** uma operação "stroke brush ao longo de um
  path selecionado" no Painter — chama `stamps_along_path` + agenda os stamps no
  pipeline GPU (`StampPipeline::encode`) ou commita via o CPU `apply_stamps`. É o
  Painter consumindo geometria vetorial. **Domínio do impl Painter.**
- **(B) Graph-node raster (futuro, quando houver ≥2 consumidores raster):** aí sim
  vale o foundational — `Domain::Raster`+`RASTER_PORT`+glue `ph2d-raster-graph`
  (espelho do `ph2d-vector-graph`) + o node `ph2d-node-raster-brush-along-path`
  embrulhando este core + consumo no renderer. Precisa ADR (Coord-only). Prematuro p/ 1 node.

## §3 — POSSE / SEGUIMENTO
- Cap de nós 18/18 intacto (este NÃO é node). Contratos `VectorNetwork`/`Stamp` não
  tocados (só lidos). Sem push (Coord shipa). `git status` conferido: nada alheio.
- Polimentos v2 do core (não-bloqueantes): pressure/size ao longo do path (taper),
  spacing variável, jitter. O `BrushAlongPathParams` é o ponto de extensão.
═══════════════════════════════════════════════════════════════════
