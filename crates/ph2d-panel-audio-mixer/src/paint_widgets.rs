//! Shared widget-row painters for the Audio Mixer panel — a labeled slider row
//! (master-fx params: EQ, reverb Size/Return, sends, ducking Depth) and a toggle
//! button (mute / solo / effect enables). Split out of `paint.rs` to keep the
//! paint orchestrator under the panel LOC cap.
//!
//! ⚠️ **O doc que esta linha substitui MENTIA**, e a mentira era o defeito: ele dizia que os dois
//! eram *«leaf helpers over the canonical gallery widgets (no bespoke chrome)»*. O slider é — ele
//! chama `paint_slider`; o **toggle não**: ele pinta `Bg3`/`active_bg` à mão. E nenhum dos dois
//! perguntava ao store, então os dois eram **inertes sob o rato** (o mesmo mecanismo que o painel
//! irmão, o Audio Editor, pagou em 2026-08-15: os ids registados, o store a saber, ninguém a
//! perguntar).
//!
//! ⚠️ **Os dois recebem o `WidgetStore` e respondem por si**, em vez de receberem o par visual. É
//! deliberadamente mais forte que a porta `visual(pair)` do catálogo: o pintor **já tem o id**, e
//! derivar as duas metades dele torna o par-descasado inexprimível.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::motion::{self, hover_of, pressed_of};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor_core::widget::{ButtonState, Slider, SliderOrientation, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

const FX_LABEL_W: f32 = 32.0; // LITERAL-PX-OK: master-fx label column width (chrome)

/// Paint a small left label + a full-width horizontal Slider on one row (the
/// master-fx parameter rows: EQ, reverb Size/Return, sends, ducking Depth).
/// Returns the next y.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_labeled_slider(
    y: f32,
    label: &str,
    id: NodeId,
    value: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
) -> f32 {
    let label_rect = Rect::new(content_x, y, FX_LABEL_W, Spacing::Md.px());
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    let slider_x = content_x + FX_LABEL_W + Spacing::Sm.px();
    let slider_w = (content_w - FX_LABEL_W - Spacing::Sm.px()).max(1.0);
    let slider_rect = Rect::new(slider_x, y, slider_w, Spacing::Md.px());
    let mut slider = Slider::new(id, label)
        .orientation(SliderOrientation::Horizontal)
        .visual(store.slider_visual(id));
    slider.set_value(value.clamp(0.0, 1.0));
    paint_slider(&slider, slider_rect, scene, theme);
    hit_index.register(id, slider_rect);
    y + Spacing::Md.px() + Spacing::Sm.px()
}

/// Paint one toggle button (mute / solo / effect enable): `active_bg` tint +
/// `AccentFg` text when engaged, else `Bg3` + `Text1`. Registers `id` as the hit
/// rect.
///
/// ⚠️ **O tom QUENTE é derivado do de repouso, não escolhido** — e aqui isso não é conveniência,
/// é a única resposta possível: o `active_bg` é um PARÂMETRO (`Danger` no Mute, `Warn` no Solo,
/// `Accent` nas master-fx), então uma tabela de pares repouso→quente teria de crescer com cada
/// chamador novo. `ColorToken::hover_of` responde pela FAMÍLIA.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_toggle(
    rect: Rect,
    label: &str,
    active: bool,
    active_bg: ColorToken,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
) {
    let (rest, fg) = if active {
        (active_bg, ColorToken::AccentFg)
    } else {
        (ColorToken::Bg3, ColorToken::Text1)
    };
    let (state, t) = store.button_visual(id);
    let bg = if state == ButtonState::Pressed {
        resolve(pressed_of(rest), theme)
    } else {
        let soft = matches!(state, ButtonState::Normal | ButtonState::Hovered);
        let hot = hover_of(rest);
        motion::hover_axis(soft, t, Some(rest.resolve(theme)), Some(hot.resolve(theme)))
            .map_or_else(
                || {
                    resolve(
                        if state == ButtonState::Hovered {
                            hot
                        } else {
                            rest
                        },
                        theme,
                    )
                },
                |c| VelloColor::from_rgba8(c.r, c.g, c.b, c.a), // LITERAL-COLOR-OK: token-bridge
            )
    };
    fill_rounded_rect(
        scene,
        rect,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        bg,
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    hit_index.register(id, rect);
}
