//! **A secção STATE MACHINE: o instantâneo e o dreno** (TOP-20 #15, W3).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o mesmo molde do
//! [`super::inspector_projectile`]: quem lê o mundo para o painel e quem escreve a edição de volta
//! fazem a MESMA tradução, e separá-los seria a porta pela qual as duas divergem.
//!
//! # ⭐⭐ O que esta secção faz que as irmãs não fazem: APAGAR UM ESTADO REESCREVE AS SETAS
//!
//! As setas guardam ÍNDICES (a lei do módulo: um nome ligaria a seta ao texto que o artista pode
//! reescrever a meio). ⇒ apagar o estado `k` desloca tudo o que vem depois dele, e uma seta que o
//! aponte deixa de ter sujeito.
//!
//! ⛔ **Uma seta que apontava o estado APAGADO é apagada com ele** — a alternativa (mantê-la a
//! apontar para outro) faria a máquina fazer, em silêncio, uma coisa que ninguém escreveu. *Um
//! elo partido que se conserta sozinho para o sítio errado é pior que um elo que desaparece.*

use ph2d_ecs::{Entity, MachineState, StateMachine, StateMachineRuntime, StateTransition, World};
use ph2d_editor_core::statemachine_edits::{
    InspectorStateMachineInfo, InspectorStateRow, InspectorTransitionRow,
    StateMachineFieldEdit as E,
};

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
pub(crate) fn build_statemachine_info(
    world: &World,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorStateMachineInfo> {
    let e = Entity::from_bits(bits);
    let m = world.get::<StateMachine>(e)?;
    let states = m
        .states
        .iter()
        .enumerate()
        .map(|(i, s)| InspectorStateRow {
            name: s.name.clone(),
            on_enter: s.on_enter.clone(),
            on_exit: s.on_exit.clone(),
            // ⚠️ **Derivado, nunca um campo** — um `bool` guardado ao lado divergiria da tabela no
            // dia em que alguém apagasse uma seta.
            has_exit: m
                .transitions
                .iter()
                .any(|t| t.from as usize == i && !t.on.is_empty()),
        })
        .collect();
    let transitions = m
        .transitions
        .iter()
        .map(|t| InspectorTransitionRow {
            from: t.from,
            on: t.on.clone(),
            to: t.to,
        })
        .collect();
    Some(InspectorStateMachineInfo {
        entity_bits: bits,
        states,
        transitions,
        initial: m.initial,
        // ⭐ O vivo — `None` enquanto ele não nasceu (a ponte cria-o no primeiro avanço).
        current: world.get::<StateMachineRuntime>(e).map(|rt| rt.current),
        clock_playing,
        selected_count,
    })
}

/// **Aplica uma edição.** `true` = o documento mudou.
pub(crate) fn apply_statemachine_edit(world: &mut World, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    let Some(mut m) = world.get_mut::<StateMachine>(e) else {
        return false;
    };
    match edit {
        E::AddState => {
            if m.states.len() >= ph2d_ecs::STATES_MAX {
                return false;
            }
            let n = m.states.len();
            m.states.push(MachineState {
                // ⚠️ **Um nome de partida e não vazio**: um estado sem nome lê-se como partido, e a
                // lista teria N linhas indistinguíveis.
                name: format!("State {n}"),
                on_enter: String::new(),
                on_exit: String::new(),
            });
            true
        }
        E::RemoveState(i) => {
            let i = *i as usize;
            if i >= m.states.len() {
                return false;
            }
            m.states.remove(i);
            // ⛔ **As setas que o apontavam SAEM; as que apontavam depois dele RECUAM.** Ver o
            // cabeçalho: um elo partido que se conserta para o sítio errado é pior que um elo que
            // desaparece.
            #[allow(clippy::cast_possible_truncation)]
            let k = i as u8;
            m.transitions.retain(|t| t.from != k && t.to != k);
            for t in &mut m.transitions {
                if t.from > k {
                    t.from -= 1;
                }
                if t.to > k {
                    t.to -= 1;
                }
            }
            if m.initial as usize >= m.states.len() {
                m.initial = 0;
            }
            true
        }
        E::StateName(i, v) => escreve(&mut m.states, *i, |s| s.name = v.clone()),
        E::StateOnEnter(i, v) => escreve(&mut m.states, *i, |s| s.on_enter = v.clone()),
        E::StateOnExit(i, v) => escreve(&mut m.states, *i, |s| s.on_exit = v.clone()),
        E::AddTransition => {
            if m.transitions.len() >= ph2d_ecs::TRANSITIONS_MAX || m.states.is_empty() {
                return false;
            }
            // ⚠️ **A seta nasce a apontar para o ÚLTIMO estado**, e não para o `0`: uma seta
            // `0 → 0` seria uma auto-transição, que a lei recusa — ela leria-se como um controlo
            // partido no instante em que nasce.
            let ultimo = u8::try_from(m.states.len().saturating_sub(1)).unwrap_or(0);
            m.transitions.push(StateTransition {
                from: 0,
                on: String::new(),
                to: ultimo,
            });
            true
        }
        E::RemoveTransition(i) => {
            let i = *i as usize;
            if i >= m.transitions.len() {
                return false;
            }
            m.transitions.remove(i);
            true
        }
        E::TransitionFrom(i, v) => {
            let topo = m.states.len();
            escreve(&mut m.transitions, *i, |t| t.from = crava(*v, topo))
        }
        E::TransitionOn(i, v) => escreve(&mut m.transitions, *i, |t| t.on = v.clone()),
        E::TransitionTo(i, v) => {
            let topo = m.states.len();
            escreve(&mut m.transitions, *i, |t| t.to = crava(*v, topo))
        }
        E::Initial(v) => {
            let novo = crava(*v, m.states.len());
            if m.initial == novo {
                return false;
            }
            m.initial = novo;
            true
        }
    }
}

/// ⚠️ **Um índice fora da lista não é um caso especial a lembrar em cada leitura: é um estado que
/// não pode existir.** Ele é cravado na porta, como o `press_point` do Input Map.
fn crava(v: u8, n_estados: usize) -> u8 {
    let topo = u8::try_from(n_estados.saturating_sub(1)).unwrap_or(0);
    v.min(topo) // CLAMP-OK: índice de estado
}

/// Escreve numa linha, se ela existir. `true` = mexeu.
fn escreve<T>(lista: &mut [T], i: u8, f: impl FnOnce(&mut T)) -> bool {
    match lista.get_mut(i as usize) {
        Some(item) => {
            f(item);
            true
        }
        None => false,
    }
}

#[cfg(test)]
#[path = "inspector_statemachine_tests.rs"]
mod tests;
