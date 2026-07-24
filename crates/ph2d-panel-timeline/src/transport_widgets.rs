//! **The transport bar's reusable pieces** — an icon-button, a number chip, a toggle, a
//! label.
//!
//! Split out of `transport.rs` under the 600-LOC panel cap (HR-18), and a unit in its own
//! right: nothing here knows what a playhead is. They are the bar's vocabulary; `transport.rs`
//! is the sentence it makes out of them.

// The moved bodies keep the parent's imports: `transport.rs` is where this bar's
// widget vocabulary is declared, and re-listing it here is a second list to drift.
use super::*;

/// Paint one square icon-button; register its hit rect. Returns the right edge.
pub(crate) fn icon_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    glyph: IconId,
) -> f32 {
    let rect = Rect::new(x, y, BTN_W, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Compact,
        state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    x + BTN_W
}

/// Paint one square icon-button DEAD: [`ButtonState::Disabled`] and **no hit
/// registered** — the refusal is in the seam, not in the paint, so there is no
/// dispatch path for a click to lie through. Used by the transport's play/pause
/// on the Containers list, where playback does not exist (Enio, 2026-07-22).
/// Returns the right edge, like [`icon_button`], so the row's flow is identical.
pub(crate) fn dead_icon_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    glyph: IconId,
) -> f32 {
    let rect = Rect::new(x, y, BTN_W, ROW_H_PX);
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Compact,
        ButtonState::Disabled,
        ctx.scene,
        theme,
    );
    x + BTN_W
}

/// Paint one number chip (value mirrored from the snapshot when unfocused);
/// register its `[0, ∞)` range + calibrated drag + hit. Returns the right edge.
///
/// **Sem TETO, e isso é o produto** (Enio, 2026-07-23: *"a timeline está limitando a
/// simulação … permita que eu possa colocar qualquer valor em Dur"*). Os três chips do
/// transporte — Time, Frame, Dur — são grandezas que este app não sabe limitar: nem o
/// relógio, nem o quadro, nem a duração de uma composição têm máximo. O `f64::from(u16::MAX)`
/// que ficava aqui não era medido, era um número redondo servindo de cerca — e o
/// doc-comment acima já dizia `[0, ∞)`, ou seja o código contradizia a intenção escrita ao
/// lado dele.
///
/// ⚠️ **Ele mordia por DUAS vias, e a segunda é a que o artista sente:** o stepper PARAVA
/// em 65535, e o arrasto — o modelo de scrub deste app é *proporcional ao alcance*
/// (`DRAG_RANGE_PX_H` = 250 px varrem `[min, max]` inteiro) — valia **262 segundos por
/// pixel**. Não dava para pousar num número, e a caixa parecia recusar edição.
///
/// ⚠️ **O piso `0` e o `step` FICAM, e é por isso que o alcance não foi simplesmente
/// apagado:** o `step` registrado É o incremento do stepper (sem ele o dispatch cai numa
/// heurística de buffer — `0.01` para um valor com ponto —, e o clique de 0,2 s que o Enio
/// pediu para a Dur some em silêncio), e uma duração/tempo/quadro negativo não é uma coisa.
///
/// ⚠️ **Registrar os DOIS é correto, e o doc do store dizia o contrário:** ele mandava não
/// combinar `set_number_range` com `set_number_drag_rate`; o código faz o oposto do que
/// aquele texto prometia — o rate **vence** o modelo proporcional (`pointer_move.rs`) *e*
/// dispensa o clamp do arrasto. É essa precedência que deixa o alcance servir só de
/// `step` + piso enquanto o arrasto ganha uma escala em que dá para pousar. O texto do
/// store foi corrigido junto.
#[allow(clippy::too_many_arguments)]
pub(crate) fn chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    value: f64,
    step: f64,
    decimals: usize,
) -> f32 {
    let rect = Rect::new(x, y, CHIP_W, ROW_H_PX);
    {
        let store = ctx.host.store_mut();
        mirror_number(store, id, value, decimals);
        // Piso e `step` do stepper; teto ABERTO.
        store.set_number_range(id, 0.0, f64::INFINITY, step);
        // E a escala do arrasto, derivada do próprio incremento do chip: **um pixel vale um
        // clique de stepper**. Não é um número escolhido — é o mesmo `step` que a setinha
        // usa, então os três chips ficam coerentes sem ninguém calibrar nada à mão (Time
        // um quadro/px, Frame um quadro/px, Dur 0,2 s/px). Sem ele o alcance infinito
        // faria o modelo proporcional produzir um delta não-finito.
        store.set_number_drag_rate(id, step);
    }
    let (state, _v, buf, caret, anchor) = read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let input = NumberInput::new(id, "", value).step(step).state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    x + CHIP_W
}

/// Paint a `label | switch` toggle in the exact Widget-Gallery form: a label
/// column then a `TypeToken::Xl3`-wide, `Density::Compact`-tall pill switch
/// (proper 2:1 track + thumb), vertically centred in the row. Register the
/// switch hit. Returns the right edge.
pub(crate) fn toggle(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    text: &str,
    on: bool,
) -> f32 {
    let sw = TypeToken::Xl3.px();
    let sh = Density::Compact.row_h_px();
    let pad = Spacing::Xs.px();
    // Outlined cell grouping [label | switch] so each toggle is demarcated from
    // its neighbours (Enio 2026-07-08).
    let cell_w = pad + TOGGLE_LABEL_W + pad + sw + pad;
    let cell = Rect::new(x, y, cell_w, ROW_H_PX);
    stroke_rounded_rect(
        ctx.scene,
        cell,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    label(ctx, theme, text, x + pad, y, TOGGLE_LABEL_W);
    let sx = x + pad + TOGGLE_LABEL_W + pad;
    let rect = Rect::new(sx, y + (ROW_H_PX - sh) * 0.5, sw, sh);
    // Mirror the snapshot's on-state into the store (when not focused) so the
    // painted switch reflects the document and the edit baseline stays correct.
    {
        let store = ctx.host.store_mut();
        if store.focus_id() != Some(id)
            && let Some(InteractiveState::Toggle { on: store_on, .. }) = store.get_mut(id)
        {
            *store_on = on;
        }
    }
    let state = ctx
        .host
        .store()
        .toggle(id)
        .map(|(s, _)| s)
        .unwrap_or(ToggleState::Normal);
    // Gallery builder form (label rendered separately, above).
    let widget = Toggle::new(id, "").on(on).state(state);
    paint_toggle(&widget, rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, rect);
    x + cell_w
}

/// A short left-aligned, vertically-centred label of width `w`.
pub(crate) fn label(ctx: &mut PaintCtx, theme: Theme, text: &str, x: f32, y: f32, w: f32) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
}

/// Mirror a committed numeric `value` into the store's chip when it isn't being
/// edited (mirror of the Inspector/painter number-field sync).
pub(crate) fn mirror_number(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    value: f64,
    decimals: usize,
) {
    if store.focus_id() == Some(id) {
        return;
    }
    let text = if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.decimals$}")
    };
    if let Some(InteractiveState::NumberInput {
        value: v,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *v = value;
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = value;
    }
}
