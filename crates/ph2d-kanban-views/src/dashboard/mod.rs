//! ⭐ A vista **Painel** (o Dashboard do Trello): uma fila de números (cartões, atrasados, em 24 h,
//! concluídos) e quatro gráficos — cartões por lista, por etiqueta, por membro e por data de
//! entrega. Lista e entrega PARTEM os cartões e podem ser barras ou anel; etiqueta e membro não
//! (um cartão conta em várias), por isso são só barras. Ao passar o rato, a dica dá o número exacto.
//! O método e as marcas são os da skill de dataviz, sobre os tokens `kanban-*`.

mod chart;
pub mod data;
mod metrics;

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, paint_text_title};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::{KanbanId, KanbanSet};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, PointerKind, Reply, c, text_top};
pub use data::{Bar, Bucket, Chart, Stats, Who, stats};
use metrics as m;

/// Barras ou anel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Form {
    #[default]
    Bars,
    Ring,
}

/// O estado do painel (da VISTA).
#[derive(Debug, Default)]
pub struct DashboardState {
    /// A forma de cada gráfico que a pode mudar (lista, entrega); por omissão: lista em barras,
    /// entrega em anel.
    pub forms: [Option<Form>; 4],
    /// A barra (ou fatia) sob o rato: o gráfico e o índice.
    pub hover: Option<(Chart, usize)>,
    pub pointer: (f32, f32),
    painted: Vec<(Chart, Rect, [Rect; 2], chart::Painted)>,
}

fn index(c: Chart) -> usize {
    Chart::ALL.iter().position(|x| *x == c).unwrap_or(0)
}

impl DashboardState {
    /// A forma do gráfico `c`.
    #[must_use]
    pub fn form(&self, c: Chart) -> Form {
        if !c.partitions() {
            return Form::Bars;
        }
        self.forms[index(c)].unwrap_or(if c == Chart::PerDue {
            Form::Ring
        } else {
            Form::Bars
        })
    }

    /// As barras que o gráfico `c` pintou no último quadro (já dobradas).
    #[must_use]
    pub fn painted_bars(&self, c: Chart) -> Option<&[Bar]> {
        self.painted
            .iter()
            .find(|p| p.0 == c)
            .map(|p| p.3.bars.as_slice())
    }

    /// O centro do alvo da barra `i` do gráfico `c` (para os testes e a a11y).
    #[must_use]
    pub fn bar_at(&self, c: Chart, i: usize) -> Option<(f32, f32)> {
        let p = &self.painted.iter().find(|p| p.0 == c)?.3;
        if let Some(r) = &p.ring {
            let (a0, a1, _) = r.spans.iter().find(|s| s.2 == i)?;
            let a = (a0 + a1) / 2.0 - std::f32::consts::FRAC_PI_2;
            let rr = (r.r_in + r.r_out) / 2.0;
            return Some((r.center.0 + rr * a.cos(), r.center.1 + rr * a.sin()));
        }
        let (rect, _) = p.rows.iter().find(|x| x.1 == i)?;
        Some((rect.x + rect.w / 2.0, rect.y + rect.h / 2.0))
    }

    /// O centro do botão `form` do gráfico `c`, se ele o tem.
    #[must_use]
    pub fn toggle_at(&self, c: Chart, form: Form) -> Option<(f32, f32)> {
        let p = self.painted.iter().find(|p| p.0 == c && c.partitions())?;
        let r = p.2[usize::from(form == Form::Ring)];
        Some((r.x + r.w / 2.0, r.y + r.h / 2.0))
    }
}

pub(crate) fn vrect(r: Rect) -> ph2d_vector::Rect {
    ph2d_vector::Rect::new(
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.x + r.w),
        f64::from(r.y + r.h),
    )
}

fn contains(r: Rect, x: f32, y: f32) -> bool {
    r.x <= x && x < r.x + r.w && r.y <= y && y < r.y + r.h
}

/// Um número da fila de cima: o rótulo (com a marca de ESTADO, se é um) e o valor.
#[allow(clippy::too_many_arguments)]
fn tile(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: Rect,
    label: &str,
    value: usize,
    mark: Option<T>,
) {
    fill_rounded_rect(out, r, m::RADIUS, c(theme, T::KanbanCard));
    let mut x = r.x + m::CARD_PAD;
    let y = r.y + m::CARD_PAD;
    let lh = m::TILE_LABEL_PX * m::LINE_RATIO;
    if let Some(t) = mark {
        let my = y + (lh - m::TILE_MARK) / 2.0;
        fill_rounded_rect(
            out,
            Rect::new(x, my, m::TILE_MARK, m::TILE_MARK),
            m::TILE_MARK / 2.0,
            c(theme, t),
        );
        x += m::TILE_MARK + m::TILE_MARK_GAP;
    }
    let ty = text_top(ts, y, lh, m::TILE_LABEL_PX);
    paint_text_elided(
        ts,
        out,
        label,
        x,
        ty,
        m::TILE_LABEL_PX,
        r.x + r.w - x - m::CARD_PAD,
        c(theme, T::KanbanCardMeta),
    );
    let vy = y + lh + m::TILE_MARK_GAP;
    paint_text_title(
        ts,
        out,
        &value.to_string(),
        r.x + m::CARD_PAD,
        vy,
        m::TILE_VALUE_PX,
        r.w - 2.0 * m::CARD_PAD,
        c(theme, T::KanbanCardFg),
    );
}

/// O seletor Barras | Anel do cabeçalho de um gráfico (direita); devolve os dois botões.
fn toggle(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    right: f32,
    y: f32,
    form: Form,
) -> [Rect; 2] {
    let names = [
        tr("kanban.views.dashboard.bars"),
        tr("kanban.views.dashboard.ring"),
    ];
    let ws = names.map(|n| ts.prefix_width(n, m::SEG_PX) + 2.0 * m::SEG_PAD_X);
    let mut x = right - ws[0] - ws[1];
    let mut rects = [Rect::new(0.0, 0.0, 0.0, 0.0); 2];
    let whole = Rect::new(x, y, ws[0] + ws[1], m::SEG_H);
    fill_rounded_rect(out, whole, m::SEG_RADIUS, c(theme, T::KanbanSlot));
    for (i, n) in names.iter().enumerate() {
        let r = Rect::new(x, y, ws[i], m::SEG_H);
        let on = usize::from(form == Form::Ring) == i;
        if on {
            fill_rounded_rect(out, r, m::SEG_RADIUS, c(theme, T::KanbanBlueBold));
        }
        let ink = if on {
            c(theme, T::KanbanLabelInkBold)
        } else {
            c(theme, T::KanbanCardFg)
        };
        let ty = text_top(ts, y, m::SEG_H, m::SEG_PX);
        paint_text_elided(ts, out, n, x + m::SEG_PAD_X, ty, m::SEG_PX, ws[i], ink);
        rects[i] = r;
        x += ws[i];
    }
    rects
}

/// A dica sob o rato: a chave de linha na cor da série, o VALOR em destaque e o nome por baixo.
fn tip(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    at: (f32, f32),
    bar: &Bar,
    total: Option<usize>,
) {
    let one = if bar.count == 1 { "_one" } else { "" };
    let value = match total.filter(|t| *t > 0) {
        Some(t) => tr_with(
            &format!("kanban.views.dashboard.tip_share{one}"),
            &[("n", &bar.count), ("p", &((bar.count * 100 + t / 2) / t))],
        ),
        None => tr_with(
            &format!("kanban.views.dashboard.tip{one}"),
            &[("n", &bar.count)],
        ),
    };
    let vw = ts.prefix_width_weighted(&value, m::TIP_VALUE_PX, FontWeight::SEMI_BOLD);
    let nw = ts.prefix_width(&bar.name, m::SMALL_PX);
    let w = vw.max(nw) + m::TIP_KEY_W + 3.0 * m::TIP_PAD;
    let h = m::TIP_VALUE_PX * m::LINE_RATIO + m::SMALL_PX * m::LINE_RATIO + 2.0 * m::TIP_PAD;
    let mut x = at.0 + m::TIP_OFFSET;
    let mut y = at.1 + m::TIP_OFFSET;
    if x + w > area.x + area.w {
        x = at.0 - m::TIP_OFFSET - w;
    }
    if y + h > area.y + area.h {
        y = at.1 - m::TIP_OFFSET - h;
    }
    let r = Rect::new(x, y, w, h);
    let shadow = Rect::new(r.x, r.y + m::TIP_SHADOW_DY, r.w, r.h);
    fill_rounded_rect(out, shadow, m::TIP_RADIUS, c(theme, T::KanbanShadowHover));
    fill_rounded_rect(out, r, m::TIP_RADIUS, c(theme, T::KanbanCard));
    let key = Rect::new(
        r.x + m::TIP_PAD,
        r.y + m::TIP_PAD + m::TIP_VALUE_PX * m::TIP_KEY_RISE,
        m::TIP_KEY_W,
        m::TIP_KEY_H,
    );
    let color = match &bar.who {
        Who::Label(_, Some(l)) => crate::common::label_colors(theme, *l).0,
        who => c(theme, who.token(theme)),
    };
    fill_rounded_rect(out, key, m::TIP_KEY_H / 2.0, color);
    let tx = key.x + m::TIP_KEY_W + m::TIP_PAD;
    paint_text_title(
        ts,
        out,
        &value,
        tx,
        r.y + m::TIP_PAD,
        m::TIP_VALUE_PX,
        w,
        c(theme, T::KanbanCardFg),
    );
    paint_text_elided(
        ts,
        out,
        &bar.name,
        tx,
        r.y + m::TIP_PAD + m::TIP_VALUE_PX * m::LINE_RATIO,
        m::SMALL_PX,
        nw + m::TIP_PAD,
        c(theme, T::KanbanCardMeta),
    );
}

/// ⭐ Pinta o painel em `area`.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut DashboardState,
    set: &KanbanSet,
    kanban: KanbanId,
    cx: &Ctx<'_>,
) {
    st.painted.clear();
    let Some(k) = set.get(kanban) else {
        return;
    };
    let s = stats(k, cx);
    let inner = Rect::new(
        area.x + m::PAD,
        area.y + m::PAD,
        area.w - 2.0 * m::PAD,
        area.h - 2.0 * m::PAD,
    );
    let tiles = [
        ("kanban.views.dashboard.kpi.cards", s.total, None),
        (
            "kanban.views.dashboard.due.overdue",
            s.due(Bucket::Overdue),
            Some(Bucket::Overdue.token()),
        ),
        (
            "kanban.views.dashboard.due.next_day",
            s.due(Bucket::NextDay),
            Some(Bucket::NextDay.token()),
        ),
        (
            "kanban.views.dashboard.due.complete",
            s.due(Bucket::Complete),
            Some(Bucket::Complete.token()),
        ),
    ];
    #[allow(clippy::cast_precision_loss)]
    let tw = (inner.w - m::GAP * (tiles.len() - 1) as f32) / tiles.len() as f32;
    for (i, (key, v, mark)) in tiles.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(inner.x + (tw + m::GAP) * i as f32, inner.y, tw, m::TILE_H);
        tile(out, ts, theme, r, tr(key), v, mark);
    }
    let gy = inner.y + m::TILE_H + m::GAP;
    let cw = (inner.w - m::GAP) / 2.0;
    let ch = ((inner.y + inner.h - gy) - m::GAP) / 2.0;
    for (i, chart) in Chart::ALL.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let card = Rect::new(
            inner.x + (cw + m::GAP) * (i % 2) as f32,
            gy + (ch + m::GAP) * (i / 2) as f32,
            cw,
            ch,
        );
        fill_rounded_rect(out, card, m::RADIUS, c(theme, T::KanbanCard));
        let hx = card.x + m::CARD_PAD;
        let hy = card.y + m::CARD_PAD;
        let ty = text_top(ts, hy, m::HEAD_H, m::TITLE_PX);
        let form = st.form(chart);
        let toggles = if chart.partitions() {
            toggle(
                out,
                ts,
                theme,
                card.x + card.w - m::CARD_PAD,
                hy + (m::HEAD_H - m::SEG_H) / 2.0,
                form,
            )
        } else {
            [Rect::new(0.0, 0.0, 0.0, 0.0); 2]
        };
        let title_w = toggles[0].x.max(hx) - hx;
        let title_w = if chart.partitions() {
            title_w
        } else {
            card.w - 2.0 * m::CARD_PAD
        };
        paint_text_title(
            ts,
            out,
            tr(chart.title_key()),
            hx,
            ty,
            m::TITLE_PX,
            title_w,
            c(theme, T::KanbanCardFg),
        );
        let body = Rect::new(
            hx,
            hy + m::HEAD_H + m::HEAD_GAP,
            card.w - 2.0 * m::CARD_PAD,
            (card.h - 2.0 * m::CARD_PAD - m::HEAD_H - m::HEAD_GAP).max(0.0),
        );
        let hover = st.hover.filter(|h| h.0 == chart).map(|h| h.1);
        let bars = s.bars(chart);
        let painted = if s.total == 0 && form == Form::Bars && bars.iter().all(|b| b.count == 0) {
            let y = text_top(ts, body.y, body.h, m::BODY_PX);
            paint_text_elided(
                ts,
                out,
                tr("kanban.views.no_cards"),
                body.x,
                y,
                m::BODY_PX,
                body.w,
                c(theme, T::KanbanCardMeta),
            );
            chart::Painted::default()
        } else if form == Form::Ring {
            chart::ring(out, ts, theme, body, bars, hover)
        } else {
            chart::bars(out, ts, theme, body, bars, hover)
        };
        st.painted.push((chart, card, toggles, painted));
    }
    if let Some((chart, i)) = st.hover
        && let Some(p) = st.painted.iter().find(|p| p.0 == chart)
        && let Some(bar) = p.3.bars.get(i)
    {
        let total = chart.partitions().then_some(s.total);
        tip(out, ts, theme, area, st.pointer, bar, total);
    }
}

/// O rato: pairar mostra a dica; carregar num botão muda a forma do gráfico.
pub fn pointer(
    st: &mut DashboardState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    p: PointerAt,
) -> Reply {
    st.pointer = (p.x, p.y);
    match p.kind {
        PointerKind::Move => {
            let hover = st.painted.iter().find_map(|(chart, card, _, painted)| {
                contains(*card, p.x, p.y)
                    .then(|| painted.hit(p.x, p.y).map(|i| (*chart, i)))
                    .flatten()
            });
            let moved = hover.is_some() || hover != st.hover;
            st.hover = hover;
            Reply {
                redraw: moved,
                ..Reply::default()
            }
        }
        PointerKind::Down => {
            for (chart, _, toggles, _) in &st.painted {
                if !chart.partitions() {
                    continue;
                }
                for (j, r) in toggles.iter().enumerate() {
                    if contains(*r, p.x, p.y) {
                        let form = if j == 1 { Form::Ring } else { Form::Bars };
                        st.forms[index(*chart)] = Some(form);
                        st.hover = None;
                        return Reply::taken();
                    }
                }
            }
            Reply::default()
        }
        PointerKind::Up => Reply::default(),
    }
}
