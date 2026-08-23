//! **O despacho da §11 Animation** (spec Sprite 08 §8.7).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_anchor`].
//!
//! # ⚠️ Clicar numa linha VAI ao barramento — ao contrário da §12
//!
//! Na §12, escolher que âncora se edita é um facto da UI e fica no painel. Aqui a linha aberta
//! **é** a animação que toca, e isso é estado da cena: o clique escreve
//! `SpriteAnimator::current` e entra no undo. A razão está no doc de
//! [`crate::sections::anim`] — separar as duas pediria um seletor que duplicaria a lista.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::AnimFieldEdit;
use ph2d_editor_core::widget::ButtonState;

use crate::state::{self, InspectorState};

/// Despacha um evento da §11. `true` = consumido.
pub(crate) fn apply_anim_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = state::current_inspector_anim() else {
        return false;
    };
    let sel = panel.anim_selected.min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        // Uma linha da lista: abre a ficha **e** escolhe o que toca.
        if let Some(i) = ids::INSP_ANIM_ROW.iter().position(|&o| o == id)
            && let Some(row) = info.rows.get(i)
        {
            panel.anim_selected = i;
            push(
                host,
                info.entity_bits,
                AnimFieldEdit::SetCurrent(row.name.clone()),
            );
            demote(host, id);
            return true;
        }
        if id == ids::INSP_ANIM_ADD_PLAYER && !info.player_present {
            push(host, info.entity_bits, AnimFieldEdit::AddPlayer);
            demote(host, id);
            return true;
        }
        if id == ids::INSP_ANIM_ADD {
            push(host, info.entity_bits, AnimFieldEdit::Add);
            demote(host, id);
            return true;
        }
        if id == ids::INSP_ANIM_REMOVE && !info.rows.is_empty() {
            push(host, info.entity_bits, AnimFieldEdit::Remove(sel_u8));
            demote(host, id);
            return true;
        }
        if id == ids::INSP_ANIM_REWIND {
            push(host, info.entity_bits, AnimFieldEdit::Rewind);
            demote(host, id);
            return true;
        }
        // Os três segmentados. ⚠️ A POSIÇÃO no array é a tag — reordenar o array faria um clique
        // escrever outra direção, e compila.
        if let Some(i) = ids::INSP_ANIM_DIR.iter().position(|&o| o == id)
            && !info.rows.is_empty()
        {
            push(
                host,
                info.entity_bits,
                AnimFieldEdit::Direction(sel_u8, i as u8),
            );
            demote(host, id);
            return true;
        }
        if let Some(i) = ids::INSP_ANIM_DIR_OVERRIDE.iter().position(|&o| o == id) {
            push(
                host,
                info.entity_bits,
                AnimFieldEdit::DirectionOverride(i as u8),
            );
            demote(host, id);
            return true;
        }
        if let Some(i) = ids::INSP_ANIM_LOOP_OVERRIDE.iter().position(|&o| o == id) {
            push(host, info.entity_bits, AnimFieldEdit::LoopOverride(i as u8));
            demote(host, id);
            return true;
        }
    }

    // ⚠️ **AS DUAS CAIXAS DECIDEM PELO QUE ESTÁ DESENHADO, e não pelo que o store lembra.**
    //
    // O pintor da §11 já tira o valor do SNAPSHOT (`sections::anim`, `.value(info.playing)`) —
    // ler aqui o `host.store().checkbox(id)` dava **duas fontes de verdade para uma caixa**, e
    // elas divergem no instante em que alguém que não é o painel escreve o facto. O `playing` é
    // exatamente esse campo: uma animação que não repete põe-se a `false` sozinha ao chegar ao
    // fim, sem que a entidade nem a linha aberta mudem, por isso a semente do sync não volta a
    // correr e o valor guardado fica a dizer «marcada» sobre uma caixa desenhada vazia.
    //
    // Sintoma medido (Enio, 2026-08-23): *«às vezes preciso clicar mais de uma vez para checar
    // Playing»* — o primeiro clique mandava `Playing(false)` a uma cena já parada.
    //
    // ⇒ O clique afirma **o contrário do que estava no ecrã**, que é o que um toggle promete.
    // Gate: `seam_anim::the_playing_box_asks_the_scene_not_its_own_memory`.
    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_ANIM_PLAYING | ids::INSP_ANIM_AUTOPLAY)
    {
        let edit = if id == ids::INSP_ANIM_PLAYING {
            AnimFieldEdit::Playing(!info.playing)
        } else {
            AnimFieldEdit::Autoplay(!info.autoplay)
        };
        push(host, info.entity_bits, edit);
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_ANIM_NAME
        && !info.rows.is_empty()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(host, info.entity_bits, AnimFieldEdit::Rename(sel_u8, text));
        return true;
    }

    // ⚠️ **A BARRA DE FRAMES** — o `0..1` do slider vira célula pela INVERSA da mesma lei que a
    // pinta (`InspectorAnimInfo::scrub_cell` ↔ `scrub_position`).
    //
    // ⚠️ **A régua vivia aqui, no pintor e no `sync`, em três cópias** — e uma mutação que mudou só
    // a do pintor **sobreviveu** à suíte inteira, porque nenhum gate ligava o que se desenha ao que
    // o clique produz. Hoje é uma função só, com gate de ida-e-volta no modelo.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == ids::INSP_ANIM_FRAME_SCRUB
    {
        let v = host.store().slider(id).map_or(0.0, |(_, v)| v);
        if let Some(cell) = info.scrub_cell(v) {
            push(host, info.entity_bits, AnimFieldEdit::SetFrame(cell));
        }
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        // A velocidade é do TOCADOR e não da linha — ela não precisa da biblioteca.
        if id == ids::INSP_ANIM_SPEED {
            #[allow(clippy::cast_possible_truncation)]
            push(host, info.entity_bits, AnimFieldEdit::Speed(v as f32));
            return true;
        }
        if info.rows.is_empty() {
            return false;
        }
        // ⚠️ Um valor negativo num campo de célula ou de milissegundos não existe: o `max(0.0)`
        // mora aqui, na entrada, para que o commit receba sempre um `u32` honesto.
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let n = v.max(0.0) as u32;
        let edit = match id {
            _ if id == ids::INSP_ANIM_FROM => Some(AnimFieldEdit::From(sel_u8, n)),
            _ if id == ids::INSP_ANIM_TO => Some(AnimFieldEdit::To(sel_u8, n)),
            _ if id == ids::INSP_ANIM_FRAME_MS => Some(AnimFieldEdit::FrameMs(sel_u8, n)),
            _ if id == ids::INSP_ANIM_HOLD_MS => Some(AnimFieldEdit::HoldMs(sel_u8, n)),
            _ if id == ids::INSP_ANIM_DELAY_MS => Some(AnimFieldEdit::DelayMs(sel_u8, n)),
            _ if id == ids::INSP_ANIM_REPEAT => Some(AnimFieldEdit::Repeat(sel_u8, n)),
            _ => None,
        };
        if let Some(edit) = edit {
            push(host, info.entity_bits, edit);
            return true;
        }
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: AnimFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorAnimEdit { entity_bits, edit });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
