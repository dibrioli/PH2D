//! ⭐ **As linhas de dentro de um cartão** — o layout Figma de [`CardCtx`]: rótulos acima, os
//! widgets lado a lado em colunas, e a pintura de cada widget (escolha, texto, amostra, lista
//! suspensa, botão).

use super::*;

impl<'s> CardCtx<'s> {
    /// As linhas de um cartão FORA da lista: o painel que abre ao lado da coluna (F43, a galeria).
    #[must_use]
    pub fn flutuante(
        scene: &'s mut VectorScene,
        text_system: &'s mut TextSystem,
        store: &'s WidgetStore,
        hit_index: &'s mut HitIndex,
        ink: CardInk,
        x: f32,
        w: f32,
    ) -> Self {
        Self {
            scene,
            text_system,
            store,
            hit_index,
            ink,
            x,
            w,
            first: true,
        }
    }

    /// A linha tal como vem, sem descer nada (`row` e `compact` em `cards_compacta.rs`).
    pub(super) fn row_inteira(&mut self, cells: &[Cell<'_>], y: f32) -> f32 {
        let rotulos = cells.iter().any(|c| c.label.is_some());
        let marcas = !rotulos && cells.iter().any(|c| c.key.is_some());
        let mut y = if rotulos {
            y + m::LABEL_TOP
        } else if self.first {
            y
        } else {
            y + m::FIELD_ROW_GAP
        };
        self.first = false;
        let topo = y;
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
        } else if marcas {
            y += rotulos::so_marcas(self, cells, &cols, y);
        }
        for (i, (c, (x, w))) in cells.iter().zip(&cols).enumerate() {
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
                    super::letras::glifo_na_fila(cells, i, glyph),
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
                Widget::ButtonOff { text } => self.button_off(r, text),
                Widget::Mark { icon, tint } => {
                    let g = m::ICON_GLYPH;
                    paint_icon(
                        self.scene,
                        icon,
                        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
                        self.ink.color(tint),
                        StrokeToken::Default.px(),
                    );
                }
                Widget::Readout { text } => {
                    crate::paint::paint_text_elided(
                        self.text_system,
                        self.scene,
                        text,
                        r.x,
                        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
                        m::VALUE_PX,
                        r.w,
                        self.ink.color(ColorToken::Text1),
                    );
                }
                Widget::Swatch {
                    id,
                    rgba,
                    text,
                    edit,
                    detach,
                } => self.swatch(r, id, rgba, text, edit, detach),
                Widget::Dropdown { id, text, options } => self.dropdown(r, id, text, options),
                Widget::Check {
                    id,
                    label,
                    value,
                    off,
                } => corpo::check_em(
                    self.scene,
                    self.text_system,
                    self.store,
                    self.hit_index,
                    self.ink,
                    r,
                    (id, label),
                    (value, off),
                    cells.len() > 1,
                ),
            }
        }
        self.anota_linha(cells, topo, y + m::FIELD_H);
        y + m::FIELD_H
    }

    /// ⭐ **A linha de tinta** `[cor] [opacidade] 👁 −`. Numa linha só quando cada caixa e cada rótulo
    /// cabem inteiros; senão a cor vai para a sua própria linha e a opacidade e os botões para a
    /// seguinte (o Figma na coluna estreita): nada vira «...» nem quebra o rótulo.
    pub fn paint_line(&mut self, cor: Cell<'_>, op: Cell<'_>, botoes: Cell<'_>, y: f32) -> f32 {
        let toda = [cor, op, botoes];
        if self.tudo_cabe(&toda) {
            return self.row(&toda, y);
        }
        let y = self.row(&[cor], y);
        self.row(&[op, botoes], y)
    }

    pub(super) fn tudo_cabe(&mut self, cells: &[Cell<'_>]) -> bool {
        let cols = self.columns(cells);
        cells
            .iter()
            .zip(&cols)
            .all(|(c, (_, w))| *w >= self.pede(c))
    }

    /// A largura em que `c` mostra o conteúdo e o rótulo sem cortar.
    pub(super) fn pede(&mut self, c: &Cell<'_>) -> f32 {
        let escolha = match c.widget {
            Widget::Choice(segs) => self.escolha_pede(segs),
            _ => 0.0,
        };
        let ts = &mut *self.text_system;
        let corpo = match c.widget {
            Widget::Choice(_) => escolha,
            Widget::Swatch { text, detach, .. } => {
                let mut mede = |t: &str| ts.prefix_width(t, m::VALUE_PX);
                colunas::caixa_da_cor(text, detach.is_some(), &mut mede)
            }
            Widget::Field { .. } | Widget::FieldFace { .. } => m::FIELD_VALUE_MIN_W,
            Widget::Button { text, .. } | Widget::ButtonOff { text } => {
                ts.prefix_width(text, m::VALUE_PX).ceil() + m::BUTTON_TEXT_PAD * 2.0
            }
            Widget::Readout { text } => ts.prefix_width(text, m::VALUE_PX).ceil(),
            Widget::Dropdown { text, .. } => {
                ts.prefix_width(text, m::VALUE_PX).ceil() + m::ICON_CELL + m::FIELD_PAD_R
            }
            Widget::Check { label, .. } => {
                m::CHECK_BOX
                    + m::CHECK_GAP
                    + ts.prefix_width(label.texto, m::VALUE_PX).ceil()
                    + label.reserva_da_marca()
            }
            _ => 0.0,
        };
        let rotulo = c.label.map_or(0.0, |l| {
            ts.prefix_width(l, m::LABEL_PX).ceil() + rotulos::reserva(c)
        });
        corpo.max(rotulo)
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
        let fim = f(&mut r, y);
        censo::linha((self.x, self.w), y, fim, Vec::new);
        fim
    }

    /// Uma linha de um widget só a toda a largura, com rótulo opcional.
    pub fn full(&mut self, label: Option<&str>, widget: Widget<'_>, y: f32) -> f32 {
        let mut c = Cell::new(widget);
        c.label = label;
        self.row(&[c], y)
    }

    pub(super) fn columns(&mut self, cells: &[Cell<'_>]) -> Vec<(f32, f32)> {
        let ts = &mut *self.text_system;
        let mut mede = |t: &str| ts.prefix_width(t, m::VALUE_PX);
        colunas::colunas(self.x, self.w, cells, &mut mede)
    }

    pub(super) fn seg_need(&mut self, s: &Seg<'_>, com_icone: bool) -> (f32, bool) {
        let tw = if s.text.is_empty() {
            0.0
        } else {
            self.text_system.prefix_width(s.text, m::SEG_TEXT_PX)
        };
        let iw = glifo_do_segmento(s, com_icone);
        let gap = if iw > 0.0 && tw > 0.0 {
            m::SEG_ICON_GAP
        } else {
            0.0
        };
        (iw + gap + tw + m::SEG_INSET * 2.0, tw == 0.0)
    }

    fn choice(&mut self, r: Rect, segs: &[Seg<'_>]) {
        // Uma escolha é UM valor, não N comandos: o censo das entradas lê-a como o `paint_choice_row`.
        crate::widget::composto::grupo(segs.iter().map(|s| s.id));
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
        // Onde ícone e palavra não cabem juntos, a palavra fica: ela é o nome do segmento.
        let avail = inner.w - m::SEG_PAD * (n - 1.0);
        let inteiros: f32 = segs.iter().map(|s| self.seg_need(s, true).0).sum();
        let com_icone = inteiros <= avail;
        let needs: Vec<(f32, bool)> = segs.iter().map(|s| self.seg_need(s, com_icone)).collect();
        let ws = m::seg_widths(&needs, avail);
        let mut sx = inner.x;
        for (s, w) in segs.iter().zip(ws) {
            let sr = Rect::new(sx, inner.y, w, inner.h);
            sx += w + m::SEG_PAD;
            self.hit_index.register(s.id, sr);
            if s.text.is_empty() && s.icon.is_some() {
                dicas::icone(s.id);
            }
            if !s.text.is_empty() && censo::ligado() {
                censo::texto(s.text, self.seg_need(s, false).0, w);
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
            let iw = glifo_do_segmento(s, com_icone);
            let gap = if iw > 0.0 && tw > 0.0 {
                m::SEG_ICON_GAP
            } else {
                0.0
            };
            let total = (iw + gap + tw).min(sr.w);
            let mut x = sr.x + (sr.w - total) * 0.5;
            if let Some(icon) = s.icon.filter(|_| iw > 0.0) {
                let ir = Rect::new(x, sr.y + (sr.h - iw) * 0.5, iw, iw);
                // A Position do traço tem duas tintas (F44c); os outros ícones, a da peça.
                crate::paint::paint_icon_toned(
                    self.scene,
                    icon,
                    ir,
                    tinta,
                    self.ink.theme,
                    StrokeToken::Default.px(),
                );
                x += iw + gap;
            }
            if tw > 0.0 {
                crate::paint::paint_text_elided(
                    self.text_system,
                    self.scene,
                    s.text,
                    x,
                    sr.y + crate::paint::folga_do_texto(sr.h, m::SEG_TEXT_PX),
                    m::SEG_TEXT_PX,
                    (sr.x + sr.w - x).max(0.0),
                    tinta,
                );
            }
        }
    }

    pub(super) fn text(&mut self, r: Rect, id: NodeId, placeholder: &str) {
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
                r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
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
    pub fn bare(&mut self, r: Rect, segs: &[Seg<'_>]) {
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
        // Sem caixa de texto, o código não faz nada: a caixa inteira abre o seletor.
        let alvo = if edit.is_some() {
            Rect::new(r.x, r.y, m::ICON_CELL, r.h)
        } else {
            r
        };
        self.hit_index.register(id, alvo);
        if let Some(c) = rgba {
            crate::widget::card_field::paint_amostra(self.scene, self.ink, amostra, c);
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
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            (r.w - m::ICON_CELL - m::FIELD_PAD_R).max(0.0),
            self.ink.color(ColorToken::Text1),
        );
    }

    fn dropdown(&mut self, r: Rect, id: NodeId, text: &str, options: &[Seg<'_>]) {
        self.hit_index.register(id, r);
        let aceso = self.store.hover_live(id) > 0.0;
        crate::widget::paint_card_chip(self.scene, self.text_system, self.ink, r, text, aceso);
        if dropdown::open() == Some(id) && !options.is_empty() {
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

    fn button_off(&mut self, r: Rect, text: &str) {
        self.anota_botao(text, r.w);
        fill_rounded_rect(
            self.scene,
            r,
            m::FIELD_RADIUS,
            self.ink.color(ColorToken::CardField),
        );
        let tw = self.text_system.prefix_width(text, m::VALUE_PX).min(r.w);
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            text,
            r.x + (r.w - tw) * 0.5,
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            r.w,
            self.ink.color(ColorToken::CardLabel),
        );
    }

    /// O texto de um botão no censo da lei 7.
    pub(super) fn anota_botao(&mut self, text: &str, w: f32) {
        if censo::ligado() {
            let tw = self.text_system.prefix_width(text, m::VALUE_PX);
            censo::texto(text, tw + m::BUTTON_TEXT_PAD * 2.0, w);
        }
    }

    fn button(&mut self, r: Rect, id: NodeId, text: &str, primary: bool) {
        self.anota_botao(text, r.w);
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
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            r.w,
            self.ink.color(tinta),
        );
    }
}

#[cfg(test)]
#[path = "cards_linhas_tests.rs"]
mod tests;

/// O lado do ícone de um segmento: sozinho é maior; ao lado de um texto fica no tamanho do texto, e
/// larga-se (`com_icone` falso) onde ícone e palavra não cabem juntos — a palavra é o nome.
fn glifo_do_segmento(s: &Seg<'_>, com_icone: bool) -> f32 {
    match s.icon {
        None => 0.0,
        Some(_) if s.text.is_empty() => m::SEG_ICON_GLYPH,
        Some(_) if com_icone => m::ICON_GLYPH,
        Some(_) => 0.0,
    }
}
