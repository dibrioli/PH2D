//! Selection overlay painter — dashed marquee + 4 corner handles +
//! floating tag above the marquee.

use super::HeroLayout;
use super::HeroSelection;
use crate::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, rect_to_vello, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Stroke, VectorScene};

pub fn paint_selection_overlay(
    layout: &HeroLayout,
    selection: &HeroSelection,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let marquee_w = (layout.canvas.w * 0.55).clamp(280.0, 520.0); // LITERAL-PX-OK: mockup marquee geometry (canvas-ratio + min/max chrome)
    let marquee_h = (layout.canvas.h * 0.5).clamp(220.0, 440.0); // LITERAL-PX-OK: mockup marquee geometry
    let cx = layout.canvas.x + layout.canvas.w * 0.5;
    let cy = layout.canvas.y + layout.canvas.h * 0.5;
    let marquee = Rect::new(
        cx - marquee_w * 0.5,
        cy - marquee_h * 0.5,
        marquee_w,
        marquee_h,
    );
    let stroke = Stroke::new(1.0).with_dashes(0.0, [6.0, 4.0]); // LITERAL-PX-OK: marching-ants dash pattern (gap+dash design-specific)
    let r = rect_to_vello(marquee);
    scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &r,
    );
    let handle = Spacing::Md.px();
    for (hx, hy) in [
        (marquee.x - handle * 0.5, marquee.y - handle * 0.5),
        (
            marquee.x + marquee.w - handle * 0.5,
            marquee.y - handle * 0.5,
        ),
        (
            marquee.x - handle * 0.5,
            marquee.y + marquee.h - handle * 0.5,
        ),
        (
            marquee.x + marquee.w - handle * 0.5,
            marquee.y + marquee.h - handle * 0.5,
        ),
    ] {
        let h_rect = Rect::new(hx, hy, handle, handle);
        fill_rounded_rect(scene, h_rect, 1.0, resolve(ColorToken::Bg0, theme));
        // FRAME-RAW-OK: o marquee de seleccao sobre o CANVAS: o contorno E' o significado
        stroke_rounded_rect(scene, h_rect, 1.0, 2.0, resolve(ColorToken::Accent, theme));
    }
    let tag_w = 220.0_f32; // LITERAL-PX-OK: selection tag width (chrome-specific)
    let tag_h = 22.0_f32; // LITERAL-PX-OK: selection tag height (chrome-specific compact)
    let tag_rect = Rect::new(
        marquee.x,
        marquee.y - tag_h - Spacing::Sm.px(),
        tag_w,
        tag_h,
    );
    fill_rounded_rect(
        scene,
        tag_rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    // FRAME-RAW-OK: o marquee de seleccao sobre o CANVAS: o contorno E' o significado
    stroke_rounded_rect(
        scene,
        tag_rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let pad = Spacing::Md.px();
    let label_y = tag_rect.y + (tag_rect.h - TypeToken::Xs.px()) * 0.5;
    let (name_col, badge_x, badge_w, pos_col) = tag_columns(tag_rect, pad);
    paint_text(
        text_system,
        scene,
        &selection.label,
        tag_rect.x + pad,
        label_y,
        TypeToken::Xs.px(),
        name_col,
        resolve(ColorToken::Text1, theme),
    );
    let badge_rect = Rect::new(
        badge_x,
        tag_rect.y + Spacing::Xs.px(),
        badge_w,
        tag_rect.h - Spacing::Md.px(),
    );
    fill_rounded_rect(
        scene,
        badge_rect,
        Radius::Xs.px(),
        resolve(ColorToken::AccentSoft, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        &selection.kind,
        badge_rect,
        TypeToken::Xs.px() - 2.0,
        resolve(ColorToken::Accent, theme),
    );
    let pos_text = format!(
        "\u{00b7} {:.0}, {:.0}",
        selection.world_pos.0, selection.world_pos.1
    );
    paint_text(
        text_system,
        scene,
        &pos_text,
        badge_x + badge_w + Spacing::Md.px(),
        label_y,
        TypeToken::Xs.px(),
        pos_col,
        resolve(ColorToken::Text3, theme),
    );
}

/// ⭐⭐⭐ **AS TRÊS COLUNAS DA ETIQUETA DE SELECÇÃO — e o orçamento de cada texto É a coluna dele.**
///
/// A etiqueta é `nome · [EMBLEMA] · x, y` dentro de uma caixa de largura FIXA (`tag_w`), e as três
/// peças eram posicionadas por uns números e orçadas por outros. Medido em 2026-09-19:
///
/// | peça | espaço REAL | orçamento que ela recebia |
/// |---|---:|---:|
/// | nome | **`60,0`** até ao emblema | **`80,0`** |
/// | posição | **`104,0`** até ao recuo direito | `100,0` |
///
/// ⛔⛔ **O nome invadia o emblema, e a reticência nunca disparava:** `Platform Player` mede
/// `79,88` e cabia nos `80` que lhe eram dados — logo era **desenhado inteiro, `19,88 px` por
/// baixo do emblema**. *Um orçamento maior que a coluna não corta texto nenhum: ele empurra as
/// letras para cima do vizinho, que é o pior dos dois resultados possíveis* — com a coluna certa o
/// artista lê `Platform Pl…` e sabe que há mais nome.
///
/// ⚠️ **A GEOMETRIA não se move nem um pixel:** o emblema continua a começar em `pad + 60` e o
/// texto de posição onde sempre esteve. O que muda são os dois ORÇAMENTOS, que passam a ser as
/// colunas — e é por isso que a cura é segura sobre uma etiqueta que já shipava.
///
/// ⛔ **As colunas são FIXAS de propósito:** dar ao nome o que sobra faria o emblema **dançar** a
/// cada objecto escolhido, e o texto de posição muda quando o objecto se MOVE — a etiqueta
/// tremeria a arrastar. *Uma coluna elástica é certa numa linha de formulário e errada numa
/// etiqueta que paira sobre o canvas.*
fn tag_columns(tag: Rect, pad: f32) -> (f32, f32, f32, f32) {
    // A distância do início do nome ao início do emblema — a geometria que esta etiqueta já tinha.
    const NAME_TO_BADGE: f32 = 60.0; // LITERAL-PX-OK: selection-tag column (chrome geometry)
    let gap = Spacing::Sm.px();
    let badge_x = tag.x + pad + NAME_TO_BADGE;
    let badge_w = Spacing::Xl3.px();
    let pos_x = badge_x + badge_w + Spacing::Md.px();
    (
        // O nome pára um VÃO antes do emblema — é essa a coluna dele, e é esse o orçamento.
        (NAME_TO_BADGE - gap).max(0.0),
        badge_x,
        badge_w,
        // A posição fica com o que sobra até ao recuo do lado direito.
        (tag.x + tag.w - pad - pos_x).max(0.0),
    )
}
