//! ⭐ **A folha** — o popover do Trello (filtro, pesquisa, copiar/mover, ordenar, envelhecimento,
//! modelos, atalhos), descrito como DADOS: uma lista de [`Item`]s com a ação de cada um. Pintar
//! dispõe-na e regista os alvos ([`super::W6::hits`]); o clique seguinte lê-os. Uma folha mais alta
//! que a área rola (a roda sobre ela).

mod draw;
pub(crate) use draw::paint;

use super::Act;
use crate::edit::Field;
use crate::metrics as m;
use crate::paint::{c, field_box, label, shadow, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, paint_text_title_elided, rect_to_vello,
    stroke_rounded_rect,
};
use ph2d_editor_core::widget::AxisBar;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{LabelColor, Member};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::VectorScene;

// ── As medidas da folha (a maquete `Filter.dc.html`: 304 de largura, linhas de 32) ──
pub(crate) const W: f32 = 304.0;
pub(crate) const WIDE: f32 = 420.0;
pub(crate) const PAD: f32 = 12.0;
pub(crate) const ROW: f32 = 32.0;
pub(crate) const TITLE_H: f32 = 40.0;
pub(crate) const CAPTION_H: f32 = 28.0;
pub(crate) const NOTE_H: f32 = 22.0;
/// Cada linha a mais de uma nota quebrada (o texto pequeno).
pub(crate) const NOTE_LINE_H: f32 = 16.0;
pub(crate) const INPUT_H: f32 = 36.0;
pub(crate) const BUTTON_GAP: f32 = 8.0;
pub(crate) const CHECK: f32 = 16.0;
pub(crate) const AVATAR: f32 = 24.0;
pub(crate) const INDENT: f32 = 12.0;
pub(crate) const KEYS_W: f32 = 120.0;
pub(crate) const GAP: f32 = 8.0;
/// O seletor segmentado: a altura e o respiro à volta das opções.
pub(crate) const SEG_H: f32 = 32.0;
pub(crate) const SEG_PAD: f32 = 2.0;
/// A folha mais baixa que ainda se abre por baixo da âncora (a mais alta rola).
pub(crate) const MIN_H: f32 = 240.0;

/// Uma linha da folha.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Item {
    /// O título centrado, com o «×» (a ação da linha é a do fechar).
    Title(String),
    Caption(String),
    /// Texto de apoio, numa linha.
    Note(String),
    /// Uma caixa de texto: o campo com o teclado (`Some`) ou o texto e a sugestão.
    Input {
        field: Option<Field>,
        text: String,
        placeholder: String,
    },
    Check {
        on: bool,
        text: String,
        swatch: Option<LabelColor>,
        person: Option<Member>,
        /// Um ícone num disco: `(ícone, fundo, tinta)` (as janelas da entrega da maquete).
        icon: Option<(IconId, T, T)>,
    },
    Radio {
        on: bool,
        text: String,
    },
    /// Um rótulo à esquerda e um seletor segmentado à direita (o «Match Any / All» da maquete).
    Segment {
        label: String,
        options: Vec<(String, bool, Act)>,
    },
    /// A cabeça de uma lista suspensa: o rótulo por cima e o valor escolhido.
    Select {
        label: String,
        value: String,
        open: bool,
    },
    /// Uma opção da lista suspensa aberta.
    Choice {
        on: bool,
        text: String,
    },
    /// Uma linha de menu (um resultado da pesquisa, uma ação).
    Entry {
        text: String,
        icon: Option<IconId>,
        on: bool,
    },
    Button {
        text: String,
        primary: bool,
    },
    /// Um atalho e o que ele faz.
    Keys {
        keys: String,
        what: String,
    },
}

/// Uma folha: as linhas e a largura.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Sheet {
    pub items: Vec<(Item, Act)>,
    pub w: f32,
    /// As primeiras linhas que NÃO rolam (o título com o «×», e a palavra-chave do filtro): o
    /// cabeçalho fixo da maquete; só o corpo rola por baixo dele.
    pub pinned: usize,
    /// O corpo é mais alto que a folha: a margem da barra à direita ([`bar`]) sai da largura dele.
    pub scrolls: bool,
}

impl Sheet {
    pub fn new(w: f32) -> Self {
        Self {
            items: Vec::new(),
            w,
            pinned: 0,
            scrolls: false,
        }
    }

    pub fn push(&mut self, item: Item, act: Act) -> &mut Self {
        self.items.push((item, act));
        self
    }

    /// O título com o «×» — fixo (as linhas até ele não rolam).
    pub fn title(&mut self, key: &str) -> &mut Self {
        self.push(Item::Title(ph2d_i18n::tr(key).to_owned()), Act::Close);
        self.pin()
    }

    /// As linhas até aqui ficam fixas no topo.
    pub fn pin(&mut self) -> &mut Self {
        self.pinned = self.items.len();
        self
    }

    /// A altura do cabeçalho fixo.
    pub fn pinned_h(&self) -> f32 {
        self.items.iter().take(self.pinned).map(|(i, _)| h(i)).sum()
    }

    pub fn caption(&mut self, key: &str) -> &mut Self {
        self.push(Item::Caption(ph2d_i18n::tr(key).to_owned()), Act::Inside)
    }

    pub fn check(&mut self, on: bool, text: impl Into<String>, act: Act) -> &mut Self {
        self.push(
            Item::Check {
                on,
                text: text.into(),
                swatch: None,
                person: None,
                icon: None,
            },
            act,
        )
    }

    /// A altura do conteúdo.
    pub fn content_h(&self) -> f32 {
        self.items.iter().map(|(i, _)| h(i)).sum::<f32>() + PAD
    }

    /// ⭐ A folha à largura `w` com as explicações e as escolhas QUEBRADAS em linhas (medidas com a
    /// fonte que as pinta): a folha cresce para baixo, nunca elide uma frase.
    #[must_use]
    pub fn fitted(&self, ts: &mut TextSystem, w: f32) -> Self {
        let mut s = self.clone();
        s.w = w;
        let iw = w - 2.0 * PAD - gutter(s.scrolls);
        for (item, _) in &mut s.items {
            match item {
                Item::Note(t) => *t = wrap(ts, t, m::SMALL_PX, iw),
                Item::Radio { text, .. } => {
                    *text = wrap(ts, text, m::BUTTON_TEXT_PX, iw - CHECK - GAP)
                }
                Item::Check {
                    text,
                    swatch: None,
                    person,
                    icon,
                    ..
                } => {
                    let mut tw = iw - CHECK - GAP;
                    if person.is_some() {
                        tw -= AVATAR + GAP;
                    }
                    if icon.is_some() {
                        tw -= m::ICON_PX + m::LABEL_GAP + GAP;
                    }
                    *text = wrap(ts, text, m::BUTTON_TEXT_PX, tw);
                }
                _ => {}
            }
        }
        s
    }
}

/// ⭐ A folha quebrada à largura `w` e o sítio dela (`place`); se o corpo não cabe, de novo com a
/// margem da barra ([`bar`]) tirada à largura do texto (a quebra fica honesta) e posta de novo.
pub(crate) fn lay(
    s: &Sheet,
    ts: &mut TextSystem,
    w: f32,
    place: impl Fn(&Sheet) -> Rect,
) -> (Sheet, Rect) {
    let fit = s.fitted(ts, w);
    let r = place(&fit);
    if fit.content_h() <= r.h {
        return (fit, r);
    }
    let fit = Sheet {
        scrolls: true,
        ..s.clone()
    }
    .fitted(ts, w);
    let r = place(&fit);
    (fit, r)
}

/// O que a margem da barra tira à largura do texto do corpo.
fn gutter(scrolls: bool) -> f32 {
    if scrolls { m::LIST_GUTTER - PAD } else { 0.0 }
}

/// ⭐ A barra da folha em `rect` (`(margem, barra)`), a MESMA das listas: só se o corpo rola. A calha
/// pára antes do canto redondo de baixo.
pub(crate) fn bar(s: &Sheet, rect: Rect) -> Option<(Rect, AxisBar)> {
    let head = s.pinned_h().min(rect.h);
    let body = rect.h - head;
    let area = Rect::new(
        rect.x,
        rect.y + head + GAP / 2.0,
        rect.w,
        body - GAP / 2.0 - m::MENU_RADIUS,
    );
    let content = (s.content_h() - head, body, m::SCROLL_STEP);
    let (band, b) = AxisBar::in_gutter(area, false, (m::LIST_GUTTER, m::LIST_BAR), content);
    (s.scrolls && b.needed()).then_some((band, b))
}

/// `text` quebrado a `width` como o pintor o quebra (o mesmo peso e o mesmo modo de desenho).
fn wrap(ts: &mut TextSystem, text: &str, px: f32, width: f32) -> String {
    let mode = ph2d_text::active_text_rendering();
    let layout = ts.layout_for_rendering(text, px, width, ph2d_text::FontWeight::MEDIUM, mode);
    let lines: Vec<&str> = layout
        .lines()
        .filter_map(|l| text.get(l.text_range()))
        .map(str::trim_end)
        .collect();
    if lines.is_empty() {
        text.to_owned()
    } else {
        lines.join("\n")
    }
}

/// As linhas a mais de um texto quebrado.
#[allow(clippy::cast_precision_loss)]
fn more(t: &str) -> f32 {
    t.matches('\n').count() as f32
}

fn h(i: &Item) -> f32 {
    match i {
        Item::Title(_) => TITLE_H,
        Item::Caption(_) => CAPTION_H,
        Item::Note(t) => NOTE_H + more(t) * NOTE_LINE_H,
        Item::Check { text, .. } | Item::Radio { text, .. } => ROW + more(text) * m::LINE_H,
        Item::Input { .. } => INPUT_H + GAP,
        Item::Select { .. } => CAPTION_H + ROW + GAP / 2.0,
        Item::Button { .. } => ROW + 2.0 * BUTTON_GAP,
        Item::Segment { .. } | Item::Choice { .. } | Item::Entry { .. } | Item::Keys { .. } => ROW,
    }
}

/// Onde a folha cai: por baixo de `anchor` (alinhada à esquerda dele, ou à direita se `right`),
/// dentro de `area`; mais alta que a área = a altura da área (rola).
pub(crate) fn place(s: &Sheet, anchor: Rect, area: Rect, right: bool) -> Rect {
    let w = s.w.min(area.w);
    let hh = s.content_h().min(area.h - 2.0 * PAD).max(TITLE_H);
    let x0 = if right {
        anchor.x + anchor.w - w
    } else {
        anchor.x
    };
    let x = x0
        .min(area.x + area.w - w - PAD)
        .max(area.x + PAD.min(area.w - w));
    let below = anchor.y + anchor.h + m::LABEL_GAP;
    let room = area.y + area.h - PAD - below;
    // Por baixo da âncora se cabe inteira; a que cabe na área sobe para caber; a mais alta que a
    // área fica por baixo da âncora (e rola) se ainda há lugar para uma folha útil.
    let taller = s.content_h() > area.h - 2.0 * PAD;
    if room >= hh || (taller && room >= MIN_H) {
        return Rect::new(x, below, w, hh.min(room));
    }
    let y = (area.y + area.h - PAD - hh).max(area.y + PAD);
    Rect::new(x, y, w, hh)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(n: usize) -> Sheet {
        let mut s = Sheet::new(W);
        s.title("kanban.import.report_title");
        for i in 0..n {
            s.check(false, format!("Label {i}"), Act::Mine);
        }
        s
    }

    /// A barra das listas só aparece na folha cujo corpo não cabe: na margem dela, por baixo do
    /// cabeçalho e antes do canto redondo, e nenhuma linha do corpo a cobre.
    #[test]
    fn a_sheet_shows_the_list_bar_only_when_its_body_overflows() {
        let mut ts = TextSystem::without_system_fonts();
        let area = Rect::new(0.0, 0.0, 800.0, 400.0);
        let at = |s: &Sheet| place(s, Rect::new(10.0, 10.0, 40.0, 20.0), area, false);
        let (short, r) = lay(&rows(3), &mut ts, W, at);
        assert!(
            !short.scrolls && bar(&short, r).is_none(),
            "cabe: sem barra"
        );
        let (tall, r) = lay(&rows(30), &mut ts, W, at);
        let (band, b) = bar(&tall, r).expect("o corpo rola: a barra");
        assert_eq!((band.x + band.w, band.w), (r.x + r.w, m::LIST_GUTTER));
        assert!(b.track.y >= r.y + tall.pinned_h());
        assert!(b.track.y + b.track.h <= r.y + r.h - m::MENU_RADIUS);
        let hits = paint(
            &mut VectorScene::new(),
            &mut ts,
            Theme::default(),
            &tall,
            r,
            0.0,
            None,
        );
        assert!(hits.contains(&(band, Act::Bar)));
        let body = hits.iter().filter(|(_, a)| *a == Act::Mine);
        let v: Vec<_> = body.collect();
        assert!(
            v.len() > 3 && v.iter().all(|(h, _)| h.x + h.w <= band.x),
            "{v:?} {band:?}"
        );
    }
}
