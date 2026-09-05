//! **COMO a caixa única se desenha** — a superfície, o valor, o rótulo, o cursor.
//!
//! ⚠️ Cortado do `mod.rs` por **tecto de LOC** em 2026-09-03, e o corte é por responsabilidade:
//! ao lado mora *ONDE as coisas ficam* — [`super::surface_rect`], [`super::value_column`],
//! [`super::decorator_rect`], [`super::form_row_columns`] —, que é a metade que **outros widgets
//! e outros painéis** consultam. Aqui mora só a tinta.

use super::{
    PAD_UNITS_BETWEEN_LABEL_AND_VALUE, PropertyBox, PropertyBoxState, decorator_rect, fit_label,
    paint_decorator, surface_rect, value_column,
};
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, SliderDesign, SliderStyle, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **Pinta a caixa.** `style` é a aparência escolhida; `rect` é a linha inteira, decorator
/// incluído.
///
/// Devolve o rectângulo do **valor** — a região que aceita clique-para-escrever. Quem regista hits
/// precisa dela, e derivá-la duas vezes poria o alvo num sítio e a tinta noutro.
pub fn paint_property_box(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    rect: Rect,
    b: PropertyBox<'_>,
    style: SliderStyle,
) -> Rect {
    let t = b.t.clamp(0.0, 1.0);
    let disabled = b.state == PropertyBoxState::Disabled;
    let pad = Spacing::Md.px();

    // A coluna de animação sai da direita ANTES de tudo — ela é do FORMULÁRIO, não da caixa.
    // ⚠️ **Pela [`surface_rect`], não por uma conta local:** este rect é o que o preenchimento
    // atravessa, e quem regista o alvo de arrasto tem de registar EXACTAMENTE o mesmo. Ver lá o
    // mecanismo da deriva.
    // ⚠️ O mesmo: a coluna é do redesenho. O `paint_property_box` é chamado directamente pela
    // bancada e pela galeria — que a querem sempre —, então a guarda vive no `decorator` e não no
    // pintor inteiro.
    let decorator = b.decorator && crate::paint::ui_is_redesign();
    let box_rect = surface_rect(rect, decorator);
    let deco = decorator.then(|| decorator_rect(rect));

    let fg = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    let accent = if disabled {
        ColorToken::Border
    } else {
        b.accent
    };

    paint_surface(scene, theme, box_rect, t, b.state, accent, style);

    let size = TypeToken::Sm.px();
    let measured = text_system.layout(b.value, size, f32::INFINITY).width();
    let text_h = text_system.layout(b.value, size, f32::INFINITY).height();
    let ty = box_rect.y + (box_rect.h - text_h) * 0.5;
    // A coluna do valor: reservada pelo formulário, ou medida pelo texto da amostra.
    let value_w = b.value_w.unwrap_or(measured);

    // O valor, encostado à direita da coluna dele. NUNCA trunca.
    // ⚠️ **Pela LEI, não por uma conta paralela.** Esta linha já foi
    // `box_rect.x + box_rect.w - pad - value_w` — aritmética idêntica à da [`value_column`], e
    // portanto a segunda expressão para a mesma pergunta. *Duas contas que hoje concordam são duas
    // contas que amanhã divergem*, e o sítio onde isso apareceria seria o pior: o número pintado
    // num `x` e o alvo de clique noutro.
    let vx = value_column(rect, value_w, decorator).x;
    if !b.value.is_empty() {
        paint_text(
            text_system,
            scene,
            b.value,
            vx + (value_w - measured).max(0.0),
            ty,
            size,
            f32::INFINITY,
            resolve(fg, theme),
        );
    }

    // O rótulo, dentro à esquerda, com o que sobra depois do valor.
    let budget = (box_rect.w - pad * PAD_UNITS_BETWEEN_LABEL_AND_VALUE - value_w).max(0.0);
    let cut = fit_label(text_system, b.label, size, budget);
    if !cut.is_empty() {
        paint_text(
            text_system,
            scene,
            &cut,
            box_rect.x + pad,
            ty,
            size,
            f32::INFINITY,
            resolve(fg, theme),
        );
    }

    // ⚠️ O cursor é da AMOSTRA. Quando a coluna é reservada e o texto vem de fora (a linha do
    // produto), quem desenha o cursor — e a selecção, e o recorte — é o campo numérico real; dois
    // cursores na mesma caixa seria o pior tipo de duplicação, porque piscam em fase diferente.
    if b.state == PropertyBoxState::Editing && !b.value.is_empty() {
        let caret = Rect::new(
            vx + value_w + StrokeToken::Thin.px(),
            box_rect.y + Spacing::Xs.px(),
            StrokeToken::Thin.px(),
            (box_rect.h - Spacing::Md.px()).max(1.0),
        );
        fill_rounded_rect(scene, caret, 0.0, resolve(ColorToken::Accent, theme));
    }

    if let Some(d) = deco {
        paint_decorator(scene, theme, d, disabled);
    }

    // **A coluna do valor** — pela LEI, não por uma segunda conta. Ver [`value_column`].
    value_column(rect, value_w, decorator)
}

/// A superfície — é aqui que os quatro desenhos divergem, e **só** aqui.
fn paint_surface(
    scene: &mut VectorScene,
    theme: Theme,
    r: Rect,
    t: f32,
    state: PropertyBoxState,
    accent: ColorToken,
    style: SliderStyle,
) {
    let rad = style.radius_px();
    let hot = matches!(
        state,
        PropertyBoxState::Hovered | PropertyBoxState::Dragging
    );
    let trough = if hot {
        ColorToken::Bg2
    } else {
        ColorToken::Bg3
    };
    let fill_w = (r.w * t).max(0.0);
    // A espessura da linha de valor do `Underline`. `Thick` é o token de 2 px.
    let base_h = StrokeToken::Thick.px();

    match style.design {
        SliderDesign::Underline => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            let base = Rect::new(r.x, r.y + r.h - base_h, r.w, base_h);
            fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));
            if fill_w > 0.5 {
                let f = Rect::new(r.x, r.y + r.h - base_h, fill_w, base_h);
                fill_rounded_rect(scene, f, 0.0, resolve(accent, theme));
            }
        }
        SliderDesign::Bar => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            if fill_w > 0.5 {
                fill_rounded_rect(
                    scene,
                    Rect::new(r.x, r.y, fill_w, r.h),
                    rad,
                    resolve(accent, theme),
                );
            }
        }
        SliderDesign::Inset => {
            fill_rounded_rect(scene, r, rad, resolve(trough, theme));
            let m = Spacing::Xxs.px();
            let inner = Rect::new(
                r.x + m,
                r.y + m,
                (r.w - m * 2.0).max(0.0),
                (r.h - m * 2.0).max(0.0),
            );
            let iw = inner.w * t;
            if iw > 0.5 {
                fill_rounded_rect(
                    scene,
                    Rect::new(inner.x, inner.y, iw, inner.h),
                    (rad - m).max(0.0),
                    resolve(accent, theme),
                );
            }
        }
        SliderDesign::Ghost => {
            if fill_w > 0.5 {
                fill_rounded_rect(
                    scene,
                    Rect::new(r.x, r.y, fill_w, r.h),
                    rad,
                    resolve(ColorToken::AccentSoft, theme),
                );
            }
            let h = StrokeToken::Thin.px();
            let base = Rect::new(r.x, r.y + r.h - h, r.w, h);
            fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));
        }
    }

    // ⭐ Pela porta do TEMA: num tema moderno a edição continua a ter o anel (é foco), e o hover
    //    deixa de ter contorno — o fundo já sobe um degrau.
    if state == PropertyBoxState::Editing {
        crate::paint::stroke_frame(
            scene,
            r,
            rad,
            theme,
            ph2d_tokens::visuals::Feel::Focused,
            StrokeToken::Thin.px(),
            resolve(ColorToken::Accent, theme),
        );
    } else if state == PropertyBoxState::Hovered {
        crate::paint::stroke_frame(
            scene,
            r,
            rad,
            theme,
            ph2d_tokens::visuals::Feel::Hovered,
            StrokeToken::Hairline.px(),
            resolve(ColorToken::BorderStrong, theme),
        );
    }
}
