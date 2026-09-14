//! **OS QUATRO POPOVERS DIFERIDOS do Inspector** — a §9 Sampling, a §7 Sorting Layer, a §12
//! «Rides Parent Anchor» e o VERBO da secção SIGNAL ACTIONS.
//!
//! ⚠️ **Irmão de [`super::paint_frame_shared`] por CAP de FICHEIRO** (600): o clamp e a rolagem
//! levaram-no a 679, e a regra da casa é cortar para o irmão, nunca subir a tolerância.
//!
//! ⚠️ **E o corte não é só de LOC — ele nunca pertenceu ali.** O ficheiro-pai é *«as quatro seções
//! COMPARTILHADAS»*, e estes quatro não são uma secção: são a mesma LEI aplicada quatro vezes —
//! *um popover aberto tem de sair da ordem em que a sua seção foi pintada, senão a seção seguinte
//! desenha-lhe por cima*. Eles pintam-se **depois de todas** elas.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::showcase::take_pending_dropdown_chip;
use ph2d_editor_core::widget::{
    self, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, scrollbar_is_needed,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

use crate::state_popovers;
use crate::{sections, state};

/// ⭐⭐⭐ **A PORTA ÚNICA por onde um popover diferido deste painel se pinta** — o clamp à região,
/// a rolagem, os hits e o que fica publicado.
///
/// ⚠️ **Ela existe porque a lei ia ser escrita QUATRO vezes**, e já tinha divergido do resto do app
/// nas duas metades que ninguém via: os quatro seletores penduravam a lista **sempre ABAIXO do
/// chip** (report do dono, 2026-09-09: *«o dropdown está abrindo fora da tela para baixo, não se
/// adapta à posição do widget»*) e **nenhum** publicava o rect do popover. Os outros painéis do app
/// já usavam o [`Dropdown::popover_rect_clamped`], que vira a lista para CIMA quando abaixo não
/// cabe. *Uma lei escrita em quatro sítios ainda não é uma lei — só uma PORTA é.*
///
/// ⚠️ **O clamp sozinho não chega, e é meia cura conhecida:** quando a lista não cabe de nenhum
/// lado ele **encolhe o painel**, e sem rolagem as entradas de baixo ficam desenhadas fora dele. É
/// por isso que esta porta faz as duas coisas — a lista do «Rides Parent Anchor» chega a **65**
/// entradas (uma por âncora do pai, mais o «—»), que não cabe em ecrã nenhum.
///
/// ⚠️ **A região do clamp é a BANDA DE CHROME** ([`HeroLayout::popover_region`]), nunca a janela:
/// dada a janela inteira, *«o lado com mais espaço»* é quase sempre para cima, e a lista nasce
/// colada à borda de cima, por cima da barra de topo, onde não há painel nenhum.
///
/// ⚠️ **Só a parte VISÍVEL de cada linha é hit-registada** — uma linha rolada para fora do painel
/// continuaria a apanhar o clique por baixo dele.
#[allow(clippy::too_many_arguments)]
fn paint_open_popover<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    store: &WidgetStore,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
) {
    let panel = dd.popover_rect_clamped(chip, region);
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    let scroll = store.panel_scroll(dd.id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    widget::paint_dropdown_popover_scrolled(
        dd,
        chip,
        panel,
        scroll,
        store.scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(dd.id)),
        scene,
        text_system,
        theme,
    );
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            // LITERAL-PX-OK: um pixel é o piso de uma faixa clicável, não uma medida de desenho
            hit_index.register(opt.id, Rect::new(r.x, top, r.w, bot - top));
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        hit_index.register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
    // ⚠️ **Publicar é do fim do `paint`** — aqui não há `&mut WidgetStore`. Ver `PAINTED_POPOVER`.
    state_popovers::set_painted_popover(dd.id, panel, content_h, visible_h);
}

/// **OS CINCO POPOVERS DIFERIDOS** — a §9 Sampling, a §7 Sorting Layer, a §12 «Rides Parent
/// Anchor», o VERBO da secção SIGNAL ACTIONS e o BARRAMENTO da secção AUDIO. Pintam-se DEPOIS de
/// todas as seções, para ficarem
/// acima de tudo.
///
/// ⚠️ **Irmãos por uma LEI, não por vizinhança:** um popover aberto tem de sair da ordem em que a
/// sua seção foi pintada, senão a seção seguinte desenha-lhe por cima. Cada um guarda o seu rect
/// num slot próprio durante o passe normal e é resgatado aqui.
///
/// Saíram do `paint_inspector` em 2026-08-23, quando o seletor da §12 o levou de 380 a 403 contra
/// uma tolerância que **só desce**. ⚠️ Levar só o novo devolveria o número a 380 exactos, e *ficar
/// no mesmo sítio não é encolher* — a mesma lição que o par de PRECISÃO e o par de sliders da
/// sprite já pagaram nesta família.
pub(crate) fn paint_deferred_popovers(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    region: Rect,
) {
    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = [
            tr("panel.inspector.sample.front"),
            tr("panel.inspector.sample.side"),
            tr("panel.inspector.sample.top"),
        ];
        let selected_label = labels
            .get(sel_idx)
            .copied()
            .unwrap_or(tr("panel.inspector.sample.front"));
        let dd = Dropdown::new(
            ids::INSP_SAMPLE_DROPDOWN,
            tr("panel.inspector.sample.view"),
            vec![
                DropdownOption::new(
                    ids::INSP_SAMPLE_DD_OPT_A,
                    "front",
                    tr("panel.inspector.sample.front"),
                ),
                DropdownOption::new(
                    ids::INSP_SAMPLE_DD_OPT_B,
                    "side",
                    tr("panel.inspector.sample.side"),
                ),
                DropdownOption::new(
                    ids::INSP_SAMPLE_DD_OPT_C,
                    "top",
                    tr("panel.inspector.sample.top"),
                ),
            ],
        )
        .selected(selected_label)
        .open(true);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    // W3 §7 Sorting Layer dropdown popover — same deferred-paint pass,
    // panel-local pending slot so it never collides with the sample dd.
    if let Some((sel_idx, chip)) = state_popovers::take_pending_ordering_dd() {
        let label = sections::ordering::LAYER_LABELS
            .get(sel_idx)
            .map_or(tr("panel.inspector.ordering.default"), |k| k.tr());
        let dd = Dropdown::new(
            crate::ids::INSP_ORDER_SORTING_LAYER,
            "",
            sections::ordering::layer_options(),
        )
        .selected(label)
        .open(true);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }

    // §12 «Rides Parent Anchor» — mesmo passe diferido, slot próprio.
    //
    // ⚠️ **As opções rederivam-se do snapshot aqui**, e não vêm no slot: guardá-las seria uma
    // segunda cópia da mesma verdade, e as duas divergiriam no quadro em que a seleção muda.
    if let Some(chip) = state_popovers::take_pending_mount_dd()
        && let Some(info) = state::current_inspector_anchor()
    {
        let mut dd = Dropdown::new(
            crate::ids::INSP_MOUNT_PICK,
            "",
            sections::anchor_mount_row::mount_options(&info),
        )
        .open(true)
        .placeholder(sections::anchor_mount_row::mount_placeholder(&info));
        if let Some(i) = info.mount_index() {
            dd.select(Some(i));
        }
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }

    // SIGNAL ACTIONS — o seletor do VERBO, mesmo passe diferido, slot próprio.
    //
    // ⚠️ **A TAG vem no slot e os RÓTULOS rederivam-se do snapshot**, e a assimetria é deliberada:
    // os rótulos são do snapshot, que é a fonte deles; a tag é a da linha ABERTA no editor, que
    // vive no `InspectorState` — e este passe não o alcança. *Guardar o que não se pode rederivar.*
    if let Some((sel, chip)) = state_popovers::take_pending_action_dd()
        && let Some(info) = state::current_inspector_action()
    {
        let mut dd = Dropdown::new(
            crate::ids::INSP_ACTION_VERB_PICK,
            "",
            sections::actions::verb_options(&info.verb_labels),
        )
        .open(true);
        dd.select(sel);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }

    // AUDIO — o seletor do BARRAMENTO, mesmo passe diferido, slot próprio.
    if let Some((sel, chip)) = state_popovers::take_pending_audio_dd()
        && let Some(info) = state::current_inspector_audio()
    {
        let mut dd = Dropdown::new(
            crate::ids::INSP_AUDIO_BUS_PICK,
            "",
            sections::audio::bus_options(&info.bus_labels),
        )
        .open(true);
        dd.select(sel);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }

    // ⭐⭐⭐ **OS TRÊS POPOVERS DE TAG** (TOP-20 #9) — no irmão, cortados em 2026-09-14 pelo tecto
    // de 200 LOC desta função (ela chegou a `210`).
    //
    // ⚠️ **O corte é por ASSUNTO e não por tamanho:** os quatro de cima escolhem um ENUM do
    // objecto (amostragem, camada, âncora, verbo); estes três escolhem uma **tag da árvore do
    // projecto**, e as opções deles saem de outra porta. ⛔ Subir o número seria adiar com juros.
    tags::paint_deferred_tag_popovers(scene, text_system, theme, hit_index, store, region);
}

/// ⭐ **Os três popovers de TAG** — irmão por `#[path]`; ver a chamada acima.
#[path = "popovers_tags.rs"]
mod tags;
