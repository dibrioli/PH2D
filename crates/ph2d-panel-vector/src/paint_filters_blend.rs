//! **A lei de MISTURA de um degrau de filtro** — irmão do `paint_filters` pelo teto de 600 LOC do
//! painel.
//!
//! O corte é por RESPONSABILIDADE: aqui vive *como um degrau se mistura com o que está embaixo* (o
//! chip de modo + o popover das opções), enquanto o irmão fica com *as rows do próprio degrau* e o
//! `paint_filters_ramp` com *o trilho da rampa*. As três perguntas são distintas, e o teto de LOC só
//! forçou a que já era a linha de corte natural.

use super::*;
use crate::ids;
use crate::state::filters as fst;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;

/// **O popover das vinte leis de mistura** de um degrau — pintado no passe DIFERIDO de `paint.rs`,
/// POR CIMA de todas as seções. Espelho exato do `paint_markers::paint_marker_popover`.
///
/// ⚠️ As opções saem da tabela PUBLICADA (`blend_names`), nunca de uma lista escrita aqui: o painel
/// não alcança o `BlendMode`, e uma cópia derivaria dele na primeira lei nova — com o rótulo a
/// nomear outra coisa.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, row: usize, chip: Rect, theme: Theme) {
    let id = ids::filter_blend_id(row);
    let names = fst::blend_names();
    if names.is_empty() {
        return;
    }
    let sel = fst::stack()
        .get(row)
        .map_or(0, |fx| usize::from(fx.blend))
        .min(names.len() - 1);
    let options: Vec<DropdownOption<usize>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| DropdownOption::new(ids::filter_blend_option_id(row, i), i, *n))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let region = ctx.layout.popover_region();
    ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
}
