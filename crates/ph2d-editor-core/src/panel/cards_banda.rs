//! ⭐ **O cromo de um painel-BANDA** (Timeline, `docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): a moldura
//! da lista sem cartão por dentro, porque a altura é do trabalho, e os controlos da lista em fluxo, numa
//! fileira: o campo com o rótulo AO LADO (o de cima custa uma linha de faixa), a caixa T64, os segmentos,
//! a fila de ícones, o chip. Os ids, o store e o despacho são os do painel.

use super::*;
use crate::widget::card_field::paint_field;
use crate::widget::key_mark::Rotulo;
use crate::widget::paint_card_chip;
pub use ph2d_card_stack::metrics;

/// A moldura pintada: a lista, a tira do cabeçalho (depois do título, antes do «×») e o interior.
#[derive(Copy, Clone, Debug)]
pub struct Moldura {
    pub lista: Rect,
    pub cab: Rect,
    pub corpo: Rect,
}

/// A geometria de [`pinta_moldura`] sem pintar: quem resolve gestos antes da pintura precisa dela.
#[must_use]
pub fn geometria(slot: Rect) -> Moldura {
    let lista = Rect::new(
        slot.x + m::LIST_MARGIN,
        slot.y + m::LIST_MARGIN,
        (slot.w - m::LIST_MARGIN * 2.0).max(0.0),
        (slot.h - m::LIST_MARGIN * 2.0).max(0.0),
    );
    let topo = lista.y + m::LIST_HEAD_H;
    let corpo = Rect::new(
        lista.x + m::LIST_PAD_X,
        topo,
        (lista.w - m::LIST_PAD_X * 2.0).max(0.0),
        (lista.y + lista.h - m::LIST_PAD_BOTTOM - topo).max(0.0),
    );
    let cab = Rect::new(
        corpo.x,
        lista.y + (m::LIST_HEAD_H - m::FIELD_H) * 0.5,
        corpo.w,
        m::FIELD_H,
    );
    Moldura { lista, cab, corpo }
}

/// A moldura flutuante, o título e o «×» (`mark_card_list_close`), como numa lista de cartões. Sem o
/// «…»: as linhas dele repõem cartões, e aqui não há cartão nenhum (doc 14).
pub fn pinta_moldura(ctx: &mut PaintCtx, panel: NodeId, title: &str, slot: Rect) -> Moldura {
    dicas::comeca(panel);
    let theme = ctx.host.theme();
    let mut g = geometria(slot);
    let lista = g.lista;
    pinta_pilula(ctx, lista);
    let nome = title.to_uppercase();
    let tx = lista.x + m::LIST_PAD_X + m::LIST_HEAD_PAD_X;
    let tracking = m::LIST_TITLE_PX * m::LIST_TITLE_TRACKING_EM * nome.chars().count() as f32;
    let tw = ctx.text_system.prefix_width(&nome, m::LIST_TITLE_PX) + tracking;
    crate::paint::paint_text_title_elided_tracked(
        ctx.text_system,
        ctx.scene,
        &nome,
        [tx, lista.y + (m::LIST_HEAD_H - m::LIST_TITLE_PX) * 0.5],
        m::LIST_TITLE_PX,
        (lista.w - m::LIST_PAD_X * 2.0 - m::HEAD_ICON_BOX).max(0.0),
        resolve(ColorToken::KanbanListFg, theme),
        m::LIST_TITLE_TRACKING_EM,
    );
    menu::pinta_fecho(
        ctx,
        panel,
        theme,
        Rect::new(lista.x, lista.y, lista.w, m::LIST_HEAD_H),
    );
    let x = tx + tw + m::CARD_GAP;
    let fim = lista.x + lista.w - m::LIST_PAD_X - m::HEAD_ICON_BOX - m::FIELD_GAP;
    g.cab.x = x;
    g.cab.w = (fim - x).max(0.0);
    g
}

/// Uma pílula flutuante: a tinta e a sombra da lista, sem título (a barra de um canvas).
pub fn pinta_pilula(ctx: &mut PaintCtx, r: Rect) {
    let theme = ctx.host.theme();
    shadow(
        ctx.scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanListShadow, theme),
        m::LIST_SHADOW_FAR,
    );
    shadow(
        ctx.scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanShadowNear, theme),
        m::LIST_SHADOW_NEAR,
    );
    fill_rounded_rect(
        ctx.scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanList, theme),
    );
}

/// Um ícone de uma fila: `id: None` = apagado, sem alvo nenhum (o gesto não existe agora).
#[derive(Copy, Clone, Debug)]
pub struct Icone {
    pub id: Option<NodeId>,
    pub icon: IconId,
    pub on: bool,
}

/// O que a 1.ª fileira do corpo sobe, para o vão até à do cabeçalho ser o das outras fileiras.
pub const SOBRA_DO_CABECALHO: f32 = (m::LIST_HEAD_H - m::FIELD_H) * 0.5 - m::FIELD_ROW_GAP;

/// O vão entre a última fileira e a folha: a altura de faixa é o recurso do painel-banda (doc 14).
pub const VAO_ATE_A_FOLHA: f32 = m::SEG_RADIUS;

/// A largura de uma fila de `n` ícones.
#[must_use]
pub fn icones_w(n: usize) -> f32 {
    m::FIELD_H * n as f32
}

/// A largura da caixa de um campo em fileira: pega + valor.
const CAMPO_W: f32 = m::ICON_CELL + m::FIELD_VALUE_MIN_W;

impl CardCtx<'_> {
    fn largura_do_texto(&mut self, texto: &str, px: f32) -> f32 {
        self.text_system.prefix_width(texto, px).ceil()
    }

    /// A coluna de rótulos de uma família de campos: a do mais largo. Quando a barra estreita os
    /// empilha, os valores arrancam no mesmo `x` (`onde_comeca_o_valor`).
    pub fn coluna_de_rotulos(&mut self, rotulos: &[&str]) -> f32 {
        rotulos
            .iter()
            .map(|r| self.largura_do_texto(r, m::LABEL_PX))
            .fold(0.0, f32::max)
    }

    /// A largura de [`Self::campo_em_fila`] com a coluna `col`.
    #[must_use]
    pub fn campo_em_fila_w(col: f32) -> f32 {
        col + m::FIELD_GAP + CAMPO_W
    }

    /// O rótulo ao lado, na coluna `col`, e a caixa do campo Figma (`mark_card_field` antes).
    pub fn campo_em_fila(
        &mut self,
        id: NodeId,
        rotulo: &str,
        glyph: Glyph<'_>,
        col: f32,
        (x, y): (f32, f32),
    ) {
        dicas::rotulo(id, rotulo);
        crate::paint::paint_text(
            self.text_system,
            self.scene,
            rotulo,
            x,
            y + (m::FIELD_H - m::LABEL_PX) * 0.5,
            m::LABEL_PX,
            col,
            self.ink.color(ColorToken::CardLabel),
        );
        let r = Rect::new(x + col + m::FIELD_GAP, y, CAMPO_W, m::FIELD_H);
        paint_field(
            self.scene,
            self.text_system,
            self.ink,
            self.store,
            self.hit_index,
            r,
            id,
            glyph,
        );
    }

    /// A largura natural de [`Self::marca_em_fila`]: a coluna da marca de animação é o vão seguinte.
    pub fn marca_em_fila_w(&mut self, nome: &str) -> f32 {
        m::CHECK_BOX + m::CHECK_GAP + self.largura_do_texto(nome, m::VALUE_PX) + m::KEY_MARK_BOX
    }

    /// A caixa T64, com o nome à direita e a linha inteira de alvo; o id continua o do `Toggle`.
    pub fn marca_em_fila(&mut self, id: NodeId, nome: &str, on: bool, x: f32, y: f32) {
        let w = self.marca_em_fila_w(nome);
        corpo::check(
            self.scene,
            self.text_system,
            self.store,
            self.hit_index,
            self.ink,
            x,
            w,
            y - m::FIELD_ROW_GAP,
            corpo::Marca {
                id,
                label: Rotulo::vista_de(nome),
                on,
            },
        );
    }

    /// A largura natural de [`Self::segmentos_em_fila`], segmentos só de texto.
    pub fn segmentos_em_fila_w(&mut self, segs: &[Seg<'_>]) -> f32 {
        let textos: f32 = segs
            .iter()
            .map(|s| self.largura_do_texto(s.text, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0)
            .sum();
        textos + m::SEG_PAD * (segs.len() + 1) as f32
    }

    /// Os segmentos à largura natural, em `(x, y)`.
    pub fn segmentos_em_fila(&mut self, segs: &[Seg<'_>], x: f32, y: f32) {
        let w = self.segmentos_em_fila_w(segs);
        let (x0, w0) = (self.x, self.w);
        (self.x, self.w, self.first) = (x, w, true);
        self.row(&[Cell::new(Widget::Choice(segs))], y);
        (self.x, self.w) = (x0, w0);
    }

    /// Ícones colados numa caixa: verbos, cada um o seu; o `Icone::on` é o estado aceso.
    pub fn icones_em_fila(&mut self, celulas: &[Icone], x: f32, y: f32) {
        let caixa = Rect::new(x, y, icones_w(celulas.len()), m::FIELD_H);
        fill_rounded_rect(
            self.scene,
            caixa,
            m::FIELD_RADIUS,
            self.ink.color(ColorToken::CardField),
        );
        for (i, c) in celulas.iter().enumerate() {
            let b = Rect::new(x + m::FIELD_H * i as f32, y, m::FIELD_H, m::FIELD_H);
            let hover = c.id.is_some_and(|id| self.store.hover_live(id) > 0.0);
            if let Some(id) = c.id {
                self.hit_index.register(id, b);
                dicas::icone(id);
            }
            if c.on || hover {
                let fundo = Rect::new(
                    b.x + m::SEG_PAD,
                    b.y + m::SEG_PAD,
                    b.w - m::SEG_PAD * 2.0,
                    b.h - m::SEG_PAD * 2.0,
                );
                let cor = if c.on {
                    ColorToken::KanbanCardRaised
                } else {
                    ColorToken::CardFieldHover
                };
                fill_rounded_rect(self.scene, fundo, m::SEG_RADIUS, self.ink.color(cor));
            }
            let tinta = if c.id.is_none() {
                ColorToken::Text3
            } else if c.on || hover {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            };
            let g = m::ICON_GLYPH;
            paint_icon(
                self.scene,
                c.icon,
                Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
                self.ink.color(tinta),
                StrokeToken::Default.px(),
            );
        }
    }

    /// A largura natural de [`Self::botao_em`].
    pub fn botao_w(&mut self, texto: &str) -> f32 {
        self.largura_do_texto(texto, m::VALUE_PX) + m::FIELD_PAD_R * 2.0
    }

    /// Um botão de texto do cartão, no rect que o chamador der.
    pub fn botao_em(&mut self, id: NodeId, texto: &str, r: Rect) {
        self.hit_index.register(id, r);
        let fundo = if self.store.hover_live(id) > 0.0 {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let tw = self.largura_do_texto(texto, m::VALUE_PX);
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            texto,
            r.x + ((r.w - tw) * 0.5).max(0.0),
            r.y + (r.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            r.w,
            self.ink.color(ColorToken::KanbanCardFg),
        );
    }

    /// A cara de uma lista suspensa cujo estado e opções são do painel (o clique é o do `Dropdown`).
    pub fn lista_em(&mut self, id: NodeId, texto: &str, aberta: bool, r: Rect) {
        self.hit_index.register(id, r);
        let aceso = aberta || self.store.hover_live(id) > 0.0;
        paint_card_chip(self.scene, self.text_system, self.ink, r, texto, aceso);
    }
}
