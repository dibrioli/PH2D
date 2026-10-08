//! As barras do quadro para PLANEAR (W6, módulo filho de [`super`]): na barra curta o botão do Mapa
//! mental e o dos Modelos (a galeria abre-se ao lado dele, como a das formas); na barra de contexto
//! de um nó de mapa, recolher/expandir, o auto layout e a direcção do mapa.
//!
//! ⚠️ Um controlo que não faria nada ao nó de agora NÃO se pinta (recolher ou o auto layout de uma
//! folha): um botão sem efeito é um controlo morto.

use super::{Item, Selected};
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{paint_icon as icon, paint_text_centered};
use crate::zones::Rect;
use ph2d_board_edit::{Command, Editor, Template, Tool};
use ph2d_board_model::{BoardDoc, Dash, History};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme, TypeToken};
use ph2d_vector::{BezPath, Color, Shape as _, VectorScene};

/// Um controlo de planear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MindItem {
    /// Abre ou fecha a galeria dos modelos.
    Templates,
    /// O modelo `i` de [`Template::ALL`] (na galeria).
    Template(usize),
    /// Recolher ou expandir o ramo do nó.
    Collapse,
    /// O auto layout dos filhos do nó.
    Auto,
    /// A direcção do mapa (para os lados ↔ para cima e para baixo).
    Vertical,
}

/// O botão do Mapa mental na barra curta.
pub(super) const MIND_TOOL: Item = Item::Tool(Tool::MindMap);
/// O botão dos Modelos na barra curta.
pub(super) const TEMPLATES: Item = Item::Mind(MindItem::Templates);

/// A largura de uma linha da galeria, em botões da barra: o nome mais longo — «Customer journey
/// map» — cabe a `Sm` (a 6 lia-se «Customer journey…» na foto da cena 8).
const GALLERY_W: f32 = 8.0;

/// Os itens da galeria (registados na tabela da barra).
pub(super) fn flyout_items() -> Vec<Item> {
    (0..Template::ALL.len())
        .map(|i| Item::Mind(MindItem::Template(i)))
        .collect()
}

/// O grupo da barra de contexto de um nó de mapa.
pub(super) fn groups() -> Vec<Vec<Item>> {
    vec![vec![
        Item::Mind(MindItem::Collapse),
        Item::Mind(MindItem::Auto),
        Item::Mind(MindItem::Vertical),
    ]]
}

/// O que está seleccionado é um nó de mapa (só um, e não a escrever).
pub(super) fn selected_kind(ed: &Editor, doc: &BoardDoc) -> Option<Selected> {
    (!ed.is_editing_text() && ed.mind_focus(doc).is_some()).then_some(Selected::Mind)
}

/// O controlo faz alguma coisa ao nó de agora? `None` = não é daqui.
pub(super) fn shows(it: Item, ed: &Editor, doc: &BoardDoc) -> Option<bool> {
    let Item::Mind(m) = it else {
        return None;
    };
    let id = ed.mind_focus(doc);
    let leaf = id.is_none_or(|id| doc.mind_children(id).is_empty());
    Some(match m {
        MindItem::Collapse | MindItem::Auto => !leaf,
        _ => true,
    })
}

/// A chave i18n da dica.
pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    Some(match it {
        MIND_TOOL => "board.tool.mindmap",
        TEMPLATES => "board.tool.templates",
        Item::Mind(MindItem::Template(i)) => Template::ALL.get(i)?.key(),
        Item::Mind(MindItem::Collapse) => "board.mind.collapse",
        Item::Mind(MindItem::Auto) => "board.mind.auto",
        Item::Mind(MindItem::Vertical) => "board.mind.vertical",
        _ => return None,
    })
}

/// Aceso? `None` = não é daqui.
pub(super) fn is_on(it: Item, ed: &Editor, doc: &BoardDoc, open: bool) -> Option<bool> {
    let node = ed
        .mind_focus(doc)
        .and_then(|id| doc.get(id)?.mind().cloned());
    let root = ed
        .mind_focus(doc)
        .and_then(|id| doc.get(doc.mind_root(id)?)?.mind().cloned());
    Some(match it {
        MIND_TOOL => ed.tool == Tool::MindMap,
        TEMPLATES => open,
        Item::Mind(MindItem::Collapse) => node.is_some_and(|m| m.collapsed),
        Item::Mind(MindItem::Auto) => node.is_some_and(|m| !m.manual),
        Item::Mind(MindItem::Vertical) => root.is_some_and(|m| m.vertical),
        Item::Mind(MindItem::Template(_)) => false,
        _ => return None,
    })
}

/// Os rectângulos da galeria: uma coluna de nomes à direita do botão dos Modelos.
pub(super) fn gallery_rects(area: Rect) -> Vec<(Item, Rect)> {
    let Some(btn) = super::toolbar_rects(area)
        .into_iter()
        .find(|(it, _)| *it == TEMPLATES)
        .map(|(_, r)| r)
    else {
        return Vec::new();
    };
    let (s, g) = (super::btn_px(), super::gap());
    let x = btn.x + btn.w + Spacing::Sm.px();
    flyout_items()
        .into_iter()
        .enumerate()
        .map(|(i, it)| {
            (
                it,
                Rect::new(x, btn.y + i as f32 * (s + g), s * GALLERY_W, s),
            )
        })
        .collect()
}

/// Pinta a galeria aberta.
pub(super) fn paint_gallery(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    area: Rect,
) {
    let rects = gallery_rects(area);
    super::paint_panel(scene, &rects, theme);
    for (it, r) in rects {
        super::paint_item(scene, text_system, theme, hit_index, store, it, r, false);
    }
}

/// O desenho de um controlo daqui. `false` = não é daqui.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_icon(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    it: Item,
    r: Rect,
    inner: Rect,
    fg: Color,
    line: f64,
    collapsed: bool,
) -> bool {
    let w = line as f32;
    match it {
        MIND_TOOL => mind_glyph(scene, inner, fg, line),
        TEMPLATES => icon(scene, IconId::Layers, inner, fg, w),
        Item::Mind(MindItem::Template(i)) => {
            let Some(t) = Template::ALL.get(i) else {
                return false;
            };
            paint_text_centered(text_system, scene, tr(t.key()), r, TypeToken::Sm.px(), fg);
        }
        Item::Mind(MindItem::Collapse) => {
            let id = if collapsed {
                IconId::Plus
            } else {
                IconId::Minus
            };
            icon(scene, id, inner, fg, w);
        }
        Item::Mind(MindItem::Auto) => icon(scene, IconId::Hierarchy, inner, fg, w),
        Item::Mind(MindItem::Vertical) => icon(scene, IconId::SplitVertical, inner, fg, w),
        _ => return false,
    }
    true
}

/// O ícone do Mapa mental: um nó com três ramos curvos para a direita.
fn mind_glyph(scene: &mut VectorScene, r: Rect, fg: Color, line: f64) {
    let (x, y, w, h) = (
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.w),
        f64::from(r.h),
    );
    let (cx, cy) = (x + w * 0.25, y + h * 0.5);
    let mut p = BezPath::new();
    p.extend(ph2d_vector::Circle::new((cx, cy), w * 0.12).path_elements(0.1));
    for t in [0.15, 0.5, 0.85] {
        let ty = y + h * t;
        p.move_to((cx + w * 0.12, cy));
        p.curve_to((x + w * 0.55, cy), (x + w * 0.5, ty), (x + w, ty));
    }
    super::look::stroke(scene, &p, fg, line, Dash::Solid);
}

/// O clique num controlo daqui. `false` = não é daqui.
pub(super) fn apply(
    ed: &mut Editor,
    doc: &mut BoardDoc,
    history: &mut History,
    it: Item,
    center: [f64; 2],
) -> bool {
    let Item::Mind(m) = it else {
        return false;
    };
    match m {
        MindItem::Templates => return false,
        MindItem::Template(i) => {
            if let Some(t) = Template::ALL.get(i) {
                ed.insert_template(doc, history, *t, center, &|k| tr(k).to_owned());
            }
        }
        MindItem::Collapse => {
            ed.organize(doc, history, Command::MindCollapse);
        }
        MindItem::Auto => {
            ed.organize(doc, history, Command::MindAuto);
        }
        MindItem::Vertical => {
            ed.organize(doc, history, Command::MindVertical);
        }
    }
    true
}
