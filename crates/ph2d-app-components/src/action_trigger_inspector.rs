//! **O instantâneo e o dreno da secção GATILHO** — as duas pontas entre o painel e o mundo.
//!
//! ⚠️ **O instantâneo responde o que o painel não pode responder:** *«existe uma acção com este
//! nome?»* está no `InputMap`, que vive no `HeroScreen` — o painel não o vê. Sem essa coluna, uma
//! linha com `fier` em vez de `fire` é indistinguível de uma que funciona: as duas mostram o mesmo
//! texto e nenhuma dispara.
//!
//! ⚠️⚠️ **E aqui o silêncio é DUPLO**, ao contrário da vigia: a lei do motor cala uma acção que o
//! mapa não conhece (senão um `Release` sobre ela dispararia em TODO quadro, porque `!pressed` é
//! trivialmente verdade) — esta coluna é o que **explica** esse silêncio ao artista.

use ph2d_ecs::{ACTION_TRIGGERS_MAX, ActionEdge, ActionTriggerRow, SignalOnAction, SimWorld};
use ph2d_editor_core::action_trigger_edits::{
    ActionTriggerFieldEdit as E, InspectorActionTriggerInfo, InspectorTriggerRow, NoMapa,
};

/// O `u8` que atravessa a fronteira ⇄ o enum do motor.
///
/// ⚠️ **A tradução vive AQUI e em mais lado nenhum** — escrita duas vezes, o dia em que uma quarta
/// aresta nascer deixa o painel a escolher uma e o motor a fazer outra. Há gate de ida-e-volta.
#[must_use]
pub const fn edge_de_u8(v: u8) -> ActionEdge {
    match v {
        1 => ActionEdge::Release,
        2 => ActionEdge::Hold,
        _ => ActionEdge::Press,
    }
}

/// O caminho de volta.
#[must_use]
pub const fn u8_de_edge(e: ActionEdge) -> u8 {
    match e {
        ActionEdge::Press => 0,
        ActionEdge::Release => 1,
        ActionEdge::Hold => 2,
    }
}

/// ⭐ **Constrói o instantâneo** da entidade escolhida, ou `None` se ela não tem o componente.
///
/// O `no_mapa` é a ponte para o Input Map: ela recebe o nome da acção e diz o que o mapa sabe
/// dela — ver [`NoMapa`], que tem **três** estados porque dois deles produzem o MESMO silêncio.
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    clock_playing: bool,
    selected_count: usize,
    no_mapa: &dyn Fn(&str) -> NoMapa,
) -> Option<InspectorActionTriggerInfo> {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let cfg = sim.world().get::<SignalOnAction>(e)?;
    let rows = cfg
        .0
        .iter()
        .map(|r| InspectorTriggerRow {
            action: r.action.clone(),
            edge: u8_de_edge(r.edge),
            signal: r.signal.clone(),
            // ⚠️ **Um nome VAZIO não é «desconhecido»** — o painel tem uma frase própria para ele
            // (*«ainda não ouve nada»*), e contá-lo como órfão poria o título a dizer «1 broken»
            // num gatilho acabado de acrescentar, que é o estado normal de quem está a escrever.
            // ⇒ ele lê-se como **`Ligada`**, que é o estado que não acusa nada.
            no_mapa: if r.action.trim().is_empty() {
                NoMapa::Ligada
            } else {
                no_mapa(r.action.trim())
            },
        })
        .collect();
    Some(InspectorActionTriggerInfo {
        entity_bits: bits,
        rows,
        clock_playing,
        selected_count,
    })
}

/// Aplica uma edição. `true` = o mundo mudou.
///
/// ⚠️ **Escrever o MESMO valor não é uma mudança** — devolver `true` aqui faria cada quadro com o
/// campo focado marcar o componente como sujo, e o undo regista por diff.
fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let Some(mut cfg) = sim.world_mut().get_mut::<SignalOnAction>(e) else {
        return false;
    };
    let i = match edit {
        E::Add => {
            // ⚠️ **O tecto é o do MODELO**, e o painel esconde o `+` quando lá chega — mas a
            // guarda mora aqui na mesma: uma edição pode vir de um barramento drenado tarde.
            if cfg.0.len() >= ACTION_TRIGGERS_MAX {
                return false;
            }
            cfg.0.push(ActionTriggerRow::default());
            return true;
        }
        E::Remove(i) => {
            let i = usize::from(*i);
            if i >= cfg.0.len() {
                return false;
            }
            cfg.0.remove(i);
            return true;
        }
        E::Action(i, _) | E::Edge(i, _) | E::Signal(i, _) => usize::from(*i),
    };
    let Some(row) = cfg.0.get_mut(i) else {
        return false;
    };
    match edit {
        E::Action(_, nome) => {
            if row.action == *nome {
                return false;
            }
            row.action.clone_from(nome);
        }
        E::Edge(_, v) => {
            let g = edge_de_u8(*v);
            if row.edge == g {
                return false;
            }
            row.edge = g;
        }
        E::Signal(_, s) => {
            if row.signal == *s {
                return false;
            }
            row.signal.clone_from(s);
        }
        E::Add | E::Remove(_) => return false,
    }
    true
}

/// Aplica todas as edições do quadro. `true` = alguma coisa mudou.
///
/// ⚠️ **Não há runtime a reconciliar**, ao contrário da vigia: a aresta é trabalho do INPUT, e este
/// componente não guarda estado nenhum (ver o cabeçalho de `ph2d_ecs::signal_on_action`).
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

/// ⚠️ **A porta que o gate da shell lê** para provar que a tabela de ids do painel cobre as
/// variantes do enum do motor.
#[must_use]
pub const fn arestas() -> usize {
    ActionEdge::ALL.len()
}

#[cfg(test)]
#[path = "action_trigger_inspector_tests.rs"]
mod tests;
