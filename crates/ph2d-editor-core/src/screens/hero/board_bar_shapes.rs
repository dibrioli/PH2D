//! As duas grelhas das formas (módulo filho de [`super`]): a do «mais formas» da barra curta escolhe
//! a FERRAMENTA; a do botão «Forma» da barra de contexto troca o TIPO das formas seleccionadas (o
//! idioma do Miro). As duas pintam os mesmos `Item::Pick` — nunca abertas juntas.

use super::look::{paint_panel, shape_icon};
use super::{HeroScreen, Item, paint_item};
use crate::documents::BoardLive;
use crate::interaction::{HitIndex, WidgetStore};
use crate::zones::Rect;
use ph2d_board_edit::{Editor, Tool};
use ph2d_board_model::{BoardDoc, ShapeType};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// O botão «Forma» da barra de contexto de uma selecção de formas.
pub(super) const RESHAPE: Item = Item::Reshape;
/// Colunas da grelha «Forma» (por baixo de uma barra deitada, larga e baixa).
const RESHAPE_COLS: usize = 6;

/// As formas da selecção que a grelha «Forma» troca: nenhuma nota (a lei de `set_shape_type`).
fn shapes<'a>(ed: &Editor, doc: &'a BoardDoc) -> impl Iterator<Item = ShapeType> + 'a {
    let sel: Vec<_> = ed.selection().iter().copied().collect();
    sel.into_iter()
        .filter_map(|id| doc.get(id)?.shape().map(|s| s.kind))
}

/// O tipo «actual» (o da 1.ª forma seleccionada) — a cara do botão e a forma acesa na grelha.
pub(super) fn current(ed: &Editor, doc: &BoardDoc) -> Option<ShapeType> {
    shapes(ed, doc).next()
}

/// O botão aparece? Só sem notas na selecção (o Miro só troca nota quadrada ↔ larga, e isso é da
/// barra das notas). `None` = não é daqui.
pub(super) fn shows(it: Item, ed: &Editor, doc: &BoardDoc) -> Option<bool> {
    (it == RESHAPE).then(|| shapes(ed, doc).all(|t| !t.is_note()))
}

/// A grelha «Forma» abre para ESTA selecção e ferramenta: trocar uma delas, ou a barra deixar de
/// estar pintada (um arrasto), fecha-a — nunca fica um estado aberto por baixo de outra coisa.
pub(super) fn sync(live: &mut BoardLive, bar_shown: bool) {
    let (Some((sel, tool)), Some(ed)) = (&live.reshape, &live.editor) else {
        return;
    };
    if !bar_shown || *tool != ed.tool || sel != ed.selection() {
        live.reshape = None;
    }
}

/// Em TODO clique da barra: o «Forma» abre (e fecha o «mais formas»), o «mais formas» alterna, e
/// qualquer clique fecha a grelha «Forma». Devolve se ela ESTAVA aberta (o `Pick` é dela).
pub(super) fn toggle(live: &mut BoardLive, it: Item) -> bool {
    let was = live.reshape.take().is_some();
    if it == RESHAPE && !was {
        live.reshape = live
            .editor
            .as_ref()
            .map(|ed| (ed.selection().clone(), ed.tool));
        live.shapes_open = false;
    }
    if it == Item::MoreShapes {
        live.shapes_open = !live.shapes_open;
    }
    was
}

/// ⭐ Os cliques daqui. `reshaping` = a grelha «Forma» estava aberta: o `Pick` troca o tipo das
/// formas seleccionadas (um passo do desfazer); senão escolhe a ferramenta. `false` = não é daqui.
pub(super) fn apply(hero: &mut HeroScreen, it: Item, reshaping: bool) -> bool {
    match it {
        Item::MoreShapes | RESHAPE => true,
        Item::Pick(t) => {
            let theme = hero.theme;
            let Some((board, live)) = hero.documents.active_parts() else {
                return true;
            };
            let history = live.histories.entry(board.id).or_default();
            let ed = super::super::board_view::editor(&mut live.editor, theme);
            if reshaping {
                ed.set_shape_type(&mut board.doc, history, t);
            } else {
                ed.tool = Tool::Shape(t);
                live.shapes_open = false;
            }
            true
        }
        _ => false,
    }
}

/// A grelha do «mais formas», à direita da barra curta; acesa a forma da ferramenta.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_tool_grid(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    area: Rect,
    tool: Tool,
) {
    let grid = super::shapes_rects(area);
    paint_panel(scene, &grid, theme);
    for (it, r) in grid {
        let on = matches!(it, Item::Pick(t) if tool == Tool::Shape(t));
        paint_item(scene, text_system, theme, hit, store, it, r, on);
    }
}

/// O botão «Forma»: a cara é o contorno da forma actual.
pub(super) fn paint_button(
    scene: &mut VectorScene,
    hit: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    r: Rect,
    (on, now): (bool, Option<ShapeType>),
) {
    let fg = super::super::float_bar::paint_button(
        scene,
        hit,
        store,
        theme,
        super::super::float_bar::Btn {
            id: super::node(RESHAPE),
            rect: r,
            on,
            tint: ColorToken::Text2,
        },
    );
    let inner = super::super::float_bar::inner(r);
    let line = f64::from(StrokeToken::Default.px());
    let t = now.unwrap_or(ShapeType::Rectangle);
    shape_icon(scene, t, inner, fg, line, false);
}

/// A grelha «Forma», por baixo do botão (por cima, se não cabe), presa dentro de `area`.
#[must_use]
pub fn reshape_rects(area: Rect, button: Rect) -> Vec<(Item, Rect)> {
    let (s, g) = (super::btn_px(), super::gap());
    let n = ShapeType::ALL.len();
    let cols = RESHAPE_COLS.min(n);
    let rows = n.div_ceil(cols) as f32;
    let (w, h) = (cols as f32 * (s + g) - g, rows * (s + g) - g);
    let pad = Spacing::Sm.px();
    let below = button.y + button.h + pad;
    let y = if below + h <= area.y + area.h {
        below
    } else {
        (button.y - pad - h).max(area.y)
    };
    let x = button.x.min(area.x + area.w - w).max(area.x);
    ShapeType::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (c, r) = ((i % cols) as f32, (i / cols) as f32);
            (
                Item::Pick(*t),
                Rect::new(x + c * (s + g), y + r * (s + g), s, s),
            )
        })
        .collect()
}

/// Pinta a grelha «Forma» aberta, ancorada no botão da barra `bar`; acesa a forma actual.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_reshape_grid(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    area: Rect,
    bar: &[(Item, Rect)],
    now: Option<ShapeType>,
) {
    let Some((_, button)) = bar.iter().find(|(it, _)| *it == RESHAPE) else {
        return;
    };
    let grid = reshape_rects(area, *button);
    paint_panel(scene, &grid, theme);
    for (it, r) in grid {
        let on = matches!(it, Item::Pick(t) if Some(t) == now);
        paint_item(scene, text_system, theme, hit, store, it, r, on);
    }
}
