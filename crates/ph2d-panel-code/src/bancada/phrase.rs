//! ⭐ **A frase do pé** (plano 13 Onda B, item 9) — fixa, em negrito, no fundo da aba: diz o que o
//! rato aponta e o que o próximo clique faz, com os nomes de agora (a frase do pé da aba Collision
//! Layers, `ph2d-panel-physics` `aba/mod.rs` `frase_do_pe`). Pura em [`say`]; o espiar da cena
//! pinta só ela e o nome sob o rato.

use super::{Aim, BancadaView, Who};
use crate::chrome::{self, Ui};
use ph2d_code_viewmodel::CellKind;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text_title, resolve,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Os nomes de uma lista, os três primeiros e «…» (o resto cabe na contagem).
fn names(who: &Who, of: impl Iterator<Item = usize>) -> String {
    let all: Vec<&str> = of
        .filter_map(|h| who.holders.get(h))
        .map(|h| h.name.as_str())
        .collect();
    if all.is_empty() {
        return tr("code.bancada.say.none_yet").to_owned();
    }
    let mut s = all.iter().take(3).copied().collect::<Vec<_>>().join(", ");
    if all.len() > 3 {
        s.push_str(", \u{2026}");
    }
    s
}

/// ⭐ **A frase** para `aim` (o que está sob o rato), `editing` (a célula em escrita), o nome do
/// script e se o espiar está ligado. Uma frase só, sempre: nunca vazia.
#[must_use]
pub fn say(
    who: &Who,
    aim: Aim,
    editing: Option<(usize, usize)>,
    script: &str,
    peek_on: bool,
) -> String {
    let name = |h: usize| who.holders.get(h).map_or("", |x| x.name.as_str());
    if let Some((h, c)) = editing {
        let prop = who.columns.get(c).map_or("", |c| c.name.as_str());
        return tr_with(
            "code.bancada.say.editing",
            &[("prop", &prop), ("name", &name(h))],
        );
    }
    match aim {
        Aim::Cell(h, c) => {
            let (Some(col), Some(cell)) = (who.columns.get(c), who.cell(h, c)) else {
                return String::new();
            };
            let (n, p, v) = (name(h), col.name.as_str(), cell.text.as_str());
            match col.kind {
                None => tr_with("code.bancada.say.cell_unsaved", &[("prop", &p)]),
                Some(CellKind::Color) => tr_with("code.bancada.say.cell_color", &[("name", &n)]),
                Some(CellKind::Bool) => {
                    let next = (v != "true").to_string();
                    tr_with(
                        "code.bancada.say.cell_bool",
                        &[("name", &n), ("prop", &p), ("value", &v), ("next", &next)],
                    )
                }
                Some(CellKind::Choice) => {
                    let next = super::next_option(&col.options, v).unwrap_or("");
                    tr_with(
                        "code.bancada.say.cell_choice",
                        &[("name", &n), ("prop", &p), ("value", &v), ("next", &next)],
                    )
                }
                Some(_) if cell.changed => tr_with(
                    "code.bancada.say.cell_changed",
                    &[
                        ("name", &n),
                        ("prop", &p),
                        ("value", &v),
                        ("default", &col.default),
                    ],
                ),
                Some(_) => tr_with(
                    "code.bancada.say.cell_default",
                    &[("name", &n), ("prop", &p), ("value", &v)],
                ),
            }
        }
        Aim::Holder(h) => {
            let key = if peek_on {
                "code.bancada.say.holder_peek"
            } else {
                "code.bancada.say.holder"
            };
            tr_with(key, &[("name", &name(h))])
        }
        Aim::Select(h) => tr_with("code.bancada.say.select", &[("name", &name(h))]),
        Aim::Column(c) => {
            let Some(col) = who.columns.get(c) else {
                return String::new();
            };
            tr_with(
                "code.bancada.say.column",
                &[("prop", &col.name), ("line", &(col.line + 1))],
            )
        }
        Aim::Entry(line) => tr_with("code.bancada.say.entry", &[("line", &(line + 1))]),
        Aim::Mark(line) => {
            if let Some(c) = who.columns.iter().position(|c| c.line == line) {
                let changed = (0..who.holders.len())
                    .filter(|&h| who.cell(h, c).is_some_and(|x| x.changed))
                    .count();
                tr_with(
                    "code.bancada.say.mark_prop",
                    &[
                        ("prop", &who.columns[c].name),
                        ("changed", &changed),
                        ("count", &who.holders.len()),
                    ],
                )
            } else {
                let signal = who
                    .hears
                    .iter()
                    .find(|e| e.1 == line)
                    .map_or("", |e| e.0.as_str());
                let all = names(who, 0..who.holders.len());
                tr_with(
                    "code.bancada.say.mark_hear",
                    &[("signal", &signal), ("line", &(line + 1)), ("names", &all)],
                )
            }
        }
        Aim::Nothing if who.holders.is_empty() => {
            tr_with("code.bancada.say.idle_none", &[("script", &script)])
        }
        Aim::Nothing => tr_with(
            "code.bancada.say.idle",
            &[
                ("script", &script),
                ("count", &who.holders.len()),
                ("names", &names(who, 0..who.holders.len())),
            ],
        ),
    }
}

impl BancadaView {
    /// Pinta a frase do pé em `area` (a faixa inteira) e guarda-a ([`BancadaView::said`]).
    pub fn paint_phrase(
        &mut self,
        who: &Who,
        script: &str,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let editing = self.editing.as_ref().map(|e| (e.holder, e.column));
        let aim = if self.peeking.is_some() {
            Aim::Nothing
        } else {
            self.aim
        };
        self.said = match self.peeking {
            Some(h) => tr_with(
                "code.bancada.say.peek",
                &[("name", &who.holders.get(h).map_or("", |x| x.name.as_str()))],
            ),
            None => say(who, aim, editing, script, self.peek_on),
        };
        let radius = frame_radius(*theme, Radius::Sm.px());
        fill_rounded_rect(scene, area, radius, resolve(ColorToken::Bg2, *theme));
        let f = chrome::font();
        let s = Spacing::Sm.px();
        let ui = Ui::new(*theme);
        paint_text_title(
            ts,
            scene,
            &self.said,
            area.x + s,
            area.y + (area.h - f) * 0.5,
            f,
            (area.w - s * 2.0).max(0.0),
            ui.text1,
        );
    }

    /// O espiar: a aba cede a área à cena — só o nome sob o rato (com a cor dele) e a frase.
    pub fn paint_peek(
        &mut self,
        who: &Who,
        phrase: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let Some(h) = self.peeking else { return };
        let Some(holder) = who.holders.get(h) else {
            return;
        };
        self.paint_phrase(who, "", phrase, scene, ts, theme);
        let ui = Ui::new(*theme);
        let s = Spacing::Sm.px();
        let f = chrome::font();
        let dot = super::chip_r();
        let w = dot * 2.0 + s * 2.0 + s + chrome::label_w(ts, &holder.name);
        let r = Rect::new(self.pointer.0 + s, self.pointer.1 + s, w, f + s * 2.0);
        fill_rounded_rect(scene, r, frame_radius(*theme, Radius::Sm.px()), ui.field);
        fill_circle(
            scene,
            r.x + s + dot,
            r.y + r.h * 0.5,
            dot,
            resolve(ColorToken::layer(holder.color), *theme),
        );
        chrome::label_in(
            scene,
            ts,
            &holder.name,
            Rect::new(r.x + s * 2.0 + dot * 2.0, r.y, w, r.h),
            ui.text1,
        );
    }
}
