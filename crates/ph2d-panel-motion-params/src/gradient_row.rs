//! The interactive **Gradient** editor (doc 85) — the params-panel row for a
//! `motion.color_ramp` Custom gradient (and, later, any multi-stop colour param).
//!
//! The colour sibling of [`crate::curve_row`]: a gradient is a `ph2d_color::ColorRamp`
//! serialized in a text param, drawn here as a **gradient bar** with **draggable position
//! markers** (the SAME foundational `InteractiveState::CurvePoint` x-drag the curve editor
//! uses — only the x is folded, the y ignored) + a **per-stop OKLCH swatch** (the canonical
//! colour UI, `register_picker_swatch` — the shell reads the pick back into the string, the
//! mirror of [`crate::snapshot::ColorRow`]). `+`/`−` add/remove a stop and the interp button
//! cycles the ramp's global interpolation. The artist never sees the string.
//!
//! **Markers never cross in position** (each drag clamps between its neighbours), so their
//! order is stable and the marker↔swatch indices stay valid frame to frame — exactly the
//! curve editor's reason for the same clamp.

use crate::snapshot::{
    GradientRow, MAX_GRADIENT_STOPS, param_grad_add_id, param_grad_editor_id, param_grad_hue_id,
    param_grad_interp_id, param_grad_preset_id, param_grad_remove_id, param_grad_space_id,
    param_grad_stop_id, param_grad_swatch_id,
};
use ph2d_a11y::NodeId;
use ph2d_color::srgb::linear_to_srgb_byte;
use ph2d_color::{
    ColorRamp, GradientPreset, RampColorMode, RampHue, RampInterp, RampStop, parse_gradient,
    serialize_gradient,
};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};
use std::cell::Cell;

const BAR_H: f32 = 24.0; // LITERAL-PX-OK: gradient-bar height
const SWATCH_H: f32 = 18.0; // LITERAL-PX-OK: per-stop swatch strip height
const PRESET_H: f32 = 14.0; // LITERAL-PX-OK: preset seed-chip strip height
const MARKER_R: f32 = 5.0; // LITERAL-PX-OK: position-marker dot radius
pub(crate) const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a marker's pointer grab box
const GRID_W: f32 = 1.0; // LITERAL-PX-OK: border / ring stroke width
const RING_W: f32 = 1.5; // LITERAL-PX-OK: selected-stop ring width
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− button width
const INTERP_W: f32 = 64.0; // LITERAL-PX-OK: interp cycle-button width
const MODE_W: f32 = 40.0; // LITERAL-PX-OK: space / hue cycle-button width ("HSV", "Near")
const MIN_DPOS: f32 = 0.001; // LITERAL-PX-OK: min position gap between stops (NORMALIZED 0..1)
/// Recuo de uma amostra dentro da célula dela — os dois lados.
///
/// ⚠️ **Estava escrito DUAS vezes como literal local** (na tira de amostras e na de presets), e
/// desde o bloco Z (doc 91) ele é lido por uma terceira: a derivação do
/// [`MAX_GRADIENT_STOPS`]. Três cópias de um número que
/// tem de concordar são duas oportunidades de ele deixar de concordar.
pub(crate) const SWATCH_PAD: f32 = 2.0; // LITERAL-PX-OK: swatch inner padding within its cell

thread_local! {
    /// `(slot, stop index)` of the last-grabbed marker — the target of `−` and the
    /// accent highlight. Panel-scoped (mirror of the curve editor's selection).
    static SELECTED: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
}

fn selected_for(slot: usize) -> Option<usize> {
    SELECTED
        .with(|s| s.get())
        .and_then(|(sl, p)| (sl == slot).then_some(p))
}

/// Parse the row's serialized gradient, else the default black→white ramp — a fresh
/// Gradient opens on something draggable, never an empty box.
fn working(value: &str) -> ColorRamp {
    parse_gradient(value).unwrap_or_default()
}

/// The sRGB8 (straight) colour of a ramp stop — what the swatch paints + the picker
/// seeds with. RGB through the linear→sRGB transfer (the wire is linear), alpha 255
/// (stops are opaque, doc 85).
fn stop_srgb(stop: &RampStop) -> [u8; 4] {
    [
        linear_to_srgb_byte(stop.color[0]),
        linear_to_srgb_byte(stop.color[1]),
        linear_to_srgb_byte(stop.color[2]),
        255,
    ]
}

/// One COLOUR row's store-registration data — the Gradient editor and the Palette strip
/// both fill it, because the palette needs an exact SUBSET (swatches + buttons, no
/// markers: a palette has no positions to drag).
///
/// One Gradient row's store-registration data (the caller's Phase B/C mutable-store pass
/// applies these — swatches become picker swatches, markers become `CurvePoint` handles).
pub(crate) struct ColourRowWidgets {
    /// `(marker id, parent, index, canvas)` — registered as `CurvePoint` (position drag).
    pub markers: Vec<(NodeId, NodeId, u8, Rect)>,
    /// `(swatch id, srgb)` — registered as a picker swatch + seeded with the colour.
    pub swatches: Vec<(NodeId, [u8; 4])>,
    /// `+`/`−`/interp buttons + the preset-seed chips — all registered as plain buttons.
    pub buttons: Vec<NodeId>,
}

impl ColourRowWidgets {
    pub(crate) fn new() -> Self {
        Self {
            markers: Vec::new(),
            swatches: Vec::new(),
            buttons: Vec::new(),
        }
    }
}

/// Paint a gradient bar into `rect` as adjacent vertical strips of `ramp.eval(t)` (linear→sRGB).
/// Shared by the main editor bar and the small preset-seed chips — the colours are DATA (the
/// swatch precedent), not tokens.
fn paint_gradient_bar(scene: &mut VectorScene, rect: Rect, ramp: &ColorRamp) {
    let strips = (rect.w * 0.5).clamp(8.0, 128.0) as usize; // LITERAL-PX-OK: strip-count clamp
    for k in 0..strips {
        let t = (k as f32 + 0.5) / strips as f32;
        let [r, g, b, _a] = ramp.eval(t);
        // ⚠️ Uma linha só, e não é estilo: o gate casa o marcador **na linha da chamada**, e numa
        // chamada quebrada em cinco linhas ele acabava no `);` — ou seja, em lugar nenhum. É a
        // forma que o irmão `paint_ramp_widget.rs` já usa (integração 2026-07-30).
        let (rb, gb, bb) = (
            linear_to_srgb_byte(r),
            linear_to_srgb_byte(g),
            linear_to_srgb_byte(b),
        );
        let col = Color::from_rgba8(rb, gb, bb, 255); // LITERAL-COLOR-OK: the ramp's OWN colour is data (the swatch precedent), not a token
        let sx = rect.x + (k as f32 / strips as f32) * rect.w;
        let sw = rect.w / strips as f32 + 1.0; // +1: overlap so no seam gap between strips
        fill_rounded_rect(scene, Rect::new(sx, rect.y, sw, rect.h), 0.0, col);
    }
}

/// Paint one Gradient row: header (label + interp + `+`/`−`), then the gradient bar with
/// draggable position markers, then the per-stop swatch strip. Registers hit rects + draws;
/// the `CurvePoint`/`Button`/picker-swatch STORE states ride back in `out` for Phase B/C.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_gradient_row(
    row: &GradientRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut ColourRowWidgets,
) -> f32 {
    let gap = Spacing::Xs.px();
    let ramp = working(&row.value);
    let n = ramp.len().min(MAX_GRADIENT_STOPS);
    let sel = selected_for(slot).filter(|&p| p < n);

    // ── Header: rótulo (esquerda) + os botões (direita) ──
    //
    // ⚠️ **A LISTA é a régua.** Os botões são colocados da direita para a esquerda e a
    // largura do rótulo é a SOBRA — a constante escrita à mão que morava aqui
    // (`- INTERP_W - BTN_W*2 - gap*3`, com um comentário contando os vãos) envelhece no
    // dia em que um botão entra, e foi exatamente o que o espaço de interpolação fez.
    //
    // ⚠️ **O botão de MATIZ só existe fora do RGB**, onde ele decide alguma coisa: o braço
    // `Rgb` do `mix2` nunca chama `lerp_hue`, então em RGB ele seria um controle que gira
    // e não muda um pixel.
    let mut buttons: Vec<(f32, &'static str, NodeId)> = Vec::with_capacity(5);
    if ramp.color_mode != RampColorMode::Rgb {
        buttons.push((MODE_W, ramp.hue.name(), param_grad_hue_id(slot)));
    }
    buttons.push((MODE_W, ramp.color_mode.name(), param_grad_space_id(slot)));
    // ⚠️ **O rótulo é o da PARADA quando há uma selecionada com interpolação
    // própria** — o botão cicla aquela, e um rótulo que continuasse a mostrar a
    // global seria pintar de uma fonte e despachar de outra.
    buttons.push((
        INTERP_W,
        sel.and_then(|i| ramp.stops()[i].interp)
            .map_or_else(|| interp_name(ramp.interp), RampInterp::name),
        param_grad_interp_id(slot),
    ));
    buttons.push((BTN_W, "+", param_grad_add_id(slot)));
    buttons.push((BTN_W, "\u{2212}", param_grad_remove_id(slot)));
    // Um vão por botão: o do primeiro é o que o separa do rótulo.
    let used: f32 = buttons.iter().map(|(bw, _, _)| bw + gap).sum();
    paint_text(
        text_system,
        scene,
        &row.label,
        x,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        (w - used).max(0.0),
        resolve(ColorToken::Text2, theme),
    );
    let mut bx = x + w;
    for (bw, label, id) in buttons.into_iter().rev() {
        bx -= bw;
        let brect = Rect::new(bx, y, bw, ROW_H_PX);
        bx -= gap;
        fill_rounded_rect(
            scene,
            brect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        paint_text_centered(
            text_system,
            scene,
            label,
            brect,
            TypeToken::Base.px(),
            resolve(ColorToken::Text1, theme),
        );
        hit_index.register(id, brect);
        out.buttons.push(id);
    }

    // ── Gradient bar: adjacent vertical strips filled with eval(t) ──
    let by0 = y + ROW_H_PX + gap;
    let bar = Rect::new(x, by0, w.max(1.0), BAR_H);
    paint_gradient_bar(scene, bar, &ramp);
    stroke_rounded_rect(
        scene,
        bar,
        Radius::Sm.px(),
        GRID_W,
        resolve(ColorToken::TextDisabled, theme),
    );

    // ── Position markers on the bar bottom edge (draggable, x-only) ──
    let accent = resolve(ColorToken::Accent, theme);
    let ring = resolve(ColorToken::TextDisabled, theme);
    for (i, stop) in ramp.stops().iter().take(n).enumerate() {
        let id = param_grad_stop_id(slot, i);
        let cx = bar.x + stop.pos.clamp(0.0, 1.0) * bar.w;
        let cy = bar.y + bar.h;
        let grab = Rect::new(cx - GRAB_R, cy - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0);
        hit_index.register(id, grab);
        // The drag canvas is the BAR: the dispatch normalizes the pointer's x to
        // `pos ∈ [0,1]` across it (the y is ignored — a stop has only a position).
        out.markers
            .push((id, param_grad_editor_id(slot), i as u8, bar));
        let ring_col = if sel == Some(i) { accent } else { ring };
        fill_circle(scene, cx, cy, MARKER_R + RING_W, ring_col);
        let sr = stop_srgb(stop);
        fill_circle(
            scene,
            cx,
            cy,
            MARKER_R,
            Color::from_rgba8(sr[0], sr[1], sr[2], 255), // LITERAL-COLOR-OK: the stop's own colour is data
        );
    }

    // ── Per-stop swatch strip: one swatch per stop, evenly spaced, opens the OKLCH picker ──
    let sy0 = bar.y + bar.h + gap;
    let cell = (w / n.max(1) as f32).max(1.0);
    for (i, stop) in ramp.stops().iter().take(n).enumerate() {
        let sid = param_grad_swatch_id(row.name, i);
        let srgb = stop_srgb(stop);
        let pad = SWATCH_PAD;
        let srect = Rect::new(
            x + i as f32 * cell + pad,
            sy0,
            (cell - pad * 2.0).max(1.0),
            SWATCH_H,
        );
        fill_rounded_rect(
            scene,
            srect,
            Radius::Sm.px(),
            Color::from_rgba8(srgb[0], srgb[1], srgb[2], 255), // LITERAL-COLOR-OK: the stop's own colour is data
        );
        stroke_rounded_rect(
            scene,
            srect,
            Radius::Sm.px(),
            if sel == Some(i) { RING_W } else { GRID_W },
            if sel == Some(i) {
                accent
            } else {
                resolve(ColorToken::TextDisabled, theme)
            },
        );
        hit_index.register(sid, srect);
        out.swatches.push((sid, srgb));
    }

    // ── Preset seeds: a row of mini gradient chips (Rainbow / Heat / Ice / Grayscale). Clicking
    // one LOADS its stops into the editable ramp (doc 85) — the presets appear IN the editor and
    // become draggable/recolourable, not a separate immutable mode. The colours ARE the identity
    // (rainbow / warm / cool / grey), so no label is needed. ──
    let py0 = sy0 + SWATCH_H + gap;
    let pcell = (w / GradientPreset::ALL.len() as f32).max(1.0);
    for (i, preset) in GradientPreset::ALL.into_iter().enumerate() {
        let id = param_grad_preset_id(slot, i);
        let pad = SWATCH_PAD;
        let prect = Rect::new(
            x + i as f32 * pcell + pad,
            py0,
            (pcell - pad * 2.0).max(1.0),
            PRESET_H,
        );
        paint_gradient_bar(scene, prect, &preset.ramp());
        stroke_rounded_rect(
            scene,
            prect,
            Radius::Sm.px(),
            GRID_W,
            resolve(ColorToken::TextDisabled, theme),
        );
        hit_index.register(id, prect);
        out.buttons.push(id);
    }

    // Content height (header + gap + bar + gap + swatches + gap + preset chips); the caller adds
    // the inter-row gap.
    (py0 + PRESET_H) - y
}

/// A dragged marker landed in the store's `curve_point_drag` slot — fold its x into the
/// stop's position and emit the new serialized gradient. `None` if the slot is empty or
/// not our editor. Sets the dragged stop as SELECTED.
pub(crate) fn drain_drag(store: &mut WidgetStore, slot: usize, value: &str) -> Option<String> {
    // The stash is a GLOBAL channel: the ownership question is part of the call, so a drag that
    // belongs to another panel is LEFT for it (a `take` would be irreversible — see
    // `WidgetStore::take_curve_point_drag_if`).
    let (_parent, _channel, index, x, _y) =
        store.take_curve_point_drag_if(|p| p == param_grad_editor_id(slot))?;
    let mut ramp = working(value);
    let i = index as usize;
    if i >= ramp.len() {
        return None;
    }
    // Clamp the position BETWEEN the neighbours so stops never cross (order + indices
    // stay stable — the curve editor's reasoning, exactly).
    let lo = if i > 0 {
        ramp.stops()[i - 1].pos + MIN_DPOS
    } else {
        0.0
    };
    let hi = if i + 1 < ramp.len() {
        ramp.stops()[i + 1].pos - MIN_DPOS
    } else {
        1.0
    };
    // `safe_clamp` (not `f32::clamp`): the bounds come from the NEIGHBOURS, so a
    // degenerate ramp could hand them inverted — the curve editor's same guard.
    ramp.set_position(i, safe_clamp(x, lo, hi));
    SELECTED.with(|s| s.set(Some((slot, i))));
    Some(serialize_gradient(&ramp))
}

/// Insert a stop at the midpoint of the widest position gap (its colour sampled off the
/// current ramp, so the gradient does not jump), capped at [`MAX_GRADIENT_STOPS`].
pub(crate) fn add_stop(value: &str) -> String {
    let mut ramp = working(value);
    if ramp.len() >= MAX_GRADIENT_STOPS {
        return serialize_gradient(&ramp);
    }
    let mut best = (0usize, 0.0f32);
    for i in 0..ramp.len().saturating_sub(1) {
        let gap = ramp.stops()[i + 1].pos - ramp.stops()[i].pos;
        if gap > best.1 {
            best = (i, gap);
        }
    }
    let (at, _) = best;
    let mid = (ramp.stops()[at].pos + ramp.stops()[at + 1].pos) * 0.5;
    let col = ramp.eval(mid);
    ramp.add_stop(RampStop::new(mid, col));
    serialize_gradient(&ramp)
}

/// Remove the selected stop (else the last), keeping at least two.
pub(crate) fn remove_stop(value: &str, slot: usize) -> String {
    let mut ramp = working(value);
    if ramp.len() <= 2 {
        return serialize_gradient(&ramp);
    }
    let idx = selected_for(slot)
        .filter(|&p| p < ramp.len())
        .unwrap_or(ramp.len() - 1);
    ramp.remove_stop(idx);
    SELECTED.with(|s| s.set(None));
    serialize_gradient(&ramp)
}

/// How many preset seed chips the editor offers (Rainbow / Heat / Ice / Grayscale).
pub(crate) const PRESET_COUNT: usize = GradientPreset::ALL.len();

/// The `i`-th preset (doc 85) serialized — what clicking its chip LOADS into the ramp.
pub(crate) fn preset_gradient(i: usize) -> String {
    serialize_gradient(&GradientPreset::ALL[i.min(PRESET_COUNT - 1)].ramp())
}

/// O passo seguinte da roda (Linear → Ease → Constant → Cardinal → B-Spline).
fn next_interp(i: RampInterp) -> RampInterp {
    match i {
        RampInterp::Linear => RampInterp::Ease,
        RampInterp::Ease => RampInterp::Constant,
        RampInterp::Constant => RampInterp::Cardinal,
        RampInterp::Cardinal => RampInterp::BSpline,
        RampInterp::BSpline => RampInterp::Linear,
    }
}

/// Cicla a interpolação — **da PARADA selecionada quando há uma**, da rampa
/// inteira quando não há.
///
/// ⚠️ **É esta a cura da célula da folha 09** (*"interpolação POR STOP — o ramp
/// parameter do Houdini: cada ponto carrega a própria interpolação"*): o botão
/// ciclava só a global, e o `_slot` que ele já recebia e ignorava era exactamente
/// onde a resposta estava.
///
/// A roda da parada tem um degrau a mais — **`Global`** —, e é ele que faz a
/// escolha ser reversível: sem um caminho de volta, dar interpolação própria a um
/// stop seria uma porta de sentido único.
pub(crate) fn cycle_interp(value: &str, slot: usize) -> String {
    let mut ramp = working(value);
    match selected_for(slot).filter(|&i| i < ramp.stops().len()) {
        Some(i) => {
            let next = match ramp.stops()[i].interp {
                None => Some(RampInterp::Linear),
                // A volta ao `Global` fecha a roda no fim, e não no início: quem
                // está a percorrer os modos quer vê-los todos antes de sair.
                Some(RampInterp::BSpline) => None,
                Some(cur) => Some(next_interp(cur)),
            };
            ramp.set_stop_interp(i, next);
        }
        None => ramp.interp = next_interp(ramp.interp),
    }
    serialize_gradient(&ramp)
}

/// Cicla o ESPAÇO de interpolação da rampa (RGB → HSV → HSL).
///
/// ⚠️ Sair do RGB muda a versão que o `serialize_gradient` emite (`g3`), e voltar ao RGB
/// **não a devolve** enquanto o matiz não for o `Near` — de propósito: a escolha do artista
/// sobrevive ao desvio (ver `a_hue_chosen_in_hsv_survives_a_detour_through_rgb`).
pub(crate) fn cycle_space(value: &str, _slot: usize) -> String {
    let mut ramp = working(value);
    ramp.color_mode = match ramp.color_mode {
        RampColorMode::Rgb => RampColorMode::Hsv,
        RampColorMode::Hsv => RampColorMode::Hsl,
        RampColorMode::Hsl => RampColorMode::Rgb,
    };
    serialize_gradient(&ramp)
}

/// Cicla o caminho de MATIZ (Near → Far → CW → CCW) — o arco que o HSV/HSL percorre
/// entre dois stops.
pub(crate) fn cycle_hue(value: &str, _slot: usize) -> String {
    let mut ramp = working(value);
    ramp.hue = match ramp.hue {
        RampHue::Near => RampHue::Far,
        RampHue::Far => RampHue::Cw,
        RampHue::Cw => RampHue::Ccw,
        RampHue::Ccw => RampHue::Near,
    };
    serialize_gradient(&ramp)
}

/// The English caption for the ramp interp (the interp button label).
fn interp_name(interp: RampInterp) -> &'static str {
    interp.name()
}

#[cfg(test)]
#[path = "gradient_row_tests.rs"]
mod tests;
