//! As medidas da aba e os controlos pequenos (botão, segmentado, número, caixa). Cada controlo pintado
//! regista o alvo dele em `doc.hits` com o rótulo que o leitor de ecrã lê: pintou ⟹ clicável.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, folga_do_texto, paint_icon, paint_text_elided};
use ph2d_editor_core::widget::card_field::{
    Glyph, handle_rect, paint_field_caret, paint_field_frame, value_rect,
};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::{card_check_rect, paint_card_check};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

use crate::doc::{Field, Hit, JointEditorDoc};

/// A coluna das montagens.
pub const LIST_W: f32 = 220.0;
/// A folga do gabarito dentro do palco (o rótulo da peça cabe por baixo da forma).
pub const STAGE_PAD: f32 = 48.0;
/// O maior zoom do desenho, px por metro: o carro de fábrica (2,4 m) fica com ~670 px e sobra palco.
pub const MAX_SCALE: f32 = 280.0;
/// O véu da cor de uma peça (o contorno vai cheio).
pub const PART_FILL_ALPHA: f32 = 0.35;
/// O traço das linhas do desenho.
pub const LINE: f32 = 2.0;
/// O traço da barra do curso (laranja grosso, como na maquete B).
pub const BAR: f32 = 5.0;
/// O raio de uma alça.
pub const HANDLE_R: f32 = 6.0;
/// A miniatura de um gabarito no seletor.
pub const THUMB: (f32, f32) = (76.0, 38.0);
/// A linha de uma opção da lista suspensa com miniatura.
pub const MENU_ROW_H: f32 = 48.0;
/// O texto pequeno (pés, papéis).
pub const SMALL_PX: f32 = 10.5;
/// O valor mais largo que um campo escreve, «-0.000» (sinal, três casas), medido no `TextSystem` a
/// `VALUE_PX` (34,6 px, teste `o_campo_minimo_cabe_o_valor_e_a_letra_mais_largos`), arredondado acima.
pub const VALUE_MIN_W: f32 = 35.0;
/// A pega mais larga, «Max» a `LABEL_PX` com `SEG_INSET` de cada lado (34,4 px, mesmo teste); as de uma
/// letra cabem na `ICON_CELL` (26).
pub const GLYPH_MIN_W: f32 = 35.0;
/// ⭐ **O campo mínimo que ainda se lê** (dono 13/11): a pega, «-0.000» inteiro e a folga da direita.
/// Abaixo disto dois campos lado a lado cortam o valor, e a fila passa a uma coluna.
pub const FIELD_MIN_W: f32 = GLYPH_MIN_W + VALUE_MIN_W + m::FIELD_PAD_R;

/// As cores das montagens na lista (e das peças no desenho): a paleta das camadas.
pub const PALETTE: [ColorToken; 8] = [
    ColorToken::ObjLayer0,
    ColorToken::ObjLayer2,
    ColorToken::ObjLayer4,
    ColorToken::ObjLayer6,
    ColorToken::ObjLayer8,
    ColorToken::ObjLayer10,
    ColorToken::ObjLayer12,
    ColorToken::ObjLayer14,
];

#[must_use]
pub fn palette(i: usize) -> ColorToken {
    PALETTE[i % PALETTE.len()]
}

/// Uma linha de texto dentro de `r` (centrada na vertical, cortada com «…»).
pub fn text(ts: &mut TextSystem, scene: &mut VectorScene, s: &str, r: Rect, px: f32, c: Color) {
    paint_text_elided(ts, scene, s, r.x, r.y + folga_do_texto(r.h, px), px, r.w, c);
}

/// Uma linha centrada na horizontal.
pub fn text_centred(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    px: f32,
    c: Color,
) {
    let w = ts.prefix_width(s, px).min(r.w);
    let x = r.x + (r.w - w) * 0.5;
    text(ts, scene, s, Rect::new(x, r.y, w + 1.0, r.h), px, c);
}

/// O pincel de um controlo: o desenho e o registo.
pub struct Ui<'a> {
    pub doc: &'a mut JointEditorDoc,
    pub scene: &'a mut VectorScene,
    pub ts: &'a mut TextSystem,
    pub ink: CardInk,
    pub theme: Theme,
}

impl Ui<'_> {
    pub fn c(&self, t: ColorToken) -> Color {
        self.ink.color(t)
    }

    pub fn hit(&mut self, r: Rect, h: Hit, label: &str) {
        self.doc.hits.push((r, h, label.to_owned()));
    }

    fn hot(&self, h: &Hit) -> bool {
        self.doc.hover.as_ref() == Some(h) || self.doc.pressed.as_ref() == Some(h)
    }

    /// Uma linha de texto na cor `t`.
    pub fn put(&mut self, s: &str, r: Rect, px: f32, t: ColorToken) {
        let c = self.c(t);
        text(self.ts, self.scene, s, r, px, c);
    }

    pub fn label(&mut self, s: &str, r: Rect) {
        let c = self.c(ColorToken::CardLabel);
        text(self.ts, self.scene, s, r, m::LABEL_PX, c);
    }

    /// Um botão; desligado (`on == false`) pinta-se apagado e não se clica.
    pub fn button(&mut self, r: Rect, label: &str, h: Hit, on: bool) {
        let (bg, fg) = match (on, self.hot(&h)) {
            (false, _) => (ColorToken::CardField, ColorToken::TextDisabled),
            (true, false) => (ColorToken::CardField, ColorToken::Text1),
            (true, true) => (ColorToken::CardFieldHover, ColorToken::Text1),
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.c(bg));
        let c = self.c(fg);
        text_centred(self.ts, self.scene, label, r, m::VALUE_PX, c);
        if on {
            self.hit(r, h, label);
        }
    }

    /// A caixa de uma lista suspensa: o campo do cartão com o valor e a seta.
    pub fn dropdown(&mut self, b: Rect, value: &str, h: Hit, a11y: &str) {
        paint_field_frame(self.scene, self.ink, b, self.hot(&h), false);
        let cw = m::CARD_CHEVRON + m::FIELD_PAD_R;
        let t = Rect::new(b.x + m::FIELD_PAD_R, b.y, b.w - m::FIELD_PAD_R - cw, b.h);
        self.put(value, t, m::VALUE_PX, ColorToken::Text1);
        let g = m::CARD_CHEVRON;
        let chev = Rect::new(b.x + b.w - cw, b.y + (b.h - g) * 0.5, g, g);
        let c = self.c(ColorToken::Text2);
        paint_icon(self.scene, IconId::ChevronDown, chev, c, m::KEY_MARK_STROKE);
        self.hit(b, h, a11y);
    }

    /// Uma escolha segmentada a toda a largura de `r` (a altura de [`seg_h`] das fileiras de [`seg_rows`]).
    pub fn segmented(&mut self, r: Rect, opts: &[(String, Hit, bool)]) {
        fill_rounded_rect(
            self.scene,
            r,
            m::FIELD_RADIUS,
            self.c(ColorToken::CardField),
        );
        let (short, rows) = seg_rows(self.ts, opts, r.w);
        let seg_h = m::FIELD_H - m::SEG_PAD * 2.0;
        let cells = rows.iter().enumerate().flat_map(|(k, ws)| {
            #[allow(clippy::cast_precision_loss)]
            let y = r.y + m::SEG_PAD + seg_h * k as f32;
            let mut x = r.x + m::SEG_PAD;
            ws.iter().map(move |w| {
                x += w;
                Rect::new(x - w, y, *w, seg_h)
            })
        });
        for ((full, h, on), s) in opts.iter().zip(cells.collect::<Vec<_>>()) {
            let label = if short {
                first_word(full)
            } else {
                full.as_str()
            };
            if *on {
                fill_rounded_rect(self.scene, s, m::SEG_RADIUS, self.c(ColorToken::Accent));
            } else if self.hot(h) {
                fill_rounded_rect(
                    self.scene,
                    s,
                    m::SEG_RADIUS,
                    self.c(ColorToken::CardFieldHover),
                );
            }
            let fg = self.c(if *on {
                ColorToken::AccentFg
            } else {
                ColorToken::Text1
            });
            text_centred(self.ts, self.scene, label, s, m::SEG_TEXT_PX, fg);
            self.hit(s, h.clone(), full);
        }
    }

    /// ⭐ O campo do cartão do app (`card_field`): a letra ou o ícone na caixa, na cor do rótulo, e o valor
    /// (ou o que se escreve, com o cursor). A unidade mora no rótulo, como no Inspector; `name` é o acessível.
    pub fn number(&mut self, b: Rect, glyph: Glyph<'_>, name: &str, f: Field) {
        let typing = self.doc.typing_shown(f);
        let hot = self.hot(&Hit::Field(f));
        paint_field_frame(self.scene, self.ink, b, hot, typing.is_some());
        let cell = handle_rect(b);
        let tint = self.c(ColorToken::CardLabel);
        let mut area = value_rect(b);
        match glyph {
            Glyph::Icon(icon) => {
                let g = m::ICON_GLYPH;
                let (gx, gy) = (cell.x + (cell.w - g) * 0.5, cell.y + (cell.h - g) * 0.5);
                let r = Rect::new(gx, gy, g, g);
                paint_icon(self.scene, icon, r, tint, StrokeToken::Default.px());
            }
            Glyph::Text(t) => {
                // Uma pega mais larga que a célula («Min») empurra o valor em vez de lhe tocar.
                let gw = self.ts.prefix_width(t, m::LABEL_PX) + m::SEG_INSET * 2.0;
                let cell = Rect::new(cell.x, cell.y, cell.w.max(gw), cell.h);
                text_centred(self.ts, self.scene, t, cell, m::LABEL_PX, tint);
                let x = cell.x + cell.w;
                area = Rect::new(x, area.y, (area.x + area.w - x).max(0.0), area.h);
            }
        }
        let shown = typing
            .clone()
            .unwrap_or_else(|| self.doc.value(f).map_or_else(String::new, format));
        let c = self.c(ColorToken::Text1);
        text(self.ts, self.scene, &shown, area, m::VALUE_PX, c);
        if typing.is_some() {
            let vw = self.ts.prefix_width(&shown, m::VALUE_PX).min(area.w);
            let y = area.y + folga_do_texto(area.h, m::VALUE_PX);
            paint_field_caret(self.scene, self.ink, area.x + vw, y, m::VALUE_PX);
        }
        self.hit(b, Hit::Field(f), name);
    }

    /// A caixa de marcar do app numa MOLDURA, como a Godot (dono 13/11): caixa e texto dentro do fundo
    /// de um campo, a moldura inteira clicável. Caixa, nunca toggle (dono 10/11).
    pub fn check(&mut self, r: Rect, label: &str, name: &str, h: Hit, on: bool) {
        let hot = self.hot(&h);
        paint_field_frame(self.scene, self.ink, r, hot, false);
        let bx = card_check_rect(r.x + m::SEG_INSET, r.y, r.h);
        paint_card_check(self.scene, self.ink, bx, on, hot);
        let x = r.x + m::SEG_INSET + m::CHECK_BOX + m::CHECK_GAP;
        let t = Rect::new(x, r.y, r.x + r.w - x, r.h);
        self.put(label, t, m::VALUE_PX, ColorToken::Text1);
        self.hit(r, h, name);
    }
}

/// ⭐ **A disposição de um segmentado na largura `w`** (o pintor e o medidor leem-na): `(curto, larguras
/// por fileira)`. D32: cada segmento com a largura do que mostra, a sobra repartida. Não cabe ⇒ cada opção
/// encolhe à primeira palavra («Rear wheel» → «Rear»; o leitor de ecrã ouve o nome inteiro). Nem assim ⇒
/// quebra em fileiras, quantas couberem por fileira (a lei do `segmented_row_counts` do app, dono 19/07).
pub fn seg_rows(
    ts: &mut TextSystem,
    opts: &[(String, Hit, bool)],
    w: f32,
) -> (bool, Vec<Vec<f32>>) {
    let avail = w - m::SEG_PAD * 2.0;
    let need_of = |ts: &mut TextSystem, short: bool| -> Vec<(f32, bool)> {
        opts.iter()
            .map(|(l, _, _)| {
                let s = if short { first_word(l) } else { l.as_str() };
                (
                    ts.prefix_width(s, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0,
                    false,
                )
            })
            .collect()
    };
    let sum = |n: &[(f32, bool)]| n.iter().map(|s| s.0).sum::<f32>();
    let full = need_of(ts, false);
    if sum(&full) <= avail {
        return (false, vec![m::seg_widths(&full, avail)]);
    }
    let need = need_of(ts, true);
    let mut rows: Vec<Vec<(f32, bool)>> = Vec::new();
    for s in need {
        match rows.last_mut() {
            Some(row) if sum(row) + s.0 <= avail => row.push(s),
            _ => rows.push(vec![s]),
        }
    }
    (true, rows.iter().map(|r| m::seg_widths(r, avail)).collect())
}

/// A altura do segmentado com `rows` fileiras.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn seg_h(rows: usize) -> f32 {
    m::SEG_PAD * 2.0 + (m::FIELD_H - m::SEG_PAD * 2.0) * rows.max(1) as f32
}

/// A forma curta de uma opção: a primeira palavra.
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

/// O número como o cartão o escreve: até três casas, sem zeros à direita.
#[must_use]
pub fn format(v: f32) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{GLYPH_MIN_W, VALUE_MIN_W, m};

    #[test]
    fn o_campo_minimo_cabe_o_valor_e_a_letra_mais_largos() {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let v = ts.prefix_width("-0.000", m::VALUE_PX);
        let keys = ["stiffness", "damping", "speed", "force", "torque"];
        let more = ["wheelbase", "length", "min", "max"];
        let g = keys
            .iter()
            .chain(&more)
            .map(|k| {
                ts.prefix_width(
                    ph2d_i18n::tr(&format!("joint_editor.glyph.{k}")),
                    m::LABEL_PX,
                )
            })
            .fold(0.0f32, f32::max)
            + m::SEG_INSET * 2.0;
        assert!(
            v <= VALUE_MIN_W && v > VALUE_MIN_W - 1.0,
            "«-0.000» mede {v}"
        );
        assert!(g <= GLYPH_MIN_W && g > GLYPH_MIN_W - 1.0, "a pega mede {g}");
    }

    #[test]
    fn o_numero_sai_sem_zeros_a_direita() {
        assert_eq!(super::format(0.15), "0.15");
        assert_eq!(super::format(-360.0), "-360");
        assert_eq!(super::format(-0.0001), "0");
    }
}
