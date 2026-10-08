//! ⭐⭐ **«Animation: Hero ▾»** — de quem é a animação que o painel mostra (doc
//! ObjectsAndComponents 01 §6). O chip segue a selecção sozinho; o ▾ escolhe outro dono da
//! cadeia (a Cena, um ancestral com Animação) ou dá Animação ao seleccionado que não a tem.
//!
//! Módulo próprio pelo tecto do `transport.rs`, e o mesmo molde do seletor de clips: o chip
//! pinta-se na barra, a lista aberta no passo adiado ([`paint_popover`]) — por baixo do dope
//! sheet ela ficaria tapada.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, PanelHostInternal};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip, paint_dropdown_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{AnimationIntent, SCENE, TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{ROW_H_PX, Theme};

use crate::ids;
use crate::state;
use crate::transport::ClipChip;

const ANIM_DD_W: f32 = 156.0; // LITERAL-PX-OK: animation-owner chip width

/// Quanto o chip ocupa na barra.
pub(crate) fn width(_snap: &TimelineViewSnapshot) -> f32 {
    ANIM_DD_W
}

fn label_of(bits: u64, name: &str) -> String {
    let who = if bits == SCENE {
        ph2d_i18n::tr("hier_rows.scene").to_string()
    } else if name.is_empty() {
        format!("#{}", bits & 0xffff)
    } else {
        name.to_string()
    };
    ph2d_i18n::tr("panel.timeline.animation_of").replace("{name}", &who)
}

/// **As opções, na ordem do ▾**: as escolhas do cabeçalho, depois a oferta. O valor é a linha;
/// [`pick_at`] lê de volta o que ela é.
pub(crate) fn options(snap: &TimelineViewSnapshot) -> Vec<DropdownOption<usize>> {
    let h = &snap.animation;
    let mut out: Vec<DropdownOption<usize>> = h
        .choices
        .iter()
        .map(|(bits, name)| (label_of(*bits, name), IconId::Film))
        .chain(h.offer.iter().map(|(bits, name)| {
            let who = if name.is_empty() { format!("#{}", bits & 0xffff) } else { name.clone() };
            (ph2d_i18n::tr("panel.timeline.animation_add").replace("{name}", &who), IconId::Plus)
        }))
        .zip(ids::TIMELINE_ANIM_OPT)
        .enumerate()
        .map(|(i, ((label, icon), id))| DropdownOption::new(id, i, label).with_icon(icon))
        .collect();
    out.truncate(ids::TIMELINE_ANIM_OPT.len());
    out
}

/// A linha do dono em vista.
fn selected(snap: &TimelineViewSnapshot) -> usize {
    snap.animation
        .choices
        .iter()
        .position(|(b, _)| *b == snap.animation.owner)
        .unwrap_or(0)
}

fn dropdown(snap: &TimelineViewSnapshot, open: bool) -> Dropdown<usize> {
    Dropdown::new(ids::TIMELINE_ANIM_DD, "", options(snap))
        .selected(selected(snap))
        .open(open)
}

/// O chip, na barra.
pub(crate) fn chip(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32, snap: &TimelineViewSnapshot) -> Option<ClipChip> {
    let rect = Rect::new(x, y, ANIM_DD_W, ROW_H_PX);
    ctx.host.hit_index_mut().register(ids::TIMELINE_ANIM_DD, rect);
    let open = matches!(
        ctx.host.store().get(ids::TIMELINE_ANIM_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let visual = ctx.host.store().dropdown_visual(ids::TIMELINE_ANIM_DD);
    paint_dropdown_chip(&dropdown(snap, open).visual(visual), rect, ctx.scene, ctx.text_system, theme);
    Some(ClipChip {
        id: ids::TIMELINE_ANIM_DD,
        rect,
        open,
    })
}

/// A lista aberta — no passo adiado, por cima do dope sheet. Regista cada opção (o rect só se
/// conhece aqui) e publica o popover para o clique de fora o fechar.
pub(crate) fn paint_popover(ctx: &mut PaintCtx, theme: Theme, snap: &TimelineViewSnapshot, chip: Rect) {
    let dd = dropdown(snap, true);
    paint_dropdown_popover(&dd, chip, ctx.scene, ctx.text_system, theme);
    for (i, opt) in dd.options.iter().enumerate() {
        ctx.host.hit_index_mut().register(opt.id, dd.option_rect(chip, i));
    }
    ctx.host
        .store_mut()
        .set_dropdown_popover(ids::TIMELINE_ANIM_DD, dd.popover_rect(chip));
}

/// O que a opção `id` pede.
fn pick_at(snap: &TimelineViewSnapshot, id: ph2d_a11y::NodeId) -> Option<AnimationIntent> {
    let row = ids::TIMELINE_ANIM_OPT.iter().position(|&o| o == id)?;
    let h = &snap.animation;
    match h.choices.get(row) {
        Some((bits, _)) => Some(AnimationIntent::Show(*bits)),
        None => (row == h.choices.len())
            .then_some(h.offer.as_ref()?)
            .map(|(bits, _)| AnimationIntent::Add(*bits)),
    }
}

/// O evento é deste chip?
pub(crate) fn owns(ev: &WidgetEvent) -> bool {
    matches!(*ev, WidgetEvent::Click(id) if ids::TIMELINE_ANIM_OPT.contains(&id))
}

/// Responde a uma opção escolhida (só chega aqui o que [`owns`] reclamou).
pub(crate) fn apply_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> EventOutcome {
    let WidgetEvent::Click(id) = ev else {
        return EventOutcome::Ignored;
    };
    let snap = state::current_snapshot();
    let pick = pick_at(&snap, id);
    if let Some(a) = pick {
        state::push_intent(TimelineIntent::Animation(a));
    }
    if let Some(InteractiveState::Dropdown {
        open, selected_index, ..
    }) = host.store_mut().get_mut(ids::TIMELINE_ANIM_DD)
    {
        *open = false;
        if let Some(row) = ids::TIMELINE_ANIM_OPT.iter().position(|&o| o == id) {
            *selected_index = Some(row);
        }
    }
    EventOutcome::Consumed
}
