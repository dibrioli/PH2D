# ADR-0026: Pluggable sprite source strategies (M14.5)

**Status:** Accepted
**Data:** 2026-05-12
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Depende de:** [ADR-0021 (Sim ↔ Present)](0021-simulation-presentation-boundary.md), [ADR-0025 (GameObject)](0025-gameobject-model.md)

## Contexto

M14.4d shipou um dynamic atlas (Skyline packer, 4096²) que cobre o
caso "muitos sprites pequenos, auto-packing". M14.5 do plano abriu
3 estratégias paralelas porque o atlas único não atende todos os
content pipelines:

| Estratégia | Quando | Trade-off |
|---|---|---|
| **A — Dynamic Atlas (Skyline)** | sprites diversos, sem pipeline externo | 1 draw call; packing runtime não-determinístico; atlas pode esgotar |
| **B — Hand-packed Atlas** | art pipeline definido (tile-sets, character sheets); curated assets | packing ótimo; workflow manual (Aseprite/TexturePacker JSON) |
| **C — Individual Textures + draw-call batching** | HD 2D, sprites grandes, conteúdo procedural | full resolution sempre; mais state changes GPU; mental model simples |

Sem esta decisão a fachada de `Sprite` ficaria amarrada ao atlas
único e os outros caminhos seriam refactors disruptivos.

## Decisão

**Sprite component gains a `source: SpriteSource` enum** que dispatcha
entre as três estratégias. As três coexistem na mesma cena; o
renderer agrupa instances por `texture_id` e emite 1 draw call por
grupo (Godot 4 `RenderingServer` pattern).

```rust
pub enum SpriteSource {
    Atlas { key: u32 },                 // M14.4d Skyline
    Individual { texture_id: u32 },     // M14.5 C — own wgpu::Texture
    // M14.5 B (hand-packed) currently surfaces only as AtlasMeta in
    // ph2d-asset — renderer-side variant + UI selector land in
    // M14.5 inspector phase.
}
```

`Sprite::VERSION` bumped to 2 quando o campo `atlas_index: u32` virou
`source: SpriteSource` (postcard prefab format breaks; fixtures regen
automaticamente via cooker JSON5).

## Implementação

### A — Dynamic Atlas (shipped M14.4d)

- `crates/ph2d-render/src/atlas.rs::TextureAtlas` (Skyline via
  `rect_packer`).
- `insert(key, w, h, rgba) -> Result<AtlasRegion>`; `remove(key)` para
  free-list; `regrow_inplace` para 2× resize quando esgota (M14.4f).
- Renderer-side `texture_id = ATLAS_TEXTURE_ID = 0` (sentinel).

### B — Hand-packed Atlas (shipped: loader half)

- `crates/ph2d-asset/src/hand_packed.rs::parse_atlas_meta` aceita
  Aseprite "Hash" + TexturePacker JSON (mesma shape).
- `AtlasMeta { image_filename, image_size, regions: BTreeMap<String, AtlasRegion> }`
  + `region_uv(region) -> [f32; 4]` para o futuro extract path.
- Renderer-side store (`HandPackedAtlasStore`) + `SpriteSource`
  variant **deferred** — sem UX exposure ainda; vem com M14.5
  inspector.

### C — Individual Textures + draw-call batching (shipped M14.5 C)

- `crates/ph2d-render/src/individual.rs::IndividualTextureStore`:
  - `acquire(gpu, bgl, w, h, rgba) -> u32 texture_id` aloca uma
    `wgpu::Texture` própria + bind group pré-construído contra o
    pipeline's `material_bgl`.
  - Refcount via `retain` / `release` para evict quando o último
    sprite libera. Drop deterministico, sem GC scan.
  - `bind_group(id)` é o handle que o renderer set_bind_group(1)
    durante o draw.
- `RenderInstance` ganhou `texture_id: u32 + _pad: [u32; 3]` (Pod
  size 48 → 64 bytes; shader vertex layout segue só 4 attributes,
  ignora o trailing).
- `SpriteRenderer::render`:
  1. Drain instances pra scratch.
  2. `scratch.sort_by_key(|i| i.texture_id)` (stable).
  3. `compute_runs(scratch, &mut runs)` agrupa por id contíguo.
  4. Walk runs: bind_group switch + draw per run.
- `IndividualTextureStore::replace_pixels` reutiliza a `wgpu::Texture`
  quando dims iguais; recria com mesmo `texture_id` se mudou. Útil
  pro hot-reload bridge (M14.5 inspector phase).

### HR / ADR compliance

- **HR-3 alloc-free**: render path reusa `scratch: Vec<RenderInstance>`
  + `runs: Vec<DrawRun>` capacities frame-to-frame.
- **HR-5 / ADR-0022**: `BTreeMap` everywhere (atlas regions, individual
  store, hand-packed regions). `HashMap` interdito no path
  determinístico.
- **ADR-0021**: `SpriteSource` é SimComponent; UV resolution + draw
  batching são present-side; cumpre extract-only boundary.
- **ADR-0024**: `RenderInstance.texture_id` é CPU metadata; o shader
  vertex layout não muda — render_pipeline pinning compatível.

## Trade-offs aceitos

- **`RenderInstance` 16-byte tail padding desperdiçado no upload GPU**:
  ~960 KB/s @ 1000 sprites @ 60 fps. Desprezível vs o GPU work; alternar
  pra parallel array `Vec<u32>` de sort keys complicaria o reuse de
  capacities. Reabrir se profile mostrar cost ≥ 1% do frame budget.
- **Last-hit-wins picking**: sem Z field, `pick_sprite_at_world`
  retorna o último archetype-order match. Within-archetype = insertion
  order; cross-archetype = implementation-defined bevy_ecs. Suficiente
  até `Sprite` ganhar Z layer (futuro M15+).
- **Premultiplied alpha não enforced**: atlas + individual passam
  RGBA sem premul; tints com `α<1` compositam errado em ambos. Bug
  latente que M14.4d já carregava — não é regressão M14.5; abrir
  ADR-0027 quando atender HD 2D real.

## Alternativas consideradas

- **`Sprite` enum top-level** (em vez de struct com `source` field):
  forçaria refactor em todos os callsites que pattern-matcham `Sprite`
  (Prefab serializer, Inspector, MCP, hero spawner). Custo > benefício.
- **TextureRef = `Arc<wgpu::Texture>`**: deferia drop ao end-of-frame e
  complicava sort-key stability. Refcount explícito é mais simples e
  deterministic.
- **Per-sprite Z field upfront**: opening a transform/render schema
  expansion antes do gizmo ship era prematuro; voltaremos quando o
  primeiro projeto-piloto pedir layering real.

## Arquivos canônicos

- `crates/ph2d-render/src/sprite.rs` — `Sprite`, `SpriteSource`,
  `RenderInstance`.
- `crates/ph2d-render/src/individual.rs` — `IndividualTextureStore`.
- `crates/ph2d-render/src/renderer.rs::render` — sort + walk_runs +
  multi-draw.
- `crates/ph2d-asset/src/hand_packed.rs` — Aseprite/TexturePacker JSON
  parser.
- `crates/ph2d-render/src/picking.rs` — `pick_sprite_at_world` +
  `selection_bbox_world` (M14.7 A foundation).

## Próximos passos

- **M14.5 inspector phase**: Inspector "Render Source" sub-panel
  (dropdown: Dynamic Atlas / Hand-packed / Individual) + strategy-
  specific fields. Wires user-facing toggle to `acquire_individual` /
  `insert_atlas_sprite` / `HandPackedAtlasStore`.
- **B renderer integration**: `HandPackedAtlasStore` + new
  `SpriteSource::HandPacked { atlas_id, region_index }` variant.
  Reuses the draw-call batching from C (each hand-packed atlas =
  one texture_id).
- **ADR-0027 premultiplied alpha** quando o primeiro HD 2D pilot
  reportar compositing wrong.
