//! **Os CHIPS numéricos da barra de transporte** (`Time` · `Frame` · `Length`) — a célula
//! `[pad | nome | pad | chip]` e a régua que a mede.
//!
//! Irmão por RESPONSABILIDADE do [`super`] (e cortado do tecto de 600 linhas em 2026-09-23, quando
//! a célula passou a partilhar a coluna medida dos toggles): aquele ficheiro dispõe a barra, este
//! responde *como se desenha um valor numérico com nome* — e é por isso que a régua ([`chip_w`]) e
//! o pintor ([`labeled_chip`]) moram juntos, no mesmo sítio onde discordaram durante meses.

use super::*;

/// The Dur(s) chip's stepper increment — **0.2 s per click** (Enio, 2026-07-23:
/// *"faça cada clique subir ou descer o valor em 0.2 seg"*). The Time/Frame chips
/// step by a frame (`1/fps`); a DURATION is coarser, so `1/fps` (≈0.04 s) produced
/// the fiddly, "wrong-looking" values the report named (4.04, 4.08, …). A round
/// 0.2 s keeps the authored duration on clean tenths.
const DUR_STEP_SECONDS: f64 = 0.2; // LITERAL-PX-OK: 0.2 s stepper increment (a time value), not a UI px metric

/// A célula `[pad | nome | pad | chip]` de um chip numérico — o VALOR cai no mesmo `x` que o
/// interruptor de um toggle ([`toggle_w`]), porque os dois começam por `pad + label_col + pad`.
pub(super) fn chip_w(label_col: f32) -> f32 {
    let pad = Spacing::Xs.px();
    pad + label_col + pad + CHIP_W
}

/// The Dur(s) chip: **the view's own duration** (`view_length_seconds` is
/// stamped per view — clip in Keys, scene in Arrange, the open container
/// inside one). The router writes the same scope (`length_scope`).
///
/// **No authored duration reads ∞** (Enio, 2026-07-28): a composition with no
/// `length_override` is UNBOUNDED (`0` = infinite), so the box shows the infinity glyph
/// rather than a derived number — the same `!view_length_explicit` that removes the veil
/// (`ruler_veil`). This is why it is the ONLY chip that can be `unbounded`; Time/Frame
/// always show a finite number. Typing/dragging a positive value authors a finite end;
/// typing `0` clears back to infinite.
pub(super) fn paint_length_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
    label_col: f32,
) {
    labeled_chip(
        ctx,
        theme,
        x,
        y,
        chip_rotulo(Item::LengthChip),
        ids::TIMELINE_LENGTH_NUM,
        snap.view_length_seconds,
        DUR_STEP_SECONDS,
        2,
        !snap.view_length_explicit,
        label_col,
    );
}

/// One labeled numeric chip — `[pad | label | pad | chip]`, the same cell the toggles draw.
/// The three transport chips (Time / Frame / Dur) are this one drawing, and the
/// measure arm ([`chip_w`]) describes exactly this layout.
#[allow(clippy::too_many_arguments)]
pub(super) fn labeled_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    text: &str,
    id: ph2d_a11y::NodeId,
    value: f64,
    step: f64,
    decimals: usize,
    unbounded: bool,
    label_col: f32,
) {
    // ⚠️ **O mesmo `pad` da régua ([`chip_w`])** — até 2026-09-23 o pintor usava `Sm/2` e a régua
    //    `Xs/2`, e o doc por cima dizia que as duas descreviam «exactamente» a mesma disposição.
    let pad = Spacing::Xs.px();
    label(ctx, theme, text, x + pad, y, label_col);
    chip(
        ctx,
        theme,
        x + pad + label_col + pad,
        y,
        id,
        value,
        step,
        decimals,
        unbounded,
    );
}
