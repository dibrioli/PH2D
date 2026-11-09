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
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::{Column, Hit, Layout, Metrics, Row, TableState, natural_height};
use ph2d_vector::VectorScene;

/// A parte da altura da aba que o quadro pode tomar: o código manda, a tabela rola.
const CARD_SHARE: f32 = 1.0 / 2.0; // LITERAL-PX-OK: fração da aba (metade), não píxeis
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
        let cap = (area.h * CARD_SHARE).max(head + rh * MIN_ROWS);
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

    /// Um clique em `(x, y)`: `None` = fora do quadro.
    pub fn press(&mut self, x: f32, y: f32) -> Option<CardClick> {
        let p = &self.painted;
        if !p.card.is_some_and(|r| r.contains(x, y)) {
            return None;
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
            self.table.press(&hit, x, &lay, &rows);
            return Some(copy.unwrap_or(CardClick::Taken));
        }
        Some(
            p.values
                .iter()
                .find(|(_, r)| r.contains(x, y))
                .map_or(CardClick::Taken, |(v, _)| CardClick::Copy(v.clone())),
        )
    }

    /// O arrasto do puxador de uma coluna. `true` = tomado.
    pub fn drag_to(&mut self, x: f32) -> bool {
        self.table.dragging() && {
            self.table.drag_to(x, &Metrics::standard());
            true
        }
    }

    /// A roda sobre a tabela. `true` = rolou.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        let Some((lay, ..)) = &self.painted.table else {
            return false;
        };
        let lay = lay.clone();
        self.table.wheel(&lay, x, y, dy)
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
