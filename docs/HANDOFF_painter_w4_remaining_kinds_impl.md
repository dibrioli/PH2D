═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter (NOVO) · W4 — os 15 kinds de ajuste restantes
Autor: Implementador Painter (sessão 2026-06-04, pós Curves/Levels) · CONTEXTO SEPARADO
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ MANDATO (Enio, herdado): toda implementação busca MÁXIMA PERFORMANCE ║
║ EM TEMPO REAL. O preview composita na GPU. Você entrega compute CPU  ║
║ (referência) + UI + o caminho GPU; o binding/shader GPU é do COORD.  ║
║ Faseamento PROVADO: CPU-first (gpu_code=None → fallback) → GPU.      ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§0 — INEGOCIÁVEIS (antes de qualquer edit)
───────────────────────────────────────────────────────────────────
- **Isolamento (CLAUDE.md §0.2):** sua pasta = `adjustments/` (compute+contrato),
  `ph2d-panel-painter-layers/` (UI), `tool/` (layers/params, NÃO compositor/cache),
  `editor-core/ids/` (aditivo). Shader/binding GPU (`ph2d-render`) + dispatch
  foundational (`editor-core/interaction/{dispatch,state}`) = **COORD, me chame**.
- **Inner loop = `cargo check -p <crate>`** no slot CoW (`scripts/slot-seed.sh`);
  gates/clippy 1× no fim. `git commit --no-verify -- <seus paths>`; NUNCA `-A`.
- **Multi-agente QUENTE:** o agente Vector cria crates novos o tempo todo
  (`ph2d-node-vector-*`, `ph2d-vector-sdf`…). Um `Cargo.toml` sem `lib.rs` quebra
  o workspace inteiro (transitório) — se `cargo` falhar em crate que você não
  tocou, **espere estabilizar** e re-rode. `git status` antes de stage; commit
  scoped CEDO cria fence.
- **UI em INGLÊS** ([[feedback-app-ui-english-only]]); zero hex/f32-literal-de-UI/
  string hardcoded (tokens/i18n, HR-15; `// LITERAL-PX-OK:` p/ dimensões).

───────────────────────────────────────────────────────────────────
§1 — ESTADO (verificado nesta sessão — NÃO refaça)
───────────────────────────────────────────────────────────────────
**9/24 kinds PRONTOS** (compute CPU + GPU + paridade):
  HSB(0), BrightnessContrast(1), Invert(2), Posterize(3), Threshold(4),
  Exposure(5), Vibrance(6) — escalares; **Curves(7) + Levels(8)** — LUT
  display-space (binding `adj_luts`, esta sessão).

**A engine está VIVA E PROVADA** — REUSE, não reinvente:
  - `compute.rs::apply_adjustment(kind, params, &mut [[f32;4]])` — dispatch
    per-kind; `acc` é straight LINEAR f32 RGBA. Adicione um arm → roda no fallback
    CPU na hora (`tool/compositor/compose.rs:~279` chama isso).
  - **GPU escalar** (`layer_composite.wgsl` `apply_adjustment` switch + `gpu_code()`
    + `gpu_params()->[f32;3]`): kind com ≤3 params escalares vira real-time só
    adicionando o case WGSL (COORD) + o code/params (você).
  - **GPU LUT** (`adj_luts` binding 6): transferência 1-D display-space per-canal
    (`curves_display_luts`/`levels_display_lut` exportam a tabela). Reuso direto
    p/ qualquer kind que seja `out_ch=f(in_ch)`.
  - **Editor bespoke** (Curves, `paint_adjust.rs::paint_curve_editor`): canvas +
    handles 2-D arrastáveis (`InteractiveState::CurvePoint` dispatch — COORD já
    landou) + abas de canal + add/remover ponto + tinta por canal. Precedente
    COMPLETO p/ qualquer Ui 1-D/2-D arrastável.

**15 STUBS** (no-op identity hoje — `_ => {}`, gpu_code None; menu mostra mas não
faz nada): ColorBalance, GradientMap, GaussianBlur, MotionBlur, Bloom, Noise,
Sharpen, Halftone, ChromaticAberration, ColorLookupLut, PhotoFilter,
SelectiveColor, ChannelMixer, ShadowsHighlights, BlackAndWhite. **Cap ≤32 (24
usados) — sobra; NÃO adicione kinds, só implemente os existentes.**

───────────────────────────────────────────────────────────────────
§2 — A SUA ETAPA: os 7 kinds PER-PIXEL não-espaciais (CPU-first)
───────────────────────────────────────────────────────────────────
São transformações por-pixel (sem vizinhança) → reusam a engine direto. Ordem
recomendada (fácil→difícil). Params já existem em `mod.rs` (contrato congelado).

**BATCH 1 — slider-rack (espelho do Levels: ZERO UI nova):** adicione arms em
`adjustment_slider_params` + `set_adjustment_slider_param` (compute.rs) e o painel
genérico já renderiza. Compute = arm em `apply_adjustment`.
  1. **PhotoFilter** `{temperature, density, preserve_luminosity}` — 2 sliders +
     1 toggle. Multiplica por uma cor de filtro quente/fria; density = força;
     preserve_lum = re-normaliza L. **Cabe em [f32;3] → GPU escalar direto** (peça
     o case WGSL ao Coord). O MAIS FÁCIL — comece aqui.
  2. **ColorBalance** `{cyan_red, magenta_green, yellow_blue, scope, preserve_lum}`
     — 3 sliders + scope (Shadows/Mid/Highlights, segmented) + toggle. Shifts
     cabem em [f32;3]; `scope`+`preserve` = >3 → **GPU precisa expansão de params
     (Coord) OU CPU-first**. O segmented de scope é um toggle bespoke pequeno.
  3. **BlackAndWhite** `{reds,yellows,greens,cyans,blues,magentas, tint_color,
     tint_amount}` — 6 sliders (peso por hue → luma) + tint (reuse o picker OKLCH
     do Sprite Inspector) + amount. >3 params → CPU-first; GPU = Coord.
  4. **ChannelMixer** `{red_out:[f32;4], green_out, blue_out, monochromatic}` —
     matriz 3×4 (12 sliders agrupados por canal de saída, ou abas tipo Curves) +
     toggle mono. >3 params → CPU-first; GPU = matriz uniform (Coord).

**BATCH 2 — bespoke (reuse os padrões do Curves):**
  5. **GradientMap** `{stops: Vec<ColorStop>, interpolation}` — mapeia LUMA →
     cor do gradiente. Compute: build de uma **LUT 256→RGB** (luma indexa), depois
     amostra. UI: editor de stops (handles 1-D arrastáveis ao longo de uma barra +
     color-pick por stop + add/remover) — **MESMA mecânica do curve editor**
     (CurvePoint dispatch + abas/add-remove). GPU: a LUT é RGB-saída (não
     per-canal-transfer) → **binding novo/estendido (`adj_luts` RGB-mode) = Coord**.
  6. **SelectiveColor** `{9× CmykAdjust, method}` — classifica o pixel em 1 dos 9
     baldes (R/Y/G/C/B/M/whites/neutrals/blacks) e aplica CMYK. UI: seletor de cor
     (dropdown 9) + 4 sliders CMYK + method (Relative/Absolute) toggle. CPU-first.

**DIFERE (precisa infra do Coord ANTES — não comece sem):**
  7. **ColorLookupLut** `{lut_3d, intensity, profile}` — LUT 3D (.cube): file IO +
     amostragem trilinear + **textura 3D GPU = Coord**. Etapa própria.

───────────────────────────────────────────────────────────────────
§3 — ETAPA SEPARADA (Coord-led): os 8 kinds ESPACIAIS multi-pass
───────────────────────────────────────────────────────────────────
GaussianBlur, MotionBlur, Bloom, Sharpen, ChromaticAberration, Noise, Halftone,
ShadowsHighlights — dependem de VIZINHANÇA (raio) → **multi-pass separável GPU
(ping-pong)**. **Essa infra NÃO existe** (confirmei: `ph2d-render` só tem o single-
pass `layer_composite.wgsl`). O mandato real-time = esses PRECISAM da GPU (CPU
per-pixel-neighborhood é lentíssimo). **É uma etapa arquitetural do COORD** (criar
o mecanismo ping-pong) antes de qualquer um desses. NÃO tente CPU-first aqui sem
alinhar — vira dívida. Reporte ao Coord quando chegar a vez.

───────────────────────────────────────────────────────────────────
§4 — RECEITA por kind (o loop que você repete)
───────────────────────────────────────────────────────────────────
1. **compute.rs:** `apply_<kind>(p, acc)` (referência canônica, display-space via
   sRGB round-trip se o kind é definido em display; LUT-it se for 1-D pra cortar
   transcendentais — vide `apply_invert`/`build_display_lut`). Arm no dispatch.
   Early-return identity no neutro (hot-path).
2. **mod.rs:** `Default` do `*Params` = NEUTRO real (cuidado: o derive all-zero pode
   ser degenerado — foi o caso do Levels). Mantenha `gpu_code()=None` até o GPU
   existir.
3. **UI:** slider-rack (`adjustment_slider_params`+`set_adjustment_slider_param`)
   p/ kinds de slider; bespoke em `paint_adjust.rs` (+ ids aditivos + state em
   `state.rs` + decode em `event.rs` + parse em `tool/trait_impls.rs`) p/ GradientMap/
   SelectiveColor. Reuse `register_button`/`paint_text_centered`/`fill_circle`/
   `stroke_polyline`.
4. **Tool:** mutadores em `tool/layers.rs` (espelho de `set_curve_point`) com o
   `after_curve_edit`-style invalidate (cut-cache fast-lane).
5. **Tests:** neutralidade, golden, alpha-preservado, LUT-vs-apply (se LUT).
6. **Coord handoff** p/ o caminho GPU: escalar (case WGSL) / expansão-params /
   LUT-RGB. Entregue a matemática pronta.

───────────────────────────────────────────────────────────────────
§5 — GATES (batched no fim; `cargo check` ESCONDE)
───────────────────────────────────────────────────────────────────
  cargo test -p ph2d-painter-brush --lib adjustments
  cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
  cargo test -p ph2d-tool-painter --lib
  cargo clippy -p ph2d-painter-brush -p ph2d-panel-painter-layers -p ph2d-tool-painter --all-targets --no-deps -- -D warnings
  fmt: `rustup run 1.95 rustfmt --edition 2024 <seus arquivos>` (NÃO `cargo fmt -p`).
  GPU (quando o Coord portar): `cargo test -p ph2d-render --test layer_compositor_gpu -- --ignored`.

───────────────────────────────────────────────────────────────────
§6 — ARMADILHAS
───────────────────────────────────────────────────────────────────
  - **GPU escalar = [f32;3] HARD LIMIT.** >3 params → CPU-first + handoff Coord
    (expansão de uniform). Não tente espremer.
  - **Default degenerado** (Levels all-zero era inválido) — sempre cheque o neutro.
  - **Display-space vs linear:** kinds "fotográficos" (filtros, gradient map sobre
    luma) costumam ser display-space → round-trip sRGB (`linear_to_srgb_f32`/
    `srgb_to_linear_f32`); os de luz (exposure) são linear. Decida por kind.
  - **Cut-cache:** edit de param = `invalidate_above`+`adjustment_cache_pending`
    (NÃO o structural `invalidate_composite`) — senão mata o FPS do drag.
  - **2-D drag em painel precisa dispatch foundational** ([[reference-panel-2d-drag-needs-dispatch]])
    — o `CurvePoint` já existe (reuse); UI 2-D nova = peça ao Coord o análogo.
  - **NÃO pusha.** Reporta commit local; Coord faz ship.

───────────────────────────────────────────────────────────────────
§7 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
  - Curves/Levels (o precedente vivo): `HANDOFF_painter_w4_bespoke_kinds_impl.md`
    (origem) + `_coord.md` (GPU LUT) + `_curve_editor_2d_*.md` (dispatch 2-D) +
    `_curve_channel_tokens_coord.md` (tokens por canal).
  - Engine/contrato: `crates/ph2d-painter-brush/src/adjustments/{mod,compute}.rs`.
  - GPU: `crates/ph2d-render/src/shaders/layer_composite.wgsl` (switch ~430,
    `adj_params` binding 5, `adj_luts` binding 6) + `layer_compositor/`.
  - UI: `crates/ph2d-panel-painter-layers/src/paint_adjust.rs` (slider-rack +
    curve editor) + `event.rs`/`state.rs`. ADR-0045 (+amendments) adjustments.
  - Plano: `docs/Painter_projeto/15_plano_de_implementacao.md` §7 (W4).
═══════════════════════════════════════════════════════════════════
