//! ⭐ **As linhas de cartão de altura própria** (bloco B, painel Physics): a dica que quebra, a caixa
//! de marcar pelo [`CardCtx`] e a grelha triangular de ligar/desligar (o mapa de colisão da Unity e
//! da Godot). Não cabem em [`Widget`], cuja linha tem a altura de um campo.

use super::{CardCtx, Cell, Widget, corpo, rotulos};
use crate::interaction::toggle_paint;
use crate::paint::{fill_rounded_rect, paint_text, paint_text_block};
use crate::widget::key_mark::Rotulo;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, StrokeToken};

/// A grelha triangular: a linha `i` tem `i + 1` células; `(i, j)` é UM facto sobre o par.
pub struct ToggleTriangle<'a> {
    /// O nome por cima, com a unidade se tiver.
    pub label: &'a str,
    /// Quantas linhas (e colunas).
    pub n: usize,
    /// O grupo do arrasto que pinta (`toggle_paint`): cada célula declara o que mostra.
    pub group: &'static str,
    /// A marca de animação da grelha inteira (um controlo, uma marca).
    pub key: Option<&'a [NodeId]>,
    pub id_of: &'a dyn Fn(usize, usize) -> NodeId,
    pub on_of: &'a dyn Fn(usize, usize) -> bool,
}

impl CardCtx<'_> {
    /// Uma frase que explica o gesto: quebra na largura do cartão e devolve o fundo MEDIDO.
    pub fn hint(&mut self, text: &str, y: f32) -> f32 {
        self.first = false;
        let top = y + m::FIELD_ROW_GAP;
        let used = paint_text_block(
            self.text_system,
            self.scene,
            text,
            self.x,
            top,
            m::LABEL_PX,
            self.w,
            self.ink.color(ColorToken::CardLabel),
        );
        top + used
    }

    /// Uma caixa de marcar à Figma (T64), como 1.ª linha do corpo ou depois de outra. O que não anima
    /// (estado da vista) passa [`Rotulo::vista`] e não leva marca.
    pub fn check<'r>(&mut self, id: NodeId, label: impl Into<Rotulo<'r>>, on: bool, y: f32) -> f32 {
        let y = if self.first { y - m::FIELD_ROW_GAP } else { y };
        self.first = false;
        corpo::check(
            self.scene,
            self.text_system,
            self.store,
            self.hit_index,
            self.ink,
            self.x,
            self.w,
            y,
            corpo::Marca {
                id,
                label: label.into(),
                on,
            },
        )
    }

    /// A grelha triangular de ligar/desligar: legendas em cima e à esquerda; aceso = o acento, a
    /// diagonal (o par consigo próprio) leva contorno para o olho achar o eixo. As células encolhem
    /// com a largura do cartão, até [`m::GRID_CELL`].
    pub fn toggle_triangle(&mut self, t: &ToggleTriangle<'_>, y: f32) -> f32 {
        self.first = false;
        let mut y = y + m::LABEL_TOP;
        let mut titulo = Cell::new(Widget::Readout { text: "" }).label(t.label);
        titulo.key = t.key;
        y += rotulos::faixa(self, &[titulo], &[(self.x, self.w)], y);
        toggle_paint::limpa_grupo(t.group);
        let colunas = t.n as f32 + 1.0;
        let lado = ((self.w - m::GRID_GAP * t.n as f32) / colunas).min(m::GRID_CELL);
        let passo = lado + m::GRID_GAP;
        let legenda = self.ink.color(ColorToken::CardLabel);
        for j in 0..t.n {
            self.legenda(&j.to_string(), self.x + passo * (j + 1) as f32, y, lado);
        }
        y += m::LABEL_PX + m::LABEL_BOTTOM;
        for i in 0..t.n {
            self.legenda(&i.to_string(), self.x, y + (lado - m::LABEL_PX) * 0.5, lado);
            for j in 0..=i {
                let id = (t.id_of)(i, j);
                let on = (t.on_of)(i, j);
                let cell = Rect::new(self.x + passo * (j + 1) as f32, y, lado, lado);
                let fundo = if on {
                    ColorToken::Accent
                } else if self.store.hover_live(id) > 0.0 {
                    ColorToken::CardFieldHover
                } else {
                    ColorToken::CardField
                };
                fill_rounded_rect(self.scene, cell, m::SEG_RADIUS, self.ink.color(fundo));
                if i == j {
                    // FRAME-RAW-OK: a diagonal da grelha, contornada para o olho achar o eixo de simetria
                    crate::paint::stroke_rounded_rect(
                        self.scene,
                        cell,
                        m::SEG_RADIUS,
                        StrokeToken::Default.px(),
                        legenda,
                    );
                }
                self.hit_index.register(id, cell);
                toggle_paint::celula(id, t.group, on, cell);
            }
            y += passo;
        }
        crate::widget::composto::grupo(
            (0..t.n).flat_map(|i| (0..=i).map(move |j| (t.id_of)(i, j))),
        );
        y - m::GRID_GAP
    }

    fn legenda(&mut self, texto: &str, x: f32, y: f32, lado: f32) {
        let tw = self.text_system.prefix_width(texto, m::LABEL_PX);
        paint_text(
            self.text_system,
            self.scene,
            texto,
            x + (lado - tw) * 0.5,
            y,
            m::LABEL_PX,
            lado,
            self.ink.color(ColorToken::CardLabel),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::{HitIndex, WidgetStore};
    use crate::widget::card_ink::CardInk;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_vector::VectorScene;

    struct Mesa {
        scene: VectorScene,
        ts: TextSystem,
        store: WidgetStore,
        hits: HitIndex,
    }

    fn mesa() -> Mesa {
        Mesa {
            scene: VectorScene::new(),
            ts: TextSystem::without_system_fonts(),
            store: WidgetStore::default(),
            hits: HitIndex::default(),
        }
    }

    fn ctx(f: &mut Mesa, w: f32) -> CardCtx<'_> {
        CardCtx::flutuante(
            &mut f.scene,
            &mut f.ts,
            &f.store,
            &mut f.hits,
            CardInk::of_card(None, Theme::Dark),
            0.0,
            w,
        )
    }

    /// Só o triângulo, cada célula no seu sítio, dentro da largura, e a mais estreita encolhe.
    #[test]
    fn a_toggle_triangle_registers_one_cell_per_pair_inside_the_card() {
        for (w, lado) in [(400.0, m::GRID_CELL), (120.0, 9.78)] {
            let mut f = mesa();
            let id_of = |i: usize, j: usize| NodeId((i * 100 + j + 1) as u64);
            let on_of = |i: usize, j: usize| i == j;
            let fim = ctx(&mut f, w).toggle_triangle(
                &ToggleTriangle {
                    label: "Layers",
                    n: 8,
                    group: "teste.triangulo",
                    key: None,
                    id_of: &id_of,
                    on_of: &on_of,
                },
                0.0,
            );
            let rects: Vec<(NodeId, Rect)> = f.hits.iter_registrations().collect();
            assert_eq!(rects.len(), 36, "8 linhas = 36 pares, sem o espelho");
            for (id, r) in &rects {
                assert!(r.x + r.w <= w + 0.01, "{id:?} sai do cartão: {r:?}");
                assert!(r.y + r.h <= fim + 0.01, "{id:?} passa do fundo devolvido");
                assert!((r.w - lado).abs() < 0.1, "lado {} em largura {w}", r.w);
            }
        }
    }

    /// Um facto não regista clique; a dica cresce quando quebra, e o fundo devolvido é o medido.
    #[test]
    fn a_readout_takes_no_click_and_a_hint_grows_when_it_wraps() {
        let mut f = mesa();
        let fim = ctx(&mut f, 200.0).row(
            &[Cell::new(Widget::Readout { text: "120" }).label("Scale")],
            0.0,
        );
        assert!(fim > m::FIELD_H);
        assert_eq!(f.hits.iter_registrations().count(), 0);
        let frase = "Paused: drag carries the whole rig, anchors included";
        let larga = ctx(&mut f, 900.0).hint(frase, 0.0);
        let estreita = ctx(&mut f, 80.0).hint(frase, 0.0);
        assert!(
            estreita > larga,
            "a quebra tem de empurrar a linha seguinte: {estreita} × {larga}"
        );
    }

    /// A marca como 1.ª linha não abre vão; depois de outra linha, abre.
    #[test]
    fn a_check_takes_the_gap_only_after_another_line() {
        let mut f = mesa();
        let mut c = ctx(&mut f, 200.0);
        let a = c.check(NodeId(1), "Enabled", true, 0.0);
        assert_eq!(a, m::FIELD_H);
        let b = c.check(NodeId(2), "Show", false, a);
        assert_eq!(b, a + m::FIELD_ROW_GAP + m::FIELD_H);
    }
}
