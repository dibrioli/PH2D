═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Painter GPU preview (Phase 3 — bridge wiring)
Autor: Implementador Painter · sessão 2026-06-03 · host/render = Coord-only
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ A ENGINE GPU DE ADJUSTMENTS ESTÁ PRONTA E PROVADA (Metal real):     ║
║ paridade GPU↔CPU (7 kinds ±4, opacity-parcial exato) + perf         ║
║ base+HSB full 1024² = 1.7ms (vs 55ms CPU, ~32×), 2048² = 3.2ms.     ║
║ Falta só PLUGAR o preview do painter no compositor GPU — host/render ║
║ = TEU. Spec + API + a lacuna de mask/clip abaixo.                   ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§1 — JÁ FEITO (engine + contrato; commits locais)
───────────────────────────────────────────────────────────────────
- **GPU adjustment kernels** (`e0a81c9`, `afe210f`): `ph2d-render::layer_compositor`
  ganhou `LayerOp::Adjustment{kind:u8, params:[f32;3], blend_mode:u8, opacity:f32}`
  + binding 5 (adj-params storage) + `apply_adjustment(kind,params,acc)` no
  `layer_composite.wgsl` (7 kinds: HSB/Vibrance OKLab via `pow`, B/C,
  Invert/Posterize/Threshold display-space, Exposure). Tratado em `cs_flat` E
  `cs_grouped`. Espelha o arm da CPU `composite_into`.
- **Contrato tool↔GPU** (`18a85a1`): `AdjustmentKind::gpu_code() -> Option<u8>`
  (None p/ kinds não-portados → fallback CPU) + `AdjustmentParams::gpu_params()
  -> [f32;3]`. É o que o flatten emite.
- **Gates:** GPU `gpu_adjustment_matches_cpu_reference_each_kind` +
  `gpu_adjustment_drag_full_canvas_perf` (em `tests/layer_compositor_gpu.rs`,
  `#[ignore]` = GPU lane); no-GPU: `shader_adjustment_coefficients_bit_identical_
  with_rust` + discriminante `OP_ADJUSTMENT`. Tudo verde.

───────────────────────────────────────────────────────────────────
§2 — O QUE FALTA (Phase 3 — a bridge plugar o preview na GPU)
───────────────────────────────────────────────────────────────────
Hoje `painter_bridge::dispatch` faz `take_preview_arc()` (composite CPU) → upload.
Troca por: compor na GPU e apontar o `PreviewOverride` pro resultado.

1. **Bridge possui um `LayerCompositor`** (`ph2d_render::LayerCompositor::new(gpu)`),
   per-sessão painter (mirror do `painter_preview_gpu`).
2. **Flatten `LayerStack` → `Vec<LayerOp>`** na ordem do `composite_into`
   (`ph2d_tool_painter::compositor`): walk `root()` … recursão de grupo …
   `iter().rev()` (bottom-to-top). Por nó:
     - Raster → `LayerOp::Layer{ key: id.0, blend_mode: blend.to_u8(), opacity }`
     - Group → `PushGroup` … filhos … `PopGroup{ blend, opacity }`
     - Adjustment → `LayerOp::Adjustment{ kind: k.gpu_code()?, params:
       a.params.gpu_params(), blend_mode: a.blend_mode.to_u8(), opacity: a.opacity }`
     - Mask layer → skip (compõe via parent — MAS ver §3).
   Onde mora o flatten: o `ph2d-tool-painter` NÃO depende de `ph2d-render` hoje.
   Opções: (a) add a dep e um `PainterTool::gpu_layer_ops() -> Option<Vec<LayerOp>>`
   (o doc do compositor diz "the tool flattens" — design pretendido); (b) flatten
   na bridge (já tem as duas deps; sem aresta nova). Tua escolha (a) é mais limpa/
   testável; (b) evita a aresta tool→render. Eu não decidi pra não cravar a
   arquitetura no teu host.
3. **`LayerPixelProvider`** sobre os `images: BTreeMap<RtLayerId, LayerImage>` do
   tool (key = `id.0`, `rgba8` canvas-sized straight sRGB8). `version` deve bumpar
   quando os pixels da layer mudam (stroke commit) — senão o cache não re-sobe.
   O tool precisa expor isso (accessor público ou impl do trait).
4. **`composite(gpu, &ops, &provider, w, h, Region::full)`** → `output_texture()`
   (region-sized straight sRGB8 `rgba8unorm`). No drag de slider, só `gpu_params`
   muda → re-`composite` (layers em cache, sem re-upload). ~1.7ms@1024².
5. **PreviewOverride** aponta pro `output_texture()`. **CUIDADO:** a saída é
   STRAIGHT sRGB8; o sprite sampla PREMULTIPLICADO (o caminho CPU faz
   `premultiply_rgba8` antes do upload). Precisa de um premultiply (passe extra,
   ou o sprite shader tratar straight, ou copiar pro `IndividualTextureStore` com
   premul). É detalhe teu (tu manténs o sprite/PreviewOverride/sim_extract).

───────────────────────────────────────────────────────────────────
§3 — LACUNA: o op-list GPU NÃO representa mask/clip/reference (fallback CPU)
───────────────────────────────────────────────────────────────────
`LayerOp` v1 só tem Layer/PushGroup/PopGroup/Adjustment — SEM máscara por-layer,
SEM clipping, SEM reference, e o `OP_ADJUSTMENT` não tem máscara. Então:
- **Decisão GPU-vs-CPU na bridge:** use a GPU SÓ quando o stack é representável
  (sem mask, sem clipping, sem reference; adjustments sem máscara). Senão,
  **fallback pro caminho CPU atual** (`take_preview_arc`) — correto, só mais lento.
  Cobre o caso do Enio (base + adjustment simples = GPU rápido); docs complexos =
  CPU (com o cache que tu landou). `gpu_code()` retornando None p/ um kind também
  força fallback.
- **Follow-up (maior, depois):** mask/clip/reference no shader GPU (binding de
  textura de máscara + alpha-mult; clip = alpha do base). Aí o GPU cobre tudo.

───────────────────────────────────────────────────────────────────
§4 — API PRONTA (de `ph2d_render`)
───────────────────────────────────────────────────────────────────
  pub enum LayerOp { Layer{key,blend_mode,opacity}, PushGroup,
                     PopGroup{blend_mode,opacity},
                     Adjustment{kind,params:[f32;3],blend_mode,opacity} }
  pub trait LayerPixelProvider { fn layer_pixels(&self, key:u64)->Option<LayerPixels>; }
  LayerCompositor::{new, composite(gpu,ops,src,w,h,region)->Result, output_texture()->Option<&Texture>}
  // contrato (ph2d_painter_brush::adjustments):
  AdjustmentKind::gpu_code()->Option<u8>;  AdjustmentParams::gpu_params()->[f32;3]
Verificação: un-ignore `gpu_adjustment_drag_full_canvas_perf` na GPU lane; smoke
do Enio (slider-drag = 60fps, sem custo perceptível). Os 4 commits GPU desta
sessão (`e0a81c9`,`afe210f`,`18a85a1` + os de perf anteriores) entram no teu ship.
═══════════════════════════════════════════════════════════════════
