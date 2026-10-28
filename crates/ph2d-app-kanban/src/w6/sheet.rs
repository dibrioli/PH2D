//! ⭐ **A folha** — o popover do Trello (filtro, pesquisa, copiar/mover, ordenar, envelhecimento,
//! modelos, atalhos), descrito como DADOS: uma lista de [`Item`]s com a ação de cada um. Pintar
//! dispõe-na e regista os alvos ([`super::W6::hits`]); o clique seguinte lê-os. Uma folha mais alta
//! que a área rola (a roda sobre ela).

use super::Act;
use crate::edit::Field;
use crate::metrics as m;
use crate::paint::{c, field_box, label, shadow, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, paint_text_title_elided, rect_to_vello,
    stroke_rounded_rect,
};
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
}

impl Sheet {
    pub fn new(w: f32) -> Self {
        Self {
            items: Vec::new(),
            w,
            pinned: 0,
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
}

fn h(i: &Item) -> f32 {
    match i {
        Item::Title(_) => TITLE_H,
        Item::Caption(_) => CAPTION_H,
        Item::Note(_) => NOTE_H,
        Item::Input { .. } => INPUT_H + GAP,
        Item::Select { .. } => CAPTION_H + ROW + GAP / 2.0,
        Item::Button { .. } => ROW + 2.0 * BUTTON_GAP,
        Item::Check { .. }
        | Item::Radio { .. }
        | Item::Segment { .. }
        | Item::Choice { .. }
        | Item::Entry { .. }
        | Item::Keys { .. } => ROW,
    }
}

/// Onde a folha cai: por baixo de `anchor` (alinhada à esquerda dele, ou à direita se `right`),
/// dentro de `area`; mais alta que a área = a altura da área (rola).
pub(crate) fn place(s: &Sheet, anchor: Rect, area: Rect, right: bool) -> Rect {
    let w = s.w.min(area.w);
    let hh = s.content_h().min(area.h - 2.0 * PAD).max(TITLE_H);
    let x0 = if right { anchor.x + anchor.w - w } else { anchor.x };
    let x = x0.min(area.x + area.w - w - PAD).max(area.x + PAD.min(area.w - w));
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

/// O texto de uma linha, centrado na altura dela.
fn line(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    s: &str,
    (x, y, w, row): (f32, f32, f32, f32),
    color: ph2d_vector::Color,
    strong: bool,
) {
    let ty = text_top(ts, y, row, m::BUTTON_TEXT_PX);
    if strong {
        paint_text_title_elided(ts, scene, s, x, ty, m::BUTTON_TEXT_PX, w, color);
    } else {
        paint_text_elided(ts, scene, s, x, ty, m::BUTTON_TEXT_PX, w, color);
    }
}

fn checkbox(scene: &mut VectorScene, theme: Theme, r: Rect, on: bool, round: bool) {
    let radius = if round { r.w / 2.0 } else { m::LABEL_RADIUS };
    if on {
        fill_rounded_rect(scene, r, radius, c(theme, T::Accent));
        if round {
            let d = r.w / 2.0;
            let dot = Rect::new(r.x + d / 2.0, r.y + d / 2.0, d, d);
            fill_rounded_rect(scene, dot, d / 2.0, c(theme, T::AccentFg));
        } else {
            paint_icon(
                scene,
                IconId::Check,
                r,
                c(theme, T::AccentFg),
                StrokeToken::Default.px(),
            );
        }
    } else {
        stroke_rounded_rect(scene, r, radius, m::HOVER_RING, c(theme, T::KanbanCardMeta));
    }
}

/// ⭐ Pinta a folha em `rect` (rolada `scroll`) e devolve os alvos dela (o 1.º é a folha inteira).
#[allow(clippy::too_many_lines)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    s: &Sheet,
    rect: Rect,
    scroll: f32,
    hover: Option<Act>,
) -> Vec<(Rect, Act)> {
    let mut hits = vec![(rect, Act::Inside)];
    shadow(
        scene,
        rect,
        m::MENU_RADIUS,
        c(theme, T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    fill_rounded_rect(scene, rect, m::MENU_RADIUS, c(theme, T::KanbanCard));
    let fg = c(theme, T::KanbanCardFg);
    let meta = c(theme, T::KanbanCardMeta);
    let raised = c(theme, T::KanbanCardRaised);
    scene.push_clip(&rect_to_vello(rect));
    let (x, iw) = (rect.x + PAD, rect.w - 2.0 * PAD);
    // O cabeçalho fixo fica no topo; o corpo rola, recortado por baixo dele.
    let head = s.pinned_h().min(rect.h);
    let body = Rect::new(rect.x, rect.y + head, rect.w, rect.h - head);
    let mut y = rect.y;
    let mut body_hits = None;
    for (idx, (item, act)) in s.items.iter().enumerate() {
        if idx == s.pinned {
            scene.push_clip(&rect_to_vello(body));
            y = body.y - scroll;
            body_hits = Some(hits.len());
        }
        let region = if idx < s.pinned { rect } else { body };
        let ih = h(item);
        let row = Rect::new(rect.x, y, rect.w, ih);
        let visible = row.y + row.h > region.y && row.y < region.y + region.h;
        let on_hover = hover == Some(*act) && *act != Act::Inside;
        match item {
            Item::Title(t) => {
                let w = crate::docs::title_width(ts, t, m::BUTTON_TEXT_PX);
                line(scene, ts, t, (rect.x + (rect.w - w) / 2.0, y, iw, ih), meta, true);
                let close = Rect::new(rect.x + rect.w - PAD / 2.0 - ROW, y + (ih - ROW) / 2.0, ROW, ROW);
                if hover == Some(Act::Close) {
                    fill_rounded_rect(scene, close, m::BUTTON_RADIUS, raised);
                }
                crate::paint::icon(scene, IconId::Close, close, meta);
                if visible {
                    hits.push((close, Act::Close));
                }
                continue_y(&mut y, ih);
                continue;
            }
            Item::Caption(t) => {
                let ty = text_top(ts, y + ih - m::LINE_H, m::LINE_H, m::SMALL_PX);
                paint_text_title_elided(ts, scene, t, x, ty, m::SMALL_PX, iw, meta);
            }
            Item::Note(t) => {
                let ty = text_top(ts, y, ih, m::SMALL_PX);
                paint_text_elided(ts, scene, t, x, ty, m::SMALL_PX, iw, meta);
            }
            Item::Input {
                field,
                text,
                placeholder,
            } => {
                let r = Rect::new(x, y + GAP / 2.0, iw, INPUT_H - GAP / 2.0);
                match field {
                    Some(f) => {
                        let inner = Rect::new(r.x, r.y + m::FIELD_PAD_Y / 2.0, r.w, r.h - m::FIELD_PAD_Y);
                        field_box(scene, ts, theme, f, inner, m::BUTTON_TEXT_PX);
                    }
                    None => {
                        fill_rounded_rect(scene, r, m::FIELD_RADIUS, raised);
                        stroke_rounded_rect(scene, r, m::FIELD_RADIUS, m::HOVER_RING, meta);
                        let (t, col) = if text.is_empty() {
                            (placeholder.as_str(), meta)
                        } else {
                            (text.as_str(), fg)
                        };
                        line(scene, ts, t, (r.x + m::FIELD_PAD_X, r.y, r.w - 2.0 * m::FIELD_PAD_X, r.h), col, false);
                    }
                }
                if visible {
                    hits.push((r, *act));
                }
                continue_y(&mut y, ih);
                continue;
            }
            Item::Check {
                on,
                text,
                swatch,
                person,
                icon,
            } => {
                if on_hover {
                    fill_rounded_rect(scene, row, 0.0, raised);
                }
                let b = Rect::new(x, y + (ih - CHECK) / 2.0, CHECK, CHECK);
                checkbox(scene, theme, b, *on, false);
                let mut tx = b.x + CHECK + GAP;
                if let Some(col) = swatch {
                    let (fill, ink) = label(theme, *col);
                    let sw = (iw - (tx - x)).min(iw);
                    let chip = Rect::new(tx, y + SEG_PAD, sw, ih - 2.0 * SEG_PAD);
                    fill_rounded_rect(scene, chip, m::LABEL_RADIUS, fill);
                    line(scene, ts, text, (chip.x + m::LABEL_PAD_X, y, chip.w - 2.0 * m::LABEL_PAD_X, ih), ink, true);
                } else {
                    if let Some(p) = person {
                        crate::w6::sheet_avatar(scene, ts, theme, (tx, y + (ih - AVATAR) / 2.0), p);
                        tx += AVATAR + GAP;
                    }
                    if let Some((ic, bg, ink)) = icon {
                        let d = m::ICON_PX + m::LABEL_GAP;
                        let ir = Rect::new(tx, y + (ih - d) / 2.0, d, d);
                        fill_rounded_rect(scene, ir, d / 2.0, c(theme, *bg));
                        crate::paint::icon(scene, *ic, ir, c(theme, *ink));
                        tx += d + GAP;
                    }
                    line(scene, ts, text, (tx, y, iw - (tx - x), ih), fg, false);
                }
            }
            Item::Radio { on, text } => {
                if on_hover {
                    fill_rounded_rect(scene, row, 0.0, raised);
                }
                let b = Rect::new(x, y + (ih - CHECK) / 2.0, CHECK, CHECK);
                checkbox(scene, theme, b, *on, true);
                line(scene, ts, text, (b.x + CHECK + GAP, y, iw - CHECK - GAP, ih), fg, false);
            }
            Item::Segment { label: l, options } => {
                let ty = text_top(ts, y, ih, m::SMALL_PX);
                paint_text_title_elided(ts, scene, l, x, ty, m::SMALL_PX, iw, meta);
                let widths: Vec<f32> = options
                    .iter()
                    .map(|(t, _, _)| crate::docs::title_width(ts, t, m::BUTTON_TEXT_PX) + 2.0 * m::BUTTON_PAD_X)
                    .collect();
                let total: f32 = widths.iter().sum::<f32>() + 2.0 * SEG_PAD;
                let bar = Rect::new(x + iw - total, y + (ih - SEG_H) / 2.0, total, SEG_H);
                fill_rounded_rect(scene, bar, m::BUTTON_RADIUS, raised);
                let mut sx = bar.x + SEG_PAD;
                for ((t, on, a), w) in options.iter().zip(widths) {
                    let r = Rect::new(sx, bar.y + SEG_PAD, w, SEG_H - 2.0 * SEG_PAD);
                    let ink = if *on {
                        fill_rounded_rect(scene, r, m::LABEL_RADIUS, c(theme, T::Accent));
                        c(theme, T::AccentFg)
                    } else {
                        if hover == Some(*a) {
                            fill_rounded_rect(scene, r, m::LABEL_RADIUS, c(theme, T::KanbanSlot));
                        }
                        fg
                    };
                    line(scene, ts, t, (r.x + m::BUTTON_PAD_X, r.y, w, r.h), ink, *on);
                    if visible {
                        hits.push((r, *a));
                    }
                    sx += w;
                }
            }
            Item::Select { label: l, value, open } => {
                let ty = text_top(ts, y + CAPTION_H - m::LINE_H, m::LINE_H, m::SMALL_PX);
                paint_text_title_elided(ts, scene, l, x, ty, m::SMALL_PX, iw, meta);
                let r = Rect::new(x, y + CAPTION_H, iw, ROW);
                fill_rounded_rect(scene, r, m::FIELD_RADIUS, raised);
                let ring = if *open || on_hover { c(theme, T::Accent) } else { meta };
                stroke_rounded_rect(scene, r, m::FIELD_RADIUS, m::HOVER_RING, ring);
                line(scene, ts, value, (r.x + m::FIELD_PAD_X, r.y, r.w - ROW, r.h), fg, false);
                let chev = if *open { IconId::ChevronUp } else { IconId::ChevronDown };
                crate::paint::icon(scene, chev, Rect::new(r.x + r.w - ROW, r.y, ROW, ROW), meta);
                if visible {
                    hits.push((r, *act));
                }
                continue_y(&mut y, ih);
                continue;
            }
            Item::Choice { on, text } => {
                if *on || on_hover {
                    fill_rounded_rect(scene, row, 0.0, raised);
                }
                let col = if *on { c(theme, T::Accent) } else { fg };
                line(scene, ts, text, (x + INDENT, y, iw - INDENT, ih), col, *on);
            }
            Item::Entry { text, icon, on } => {
                if *on || on_hover {
                    fill_rounded_rect(scene, row, 0.0, raised);
                }
                let mut tx = x;
                if let Some(ic) = icon {
                    crate::paint::icon(scene, *ic, Rect::new(x, y, m::ICON_PX, ih), meta);
                    tx += m::ICON_PX + GAP;
                }
                line(scene, ts, text, (tx, y, iw - (tx - x), ih), fg, false);
            }
            Item::Button { text, primary } => {
                let r = Rect::new(x, y + BUTTON_GAP, iw, ROW);
                let (bg, ink) = if *primary {
                    let t = if on_hover { T::AccentHover } else { T::Accent };
                    (c(theme, t), c(theme, T::AccentFg))
                } else {
                    (raised, fg)
                };
                fill_rounded_rect(scene, r, m::BUTTON_RADIUS, bg);
                let w = crate::docs::title_width(ts, text, m::BUTTON_TEXT_PX);
                line(scene, ts, text, (r.x + ((r.w - w) / 2.0).max(0.0), r.y, r.w, r.h), ink, true);
                if visible {
                    hits.push((r, *act));
                }
                continue_y(&mut y, ih);
                continue;
            }
            Item::Keys { keys, what } => {
                let kw = crate::docs::title_width(ts, keys, m::SMALL_PX) + 2.0 * m::COUNT_PAD_X;
                let chip = Rect::new(x, y + (ih - m::COUNT_H) / 2.0, kw.min(KEYS_W), m::COUNT_H);
                fill_rounded_rect(scene, chip, m::LIMIT_RADIUS, raised);
                let ty = text_top(ts, chip.y, chip.h, m::SMALL_PX);
                paint_text_title_elided(ts, scene, keys, chip.x + m::COUNT_PAD_X, ty, m::SMALL_PX, KEYS_W, fg);
                line(scene, ts, what, (x + KEYS_W + GAP, y, iw - KEYS_W - GAP, ih), fg, false);
            }
        }
        if visible && *act != Act::Inside {
            hits.push((row, *act));
        }
        continue_y(&mut y, ih);
    }
    if let Some(from) = body_hits {
        scene.pop_layer();
        // Um alvo do corpo meio escondido pelo cabeçalho só se acerta na parte à vista.
        let clipped: Vec<(Rect, Act)> = hits
            .drain(from..)
            .filter_map(|(r, a)| clip(r, body).map(|r| (r, a)))
            .collect();
        hits.extend(clipped);
    }
    scene.pop_layer();
    hits
}

/// `r` recortado a `to` (`None` = nada à vista).
fn clip(r: Rect, to: Rect) -> Option<Rect> {
    let (y0, y1) = (r.y.max(to.y), (r.y + r.h).min(to.y + to.h));
    (y1 > y0).then(|| Rect::new(r.x, y0, r.w, y1 - y0))
}

fn continue_y(y: &mut f32, h: f32) {
    *y += h;
}
