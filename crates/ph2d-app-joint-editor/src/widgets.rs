//! As medidas da aba e os controlos pequenos (botão, segmentado, número, caixa). Cada controlo pintado
//! regista o alvo dele em `doc.hits` com o rótulo que o leitor de ecrã lê: pintou ⟹ clicável.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, paint_text_elided, stroke_rounded_rect,
};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::{Color, VectorScene};

use crate::doc::{Field, Hit, JointEditorDoc};

/// A coluna das montagens.
pub const LIST_W: f32 = 220.0;
/// O painel de baixo quando não há junta (o título e uma frase).
pub const PANEL_MIN_H: f32 = 96.0;
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

    pub fn label(&mut self, s: &str, r: Rect) {
        let c = self.c(ColorToken::CardLabel);
        text(self.ts, self.scene, s, r, m::LABEL_PX, c);
    }

    /// Um botão: `primary` = o azul.
    pub fn button(&mut self, r: Rect, label: &str, h: Hit, primary: bool) {
        let (bg, fg) = match (primary, self.hot(&h)) {
            (true, false) => (ColorToken::Accent, ColorToken::AccentFg),
            (true, true) => (ColorToken::AccentHover, ColorToken::AccentFg),
            (false, false) => (ColorToken::CardField, ColorToken::Text1),
            (false, true) => (ColorToken::CardFieldHover, ColorToken::Text1),
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.c(bg));
        let c = self.c(fg);
        text_centred(self.ts, self.scene, label, r, m::VALUE_PX, c);
        self.hit(r, h, label);
    }

    /// Uma escolha segmentada a toda a largura de `r`.
    pub fn segmented(&mut self, r: Rect, opts: &[(String, Hit, bool)]) {
        fill_rounded_rect(
            self.scene,
            r,
            m::FIELD_RADIUS,
            self.c(ColorToken::CardField),
        );
        let n = opts.len().max(1);
        #[allow(clippy::cast_precision_loss)]
        let w = (r.w - m::SEG_PAD * 2.0) / n as f32;
        for (i, (label, h, on)) in opts.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let s = Rect::new(
                r.x + m::SEG_PAD + w * i as f32,
                r.y + m::SEG_PAD,
                w,
                r.h - m::SEG_PAD * 2.0,
            );
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
            self.hit(s, h.clone(), label);
        }
    }

    /// Um número: o rótulo por cima, a caixa com o valor (ou o que se está a escrever, com o cursor).
    pub fn number(&mut self, r: Rect, label: &str, f: Field, value: Option<f32>) {
        let lr = Rect::new(r.x, r.y, r.w, m::LABEL_PX + m::LABEL_BOTTOM);
        self.label(label, lr);
        let b = Rect::new(r.x, r.y + lr.h, r.w, m::FIELD_H);
        let typing = self.doc.typing_shown(f);
        let bg = if typing.is_some() || self.hot(&Hit::Field(f)) {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, b, m::FIELD_RADIUS, self.c(bg));
        if typing.is_some() {
            stroke_rounded_rect(
                self.scene,
                b,
                m::FIELD_RADIUS,
                m::FIELD_BORDER,
                self.c(ColorToken::FocusRing),
            );
        }
        let shown = typing
            .clone()
            .unwrap_or_else(|| value.map_or_else(String::new, format));
        let inner = Rect::new(b.x + m::FIELD_PAD_R, b.y, b.w - m::FIELD_PAD_R * 2.0, b.h);
        let c = self.c(ColorToken::Text1);
        text(self.ts, self.scene, &shown, inner, m::VALUE_PX, c);
        if typing.is_some() {
            let x = inner.x + self.ts.prefix_width(&shown, m::VALUE_PX).min(inner.w);
            let caret = Rect::new(
                x,
                b.y + m::SEG_PAD * 2.0,
                m::FIELD_BORDER,
                b.h - m::SEG_PAD * 4.0,
            );
            fill_rounded_rect(self.scene, caret, 0.0, c);
        }
        self.hit(b, Hit::Field(f), label);
    }

    /// Uma caixa de marcar com o rótulo à direita (caixas, nunca toggles: dono 10/11).
    pub fn check(&mut self, r: Rect, label: &str, h: Hit, on: bool) {
        let side = m::CHECK_BOX;
        let b = Rect::new(r.x, r.y + (r.h - side) * 0.5, side, side);
        if on {
            fill_rounded_rect(self.scene, b, m::SEG_RADIUS, self.c(ColorToken::Accent));
            let c = self.c(ColorToken::AccentFg);
            let tick = [
                (b.x + side * 0.25, b.y + side * 0.52),
                (b.x + side * 0.43, b.y + side * 0.70),
                (b.x + side * 0.76, b.y + side * 0.32),
            ];
            ph2d_editor_core::paint::stroke_polyline(self.scene, &tick, LINE, c);
        } else {
            let bg = if self.hot(&h) {
                ColorToken::CardFieldHover
            } else {
                ColorToken::CardField
            };
            fill_rounded_rect(self.scene, b, m::SEG_RADIUS, self.c(bg));
            stroke_rounded_rect(
                self.scene,
                b,
                m::SEG_RADIUS,
                m::FIELD_BORDER,
                self.c(ColorToken::Border),
            );
        }
        let t = Rect::new(
            b.x + side + m::CHECK_GAP,
            r.y,
            r.w - side - m::CHECK_GAP,
            r.h,
        );
        let c = self.c(ColorToken::Text1);
        text(self.ts, self.scene, label, t, m::VALUE_PX, c);
        self.hit(r, h, label);
    }
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
    #[test]
    fn o_numero_sai_sem_zeros_a_direita() {
        assert_eq!(super::format(0.15), "0.15");
        assert_eq!(super::format(-360.0), "-360");
        assert_eq!(super::format(-0.0001), "0");
    }
}
