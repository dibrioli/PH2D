//! ⭐⭐⭐ **O INTERRUPTOR DESLIZANTE de sempre** — corpo em pílula + disco que corre.
//!
//! Enio, 2026-09-03, ao mandar integrar: *«essa nova UI ainda deve ficar desativada até que esteja
//! concluída. Por enquanto permanece a antiga.»*
//!
//! ⚠️ **Este é o caminho de OMISSÃO.** O [`super::paint_toggle`] escolhe entre ele e a marca da
//! caixa de verificação pela [`crate::paint::ui_look`]. ⛔ Não é histórico: é o que o app pinta
//! para quem não liga `PH2D_UI_NEW=1`.
//!
//! ⚠️ **RECUPERADO do commit `a7c077804^`**, não reescrito — o `thumb_circle` traz consigo as duas
//! leis que ele já tinha pago: numa moldura quadrada o curso é **zero** (ON e OFF no mesmo pixel) e
//! numa moldura em pé ele seria **negativo**, com a semântica invertida e o disco fora do corpo.

use super::{Toggle, ToggleState};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

/// Pinta o interruptor CLÁSSICO.
pub(super) fn paint_classic_toggle(
    toggle: &Toggle,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = Radius::Full.px();
    let body_token = match (toggle.on, toggle.state) {
        // Disabled + on stays on a muted accent so users can still
        // read the current value at a glance; disabled + off keeps
        // the neutral Border fill. Previously both collapsed to
        // Border which read as "off, locked" regardless of the
        // actual `on` flag.
        (true, ToggleState::Disabled) => ColorToken::AccentSoft,
        (false, ToggleState::Disabled) => ColorToken::Border,
        (true, _) => ColorToken::Accent,
        (false, _) => ColorToken::Bg2,
    };
    fill_rounded_rect(scene, rect, radius, resolve(body_token, theme));

    // Subtle border emphasis on Hover/Focus so the off-state pill
    // doesn't look static when the cursor is over it.
    // ⚠️ **Aqui o eixo é uma PRESENÇA, não um par de cores:** em repouso não há traço nenhum, e o
    //    do hover tem de EMERGIR do nada — é o caso `(None, Some(hot))` da `blend_token_color`, o
    //    mesmo que o botão `Default` já usava. O `Focused` fica de fora (estado duro, traço de
    //    2 px), e o neutro `SETTLED` devolve o traço cheio de sempre.
    let soft = matches!(toggle.state, ToggleState::Normal | ToggleState::Hovered);
    let emph = crate::motion::hover_axis(
        soft,
        toggle.hover_t,
        None,
        Some(ColorToken::BorderEmph.resolve(theme)),
    );
    if let Some(c) = emph {
        stroke_rounded_rect(scene, rect, radius, 1.0, crate::paint::token_to_vello(c));
    } else if matches!(toggle.state, ToggleState::Hovered | ToggleState::Focused) {
        let stroke_w = if toggle.state == ToggleState::Focused {
            2.0
        } else {
            1.0
        };
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            stroke_w,
            resolve(ColorToken::BorderEmph, theme),
        );
    }

    // Circular thumb. Diameter = body height - 2*pad. Off → left,
    // On → right. Disabled tints to Text3 so it stays visible but mute.
    let (cx, cy, r) = thumb_circle(rect, toggle.on);
    let thumb_token = if toggle.state == ToggleState::Disabled {
        ColorToken::Text3
    } else if toggle.on {
        ColorToken::AccentFg
    } else {
        ColorToken::Text2
    };
    let thumb = Circle::new(Point::new(cx as f64, cy as f64), r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(thumb_token, theme)),
        None,
        &thumb,
    );
}

/// Pill body + circular thumb (left=off, right=on). Focus ring is
/// **O disco do thumb** — centro e raio, e ele NUNCA sai do corpo.
///
/// ⚠️ **O curso de um toggle é exatamente `w − h`**, porque o thumb é dimensionado pela ALTURA e
/// posicionado pela LARGURA. Nada perguntava se sobrava largura, e o preço era medido: numa
/// moldura **quadrada** o curso é **zero** — ON e OFF pintam o disco no MESMO pixel, um controle
/// que não consegue mostrar o próprio estado; e numa moldura **em pé** (30×60) o curso é
/// **−30**, ou seja a semântica **INVERTE** e o disco cai **24 px para fora do corpo**, 80% do
/// diâmetro, nos dois estados.
///
/// ⚠️ **Isto deixou de ser hipotético quando a pele chegou:** o `widget_live::frame_of` entrega a
/// bbox da forma que o ARTISTA desenhou, e o único guard dela é `w > 1 && h > 1`.
///
/// A lei é a do `glyph_box`: a caixa mede pelo lado que a contém. O diâmetro sai de
/// `min(h, w)` e o centro é clampado ao corpo — e ⚠️ **para todo `w >= h` isto REDUZ à expressão
/// que shipava, termo a termo** (`min(h,w)` é `h`, e o clamp é no-op porque `pad >= 0`), que é o
/// que mantém todos os chamadores de hoje byte-idênticos.
///
/// Um toggle mais alto que largo continua **mudo**, e isso é honesto: o curso de um toggle é
/// horizontal, e ali não há largura para percorrer. O que ele deixa de fazer é pintar fora de si
/// mesmo e dizer o contrário do que vale.
#[must_use]
pub fn thumb_circle(rect: Rect, on: bool) -> (f32, f32, f32) {
    let pad = (rect.h * 0.15).clamp(2.0, 6.0); // LITERAL-PX-OK: toggle inner pad 15% of body height with min/max
    // ⚠️ O piso de `Xs` vem do desenho original (um disco não pode sumir num toggle normal), mas
    // ele **não pode vencer o corpo**: num corpo de 2 px ele sozinho põe 1 px de disco para fora.
    // O `.min(w)` é o que torna a contenção incondicional — e para `w >= h` ele é no-op, porque
    // `h - 2·pad < h <= w`.
    let diameter = (rect.h.min(rect.w) - pad * 2.0)
        .max(Spacing::Xs.px())
        .min(rect.w.max(0.0));
    let r = diameter * 0.5;
    let cy = rect.y + rect.h * 0.5;
    // ⚠️ **O CURSO é a lei, e ele pode ser negativo:** o `pad` sai da ALTURA e é subtraído da
    // LARGURA, então numa moldura estreita e alta os dois extremos **se cruzam** — ON à esquerda de
    // OFF, a semântica ao contrário. Sem largura para percorrer, os dois estados colapsam no
    // centro: mudo, mas nunca mentindo.
    let travel = rect.w - pad * 2.0 - diameter;
    let cx = if travel <= 0.0 {
        rect.x + rect.w * 0.5
    } else if on {
        rect.x + rect.w - pad - r
    } else {
        rect.x + pad + r
    };
    // O clamp é a GARANTIA, e não o cálculo: mesmo com um `pad` grande num corpo minúsculo, o
    // disco fica dentro. ⚠️ É o `safe_clamp` do repo, e não o da `std`, porque o `arch_safe_clamp`
    // recusa bounds variáveis — aqui o ramo de troca dele é **inalcançável** (o `.min(w)` do
    // diâmetro garante `r <= w/2`, logo `lo <= hi`), e o que ele acrescenta de facto é a recusa do
    // NaN, que numa bbox autorada não é hipótese ociosa.
    let cx = crate::math::safe_clamp(cx, rect.x + r, rect.x + rect.w - r);
    (cx, cy, r)
}
