═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · BUG de espaço de cor no stroke do Painter (gamma)
Autor: Implementador Painter (sessão 2026-05-31) · reporte de fronteira foundational
Origem: Enio smoke da W2 — "a cor da caixa não corresponde a cor pintada"
═══════════════════════════════════════════════════════════════════

> ✅ **RESOLVIDO pelo Coord (2026-05-31, commit `d58bc37`).** Decode sRGB→linear
> na leitura do dst + encode linear→sRGB na escrita, em `cpu_render.rs` (VIVO) +
> `stamp.wgsl` (futuro), bit-idênticos via `ph2d_color::srgb::{srgb_to_linear_byte,
> linear_to_srgb_byte}`. Regressão `painted_byte_is_srgb_encoded_color_not_raw_linear`
> prova byte pintado == swatch. 155 tests + parity + clippy verdes. **Pendente: smoke
> do Enio.** Implementador: segue T2.4/T2.6/T2.7 — este bug não é mais teu.

╔═══════════════════════════════════════════════════════════════════╗
║ TL;DR — O swatch (W2 sidebar, commit c43d5d7) está CORRETO: exibe a ║
║ cor escolhida no picker. O BUG é o **stroke ao vivo**, que grava    ║
║ cor LINEAR num canvas que é sRGB (`Rgba8UnormSrgb`) → traço sai      ║
║ escuro/dessaturado. Fix mora em `ph2d-painter-brush` (CPU + GPU,    ║
║ parity-gated). NÃO toca `ph2d-render` (tua área ativa KTX2).        ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
SINTOMA (Enio)
───────────────────────────────────────────────────────────────────
Painter ativo → swatch "Color" no painel mostra a cor X (ex.: laranja
#FF8800). Pinta-se um traço → o traço aparece numa cor diferente
(mais escura / mais avermelhada) que o swatch. Pré-existente: o thumb
flutuante antigo (removido em 6125409) exibia a mesma cor — o swatch no
painel só tornou o desencontro óbvio ao colocá-lo ao lado da pintura.

───────────────────────────────────────────────────────────────────
CAUSA RAIZ (estática, alta confiança)
───────────────────────────────────────────────────────────────────
Duas escritas da MESMA `active_color` (OKLCH) em espaços de gamma
DIFERENTES:

1. SWATCH / replay / Inspector = **sRGB gamma-encoded** (correto):
   - `PainterUiSnapshot::active_color_srgb8()` → `painter_oklch_to_srgb8`
     → `LinearRgba::to_srgb(...)`  ── crates/ph2d-tool-painter/src/color.rs:179-185
   - É o mesmo `to_srgb` do caminho de render/reproject/Inspector; o teste
     `stored_stroke_color_renders_to_the_same_srgb_as_the_painter_swatch`
     (crates/ph2d-tool-painter/src/tool.rs:2607) prova round-trip exato
     (±1) pra cor escolhida no picker.

2. STROKE AO VIVO (CPU + GPU) = **linear, sem gamma** (errado):
   - CPU: `oklab_to_linear_srgb(...)` e grava o LINEAR direto no canvas
     ── crates/ph2d-painter-brush/src/cpu_render.rs:147-154
     com comentário 226-228 assumindo o canvas como `Rgba8Unorm` (sem
     gamma); dst lido como linear, resultado escrito como linear.
   - GPU: idem — `textureStore(canvas_out: rgba8unorm, ...)` grava linear
     ── crates/ph2d-painter-brush/src/shader/stamp.wgsl:503,546-551

3. MAS o canvas/sprite é **`Rgba8UnormSrgb`** (espera bytes sRGB-encoded;
   a GPU faz hw sRGB-decode ao samplear):
   - crates/ph2d-render/src/atlas.rs:471 · crates/ph2d-render/src/individual.rs:333
   - confirmado pelos comentários em shells/.../bgremoval_preview.rs:247 e
     hero_intents/image_edit/color_equalization.rs:64 ("hw sRGB-decode").

4. O boundary do Painter NÃO faz conversão de gamma:
   - `read_sprite_source` (shells/.../hero_intents/texture_edit.rs:48) lê os
     bytes do sprite direto pro `canvas_rgba`; o writeback (image_edit) idem.
   - Nenhum `linear_to_srgb` no pipeline de stamp (só nos helpers de
     display em color.rs).

⇒ O stamp escreve valor LINEAR num buffer interpretado como sRGB pela GPU
  → "decodifica de novo" → traço escuro/dessaturado. Ex.: laranja, canal G
  sRGB=136 → painter calcula linear≈0.245 → grava byte 62 → sprite sRGB
  exibe ~62 em vez de 136. Swatch mostra 136 (certo); traço mostra 62.

  Bônus crítico: o REPLAY de stroke já usa `to_srgb` (sRGB) → o mesmo traço
  ao vivo (linear) e recarregado (sRGB) divergiriam entre si.

───────────────────────────────────────────────────────────────────
FIX RECOMENDADO (padrão-ouro: compositing linear correto num canvas sRGB)
───────────────────────────────────────────────────────────────────
Em `crates/ph2d-painter-brush/src/cpu_render.rs` E `.../shader/stamp.wgsl`
(IDÊNTICOS — gate `shader_oklab_coefficients_bit_identical_with_rust`):
  a) ao LER o dst do canvas: `sRGB → linear` antes de premul/blend
     (hoje trata o byte como linear — cpu_render.rs:232-244 + análogo wgsl).
  b) ao ESCREVER o resultado: `linear → sRGB` antes de gravar o u8/textureStore
     (hoje grava linear — cpu_render.rs ~251+ e stamp.wgsl:546-551).
  c) fonte (oklab→linear) permanece linear — correto pro blend.

Isso alinha o canvas ao upload `Rgba8UnormSrgb` E ao caminho de replay
(`to_srgb`). Reusar a MESMA transfer function que `ph2d-color`
(`SrgbRgba::to_linear` / `LinearRgba::to_srgb`) pra não introduzir um 3º
encoder. Atenção: o gate de parity exige CPU e WGSL bit-idênticos — a
encode/decode precisa dos mesmos literais nos dois.

Revalidar: parity gate CPU/GPU + golden fixtures de stamp (provavelmente
re-lock) + smoke do Enio (traço == swatch).

Decisão de arquitetura embutida (tua/ADR-0051): confirmar que o canvas do
Painter é canônico-sRGB (consistente com o resto da engine). Se algum dia
mover pra working-space linear (Rgba16F etc.), aí o swatch/replay é que
mudariam — mas hoje TUDO no resto da engine é sRGB, então a correção é no
stamp render.

───────────────────────────────────────────────────────────────────
ESCOPO / FRONTEIRA
───────────────────────────────────────────────────────────────────
- Fix é em `ph2d-painter-brush` (crate compartilhada do módulo Painter,
  parity-gated). NÃO toca `ph2d-render` (KTX2 W2 — tua área ativa). Sem
  colisão de arquivo esperada.
- Distinto do carry-over conhecido `premult×opacity` da W2 (aquele é
  alpha; este é gamma de cor).
- Por ser correção de espaço de cor em crate compartilhada + território
  ADR-0051, o Implementador PAROU e reportou (não corrigiu unilateralmente),
  conforme §0 #2. Enio decidiu: Coordenador corrige; Implementador segue o
  plano (T2.4/T2.6/T2.7).

───────────────────────────────────────────────────────────────────
ESTADO DO IMPLEMENTADOR (não-bloqueante pra ti)
───────────────────────────────────────────────────────────────────
- TASK 1 (swatch dentro do painel) = commit `c43d5d7`. Correto e
  independente deste bug.
- T2.4 (modifier square) + T2.6 (a11y) = commit `59555b7`. Square pinta +
  arma/desarma `eyedropper_armed` (Accent quando armado); gates verdes.
- T2.7 = audit in-session feito (sem soft-lock, sem mudança de contrato,
  LOC/a11y/labels OK); smoke do Enio pendente.

═══ FOLLOW-UP FOUNDATIONAL #2 (separado do gamma — pro teu radar) ═══
T2.4 só fecha de ponta-a-ponta quando o SHELL consumir `eyedropper_armed`:
o gesto hold-modifier + tap-no-canvas precisa LER o pixel sob o cursor e
aplicar `PainterUiEdit::SetColorSrgb(rgba)` no tool (+ desarmar). Isso toca
shell/canvas (foundational) → fora do escopo do Implementador. Hoje armar o
eyedropper só acende o highlight; não samplea (e NÃO quebra a pintura —
`eyedropper_armed` é flag de display, não gateia `begin_stroke`). O id e o
edit já existem (`PAINTER_SIDEBAR_MODIFIER_SQUARE` + `SetColorSrgb` da
T2.3), então é só o wire do gesto no shell.
═══════════════════════════════════════════════════════════════════
