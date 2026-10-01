//! Gates da PONTE do cérebro — a lei tem os dela em `ph2d-ecs`; estes medem o mundo.

use super::*;
use ph2d_ecs::{MachineState, Name, StableId, StateMachine, StateTransition};

fn maquina() -> StateMachine {
    StateMachine {
        states: vec![
            MachineState {
                name: "Fechada".into(),
                on_enter: "porta_fechada".into(),
                on_exit: String::new(),
            },
            MachineState {
                name: "Aberta".into(),
                on_enter: "porta_aberta".into(),
                on_exit: String::new(),
            },
        ],
        transitions: vec![StateTransition {
            from: 0,
            on: "botao".into(),
            to: 1,
        }],
        initial: 0,
    }
}

/// ⭐⭐ **O vivo NASCE sem ninguém o pedir**, e antes de qualquer sinal.
///
/// **Mutação que deve sangrar:** apagar a chamada ao `ensure_runtime`.
#[test]
fn o_vivo_de_uma_maquina_nasce_no_primeiro_avanco() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Name::new("Porta"), StableId(1), maquina()))
        .id();
    assert!(
        sim.world().get::<StateMachineRuntime>(e).is_none(),
        "a fixtura tem de comecar SEM o vivo — ele nao vem do ficheiro"
    );
    let anunciados = advance_machines(&mut sim, &[]);
    let rt = sim.world().get::<StateMachineRuntime>(e).expect("o vivo");
    assert_eq!(rt.current, 0);
    assert_eq!(
        anunciados.len(),
        1,
        "entrar no inicial anuncia-se: {:?}",
        anunciados.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    assert_eq!(anunciados[0].name, "porta_fechada");
    assert_eq!(anunciados[0].entity, e, "quem pensou vai no anuncio");
}

/// ⭐⭐⭐ **O sinal chega à máquina e ela anuncia** — a costura inteira.
#[test]
fn um_sinal_move_a_maquina_do_mundo() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Name::new("Porta"), StableId(1), maquina()))
        .id();
    advance_machines(&mut sim, &[]);
    let anunciados = advance_machines(&mut sim, &["botao"]);
    assert_eq!(
        sim.world()
            .get::<StateMachineRuntime>(e)
            .expect("o vivo")
            .current,
        1
    );
    let nomes: Vec<&str> = anunciados.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(nomes, vec!["porta_aberta"]);
}

/// ⚠️ **A ORDEM é a da IDENTIDADE, nunca a da query** — a lei de determinismo da casa (HR-5).
///
/// **Mutação que deve sangrar:** apagar o `quem.sort_unstable_by_key`.
#[test]
fn duas_maquinas_anunciam_na_ordem_da_identidade() {
    // ⚠️⚠️ **TRÊS e não duas, e a fixtura foi CORRIGIDA por uma mutação que sobreviveu:** com
    // duas, semeadas ao contrário, `reverse()` devolve **exactamente** a ordem certa — a fixtura
    // não distinguia *«ordenado»* de *«ao contrário da criação»*, e a mutação que apaga o `sort`
    // passava. Com três em ordem arbitrária (`5, 1, 9`), ordenar dá `1,5,9` e reverter dá `9,1,5`.
    //
    // *Uma fixtura que não separa a lei da sua coincidência não testa a lei.*
    let mut sim = SimWorld::new();
    for (nome, id, sinal) in [
        ("M", 5_u64, "m_entrou"),
        ("A", 1, "a_entrou"),
        ("Z", 9, "z_entrou"),
    ] {
        let mut m = maquina();
        m.states[0].on_enter = sinal.into();
        sim.world_mut().spawn((Name::new(nome), StableId(id), m));
    }
    let anunciados = advance_machines(&mut sim, &[]);
    let nomes: Vec<&str> = anunciados.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        nomes,
        vec!["a_entrou", "m_entrou", "z_entrou"],
        "o `StableId` manda, venha a entidade de onde vier"
    );
}

/// ⚠️ **Uma cena sem cérebros não paga nada** — e não aloca.
#[test]
fn uma_cena_sem_maquinas_devolve_nada() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((Name::new("Caixa"), StableId(1)));
    assert!(advance_machines(&mut sim, &["botao"]).is_empty());
}
