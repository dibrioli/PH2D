//! ⭐⭐⭐ **A LINHA DE PROPRIEDADE DE SEMPRE** — rótulo `70` | trilha | caixa numérica `72`.
//!
//! Enio, 2026-09-03, ao mandar integrar: *«essa nova UI ainda deve ficar desativada até que esteja
//! concluída. Por enquanto permanece a antiga.»*
//!
//! ⚠️ **Este ficheiro é o caminho de OMISSÃO**, e o [`super::paint_slider_with_chip_layout`]
//! escolhe entre ele e a caixa única pela [`crate::paint::ui_look`]. ⛔ Ele não é código morto nem
//! histórico: é o que o app pinta para toda a gente que não liga `PH2D_UI_NEW=1`.
//!
//! ⚠️ **Ele foi RECUPERADO do commit `895d434a9^`**, não reescrito de memória — a linha tinha-o
//! substituído, e reescrever à mão o que o git guarda é a forma mais cara de introduzir uma
//! diferença que ninguém procura.

use super::paint_number_chip;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{paint_text, resolve};
use crate::widget::TextInputState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Pinta a linha CLÁSSICA. Assinatura idêntica à da caixa única, para o despacho ser um `if`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_classic_row(
    rect: Rect,
    label: &str,
    value: f32,
    chip_value: f64,
    display_override: Option<&str>,
    slider_id: NodeId,
    chip_id: NodeId,
    label_w: f32,
    chip_w: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let gap = Spacing::Sm.px();
    let track_x = rect.x + label_w + gap;
    let track_w = (rect.w - label_w - chip_w - gap * 2.0).max(1.0);
    let label_rect = Rect::new(rect.x, rect.y, label_w, rect.h);
    let track_rect = Rect::new(
        track_x,
        rect.y + Spacing::Sm.px(),
        track_w,
        rect.h - Spacing::Lg.px(),
    );
    let chip_rect = Rect::new(rect.x + rect.w - chip_w, rect.y, chip_w, rect.h);

    // Plain text label, LEFT-aligned (canon 2026-05-24, user:
    // "padrão para fonts das labels [...] deve ser como está no
    // painel grid settings"). Grid Settings rows use Base (13 px) +
    // Text1 + left-align via `paint_text`; mirroring here so every
    // row layout looks the same regardless of which painter renders it.
    let font = TypeToken::Base.px();
    paint_text(
        text_system,
        scene,
        label,
        label_rect.x,
        label_rect.y + (label_rect.h - font) * 0.5,
        font,
        label_rect.w,
        resolve(ColorToken::Text1, theme),
    );

    // Track background + filled portion — shared canonical painter so
    // this matches the bare `paint_slider` look exactly.
    //
    // ⚠️ **O par visual vem do STORE, que é o que o doc-header deste módulo já PROMETIA** (*"reads
    // the slider's state … straight from the WidgetStore"*) e não cumpria: o despachante escrevia
    // `Hovered`/`Dragging` ali desde sempre, esta linha nunca perguntou, e o `paint_slider_track`
    // não tinha por onde recebê-lo. É por aqui que os ~67 sítios de linha-com-chip do app ganham
    // a reacção sem tocar num deles.
    crate::widget::paint_slider_track(
        track_rect,
        value,
        crate::widget::SliderOrientation::Horizontal,
        store.slider_visual(slider_id),
        scene,
        theme,
    );
    if slider_id.0 != 0 {
        // The clickable / draggable zone is TALLER than the thin visual
        // track — it spans the full row height at the track's horizontal
        // span, so the slider is easy to grab (color-picker channel rows
        // were a ~10 px sliver). Same x/w as `track_rect`, so the
        // cursor→value mapping (horizontal) is unchanged; only the
        // vertical catch area grows. Stays within the track column (label
        // is left of `track_x`, chip is right of `track_x + track_w`), so
        // it never steals their clicks.
        let hit_rect = Rect::new(track_x, rect.y, track_w, rect.h);
        hit_index.register(slider_id, hit_rect);
    }

    // Chip — read its NumberInput state straight from the store so
    // typing / caret / selection are live.
    let (chip_state, chip_buffer, chip_caret, chip_anchor) = match store.get(chip_id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, Some(buffer.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    paint_number_chip(
        chip_rect,
        chip_state,
        chip_value,
        display_override,
        chip_buffer,
        chip_caret,
        chip_anchor,
        scene,
        text_system,
        theme,
    );
    if chip_id.0 != 0 {
        hit_index.register(chip_id, chip_rect);
    }
}
