//! A TINTA da [`super::Sheet`]: dispõe as linhas na folha rolada, pinta-as e devolve os alvos.

use super::*;

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

/// Um texto quebrado (por [`Sheet::fitted`]) numa linha de escolha: a 1.ª ao centro da linha de
/// `ROW`, as outras por baixo, `LINE_H` a `LINE_H`.
fn lines(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    s: &str,
    (x, y, w, row): (f32, f32, f32, f32),
    color: ph2d_vector::Color,
) {
    if !s.contains('\n') {
        line(scene, ts, s, (x, y, w, row), color, false);
        return;
    }
    let mut ly = y;
    for l in s.split('\n') {
        line(scene, ts, l, (x, ly, w, ROW), color, false);
        ly += m::LINE_H;
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
    let (x, full) = (rect.x + PAD, rect.w - 2.0 * PAD);
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
        let (region, g) = if idx < s.pinned {
            (rect, 0.0)
        } else {
            (body, gutter(s.scrolls))
        };
        let (ih, iw) = (h(item), full - g);
        // A linha que rola pára na margem da barra.
        let rw = rect.w - if g > 0.0 { m::LIST_GUTTER } else { 0.0 };
        let row = Rect::new(rect.x, y, rw, ih);
        let visible = row.y + row.h > region.y && row.y < region.y + region.h;
        let on_hover = hover == Some(*act) && *act != Act::Inside;
        match item {
            Item::Title(t) => {
                let w = crate::docs::title_width(ts, t, m::BUTTON_TEXT_PX);
                line(
                    scene,
                    ts,
                    t,
                    (rect.x + (rect.w - w) / 2.0, y, iw, ih),
                    meta,
                    true,
                );
                let close = Rect::new(
                    rect.x + rect.w - PAD / 2.0 - ROW,
                    y + (ih - ROW) / 2.0,
                    ROW,
                    ROW,
                );
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
                let mut ly = y;
                for l in t.split('\n') {
                    let ty = text_top(ts, ly, NOTE_H, m::SMALL_PX);
                    paint_text_elided(ts, scene, l, x, ty, m::SMALL_PX, iw, meta);
                    ly += NOTE_LINE_H;
                }
            }
            Item::Input {
                field,
                text,
                placeholder,
            } => {
                let r = Rect::new(x, y + GAP / 2.0, iw, INPUT_H - GAP / 2.0);
                match field {
                    Some(f) => {
                        let inner =
                            Rect::new(r.x, r.y + m::FIELD_PAD_Y / 2.0, r.w, r.h - m::FIELD_PAD_Y);
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
                        line(
                            scene,
                            ts,
                            t,
                            (r.x + m::FIELD_PAD_X, r.y, r.w - 2.0 * m::FIELD_PAD_X, r.h),
                            col,
                            false,
                        );
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
                let b = Rect::new(x, y + (ROW.min(ih) - CHECK) / 2.0, CHECK, CHECK);
                checkbox(scene, theme, b, *on, false);
                let mut tx = b.x + CHECK + GAP;
                if let Some(col) = swatch {
                    let (fill, ink) = label(theme, *col);
                    let sw = (iw - (tx - x)).min(iw);
                    let chip = Rect::new(tx, y + SEG_PAD, sw, ih - 2.0 * SEG_PAD);
                    fill_rounded_rect(scene, chip, m::LABEL_RADIUS, fill);
                    line(
                        scene,
                        ts,
                        text,
                        (
                            chip.x + m::LABEL_PAD_X,
                            y,
                            chip.w - 2.0 * m::LABEL_PAD_X,
                            ih,
                        ),
                        ink,
                        true,
                    );
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
                    lines(scene, ts, text, (tx, y, iw - (tx - x), ih), fg);
                }
            }
            Item::Radio { on, text } => {
                if on_hover {
                    fill_rounded_rect(scene, row, 0.0, raised);
                }
                let b = Rect::new(x, y + (ROW - CHECK) / 2.0, CHECK, CHECK);
                checkbox(scene, theme, b, *on, true);
                lines(
                    scene,
                    ts,
                    text,
                    (b.x + CHECK + GAP, y, iw - CHECK - GAP, ih),
                    fg,
                );
            }
            Item::Segment { label: l, options } => {
                let ty = text_top(ts, y, ih, m::SMALL_PX);
                paint_text_title_elided(ts, scene, l, x, ty, m::SMALL_PX, iw, meta);
                let widths: Vec<f32> = options
                    .iter()
                    .map(|(t, _, _)| {
                        crate::docs::title_width(ts, t, m::BUTTON_TEXT_PX) + 2.0 * m::BUTTON_PAD_X
                    })
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
            Item::Select {
                label: l,
                value,
                open,
            } => {
                let ty = text_top(ts, y + CAPTION_H - m::LINE_H, m::LINE_H, m::SMALL_PX);
                paint_text_title_elided(ts, scene, l, x, ty, m::SMALL_PX, iw, meta);
                let r = Rect::new(x, y + CAPTION_H, iw, ROW);
                fill_rounded_rect(scene, r, m::FIELD_RADIUS, raised);
                let ring = if *open || on_hover {
                    c(theme, T::Accent)
                } else {
                    meta
                };
                stroke_rounded_rect(scene, r, m::FIELD_RADIUS, m::HOVER_RING, ring);
                line(
                    scene,
                    ts,
                    value,
                    (r.x + m::FIELD_PAD_X, r.y, r.w - ROW, r.h),
                    fg,
                    false,
                );
                let chev = if *open {
                    IconId::ChevronUp
                } else {
                    IconId::ChevronDown
                };
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
                line(
                    scene,
                    ts,
                    text,
                    (r.x + ((r.w - w) / 2.0).max(0.0), r.y, r.w, r.h),
                    ink,
                    true,
                );
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
                paint_text_title_elided(
                    ts,
                    scene,
                    keys,
                    chip.x + m::COUNT_PAD_X,
                    ty,
                    m::SMALL_PX,
                    KEYS_W,
                    fg,
                );
                line(
                    scene,
                    ts,
                    what,
                    (x + KEYS_W + GAP, y, iw - KEYS_W - GAP, ih),
                    fg,
                    false,
                );
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
        if scroll > 0.0 {
            let rule = Rect::new(rect.x, body.y, rect.w, StrokeToken::Hairline.px());
            scene.fill_rect(rect_to_vello(rule), meta);
        }
    }
    if let Some((band, b)) = bar(s, rect) {
        let thumb = if hover == Some(Act::Bar) { fg } else { meta };
        b.paint(scroll, (raised, thumb), scene);
        hits.push((band, Act::Bar));
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
