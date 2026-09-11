//! ⭐⭐⭐ **A secção SIGNAL ACTIONS** (TOP-20 #5, W3) — o snapshot que a secção lê e o commit que ela
//! escreve. Irmão do [`crate::render_loop::inspector_timer`], pela mesma razão dele.
//!
//! # ⚠️ O VERBO atravessa a fronteira como TAG, e o RÓTULO viaja com ele
//!
//! O painel é chrome e não depende do `ph2d-ecs` (ADR-0029). ⇒ o `SignalVerb` sai daqui como o
//! `u8` da posição dele em `SignalVerb::ALL`, e os rótulos vão no snapshot em vez de serem
//! copiados para o painel — cinco strings copiadas envelheceriam no primeiro verbo novo, e o
//! artista leria o nome errado sobre o botão certo.
//!
//! # ⚠️ O `uses_arg` é DERIVADO aqui, e não no painel
//!
//! É o mesmo motivo: o painel não conhece o enum. Re-derivá-lo lá seria a segunda resposta a
//! *«este campo serve para alguma coisa?»*, e um campo mostrado onde o verbo não o lê é um controlo
//! morto — a família que a caça de 2026-08-30 mediu em 34 controlos.
//!
//! # ⚠️ O snapshot só existe para quem TEM `SignalActions` (ADR-0166)

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    Entity, SIGNAL_ACTIONS_MAX, SignalAction, SignalActions, SignalVerb, SimWorld, World,
};
use ph2d_editor::{ActionFieldEdit, InspectorActionInfo, InspectorActionRow, Toast};

use crate::render_loop::inspector_ordering::queue_set;

const ACTIONS: &str = "ph2d::ecs::SignalActions";

/// O snapshot da secção, ou `None` quando o objecto não tem a tabela.
pub(super) fn build_action_info(
    world: &World,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorActionInfo> {
    let entity = Entity::from_bits(entity_bits);
    let table = world.get::<SignalActions>(entity)?;
    let rows = table
        .0
        .iter()
        .map(|a| InspectorActionRow {
            on: a.on.clone(),
            target: a.target.clone(),
            verb_tag: a.verb.tag(),
            arg: a.arg.clone(),
            uses_arg: a.verb.uses_arg(),
        })
        .collect();
    Some(InspectorActionInfo {
        entity_bits,
        rows,
        // ⚠️ **Os rótulos saem de `SignalVerb::ALL`, que é a FONTE** — nunca de uma lista escrita
        // à mão neste ficheiro.
        verb_labels: SignalVerb::ALL
            .iter()
            .map(|v| v.label().to_string())
            .collect(),
        selected_count,
    })
}

/// Aplica uma [`ActionFieldEdit`]. Devolve um aviso quando a edição foi **recusada**.
///
/// ⚠️ **Ler-modificar-escrever sobre o componente inteiro**, como as irmãs.
pub(super) fn apply_action_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: &ActionFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Toast> {
    let entity = Entity::from_bits(entity_bits);
    let mut table = sim.world().get::<SignalActions>(entity).cloned()?;

    match edit {
        ActionFieldEdit::Add => {
            if table.0.len() >= SIGNAL_ACTIONS_MAX {
                return Some(Toast::warning(format!(
                    "This object already has the maximum of {SIGNAL_ACTIONS_MAX} actions."
                )));
            }
            // ⚠️ **Nasce SEM nome de sinal**, e o painel di-lo em WARN. Um default que disparasse
            // em alguma coisa faria um `+` mudar a cena sem ninguém pedir.
            table.0.push(SignalAction::default());
        }
        ActionFieldEdit::Remove(i) => {
            let i = usize::from(*i);
            if i >= table.0.len() {
                return None;
            }
            table.0.remove(i);
        }
        ActionFieldEdit::On(i, name) => {
            let a = table.0.get_mut(usize::from(*i))?;
            // ⚠️ **Vazio é legítimo e não se recusa** — é uma linha por acabar, e o painel avisa.
            // ⛔ Recusá-la impediria o artista de limpar um nome errado.
            a.on = name.trim().to_string();
        }
        ActionFieldEdit::Target(i, name) => {
            let a = table.0.get_mut(usize::from(*i))?;
            a.target = name.trim().to_string();
        }
        ActionFieldEdit::Verb(i, tag) => {
            let a = table.0.get_mut(usize::from(*i))?;
            // ⚠️ **A tag volta a ser verbo pela porta do motor** (`from_tag`), que satura numa tag
            // fora da faixa — ela pode vir de um ficheiro gravado por uma versão com mais verbos.
            a.verb = SignalVerb::from_tag(*tag);
        }
        ActionFieldEdit::Arg(i, arg) => {
            let a = table.0.get_mut(usize::from(*i))?;
            a.arg = arg.trim().to_string();
        }
    }
    queue_set(queue, registry, entity_bits, ACTIONS, &table);
    None
}

/// Os gates desta secção — módulo irmão, pelo teto de 600 LOC da shell.
#[cfg(test)]
#[path = "inspector_action_tests.rs"]
mod tests;
