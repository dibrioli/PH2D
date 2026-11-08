//! **O CARTÃO de um grupo**, pintado — fundo, título com a contagem, filete, sub-títulos e pílulas.
//!
//! Irmão de [`super`] pelo tecto de 500 LOC dos primitivos; as medidas e os pesos vêm do
//! [`Look`] (o de sempre, ou o da referência no Add Object).

use super::layout::{badge_font, badge_gap, badge_size};
use super::look::Look;
use super::{CardLayout, Placed};
use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, paint_text, paint_text_weighted, resolve};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// Paint one category card: a rounded `Bg2` box + `Border`, then its header (dot + name + count +
/// coloured rule), the grey sub-cluster headers, and the pills (each registering its hit rect).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_card(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    cl: &CardLayout,
    ox: f32,
    oy: f32,
    card_w: f32,
    rise: f32,
    [hot, chosen]: [Option<NodeId>; 2],
    lk: &Look,
) {
    // ⚠️ `oy` é a posição ASSENTE — a que o dedo procura — e `dy` a que o olho vê durante a
    //    entrada. Tudo o que DESENHA lê `dy`; o único `register` deste corpo lê `oy`.
    let dy = oy + rise;
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    let card = Rect::new(ox, dy, card_w, cl.height);
    let superficie = Superficie::de(theme);
    let ink = superficie.tinta();
    if let Some(ink) = ink {
        let fundo = ink.color(ColorToken::KanbanCard);
        fill_rounded_rect(scene, card, m::CARD_RADIUS, fundo);
    } else {
        fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg2, theme));
        crate::paint::stroke_frame(
            scene,
            card,
            radius,
            theme,
            ph2d_tokens::visuals::Feel::Rest,
            1.0,
            resolve(ColorToken::Border, theme),
        );
    }

    let cat_color = resolve(cl.color, theme);
    let sm = TypeToken::Sm.px();
    let xs = TypeToken::Xs.px();
    let (pad, dot_r, pad_x) = (lk.card_pad_x, lk.dot_r, lk.pill_pad_x);
    let content_w = card_w - pad * 2.0;
    for p in &cl.placed {
        match p {
            Placed::Header { title, count, y } => {
                let hx = ox + pad;
                let hy = dy + *y;
                let ty = hy
                    + if ink.is_some() {
                        superficie.folga(sm, sm)
                    } else {
                        0.0
                    };
                if lk.count_right {
                    // ⭐ A referência: o nome na cor do grupo, a contagem em cinza encostada à direita.
                    let n = count.to_string();
                    let n_w = ts.prefix_width_weighted(&n, sm, FontWeight::NORMAL);
                    let gap = Spacing::Sm.px();
                    let t_w = (content_w - n_w - gap).max(0.0);
                    paint_text(ts, scene, title, hx, ty, sm, t_w, cat_color);
                    let n_x = hx + content_w - n_w;
                    let cinza = superficie.cor(ColorToken::Text3);
                    let _ = paint_text_weighted(
                        ts,
                        scene,
                        &n,
                        n_x,
                        ty,
                        sm,
                        n_w + 1.0, // LITERAL-PX-OK: folga de arredondamento para o número não quebrar
                        cinza,
                        FontWeight::NORMAL,
                    );
                } else {
                    dot(scene, hx + dot_r, hy + sm * 0.5, dot_r, cat_color);
                    let label = format!("{title} \u{00b7} {count}");
                    let tx = hx + dot_r * 2.0 + Spacing::Xs.px();
                    paint_text(ts, scene, &label, tx, ty, sm, content_w, cat_color);
                }
                // Coloured underline rule across the card content width.
                fill_rounded_rect(
                    scene,
                    Rect::new(hx, hy + sm + lk.rule_gap, content_w, lk.rule_h),
                    lk.rule_h * 0.5,
                    cat_color,
                );
            }
            Placed::Sub { title, x, y } => {
                paint_text(
                    ts,
                    scene,
                    title,
                    ox + *x,
                    dy + *y,
                    xs,
                    content_w,
                    superficie.cor(ColorToken::Text3),
                );
            }
            Placed::Pill {
                rect,
                id,
                label,
                badge,
            } => {
                let r = Rect::new(ox + rect.x, dy + rect.y, rect.w, rect.h);
                let fg = match ink {
                    _ if chosen == Some(*id) => superficie.botao(scene, r, true, false),
                    Some(ink) => {
                        let tom = if hot == Some(*id) {
                            ColorToken::CardFieldHover
                        } else {
                            ColorToken::CardField
                        };
                        fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(tom));
                        superficie.cor(ColorToken::Text1)
                    }
                    None => {
                        let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
                        fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg3, theme));
                        superficie.cor(ColorToken::Text1)
                    }
                };
                dot(
                    scene,
                    r.x + pad_x + dot_r,
                    r.y + r.h * 0.5,
                    dot_r,
                    cat_color,
                );
                let ink_px = lk.pill_ink;
                let (lx, ly) = (
                    r.x + pad_x + dot_r * 2.0 + Spacing::Xs.px(),
                    r.y + superficie.folga(r.h, ink_px),
                );
                let chip = badge.as_deref().map(|b| (b, badge_size(ts, b)));
                let reserva = chip.map_or(0.0, |(_, (w, _))| badge_gap() + w);
                let lw = r.w - pad_x * 2.0 - dot_r * 2.0 - reserva;
                if let Some((b, (w, h))) = chip {
                    paint_badge(
                        scene,
                        ts,
                        b,
                        r.x + r.w - pad_x - w,
                        r.y,
                        [w, h, r.h],
                        &superficie,
                    );
                }
                // O peso de sempre pinta pela porta de sempre (que corta numa linha); o da
                // referência pede o seu peso — o rótulo foi MEDIDO nele, e cabe.
                if lk.pill_weight == FontWeight::MEDIUM {
                    paint_text(ts, scene, label, lx, ly, ink_px, lw, fg);
                } else {
                    let w = lw + Spacing::Xs.px();
                    let _ = paint_text_weighted(
                        ts,
                        scene,
                        label,
                        lx,
                        ly,
                        ink_px,
                        w,
                        fg,
                        lk.pill_weight,
                    );
                }
                // ⚠️ ASSENTE, nunca `r`: durante a entrada o cartão está 12 px abaixo, e um
                //    clique apressado cairia na row de cima.
                hit_index.register(*id, Rect::new(ox + rect.x, oy + rect.y, rect.w, rect.h));
            }
        }
    }
}

/// ⭐ **A plaquinha** («+3», maquete B `.bri`): uma pílula cinza (a cor do texto apagado) com o
/// texto na cor do fundo do painel, em negrito pequeno, centrada na altura `ph` da pílula do item.
fn paint_badge(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &str,
    x: f32,
    py: f32,
    [w, h, ph]: [f32; 3],
    superficie: &Superficie,
) {
    let r = Rect::new(x, py + (ph - h) * 0.5, w, h);
    fill_rounded_rect(scene, r, h * 0.5, superficie.cor(ColorToken::Text3));
    let tinta = match superficie.tinta() {
        Some(ink) => ink.color(ColorToken::KanbanList),
        None => superficie.cor(ColorToken::BgElev),
    };
    let (px, weight) = badge_font();
    let _ = paint_text_weighted(
        ts,
        scene,
        text,
        r.x + Spacing::Xs.px(),
        r.y + superficie.folga(h, px),
        px,
        w,
        tinta,
        weight,
    );
}

/// A small filled category dot of radius `r` centred at `(cx, cy)`.
fn dot(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, color: Color) {
    fill_rounded_rect(scene, Rect::new(cx - r, cy - r, r * 2.0, r * 2.0), r, color);
}
