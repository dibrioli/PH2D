//! **As linhas da PONTA** (L9 · L18 · L19, `docs/Painter/Pinceis/03_MOTORES.md` §4): a ponta de píxel na secção
//! Shape (só no Digital) e a lei do grão na secção Grain (com um Grain armado). Uma tabela no tool
//! (`ph2d_tool_painter::ponta`): as linhas com opções são MENUS, as outras campos numéricos.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::{BrushSettings, CartaoDaPonta, LINHAS_DA_PONTA, ids};
use std::cell::Cell;

thread_local! {
    /// O menu de uma linha da ponta aberto neste quadro: o chip, a linha e a opção de agora.
    static PENDING_PONTA_DD: Cell<Option<(Rect, usize, u8)>> = const { Cell::new(None) };
}

/// O rótulo de cada linha da ponta, pela ordem de [`ph2d_tool_painter::LINHAS_DA_PONTA`].
pub(crate) const ROTULOS_DA_PONTA: [&str; ph2d_tool_painter::N_LINHAS_DA_PONTA] = [
    "panel.painter_ponta.pixel",
    "panel.painter_ponta.grain_mode",
    "panel.painter_ponta.depth_min",
    "panel.painter_ponta.depth_jitter",
    "panel.painter_ponta.depth_control",
];

/// Os índices das linhas da ponta que se pintam no cartão `c`.
pub(crate) fn linhas_do_cartao(c: CartaoDaPonta) -> impl Iterator<Item = usize> {
    LINHAS_DA_PONTA
        .into_iter()
        .enumerate()
        .filter(move |(_, l)| l.cartao() == c)
        .map(|(i, _)| i)
}

/// As linhas que são MENUS (as do modo).
pub(crate) fn e_menu(i: usize) -> bool {
    !LINHAS_DA_PONTA[i].opcoes().is_empty()
}

/// Pinta as linhas da ponta do cartão `c`; devolve o `y` seguinte.
pub(crate) fn paint_linhas_da_ponta(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    mut y: f32,
    brush: &BrushSettings,
    c: CartaoDaPonta,
) -> f32 {
    let sec = crate::seccoes::seccao_da_chave(ctx.text_system, ROTULOS_DA_PONTA[0]);
    for i in linhas_do_cartao(c) {
        let l = LINHAS_DA_PONTA[i];
        let id = ids::PAINTER_PONTA_FIELDS[i];
        let opcoes = l.opcoes();
        if !opcoes.is_empty() {
            let cur = (brush.ponta[i].round().max(0.0) as u8).min((opcoes.len() - 1) as u8);
            let (ny, open) = crate::paint_brush_rows::paint_dropdown_row(
                ctx,
                theme,
                x,
                w,
                y,
                ROTULOS_DA_PONTA[i],
                id,
                cur,
                tr(opcoes[usize::from(cur)]),
            );
            if let Some(r) = open {
                PENDING_PONTA_DD.with(|p| p.set(Some((r, i, cur))));
            }
            y = ny;
            continue;
        }
        let (lo, hi, passo) = l.faixa();
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            w,
            y,
            tr(ROTULOS_DA_PONTA[i]),
            id,
            brush.ponta[i] as f32,
            lo as f32,
            hi as f32,
            passo,
            l.casas(),
            sec,
        );
    }
    y
}

/// O menu aberto de uma linha da ponta — pintado por cima do corpo, depois do clip (como os outros).
pub(crate) fn paint_ponta_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    let Some((r, i, cur)) = PENDING_PONTA_DD.with(Cell::take) else {
        return;
    };
    let opcoes = LINHAS_DA_PONTA[i]
        .opcoes()
        .iter()
        .enumerate()
        .map(|(k, chave)| DropdownOption::new(ids::painter_ponta_option_id(i, k as u8), k as u8, tr(chave)))
        .collect();
    crate::dropdown_popover::paint_dropdown_popover(
        ctx,
        theme,
        ids::PAINTER_PONTA_FIELDS[i],
        opcoes,
        r,
        cur,
    );
}

/// A opção `id` do menu da linha `i` → o índice dela.
fn decode(i: usize, id: ph2d_a11y::NodeId) -> Option<u8> {
    (0..LINHAS_DA_PONTA[i].opcoes().len() as u8).find(|&k| ids::painter_ponta_option_id(i, k) == id)
}

/// O menu da ponta de píxel (L18).
pub(crate) fn decode_pixel_option(id: ph2d_a11y::NodeId) -> Option<u8> {
    decode(0, id)
}

/// O menu do modo do grão (L9).
pub(crate) fn decode_grain_mode_option(id: ph2d_a11y::NodeId) -> Option<u8> {
    decode(1, id)
}

/// O menu do controlo da profundidade (L9).
pub(crate) fn decode_depth_control_option(id: ph2d_a11y::NodeId) -> Option<u8> {
    decode(4, id)
}
