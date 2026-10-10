//! ⭐⭐ **O seletor de fontes do Arrange da CENA** (doc ObjectsAndComponents 01 §14, onda 4 item 5).
//!
//! A Cena arruma no Arrange dela clips de QUALQUER objecto (§12), e o ▾ só oferecia a animação em
//! vista. Aqui ele lista os clips de TODAS as animações, um grupo por dono — o cabeçalho é o nome
//! do dono e não se clica — e rola quando não cabe (a porta da fundação `paint_dropdown_list`,
//! nunca uma barra nova). Irmão de [`crate::transport_clips`], que está perto do tecto de LOC.
//!
//! Escolher um clip da CENA continua a ser `SetActiveClip`; um de OUTRO dono não pode ser o clip
//! activo da vista (o `follow` devolve-o à Cena no quadro seguinte), logo fica como FONTE do painel
//! ([`TimelinePanelState::source_clip`]) e é ele que o `+` da lane põe no Arrange.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, PanelHostInternal, paint_dropdown_list};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, popover_width_for};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{SCENE, StripSource, TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{Theme, TypeToken};

use crate::ids::{self, TIMELINE_SOURCE_ROWS, timeline_source_opt};
use crate::state::{self, TimelinePanelState};
use crate::tab::Tab;
use crate::transport::BarView;

/// Um clip da lista agrupada: o índice do documento, se é da Cena, e o nome.
#[derive(Clone)]
pub(crate) struct Pick {
    index: usize,
    scene: bool,
    name: String,
}

/// O ▾ mostra a lista agrupada? Numa aba de LANES, na vista da Cena, com mais de uma animação.
pub(crate) fn grouped(snap: &TimelineViewSnapshot, tab: Tab) -> bool {
    tab != Tab::Keys && snap.animation.sources.len() > 1
}

/// As linhas, de cima para baixo: `(rótulo, None)` = cabeçalho, `(nome, Some(pick))` = clip.
fn rows(snap: &TimelineViewSnapshot) -> Vec<(String, Option<Pick>)> {
    let mut out = Vec::new();
    for g in &snap.animation.sources {
        // O cabeçalho nomeia o dono pela MESMA porta do chip «Animation: X ▾».
        out.push((crate::transport_animation::who(g.owner, &g.name), None));
        for (index, name, _) in &g.clips {
            let pick = Pick {
                index: *index,
                scene: g.owner == SCENE,
                name: name.clone(),
            };
            out.push((name.clone(), Some(pick)));
        }
    }
    out.truncate(TIMELINE_SOURCE_ROWS);
    out
}

/// As opções da lista agrupada (os containers seguem-nas, em [`crate::transport_clips`]).
pub(crate) fn options(snap: &TimelineViewSnapshot) -> Vec<DropdownOption<usize>> {
    rows(snap)
        .into_iter()
        .enumerate()
        .map(|(i, (label, pick))| {
            let o = DropdownOption::new(timeline_source_opt(i), 0, label);
            match pick {
                Some(_) => o.with_icon(crate::transport_clips::CLIP_GLYPH),
                None => o.disabled(true),
            }
        })
        .collect()
}

/// A linha do clip que o chip nomeia: a fonte de outro dono, senão o clip activo.
pub(crate) fn selected_row(snap: &TimelineViewSnapshot, foreign: Option<usize>) -> usize {
    let want = foreign.unwrap_or(snap.active_clip);
    rows(snap)
        .iter()
        .position(|(_, p)| p.as_ref().is_some_and(|p| p.index == want))
        .unwrap_or(0)
}

/// A fonte de OUTRO dono, se ainda é o clip que foi escolhido: `(índice, duração)`.
fn foreign(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> Option<(usize, f64)> {
    let (index, name) = state.source_clip.as_ref()?;
    snap.animation
        .sources
        .iter()
        .filter(|g| g.owner != SCENE)
        .flat_map(|g| g.clips.iter())
        .find(|(i, n, _)| i == index && n == name)
        .map(|(i, _, len)| (*i, *len))
}

/// A vista da barra — a mesma porta para o chip e para a lista aberta.
pub(crate) fn bar_view(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> BarView {
    BarView {
        tab: state.tab,
        speed_view: state.speed_view,
        source_container: state.source_container,
        source_clip: foreign(state, snap).map(|(i, _)| i),
    }
}

/// **O que o `+` da lane põe** quando a fonte não é um container: o clip de outro dono escolhido
/// no ▾, senão o clip activo — `(fonte, duração efectiva)`.
pub(crate) fn placed_clip(
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) -> (StripSource, f64) {
    let (index, len) = foreign(state, snap).unwrap_or((snap.active_clip, snap.clip_length_seconds));
    (
        StripSource::Clip(u16::try_from(index).unwrap_or(u16::MAX)),
        len,
    )
}

/// A linha da lista agrupada cujo id é `id`, contra a lista que a vista de hoje pinta.
pub(crate) fn row_of(id: ph2d_a11y::NodeId) -> Option<usize> {
    let n = row_count(&state::current_snapshot());
    (0..n).find(|&i| timeline_source_opt(i) == id)
}

/// Quantas linhas a lista agrupada tem (cabeçalhos incluídos), sem as construir.
pub(crate) fn row_count(snap: &TimelineViewSnapshot) -> usize {
    snap.animation
        .sources
        .iter()
        .map(|g| 1 + g.clips.len())
        .sum::<usize>()
        .min(TIMELINE_SOURCE_ROWS)
}

/// O clique numa linha (só chega aqui o que [`row_of`] reconheceu).
pub(crate) fn pick(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    row: usize,
) -> EventOutcome {
    let snap = state::current_snapshot();
    let Some(p) = rows(&snap).into_iter().nth(row).and_then(|(_, p)| p) else {
        return EventOutcome::Consumed; // um cabeçalho: nunca registado, e nada a escolher
    };
    if p.scene {
        state::push_intent(TimelineIntent::SetActiveClip { index: p.index });
        state.source_clip = None;
    } else {
        state.source_clip = Some((p.index, p.name));
    }
    state.source_container = None;
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(ids::TIMELINE_CLIP_DD)
    {
        *open = false;
        *selected_index = Some(row);
    }
    EventOutcome::Consumed
}

/// **A lista aberta do ▾**, por cima de tudo, pela porta da fundação — clampada à banda do chrome,
/// ROLADA quando não cabe, só as linhas VISÍVEIS no store (a lista pode ter milhares, e o Tab
/// percorre o store) e o rect com as duas alturas publicado.
pub(crate) fn paint_popover(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    chip: Rect,
) {
    let view = bar_view(state, snap);
    let dd = Dropdown::new(
        ids::TIMELINE_CLIP_DD,
        "",
        crate::transport_clips::source_options(snap, state.tab),
    )
    .selected(crate::transport_clips::selected_source(snap, view))
    .open(true);
    let text_w = ph2d_editor_core::paint::label_column_width(
        ctx.text_system,
        TypeToken::Base.px(),
        dd.options.iter().map(|o| o.label.as_str()),
    );
    let chip = Rect::new(
        chip.x,
        chip.y,
        chip.w.max(popover_width_for(text_w, true)),
        chip.h,
    );
    let region = ctx.layout.popover_region();
    paint_dropdown_list(ctx, &dd, chip, region, theme, None);
}
