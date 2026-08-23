//! **O `sync` das duas seções nascidas em 2026-08-21** — §5 9-Slice e §12 Sockets / Anchors.
//!
//! ⚠️ **Irmão do [`crate::sync`] por CAP** — de FICHEIRO (o `sync.rs` chegou a 661 contra 600) e
//! de FUNÇÃO (a `sync_inspector_from_snapshots` chegou a 208 contra 200). *Um cap de ficheiro e
//! um cap de função medem grandezas diferentes, e aqui os dois estouraram juntos.*
//!
//! A lei que ambas seguem é a das irmãs mais velhas: **um campo com foco ou a ser arrastado não
//! é reescrito** (o artista está a escrever nele), e **as caixas semeiam só na troca de
//! entidade** (reescrevê-las todo o quadro faria um clique voltar atrás antes de o commit
//! chegar).

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::CheckboxValue;

use crate::state::InspectorState;

/// A porta única: reflete as duas seções novas.
pub(crate) fn sync_new_sections(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    if let Some(sl) = crate::state::current_inspector_slice() {
        sync_slice_fields(host, &sl, entity_changed);
    }
    if let Some(an) = crate::state::current_inspector_anchor() {
        // ⚠️ **A semente é uma ARESTA, e ela tem DUAS causas** — a entidade mudou, ou a linha
        // aberta mudou. Só a primeira estava aqui, e o defeito que isso produzia foi medido em
        // 2026-08-23: clicar noutra âncora da mesma sprite deixava o nome e as caixas a mostrar
        // a âncora anterior (sonda: nome `""`, `Bounds` desmarcada sobre uma que tem área).
        let row = if an.rows.is_empty() {
            0
        } else {
            inspector_state.anchor_selected.min(an.rows.len() - 1)
        };
        let row_changed = inspector_state.last_anchor_row != Some(row);
        inspector_state.last_anchor_row = Some(row);
        sync_anchor_fields(host, &an, row, entity_changed || row_changed);
    }
    if let Some(an) = crate::state::current_inspector_anim() {
        // ⚠️ **A mesma ARESTA da §12**, e pelo mesmo defeito medido: os campos do editor
        // semeiam-se quando a ENTIDADE **ou** a linha aberta muda, nunca por quadro. Reescrever
        // sempre desfaria o número que o artista acabou de digitar antes de o commit chegar.
        let row = if an.rows.is_empty() {
            0
        } else {
            inspector_state.anim_selected.min(an.rows.len() - 1)
        };
        let row_changed = inspector_state.last_anim_row != Some(row);
        inspector_state.last_anim_row = Some(row);
        sync_anim_fields(host, &an, row, entity_changed || row_changed);
    }
}

/// Semeia os campos da §11 a partir do snapshot.
///
/// ⚠️ **A VELOCIDADE e as duas caixas do tocador semeiam-se em TODA aresta de entidade** — elas
/// são do animador, e não da linha aberta. Os campos da biblioteca (nome, intervalo, ritmo) são
/// da linha, e por isso re-semeiam-se também ao trocar de linha.
fn sync_anim_fields(
    host: &mut dyn PanelHostInternal,
    an: &ph2d_editor_core::screens::hero::InspectorAnimInfo,
    selected: usize,
    seed: bool,
) {
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    let put = |host: &mut dyn PanelHostInternal, id, v: f64| {
        if focus != Some(id) && drag != Some(id) {
            host.store_mut().set_number_value(id, v);
        }
    };
    put(host, ids::INSP_ANIM_SPEED, f64::from(an.speed));
    if let Some(row) = an.rows.get(selected) {
        put(host, ids::INSP_ANIM_FROM, f64::from(row.from));
        put(host, ids::INSP_ANIM_TO, f64::from(row.to));
        put(host, ids::INSP_ANIM_FRAME_MS, f64::from(row.frame_ms));
        put(host, ids::INSP_ANIM_HOLD_MS, f64::from(row.hold_ms));
        put(
            host,
            ids::INSP_ANIM_DELAY_MS,
            f64::from(row.repeat_delay_ms),
        );
        put(host, ids::INSP_ANIM_REPEAT, f64::from(row.repeat));
    }
    if !seed {
        return;
    }
    if focus != Some(ids::INSP_ANIM_NAME)
        && let Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = host.store_mut().get_mut(ids::INSP_ANIM_NAME)
    {
        text.clear();
        if let Some(row) = an.rows.get(selected) {
            text.push_str(&row.name);
        }
        *caret = text.len();
        *selection_anchor = None;
    }
    for (id, on) in [
        (ids::INSP_ANIM_PLAYING, an.playing),
        (ids::INSP_ANIM_AUTOPLAY, an.autoplay),
    ] {
        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
            *value = if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
}

/// Reflete o snapshot da §5 9-Slice nos widgets.
///
/// ⚠️ **Função irmã por CAP** (200): com este bloco inline a `sync_inspector_from_snapshots`
/// media 240. *A cura de um teto estourado é o corte.*
fn sync_slice_fields(
    host: &mut dyn PanelHostInternal,
    sl: &ph2d_editor_core::screens::hero::InspectorSliceInfo,
    entity_changed: bool,
) {
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, value) in [
        (ids::INSP_SLICE_BORDER[0], f64::from(sl.borders[0])),
        (ids::INSP_SLICE_BORDER[1], f64::from(sl.borders[1])),
        (ids::INSP_SLICE_BORDER[2], f64::from(sl.borders[2])),
        (ids::INSP_SLICE_BORDER[3], f64::from(sl.borders[3])),
        (ids::INSP_SLICE_SIZE[0], f64::from(sl.size[0])),
        (ids::INSP_SLICE_SIZE[1], f64::from(sl.size[1])),
    ] {
        if focus != Some(id) && drag != Some(id) {
            host.store_mut().set_number_value(id, value);
        }
    }
    // ⚠️ O checkbox só semeia na TROCA de entidade: reescrevê-lo todo o quadro faria um
    // clique voltar atrás antes de o commit chegar (o snapshot deste quadro ainda é o de
    // antes da edição). É a mesma lei dos checkboxes da §7.
    if entity_changed {
        // ⚠️ **A caixa que LIGA o 9-slice** — ela lê `present` E o modo. Um sprite sem componente
        // e um com o modo desligado são o mesmo estado para quem olha: 9-slice desligado.
        let enable = if sl.mixed.enabled {
            CheckboxValue::Indeterminate
        } else if sl.present && sl.draw_mode_tag == 1 {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
        if let Some(InteractiveState::Checkbox { value: slot, .. }) =
            host.store_mut().get_mut(ids::INSP_SLICE_ENABLE)
        {
            *slot = enable;
        }
        let value = if sl.mixed.fill_center {
            CheckboxValue::Indeterminate
        } else if sl.fill_center {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
        if let Some(InteractiveState::Checkbox { value: slot, .. }) =
            host.store_mut().get_mut(ids::INSP_SLICE_FILL_CENTER)
        {
            *slot = value;
        }
    }
}

/// Reflete a âncora ABERTA da §12 nos widgets do editor.
///
/// ⚠️ **Sem âncoras, não escreve nada.** Reescrever os campos a zero deixaria o editor a mostrar
/// uma pose que não pertence a âncora nenhuma — e o despacho, que só age com a lista não-vazia,
/// não a consumiria. *Um campo que mostra um valor que ninguém lê é a mentira nº 1 desta
/// auditoria.*
fn sync_anchor_fields(
    host: &mut dyn PanelHostInternal,
    an: &ph2d_editor_core::screens::hero::InspectorAnchorInfo,
    selected: usize,
    entity_changed: bool,
) {
    let Some(row) = an.rows.get(selected.min(an.rows.len().saturating_sub(1))) else {
        return;
    };
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    let b = row.bounds.unwrap_or([0.0; 4]);
    let c = row.center.unwrap_or([0.0; 4]);
    for (id, value) in [
        (ids::INSP_ANCHOR_POS[0], f64::from(row.pos[0])),
        (ids::INSP_ANCHOR_POS[1], f64::from(row.pos[1])),
        (ids::INSP_ANCHOR_ROT, f64::from(row.rot_deg)),
        (ids::INSP_ANCHOR_BOUNDS[0], f64::from(b[0])),
        (ids::INSP_ANCHOR_BOUNDS[1], f64::from(b[1])),
        (ids::INSP_ANCHOR_BOUNDS[2], f64::from(b[2])),
        (ids::INSP_ANCHOR_BOUNDS[3], f64::from(b[3])),
        (ids::INSP_ANCHOR_CENTER[0], f64::from(c[0])),
        (ids::INSP_ANCHOR_CENTER[1], f64::from(c[1])),
        (ids::INSP_ANCHOR_CENTER[2], f64::from(c[2])),
        (ids::INSP_ANCHOR_CENTER[3], f64::from(c[3])),
    ] {
        if focus != Some(id) && drag != Some(id) {
            host.store_mut().set_number_value(id, value);
        }
    }
    // As duas caixas e o NOME semeiam na troca de entidade — reescrevê-los todo o quadro faria
    // um clique voltar atrás antes de o commit chegar, e apagaria o que se estivesse a escrever.
    if entity_changed
        && focus != Some(ids::INSP_ANCHOR_NAME)
        && let Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = host.store_mut().get_mut(ids::INSP_ANCHOR_NAME)
    {
        text.clear();
        text.push_str(&row.name);
        *caret = text.len();
        *selection_anchor = None;
    }
    if entity_changed {
        for (id, on) in [
            (ids::INSP_ANCHOR_BOUNDS_ON, row.bounds.is_some()),
            (ids::INSP_ANCHOR_CENTER_ON, row.center.is_some()),
        ] {
            if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
                *value = if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                };
            }
        }
    }
}
