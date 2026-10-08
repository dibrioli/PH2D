//! ⭐ **A busca** (`Ctrl+F`, a do Miro: *«Found objects will be highlighted on the board, while
//! other content will be dimmed»*) — o texto das formas e notas, os rótulos das setas, os nomes dos
//! frames e as etiquetas; `Enter` vai ao seguinte; as etiquetas do quadro, por baixo do campo,
//! filtram (*«search specifically for stickies with a particular tag»*). E o campo de pôr uma
//! etiqueta nas notas seleccionadas (o botão da barra das notas).

use super::{GO_CONTEXT, HeroScreen};
use crate::interaction::WidgetEvent;
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardDoc, ElementId, Rgba};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill, Stroke, VectorScene};

/// A largura do campo em passos de `Spacing::Xl4`.
const FIELD_STEPS: f32 = 6.0; // LITERAL-PX-OK: proporção do campo

/// ⭐ **O que a busca acha**: `query` (sem distinguir maiúsculas) no texto de cada elemento, e só
/// os que têm a etiqueta `tag`, se há — por ordem de LEITURA (de cima para baixo, da esquerda para
/// a direita). Nem uma coisa nem outra: nada.
#[must_use]
pub fn find(doc: &BoardDoc, query: &str, tag: Option<&str>) -> Vec<ElementId> {
    let q = query.trim().to_lowercase();
    if q.is_empty() && tag.is_none() {
        return Vec::new();
    }
    let mut hits: Vec<(&ph2d_board_model::Element, [f64; 4])> = doc
        .live()
        .filter(|el| tag.is_none_or(|t| el.tags().iter().any(|x| x.name == t)))
        .filter(|el| {
            q.is_empty() || {
                let mut texts: Vec<&str> = el.tags().iter().map(|t| t.name.as_str()).collect();
                if let Some(s) = el.shape() {
                    texts.push(s.text.as_str());
                }
                if let Some(c) = el.connector() {
                    texts.push(c.label.as_str());
                }
                if let Some(f) = el.as_frame() {
                    texts.push(&f.name);
                }
                texts.iter().any(|t| t.to_lowercase().contains(&q))
            }
        })
        .map(|el| (el, el.aabb()))
        .collect();
    hits.sort_by(|a, b| a.1[1].total_cmp(&b.1[1]).then(a.1[0].total_cmp(&b.1[0])));
    hits.into_iter().map(|(el, _)| el.id).collect()
}

/// As etiquetas do quadro (nome e a cor da primeira que se achou), por nome.
#[must_use]
pub fn board_tags(doc: &BoardDoc) -> Vec<(String, Rgba)> {
    let mut m: std::collections::BTreeMap<String, Rgba> = std::collections::BTreeMap::new();
    for el in doc.live() {
        for t in el.tags() {
            m.entry(t.name.clone()).or_insert(t.color);
        }
    }
    m.into_iter().collect()
}

/// A cor de uma etiqueta NOVA: a de uma com o mesmo nome no quadro, senão a pastel seguinte.
fn tag_color(doc: &BoardDoc, name: &str) -> Rgba {
    let tags = board_tags(doc);
    if let Some((_, c)) = tags.iter().find(|(n, _)| n == name) {
        return *c;
    }
    let pastels = crate::widget::panel_chrome::HIGHLIGHTER_RGBA;
    Rgba(pastels[tags.len() % 5])
}

/// O id da fita `i` das etiquetas do quadro (na busca).
#[must_use]
pub fn chip_node(i: usize) -> NodeId {
    NodeId(super::SALT_TAG_CHIP ^ (i as u64 + 1))
}

fn chip_of(id: NodeId) -> Option<usize> {
    let v = id.0 ^ super::SALT_TAG_CHIP;
    (v >= 1 && v < 1 << 16).then(|| v as usize - 1)
}

pub(super) fn open(hero: &mut HeroScreen) {
    let query = super::field_text(&hero.store, crate::ids::BOARD_SEARCH_INPUT);
    hero.documents.live.nav.search_open = true;
    super::open_field(&mut hero.store, crate::ids::BOARD_SEARCH_INPUT, query);
    refresh(hero);
}

pub(super) fn close(hero: &mut HeroScreen) {
    let n = &mut hero.documents.live.nav;
    n.search_open = false;
    n.results.clear();
    n.tag_filter = None;
    if hero.store.focus_id() == Some(crate::ids::BOARD_SEARCH_INPUT) {
        hero.store.set_focus(None);
    }
}

/// O campo de pôr uma etiqueta nas notas seleccionadas.
pub fn open_tag_field(hero: &mut HeroScreen) {
    hero.documents.live.nav.tagging = true;
    super::open_field(&mut hero.store, crate::ids::BOARD_TAG_INPUT, String::new());
}

fn refresh(hero: &mut HeroScreen) {
    let query = super::field_text(&hero.store, crate::ids::BOARD_SEARCH_INPUT);
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let n = &mut live.nav;
    n.results = find(&board.doc, &query, n.tag_filter.as_deref());
    n.current = 0;
}

/// Vai ao resultado `current` e avança para o seguinte.
fn go_next(hero: &mut HeroScreen) {
    let area = super::super::board_view::area_of(hero.last_canvas);
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let n = &mut live.nav;
    if n.results.is_empty() {
        return;
    }
    let id = n.results[n.current % n.results.len()];
    n.current = (n.current + 1) % n.results.len();
    super::go_to(board, area, id, GO_CONTEXT);
}

pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (search, tag) = (crate::ids::BOARD_SEARCH_INPUT, crate::ids::BOARD_TAG_INPUT);
    match event {
        WidgetEvent::TextChanged(id) if id == search => refresh(hero),
        WidgetEvent::Submit(id) if id == search => {
            go_next(hero);
            hero.store.set_focus(Some(search));
        }
        WidgetEvent::Cancel(id) if id == search => close(hero),
        WidgetEvent::Blur(id) if id == search => {}
        WidgetEvent::Submit(id) if id == tag => {
            let name = super::field_text(&hero.store, tag);
            hero.documents.live.nav.tagging = false;
            let theme = hero.theme;
            if let Some((board, live)) = hero.documents.active_parts() {
                let color = tag_color(&board.doc, name.trim());
                let history = live.histories.entry(board.id).or_default();
                let ed = super::super::board_view::editor(&mut live.editor, theme);
                ed.toggle_tag(&mut board.doc, history, &name, color);
            }
        }
        WidgetEvent::Cancel(id) | WidgetEvent::Blur(id) if id == tag => {
            hero.documents.live.nav.tagging = false;
        }
        WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id) if chip_of(id).is_some() => {
            let i = chip_of(id).expect("visto acima");
            let tags = hero
                .documents
                .active_board()
                .map(|b| board_tags(&b.doc))
                .unwrap_or_default();
            if let Some((name, _)) = tags.get(i) {
                let n = &mut hero.documents.live.nav;
                n.tag_filter = if n.tag_filter.as_deref() == Some(name) {
                    None
                } else {
                    Some(name.clone())
                };
                refresh(hero);
            }
        }
        _ => return false,
    }
    true
}

fn field_rect(area: Rect, row: f32) -> Rect {
    let w = (Spacing::Xl4.px() * FIELD_STEPS).min(area.w - 2.0 * Spacing::Md.px());
    let h = Spacing::Xl2.px();
    Rect::new(
        area.x + (area.w - w) / 2.0,
        area.y + Spacing::Md.px() + row * (h + Spacing::Xs.px()),
        w,
        h,
    )
}

pub(super) fn paint_fields(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let (open, tagging) = {
        let n = &hero.documents.live.nav;
        (n.search_open, n.tagging)
    };
    let font = TypeToken::Sm.px();
    let mut row = 0.0;
    if open {
        let r = field_rect(area, row);
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        super::paint_field(
            crate::ids::BOARD_SEARCH_INPUT,
            r,
            scene,
            ts,
            theme,
            hit,
            store,
        );
        let n = &hero.documents.live.nav;
        let count = ph2d_i18n::tr_with("board.nav.search_count", &[("n", &n.results.len())]);
        let pad = Spacing::Md.px();
        paint_text(
            ts,
            scene,
            &count,
            r.x + r.w + pad,
            r.y + (r.h - font) / 2.0,
            font,
            area.w,
            resolve(ColorToken::Text2, theme),
        );
        row += 1.0;
        let tags = hero
            .documents
            .active_board()
            .map(|b| board_tags(&b.doc))
            .unwrap_or_default();
        let filter = hero.documents.live.nav.tag_filter.clone();
        let line = field_rect(area, row);
        let mut x = line.x;
        for (i, (name, color)) in tags.iter().enumerate() {
            let w = crate::paint::rect_for_label(ts.prefix_width(name, font));
            if x + w > line.x + line.w {
                break;
            }
            let chip = Rect::new(x, line.y, w, line.h);
            let radius = crate::paint::frame_radius(theme, Radius::Lg.px());
            let Rgba([r8, g8, b8, a8]) = *color;
            let c = ph2d_vector::Color::from_rgba8(r8, g8, b8, a8); // LITERAL-COLOR-OK: cor da etiqueta (documento)
            fill_rounded_rect(scene, chip, radius, c);
            if filter.as_deref() == Some(name.as_str()) {
                let edge = Stroke::new(f64::from(StrokeToken::Thick.px()));
                let accent = Brush::Solid(resolve(ColorToken::Accent, theme));
                let r = crate::paint::rect_to_vello(chip);
                scene
                    .inner_mut()
                    .stroke(&edge, Affine::IDENTITY, &accent, None, &r);
            }
            let ink = color.readable_ink();
            let ink = ph2d_vector::Color::from_rgba8(ink.0[0], ink.0[1], ink.0[2], ink.0[3]); // LITERAL-COLOR-OK: tinta do documento
            crate::paint::paint_text_centered(ts, scene, name, chip, font, ink);
            super::register(&mut hero.store, chip_node(i));
            hero.hit_index.register(chip_node(i), chip);
            x += w + Spacing::Xs.px();
        }
        if !tags.is_empty() {
            row += 1.0;
        }
    }
    if tagging {
        let r = field_rect(area, row);
        let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
        let label = ph2d_i18n::tr("board.nav.tag_prompt");
        let lw = crate::paint::rect_for_label(ts.prefix_width(label, font));
        let lr = Rect::new(r.x - lw, r.y, lw, r.h);
        fill_rounded_rect(scene, lr, radius, resolve(ColorToken::BgElev, theme));
        crate::paint::paint_text_centered(
            ts,
            scene,
            label,
            lr,
            font,
            resolve(ColorToken::Text2, theme),
        );
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        super::paint_field(crate::ids::BOARD_TAG_INPUT, r, scene, ts, theme, hit, store);
    }
}

/// ⭐ O realce: o quadro escurece por um véu com BURACOS nos encontrados (regra par-ímpar) — o
/// encontrado fica com a cor dele, o resto apaga-se; o que está à vista leva o contorno de destaque.
pub(super) fn paint_highlight(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let Some(board) = hero.documents.active_board() else {
        return;
    };
    let n = &hero.documents.live.nav;
    let query = super::field_text(&hero.store, crate::ids::BOARD_SEARCH_INPUT);
    if !n.search_open || (query.trim().is_empty() && n.tag_filter.is_none()) {
        return;
    }
    let a = super::super::board_view::area_of(area);
    let screen = |id: &ElementId| {
        board.doc.get(*id).map(|el| {
            let [x0, y0, x1, y1] = el.aabb();
            let p = board.camera.to_screen(a, [x0, y0]);
            let q = board.camera.to_screen(a, [x1, y1]);
            ph2d_vector::Rect::new(p[0], p[1], q[0], q[1])
        })
    };
    let mut veil = BezPath::new();
    let whole = crate::paint::rect_to_vello(area);
    veil.extend(ph2d_vector::Shape::path_elements(&whole, 0.1)); // LITERAL-PX-OK: tolerância de achatar o rectângulo
    for r in n.results.iter().filter_map(screen) {
        veil.extend(ph2d_vector::Shape::path_elements(&r, 0.1)); // LITERAL-PX-OK: tolerância de achatar o rectângulo
    }
    scene.push_clip_with_rule(&veil, Fill::EvenOdd);
    scene.fill_rect(whole, resolve(ColorToken::BgScrim, theme));
    scene.pop_layer();
    let shown = (n.current + n.results.len().saturating_sub(1)) % n.results.len().max(1);
    if let Some(r) = n.results.get(shown).and_then(screen) {
        let edge = Stroke::new(f64::from(StrokeToken::Thick.px()));
        let accent = Brush::Solid(resolve(ColorToken::Accent, theme));
        scene
            .inner_mut()
            .stroke(&edge, Affine::IDENTITY, &accent, None, &r);
    }
}
