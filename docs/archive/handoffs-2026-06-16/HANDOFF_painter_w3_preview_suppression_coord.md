═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter W3 — preview "replaces sprite" (sprite suppression)
Autor: Implementador Painter (sessão 2026-06-01) · BLOQUEADO por isolamento
═══════════════════════════════════════════════════════════════════

## Sintoma (Enio, smoke 2026-06-01)
"A primeira camada não é a imagem real ainda: ao mudar a opacidade da Layer 1
(base) muda só a opacidade do traço, não da imagem. Se for preciso modificar a
própria imagem da sprite, não dá."

## Diagnóstico (confirmado, estático)
O Painter desenha o composite das layers como um **overlay POR CIMA** da sprite
real, que continua sendo renderizada normalmente por baixo:

- `painter_bridge::dispatch` (fim) faz `vector_scene.draw_image_rgba(preview…)`
  — overlay no topo do footprint da sprite.
- A sprite real continua indo pelo `sim_extract::run` (pipeline normal).

A base layer (`canvas_rgba`, carregada de `set_source`) = pixels da sprite +
traços. Ao baixar a opacidade da base, o **overlay** desbota e revela a sprite
real (opacidade cheia) por baixo → parece que "só o traço desbota". O comentário
no `painter_bridge.rs` já admite o débito: *"T1.5 MVP intentionally does NOT
suppress the underlying sprite … sprite suppression is W2."*

**O que JÁ funciona:** pintar na Layer 1 + Apply assa o composite de volta na
sprite (`run_full` → `OneShotImageOp`), então modificar a imagem É possível. O
furo é só o **preview ao vivo** (opacidade/representação da base não batem com a
sprite real porque há duplicação overlay-vs-sprite).

## Fix (canônico — espelhar o BgRemoval)
O BgRemoval já resolve isto via `PreviewOverride` (troca a textura da sprite
in-place, sem overlay nem duplicação):

1. `bgremoval_preview::dispatch` faz upload do preview p/ uma textura GPU →
   `self.bgremoval_preview_gpu` (`{entity_bits, texture_id}`).
2. `mod.rs:246` constrói `sim_extract::PreviewOverride { entity_bits,
   texture_id, premultiplied: true }`.
3. `mod.rs:284` passa pro `sim_extract::run(… preview_override …)`; o extract
   (sim_extract.rs:339-432) troca o binding de textura da sprite → a sprite
   renderiza COM o preview, in-place. Sem overlay, sem sprite duplicada.

**Para o Painter, mesma receita:**
- (bridge, in-pasta do impl) `painter_bridge` faz upload do composite
  (`take_preview_arc`, que já compõe — fix `c2d34f3`) p/ uma textura GPU →
  `painter_preview_gpu` (mirror de `bgremoval_preview_gpu`); **remover** o
  `draw_image_rgba` overlay.
- (mod.rs, **Coord** — está sendo editado por você agora) construir o
  `PreviewOverride` do painter + passar pro `sim_extract::run`. Provavelmente
  unificar com o slot único `preview_override` (hoje só o bgremoval usa): quando
  Painter ativo, passar o override do painter.
- Premultiplied: o canvas do painter é straight RGBA8 (`into_straight()` no
  source-push). O upload precisa casar o flag `premultiplied` com o formato real
  (bgremoval usa premul byte-space). Conferir paridade (audit-lente cor) p/ não
  repetir o bug histórico straight↔premul.

## Por que está bloqueado pro impl (isolamento §0.2)
- É mudança de **pipeline de render** (sim_extract + mod.rs + upload GPU) =
  foundational/shell, fora da pasta autorizada (só o bridge é).
- `shells/desktop/src/render_loop/mod.rs` está **modificado por você (Coord)**
  agora — um commit escopado do impl tocando mod.rs capturaria teu WIP (colisão).
- Você está editando `ph2d-render` (compositor GPU) em paralelo — este é o teu
  domínio; o GPU `LayerCompositor` do Bloco 2 pode até ser a via de upload.

## Estado do Painter W3 UI (impl, local, não-pushado)
Commits: `71f5f3a` (UI-plumbing) · `920ccda` (z-order render) · `c2d34f3`
(composite preview + click-suppress + blend dropdown + outline) · `fe48f82`
(base=sprite no-blend + chip 1-linha + padding). Tudo verde (gates + clippy +
nextest). Falta só esta peça de preview-suppression (Coord) p/ a base layer se
comportar como a imagem real ao vivo.
═══════════════════════════════════════════════════════════════════
