//! ⭐⭐⭐ **A secção SIGNAL ACTIONS** (TOP-20 #5, W3) — o snapshot que a secção lê e o commit que ela
//! escreve. Irmão do [`super::inspector_timer`], pela mesma razão dele.
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
    Entity, SIGNAL_ACTIONS_MAX, SignalAction, SignalActions, SignalTarget, SignalVerb, SimWorld,
    World,
};
use ph2d_editor_core::screens::hero::ActionTargetMode;
use ph2d_editor_core::{ActionFieldEdit, InspectorActionInfo, InspectorActionRow, Toast};
use ph2d_i18n::{tr, tr_with};
use ph2d_tags::TagTree;

use ph2d_inspector_ordering::queue_set;

const ACTIONS: &str = "ph2d::ecs::SignalActions";

/// O snapshot da secção, ou `None` quando o objecto não tem a tabela.
pub(super) fn build_action_info(
    world: &World,
    // ⭐ A árvore de tags (TOP-20 #9, W3b) — para a linha mostrar o CAMINHO da tag alvo, e não o
    // número dela. ⚠️ O painel não conhece a árvore, e um id cru na tela não diz nada a ninguém.
    tree: &TagTree,
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
            uses_target: a.verb.uses_target(),
            // ⭐⭐⭐ **Os dois campos do suplente #24** — o modo do alvo e a cerca. ⚠️ **A tradução
            // mora AQUI e nos dois sentidos** (ver o dreno abaixo): o painel é chrome e não vê o
            // `ph2d-ecs` (ADR-0029), logo o que atravessa é a POSIÇÃO.
            target_mode: match a.target_by {
                ph2d_ecs::SignalTarget::Named => ActionTargetMode::Name,
                ph2d_ecs::SignalTarget::Tagged(_) => ActionTargetMode::Tag,
                ph2d_ecs::SignalTarget::Other => ActionTargetMode::Other,
            }
            .tag(),
            from_tag: a.from.tag(),
            target_tag: a.target_by.tag().map(|t| t.0),
            // ⚠️ **Vazio quando a tag já não existe**, e é isso que faz a secção poder dizer que a
            // linha partiu — ver `InspectorActionRow::target_tag_missing`.
            target_tag_path: a
                .target_by
                .tag()
                .and_then(|t| tree.get(t))
                .map(|t| t.path.clone())
                .unwrap_or_default(),
        })
        .collect();
    Some(InspectorActionInfo {
        entity_bits,
        rows,
        // ⚠️ **Os rótulos saem de `SignalVerb::ALL`, que é a FONTE** — nunca de uma lista escrita
        // à mão neste ficheiro.
        // ⭐ E a palavra vem da TABELA (2026-09-19): o motor publica a chave, e este é o único
        // sítio onde os sete verbos chegam a um pixel.
        verb_labels: SignalVerb::ALL
            .iter()
            .map(|v| tr(v.label_key()).to_string())
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
                return Some(Toast::warning(tr_with(
                    "shell.inspector_action.this_object_already",
                    &[("SIGNAL_ACTIONS_MAX", &SIGNAL_ACTIONS_MAX)],
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
        // ⭐⭐⭐ **O alvo por TAG** (TOP-20 #9, W3b).
        ActionFieldEdit::TargetMode(i, modo) => {
            let a = table.0.get_mut(usize::from(*i))?;
            // ⚠️ **`Tagged(0)` é «por tag, e ainda não escolheu qual»** — o `TagId(0)` nunca é dado
            // pela árvore, logo a linha alcança ninguém até o artista escolher, e a secção di-lo.
            // ⛔ Escolher uma tag por ele (a primeira da árvore, digamos) faria um clique num
            // segmentado mudar a quem a acção acerta.
            //
            // ⚠️ **A posição volta a ser modo pela porta do vocabulário** (`from_tag`), que satura
            // num valor fora da faixa — a mesma lei do verbo, um campo ao lado.
            a.target_by = match ActionTargetMode::from_tag(*modo) {
                ActionTargetMode::Name => SignalTarget::Named,
                ActionTargetMode::Tag => SignalTarget::Tagged(0),
                ActionTargetMode::Other => SignalTarget::Other,
            };
        }
        ActionFieldEdit::TargetTag(i, id) => {
            let a = table.0.get_mut(usize::from(*i))?;
            // ⚠️ **Escolher uma tag PÕE a linha no modo tag**, mesmo que ela estivesse por nome: o
            // gesto que chega aqui é o de uma caixa que só existe no modo tag, e recusá-lo por a
            // linha estar noutro modo seria um clique que não faz nada.
            a.target_by = SignalTarget::Tagged(*id);
        }
        // ⭐⭐⭐ **A CERCA** (suplente #24) — *de quem o sinal tem de vir*.
        ActionFieldEdit::From(i, tag) => {
            let a = table.0.get_mut(usize::from(*i))?;
            // ⚠️ **Pela porta do motor** (`SignalFrom::from_tag`), que satura numa tag fora da
            // faixa: ela pode vir de um ficheiro gravado por uma versão com mais cercas.
            a.from = ph2d_ecs::SignalFrom::from_tag(*tag);
        }
    }
    queue_set(queue, registry, entity_bits, ACTIONS, &table);
    None
}

/// Os gates desta secção — módulo irmão, pelo teto de 600 LOC da shell.
#[cfg(test)]
#[path = "inspector_action_tests.rs"]
mod tests;
