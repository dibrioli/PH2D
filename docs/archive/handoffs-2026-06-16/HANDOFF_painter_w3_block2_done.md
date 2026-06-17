═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter · W3 Bloco 2 ENTREGUE + Bloco 1 ratificado
Autor: Coordenador (sessão 2026-05-31) · responde ao HANDOFF_painter_w3_coord.md
═══════════════════════════════════════════════════════════════════

STATUS DOS 3 BLOCOS
- Bloco 1 (tool↔LayerStack): **RATIFICADO** — pode codar a parte interna da tool.
- Bloco 2 (compositor GPU): **FEITO** por mim (commits `411b3ae` fmt + `6ba3ed7` feat).
- Bloco 3 (dock): aguarda decisão do Enio (recomendei **C = toggle**).

═══════════════════════════════════════════════════════════════════
BLOCO 1 — ratificação (faça a parte INTERNA da tool; eu fiz/faço o shell)
═══════════════════════════════════════════════════════════════════
A abordagem recomendada do teu handoff está aprovada como está:
- `PainterTool` ganha `layers: LayerStack` + `images: BTreeMap<LayerId, LayerImage>`
  substituindo `canvas_rgba`. `set_source` = stack N=1 raster (back-compat).
- strokes miram a layer ATIVA; `current_preview()` = composite; `on_deactivate`
  commita o composite (blake3 igual hoje, sobre o composite).

Decisões (Coord):
- (a) buffers no `PainterTool` (RAM) com eviction §2.13. ✅ (migram pro GPU cache do
  Bloco 2 depois — ver "wiring" abaixo).
- (c) undo de op-de-stack no MESMO ring, transações espelhando `ImageEditTransaction`. ✅
- (b) persistência do stack no `.ph2d-painter` (formato cooked + cook-hash) = **ADR-amendment
  0043, EU autoro** (não bloqueia o caminho in-memory; o stack já é serde). NÃO mexa no
  formato congelado — pare e reporte se precisar.

═══════════════════════════════════════════════════════════════════
BLOCO 2 — API que você consome (crate `ph2d-render`, já no `pub use` raiz)
═══════════════════════════════════════════════════════════════════
Tipos: `LayerCompositor`, `LayerOp`, `LayerPixelProvider`, `LayerPixels`, `Region`,
`LayerCompositeError`, `flatten_layer_ops`, `GpuOpScratch`, `max_layers_for_budget`.

1. **Provider** — implemente sobre o teu `BTreeMap<LayerId, LayerImage>`:
   ```rust
   impl ph2d_render::LayerPixelProvider for MeuSource<'_> {
       fn layer_pixels(&self, key: u64) -> Option<ph2d_render::LayerPixels<'_>> {
           self.images.get(&LayerId(key)).map(|img| ph2d_render::LayerPixels {
               version: img.version,      // BUMP no commit de stroke (senão não re-sobe)
               rgba8: &img.rgba8,         // canvas_w*canvas_h*4, straight sRGB8
           })
       }
   }
   ```
   `key` = `LayerId.0` (u64). `version` = qualquer contador que muda quando os pixels mudam.

2. **Op-list** — achate o `LayerStack` na MESMA ordem que o CPU `composite_into`:
   top-down, **bottom-to-top dentro de cada lista de irmãos** (`ids.iter().rev()`).
   Por nó visível com opacity>0:
   - Raster → `LayerOp::Layer { key: id.0, blend_mode: layer.blend_mode.to_u8(), opacity }`
   - Group → `PushGroup`, depois emita os filhos (recursivo), depois
     `PopGroup { blend_mode: group.blend_mode.to_u8(), opacity: group.opacity }`
   - Mask → pule (T3.5). Grupos: profundidade ≤ 8 (`MAX_GROUP_DEPTH`), senão `MalformedOpList`.

3. **Compose**:
   ```rust
   compositor.composite(&gpu, &ops, &provider, w, h, Region { x, y, w, h })?;
   let tex = compositor.output_texture(); // região-sized rgba8unorm p/ blitar na sprite
   ```
   `Region::full(w,h)` p/ recompose total; região pequena = dirty-rect (stroke só toca a dab).
   Para CPU/preview sem GPU continue usando o `compositor::composite` da tool (referência).

PARIDADE GARANTIDA: o GPU bate o teu `apply_blend` ≤1 byte nos 22 modos + grupos
(gate `gpu_composite_matches_cpu_reference_*`). Então CPU-preview e GPU-final concordam.

═══════════════════════════════════════════════════════════════════
PERF (decisão minha — full-4K-50-layer é bandwidth-bound, não shader-bound)
═══════════════════════════════════════════════════════════════════
Recompose 4K cheio × 50 layers lê 1.66 GB → ~23ms neste Mac (~70 GB/s), ~4ms em GPU
≥330 GB/s. O gate de 5ms vale no caminho INTERATIVO (dirty-rect 512²: medido 3.83ms);
o full-recompose é gateado por escala linear. **Implicação pra você:** sempre passe a
`Region` suja do stroke (não `full`) no hot path — é o que mantém interativo em qualquer GPU.

═══════════════════════════════════════════════════════════════════
DEPOIS (você, in-scope): T3.4 fill (rows do painel) · T3.5 mask · T3.6 clipping
(estendem o compositor CPU; o GPU já tem o ponto de extensão de grupos pronto).
═══════════════════════════════════════════════════════════════════
