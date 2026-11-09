//! ⭐ **O mapa «Where»** (dono 09/11, «mapa na Bancada», ligado por omissão; View ▸ Scene Map): a
//! cena em miniatura ao lado da grelha — só o contorno de cada objecto que corre o script, na cor
//! dele, como o canvas o mostra (a shell entrega os anéis já pela câmara do canvas). O objecto sob o
//! rato (aqui, na grelha, no balão ou no Console) acende; o clique selecciona-o.

use super::{Aim, BancadaView, Who};
use crate::chrome::{self, Ui};
use ph2d_editor_core::paint::{fill_circle, rect_to_vello, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, LAYER_OUTLINE_OFF_ALPHA, ROW_H_PX, ROW_SELECTED_ALPHA, Spacing, StrokeToken, Theme,
};
use ph2d_vector::{Affine, BezPath, Brush, Stroke, VectorScene};

/// A caixa de todos os pontos dos anéis (mín. x, mín. y, máx. x, máx. y), ou `None` sem pontos.
fn bounds(who: &Who) -> Option<[f32; 4]> {
    let mut it = who.map.iter().flat_map(|(_, rings)| rings.iter().flatten());
    let first = it.next()?;
    Some(it.fold([first[0], first[1], first[0], first[1]], |b, p| {
        [
            b[0].min(p[0]),
            b[1].min(p[1]),
            b[2].max(p[0]),
            b[3].max(p[1]),
        ]
    }))
}

impl BancadaView {
    pub(super) fn paint_map(
        &mut self,
        who: &Who,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        scene.fill_rect(rect_to_vello(area), ui.bg);
        let s = Spacing::Sm.px();
        let w = (area.w - s * 2.0).max(0.0);
        super::outline::card_title(
            scene,
            ts,
            tr("code.bancada.map.title"),
            Rect::new(area.x + s, area.y + s, w, ROW_H_PX),
            *theme,
        );
        let inner = Rect::new(
            area.x + s,
            area.y + s + ROW_H_PX,
            w,
            (area.h - s * 2.0 - ROW_H_PX).max(0.0),
        );
        let Some(b) = bounds(who) else {
            chrome::label_in(
                scene,
                ts,
                tr("code.bancada.map.empty"),
                Rect::new(inner.x, inner.y, inner.w, ROW_H_PX),
                ui.text3,
            );
            return;
        };
        // Cabe tudo, com a proporção do canvas e uma folga de meio ponto à volta.
        let dot = Spacing::Xs.px();
        let (bw, bh) = ((b[2] - b[0]).max(1.0), (b[3] - b[1]).max(1.0));
        let k = ((inner.w - dot * 2.0) / bw)
            .min((inner.h - dot * 2.0) / bh)
            .max(0.0);
        let ox = inner.x + (inner.w - bw * k) * 0.5 - b[0] * k;
        let oy = inner.y + (inner.h - bh * k) * 0.5 - b[1] * k;
        let lit = match self.aim {
            Aim::Cell(h, _) | Aim::Holder(h) | Aim::Select(h) => Some(h),
            _ => self.peeking,
        };
        scene.push_clip(&rect_to_vello(inner));
        // O aceso por último: fica por cima dos outros.
        let mut order: Vec<usize> = (0..who.map.len()).collect();
        order.sort_by_key(|&i| lit == Some(who.map[i].0));
        for i in order {
            let (h, rings) = &who.map[i];
            let Some(holder) = who.holders.get(*h) else {
                continue;
            };
            let color = resolve(ColorToken::layer(holder.color), *theme);
            let on = lit == Some(*h);
            let mut path = BezPath::new();
            let mut bb = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
            for ring in rings {
                for (j, p) in ring.iter().enumerate() {
                    let q = (f64::from(ox + p[0] * k), f64::from(oy + p[1] * k));
                    if j == 0 {
                        path.move_to(q)
                    } else {
                        path.line_to(q)
                    }
                    bb = [
                        bb[0].min(q.0 as f32),
                        bb[1].min(q.1 as f32),
                        bb[2].max(q.0 as f32),
                        bb[3].max(q.1 as f32),
                    ];
                }
                path.close_path();
            }
            if bb[0] > bb[2] {
                continue;
            }
            let fill = if on {
                ROW_SELECTED_ALPHA
            } else {
                LAYER_OUTLINE_OFF_ALPHA * ROW_SELECTED_ALPHA
            };
            scene.fill_path(
                &path,
                &Brush::Solid(color.with_alpha(fill)),
                Affine::IDENTITY,
            );
            let width = if on {
                StrokeToken::Thick.px()
            } else {
                StrokeToken::Default.px()
            };
            let ink = if on {
                color
            } else {
                color.with_alpha(LAYER_OUTLINE_OFF_ALPHA.max(ROW_SELECTED_ALPHA))
            };
            scene.stroke_path(
                &Stroke::new(f64::from(width)),
                Affine::IDENTITY,
                &Brush::Solid(ink),
                &path,
            );
            // Um objecto pequeno de mais para se ver ganha um ponto no meio.
            let (cx, cy) = ((bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5);
            if bb[2] - bb[0] < dot * 2.0 && bb[3] - bb[1] < dot * 2.0 {
                fill_circle(scene, cx, cy, dot, color);
            }
            let r = Rect::new(
                bb[0].min(cx - dot),
                bb[1].min(cy - dot),
                (bb[2] - bb[0]).max(dot * 2.0),
                (bb[3] - bb[1]).max(dot * 2.0),
            );
            self.map_hits
                .push((Aim::Holder(*h), super::grid::clip(r, inner)));
        }
        scene.pop_layer();
    }
}
