//! ⭐⭐ **O painel como LISTA DE CARTÕES** (`docs/CartoesUI`, D3–D6, D16): a lista flutua sobre o
//! canvas (sem fundo de dock) e abraça o conteúdo; cada secção é um cartão do Kanban que se
//! arrasta pelo cabeçalho (o `ph2d_card_stack::Stack`: vão, voo inclinado, assentar com mola);
//! dentro do cartão o layout é o do Figma (rótulo com unidade acima, campo ícone + valor, marca de
//! animação na linha do rótulo, ações = ícones no cabeçalho).
//!
//! Tema, contorno e notas por secção (D6; o solto tem os da sua face, T63) pelo menu da pega — o
//! cabeçalho É a pega. A geometria vive em `ph2d_card_stack::metrics`. ⚠️ **As alturas
//! são as do quadro ANTERIOR**: cada cartão mede-se ao pintar e a mola leva a diferença ao seguinte.

use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, resolve};
use crate::panel::PaintCtx;
use crate::widget::card_field::{self, Glyph};
use crate::widget::card_ink::CardInk;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_card_stack::{Item, reorder};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, VectorScene};

/// Um ícone de ação no cabeçalho do cartão (T2).
#[derive(Copy, Clone, Debug)]
pub struct HeadAction {
    pub id: NodeId,
    pub icon: IconId,
    /// Aceso (o 👁 de um limite visível, o alfinete preso).
    pub on: bool,
}

/// O corpo de um cartão: pinta a partir de `y` e devolve o fundo.
pub type CardBody<'a> = Box<dyn Fn(&mut CardCtx<'_>, f32) -> f32 + 'a>;

/// Um cartão (uma secção).
pub struct CardDef<'a> {
    pub section: NodeId,
    pub title: &'a str,
    pub actions: Vec<HeadAction>,
    /// Secção AUSENTE: título esmaecido e «+» (este id) que a cria (T3).
    pub add: Option<NodeId>,
    /// O resumo da secção dobrada, à direita do título (T4).
    pub summary: Option<String>,
    pub body: Option<CardBody<'a>>,
}

/// Um segmento de uma escolha: `(id, ícone, texto, aceso)`.
#[derive(Copy, Clone, Debug)]
pub struct Seg<'a> {
    pub id: NodeId,
    pub icon: Option<IconId>,
    pub text: &'a str,
    pub on: bool,
}

/// O que ocupa uma coluna de uma linha.
#[derive(Copy, Clone, Debug)]
pub enum Widget<'a> {
    /// O campo Figma (o painel marca-o com `WidgetStore::mark_card_field` ANTES de pintar).
    Field { id: NodeId, glyph: Glyph<'a> },
    /// Uma escolha segmentada que estica.
    Choice(&'a [Seg<'a>]),
    /// Uma escolha de ÍCONES de largura fixa (um quadrado por segmento).
    Icons(&'a [Seg<'a>]),
    /// Um botão; `primary` = o de destaque (azul).
    Button {
        id: NodeId,
        text: &'a str,
        primary: bool,
    },
    /// A cara de um campo que um clique ABRE (a opacidade de uma camada fechada): glifo e valor.
    FieldFace {
        id: NodeId,
        glyph: Glyph<'a>,
        text: &'a str,
    },
    /// Um campo de TEXTO (o nome do objeto): o `TextInput` de hoje dentro da caixa do cartão.
    Text { id: NodeId, placeholder: &'a str },
    /// Um ícone fixo numa caixa quadrada (o tipo do objeto) — não é clicável.
    Badge(IconId),
    /// Ícones soltos, sem caixa (👁 · 🔒, girar 90° · espelhar, a corrente) — um quadrado cada.
    Bare(&'a [Seg<'a>]),
    /// A amostra de cor + o código (Fill/Stroke, 02 §2.7): o clique na amostra abre o seletor; com
    /// `edit`, o código é uma caixa de texto (esse id) — digitar o hex é escolher a cor.
    Swatch {
        id: NodeId,
        rgba: Option<[u8; 4]>,
        text: &'a str,
        edit: Option<NodeId>,
        /// D30: a camada ligada a um estilo mostra o NOME dele e a corrente partida (este id) solta-a.
        detach: Option<NodeId>,
    },
    /// Uma lista suspensa: a caixa com o valor e a seta; as opções abrem por baixo (por cima de tudo).
    Dropdown {
        id: NodeId,
        text: &'a str,
        options: &'a [Seg<'a>],
    },
}

/// Uma coluna: rótulo opcional (com a marca de animação dos `key` quando há), e o widget.
#[derive(Copy, Clone, Debug)]
pub struct Cell<'a> {
    pub label: Option<&'a str>,
    pub key: Option<&'a [NodeId]>,
    pub widget: Widget<'a>,
}

impl<'a> Cell<'a> {
    #[must_use]
    pub fn new(widget: Widget<'a>) -> Self {
        Self {
            label: None,
            key: None,
            widget,
        }
    }
    #[must_use]
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    /// Uma propriedade animável: a marca sobre a borda direita da caixa (D17).
    #[must_use]
    pub fn key(mut self, fields: &'a [NodeId]) -> Self {
        self.key = Some(fields);
        self
    }
}

/// O contexto de pintura de DENTRO de um cartão.
pub struct CardCtx<'s> {
    pub scene: &'s mut VectorScene,
    pub text_system: &'s mut TextSystem,
    pub store: &'s WidgetStore,
    pub hit_index: &'s mut HitIndex,
    /// A tinta do cartão (o tema dele, ou o Kanban da lista).
    pub ink: CardInk,
    pub x: f32,
    pub w: f32,
    first: bool,
}

impl CardCtx<'_> {
    /// ⭐ **Uma linha**: os rótulos (se algum) e os widgets lado a lado.
    pub fn row(&mut self, cells: &[Cell<'_>], y: f32) -> f32 {
        let rotulos = cells.iter().any(|c| c.label.is_some());
        let mut y = if rotulos {
            y + m::LABEL_TOP
        } else if self.first {
            y
        } else {
            y + m::FIELD_ROW_GAP
        };
        self.first = false;
        let cols = self.columns(cells);
        let grupo = cells.iter().find_map(|c| c.label);
        for c in cells {
            if let Widget::Field { id, .. } = c.widget
                && let Some(l) = c.label.or(grupo)
            {
                dicas::rotulo(id, l);
            }
        }
        if rotulos {
            y += rotulos::faixa(self, cells, &cols, y);
        }
        for (c, (x, w)) in cells.iter().zip(&cols) {
            let r = Rect::new(*x, y, *w, m::FIELD_H);
            match c.widget {
                Widget::Field { id, glyph } => card_field::paint_field(
                    self.scene,
                    self.text_system,
                    self.ink,
                    self.store,
                    self.hit_index,
                    r,
                    id,
                    glyph,
                ),
                Widget::Choice(segs) | Widget::Icons(segs) => self.choice(r, segs),
                Widget::FieldFace { id, glyph, text } => card_field::paint_field_face(
                    self.scene,
                    self.text_system,
                    self.ink,
                    self.store,
                    self.hit_index,
                    r,
                    id,
                    glyph,
                    text,
                ),
                Widget::Button { id, text, primary } => self.button(r, id, text, primary),
                Widget::Text { id, placeholder } => self.text(r, id, placeholder),
                Widget::Badge(icon) => {
                    fill_rounded_rect(
                        self.scene,
                        r,
                        m::FIELD_RADIUS,
                        self.ink.color(ColorToken::CardField),
                    );
                    let g = m::HEAD_ICON_GLYPH;
                    paint_icon(
                        self.scene,
                        icon,
                        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
                        self.ink.color(ColorToken::Text2),
                        StrokeToken::Default.px(),
                    );
                }
                Widget::Bare(segs) => self.bare(r, segs),
                Widget::Swatch {
                    id,
                    rgba,
                    text,
                    edit,
                    detach,
                } => self.swatch(r, id, rgba, text, edit, detach),
                Widget::Dropdown { id, text, options } => self.dropdown(r, id, text, options),
            }
        }
        y + m::FIELD_H
    }

    /// ⚠️ **A ponte para uma linha do vocabulário de hoje** (`RowCtx`: a lista da ponta do osso, a do
    /// Smart Bone), na largura do cartão. Devolve o fundo.
    pub fn legacy(
        &mut self,
        y: f32,
        f: impl FnOnce(&mut crate::panel::RowCtx<'_>, f32) -> f32,
    ) -> f32 {
        let y = if self.first { y } else { y + m::FIELD_ROW_GAP };
        self.first = false;
        let mut r = crate::panel::RowCtx {
            scene: &mut *self.scene,
            text_system: &mut *self.text_system,
            store: self.store,
            hit_index: &mut *self.hit_index,
            theme: self.ink.theme,
            inner_x: self.x,
            inner_w: self.w,
            row_h: m::FIELD_H,
            row_gap: 0.0,
            font: m::VALUE_PX,
            open_fold: None,
        };
        f(&mut r, y)
    }

    /// Uma linha de um widget só a toda a largura, com rótulo opcional.
    pub fn full(&mut self, label: Option<&str>, widget: Widget<'_>, y: f32) -> f32 {
        let mut c = Cell::new(widget);
        c.label = label;
        self.row(&[c], y)
    }

    fn columns(&mut self, cells: &[Cell<'_>]) -> Vec<(f32, f32)> {
        let ts = &mut *self.text_system;
        let mut mede = |t: &str| ts.prefix_width(t, m::VALUE_PX);
        colunas::colunas(self.x, self.w, cells, &mut mede)
    }

    fn seg_need(&mut self, s: &Seg<'_>) -> (f32, bool) {
        let tw = if s.text.is_empty() {
            0.0
        } else {
            self.text_system.prefix_width(s.text, m::SEG_TEXT_PX)
        };
        let iw = if s.icon.is_some() { m::ICON_GLYPH } else { 0.0 };
        let gap = if iw > 0.0 && tw > 0.0 {
            m::SEG_ICON_GAP
        } else {
            0.0
        };
        (iw + gap + tw + m::SEG_INSET * 2.0, tw == 0.0)
    }

    fn choice(&mut self, r: Rect, segs: &[Seg<'_>]) {
        fill_rounded_rect(
            self.scene,
            r,
            m::FIELD_RADIUS,
            self.ink.color(ColorToken::CardField),
        );
        let n = segs.len().max(1) as f32;
        let inner = Rect::new(
            r.x + m::SEG_PAD,
            r.y + m::SEG_PAD,
            r.w - m::SEG_PAD * 2.0,
            r.h - m::SEG_PAD * 2.0,
        );
        let needs: Vec<(f32, bool)> = segs.iter().map(|s| self.seg_need(s)).collect();
        let ws = m::seg_widths(&needs, inner.w - m::SEG_PAD * (n - 1.0));
        let mut sx = inner.x;
        for (s, w) in segs.iter().zip(ws) {
            let sr = Rect::new(sx, inner.y, w, inner.h);
            sx += w + m::SEG_PAD;
            self.hit_index.register(s.id, sr);
            if s.text.is_empty() && s.icon.is_some() {
                dicas::icone(s.id);
            }
            let hover = self.store.hover_live(s.id) > 0.0;
            if s.on || hover {
                let cor = if s.on {
                    ColorToken::KanbanCardRaised
                } else {
                    ColorToken::CardFieldHover
                };
                fill_rounded_rect(self.scene, sr, m::SEG_RADIUS, self.ink.color(cor));
            }
            let tinta = self.ink.color(if s.on {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            });
            let tw = if s.text.is_empty() {
                0.0
            } else {
                self.text_system.prefix_width(s.text, m::SEG_TEXT_PX)
            };
            let iw = if s.icon.is_some() { m::ICON_GLYPH } else { 0.0 };
            let gap = if iw > 0.0 && tw > 0.0 {
                m::SEG_ICON_GAP
            } else {
                0.0
            };
            let total = (iw + gap + tw).min(sr.w);
            let mut x = sr.x + (sr.w - total) * 0.5;
            if let Some(icon) = s.icon {
                let ir = Rect::new(x, sr.y + (sr.h - iw) * 0.5, iw, iw);
                paint_icon(self.scene, icon, ir, tinta, StrokeToken::Default.px());
                x += iw + gap;
            }
            if tw > 0.0 {
                crate::paint::paint_text_elided(
                    self.text_system,
                    self.scene,
                    s.text,
                    x,
                    sr.y + (sr.h - m::SEG_TEXT_PX) * 0.5,
                    m::SEG_TEXT_PX,
                    (sr.x + sr.w - x).max(0.0),
                    tinta,
                );
            }
        }
    }

    fn text(&mut self, r: Rect, id: NodeId, placeholder: &str) {
        use crate::interaction::InteractiveState;
        use crate::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
        self.hit_index.register(id, r);
        let (state, text, caret, anchor) = match self.store.get(id) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
            _ => (TextInputState::Normal, None, 0, None),
        };
        // Em repouso, a mesma caixa dos outros campos (maquete C); a editar, a caixa de texto.
        if matches!(state, TextInputState::Normal | TextInputState::Hovered) {
            let fundo = if self.store.hover_live(id) > 0.0 {
                ColorToken::CardFieldHover
            } else {
                ColorToken::CardField
            };
            fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
            let (s, tinta) = match text.filter(|t| !t.is_empty()) {
                Some(t) => (t, ColorToken::Text1),
                None => (placeholder, ColorToken::Text3),
            };
            crate::paint::paint_text_elided(
                self.text_system,
                self.scene,
                s,
                r.x + m::FIELD_PAD_R,
                r.y + (r.h - m::VALUE_PX) * 0.5,
                m::VALUE_PX,
                (r.w - m::FIELD_PAD_R * 2.0).max(0.0),
                self.ink.color(tinta),
            );
            return;
        }
        let input = TextInput::new(id, "")
            .placeholder(placeholder)
            .visual((state, self.store.hover_live(id)));
        paint_text_input_with_buffer(
            &input,
            text,
            Some(caret),
            anchor,
            r,
            self.scene,
            self.text_system,
            self.ink.theme,
        );
    }

    /// Ícones sem caixa: o aceso ganha o fundo do segmento ativo; o pairado, o do hover.
    fn bare(&mut self, r: Rect, segs: &[Seg<'_>]) {
        for (i, s) in segs.iter().enumerate() {
            let b = Rect::new(r.x + m::FIELD_H * i as f32, r.y, m::FIELD_H, m::FIELD_H);
            self.hit_index.register(s.id, b);
            if s.text.is_empty() && s.icon.is_some() {
                dicas::icone(s.id);
            }
            if s.on || self.store.hover_live(s.id) > 0.0 {
                fill_rounded_rect(
                    self.scene,
                    b,
                    m::FIELD_RADIUS,
                    self.ink.color(ColorToken::CardFieldHover),
                );
            }
            if let Some(icon) = s.icon {
                let g = m::ICON_GLYPH;
                paint_icon(
                    self.scene,
                    icon,
                    Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
                    self.ink.color(if s.on {
                        ColorToken::Text1
                    } else {
                        ColorToken::Text2
                    }),
                    StrokeToken::Default.px(),
                );
            }
        }
    }

    fn swatch(
        &mut self,
        r: Rect,
        id: NodeId,
        rgba: Option<[u8; 4]>,
        text: &str,
        edit: Option<NodeId>,
        detach: Option<NodeId>,
    ) {
        use crate::interaction::InteractiveState;
        use crate::widget::TextInputState;
        let fundo = if edit.is_some_and(|e| self.store.hover_live(e) > 0.0) {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let solta = detach.map_or(0.0, |_| m::ICON_CELL - m::FIELD_PAD_R);
        if let Some(d) = detach {
            let b = Rect::new(r.x + r.w - m::ICON_CELL, r.y, m::ICON_CELL, r.h);
            self.hit_index.register(d, b);
            dicas::icone(d);
            let g = m::ICON_GLYPH;
            paint_icon(
                self.scene,
                IconId::Unlink,
                Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
                self.ink.color(if self.store.hover_live(d) > 0.0 {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                }),
                StrokeToken::Default.px(),
            );
        }
        let r = Rect::new(r.x, r.y, (r.w - solta).max(0.0), r.h);
        let codigo = Rect::new(r.x + m::ICON_CELL, r.y, (r.w - m::ICON_CELL).max(0.0), r.h);
        let editando = edit.filter(|e| {
            matches!(
                self.store.get(*e),
                Some(InteractiveState::TextInput { state, .. })
                    if !matches!(state, TextInputState::Normal | TextInputState::Hovered)
            )
        });
        let g = m::ICON_GLYPH;
        let amostra = Rect::new(r.x + (m::ICON_CELL - g) * 0.5, r.y + (r.h - g) * 0.5, g, g);
        self.hit_index
            .register(id, Rect::new(r.x, r.y, m::ICON_CELL, r.h));
        if let Some(c) = rgba {
            fill_rounded_rect(
                self.scene,
                amostra,
                m::SEG_RADIUS,
                ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], c[3]),
            );
        }
        if let Some(e) = edit {
            self.hit_index.register(e, codigo);
        }
        if let Some(e) = editando {
            self.text(codigo, e, text);
            return;
        }
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            text,
            r.x + m::ICON_CELL,
            r.y + (r.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            (r.w - m::ICON_CELL - m::FIELD_PAD_R).max(0.0),
            self.ink.color(ColorToken::Text1),
        );
    }

    fn dropdown(&mut self, r: Rect, id: NodeId, text: &str, options: &[Seg<'_>]) {
        self.hit_index.register(id, r);
        let aceso = self.store.hover_live(id) > 0.0;
        corpo::chip(self.scene, self.text_system, self.ink, r, text, aceso);
        if dropdown::open() == Some(id) {
            dropdown::stash(
                r,
                self.ink,
                options
                    .iter()
                    .map(|s| (s.id, s.text.to_string(), s.on))
                    .collect(),
            );
        }
    }

    fn button(&mut self, r: Rect, id: NodeId, text: &str, primary: bool) {
        self.hit_index.register(id, r);
        let hover = self.store.hover_live(id) > 0.0;
        let (fundo, tinta) = if primary {
            (ColorToken::KanbanBlueBold, ColorToken::KanbanBtnOnFg)
        } else if hover {
            (ColorToken::CardFieldHover, ColorToken::Text1)
        } else {
            (ColorToken::CardField, ColorToken::Text1)
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let tw = self.text_system.prefix_width(text, m::VALUE_PX).min(r.w);
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            text,
            r.x + (r.w - tw) * 0.5,
            r.y + (r.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            r.w,
            self.ink.color(tinta),
        );
    }
}

#[path = "cards_colunas.rs"]
mod colunas;
#[path = "cards_corpo.rs"]
pub mod corpo;
#[path = "cards_entra_sai.rs"]
pub mod entra_sai;
#[path = "cards_estilos.rs"]
pub mod estilos;
#[path = "cards_menu.rs"]
mod menu;
#[path = "cards_rolagem.rs"]
mod rolagem;
#[path = "cards_rotulos.rs"]
mod rotulos;
#[path = "cards_solto.rs"]
mod solto;
#[path = "cards_solto_some.rs"]
pub mod solto_some;
pub use menu::{apply_list_menu_click, list_menu_of};

/// O id da pega (o cabeçalho) de um cartão — o mesmo das secções de hoje.
#[must_use]
pub fn grip_of(section: NodeId) -> NodeId {
    crate::ids::grip_de(section)
}

/// ⭐ C5: a linha-fantasma (traz o solto à vista) e «Return to panel» — ANTES dos painéis, que
/// consomem todo `Click` (o Bones manda-os à ferramenta). Devolve se consumiu.
pub fn apply_loose_click(store: &mut WidgetStore, id: NodeId) -> bool {
    solto::clique(store, id)
}

/// ⭐⭐ **Pinta o painel como lista de cartões** no `ctx.slot`. Devolve o rect da lista.
pub fn paint_card_list(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
) -> Rect {
    paint_card_list_with(ctx, panel, title, cards, None)
}

/// O pé da lista: `(id, texto)` — o «+ Add component» do Inspector (T5).
#[derive(Copy, Clone, Debug)]
pub struct Footer<'a> {
    pub id: NodeId,
    pub text: &'a str,
    /// D25: repetir o «+» no cabeçalho da lista (o Inspector só em modo Object, D25b).
    pub at_top: bool,
}

/// [`paint_card_list`] com o pé da lista.
pub fn paint_card_list_with(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
    footer: Option<Footer<'_>>,
) -> Rect {
    paint_card_list_pinned(ctx, panel, title, cards, footer, &[])
}

/// [`paint_card_list_with`] e os cartões dos soltos PRESOS a outro sujeito (T40): a lista mostra
/// os da seleção e o solto o dele — a mesma secção nos dois sítios.
pub fn paint_card_list_pinned(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
    footer: Option<Footer<'_>>,
    pinned: &[CardDef<'_>],
) -> Rect {
    let theme = ctx.host.theme();
    dicas::comeca(panel);
    let slot = ctx.slot;
    let x = slot.x + m::LIST_MARGIN;
    let w = (slot.w - m::LIST_MARGIN * 2.0).max(0.0);
    // A barra de opções da ferramenta atravessa a janela por cima dos painéis: a lista começa abaixo.
    let barra = ctx.layout.tool_bar;
    let topo = if barra.h > 0.0 {
        slot.y.max(barra.y + barra.h)
    } else {
        slot.y
    };
    // D31: e abaixo da régua de cima (a lista flutua sobre a área de desenho, D24).
    let topo = ctx
        .host
        .content_area()
        .filter(|c| c.h > 0.0)
        .map_or(topo, |c| topo.max(c.y));
    let top_list = topo + m::LIST_MARGIN;
    let body_top = top_list + m::LIST_HEAD_H;
    let keys: Vec<NodeId> = cards.iter().map(|c| c.section).collect();
    let store = ctx.host.store_mut();
    let cw = rolagem::largura_dos_cartoes(store, panel, x, w);
    // A rolagem (a lista que passa da janela): a origem dos cartões sobe o que rolou.
    let origem = body_top - store.panel_scroll(panel).max(0.0);
    for k in &keys {
        store.mark_card_section(*k);
    }
    // A ordem: a das secções (`sections.txt`), sobre a ordem natural do painel.
    let mut ordem = crate::widget::section_plan::ordem(&keys, store);
    let drop = store.take_card_drop(panel, &keys);
    let drag = store
        .card_drag()
        .filter(|d| d.active && store.card_drag_is_of(d, panel, &keys));
    // ⭐ C5: a mão fora da COLUNA leva o cartão ao canvas; o solto que volta a uma lista que o mostra
    // é levantado como voo dela. Aba atrás: a lista não se vê, só os soltos continuam no canvas.
    let escondida = store.loose_only_pass() == Some(panel);
    let na_lista = |p: (f32, f32)| !escondida && p.0 >= x && p.0 <= x + w;
    let canvas = ctx.layout.canvas;
    let era_solto = |s: &WidgetStore, k: NodeId| s.is_loose_in(k, panel);
    let mut reentra = None;
    if let Some(d) = drag
        && d.loose
        && era_solto(store, d.section)
    {
        let cabe = na_lista(d.at) && keys.contains(&store.loose_section(d.section));
        reentra = solto::arrasta(store, d, cabe, canvas);
    }
    let drop = match drop {
        Some(d) if d.loose && era_solto(store, d.section) => {
            store.settle_loose_card(d.section);
            None
        }
        outra => outra,
    };
    let drag = drag.filter(|d| !d.loose);
    if let Some(d) = drag.filter(|d| na_lista(d.at)) {
        rolagem::na_borda(store, panel, d.at.1, body_top);
    }
    let todos: Vec<NodeId> = keys
        .iter()
        .copied()
        .filter(|k| era_solto(store, *k))
        .collect();
    solto::prepara(store, &todos, cards);
    estilos::prepara(store);
    if escondida {
        solto::pinta_os_soltos(ctx, panel, theme, canvas, cards, pinned);
        return Rect::new(x, top_list, 0.0, 0.0);
    }
    // T40: o solto preso a OUTRO sujeito não deixa fantasma — a lista mostra a secção da seleção.
    let soltos = solto::fantasmas(store, panel, &todos);
    let cx = x + m::LIST_PAD_X;
    let chegadas: Vec<(NodeId, (f32, f32))> = keys
        .iter()
        .filter_map(|k| {
            store
                .take_card_arrival(*k)
                .map(|a| (*k, (a.0 - cx, a.1 - origem)))
        })
        .collect();
    let law = crate::motion::UiMotion::default().law(crate::motion::Role::Travel);
    let list = store.card_list_mut(panel);
    let now = std::time::Instant::now();
    let dt = list
        .last
        .map_or(0.0, |t| now.duration_since(t).as_secs_f64().min(0.1));
    list.last = Some(now);
    let soltas_h = list.loose_notes_h;
    // A queda ANTES da disposição: o cartão assenta já no lugar novo, sem passar pelo velho.
    let mut nova_ordem = None;
    let mut solta = None;
    if let Some(d) = drop {
        if d.cancelled || list.stack.flight().is_some_and(|f| soltos.contains(&f.key)) {
            list.stack.cancel(origem);
        } else if list.stack.flight().is_some_and(|f| f.to.is_none()) {
            solta = list.stack.detach();
        } else if let Some((k, to)) = list.stack.land(origem) {
            reorder(&mut ordem, &k, to);
            nova_ordem = Some(ordem.clone());
        }
    }
    // As alturas com mola (T41: dobrar, desdobrar, entrar e sair não saltam).
    let mut items = entra_sai::itens(panel, &mut list.stack, &list.items, &ordem, &soltos);
    let crescem = list.stack.smooth_heights(&mut items, dt, law);
    if let Some((d, (grab, size))) = reentra {
        let i = items.iter().position(|i| i.key == d.section).unwrap_or(0);
        list.stack.lift(d.section, d.at, grab, size, i);
    }
    let drag = drag.or(reentra.map(|(d, _)| d));
    if let Some(d) = drag {
        if list.stack.flight().is_none() {
            let mut y = origem;
            for it in &items {
                if it.key == d.section {
                    let i = items.iter().position(|i| i.key == d.section).unwrap_or(0);
                    list.stack.lift(
                        d.section,
                        d.down,
                        (d.down.0 - x, d.down.1 - y),
                        (cw, it.h),
                        i,
                    );
                    break;
                }
                y += it.h + m::CARD_GAP;
            }
        }
        list.stack
            .drag(&items, m::CARD_GAP, d.at, na_lista(d.at).then_some(origem));
    }
    for (k, de) in chegadas {
        list.stack.arrive(k, de);
    }
    let placed = list.stack.place(&items, m::CARD_GAP, dt, law);
    let regressam: Vec<(NodeId, f32)> = items
        .iter()
        .filter_map(|i| list.stack.arriving_dx(i.key).map(|d| (i.key, d)))
        .collect();
    let voo = list.stack.flight().copied();
    let tilt = list.stack.tilt_deg();
    solto::larga(store, panel, drop, solta);
    if let Some(o) = nova_ordem {
        let atual = store.section_order().to_vec();
        let mut todas: Vec<NodeId> = atual.into_iter().filter(|k| !keys.contains(k)).collect();
        todas.extend(o);
        store.set_section_order(todas);
    }

    // ── A lista: flutua, abraça o conteúdo até à altura do encaixe (D16) ──
    let pe = if footer.is_some() {
        m::LIST_FOOTER_H
    } else {
        0.0
    };
    let content_h = placed.content_h + soltas_h + pe;
    let natural = m::LIST_HEAD_H + content_h + m::LIST_PAD_BOTTOM;
    let tecto = slot.y + slot.h - top_list - m::LIST_MARGIN;
    let list_h = natural.min(tecto);
    let lista = Rect::new(x, top_list, w, list_h.max(m::LIST_HEAD_H));
    // ⭐ D24: a lista FLUTUA — publica o rect DELA (o clique abaixo vai ao canvas) sem reservar a
    // coluna; a costura de largura passa para a beirada dela (`DockSides::with_floating`).
    ctx.host.store_mut().set_floating_panel_rect(panel, lista);
    shadow(
        ctx.scene,
        lista,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanListShadow, theme),
        m::LIST_SHADOW_FAR,
    );
    shadow(
        ctx.scene,
        lista,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanShadowNear, theme),
        m::LIST_SHADOW_NEAR,
    );
    fill_rounded_rect(
        ctx.scene,
        lista,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanList, theme),
    );
    crate::paint::paint_text_title_elided_tracked(
        ctx.text_system,
        ctx.scene,
        &title.to_uppercase(),
        [
            x + m::LIST_PAD_X + m::LIST_HEAD_PAD_X,
            top_list + (m::LIST_HEAD_H - m::LIST_TITLE_PX) * 0.5,
        ],
        m::LIST_TITLE_PX,
        w - m::LIST_PAD_X * 2.0
            - m::HEAD_ICON_BOX
                * if footer.is_some_and(|f| f.at_top) {
                    2.0
                } else {
                    1.0
                },
        resolve(ColorToken::KanbanListFg, theme),
        m::LIST_TITLE_TRACKING_EM,
    );
    // ⭐ D21: o ícone de menu do painel (Reset Positions · Reset Colors · Reset All).
    menu::pinta(
        ctx,
        panel,
        theme,
        Rect::new(x, top_list, w, m::LIST_HEAD_H),
        footer,
    );

    // ── Os cartões ── (T43: o corpo, e com ele a pista da barra, pára acima da margem do fundo)
    let clip = Rect::new(
        x,
        body_top,
        w,
        (lista.y + lista.h - m::LIST_PAD_BOTTOM - body_top).max(0.0),
    );
    let mut voltam = Vec::new();
    let mut alturas: Vec<Item<NodeId>> = Vec::with_capacity(cards.len());
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let area = crate::widget::scroll_area::open_with(
        ctx.scene,
        hit_index,
        store,
        panel,
        scroll_bar_of(panel),
        clip,
    );
    if let Some((sy, sh)) = placed.slot {
        let vao = Rect::new(cx, origem + sy, cw, sh);
        fill_rounded_rect(
            ctx.scene,
            vao,
            m::CARD_RADIUS,
            resolve(ColorToken::KanbanSlot, theme),
        );
    }
    for (k, y) in &placed.ys {
        let h_antes = items.iter().find(|i| i.key == *k).map_or(0.0, |i| i.h);
        let r = Rect::new(cx, origem + y, cw, h_antes.max(0.0));
        let Some(card) = cards.iter().find(|c| c.section == *k) else {
            entra_sai::pinta_saida(ctx.scene, panel, *k, Rect { h: h_antes, ..r });
            continue;
        };
        if let Some(dx) = regressam.iter().find(|(c, _)| c == k).map(|(_, d)| *d) {
            voltam.push((Rect::new(r.x + dx, r.y, r.w, r.h), card));
            continue;
        }
        if soltos.contains(k) {
            solto::fantasma(ctx.scene, ctx.text_system, store, hit_index, theme, r, card);
            alturas.push(Item {
                key: *k,
                h: m::LOOSE_GHOST_H,
            });
            continue;
        }
        // O cartão cabe na altura VISTA: a crescer ou a encolher, o corpo recorta-se nela.
        let o = if crescem.contains(k) {
            0.0
        } else {
            m::CARD_OUTLINE_GAP + StrokeToken::Thick.px()
        };
        let corte = Rect::new(r.x - o, r.y - o, r.w + o * 2.0, r.h + o * 2.0);
        ctx.scene.push_clip(&crate::paint::rect_to_vello(corte));
        hit_index.push_clip(corte);
        let h = entra_sai::fotografa(ctx.scene, panel, *k, r, |s| {
            paint_card(
                s,
                ctx.text_system,
                store,
                hit_index,
                panel,
                theme,
                r,
                card,
                true,
            )
        });
        hit_index.pop_clip();
        ctx.scene.pop_layer();
        alturas.push(Item { key: *k, h });
    }
    // As notas sem cartão (criadas sobre a lista, fora de um cartão) e o fantasma da nota
    // arrastada: sem isto a lista não se declara anfitriã e o botão direito não oferece «Create Note».
    let topo_soltas = origem + placed.content_h;
    let fim_soltas = crate::widget::showcase::notes_chrome::pinta_as_que_sobram(
        ctx.scene,
        ctx.text_system,
        hit_index,
        store,
        panel,
        theme,
        cx,
        cw,
        topo_soltas,
    );
    let notas_h = (fim_soltas - topo_soltas).max(0.0);
    if let Some(f) = footer {
        let fr = Rect::new(cx, topo_soltas + soltas_h, cw, m::LIST_FOOTER_H);
        hit_index.register(f.id, fr);
        if store.hover_live(f.id) > 0.0 {
            fill_rounded_rect(
                ctx.scene,
                fr,
                m::FIELD_RADIUS,
                resolve(ColorToken::CardFieldHover, theme),
            );
        }
        let g = m::ICON_GLYPH;
        let tinta = resolve(ColorToken::KanbanListFg, theme);
        paint_icon(
            ctx.scene,
            IconId::Add,
            Rect::new(fr.x + m::CARD_PAD_X, fr.y + (fr.h - g) * 0.5, g, g),
            tinta,
            StrokeToken::Default.px(),
        );
        crate::paint::paint_text(
            ctx.text_system,
            ctx.scene,
            f.text,
            fr.x + m::CARD_PAD_X + g + m::SEG_ICON_GAP,
            fr.y + (fr.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            fr.w,
            tinta,
        );
    }
    let rolagem = crate::widget::scroll_area::close_parts(
        area,
        ctx.scene,
        hit_index,
        store,
        rolagem::conteudo(placed.content_h + notas_h + pe, natural <= tecto),
        theme,
    );

    // ── O que voa: por cima de tudo, inclinado, sem registar cliques ──
    if let Some(f) = voo
        && let Some(card) = cards.iter().find(|c| c.section == f.key)
    {
        let (gx, gy) = f.ghost();
        let local = Rect::new(0.0, 0.0, f.size.0, f.size.1);
        let mut s = VectorScene::new();
        let mut nada = HitIndex::default();
        flight_shadow(&mut s, theme, local);
        let h = paint_card(
            &mut s,
            ctx.text_system,
            store,
            &mut nada,
            panel,
            theme,
            local,
            card,
            false,
        );
        alturas.push(Item { key: f.key, h });
        let pivot = ph2d_card_stack::flight_pivot(local.w, local.h);
        let t = Affine::translate((f64::from(gx), f64::from(gy)))
            * Affine::translate(pivot)
            * Affine::rotate(tilt.to_radians())
            * Affine::translate((-pivot.0, -pivot.1));
        ctx.scene.append_transformed(&s, t);
    }
    for (r, card) in voltam {
        flight_shadow(ctx.scene, theme, r);
        let mut nada = HitIndex::default();
        let h = paint_card(
            ctx.scene,
            ctx.text_system,
            ctx.host.store_mut(),
            &mut nada,
            panel,
            theme,
            r,
            card,
            false,
        );
        alturas.push(Item {
            key: card.section,
            h,
        });
    }
    solto::pinta_os_soltos(ctx, panel, theme, canvas, cards, pinned);
    rolagem.publish(ctx.host.store_mut());
    let list = ctx.host.store_mut().card_list_mut(panel);
    list.items = alturas;
    list.loose_notes_h = notas_h;
    lista
}

/// ⭐ **As dicas dos cartões**: todo ícone sem texto pintado num cartão regista-se aqui (o censo
/// `every_icon_in_a_card_has_a_tooltip` exige-lhe uma dica), e o campo numérico ganha a dica do
/// arrasto com o rótulo dele (maquete B: «Length — drag the icon · Shift ×10 · Alt fine»).
#[path = "cards_dicas.rs"]
pub mod dicas;

/// O id da barra de rolagem da lista de `panel`.
#[must_use]
pub fn scroll_bar_of(panel: NodeId) -> NodeId {
    const SAL_DA_BARRA: u64 = 0x5B1D_7E09_C3A4_6F28;
    NodeId(panel.0 ^ SAL_DA_BARRA)
}

/// ⭐ **Um cartão** em `r` (a altura de `r` é a do quadro anterior). Devolve a altura medida.
#[allow(clippy::too_many_arguments)]
fn paint_card(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    panel: NodeId,
    painel: Theme,
    r: Rect,
    card: &CardDef<'_>,
    rest_shadow: bool,
) -> f32 {
    paint_card_with(
        scene,
        text_system,
        store,
        hit_index,
        panel,
        painel,
        r,
        card,
        rest_shadow,
        Extra::default(),
    )
}

/// O que o cartão SOLTO pinta a mais (C5/T7): ações no fim do cabeçalho (alfinete, «Return to
/// panel»), o subtítulo «de quem» e o esmaecido do «Not available».
#[derive(Default)]
struct Extra<'a> {
    acoes: Vec<HeadAction>,
    sub: Option<&'a str>,
    /// T42: o sujeito preso não é a seleção — o nome em destaque, com esta opacidade (o piscar).
    destaque: Option<f32>,
    /// T57: o corpo do solto rola por dentro, até ao fundo do canvas.
    rola: Option<solto::rola::Rola>,
}

/// [`paint_card`] com o [`Extra`] do solto.
#[allow(clippy::too_many_arguments)]
fn paint_card_with(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    panel: NodeId,
    painel: Theme,
    r: Rect,
    card: &CardDef<'_>,
    rest_shadow: bool,
    extra: Extra<'_>,
) -> f32 {
    let face = extra.rola.map_or(card.section, |k| k.face);
    let ink = CardInk::of_card(store, face, painel);
    if rest_shadow {
        shadow(
            scene,
            r,
            m::CARD_RADIUS,
            ink.color(ColorToken::KanbanShadowNear),
            ph2d_card_stack::shadow::SHADOW_NEAR,
        );
        shadow(
            scene,
            r,
            m::CARD_RADIUS,
            ink.color(ColorToken::KanbanShadowRim),
            ph2d_card_stack::shadow::SHADOW_RIM,
        );
    }
    fill_rounded_rect(scene, r, m::CARD_RADIUS, ink.color(ColorToken::KanbanCard));

    // ── O cabeçalho = a pega (e o menu do botão direito da secção) ──
    let sem_titulo = card.title.is_empty();
    let head_h = if sem_titulo {
        m::CARD_PAD_TOP
    } else {
        m::CARD_PAD_TOP + m::CARD_HEAD_H
    };
    let head = Rect::new(r.x, r.y, r.w, head_h);
    let ix = r.x + m::CARD_PAD_X;
    let iw = r.w - m::CARD_PAD_X * 2.0;
    let hy = r.y + m::CARD_PAD_TOP;
    let dobrado = store.is_collapsed(card.section);
    let ausente = card.add.is_some();
    let mut acoes: Vec<HeadAction> = card.actions.clone();
    if let Some(add) = card.add {
        acoes.push(HeadAction {
            id: add,
            icon: IconId::Add,
            on: false,
        });
    }
    acoes.extend(extra.acoes);
    let mut ax = ix + iw;
    // ⚠️ Os ícones registam-se DEPOIS do cabeçalho (o `hit` anda do fim para o princípio: o
    // cabeçalho registado a seguir cobria-os — `HitIndex::register`, «Pitfall»).
    let mut icones: Vec<(NodeId, Rect)> = Vec::with_capacity(acoes.len());
    for a in acoes.iter().rev() {
        ax -= m::HEAD_ICON_BOX;
        let b = Rect::new(
            ax,
            hy + (m::CARD_HEAD_H - m::HEAD_ICON_BOX) * 0.5,
            m::HEAD_ICON_BOX,
            m::HEAD_ICON_BOX,
        );
        icones.push((a.id, b));
        dicas::icone(a.id);
        if a.on || store.hover_live(a.id) > 0.0 {
            fill_rounded_rect(
                scene,
                b,
                m::HEAD_ICON_RADIUS,
                ink.color(ColorToken::CardFieldHover),
            );
        }
        let g = m::HEAD_ICON_GLYPH;
        let gr = Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g);
        let tinta = if a.on {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        };
        paint_icon(
            scene,
            a.icon,
            gr,
            ink.color(tinta),
            StrokeToken::Default.px(),
        );
    }
    let grip = Rect::new(head.x, head.y, (ax - head.x).max(0.0), head.h);
    hit_index.register_section(card.section, head, grip_of(card.section), grip);
    for (id, b) in icones {
        hit_index.register(id, b);
    }
    // Dobrado: «›» antes do título, e o título um tom abaixo (maquete C).
    let seta = dobrado && !sem_titulo;
    let tx = if seta {
        let g = m::CARD_CHEVRON;
        paint_icon(
            scene,
            IconId::ChevronRight,
            Rect::new(ix, hy + (m::CARD_HEAD_H - g) * 0.5, g, g),
            ink.color(ColorToken::Text3),
            StrokeToken::Default.px(),
        );
        ix + g + m::CARD_CHEVRON_GAP
    } else {
        ix
    };
    let mut titulo_w = (ax - tx).max(0.0);
    if let Some(sub) = extra.sub.filter(|s| !s.is_empty() && !sem_titulo) {
        let sw = text_system
            .prefix_width(sub, m::LABEL_PX)
            .min(titulo_w * 0.5);
        crate::paint::paint_text(
            text_system,
            scene,
            sub,
            ax - sw - m::FIELD_GAP,
            hy + (m::CARD_HEAD_H - m::LABEL_PX) * 0.5,
            m::LABEL_PX,
            sw,
            extra.destaque.map_or(ink.color(ColorToken::Text3), |a| {
                ink.color(ColorToken::Accent).multiply_alpha(a)
            }),
        );
        titulo_w = (titulo_w - sw - m::FIELD_GAP * 2.0).max(0.0);
    }
    let tinta = if ausente {
        ColorToken::Text3
    } else if seta {
        ColorToken::Text2
    } else {
        ColorToken::Text1
    };
    crate::paint::paint_text_title_elided_tracked(
        text_system,
        scene,
        card.title,
        [tx, hy + (m::CARD_HEAD_H - m::CARD_TITLE_PX) * 0.5],
        m::CARD_TITLE_PX,
        titulo_w,
        ink.color(tinta),
        0.0,
    );
    if dobrado && let Some(s) = card.summary.as_deref() {
        let sw = text_system.prefix_width(s, m::LABEL_PX);
        crate::paint::paint_text(
            text_system,
            scene,
            s,
            ax - sw - m::FIELD_GAP,
            hy + (m::CARD_HEAD_H - m::LABEL_PX) * 0.5,
            m::LABEL_PX,
            sw,
            ink.color(ColorToken::Text3),
        );
    }
    let mut fundo = if sem_titulo { hy } else { hy + m::CARD_HEAD_H };

    // ── O corpo (os campos e as notas, D6) ──
    if !dobrado && !ausente && card.body.is_some() {
        let topo = fundo + if sem_titulo { 0.0 } else { m::CARD_HEAD_BOTTOM };
        let (s, t) = (&mut *scene, &mut *text_system);
        fundo = solto::rola::corpo(
            s, t, store, hit_index, panel, painel, ink, r, card, topo, extra.rola,
        );
        fundo += m::CARD_PAD_BOTTOM;
    } else {
        fundo += m::CARD_PAD_TOP;
    }
    let h = fundo - r.y;
    if let Some(cor) = store.section_outline_color(face) {
        let o = m::CARD_OUTLINE_GAP;
        let caixa = Rect::new(r.x - o, r.y - o, r.w + o * 2.0, h + o * 2.0);
        crate::widget::showcase::notes_chrome::contorno_do_cartao(
            scene,
            cor,
            caixa,
            m::CARD_RADIUS + o,
        );
    }
    h
}

fn shadow(
    scene: &mut VectorScene,
    r: Rect,
    radius: f32,
    color: ph2d_vector::Color,
    (dy, blur): (f64, f64),
) {
    let rr = ph2d_vector::Rect::new(
        f64::from(r.x),
        f64::from(r.y) + dy,
        f64::from(r.x + r.w),
        f64::from(r.y + r.h) + dy,
    );
    scene.blurred_rounded_rect(Affine::IDENTITY, rr, color, f64::from(radius), blur / 2.0);
}

fn flight_shadow(scene: &mut VectorScene, theme: Theme, r: Rect) {
    use ph2d_card_stack::shadow as s;
    let far = if theme.is_dark() {
        s::SHADOW_FLIGHT_DARK
    } else {
        s::SHADOW_FLIGHT_LIGHT
    };
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        resolve(ColorToken::KanbanShadowFlight, theme),
        far,
    );
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        resolve(ColorToken::KanbanFlightNear, theme),
        s::SHADOW_FLIGHT_NEAR,
    );
}

/// ⭐ **A lista suspensa de um cartão** — qual está aberta (um clique na caixa abre/fecha; qualquer
/// outro clique fecha) e as opções do quadro, pintadas por CIMA de tudo no fim da lista. Cada opção
/// leva o id do painel dono: o clique nela é o `Click` que o `apply_event` dele já trata.
pub mod dropdown {
    use super::*;
    use std::cell::RefCell;

    type Opcoes = (Rect, CardInk, Vec<(NodeId, String, bool)>);

    thread_local! {
        static ABERTA: RefCell<Option<NodeId>> = const { RefCell::new(None) };
        static PENDENTE: RefCell<Option<Opcoes>> = const { RefCell::new(None) };
        static CAIXAS: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    }

    #[must_use]
    pub fn open() -> Option<NodeId> {
        ABERTA.with(|a| *a.borrow())
    }

    pub(super) fn stash(r: Rect, ink: CardInk, options: Vec<(NodeId, String, bool)>) {
        PENDENTE.with(|p| *p.borrow_mut() = Some((r, ink, options)));
    }

    /// Declara `id` como caixa de lista suspensa (o painel, ao montar os cartões).
    pub fn declare(id: NodeId) {
        CAIXAS.with(|c| {
            let mut c = c.borrow_mut();
            if !c.contains(&id) {
                c.push(id);
            }
        });
    }

    /// O clique: na caixa abre/fecha (consome); noutro sítio fecha e deixa o clique seguir.
    pub fn on_click(id: NodeId) -> bool {
        if CAIXAS.with(|c| c.borrow().contains(&id)) {
            ABERTA.with(|a| {
                let mut a = a.borrow_mut();
                *a = if *a == Some(id) { None } else { Some(id) };
            });
            return true;
        }
        ABERTA.with(|a| *a.borrow_mut() = None);
        false
    }

    /// Pinta as opções da lista aberta por baixo da caixa, por cima de tudo.
    pub(super) fn paint_pending(
        scene: &mut VectorScene,
        text_system: &mut TextSystem,
        hit_index: &mut HitIndex,
        store: &WidgetStore,
    ) {
        let Some((r, ink, opts)) = PENDENTE.with(|p| p.borrow_mut().take()) else {
            return;
        };
        let h = m::FIELD_H * opts.len() as f32 + m::SEG_PAD * 2.0;
        let caixa = Rect::new(r.x, r.y + r.h + m::SEG_PAD, r.w, h);
        shadow(
            scene,
            caixa,
            m::FIELD_RADIUS,
            ink.color(ColorToken::KanbanListShadow),
            m::LIST_SHADOW_FAR,
        );
        fill_rounded_rect(
            scene,
            caixa,
            m::FIELD_RADIUS,
            ink.color(ColorToken::KanbanCardRaised),
        );
        for (i, (id, text, on)) in opts.iter().enumerate() {
            let lr = Rect::new(
                caixa.x + m::SEG_PAD,
                caixa.y + m::SEG_PAD + m::FIELD_H * i as f32,
                caixa.w - m::SEG_PAD * 2.0,
                m::FIELD_H,
            );
            hit_index.register(*id, lr);
            if *on || store.hover_live(*id) > 0.0 {
                fill_rounded_rect(
                    scene,
                    lr,
                    m::SEG_RADIUS,
                    ink.color(ColorToken::CardFieldHover),
                );
            }
            crate::paint::paint_text_elided(
                text_system,
                scene,
                text,
                lr.x + m::FIELD_PAD_R,
                lr.y + (lr.h - m::VALUE_PX) * 0.5,
                m::VALUE_PX,
                lr.w - m::FIELD_PAD_R * 2.0,
                ink.color(ColorToken::Text1),
            );
        }
    }
}
