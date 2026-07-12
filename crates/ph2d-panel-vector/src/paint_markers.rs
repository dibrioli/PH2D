//! As **pontas do traço** (arrowheads) na seção STROKE — módulo irmão de
//! `paint_sections` (teto de 600 LOC por arquivo de painel).
//!
//! ## Por que aqui, e não no catálogo de formas
//!
//! Uma ponta não é uma forma: é uma **propriedade do stroke**. Nasce no extremo de um
//! caminho aberto, herda a cor e a largura dele, e gira sozinha com a tangente da curva.
//! Por isso mora ao lado de Cap / Join / Dash — as outras três decisões sobre *como* a
//! caneta termina —, e não na grade de formas.
//!
//! ## Por que um DROPDOWN, e não um botão que cicla
//!
//! O painel tem os dois vocabulários: o botão que cicla (`labeled_choice_button`, para as
//! escolhas de 2-3 opções de uma forma) e o chip de dropdown (a CATEGORIA do catálogo,
//! sete famílias). As pontas são **oito** (`ALL_MARKERS`), e a régua já foi decidida uma
//! vez neste mesmo painel: sete opções viraram chip. Ciclar às cegas por oito nomes até
//! achar "Diamond (hollow)" é pior que ler os oito e apontar — e o chip mostra a ponta
//! corrente sem gastar altura nenhuma.
//!
//! ## Gotcha (o mesmo do catálogo)
//!
//! O popover **tem** de ser pintado no passe DIFERIDO de `paint.rs`: dentro do corpo, o
//! `push_clip` do scroll o cortaria na borda da seção. O chip só guarda o rect (+ o SLOT,
//! porque são dois seletores e o passe é um só).

use crate::ids;
use crate::paint_sections::{BodyCtx, LABEL_COL_W};
use crate::state;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_vec_scene::{ALL_MARKERS, Marker};

/// O chip do seletor `slot` (0 = começo, 1 = fim).
#[must_use]
pub(crate) fn marker_dd_id(slot: usize) -> ph2d_a11y::NodeId {
    if slot == ph2d_tool_vector::params::MARKER_SLOT_END {
        ids::VECTOR_MARKER_END_DD
    } else {
        ids::VECTOR_MARKER_START_DD
    }
}

/// A ponta que o seletor `slot` mostra no snapshot corrente.
#[must_use]
fn marker_of(snap: &VectorStyleSnapshot, slot: usize) -> Marker {
    if slot == ph2d_tool_vector::params::MARKER_SLOT_END {
        snap.marker_end
    } else {
        snap.marker_start
    }
}

/// A chave i18n do rótulo do seletor. Só o RÓTULO da linha passa pelo i18n: os NOMES das
/// pontas vêm de [`Marker::label`], que é a fonte única deles (a mesma regra dos nomes de
/// forma, que vivem no catálogo da tool).
#[must_use]
fn slot_i18n_key(slot: usize) -> &'static str {
    if slot == ph2d_tool_vector::params::MARKER_SLOT_END {
        "panel.vector.marker.end"
    } else {
        "panel.vector.marker.start"
    }
}

impl BodyCtx<'_> {
    /// As duas linhas de ponta (`Start` / `End`), no fim da seção STROKE. Devolve o `y`
    /// avançado.
    pub(crate) fn marker_rows(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        for slot in 0..ids::MARKER_SLOTS {
            y = self.marker_row(snap, slot, y);
        }
        y
    }

    /// Uma linha: rótulo + chip de dropdown mostrando a ponta corrente. Mesmo desenho da
    /// linha de CATEGORIA (rótulo na coluna de rótulos, chip ocupando o resto).
    fn marker_row(&mut self, snap: &VectorStyleSnapshot, slot: usize, y: f32) -> f32 {
        let gap = Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            tr(slot_i18n_key(slot)),
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            LABEL_COL_W,
            resolve(ColorToken::Text2, self.theme),
        );
        let id = marker_dd_id(slot);
        let chip = Rect::new(
            self.inner_x + LABEL_COL_W + gap,
            y,
            (self.inner_w - LABEL_COL_W - gap).max(1.0),
            self.row_h,
        );
        let open = matches!(
            self.store.get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(
            id,
            "",
            vec![DropdownOption::new(id, (), marker_of(snap, slot).label())],
        )
        .selected(())
        .open(open);
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, chip);
        if open {
            // O popover pinta no passe DIFERIDO (fora do clip do scroll) — `paint.rs`.
            state::set_pending_marker_dd(Some((slot, chip)));
        }
        y + self.row_h + self.row_gap
    }
}

/// O popover de um seletor de ponta — pintado no passe DIFERIDO de `paint.rs`, POR CIMA
/// de todas as seções. Espelho de `paint_catalog::paint_group_popover`: as opções são
/// `ALL_MARKERS` (dado, não código), registradas por índice.
pub(crate) fn paint_marker_popover(ctx: &mut PaintCtx, slot: usize, chip: Rect, theme: Theme) {
    let id = marker_dd_id(slot);
    let current = marker_of(&state::current_snapshot(), slot);
    let sel = ALL_MARKERS.iter().position(|m| *m == current).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = ALL_MARKERS
        .iter()
        .enumerate()
        .map(|(i, m)| DropdownOption::new(ids::vector_marker_option_id(slot, i), i, m.label()))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.viewport);
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        scrollbar_active,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha (a barra de rolagem é o alvo do drag).
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..ALL_MARKERS.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_marker_option_id(slot, i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O catálogo de pontas cabe no espaço de ids que o painel registra — senão a última
    /// ponta nasceria sem opção clicável no popover (pintada e morta).
    #[test]
    fn every_marker_has_an_option_slot() {
        assert!(
            ALL_MARKERS.len() <= ids::MAX_MARKER_OPTIONS,
            "ALL_MARKERS cresceu além de MAX_MARKER_OPTIONS — as últimas pontas não teriam id"
        );
    }

    /// Os dois seletores endereçam CHIPS DIFERENTES e opções DIFERENTES: um id repetido
    /// faria a ponta do começo e a do fim se sobrescreverem.
    #[test]
    fn the_two_selectors_never_share_an_id() {
        assert_ne!(marker_dd_id(0), marker_dd_id(1));
        for i in 0..ALL_MARKERS.len() {
            assert_ne!(
                ids::vector_marker_option_id(0, i),
                ids::vector_marker_option_id(1, i),
                "opcao {i}: comeco e fim colidiram"
            );
        }
    }
}
