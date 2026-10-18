//! W23 B5 — TODAS as pontas de uma seta (módulo filho de [`super`]): a barra estreita oferece as cinco
//! do fluxograma; o «…» de cada grupo de pontas abre a grelha das quinze do catálogo para essa ponta
//! (as de UML e as de cardinalidade incluídas) — o Excalidraw e o Miro abrem as pontas numa janela.

use super::{HEADS, Item, paint_item};
use crate::documents::BoardLive;
use crate::interaction::{HitIndex, WidgetStore};
use crate::zones::Rect;
use ph2d_board_model::Head;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// Colunas da grelha das pontas.
const COLS: usize = 5;

/// O «…» do grupo da ponta `i` (`0` início, `1` fim).
pub(super) const fn more(i: usize) -> Item {
    Item::MoreHeads(i)
}

/// Os itens que só a grelha pinta (os dois «…» e as pontas fora das cinco da barra), para a tabela.
pub(super) fn extra_items() -> Vec<Item> {
    let mut v = vec![more(0), more(1)];
    for i in 0..2 {
        v.extend(
            Head::ALL
                .iter()
                .filter(|h| !HEADS.contains(h))
                .map(|h| Item::Head(i, *h)),
        );
    }
    v
}

/// A grelha abre para ESTA selecção: trocá-la, ou a barra deixar de estar pintada, fecha-a.
pub(super) fn sync(live: &mut BoardLive, bar_shown: bool) {
    let (Some((sel, _)), Some(ed)) = (&live.heads, &live.editor) else {
        return;
    };
    if !bar_shown || sel != ed.selection() {
        live.heads = None;
    }
}

/// Em TODO clique da barra: o «…» de uma ponta abre a grelha dela (fecha a outra e a das formas);
/// qualquer outro clique fecha-a.
pub(super) fn toggle(live: &mut BoardLive, it: Item) {
    let was = live.heads.take().map(|(_, i)| i);
    if let Item::MoreHeads(i) = it
        && was != Some(i)
    {
        live.heads = live.editor.as_ref().map(|ed| (ed.selection().clone(), i));
        live.reshape = None;
        live.shapes_open = false;
    }
}

/// A grelha das pontas `i`, por baixo do «…» (por cima, se não cabe), presa dentro de `area`.
#[must_use]
pub fn heads_rects(area: Rect, button: Rect, i: usize) -> Vec<(Item, Rect)> {
    let (s, g) = (super::btn_px(), super::gap());
    let n = Head::ALL.len();
    let rows = n.div_ceil(COLS) as f32;
    let (w, h) = (COLS as f32 * (s + g) - g, rows * (s + g) - g);
    let pad = Spacing::Sm.px();
    let below = button.y + button.h + pad;
    let y = if below + h <= area.y + area.h {
        below
    } else {
        (button.y - pad - h).max(area.y)
    };
    let x = button.x.min(area.x + area.w - w).max(area.x);
    Head::ALL
        .iter()
        .enumerate()
        .map(|(k, h)| {
            let (c, r) = ((k % COLS) as f32, (k / COLS) as f32);
            (
                Item::Head(i, *h),
                Rect::new(x + c * (s + g), y + r * (s + g), s, s),
            )
        })
        .collect()
}

/// Pinta a grelha aberta da ponta `i`, ancorada no «…» dela na barra `bar`; acesa a ponta actual.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_grid(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    area: Rect,
    bar: &[(Item, Rect)],
    (i, now): (usize, Option<Head>),
) {
    let Some((_, button)) = bar.iter().find(|(it, _)| *it == more(i)) else {
        return;
    };
    let grid = heads_rects(area, *button, i);
    super::look::paint_panel(scene, &grid, theme);
    for (it, r) in grid {
        let on = matches!(it, Item::Head(_, h) if Some(h) == now);
        paint_item(scene, text_system, theme, hit, store, it, r, on);
    }
}
