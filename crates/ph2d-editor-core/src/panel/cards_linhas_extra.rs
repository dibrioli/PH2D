//! ⭐ **As linhas de cartão de altura própria** (bloco B, painel Physics): a dica que quebra, a caixa
//! de marcar pelo [`CardCtx`] e a grelha triangular de ligar/desligar (o mapa de colisão da Unity e
//! da Godot). Não cabem em [`Widget`], cuja linha tem a altura de um campo.

use super::{CardCtx, Cell, Widget, corpo, rotulos};
use crate::icons::IconId;
use crate::interaction::toggle_paint;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_block, paint_text_elided,
};
use crate::widget::key_mark::Rotulo;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, StrokeToken};

#[path = "cards_linhas_voo.rs"]
pub(super) mod voo;

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

/// Uma linha de lista ou árvore (Tags, catálogos de Assets): o painel que era uma lista continua a
/// sê-lo, dentro de um cartão. As linhas tocam-se: não há vão entre elas.
#[derive(Copy, Clone, Debug)]
pub struct ListRow<'a> {
    pub id: NodeId,
    pub label: &'a str,
    /// O nível na árvore: recua pela porta `list_indent_px`.
    pub depth: usize,
    /// Um ícone antes do nome.
    pub icon: Option<IconId>,
    /// O facto à direita (quantos pertencem), esmaecido.
    pub trailing: Option<&'a str>,
    /// A linha em mãos.
    pub selected: bool,
    /// Onde o arrasto em curso largaria.
    pub target: bool,
    /// A linha que está a voar: fica esmaecida no lugar.
    pub dim: bool,
    /// O campo de renomear por cima do nome (este id); o facto à direita continua à vista.
    pub edit: Option<NodeId>,
}

/// Uma barra arrastável no tom do cartão: a mesa de som tem fileiras de barras onde um campo por
/// faixa não caberia. O store é o de sempre (`InteractiveState::Slider`), o despacho também.
#[derive(Copy, Clone, Debug)]
pub struct CardBar<'a> {
    pub id: NodeId,
    /// A posição, de 0 a 1.
    pub value: f32,
    pub vertical: bool,
    /// Riscos na pista, em posição de 0 a 1 (o 0 dB de um fader).
    pub ticks: &'a [f32],
}

impl<'a> ListRow<'a> {
    /// Uma linha sem nada a mais: raiz, sem ícone, sem facto, em repouso.
    #[must_use]
    pub fn new(id: NodeId, label: &'a str) -> Self {
        Self {
            id,
            label,
            depth: 0,
            icon: None,
            trailing: None,
            selected: false,
            target: false,
            dim: false,
            edit: None,
        }
    }
}

impl CardCtx<'_> {
    /// Uma frase que explica o gesto: quebra na largura do cartão e devolve o fundo MEDIDO.
    pub fn hint(&mut self, text: &str, y: f32) -> f32 {
        self.frase(text, ColorToken::CardLabel, y)
    }

    /// Uma recusa, na linha em que ela aconteceu: a frase da lei, na cor do perigo.
    pub fn problem(&mut self, text: &str, y: f32) -> f32 {
        self.frase(text, ColorToken::Danger, y)
    }

    fn frase(&mut self, text: &str, cor: ColorToken, y: f32) -> f32 {
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
            self.ink.color(cor),
        );
        super::censo::linha((self.x, self.w), top, top + used, Vec::new);
        top + used
    }

    /// Um botão destrutivo a toda a largura: o rótulo diz o tamanho do estrago, na cor do perigo.
    pub fn danger_button(&mut self, id: NodeId, text: &str, y: f32) -> f32 {
        let y = if self.first { y } else { y + m::FIELD_ROW_GAP };
        self.first = false;
        let r = Rect::new(self.x, y, self.w, m::FIELD_H);
        self.hit_index.register(id, r);
        self.anota_botao(text, r.w);
        super::censo::linha((self.x, self.w), y, y + m::FIELD_H, || {
            vec![super::censo::CelulaMedida {
                nome: text.to_owned(),
                curta: false,
                pede: 0.0,
            }]
        });
        let fundo = if self.store.hover_live(id) > 0.0 {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let tw = self.text_system.prefix_width(text, m::VALUE_PX).min(r.w);
        paint_text_elided(
            self.text_system,
            self.scene,
            text,
            r.x + (r.w - tw) * 0.5,
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            r.w,
            self.ink.color(ColorToken::Danger),
        );
        y + m::FIELD_H
    }

    /// Um interruptor com a cor própria quando ligado (o `M` vermelho, o `S` âmbar). `r` é do chamador.
    pub fn tone_button(&mut self, r: Rect, id: NodeId, text: &str, on: bool, tom: ColorToken) {
        self.hit_index.register(id, r);
        let (fundo, tinta) = if on {
            (tom, ColorToken::KanbanBtnOnFg)
        } else if self.store.hover_live(id) > 0.0 {
            (ColorToken::CardFieldHover, ColorToken::Text1)
        } else {
            (ColorToken::CardField, ColorToken::Text1)
        };
        fill_rounded_rect(self.scene, r, m::SEG_RADIUS, self.ink.color(fundo));
        let tw = self.text_system.prefix_width(text, m::SEG_TEXT_PX).min(r.w);
        paint_text_elided(
            self.text_system,
            self.scene,
            text,
            r.x + (r.w - tw) * 0.5,
            r.y + crate::paint::folga_do_texto(r.h, m::SEG_TEXT_PX),
            m::SEG_TEXT_PX,
            r.w,
            self.ink.color(tinta),
        );
    }

    /// ⭐ Uma barra no tom do cartão: pista, cheio até ao valor, riscos e polegar. O alvo é `r` inteiro.
    pub fn bar(&mut self, b: &CardBar<'_>, r: Rect) {
        self.hit_index.register(b.id, r);
        let v = b.value.clamp(0.0, 1.0);
        let t = m::BAR_TRACK;
        let (pista, cheio, polegar) = if b.vertical {
            let x = r.x + (r.w - t) * 0.5;
            // O polegar fica dentro do alvo: nos extremos não invade a vizinha.
            let topo = r.y + m::BAR_THUMB * 0.5 + (r.h - m::BAR_THUMB) * (1.0 - v);
            (
                Rect::new(x, r.y, t, r.h),
                Rect::new(x, topo, t, (r.y + r.h - topo).max(0.0)),
                Rect::new(r.x, topo - m::BAR_THUMB * 0.5, r.w, m::BAR_THUMB),
            )
        } else {
            let y = r.y + (r.h - t) * 0.5;
            let fim = r.x + m::BAR_THUMB * 0.5 + (r.w - m::BAR_THUMB) * v;
            (
                Rect::new(r.x, y, r.w, t),
                Rect::new(r.x, y, (fim - r.x).max(0.0), t),
                Rect::new(
                    fim - m::BAR_THUMB * 0.5,
                    r.y + (r.h - m::BAR_THUMB) * 0.5,
                    m::BAR_THUMB,
                    m::BAR_THUMB,
                ),
            )
        };
        let raio = t * 0.5;
        fill_rounded_rect(
            self.scene,
            pista,
            raio,
            self.ink.color(ColorToken::CardField),
        );
        fill_rounded_rect(self.scene, cheio, raio, self.ink.color(ColorToken::Accent));
        for &tick in b.ticks {
            let risco = if b.vertical {
                Rect::new(
                    r.x,
                    r.y + r.h * (1.0 - tick),
                    r.w,
                    StrokeToken::Default.px(),
                )
            } else {
                Rect::new(r.x + r.w * tick, r.y, StrokeToken::Default.px(), r.h)
            };
            fill_rounded_rect(
                self.scene,
                risco,
                0.0,
                self.ink.color(ColorToken::CardLabel),
            );
        }
        let quente = self.store.hover_live(b.id) > 0.0;
        fill_rounded_rect(
            self.scene,
            polegar,
            m::SEG_RADIUS,
            self.ink.color(if quente {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            }),
        );
    }

    /// Um medidor de nível (as cores são as do sinal, verde a vermelho, e não do cartão). `click` é o
    /// id de limpar o aviso de clip.
    pub fn meter(&mut self, click: NodeId, meter: &crate::widget::LevelMeter, r: Rect) {
        crate::widget::paint_level_meter(meter, r, self.scene, self.ink.theme);
        self.hit_index.register(click, r);
    }

    /// Uma caixa de texto num rect dado (a busca e o renomear da árvore, que não têm a coluna de `row`).
    pub fn text_box(&mut self, r: Rect, id: NodeId, placeholder: &str) {
        self.text(r, id, placeholder);
    }

    /// ⭐ Uma linha de lista ou árvore. Devolve o `y` seguinte, colado: as linhas tocam-se.
    pub fn list_row(&mut self, row: &ListRow<'_>, y: f32) -> f32 {
        self.first = false;
        let r = Rect::new(self.x, y, self.w, m::FIELD_H);
        self.hit_index.register(row.id, r);
        self.list_row_face(row, r);
        if row.target {
            // FRAME-RAW-OK: o alvo da largada, contornado no acento do cartão
            crate::paint::stroke_rounded_rect(
                self.scene,
                r,
                m::SEG_RADIUS,
                StrokeToken::Default.px(),
                self.ink.color(ColorToken::Accent),
            );
        }
        let recuo = ph2d_tokens::list_indent_px() * row.depth as f32;
        let x =
            r.x + recuo + m::SEG_INSET + row.icon.map_or(0.0, |_| m::SEG_TEXT_PX + m::SEG_ICON_GAP);
        let fim = r.x + r.w - m::SEG_INSET;
        // A coluna do facto: reservada por inteiro, para os nomes não dançarem.
        let reserva = row.trailing.map_or(0.0, |t| {
            self.text_system.prefix_width(t, m::LABEL_PX) + m::SEG_INSET
        });
        let (nome_w, tinta) = ((fim - x - reserva).max(0.0), self.tinta_da_linha(row));
        if let Some(edit) = row.edit {
            self.text(Rect::new(x, y, nome_w, m::FIELD_H), edit, "");
        } else {
            paint_text_elided(
                self.text_system,
                self.scene,
                row.label,
                x,
                y + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
                m::VALUE_PX,
                nome_w,
                tinta,
            );
        }
        if let Some(t) = row.trailing {
            let tw = self.text_system.prefix_width(t, m::LABEL_PX);
            paint_text(
                self.text_system,
                self.scene,
                t,
                fim - tw,
                y + crate::paint::folga_do_texto(m::FIELD_H, m::LABEL_PX),
                m::LABEL_PX,
                tw + m::SEG_INSET,
                self.ink.color(ColorToken::CardLabel),
            );
        }
        y + m::FIELD_H
    }

    fn tinta_da_linha(&self, row: &ListRow<'_>) -> ph2d_vector::Color {
        self.ink.color(if row.dim {
            ColorToken::CardLabel
        } else if row.selected {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        })
    }

    fn list_row_face(&mut self, row: &ListRow<'_>, r: Rect) {
        let fundo = if row.selected {
            Some(ColorToken::KanbanCardRaised)
        } else if self.store.hover_live(row.id) > 0.0 {
            Some(ColorToken::CardFieldHover)
        } else {
            None
        };
        if let Some(f) = fundo {
            fill_rounded_rect(self.scene, r, m::SEG_RADIUS, self.ink.color(f));
        }
        if let Some(icon) = row.icon {
            let lado = m::SEG_TEXT_PX;
            let recuo = ph2d_tokens::list_indent_px() * row.depth as f32;
            let tinta = self.tinta_da_linha(row);
            paint_icon(
                self.scene,
                icon,
                Rect::new(
                    r.x + recuo + m::SEG_INSET,
                    r.y + (r.h - lado) * 0.5,
                    lado,
                    lado,
                ),
                tinta,
                StrokeToken::Default.px(),
            );
        }
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
            self.legenda(
                &i.to_string(),
                self.x,
                y + crate::paint::folga_do_texto(lado, m::LABEL_PX),
                lado,
            );
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

/// As alças de uma lista que flutua: o id de sempre de cada gesto do painel.
#[derive(Copy, Clone, Debug)]
pub struct FloatHandles {
    pub drag: NodeId,
    pub resize_br: Option<NodeId>,
    pub resize_bl: NodeId,
}

/// ⭐ **A lista de um painel que FLUTUA** (Assets, Authored, Widget Lab): pinta no `rect` dele, a lista
/// abraça o conteúdo, e o rect do painel e as alças passam a ser os da lista, senão a roda e o clique
/// morreriam no vazio abaixo dela. O «…» e o «×» ficam à direita da faixa de arrasto.
pub fn paint_floating_list(
    ctx: &mut crate::panel::PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[super::CardDef<'_>],
    rect: Rect,
    h: FloatHandles,
) -> Rect {
    use crate::widget::panel_chrome as chrome;
    let encaixe = ctx.slot;
    ctx.slot = rect;
    let lista = super::paint_card_list(ctx, panel, title, cards);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(panel, lista);
    let theme = ctx.host.theme();
    chrome::paint_panel_corner_dot_bl(lista, ctx.scene, theme);
    let hit = ctx.host.hit_index_mut();
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    hit.register(
        h.drag,
        chrome::panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    if let Some(br) = h.resize_br {
        hit.register(br, chrome::panel_resize_handle_rect(lista));
    }
    hit.register(h.resize_bl, chrome::panel_resize_handle_rect_bl(lista));
    lista
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

    /// As linhas tocam-se, recuam pelo nível, registam o id; o campo de renomear fica por cima do
    /// nome e deixa o facto à direita livre; o voo não regista clique.
    #[test]
    fn a_list_row_is_flush_indents_and_its_edit_box_spares_the_trailing_fact() {
        let mut f = mesa();
        let mut c = ctx(&mut f, 200.0);
        let linha = ListRow {
            trailing: Some("12"),
            ..ListRow::new(NodeId(1), "Enemy")
        };
        let a = c.list_row(&linha, 0.0);
        assert_eq!(a, m::FIELD_H);
        let b = c.list_row(
            &ListRow {
                id: NodeId(2),
                depth: 2,
                edit: Some(NodeId(3)),
                ..linha
            },
            a,
        );
        assert_eq!(b, a + m::FIELD_H, "entre linhas de lista não há vão");
        let antes = c.scene.inner().encoding().n_path_segments;
        c.list_row_flight(&linha, 10.0, 10.0);
        assert_eq!(
            c.scene.inner().encoding().n_path_segments,
            antes,
            "o voo pintou-se dentro do recorte do cartão"
        );
        voo::pinta_os_voos(c.scene);
        assert!(
            c.scene.inner().encoding().n_path_segments > antes,
            "o voo perdeu-se"
        );
        let rects: Vec<(NodeId, Rect)> = f.hits.iter_registrations().collect();
        assert_eq!(rects.len(), 3, "duas linhas e um campo; o voo não regista");
        let campo = rects.iter().find(|(id, _)| *id == NodeId(3)).unwrap().1;
        assert!(
            (campo.x - (2.0 * ph2d_tokens::list_indent_px() + m::SEG_INSET)).abs() < 0.01,
            "o campo recua com o nível: {campo:?}"
        );
        assert!(
            campo.x + campo.w < 200.0 - m::SEG_INSET,
            "tapou o facto: {campo:?}"
        );
    }

    /// A barra regista o alvo inteiro; o polegar acompanha o valor nos dois sentidos, e fica no sítio
    /// onde o despacho lê o mesmo valor: de baixo para cima na vertical.
    #[test]
    fn a_bar_registers_its_whole_rect_and_paints_within_it() {
        for vertical in [false, true] {
            let mut f = mesa();
            let mut c = ctx(&mut f, 200.0);
            let r = Rect::new(4.0, 8.0, 40.0, 100.0);
            let b = CardBar {
                id: NodeId(1),
                value: 0.5,
                vertical,
                ticks: &[0.25],
            };
            c.bar(&b, r);
            assert_eq!(f.hits.iter_registrations().next(), Some((NodeId(1), r)));
        }
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
