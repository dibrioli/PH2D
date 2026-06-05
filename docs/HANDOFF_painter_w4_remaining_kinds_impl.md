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
**12/24 kinds PRONTOS** (compute CPU + GPU + paridade):
  HSB(0), BrightnessContrast(1), Invert(2), Posterize(3), Threshold(4),
  Exposure(5), Vibrance(6) — escalares; **Curves(7) + Levels(8)** — LUT
  display-space (binding `adj_luts`); **PhotoFilter** — CPU-first (gel
  warm/cool linear + preserve-lum), `gpu_params` empacotado mas `gpu_code=None`
  até o Coord landar `ADJ_PHOTO_FILTER` (§GPU-COORD); **ColorBalance** — CPU-first
  (shift per-canal display-space pesado por tonal-range + preserve-lum); GPU =
  per-channel `adj_luts` (reuso da máquina de Curves) + flag preserve (§GPU-COORD-CB);
  **ChannelMixer** — CPU-first (matriz 3×4 display-space + mono); GPU = matriz
  uniform 12-float (cross-channel, NÃO é LUT) → expansão de uniform do Coord;
  **BlackAndWhite** — CPU-first (decomposição 6-hue → luma ponderada + tint OKLab);
  GPU = algoritmo per-pixel nonlinear (nem LUT nem matriz) → port WGSL do Coord.

**✅ BATCH-1 COMPLETA** (PhotoFilter + ColorBalance + ChannelMixer + BlackAndWhite).
Os 4 kinds per-pixel slider/toggle/segment estão prontos CPU-first; o caminho GPU
de cada um está especificado pro Coord (escalar / `adj_luts` / matriz uniform /
algoritmo WGSL). Próximo da implementação = **BATCH-2** (GradientMap + SelectiveColor).

**INFRA NOVA (REUSE p/ os próximos):**
  - **toggle rack** genérica: kind com `bool` vira UI só com arms em
    `adjustment_toggle_params` + `set_adjustment_toggle_param` (compute.rs) — o
    painel renderiza um switch por slot (`AdjToggle0/1`, click→flip espelho do
    mask-invert; tool `flip_adjustment_toggle`). Usado por PhotoFilter +
    ColorBalance; pronto p/ ChannelMixer (monochromatic), Noise (monochromatic).
  - **segment rack** genérica (1-de-N, N≤3): `adjustment_segment_params` +
    `set_adjustment_segment_param` (compute.rs); painel renderiza uma fileira de
    segment-buttons (`AdjSegment0/1/2`, padrão das abas de canal do Curves; tool
    `set_adjustment_segment`). Usado pelo scope do ColorBalance; pronto p/
    SelectiveColor `method` (Relative/Absolute), GradientMap `interpolation`.

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

**11 STUBS** (no-op identity hoje — `_ => {}`, gpu_code None; menu mostra mas não
faz nada): GradientMap, GaussianBlur, MotionBlur, Bloom, Noise,
Sharpen, Halftone, ChromaticAberration, ColorLookupLut,
SelectiveColor, ShadowsHighlights. **Cap ≤32 (24
usados) — sobra; NÃO adicione kinds, só implemente os existentes.**

───────────────────────────────────────────────────────────────────
§2 — A SUA ETAPA: os 7 kinds PER-PIXEL não-espaciais (CPU-first)
───────────────────────────────────────────────────────────────────
São transformações por-pixel (sem vizinhança) → reusam a engine direto. Ordem
recomendada (fácil→difícil). Params já existem em `mod.rs` (contrato congelado).

**BATCH 1 — slider-rack (espelho do Levels: ZERO UI nova):** adicione arms em
`adjustment_slider_params` + `set_adjustment_slider_param` (compute.rs) e o painel
genérico já renderiza. Compute = arm em `apply_adjustment`.
  1. **PhotoFilter** `{temperature, density, preserve_luminosity}` — ✅ **PRONTO**
     (commit `d45aa8e`). Gel warm/cool em LINEAR (multiply físico) + preserve-lum
     (renorm Rec.709). 2 sliders (Temp centrado / Density) + 1 toggle via a toggle
     rack nova. `gpu_params=[temp,density,preserve]` empacotado; falta só o case
     WGSL do Coord (§GPU-COORD). Precedente vivo p/ os toggles dos próximos.
  2. **ColorBalance** `{cyan_red, magenta_green, yellow_blue, scope, preserve_lum}`
     — ✅ **PRONTO** (commit pendente). 3 sliders (C/R, M/G, Y/B) + scope via a
     segment rack nova + toggle preserve via a toggle rack. Compute = shift
     per-canal display-space pesado por máscara tonal (Shadows `(1-s)²` / Mid
     `1-(2s-1)²` / Highlights `s²`) + preserve-lum (renorm Rec.601). GPU NÃO é
     escalar: é per-channel `adj_luts` (reuso de Curves) + flag preserve — export
     `colorbalance_display_luts` pronto; spec no §GPU-COORD-CB.
  3. **BlackAndWhite** `{reds,yellows,greens,cyans,blues,magentas, tint_color,
     tint_amount}` — ✅ **PRONTO** (commit `85c96ec`). 6 sliders de peso por hue
     (decomposição RGB-hexágono → luma display-space) + Tint switch + (com tint)
     sliders Hue + amount. **Default manual = pesos PS 40/60/40/60/20/80** (o
     all-zero colapsava tudo em `min(r,g,b)`). Tint controlado por Hue+amount
     sliders (dirige o `OklchColor` por completo) — SEM popover de picker (decisão:
     o BlenderColorPicker é integração de popover grande; sliders Hue+amount são
     gold-standard e bastam). UI bespoke só pela ORDEM (switch entre os 6 hue e os
     tint sliders). GPU = algoritmo per-pixel nonlinear → port WGSL do Coord.
  4. **ChannelMixer** `{red_out:[f32;4], green_out, blue_out, monochromatic}` —
     ✅ **PRONTO** (commit `e596f41`). Matriz 3×4 display-space (PS aplica no
     canal gamma-encoded) + mono (linha red_out → gray em todos os canais).
     **Default manual = matriz identidade** (o derive all-zero era degenerado →
     tudo preto, a armadilha do Levels). UI bespoke tipo Curves: abas de canal de
     saída (view-state `active_mixer_channel`) + 4 sliders (R/G/B source + Const)
     da linha ativa + toggle mono. Edit forwarda `SelectOption(PAINTER_MIXER_EDIT,
     "layer:output:slot:value")` (a aba ativa carrega o canal). GPU = matriz
     uniform 12-float (cross-channel, NÃO cabe em `adj_luts`) → expansão de uniform
     do Coord; `gpu_code=None` até lá. Helpers `paint_labeled_slider`/
     `paint_toggle_row` extraídos (DRY entre racks genéricos + mixer).

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
§GPU-COORD — port WGSL do PhotoFilter (escalar, [f32;3]) — pendência do Coord
───────────────────────────────────────────────────────────────────
CPU pronto (`apply_photo_filter`, compute.rs) + `gpu_params=[temperature, density,
preserve_lum01]` já empacotado. Para virar real-time: (1) add const
`ADJ_PHOTO_FILTER` no `layer_composite.wgsl` + o case em `apply_adjustment`;
(2) flip `gpu_code(PhotoFilter)` de `None` p/ o código (próximo livre = 9) em
`mod.rs`. O case opera no acc LINEAR (mesmo espaço da CPU):
```
let t = p0; let density = p1;
if (density == 0.0 || t == 0.0) { return rgb; }          // identidade
let WARM = vec3(1.0, 0.75, 0.45); let COOL = vec3(0.55, 0.80, 1.0);
let anchor = select(COOL, WARM, t >= 0.0);
let mag = abs(t);
let gel = vec3(1.0) + (anchor - vec3(1.0)) * mag;        // white→anchor por |t|
let eff = vec3(1.0) + (gel - vec3(1.0)) * density;       // white→gel por density
let LW = vec3(0.2126, 0.7152, 0.0722);                   // Rec.709 linear luma
let l_in = dot(rgb, LW);
var outc = rgb * eff;
if (p2 > 0.5) {                                          // preserve_luminosity
    let l_out = dot(outc, LW);
    if (l_out > 1e-6) { outc = outc * (l_in / l_out); }
}
return outc;
```
Gate de paridade: `gpu_adjustment_matches_cpu_reference_each_kind` (já cobre
auto quando `gpu_code` vira Some — neutro garante finitos via o contract test).

───────────────────────────────────────────────────────────────────
§GPU-COORD-CB — port WGSL do ColorBalance (LUT per-channel + preserve) — Coord
───────────────────────────────────────────────────────────────────
ColorBalance NÃO é escalar [f32;3] — é um transfer per-canal display-space (com
scope baked na LUT) + um renorm de luma opcional. CPU pronto
(`apply_color_balance`). O caminho GPU real-time **reusa a máquina do Curves**:
1. flatten chama `colorbalance_display_luts(p) -> [[f32;256];3]` (já exportado) e
   sobe nas MESMAS `adj_luts` (3×256) que o `ADJ_CURVES` lê;
2. add `ADJ_COLOR_BALANCE` no `layer_composite.wgsl`: amostra `adj_luts` per-canal
   (idêntico ao case Curves) e, se `preserve_luminosity`, faz o renorm display:
```
// rgb_lin (linear) -> display, amostra LUT per-canal, (renorm), -> linear
let s = vec3(lin_to_srgb(rgb.r), lin_to_srgb(rgb.g), lin_to_srgb(rgb.b));
var o = vec3(sample_adj_lut(base, 0u, s.r), sample_adj_lut(base, 1u, s.g),
             sample_adj_lut(base, 2u, s.b));
if (preserve > 0.5) {
    let LW = vec3(0.299, 0.587, 0.114);                 // Rec.601 display luma
    let l_in = dot(s, LW); let l_out = dot(o, LW);
    if (l_out > 1e-6) { o = clamp(o * (l_in / l_out), vec3(0.0), vec3(1.0)); }
}
return vec3(srgb_to_lin(o.r), srgb_to_lin(o.g), srgb_to_lin(o.b));
```
   `preserve_luminosity` precisa chegar ao shader (1 scalar — via `adj_params` ou
   um bit no header da LUT; decisão do Coord). Sem preserve, o case é literalmente
   o do Curves (3×256 transfer) — pode até compartilhar `ADJ_CURVES` se o flatten
   preencher `adj_luts` com as tabelas do ColorBalance.

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
