//! ⭐ **O quadro pintado** — o fundo, o cabeçalho, os campos de texto e o que voa. As listas moram
//! em [`crate::paint_list`], a frente do cartão em [`crate::paint_card`]; os menus e o arquivo nos
//! módulos deles. Tudo pelas cores `kanban-*` e pelas medidas da maquete ([`crate::metrics`]).

use crate::edit::{Field, Target};
use crate::layout::ListBox;
use crate::metrics as m;
use crate::paint_card::{self, CardLook, Ink};
use crate::paint_list;
use crate::tab::{Hit, Lift, Tab};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_block, paint_text_elided, paint_text_title_elided,
    rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{Background, Hue, Kanban, LabelColor, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{Affine, Color, VectorScene};

/// A cor de um token no tema.
pub(crate) fn c(theme: Theme, t: T) -> Color {
    resolve(t, theme)
}

fn hue_index(h: Hue) -> usize {
    Hue::ALL.iter().position(|x| *x == h).unwrap_or(0)
}

fn shade_index(s: Shade) -> usize {
    Shade::ALL.iter().position(|x| *x == s).unwrap_or(0)
}

/// A cor de uma das 30 etiquetas (e a tinta que a lê).
pub(crate) fn label(theme: Theme, l: LabelColor) -> (Color, Color) {
    let s = shade_index(l.shade);
    let tok = ph2d_tokens::kanban::LABELS[s][hue_index(l.hue)];
    (c(theme, tok), c(theme, ph2d_tokens::kanban::label_ink(s)))
}

/// Uma lista com cor usa a intensidade SUTIL do matiz (a paleta do Trello) e a tinta das etiquetas —
/// a MESMA lei que o Quadro pinta no nível baixo (`ph2d_board_render::kanban_list_colors`).
pub(crate) fn list_colors(theme: Theme, hue: Option<Hue>) -> (Color, Color, Color) {
    ph2d_board_render::kanban_list_colors(theme, hue)
}

/// O fundo do quadro: o azul do Trello; outro matiz usa a etiqueta forte dele (a lei do Quadro).
fn board_bg(theme: Theme, bg: Background) -> Color {
    ph2d_board_render::kanban_board_bg(theme, bg)
}

/// O topo de uma linha de texto de `px` centrada numa faixa de `row_h` que começa em `y`.
pub(crate) fn text_top(ts: &mut TextSystem, y: f32, row_h: f32, px: f32) -> f32 {
    let h = ts.layout("Ag", px, f32::INFINITY).height();
    y + ((row_h - h) / 2.0).max(0.0)
}

pub(crate) fn icon(scene: &mut VectorScene, id: IconId, around: Rect, color: Color) {
    let r = Rect::new(
        around.x + (around.w - m::ICON_PX) / 2.0,
        around.y + (around.h - m::ICON_PX) / 2.0,
        m::ICON_PX,
        m::ICON_PX,
    );
    paint_icon(scene, id, r, color, StrokeToken::Default.px());
}

/// O fundo de um botão do cabeçalho ou da lista, quando paira.
pub(crate) fn hover_fill(scene: &mut VectorScene, r: Rect, on: bool, theme: Theme, radius: f32) {
    if on {
        fill_rounded_rect(scene, r, radius, c(theme, T::KanbanSlot));
    }
}

/// ⭐ O quadro inteiro, a partir da disposição VISTA.
pub(crate) fn board(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &Tab,
    looks: &mut dyn FnMut(ph2d_kanban_model::CardId) -> Option<CardLook>,
) {
    let l = &tab.layout;
    scene.fill_rect(rect_to_vello(l.area), board_bg(theme, k.header.background));
    header(scene, ts, theme, k, tab);
    scene.push_clip(&rect_to_vello(l.board));
    if let Some(s) = l.list_slot {
        fill_rounded_rect(scene, s, m::LIST_RADIUS, c(theme, T::KanbanNewList));
    }
    for lb in &l.lists {
        if lb.collapsed {
            paint_list::collapsed(scene, ts, theme, k, tab, lb);
        } else {
            paint_list::list(scene, ts, theme, k, tab, lb, looks);
        }
    }
    paint_list::add_list(scene, ts, theme, tab);
    scene.pop_layer();
    flying(scene, ts, theme, k, tab, looks);
}

fn header(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, k: &Kanban, tab: &Tab) {
    let l = &tab.layout;
    scene.fill_rect(rect_to_vello(l.header), c(theme, T::KanbanHeaderBg));
    let fg = c(theme, T::KanbanHeaderFg);
    match tab
        .field
        .as_ref()
        .filter(|f| f.target == Target::KanbanName)
    {
        Some(f) => field_box(scene, ts, theme, f, l.title, m::HEADER_TITLE_PX),
        None => {
            hover_fill(
                scene,
                l.title,
                tab.hover == Some(Hit::KanbanTitle),
                theme,
                m::BUTTON_RADIUS,
            );
            let y = text_top(ts, l.title.y, l.title.h, m::HEADER_TITLE_PX);
            paint_text_title_elided(
                ts,
                scene,
                &k.header.name,
                l.title.x + m::FIELD_PAD_X,
                y,
                m::HEADER_TITLE_PX,
                l.title.w - 2.0 * m::FIELD_PAD_X,
                fg,
            );
        }
    }
    hover_fill(
        scene,
        l.star,
        tab.hover == Some(Hit::Star),
        theme,
        m::BUTTON_RADIUS,
    );
    star(scene, l.star, k.header.starred, theme);
    hover_fill(
        scene,
        l.board_menu,
        tab.hover == Some(Hit::BoardMenu),
        theme,
        m::BUTTON_RADIUS,
    );
    icon(scene, IconId::MoreHorizontal, l.board_menu, fg);
}

/// A estrela do cabeçalho: cheia (amarela) se o Kanban tem estrela; senão o contorno.
fn star(scene: &mut VectorScene, r: Rect, on: bool, theme: Theme) {
    let (cx, cy) = (f64::from(r.x + r.w / 2.0), f64::from(r.y + r.h / 2.0));
    let (outer, inner) = (f64::from(m::ICON_PX) / 2.0, f64::from(m::ICON_PX) / 4.6);
    let mut p = ph2d_vector::BezPath::new();
    for i in 0..10 {
        let a = std::f64::consts::PI * (f64::from(i) / 5.0 - 0.5);
        let rad = if i % 2 == 0 { outer } else { inner };
        let pt = (cx + rad * a.cos(), cy + rad * a.sin());
        if i == 0 {
            p.move_to(pt);
        } else {
            p.line_to(pt);
        }
    }
    p.close_path();
    if on {
        scene.fill_path(
            &p,
            &ph2d_vector::Brush::Solid(c(theme, T::KanbanStar)),
            Affine::IDENTITY,
        );
    } else {
        let stroke = ph2d_vector::Stroke::new(f64::from(StrokeToken::Default.px()));
        scene.inner_mut().stroke(
            &stroke,
            Affine::IDENTITY,
            c(theme, T::KanbanHeaderFg),
            None,
            &p,
        );
    }
}

/// Um cartão a ser escrito (o título no lugar, ou o compositor): a caixa do cartão com o texto, o
/// cursor e a selecção.
pub(crate) fn editing_card(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    f: &Field,
    r: Rect,
) {
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        c(theme, T::KanbanShadowNear),
        m::SHADOW_NEAR,
    );
    fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, T::KanbanCard));
    let inner = Rect::new(
        r.x + m::CARD_PAD_X,
        r.y + m::CARD_PAD_Y,
        r.w - 2.0 * m::CARD_PAD_X,
        r.h - 2.0 * m::CARD_PAD_Y,
    );
    text_with_caret(scene, ts, theme, f, inner, m::CARD_TITLE_PX, true);
    stroke_rounded_rect(scene, r, m::CARD_RADIUS, m::FOCUS_RING, c(theme, T::Accent));
}

/// Um campo de uma linha (o título do Kanban ou de uma lista, o nome da lista nova).
pub(crate) fn field_box(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    f: &Field,
    r: Rect,
    px: f32,
) {
    let r = Rect::new(r.x, r.y - m::FIELD_PAD_Y / 2.0, r.w, r.h + m::FIELD_PAD_Y);
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, c(theme, T::KanbanCard));
    stroke_rounded_rect(
        scene,
        r,
        m::FIELD_RADIUS,
        m::FOCUS_RING,
        c(theme, T::Accent),
    );
    let inner = Rect::new(r.x + m::FIELD_PAD_X, r.y, r.w - 2.0 * m::FIELD_PAD_X, r.h);
    text_with_caret(scene, ts, theme, f, inner, px, false);
}

/// O texto de um campo com o cursor (e a selecção pintada por baixo).
fn text_with_caret(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    f: &Field,
    r: Rect,
    px: f32,
    wrap: bool,
) {
    let fg = c(theme, T::KanbanCardFg);
    let top = if wrap {
        r.y + (m::LINE_H - ts.layout("Ag", px, f32::INFINITY).height()) / 2.0
    } else {
        text_top(ts, r.y, r.h, px)
    };
    // Uma linha: o cursor e a selecção medem o prefixo. Várias: a última linha leva o cursor.
    let caret_x = |ts: &mut TextSystem, at: usize| r.x + ts.prefix_width(&f.text[..at], px);
    if !wrap {
        if let Some((a, b)) = f.selection() {
            let (x0, x1) = (caret_x(ts, a), caret_x(ts, b));
            fill_rounded_rect(
                scene,
                Rect::new(
                    x0,
                    r.y + m::FIELD_PAD_Y / 2.0,
                    x1 - x0,
                    r.h - m::FIELD_PAD_Y,
                ),
                0.0,
                c(theme, T::Selection),
            );
        }
        paint_text_elided(ts, scene, &f.text, r.x, top, px, f32::INFINITY, fg);
        let x = caret_x(ts, f.caret);
        fill_rounded_rect(
            scene,
            Rect::new(
                x,
                r.y + m::FIELD_PAD_Y / 2.0,
                m::CARET_W,
                r.h - m::FIELD_PAD_Y,
            ),
            0.0,
            fg,
        );
        return;
    }
    if f.selection().is_some() && !f.text.is_empty() {
        let lines = crate::front::title_lines(ts, &f.text, r.w);
        #[allow(clippy::cast_precision_loss)]
        let h = lines as f32 * m::LINE_H;
        fill_rounded_rect(
            scene,
            Rect::new(r.x, r.y, r.w, h),
            0.0,
            c(theme, T::Selection),
        );
    }
    paint_text_block(ts, scene, &f.text, r.x, top, px, r.w, fg);
    let (line_start, line_y) = last_line(ts, &f.text[..f.caret], px, r.w);
    let x = r.x + ts.prefix_width(&f.text[line_start..f.caret], px);
    fill_rounded_rect(
        scene,
        Rect::new(x, r.y + line_y, m::CARET_W, m::LINE_H),
        0.0,
        fg,
    );
}

/// Onde começa a última linha de `prefix` quebrado a `width`, e o topo dela.
fn last_line(ts: &mut TextSystem, prefix: &str, px: f32, width: f32) -> (usize, f32) {
    if prefix.is_empty() {
        return (0, 0.0);
    }
    let layout = ts.layout(prefix, px, width);
    let n = layout.len();
    let start = layout.lines().last().map_or(0, |l| l.text_range().start);
    #[allow(clippy::cast_precision_loss)]
    (start, n.saturating_sub(1) as f32 * m::LINE_H)
}

/// Uma camada de sombra desfocada por baixo de `r`.
pub(crate) fn shadow(
    scene: &mut VectorScene,
    r: Rect,
    radius: f32,
    color: Color,
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

/// ⭐ O que voa: o cartão (ou a lista) agarrado, inclinado, com a sombra de voo, por cima de tudo.
fn flying(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &Tab,
    looks: &mut dyn FnMut(ph2d_kanban_model::CardId) -> Option<CardLook>,
) {
    let Some(d) = tab.drag else {
        return;
    };
    let mut r = Rect::new(d.at.0 - d.grab.0, d.at.1 - d.grab.1, d.size.0, d.size.1);
    let mut s = VectorScene::new();
    let mut local = Rect::new(0.0, 0.0, r.w, r.h);
    match d.lift {
        Lift::Card(id) => {
            let Some(look) = looks(id) else {
                return;
            };
            paint_card::flight_shadow(&mut s, theme, local, m::CARD_RADIUS);
            paint_card::card(
                &mut s,
                ts,
                theme,
                &Ink::of(theme),
                &look,
                local,
                tab.label_text,
            );
        }
        Lift::List(_) => {
            // A disposição VISTA da que voa (a mão e as molas dos cartões dela já estão lá).
            let Some(lb) = tab.layout.flying.as_ref() else {
                return;
            };
            r = lb.rect;
            local = Rect::new(0.0, 0.0, r.w, r.h);
            let moved = shifted(lb, -r.x, -r.y);
            paint_card::flight_shadow(&mut s, theme, local, m::LIST_RADIUS);
            let cards = if lb.collapsed {
                paint_list::collapsed(&mut s, ts, theme, k, tab, &moved);
                Vec::new()
            } else {
                paint_list::list(&mut s, ts, theme, k, tab, &moved, looks)
            };
            tab.painted.borrow_mut().flight = Some((lb.id, r, tab.springs.tilt.pos, cards));
        }
        Lift::Pan { .. } => return,
    }
    let pivot = (f64::from(r.w) / 2.0, f64::from(r.h) * m::FLIGHT_PIVOT_Y);
    let t = Affine::translate((f64::from(r.x), f64::from(r.y)))
        * Affine::translate(pivot)
        * Affine::rotate(tab.springs.tilt.pos.to_radians())
        * Affine::translate((-pivot.0, -pivot.1));
    scene.append_transformed(&s, t);
}

/// A lista `lb` deslocada (para pintar a que voa na origem).
fn shifted(lb: &ListBox, dx: f32, dy: f32) -> ListBox {
    let s = |r: Rect| Rect::new(r.x + dx, r.y + dy, r.w, r.h);
    ListBox {
        id: lb.id,
        rect: s(lb.rect),
        collapsed: lb.collapsed,
        title: s(lb.title),
        collapse: s(lb.collapse),
        menu: s(lb.menu),
        body: s(lb.body),
        content_h: lb.content_h,
        cards: lb
            .cards
            .iter()
            .map(|c| crate::layout::CardBox {
                id: c.id,
                rect: s(c.rect),
            })
            .collect(),
        slot: None,
        composer: None,
        add: s(lb.add),
        close: None,
        count: lb.count,
        limit: lb.limit.map(s),
    }
}

/// Um Kanban que já não existe (apagado noutra janela, ou desfeito): a aba diz isso.
pub(crate) fn missing(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, area: Rect) {
    scene.fill_rect(rect_to_vello(area), c(theme, T::Bg1));
    let y = text_top(ts, area.y, area.h, m::BUTTON_TEXT_PX);
    paint_text_elided(
        ts,
        scene,
        tr("kanban.missing"),
        area.x + m::HOME_PAD,
        y,
        m::BUTTON_TEXT_PX,
        area.w,
        c(theme, T::Text2),
    );
}
