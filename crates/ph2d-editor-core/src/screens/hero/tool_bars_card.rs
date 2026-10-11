//! ⭐ **O CARTÃO lateral** (F10, pedido do dono de 11/10: *«se algum parâmetro não pode ser
//! colocado no gizmo então precisamos criar um modal que aparece na lateral»*) — os campos que
//! nenhuma alça move ([`crate::interaction::ToolBars::card`]), num painel flutuante JUNTO da
//! seleção, enquanto as alças estão à vista.
//!
//! ⚠️ Os ids são os da fila ② (um comando, um id): o `HitIndex` aceita o mesmo id em dois rects e
//! o clique segue pela porta do painel dono. A geometria é UMA conta ([`placed_card`]): o pintor e
//! os gates perguntam-na.

use super::{paint_panel, panel_of};
use crate::interaction::{HitIndex, ToolBars, WidgetStore};
use crate::widget::{
    RailAxis, RailButtonSize, ToolRail, ToolRailEntry, entry_advance, entry_rects,
    horizontal_lines, line_pitch, paint_tool_rail_axis, size_row_pulldowns,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// O id (a11y) do rail do cartão — irmão dos rails `203..205` da fila e dos flyouts dela.
const CARD_RAIL: NodeId = NodeId(206);

/// A folga entre a seleção (ou um obstáculo) e o painel do cartão.
#[must_use]
pub fn card_gap() -> f32 {
    Spacing::Sm.px()
}

/// O recuo do rail dentro do painel.
#[must_use]
pub fn card_pad() -> f32 {
    Spacing::Sm.px()
}

/// O cartão colocado: o rail medido, o painel (o fundo) e o rect em que os chips correm.
#[derive(Clone, Debug)]
pub struct PlacedCard {
    pub rail: ToolRail,
    pub panel: Rect,
    pub content: Rect,
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// ⭐ **Onde cai o painel do cartão**, de tamanho `size` (`[w, h]`), junto de `sel` (a seleção no
/// ecrã) dentro de `area`, fora de todo rect de `avoid` (os blocos das outras barras).
///
/// A regra: à DIREITA da seleção (folga [`card_gap`]), alinhado ao TOPO dela e preso na área; se
/// não cabe à direita, espelha para a ESQUERDA. Se o lugar cobre um obstáculo, desce para baixo
/// dele (ou sobe para cima), o mais perto do topo da seleção; sem lugar ao lado da seleção, vai ao
/// lado do obstáculo mais perto. Sem lugar livre, fica no primeiro candidato preso à área.
#[must_use]
pub fn card_rect(area: Rect, sel: Rect, size: [f32; 2], avoid: &[Rect]) -> Rect {
    let [w, h] = size;
    let clamp_y = |y: f32| y.min(area.y + area.h - h).max(area.y);
    let clamp_x = |x: f32| x.min(area.x + area.w - w).max(area.x);
    let fits = |x: f32| x >= area.x && x + w <= area.x + area.w;
    let mut xs: Vec<f32> = [sel.x + sel.w + card_gap(), sel.x - card_gap() - w]
        .into_iter()
        .filter(|&x| fits(x))
        .collect();
    if xs.is_empty() {
        xs.push(clamp_x(sel.x + sel.w + card_gap()));
    }
    let y0 = clamp_y(sel.y);
    let free = |r: Rect| !overlaps(sel, r) && !avoid.iter().any(|a| overlaps(*a, r));
    let near =
        |v: &mut Vec<f32>, at: f32| v.sort_by(|a, b| (a - at).abs().total_cmp(&(b - at).abs()));
    let mut ys: Vec<f32> = std::iter::once(y0)
        .chain(avoid.iter().flat_map(|a| {
            [
                clamp_y(a.y + a.h + card_gap()),
                clamp_y(a.y - card_gap() - h),
            ]
        }))
        .collect();
    near(&mut ys, y0);
    // Depois dos lados da seleção, os lados de cada obstáculo (o mais perto da seleção primeiro).
    let mut beside: Vec<f32> = avoid
        .iter()
        .flat_map(|a| [a.x + a.w + card_gap(), a.x - card_gap() - w])
        .filter(|&x| fits(x))
        .collect();
    near(&mut beside, xs[0]);
    for &x in xs.iter().chain(&beside) {
        for &y in &ys {
            let r = Rect::new(x, y, w, h);
            if free(r) {
                return r;
            }
        }
    }
    Rect::new(xs[0], y0, w, h)
}

/// O rail do cartão, com os pulldowns MEDIDOS (a largura de uma escolha é a da face mais larga).
/// `None` ⇒ sem cartão.
#[must_use]
pub fn card_rail(bars: &ToolBars, text_system: &mut TextSystem) -> Option<ToolRail> {
    let entries: Vec<ToolRailEntry> = bars
        .card
        .iter()
        .filter(|e| !matches!(e, ToolRailEntry::Divider))
        .cloned()
        .collect();
    if entries.is_empty() {
        return None;
    }
    let mut rail = ToolRail::new(CARD_RAIL, tr("chrome.flyout.shape_options"), entries);
    size_row_pulldowns(&mut rail, text_system);
    Some(rail)
}

/// O tamanho do rect em que os chips correm: a largura da entrada mais larga (⇒ um campo por
/// linha, com o rótulo por cima) e a altura das linhas que a porta da geometria quebra.
#[must_use]
pub fn card_content_size(rail: &ToolRail, size: RailButtonSize) -> [f32; 2] {
    let chip = size.chip_px();
    let w = rail
        .entries
        .iter()
        .map(|e| entry_advance(e, chip, RailAxis::Horizontal))
        .fold(0.0_f32, f32::max);
    let lines = horizontal_lines(rail, w, size) as f32;
    [w, lines * line_pitch(chip) - Spacing::Xs.px()]
}

/// ⭐ **A porta única do cartão**: o rail, o painel e o rect dos chips neste quadro — `None` sem
/// campos ou sem seleção no ecrã. `avoid` são os painéis das outras barras do módulo (os blocos
/// do [`super::tool_bars::placed`]).
#[must_use]
pub fn placed_card(
    bars: &ToolBars,
    area: Rect,
    size: RailButtonSize,
    avoid: &[Rect],
    text_system: &mut TextSystem,
) -> Option<PlacedCard> {
    let sel = bars.selection?;
    let rail = card_rail(bars, text_system)?;
    let [cw, ch] = card_content_size(&rail, size);
    let pad = card_pad();
    let panel = card_rect(area, sel, [cw + 2.0 * pad, ch + 2.0 * pad], avoid);
    let content = Rect::new(panel.x + pad, panel.y + pad, cw, ch);
    Some(PlacedCard {
        rail,
        panel,
        content,
    })
}

/// Os painéis dos blocos (a coluna, o flyout, a ④) — os obstáculos do cartão.
#[must_use]
pub fn obstacles(blocks: &[std::ops::Range<usize>], all: &[Rect]) -> Vec<Rect> {
    blocks
        .iter()
        .filter(|b| b.start < b.end && b.end <= all.len())
        .filter_map(|b| panel_of(all[b.clone()].iter().copied()))
        .collect()
}

/// Pinta o cartão e regista os ids dele (presos à área, na tinta e no dedo).
#[allow(clippy::too_many_arguments)] // o relógio é o 8º, como na fila
pub fn paint_card(
    card: &PlacedCard,
    area: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
) {
    let clip = ph2d_vector::Rect::new(
        f64::from(area.x),
        f64::from(area.y),
        f64::from(area.x + area.w),
        f64::from(area.y + area.h),
    );
    scene.push_clip(&clip);
    hit_index.push_clip(area);
    // `panel_of` acrescenta o seu respiro: o rect dado já é o painel, então entra recuado.
    let k = Spacing::Xs.px();
    let p = card.panel;
    paint_panel(
        scene,
        [Rect::new(p.x + k, p.y + k, p.w - 2.0 * k, p.h - 2.0 * k)],
        theme,
    );
    // O fundo engole o clique entre os campos (a forma por baixo não se desselecciona).
    hit_index.register(crate::ids::RAIL_BACKDROP, card.panel);
    paint_tool_rail_axis(
        &card.rail,
        card.content,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        RailAxis::Horizontal,
    );
    let size = store.rail_button_size();
    for slot in entry_rects(&card.rail, card.content, size, RailAxis::Horizontal) {
        if let Some(id) = slot.id {
            hit_index.register(id, slot.rect);
        }
    }
    scene.pop_layer();
    hit_index.pop_clip();
}

#[cfg(test)]
#[path = "tool_bars_card_tests.rs"]
mod tests;
