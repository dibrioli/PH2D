═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · W4 SPATIAL MESH WIRED + Noise/Halftone landed (impl)
Autor: Implementador Painter (jornada 2026-06-05) · resposta a
       `HANDOFF_painter_eval_and_next_sprint_impl.md` §3 P0 +
       `HANDOFF_painter_w4_spatial_gaussian_impl.md`
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
**P0 fechado e verde.** A malha espacial do W4 está LIGADA: o flatten agora emite
`LayerOp::SpatialAdjustment` pros 4 kernels espaciais (Gaussian/Sharpen/Motion/
Chroma) → a tua infra GPU pass-graph os executa no slider-drag. **Noise + Halftone**
(per-pixel coord) landaram via CPU. **Commit-path do W2 (P2): VERIFICADO wirado** —
não era o "P0 disfarçado" que temíamos. Tudo compila + 79 testes adjustment + 7 flatten
+ 22 compositor verdes.

## §1 — O QUE LANDEI (TUA reconciliação é trivial — ver §2)
**`ph2d-painter-brush` (meu):**
- `AdjustmentKind::gpu_spatial_code() -> Option<u8>` — Gaussian=0/Sharpen=1/Motion=2/
  Chroma=3, **espelho lock-step dos teus `SPATIAL_*`** (mesmo padrão do `gpu_code`↔`ADJ_*`).
- `AdjustmentParams::spatial_params() -> Option<[f32;4]>` — packing dos 4 escalares
  por kernel (`Gaussian=[radius,0,0,0]`, `Sharpen=[amount,radius,0,0]`,
  `Motion=[distance,angle_rad,0,0]`, `Chroma=[r,g,b,falloff_center]`).
- **Novo módulo `adjustments/spatial.rs`** (pub via `adjustments::`):
  - `gaussian_weights(radius)` + `motion_weights(distance)` — **CANÔNICAS, math
    IDÊNTICA aos teus placeholders** (σ=radius/3, half=ceil(radius); motion box
    uniforme). **Zero-diff** → ver §2.
  - `apply_gaussian/apply_sharpen/apply_motion_blur/apply_chromatic_aberration` —
    refs CPU canônicas (separável c/ clamp-to-edge; sharpen = `base+amount·(base−blur)`;
    motion = box nearest-tap; chroma = gather radial `scale_c=1+shift_c/corner`).
  - `apply_noise/apply_halftone` — per-pixel mas **coordinate-dependent** (hash/screen
    na coord absoluta), dirty-rect-exatos.
  - `AdjustWindow{width,height,origin_x,origin_y}` + `apply_adjustment_windowed(...)`
    — dispatch que dá geometria de janela aos kernels espaciais/coord e **delega
    `apply_adjustment` (flat) pros per-pixel** (teu gate GPU-parity que chama
    `apply_adjustment` direto fica INTOCADO — não mudei a assinatura dele).

**`ph2d-tool-painter` (meu):** `compositor/compose.rs` chama `apply_adjustment_windowed`
com a janela `(rw,rh,rx,ry)` no lugar de `apply_adjustment`.

**`shells/desktop/render_loop/painter_gpu_flatten.rs` (bridge painter — ver §4):**
emite `SpatialAdjustment{kernel,params,blend_mode,opacity}` pros kinds com
`gpu_spatial_code()`. O resto do chain já estava vivo (`painter_gpu_preview::try_drive`
→ `composite_with_luts` → premul → preview slot) — **o smoke do Gaussian agora fecha**.

## §2 — TUA AÇÃO (reconciliação ZERO-DIFF — gates ficam verdes)
As minhas `gaussian_weights`/`motion_weights` têm **a fórmula idêntica** aos teus
placeholders em `ph2d-render`. Como `ph2d-render` JÁ depende de `ph2d-painter-brush`,
o end-state limpo é **deletar as tuas duas e delegar**:
```rust
// ph2d-render::layer_compositor
pub use ph2d_painter_brush::adjustments::{gaussian_weights, motion_weights};
```
Isso mata a duplicação + trava a fonte-única. **Como a math é igual, a paridade não
muda** (não precisa re-baseline). Sharpen/Chroma: as fórmulas canônicas vivem nas
minhas `apply_sharpen`/`apply_chromatic_aberration` — usa-as como referência CPU dos
teus gates `gpu_sharpen_/gpu_chroma_matches_cpu_reference` se quiseres dedup. **Nada
bloqueia: se preferir deixar as tuas como estão, também fica verde** (são bit-iguais).

## §3 — FOLLOW-UPS (precisam de TI / coordenados — não são deferrals meus)
1. **Noise/Halftone no GPU:** hoje rodam só no **CPU fallback** (`gpu_code`/`gpu_spatial_code`
   = None → flatten devolve None p/ stacks com eles). São per-pixel mas dependem da
   **coord absoluta** (`global_id`), que o WGSL tem naturalmente. Pra acelerar:
   um `ADJ_*` coord-aware no teu `layer_composite.wgsl` (hash/screen na coord) + flip
   do `gpu_code`. **Não-bloqueante** (CPU já os deixa corretos + dirty-rect-exatos).
2. **Bilinear motion/chroma:** entreguei **nearest-tap** (zero-diff c/ teu spike). O
   refino bilinear (`cs_blur_dir`/`cs_chroma`) é conjunto — quando priorizar, troco a
   ref CPU junto pra manter paridade.
3. **Sharpen `mask_edges`:** DEFERIDO (semântica: gate do unsharp a áreas de alto
   gradiente — `|∇luma| > thr`). Me confirma o modelo que ligo CPU+GPU juntos.
4. **Gaussian premultiplied:** hoje borra straight RGBA (igual teu spike; combine
   preserva `acc.a`). Premul correto p/ transparência = mudança localizada no
   materialize/combine (TEU lado, §2 do teu handoff). Documentado em `apply_gaussian`.
5. **Bloom** (mip pyramid) + **ShadowsHighlights** (contraste local) — precisam da tua
   infra extra; ficam `None`/CPU-noop com nota até priorizarmos. (ColorLookupLut =
   .cube parser + LUT cache, é meu P1 — não-iniciado.)

## §4 — NOTA DE LOCALIZAÇÃO (discrepância no teu handoff)
O teu §3 P0.1 diz "Wire do flatten (`ph2d-tool-painter`)", mas o `flatten_for_gpu`
mora em **`shells/desktop/src/render_loop/painter_gpu_flatten.rs`** (bridge), não na
crate do tool — foi assim que o Painter impl o criou (commit `6044cc1`: "Flatten lives
in the bridge, not the tool"). Editei lá (git status de início estava limpo no shell;
sem colisão). Só pra alinhar o mapa de posse.

## §5 — P2: COMMIT-PATH DO W2 (verificado, não é P0-disfarçado)
Trace e2e (paint→preview→Apply→sprite): **strokes PERSISTEM.** `request_commit()` →
`pending_commit` → `drive_pending_commit()` (painter_bridge.rs:342) → `drain_painter`
(image_edit.rs:432) → `run_full()` → `commit_edited_texture()`. Os 3 riscos antigos
(R3-LE-4 unwired / R3-LF-3 failed-Apply destrói / R3-LF-4 cancel dropa) estão
**mitigados** (guards `has_painted_since_source`, error-returns sem zerar canvas, WAL
cancel no deactivate). **Único gap real (UX, meu, menor):** falta o `Toast::warning`
de "strokes não-aplicados" no tool-switch (TODO documentado `tool/mod.rs:54`; semântica
canvas-efêmero-até-Apply é by-design). Confirmação final = smoke manual do Enio
(pinta → Apply → reabre doc).

## §6 — GATES RODADOS (verde)
- `cargo test -p ph2d-painter-brush --lib adjustments::tests` → **79 ok** (incl. 11
  novos: spatial-code mapping, weights-sum-1, blur-flat-identity/impulse-conserva-energia,
  motion/chroma neutro-identidade, noise determinístico+dirty-rect-exato+mono,
  halftone só-ink/paper+extremos, windowed-delega-per-pixel).
- `cargo test -p ph2d-host-desktop --bins painter_gpu_flatten` → **7 ok** (novo
  `spatial_adjustment_emits_pass_graph_op`; ajustei `non_ported_*` p/ Bloom).
- `cargo test -p ph2d-tool-painter --lib compositor` → **22 ok** (dirty-rect invariants
  intactos — não quebrei `composite_region == crop(composite)`).
- `cargo check -p` painter-brush / tool-painter / host-desktop → limpo.

## §7 — POSSE / GIT
Commit local scoped (`--no-verify`, sem push): `ph2d-painter-brush` (mod+spatial+tests),
`ph2d-tool-painter/compositor/compose.rs`, `shells/desktop/.../painter_gpu_flatten.rs`,
+ este handoff. **Não toquei** `ph2d-render` (teu), nem WIP alheio (vector W6, docs).
Contrato `AdjustmentKind` intacto (24 variantes, ≤32; só métodos novos). **Você shipa
1×/jornada** quando o Enio mandar.
═══════════════════════════════════════════════════════════════════
