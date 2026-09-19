//! **O despacho da secção PATH FOLLOW** (suplente #23).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️⚠️ UMA CAIXA emite `Toggled`, UM CHIP emite `Click`
//!
//! É a lição que a **W9 do tween** pagou nesta mesma linha, com o gate de costura a apanhá-la antes
//! de shipar: um interruptor no ramo do clique fica **pintado, hit-registado, vivo sob o dedo — e
//! com o braço INALCANÇÁVEL**, com a suíte inteira verde. *Um controlo que responde ao evento
//! errado lê-se exactamente como um controlo morto.*
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::TimerFieldEdit;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::path_follow_edits::PathFollowFieldEdit;

/// A posição de `id` no array, se ele estiver lá.
fn tag(ids: &[ph2d_a11y::NodeId], id: ph2d_a11y::NodeId) -> Option<u8> {
    ids.iter()
        .position(|&x| x == id)
        .and_then(|i| u8::try_from(i).ok())
}

/// Despacha um evento da secção PATH FOLLOW. `true` = consumido.
#[allow(clippy::too_many_lines)]
pub(crate) fn apply_path_follow_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_path_follow() else {
        return false;
    };
    let bits = info.entity_bits;

    if let WidgetEvent::Toggled(id) = ev {
        // ⭐⭐⭐ **As duas do RELÓGIO escrevem no `Timers[i]`, pela MESMA porta que a secção TIMERS
        // usa** — não é uma segunda superfície sobre um valor, é uma porta com dois chamadores.
        if id == crate::ids::INSP_PF_REPEAT {
            relogio(
                host,
                bits,
                TimerFieldEdit::Repeat(info.relogio, !info.repeat),
            );
            return true;
        }
        if id == crate::ids::INSP_PF_AUTOSTART {
            relogio(
                host,
                bits,
                TimerFieldEdit::Autostart(info.relogio, !info.autostart),
            );
            return true;
        }
        if id == crate::ids::INSP_PF_ALINHA {
            push(host, bits, PathFollowFieldEdit::Alinha(!info.alinha));
            return true;
        }
        return false;
    }

    if let WidgetEvent::Click(id) = ev {
        // ⚠️ **A POSIÇÃO no array É a tag** — e é por isso que os arrays de ids são append-only.
        if let Some(t) = tag(&crate::ids::INSP_PF_CICLO, id) {
            push(host, bits, PathFollowFieldEdit::Ciclo(t));
            return true;
        }
        if let Some(t) = tag(&crate::ids::INSP_PF_AO_ACABAR, id) {
            push(host, bits, PathFollowFieldEdit::AoAcabar(t));
            return true;
        }
        if let Some(t) = tag(&crate::ids::INSP_PF_FAMILIA, id) {
            push(host, bits, PathFollowFieldEdit::Familia(t));
            return true;
        }
        if let Some(t) = tag(&crate::ids::INSP_PF_MODO, id) {
            push(host, bits, PathFollowFieldEdit::Modo(t));
            return true;
        }
        return false;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == crate::ids::INSP_PF_CAMINHO
    {
        // ⚠️ **O NOME cru** — quem o resolve numa curva é a ponte, que tem o mundo.
        let nome = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, PathFollowFieldEdit::Caminho(nome));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        // ⭐ **A DURAÇÃO sai por OUTRA porta** — ela é do `Timers[i]`, e em MICROSSEGUNDOS.
        if id == crate::ids::INSP_PF_DURACAO {
            relogio(
                host,
                bits,
                TimerFieldEdit::DurationSecs(info.relogio, f.max(0.0)),
            );
            return true;
        }
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_PF_RELOGIO => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                // ⚠️ O clamp do MODELO satura outra vez, no `TIMERS_MAX` — este só garante que a
                // conversão cabe.
                let n = v.clamp(0.0, f64::from(u8::MAX)) as u8; // CLAMP-OK: extremos literais, nao-NaN
                PathFollowFieldEdit::Relogio(n)
            }
            crate::ids::INSP_PF_DESLOCAMENTO => PathFollowFieldEdit::Deslocamento(f),
            crate::ids::INSP_PF_ANGULO => PathFollowFieldEdit::Angulo(f),
            crate::ids::INSP_PF_LADO => PathFollowFieldEdit::Lado(f),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: PathFollowFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::PathFollow(edit),
    });
}

/// ⭐⭐⭐ **A TERCEIRA chamadora da porta do relógio** — a primeira é o [`crate::event_timer`] e a
/// segunda o [`crate::event_tween`].
///
/// ⛔ *Não* é uma segunda superfície sobre um valor: é a MESMA `TimerFieldEdit`, no ÍNDICE que este
/// seguidor declara, e o dreno da shell é um só. Quem satura no `TIMER_MAX_US`, quem recusa e quem
/// grava continua a ser um.
fn relogio(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: TimerFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorTimerEdit { entity_bits, edit });
}
