//! **O PINO da preview viva do pincel** (dono 22/10: *«um ícone de pino para fixar o preview (não rolar com
//! o scroll)»*) — o «pin» de painel do Krita/Blender/Procreate.
//!
//! - Desafixado (omissão): a preview pinta-se no cartão do topo e rola com o corpo, como sempre.
//! - Fixado: a preview sai do corpo e pinta-se numa FAIXA por cima da área rolável, que passa a começar
//!   por baixo dela ([`crate::paint`] `paint_brush_view`): o corpo rola por baixo e nada fica escondido
//!   atrás dela (o recorte do desenho e do clique é o da área).
//!
//! O estado é um `Toggle` do store (o despacho genérico vira-o no clique e no Enter/Espaço) e vive o
//! mesmo tempo que a dobra das secções: a sessão. O tool nunca o vê.

use crate::state_wet_presets as st;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonState, IconButtonStyle, IconGlyph, ToggleState, paint_icon_button,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids::PAINTER_BRUSH_PREVIEW_PIN as PINO;
use ph2d_vector::ImageQuality;
use std::cell::Cell;

thread_local! {
    /// Onde a preview caiu na última pintura da vista Brush (`None` = não pintou).
    static RECT_DA_PREVIEW: Cell<Option<Rect>> = const { Cell::new(None) };
}

/// O pino no store — `register_if_absent`: um `populate` que volte a correr não desafixa.
pub(crate) fn regista(store: &mut WidgetStore) {
    store.register_if_absent(
        PINO,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
    );
}

/// A preview está fixada?
#[must_use]
pub fn preview_fixada(store: &WidgetStore) -> bool {
    store.toggle(PINO).is_some_and(|(_, on)| on)
}

/// Onde a preview caiu na última pintura da vista Brush, no ECRÃ.
#[must_use]
pub fn rect_da_preview() -> Option<Rect> {
    RECT_DA_PREVIEW.with(Cell::get)
}

/// O início de cada pintura da vista Brush: quem pintar a preview volta a dizer onde.
pub(crate) fn esquece() {
    RECT_DA_PREVIEW.with(|c| c.set(None));
}

/// Há quadro publicado para mostrar?
pub(crate) fn ha_preview() -> bool {
    st::current_wet_preview().is_some_and(|(_, w, _)| w > 0)
}

/// Pinta a preview de `x` a `x + w` a partir de `y`, com o pino por cima; devolve o `y` de baixo da
/// imagem (`None` sem quadro publicado).
pub(crate) fn pinta(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
) -> Option<f32> {
    let (px, pw, ph) = st::current_wet_preview().filter(|(_, pw, _)| *pw > 0)?;
    let alto = w * ph as f32 / pw as f32;
    ctx.scene.draw_image_rgba(
        &px,
        pw,
        ph,
        (
            f64::from(x),
            f64::from(y),
            f64::from(x + w),
            f64::from(y + alto),
        ),
        ImageQuality::Medium,
    );
    let r = Rect::new(x, y, w, alto);
    RECT_DA_PREVIEW.with(|c| c.set(Some(r)));
    pinta_pino(ctx, theme, r);
    Some(y + alto)
}

/// O rect do pino: o canto superior direito da preview, um botão de ícone recuado `Xs` da borda.
#[must_use]
pub fn rect_do_pino(preview: Rect) -> Rect {
    let m = ph2d_tokens::Spacing::Xs.px();
    let lado = ph2d_tokens::ICON_BTN_SIZE_PX
        .min(preview.h - m - m)
        .min(preview.w - m - m)
        .max(0.0);
    Rect::new(preview.x + preview.w - lado - m, preview.y + m, lado, lado)
}

fn dica(fixada: bool) -> &'static str {
    tr(if fixada {
        "panel.painter_preview.unpin"
    } else {
        "panel.painter_preview.pin"
    })
}

/// Pinta e regista o pino — o rect que se pinta é o que se clica.
fn pinta_pino(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, preview: Rect) {
    let r = rect_do_pino(preview);
    let fixada = preview_fixada(ctx.host.store());
    let (estado, t) = ctx.host.store().toggle_visual(PINO);
    // Fixado = o glifo aceso (`Accent`), como um botão premido; desafixado segue o rato.
    let visual = if fixada {
        (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
    } else {
        let e = match estado {
            ToggleState::Normal => ButtonState::Normal,
            ToggleState::Hovered => ButtonState::Hovered,
            ToggleState::Pressed => ButtonState::Pressed,
            ToggleState::Focused => ButtonState::Focused,
            ToggleState::Disabled => ButtonState::Disabled,
        };
        (e, t)
    };
    paint_icon_button(
        r,
        IconGlyph::Builtin(IconId::Pin),
        IconButtonStyle::Compact,
        visual,
        ctx.scene,
        theme,
    );
    ctx.host.store_mut().set_tooltip(PINO, dica(fixada));
    ctx.host.hit_index_mut().register(PINO, r);
}

/// O nó de AccessKit do pino (HR-12): um botão de ALTERNAR com o estado `pressed` e a dica como nome.
#[must_use]
pub fn a11y_do_pino(store: &WidgetStore, rect: Rect) -> ph2d_a11y::Node {
    let fixada = preview_fixada(store);
    Button::new(PINO, dica(fixada))
        .icon_only(IconId::Pin)
        .build_a11y_toggle(
            fixada,
            f64::from(rect.x),
            f64::from(rect.y),
            f64::from(rect.w),
            f64::from(rect.h),
        )
}
