//! ⭐ **O quadro do depurador na aba** (plano 14 — decisão do dono 13): a faixa de cima (estado com a
//! cor, `line N · fn()`, o objecto com o ponto da cor dele, Continue / Next line) e o corpo numa das
//! três aparências do `View ▸ Card Style`:
//!
//! - **Sheet** (padrão, [`sheet`]): a tabela da casa Name · Value · Type, Locals e self dobráveis, as
//!   tabelas aninhadas com ▸; ao lado a Call stack (clicar vai à linha) e Waiting here;
//! - **Cards** ([`boxes`]): um cartão por grupo, cada valor numa caixa de campo, como o Inspector;
//! - **Strip** ([`boxes`]): uma célula por variável, nome pequeno em cima e valor em baixo.
//!
//! «Mudou desde o passo anterior» é âmbar com o valor antigo riscado nas três.

mod boxes;
mod head;
mod sheet;

use crate::view::CodeView;
use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{Ask, CardStyle, DebugCard, columns, sheet_rows};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::widget::scroll_area::{BarDrag, OwnBar};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::{Column, Hit, Layout, Metrics, Row, TableState, natural_height};
use ph2d_vector::VectorScene;

/// A parte da altura da aba que o quadro toma sozinho: o código manda, o corpo rola (smoke do
/// dono 09/11: a metade tirava ao texto mais do que lhe deixava).
const CARD_SHARE: f32 = 1.0 / 3.0; // LITERAL-PX-OK: fração da aba (um terço), não píxeis
/// O máximo a que o puxador de cima o leva: o código fica sempre com o resto.
const USER_SHARE: f32 = 2.0 / 3.0; // LITERAL-PX-OK: fração da aba (dois terços), não píxeis
/// As linhas que o quadro parado mostra no mínimo, mesmo numa aba baixa.
const MIN_ROWS: f32 = 4.0; // LITERAL-PX-OK: uma contagem de linhas, não píxeis
/// A parte da letra que o ■ do estado e o ● de um objecto ocupam.
pub(super) const MARK_SHARE: f32 = 0.6; // LITERAL-PX-OK: fração da letra, não píxeis

/// O recuo do texto dentro do quadro (dos lados).
pub(super) fn inset() -> f32 {
    Spacing::Sm.px() * 2.0
}

/// O que um clique no quadro pede.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardClick {
    /// Continue / Next line.
    Ask(Ask),
    /// Ir à linha (1-based) de uma chamada da pilha.
    Jump(u32),
    /// Copiar um valor.
    Copy(String),
    /// Dentro do quadro, sem mais efeito (o ▸, o puxador, o fundo).
    Taken,
}

/// O que o quadro pintou no último quadro: o rato lê daqui.
#[derive(Clone, Debug, Default)]
pub struct CardPainted {
    pub card: Option<Rect>,
    /// Os botões (Continue, Next line).
    pub buttons: Vec<(Ask, Rect)>,
    /// A tabela da Sheet: a geometria, as linhas e as colunas.
    pub table: Option<(Layout, Vec<Row>, Vec<Column>)>,
    /// As chamadas da pilha (linha 1-based) e a caixa de cada uma.
    pub stack: Vec<(u32, Rect)>,
    /// O valor (escrito) de cada caixa que copia ao clicar (Cards e Strip).
    pub values: Vec<(String, Rect)>,
    /// O puxador da borda de cima (arrastar muda a altura do quadro).
    pub grip: Option<Rect>,
    /// Os Cards, na ordem Locals · self · Call stack: cada um rola por si.
    pub cards: Vec<CardBox>,
}

/// Um cartão dos Cards: a caixa, a rolagem máxima dele e o indicador (só se transborda).
#[derive(Clone, Copy, Debug, Default)]
pub struct CardBox {
    pub rect: Rect,
    pub max: f32,
    pub bar: Option<Rect>,
    /// A geometria do indicador (agarra-se no polegar ou na trilha).
    pub track: OwnBar,
}

/// ⭐ O quadro de uma aba: a aparência, a tabela (larguras, Type escondida, dobras, rolagem) e o
/// que pintou.
#[derive(Clone, Debug, Default)]
pub struct DebugCardView {
    pub style: CardStyle,
    pub table: TableState,
    pub painted: CardPainted,
    /// A linha âmbar já trazida à vista (`linha da paragem:chave`): o passo seguinte rola até à
    /// nova; rolar à mão depois disso fica.
    pub revealed: Option<String>,
    /// A altura que o artista deu ao quadro pelo puxador (`None` = a natural, até ao tecto).
    pub user_h: Option<f32>,
    /// A rolagem de cada cartão dos Cards (Locals · self · Call stack), em píxeis.
    pub cards_scroll: [f32; 3],
    /// O arrasto do puxador: `(y onde começou, altura de então)`.
    resizing: Option<(f32, f32)>,
    /// O arrasto da barra de um cartão dos Cards: `(cartão, arrasto)`.
    cards_drag: Option<(usize, BarDrag)>,
}

/// Os ids dos dois botões (os comandos da aba: Continue, Next line).
pub type ButtonIds = [NodeId; 2];

impl DebugCardView {
    /// A altura do quadro de `c` na aba `area`.
    #[must_use]
    pub fn height(&self, c: &DebugCard, area: Rect) -> f32 {
        let pad = Spacing::Sm.px();
        let rh = ph2d_tokens::row_pitch_px();
        let head = rh + pad * 2.0 + if c.error.is_some() { rh } else { 0.0 };
        if !c.stopped() {
            return head;
        }
        let least = head + rh * MIN_ROWS;
        if let Some(h) = self.user_h {
            return ph2d_editor_core::math::safe_clamp(h, least, (area.h * USER_SHARE).max(least));
        }
        let cap = (area.h * CARD_SHARE).max(least);
        let body = match self.style {
            CardStyle::Sheet => {
                let table = natural_height(&sheet_rows(c), &self.table, &Metrics::standard());
                let side = rh * (2 + c.stack.len() + c.waiting.len()) as f32;
                table.max(side) + rh
            }
            CardStyle::Cards => {
                let n = c.groups().iter().map(|g| g.1.len()).max().unwrap_or(0);
                rh * (1 + n.max(c.stack.len())) as f32 + pad * 2.0
            }
            CardStyle::Strip => rh * 2.0 + pad,
        };
        (head + body).min(cap)
    }

    /// ⭐ Pinta `c` em `r` (dentro do seu molde, [`crate::cards::open`]).
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        c: &DebugCard,
        ids: ButtonIds,
        r: Rect,
        view: &mut CodeView,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: Theme,
    ) {
        self.painted = CardPainted {
            card: Some(r),
            ..CardPainted::default()
        };
        let pad = Spacing::Sm.px();
        let rh = ph2d_tokens::row_pitch_px();
        let x = r.x + inset();
        let w = (r.w - inset() * 2.0).max(0.0);
        let mut y = r.y + pad;
        head::paint(self, c, ids, Rect::new(x, y, w, rh), scene, ts, theme);
        y += rh;
        let ink = |t: ColorToken| resolve(t, theme);
        if let Some(e) = &c.error {
            let font = TypeToken::Sm.px();
            paint_text(
                ts,
                scene,
                e,
                x,
                y + (rh - font) * 0.5,
                font,
                w,
                ink(ColorToken::Danger),
            );
            y += rh;
        }
        if !c.stopped() {
            return;
        }
        self.paint_grip(r, scene, theme);
        fill_rounded_rect(
            scene,
            Rect::new(r.x, y, r.w, StrokeToken::Thin.px()),
            0.0,
            ink(ColorToken::Border),
        );
        let body = Rect::new(r.x, y, r.w, (r.y + r.h - y - pad).max(0.0));
        match self.style {
            CardStyle::Sheet => sheet::paint(self, c, body, view, scene, ts, theme),
            CardStyle::Cards => boxes::paint_cards(self, c, body, view, scene, ts, theme),
            CardStyle::Strip => boxes::paint_strip(self, c, body, view, scene, ts, theme),
        }
    }

    /// O puxador: a faixa da borda de cima recebe o arrasto; ao meio, um traço curto mostra-o.
    fn paint_grip(&mut self, r: Rect, scene: &mut VectorScene, theme: Theme) {
        let band = ph2d_tokens::PANEL_RESIZE_HANDLE_SIZE_PX;
        self.painted.grip = Some(Rect::new(r.x, r.y - band * 0.5, r.w, band));
        let (w, h) = (Spacing::Xl4.px(), StrokeToken::Thick.px());
        let mark = Rect::new(
            r.x + (r.w - w) * 0.5,
            r.y + (Spacing::Sm.px() - h) * 0.5,
            w,
            h,
        );
        fill_rounded_rect(scene, mark, h * 0.5, resolve(ColorToken::Text3, theme));
    }

    /// Um clique em `(x, y)`: `None` = fora do quadro.
    pub fn press(&mut self, x: f32, y: f32) -> Option<CardClick> {
        if let Some(r) = self
            .painted
            .card
            .filter(|_| self.painted.grip.is_some_and(|g| g.contains(x, y)))
        {
            self.resizing = Some((y, r.h));
            return Some(CardClick::Taken);
        }
        let p = &self.painted;
        if !p.card.is_some_and(|r| r.contains(x, y)) {
            return None;
        }
        let cards = p.cards.iter().enumerate();
        if let Some((i, (d, s))) = cards
            .filter_map(|(i, b)| Some((i, b.track.press(x, y, self.cards_scroll[i])?)))
            .next()
        {
            self.cards_scroll[i] = s;
            self.cards_drag = Some((i, d));
            return Some(CardClick::Taken);
        }
        if let Some(&(ask, _)) = p.buttons.iter().find(|(_, r)| r.contains(x, y)) {
            return Some(CardClick::Ask(ask));
        }
        if let Some(&(line, _)) = p.stack.iter().find(|(_, r)| r.contains(x, y)) {
            return Some(CardClick::Jump(line));
        }
        if let Some((lay, rows, _)) = &p.table
            && let Some(hit) = lay.hit(rows, x, y)
        {
            let copy = match &hit {
                Hit::Cell { row, col: "value" } => rows
                    .get(*row)
                    .and_then(|r| r.cells.get(1))
                    .map(|c| CardClick::Copy(c.text.clone())),
                _ => None,
            };
            let (lay, rows) = (lay.clone(), rows.clone());
            self.table.press(&hit, (x, y), &lay, &rows);
            return Some(copy.unwrap_or(CardClick::Taken));
        }
        Some(
            p.values
                .iter()
                .find(|(_, r)| r.contains(x, y))
                .map_or(CardClick::Taken, |(v, _)| CardClick::Copy(v.clone())),
        )
    }

    /// O ponteiro: o puxador de cima é a mão aberta (fechada a arrastar); a barra da Sheet ou de um
    /// cartão é ↕ por cima dela e durante todo o arrasto (smoke do dono 09/11, cena 14).
    #[must_use]
    pub fn look(&self, x: f32, y: f32) -> Option<ph2d_editor_core::doc_surface::PointerLook> {
        use ph2d_editor_core::doc_surface::PointerLook::{Grab, Grabbing, ResizeNs};
        let p = &self.painted;
        if self.resizing.is_some() {
            return Some(Grabbing);
        }
        if p.grip.is_some_and(|g| g.contains(x, y)) {
            return Some(Grab);
        }
        let sheet = p.table.as_ref().and_then(|(lay, ..)| lay.bar().hit());
        let mut bars = sheet
            .into_iter()
            .chain(p.cards.iter().filter_map(|b| b.track.hit()));
        (self.bar_dragging() || bars.any(|b| b.contains(x, y))).then_some(ResizeNs)
    }

    /// O arrasto do puxador de cima (a altura) ou do de uma coluna. `true` = tomado.
    pub fn drag_to(&mut self, x: f32, y: f32) -> bool {
        if let Some((y0, h0)) = self.resizing {
            self.user_h = Some((h0 + y0 - y).max(0.0));
            return true;
        }
        if let Some((i, d)) = self.cards_drag {
            self.cards_scroll[i] = d.scroll_at(y);
            return true;
        }
        self.table.dragging() && {
            self.table.drag_to((x, y), &Metrics::standard());
            true
        }
    }

    /// Largar o botão (o puxador de cima, o de uma coluna ou uma barra). `true` = havia arrasto.
    pub fn release(&mut self) -> bool {
        let resized = self.resizing.take().is_some() | self.cards_drag.take().is_some();
        self.table.release() || resized
    }

    /// Uma barra do quadro (Sheet ou um cartão) está a ser arrastada?
    #[must_use]
    pub fn bar_dragging(&self) -> bool {
        self.cards_drag.is_some() || self.table.bar_dragging()
    }

    /// A roda em `(x, y)`: sobre o quadro rola a tabela da Sheet ou o cartão debaixo dela. `true` = sobre o
    /// quadro (tomada, mesmo sem nada a rolar: o código por baixo não rola).
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        if !self.painted.card.is_some_and(|r| r.contains(x, y)) {
            return false;
        }
        if let Some((lay, ..)) = &self.painted.table {
            let lay = lay.clone();
            let _ = self.table.wheel(&lay, x, y, dy);
        } else if let Some(i) = self
            .painted
            .cards
            .iter()
            .position(|b| b.rect.contains(x, y))
        {
            let max = self.painted.cards[i].max;
            self.cards_scroll[i] =
                ph2d_editor_core::math::safe_clamp(self.cards_scroll[i] - dy, 0.0, max);
        }
        true
    }

    /// O cabeçalho da tabela contém `(x, y)`? (O botão direito abre o menu das colunas.)
    #[must_use]
    pub fn header_at(&self, x: f32, y: f32) -> bool {
        self.painted
            .table
            .as_ref()
            .is_some_and(|(lay, ..)| lay.header.contains(x, y))
    }

    /// As colunas que se escondem: `(chave, título, à vista)`.
    #[must_use]
    pub fn column_menu(&self) -> Vec<(&'static str, String, bool)> {
        let cols = columns();
        self.table
            .column_menu(&cols)
            .into_iter()
            .map(|(k, t, on)| (k, t.to_owned(), on))
            .collect()
    }

    /// ⚠️ SONDA (o condutor das fotos): o ▸ da linha `key` da tabela pintada.
    #[must_use]
    pub fn chevron_of(&self, key: &str) -> Option<Rect> {
        let (lay, rows, _) = self.painted.table.as_ref()?;
        let m = lay.m?;
        let span = lay.rows.iter().find(|s| rows[s.index].key == key)?;
        let first = lay.cols.first()?;
        let x = first.x + m.pad + f32::from(rows[span.index].depth) * m.indent;
        Some(Rect::new(x, span.rect.y, m.chevron(), span.rect.h))
    }
}
