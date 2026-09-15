//! Gates do instantâneo e do dreno da secção STATE MACHINE (TOP-20 #15, W3).

use super::*;
use ph2d_ecs::{Name, SimWorld, StableId};

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let m = StateMachine {
        states: vec![
            MachineState {
                name: "Fechada".into(),
                on_enter: "porta_fechada".into(),
                on_exit: String::new(),
            },
            MachineState {
                name: "A abrir".into(),
                on_enter: "porta_a_abrir".into(),
                on_exit: String::new(),
            },
            MachineState {
                name: "Aberta".into(),
                on_enter: "porta_aberta".into(),
                on_exit: String::new(),
            },
        ],
        transitions: vec![
            StateTransition {
                from: 0,
                on: "botao".into(),
                to: 1,
            },
            StateTransition {
                from: 1,
                on: "fim".into(),
                to: 2,
            },
        ],
        initial: 0,
    };
    let e = sim
        .world_mut()
        .spawn((Name::new("Porta"), StableId(1), m))
        .id();
    (sim, e.to_bits())
}

/// ⚠️ **Sem o componente não há secção** (ADR-0166).
#[test]
fn quem_nao_tem_cerebro_nao_tem_seccao() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Name::new("Caixa"), StableId(1)))
        .id();
    assert!(build_statemachine_info(sim.world(), e.to_bits(), 1, true).is_none());
}

/// ⭐⭐ **O `has_exit` é DERIVADO**, e é o que acende o aviso de beco.
///
/// **Mutação que deve sangrar:** `has_exit: true` fixo.
#[test]
fn um_estado_sem_seta_de_saida_e_um_beco() {
    let (sim, bits) = cena();
    let i = build_statemachine_info(sim.world(), bits, 1, true).expect("tem o componente");
    assert_eq!(i.states.len(), 3);
    assert!(i.states[0].has_exit, "«Fechada» tem a seta do botao");
    assert!(i.states[1].has_exit, "«A abrir» tem a seta do fim");
    assert!(
        !i.states[2].has_exit,
        "«Aberta» e' um beco — e o painel tem de o dizer"
    );
}

/// ⭐⭐⭐ **Apagar um estado reescreve as setas** — a lei que só esta secção tem.
///
/// **Mutação que deve sangrar:** apagar o `retain` (as setas órfãs sobrevivem) ou o laço que recua
/// os índices (as setas passam a apontar o estado errado, **em silêncio**).
#[test]
fn apagar_um_estado_leva_as_setas_que_o_apontavam_e_recua_as_outras() {
    let (mut sim, bits) = cena();
    // Apaga o «A abrir» (índice 1): a seta `0->1` aponta-o e SAI; a `1->2` parte dele e SAI também.
    assert!(apply_statemachine_edit(
        sim.world_mut(),
        bits,
        &E::RemoveState(1)
    ));
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    assert_eq!(i.states.len(), 2);
    assert_eq!(
        i.transitions.len(),
        0,
        "as duas setas tocavam o estado apagado: {:?}",
        i.transitions
    );

    // E agora uma seta que NÃO o toca, mas que aponta para depois dele.
    let (mut sim, bits) = cena();
    assert!(apply_statemachine_edit(
        sim.world_mut(),
        bits,
        &E::RemoveState(0)
    ));
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    assert_eq!(i.transitions.len(), 1, "so' a `0->1` tocava o apagado");
    assert_eq!(
        (i.transitions[0].from, i.transitions[0].to),
        (0, 1),
        "a seta `1->2` RECUOU para `0->1` — sem isso ela apontaria o estado errado, em silencio"
    );
}

/// ⚠️ **Um índice fora da lista é cravado na PORTA** — ele não é um caso especial a lembrar em
/// cada leitura.
#[test]
fn um_indice_fora_da_lista_e_cravado_na_porta() {
    let (mut sim, bits) = cena();
    apply_statemachine_edit(sim.world_mut(), bits, &E::TransitionTo(0, 99));
    apply_statemachine_edit(sim.world_mut(), bits, &E::Initial(200));
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    assert_eq!(i.transitions[0].to, 2, "cravado no ultimo estado");
    assert_eq!(i.initial, 2);
}

/// ⚠️ **Uma seta nova NÃO nasce como auto-transição** — a lei recusa-a, e ela leria-se como um
/// controlo partido no instante em que nasce.
#[test]
fn uma_seta_nova_nao_nasce_apontando_para_si_mesma() {
    let (mut sim, bits) = cena();
    assert!(apply_statemachine_edit(
        sim.world_mut(),
        bits,
        &E::AddTransition
    ));
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    let nova = i.transitions.last().unwrap();
    assert_ne!(nova.from, nova.to);
}

/// ⚠️ **Os tectos da lei são honrados na porta** — um `+` que passasse deles produziria estado que
/// o painel não mostra.
#[test]
fn as_listas_param_nos_tectos_da_lei() {
    let (mut sim, bits) = cena();
    for _ in 0..40 {
        apply_statemachine_edit(sim.world_mut(), bits, &E::AddState);
    }
    for _ in 0..80 {
        apply_statemachine_edit(sim.world_mut(), bits, &E::AddTransition);
    }
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    assert_eq!(i.states.len(), ph2d_ecs::STATES_MAX);
    assert_eq!(i.transitions.len(), ph2d_ecs::TRANSITIONS_MAX);
}

/// ⭐ **O estado CORRENTE vem do VIVO, e é `None` enquanto ele não nascer.**
#[test]
fn o_estado_corrente_vem_do_vivo() {
    let (mut sim, bits) = cena();
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    assert_eq!(i.current, None, "o vivo ainda nao nasceu");
    crate::render_loop::state_machine_tick::advance_machines(&mut sim, &["botao"]);
    let i = build_statemachine_info(sim.world(), bits, 1, true).unwrap();
    assert_eq!(
        i.current,
        Some(1),
        "depois de um avanco com o `botao`, ele esta' em «A abrir»"
    );
}
