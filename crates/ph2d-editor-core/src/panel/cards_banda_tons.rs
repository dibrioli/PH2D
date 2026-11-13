//! O degradê da peça e os tons da barra de uma banda (Timeline, doc 15 onda 2/2i).
//! Irmão de `cards_banda.rs` (tecto de LOC); re-exportado por `banda`, o caminho não muda.

use super::*;

/// ⭐ **Um degradê VERTICAL `[topo, pé]` dentro do rect de quinas `raio`** — a peça da Timeline
/// (doc 16 onda 3, `.c13st` da maquete: a cor a 58 % em cima, a 34 % em baixo). O raio passa pela
/// mesma escala do [`crate::paint::fill_rounded_rect`] (o recorte é o [`crate::paint::push_rounded_clip`]).
pub fn degrade_vertical_em(
    scene: &mut VectorScene,
    r: Rect,
    raio: f32,
    [topo, pe]: [ph2d_vector::Color; 2],
) {
    use ph2d_vector::{Affine, Brush, Fill, Gradient, Point};
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }
    let (x, y0, y1) = (f64::from(r.x), f64::from(r.y), f64::from(r.y + r.h));
    let g = Gradient::new_linear(Point::new(x, y0), Point::new(x, y1))
        .with_stops([topo, pe].as_slice());
    crate::paint::push_rounded_clip(scene, r, raio);
    scene.fill_shape(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(g),
        &crate::paint::rect_to_vello(r),
    );
    scene.pop_layer();
}

/// As tintas de [`CardCtx::abas_em`] e das peças discretas — o painel dá-as (a Timeline, das suas
/// tintas da maquete B1); a geometria é a do cartão.
#[derive(Copy, Clone, Debug)]
pub struct TonsDaBarra {
    /// O fundo da aba escolhida (a pílula preenchida).
    pub ativa_fundo: ph2d_vector::Color,
    /// O texto da aba escolhida (em negrito).
    pub ativa_texto: ph2d_vector::Color,
    /// O texto das outras abas, e o das peças discretas.
    pub inativa_texto: ph2d_vector::Color,
    /// O fundo sob o rato (abas e peças discretas).
    pub sob_o_rato: ph2d_vector::Color,
    /// O texto e o ícone das peças discretas (Auto Key, Record, Marker), um degrau abaixo das abas.
    pub discreta_texto: ph2d_vector::Color,
    /// O contorno da caixa de marcar VAZIA das peças discretas (dono 12/11: «mais contraste com o fundo»).
    pub borda_da_caixa: ph2d_vector::Color,
    /// O corpo das peças discretas em repouso (dono 12/11: Auto Key e Marker soltos no fundo pedem a
    /// caixa dos outros campos da barra); sob o rato, [`Self::sob_o_rato`].
    pub corpo_da_peca: ph2d_vector::Color,
}

impl CardCtx<'_> {
    /// ⭐ **As abas da navegação principal** (Timeline, doc 15 onda 2i): a mesma largura e os mesmos
    /// segmentos de [`Self::segmentos_em_fila`], sem a caixa em volta; a escolhida numa pílula
    /// preenchida com o texto em negrito, as outras só texto. O alvo de cada aba é a altura INTEIRA do
    /// campo (nunca menor que o segmento de antes).
    pub fn abas_em(&mut self, segs: &[Seg<'_>], x: f32, y: f32, t: TonsDaBarra) {
        crate::widget::composto::grupo(segs.iter().map(|s| s.id));
        if let Some(s) = segs.first() {
            crate::property_row::escolha::anota_do_cartao(s.id, segs.len());
        }
        let w = self.segmentos_em_fila_w(segs);
        let n = segs.len().max(1) as f32;
        let inner = Rect::new(
            x + m::SEG_PAD,
            y + m::SEG_PAD,
            w - m::SEG_PAD * 2.0,
            m::FIELD_H - m::SEG_PAD * 2.0,
        );
        let avail = inner.w - m::SEG_PAD * (n - 1.0);
        let needs: Vec<(f32, bool)> = segs.iter().map(|s| self.seg_need(s, false)).collect();
        let ws = m::seg_widths(&needs, avail);
        let mut sx = inner.x;
        for (s, sw) in segs.iter().zip(ws) {
            let sr = Rect::new(sx, inner.y, sw, inner.h);
            let alvo = Rect::new(sx - m::SEG_PAD * 0.5, y, sw + m::SEG_PAD, m::FIELD_H);
            sx += sw + m::SEG_PAD;
            self.hit_index.register(s.id, alvo);
            let hover = self.store.hover_live(s.id) > 0.0;
            if s.on {
                fill_rounded_rect(self.scene, sr, m::SEG_RADIUS, t.ativa_fundo);
            } else if hover {
                fill_rounded_rect(self.scene, sr, m::SEG_RADIUS, t.sob_o_rato);
            }
            let tw = self.text_system.prefix_width(s.text, m::SEG_TEXT_PX);
            let tx = sr.x + ((sr.w - tw) * 0.5).max(0.0);
            let ty = sr.y + crate::paint::folga_do_texto(sr.h, m::SEG_TEXT_PX);
            if s.on {
                crate::paint::paint_text_title_elided(
                    self.text_system,
                    self.scene,
                    s.text,
                    tx,
                    ty,
                    m::SEG_TEXT_PX,
                    sr.w,
                    t.ativa_texto,
                );
            } else {
                crate::paint::paint_text_elided(
                    self.text_system,
                    self.scene,
                    s.text,
                    tx,
                    ty,
                    m::SEG_TEXT_PX,
                    sr.w,
                    t.inativa_texto,
                );
            }
        }
    }

    /// [`Self::emoldurada_em`] DISCRETA: sem o fundo de caixa (só sob o rato), o texto no tom
    /// secundário; a caixa de marcar e os sinais são os mesmos (o sinal acende quando liga). Mesma
    /// largura e mesmo alvo.
    pub fn emoldurada_discreta_em(&mut self, c: &Emoldurada<'_>, x: f32, y: f32, t: TonsDaBarra) {
        let r = Rect::new(x, y, self.emoldurada_w(c), m::FIELD_H);
        self.hit_index.register(c.id, r);
        dicas::rotulo(c.id, c.nome);
        let hover = self.store.hover_live(c.id) > 0.0;
        let corpo = if hover { t.sob_o_rato } else { t.corpo_da_peca };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, corpo);
        let mut cx = x + m::FIELD_PAD_R;
        let caixa = crate::widget::card_check_rect(cx, y, m::FIELD_H);
        crate::widget::paint_card_check_com_borda(
            self.scene,
            self.ink,
            caixa,
            c.on,
            hover,
            t.borda_da_caixa,
        );
        cx += m::CHECK_BOX + m::CHECK_GAP;
        let meio = y + m::FIELD_H * 0.5;
        if let Some(s) = c.antes {
            self.pinta_sinal(s, cx, meio);
            cx += SINAL + m::SEG_ICON_GAP;
        }
        let tw = self.largura_do_texto(c.nome, m::VALUE_PX);
        let texto = if c.on {
            t.inativa_texto
        } else {
            t.discreta_texto
        };
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            c.nome,
            cx,
            y + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
            m::VALUE_PX,
            tw,
            texto,
        );
        if let Some(s) = c.fim {
            self.pinta_sinal(s, cx + tw + m::SEG_ICON_GAP, meio);
        }
    }

    /// [`Self::botao_com_icone_em`] DISCRETO: sem fundo (só sob o rato), ícone e texto no tom
    /// secundário. Mesmo alvo.
    pub fn botao_com_icone_discreto_em(
        &mut self,
        id: NodeId,
        icon: IconId,
        texto: &str,
        r: Rect,
        t: TonsDaBarra,
    ) {
        self.hit_index.register(id, r);
        dicas::rotulo(id, texto);
        let corpo = if self.store.hover_live(id) > 0.0 {
            t.sob_o_rato
        } else {
            t.corpo_da_peca
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, corpo);
        let x = r.x + m::FIELD_PAD_R;
        let meio = r.y + r.h * 0.5;
        paint_icon(
            self.scene,
            icon,
            Rect::new(x, meio - SINAL * 0.5, SINAL, SINAL),
            t.discreta_texto,
            StrokeToken::Default.px(),
        );
        let tx = x + SINAL + m::SEG_ICON_GAP;
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            texto,
            tx,
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            (r.x + r.w - tx).max(0.0),
            t.discreta_texto,
        );
    }
}
