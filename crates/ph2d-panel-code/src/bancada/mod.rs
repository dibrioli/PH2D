//! ⭐ **A Bancada** (plano 13 Onda B, itens 6–10; dono 08/11: D1 «Bancada», D2 «cor por objecto»;
//! 09/11: «espiar a cena» e «mapa», os dois ligados por omissão, cada um com o seu botão no menu
//! View). O «In this script» por baixo da lista ([`outline`]), o balão «em quem» das marcas da margem
//! ([`popover`]), a grelha «Who has what» com o mapa ao lado ([`grid`], [`map`]) e a frase fixa do
//! pé ([`phrase`]). O idioma é o da aba Collision Layers da GEF02: a cor própria de cada objecto
//! (`ColorToken::layer`), a faixa acesa sob o rato, os nomes nas duas bordas e a frase que diz o que
//! o próximo clique faz. Aqui pinta-se e responde-se «onde caiu»; o que um clique PEDE sai em
//! [`Intent`] para a família (`ph2d-app-code`), que o vira na edição do Inspector.

mod grid;
mod map;
mod outline;
mod phrase;
mod popover;

use crate::field::{FieldKey, field_key};
use ph2d_code_buffer::Buffer;
use ph2d_code_viewmodel::CellKind;
pub use ph2d_code_viewmodel::{Aim, Intent, Who};
use ph2d_editor_core::doc_surface::KeyOutcome;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

pub use phrase::say;

/// Quanto o rato pára num nome antes de a cena aparecer: 1 s (dono 09/11 — os 0,5 s do balão do
/// código disparavam sem querer, e a cena tira o editor da frente; o balão continua a 0,5 s).
pub const PEEK_PAUSE_NS: u128 = 1_000_000_000;

/// Abaixo disto (o título e três linhas) o «In this script» não aparece: a lista fica com a coluna.
const OUTLINE_MIN_ROWS: usize = 3;

/// O raio do ponto da cor de um objecto ao lado do nome (o quadradinho da camada da GEF02,
/// `lado_do_quadradinho`, feito redondo).
pub(crate) fn chip_r() -> f32 {
    (crate::chrome::font() - Spacing::Xs.px()) * 0.5
}

/// Quanto o balão espera pelo rato que saiu da marca a caminho dele (ou dele para fora) antes de
/// fechar: o `editor.hover.hidingDelay` do VS Code 1.142 (300 ms) — o mesmo do balão do código
/// (`ph2d-app-code` `hint.rs` `HOVER_HIDE_NS`; smoke do dono 09/11: «ao levar o mouse pro cartão
/// o cartão some»).
pub const POP_HIDE_NS: u128 = 300_000_000;

/// O balão «em quem» aberto: a linha da marca, se um clique o prendeu, a marca que o abriu (ele
/// encosta-se por baixo dela) e desde quando o rato está fora dos dois.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pop {
    line: usize,
    pinned: bool,
    anchor: Rect,
    left: Option<u128>,
}

/// A célula a ser escrita: o objecto, a propriedade e o campo.
struct Editing {
    holder: usize,
    column: usize,
    buf: Buffer,
}

/// O estado de vista da Bancada da aba activa.
pub struct BancadaView {
    /// O espiar da cena (menu View ▸ Peek at Scene).
    pub peek_on: bool,
    /// O mapa ao lado da grelha (menu View ▸ Scene Map).
    pub map_on: bool,
    /// A grelha dobrada (só o título).
    pub folded: bool,
    aim: Aim,
    aim_rect: Rect,
    aim_since: u128,
    peeking: Option<usize>,
    pop: Option<Pop>,
    editing: Option<Editing>,
    intents: Vec<Intent>,
    focus: bool,
    pointer: (f32, f32),
    /// As marcas da margem pintadas pelo código no último quadro (a linha e a caixa).
    marks: Vec<(usize, Rect)>,
    /// Os alvos de cada peça no último quadro, por ordem de pintura.
    outline_hits: Vec<(Aim, Rect)>,
    grid_hits: Vec<(Aim, Rect)>,
    map_hits: Vec<(Aim, Rect)>,
    pop_hits: Vec<(Aim, Rect)>,
    console_hit: Option<(Aim, Rect)>,
    fold_rect: Rect,
    pop_rect: Option<Rect>,
    /// A tabela do balão (o que ela acende) e as larguras dele, por marca (só crescem).
    pop_table: ph2d_ui_table::TableState,
    pop_steady: Option<(usize, ph2d_ui_table::StableWidths)>,
    pop_painted: Option<(ph2d_ui_table::Layout, Vec<ph2d_ui_table::Row>)>,
    outline: outline::Scroll,
    /// A tabela da grelha: a rolagem, as larguras arrastadas e o que ela acende.
    table: ph2d_ui_table::TableState,
    steady: ph2d_ui_table::StableWidths,
    painted: Option<grid::Painted>,
    /// O separador ↔ entre a grelha e o mapa.
    split: crate::splitter::Splitter,
    /// O alvo da grelha premido (a moldura do clique), até largar.
    pressed: Option<Aim>,
    said: String,
}

impl Default for BancadaView {
    fn default() -> Self {
        Self {
            peek_on: true,
            map_on: true,
            folded: false,
            aim: Aim::Nothing,
            aim_rect: crate::chrome::NO_RECT,
            aim_since: 0,
            peeking: None,
            pop: None,
            editing: None,
            intents: Vec::new(),
            focus: false,
            pointer: (0.0, 0.0),
            marks: Vec::new(),
            outline_hits: Vec::new(),
            grid_hits: Vec::new(),
            map_hits: Vec::new(),
            pop_hits: Vec::new(),
            console_hit: None,
            fold_rect: crate::chrome::NO_RECT,
            pop_rect: None,
            pop_table: ph2d_ui_table::TableState::default(),
            pop_steady: None,
            pop_painted: None,
            outline: outline::Scroll::default(),
            table: ph2d_ui_table::TableState::default(),
            steady: ph2d_ui_table::StableWidths::default(),
            painted: None,
            split: crate::splitter::Splitter::default(),
            pressed: None,
            said: String::new(),
        }
    }
}

/// A coluna da lista de scripts partida: a lista em cima (a altura que as linhas pedem, até metade)
/// e o «In this script» por baixo — nenhum se a coluna está recolhida ou não sobra para três linhas.
#[must_use]
pub fn left_split(col: Rect, list_rows: usize, open: bool) -> (Rect, Option<Rect>) {
    let g = crate::cards::margin();
    let want = crate::ScriptListView::height_for(list_rows);
    let list_h = want.min(col.h * 0.5);
    let rest = col.h - list_h - g;
    if !open || rest < ROW_H_PX * (OUTLINE_MIN_ROWS + 1) as f32 {
        return (col, None);
    }
    (
        Rect::new(col.x, col.y, col.w, list_h),
        Some(Rect::new(col.x, col.y + list_h + g, col.w, rest)),
    )
}

/// A altura da faixa da frase do pé.
#[must_use]
pub fn phrase_height() -> f32 {
    ph2d_tokens::row_pitch_px()
}

/// A coluna do documento partida: o resto em cima e a faixa da frase em baixo.
#[must_use]
pub fn phrase_split(doc: Rect) -> (Rect, Rect) {
    let h = (phrase_height() + crate::cards::margin()).min(doc.h);
    (
        Rect::new(doc.x, doc.y, doc.w, doc.h - h),
        Rect::new(
            doc.x,
            doc.y + doc.h - phrase_height().min(doc.h),
            doc.w,
            phrase_height().min(doc.h),
        ),
    )
}

/// A coluna da propriedade cujo nome está sob o cursor do texto (`self.speed`, `"speed"`).
#[must_use]
pub fn caret_column(buf: &Buffer, who: &Who) -> Option<usize> {
    let t = buf.text();
    let head = buf.selection().primary().head;
    let line = t.line_of_byte(head);
    let start = t.byte_of_line(line);
    let s = t.slice(start..t.byte_of_line(line + 1).max(start));
    let at = (head - start).min(s.len());
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let a = s[..at].rfind(|c: char| !word(c)).map_or(0, |i| i + 1);
    let b = s[at..].find(|c: char| !word(c)).map_or(s.len(), |i| at + i);
    let w = &s[a..b];
    who.columns.iter().position(|c| c.name == w)
}

impl BancadaView {
    /// O que está sob o rato (a frase di-lo; os testes lêem-no).
    #[must_use]
    pub fn aim(&self) -> Aim {
        self.aim
    }

    /// O objecto que a cena mostra agora no lugar da aba (espiar).
    #[must_use]
    pub fn peeking(&self) -> Option<usize> {
        self.peeking
    }

    /// A frase pintada no último quadro.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// A Bancada tem o teclado e o desfazer: o último clique foi nela (o Ctrl+Z desfaz o projecto).
    #[must_use]
    pub fn has_focus(&self) -> bool {
        self.focus || self.editing.is_some()
    }

    /// O balão aberto: a linha da marca.
    #[must_use]
    pub fn popover_line(&self) -> Option<usize> {
        self.pop.map(|p| p.line)
    }

    /// A caixa de um alvo do último quadro (os testes clicam nela).
    #[must_use]
    pub fn target(&self, aim: Aim) -> Option<Rect> {
        self.all_hits().find(|(a, _)| *a == aim).map(|(_, r)| r)
    }

    /// O botão da dobra da grelha no último quadro.
    #[must_use]
    pub fn fold_rect(&self) -> Rect {
        self.fold_rect
    }

    /// Os pedidos dos cliques desde o último quadro.
    pub fn take_intents(&mut self) -> Vec<Intent> {
        std::mem::take(&mut self.intents)
    }

    /// As marcas que o código pintou neste quadro ([`crate::CodeView::bancada_marks`]).
    pub fn set_marks(&mut self, marks: &[(usize, Rect)]) {
        self.marks.clear();
        self.marks.extend_from_slice(marks);
    }

    /// O nome de um objecto no Console sob o rato (a família sabe de quem é cada linha).
    pub fn set_console_aim(&mut self, hit: Option<(usize, Rect)>) {
        self.console_hit = hit.map(|(h, r)| (Aim::Holder(h), r));
    }

    fn all_hits(&self) -> impl Iterator<Item = (Aim, Rect)> + '_ {
        let marks = self.marks.iter().map(|(l, r)| (Aim::Mark(*l), *r));
        let pop: Vec<(Aim, Rect)> = if self.pop.is_some() {
            self.pop_hits.clone()
        } else {
            Vec::new()
        };
        pop.into_iter()
            .chain(self.grid_hits.iter().rev().copied())
            .chain(self.map_hits.iter().rev().copied())
            .chain(self.outline_hits.iter().copied())
            .chain(self.console_hit)
            .chain(marks)
    }

    fn hit(&self, x: f32, y: f32) -> Option<(Aim, Rect)> {
        self.all_hits().find(|(_, r)| r.contains(x, y))
    }

    /// O relógio do quadro: o rato parado num nome tempo bastante faz a cena aparecer.
    pub fn tick(&mut self, now: u128) {
        self.tick_pop(now);
        if self.peeking.is_some() {
            return;
        }
        if let Aim::Holder(h) = self.aim
            && self.peek_on
            && self.editing.is_none()
            && now.saturating_sub(self.aim_since) >= PEEK_PAUSE_NS
        {
            self.peeking = Some(h);
        }
    }

    /// O rato em `(x, y)`. `true` = está sobre uma peça da Bancada.
    pub fn hover(&mut self, x: f32, y: f32, now: u128) -> bool {
        self.pointer = (x, y);
        if self.grid_drag(x, y) {
            return true;
        }
        if self.peeking.is_some() {
            if self.aim_rect.contains(x, y) {
                return true;
            }
            self.peeking = None;
        }
        let (aim, rect) = self
            .hit(x, y)
            .unwrap_or((Aim::Nothing, crate::chrome::NO_RECT));
        if aim != self.aim {
            self.aim = aim;
            self.aim_since = now;
        }
        self.aim_rect = rect;
        match aim {
            Aim::Mark(line) if self.pop.is_none_or(|p| !p.pinned || p.line == line) => {
                let same = self.pop.filter(|p| p.line == line);
                let on_mark = self.marks.iter().any(|(l, r)| *l == line && *r == rect);
                self.pop = Some(Pop {
                    line,
                    pinned: same.is_some_and(|p| p.pinned),
                    anchor: match same {
                        Some(p) if !on_mark => p.anchor,
                        _ => rect,
                    },
                    left: None,
                });
            }
            _ => {
                let inside = self.pop_rect.is_some_and(|r| r.contains(x, y));
                if let Some(p) = self.pop.as_mut().filter(|p| !p.pinned) {
                    if inside {
                        p.left = None;
                    } else if p.left.is_none() {
                        p.left = Some(now);
                    }
                }
                self.tick_pop(now);
            }
        }
        aim != Aim::Nothing || self.fold_rect.contains(x, y)
    }

    /// O balão solto fecha quando o rato está fora dele e da marca há [`POP_HIDE_NS`].
    fn tick_pop(&mut self, now: u128) {
        if self.pop.is_some_and(|p| {
            !p.pinned && p.left.is_some_and(|t| now.saturating_sub(t) >= POP_HIDE_NS)
        }) {
            self.pop = None;
        }
    }

    /// Um carregar do botão principal. `true` = foi na Bancada (tomado).
    pub fn press(&mut self, who: &Who, x: f32, y: f32) -> bool {
        if self.fold_rect.contains(x, y) {
            self.commit();
            self.folded = !self.folded;
            return true;
        }
        if self.grid_press(x, y) {
            self.commit();
            return true;
        }
        let Some((aim, rect)) = self.hit(x, y) else {
            self.blur();
            return false;
        };
        let editing_here = matches!(aim, Aim::Cell(h, c)
            if self.editing.as_ref().is_some_and(|e| e.holder == h && e.column == c));
        if !editing_here {
            self.commit();
        }
        self.focus = true;
        self.pressed = Some(aim);
        match aim {
            Aim::Nothing => {}
            Aim::Entry(line) => self.intents.push(Intent::Jump(line)),
            Aim::Column(c) => {
                if let Some(col) = who.columns.get(c) {
                    self.intents.push(Intent::Jump(col.line));
                }
            }
            Aim::Holder(h) | Aim::Select(h) => self.intents.push(Intent::Select(h)),
            // No corpo do balão o clique só o prende; na marca, prende ou solta.
            Aim::Mark(line)
                if !self
                    .marks
                    .iter()
                    .any(|(l, r)| *l == line && r.contains(x, y)) =>
            {
                if let Some(p) = self.pop.as_mut() {
                    (p.pinned, p.left) = (true, None);
                }
            }
            Aim::Mark(line) => {
                let pinned = self.pop.is_some_and(|p| p.pinned && p.line == line);
                self.pop = (!pinned).then_some(Pop {
                    line,
                    pinned: true,
                    anchor: rect,
                    left: None,
                });
            }
            Aim::Cell(h, c) if !editing_here => self.click_cell(who, h, c),
            Aim::Cell(..) => {}
        }
        true
    }

    fn click_cell(&mut self, who: &Who, h: usize, c: usize) {
        let (Some(col), Some(cell)) = (who.columns.get(c), who.cell(h, c)) else {
            return;
        };
        match col.kind {
            None => {}
            Some(CellKind::Bool) => {
                let next = (cell.text != "true").to_string();
                self.intents.push(Intent::Edit(h, c, next));
            }
            Some(CellKind::Choice) => {
                if let Some(next) = next_option(&col.options, &cell.text) {
                    self.intents.push(Intent::Edit(h, c, next.to_owned()));
                }
            }
            Some(CellKind::Color) => self.intents.push(Intent::Select(h)),
            Some(_) => {
                let mut buf = Buffer::new(&cell.text);
                buf.select_all();
                self.editing = Some(Editing {
                    holder: h,
                    column: c,
                    buf,
                });
            }
        }
    }

    /// Um carregar FORA da Bancada: a célula em escrita guarda-se e o teclado volta ao texto.
    pub fn blur(&mut self) {
        self.commit();
        self.focus = false;
        if self.pop.is_some_and(|p| p.pinned) {
            self.pop = None;
        }
    }

    fn commit(&mut self) {
        if let Some(e) = self.editing.take() {
            self.intents
                .push(Intent::Edit(e.holder, e.column, e.buf.text().to_string()));
        }
    }

    /// Uma tecla: a da célula em escrita, ou o Esc que fecha um balão preso. `None` = não é daqui.
    pub fn key(&mut self, k: FieldKey<'_>) -> Option<KeyOutcome> {
        if let Some(e) = self.editing.as_mut() {
            let r = field_key(&mut e.buf, k);
            if r.escape {
                self.editing = None;
            } else if r.enter || r.tab {
                self.commit();
            }
            return Some(KeyOutcome {
                consumed: true,
                copy: r.copy,
            });
        }
        if matches!(k.key, ph2d_editor_core::doc_surface::Key::Escape)
            && self.pop.is_some_and(|p| p.pinned)
        {
            self.pop = None;
            return Some(KeyOutcome {
                consumed: true,
                copy: None,
            });
        }
        None
    }

    /// A roda sobre o «In this script» ou a grelha. `true` = tomada.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        self.outline.wheel(x, y, dy) || self.grid_wheel(x, y, dy)
    }
}

/// A opção a seguir a `now` (a primeira depois da última, ou se `now` não está na lista).
fn next_option<'a>(options: &'a [String], now: &str) -> Option<&'a str> {
    let i = options.iter().position(|o| o == now).map_or(0, |i| i + 1);
    options.get(i % options.len().max(1)).map(String::as_str)
}
