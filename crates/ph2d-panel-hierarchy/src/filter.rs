//! ⭐⭐ **O filtro ▾ da Hierarquia por componente** — «mostrar só o que tem Health» (doc
//! ObjectsAndComponents 01 §7.5 e §13, F6; escolha do dono 08/10: um botão ▾ ao lado da busca que
//! lista só os componentes presentes na cena, com contagem).
//!
//! A escolha vive no [`HierarchyState`]; o que cada linha TEM chega no snapshot
//! ([`HierarchyEntity::has`]). O filtro soma-se à busca por nome (as duas têm de casar) e, como ela,
//! mantém os ancestrais de quem casa. Mesmo molde do chip «Animation: X ▾» da Timeline: o chip
//! pinta-se no cabeçalho, a lista aberta depois das linhas ([`paint_popover`]) — por baixo delas
//! ficaria tapada.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::{ComponentCategory, ComponentSet};
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, paint_dropdown_chip, paint_dropdown_popover_in_viewport,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::ids::{HIER_FILTER_DD, HIER_FILTER_ROWS, hier_filter_opt};
use crate::state::HierarchyState;

/// A largura do chip — cabe «Health · 12»; nomes maiores encurtam no chip e aparecem inteiros na lista.
pub(crate) const FILTER_W: f32 = 104.0; // LITERAL-PX-OK: chip width, measured against «Health · 12» at TypeToken::Base

/// O nome de um bit: o do componente, ou «Animation».
fn name_of(bit: u8) -> &'static str {
    match ComponentSet::desc_of(bit) {
        Some(d) => ph2d_i18n::tr(d.display_key),
        None => ph2d_i18n::tr("panel.hierarchy.filter.animation"),
    }
}

/// A família de um bit, para ordenar a lista como a paleta.
fn rank(bit: u8) -> usize {
    let c = ComponentSet::desc_of(bit).map_or(ComponentCategory::Animation, |d| d.category);
    ComponentCategory::ALL
        .iter()
        .position(|x| *x == c)
        .unwrap_or(usize::MAX)
}

/// ⭐ **As linhas da lista, por ordem** — «All» primeiro, e depois cada componente que alguma linha
/// tem, com quantas o têm, na ordem das famílias da paleta. O escolhido fica na lista mesmo que ninguém
/// já o tenha (a contagem diz 0 e a árvore vazia explica-se). Uma porta, dois leitores — o pintor e o
/// clique ([`apply_event`]) —, para a linha pintada e a escolhida nunca discordarem.
pub(crate) fn rows(
    entries: &BTreeMap<NodeId, HierarchyEntity>,
    chosen: Option<u8>,
) -> Vec<(Option<u8>, String)> {
    let mut counts: BTreeMap<u8, u32> = BTreeMap::new();
    for e in entries.values() {
        for bit in e.has.iter() {
            *counts.entry(bit).or_default() += 1;
        }
    }
    if let Some(bit) = chosen {
        counts.entry(bit).or_default();
    }
    let mut bits: Vec<(u8, u32)> = counts.into_iter().collect();
    bits.sort_by_key(|(bit, _)| (rank(*bit), *bit));
    let mut out = vec![(None, ph2d_i18n::tr("panel.hierarchy.filter.all").to_string())];
    out.extend(bits.into_iter().map(|(bit, n)| {
        let label = ph2d_i18n::tr_with(
            "panel.hierarchy.filter.option",
            &[("name", &name_of(bit)), ("count", &n)],
        );
        (Some(bit), label)
    }));
    out.truncate(HIER_FILTER_ROWS);
    out
}

fn dropdown(rows: &[(Option<u8>, String)], chosen: Option<u8>, open: bool) -> Dropdown<usize> {
    let options = rows
        .iter()
        .enumerate()
        .map(|(i, (_, label))| DropdownOption::new(hier_filter_opt(i), i, label.clone()))
        .collect();
    let selected = rows.iter().position(|(b, _)| *b == chosen).unwrap_or(0);
    Dropdown::new(HIER_FILTER_DD, "", options)
        .selected(selected)
        .open(open)
}

/// A lista está aberta?
pub(crate) fn is_open(store: &WidgetStore) -> bool {
    matches!(
        store.get(HIER_FILTER_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// O chip, no cabeçalho, em `rect`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_chip(
    entries: &BTreeMap<NodeId, HierarchyEntity>,
    chosen: Option<u8>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    hit_index.register(HIER_FILTER_DD, rect);
    let dd = dropdown(&rows(entries, chosen), chosen, is_open(store))
        .visual(store.dropdown_visual(HIER_FILTER_DD));
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
}

/// A lista aberta — depois das linhas, por cima delas. Regista cada linha (o rect só se conhece
/// aqui) e devolve o rect do popover, que quem chama publica para o clique de fora a fechar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_popover(
    entries: &BTreeMap<NodeId, HierarchyEntity>,
    chosen: Option<u8>,
    chip: Rect,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> Rect {
    let dd = dropdown(&rows(entries, chosen), chosen, true);
    let text_w = ph2d_editor_core::paint::label_column_width(
        text_system,
        ph2d_tokens::TypeToken::Base.px(),
        dd.options.iter().map(|o| o.label.as_str()),
    );
    let chip = Rect::new(
        chip.x,
        chip.y,
        chip.w
            .max(ph2d_editor_core::widget::popover_width_for(text_w, false)),
        chip.h,
    );
    paint_dropdown_popover_in_viewport(&dd, chip, Some(viewport), scene, text_system, theme);
    let panel = dd.popover_rect_clamped(chip, viewport);
    for i in 0..dd.options.len() {
        let r = dd.option_rect_in(chip, panel, i);
        if r.y + r.h <= panel.y + panel.h {
            hit_index.register(hier_filter_opt(i), r);
        }
    }
    panel
}

/// A linha `id` da lista, se é dela.
fn row_of(id: NodeId) -> Option<usize> {
    (0..HIER_FILTER_ROWS).find(|&i| hier_filter_opt(i) == id)
}

/// O evento é desta lista?
pub(crate) fn owns(ev: &WidgetEvent) -> bool {
    matches!(*ev, WidgetEvent::Click(id) if row_of(id).is_some())
}

/// Responde a uma linha escolhida (só chega aqui o que [`owns`] reclamou): o filtro passa a ser
/// o dela, e a lista fecha.
pub(crate) fn apply_event(
    state: &mut HierarchyState,
    host: &mut dyn PanelHostInternal,
    ev: &WidgetEvent,
) -> EventOutcome {
    let WidgetEvent::Click(id) = *ev else {
        return EventOutcome::Ignored;
    };
    let Some(row) = row_of(id) else {
        return EventOutcome::Ignored;
    };
    let entries = crate::state::current_live_entries().unwrap_or_default();
    if let Some((bit, _)) = rows(&entries, state.component_filter).get(row) {
        state.component_filter = *bit;
    }
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(HIER_FILTER_DD)
    {
        *open = false;
        *selected_index = Some(row);
    }
    EventOutcome::Consumed
}
