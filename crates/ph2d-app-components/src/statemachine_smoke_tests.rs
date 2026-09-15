//! Gates da cena de smoke do cérebro — ela tem de PRODUZIR o sujeito que o roteiro promete.

use super::*;
use ph2d_ecs::{SimWorld, StateMachineRuntime};

fn cena() -> SimWorld {
    let mut sim = SimWorld::new();
    assert_eq!(montar(sim.world_mut(), 1), 1);
    sim
}

fn por_nome(sim: &SimWorld, nome: &str) -> ph2d_ecs::Entity {
    let world = sim.world();
    let mut q = world
        .try_query::<(ph2d_ecs::Entity, &Name)>()
        .expect("query");
    for (e, n) in q.iter(world) {
        if n.as_str() == nome {
            return e;
        }
    }
    panic!("a cena tem de ter «{nome}»");
}

/// ⭐⭐⭐ **A cena produz o SUJEITO: uma porta com cérebro e outra sem.**
///
/// ⚠️ Sem as duas, a cena ensina metade — e a metade que falta é a que torna a outra legível.
#[test]
fn a_cena_traz_a_porta_e_o_controlo() {
    let sim = cena();
    let esq = por_nome(&sim, "Door");
    let dir = por_nome(&sim, "Door (no brain)");
    assert!(
        sim.world().get::<StateMachine>(esq).is_some(),
        "a da esquerda TEM cerebro"
    );
    assert!(
        sim.world().get::<StateMachine>(dir).is_none(),
        "o CONTROLO nao tem cerebro — e' isso que ele controla"
    );
    // ⚠️ **E as duas tabelas têm o MESMO tamanho**: um controlo com menos acções mediria outra
    // coisa (*«a da direita faz menos»*), e não o que um ESTADO compra.
    let n_esq = sim
        .world()
        .get::<SignalActions>(esq)
        .expect("tabela")
        .0
        .len();
    let n_dir = sim
        .world()
        .get::<SignalActions>(dir)
        .expect("tabela")
        .0
        .len();
    assert_eq!(n_esq, n_dir, "as duas tabelas tem de ter as MESMAS accoes");
}

/// ⭐⭐⭐ **O ciclo anda UM passo por toque** — o que o roteiro promete ao dono.
///
/// **Mutação que deve sangrar:** apagar o `gasto[i] = true` da lei (o ciclo daria a volta inteira
/// num tique, e a porta ficaria sempre na mesma cor).
#[test]
fn um_toque_do_botao_da_um_passo_no_ciclo() {
    let mut sim = cena();
    let porta = por_nome(&sim, "Door");
    let cfg = sim
        .world()
        .get::<StateMachine>(porta)
        .expect("cerebro")
        .clone();
    let mut rt = ph2d_ecs::state_machine::born(&cfg);
    ph2d_ecs::state_machine::advance(&cfg, &mut rt, &[]); // a entrada no inicial

    for esperado in [1_u8, 2, 0, 1] {
        let a = ph2d_ecs::state_machine::advance(&cfg, &mut rt, &["botao"]);
        assert_eq!(a.steps, 1, "um toque = um passo");
        assert_eq!(rt.current, esperado);
    }
    let _ = sim.world_mut();
}

/// ⭐⭐ **Cada estado acende uma placa DIFERENTE** — três nomes, três linhas.
///
/// ⚠️ Sem nomes distintos a máquina andava e o ecrã não mudava: o artista leria *«não faz nada»*
/// sobre um cérebro que funciona.
#[test]
fn os_tres_estados_anunciam_nomes_diferentes() {
    let sim = cena();
    let porta = por_nome(&sim, "Door");
    let m = sim.world().get::<StateMachine>(porta).expect("cerebro");
    let mut nomes: Vec<&str> = m.states.iter().map(|s| s.on_enter.as_str()).collect();
    assert_eq!(nomes.len(), 3);
    nomes.sort_unstable();
    nomes.dedup();
    assert_eq!(nomes.len(), 3, "tres nomes DISTINTOS");
    assert!(
        nomes.iter().all(|n| !n.is_empty()),
        "um nome vazio e' um estado CALADO — a placa dele nunca acenderia"
    );
}

/// ⚠️ **O botão é um RELÓGIO que repete** — sem `repeat` a cena dava um passo e parava, e o dono
/// leria *«travou»*.
#[test]
fn o_botao_repete_e_arranca_sozinho() {
    let sim = cena();
    let b = por_nome(&sim, "Button");
    let t = &sim.world().get::<Timers>(b).expect("o relogio").0[0];
    assert!(t.repeat, "sem repetir, a cena da' um passo e para");
    assert!(
        t.autostart,
        "sem autostart, nada acontece ate' alguem mexer"
    );
    assert_eq!(t.signal, "botao", "e' este o nome que as setas ouvem");
}

/// ⭐ **E a máquina de facto responde ao nome que o relógio publica** — a costura inteira, sem a
/// qual os dois lados podem estar certos e não se conhecerem.
#[test]
fn o_nome_que_o_relogio_publica_e_o_que_as_setas_ouvem() {
    let sim = cena();
    let b = por_nome(&sim, "Button");
    let sinal = sim.world().get::<Timers>(b).expect("o relogio").0[0]
        .signal
        .clone();
    let porta = por_nome(&sim, "Door");
    let m = sim.world().get::<StateMachine>(porta).expect("cerebro");
    assert!(
        m.transitions.iter().all(|t| t.on == sinal),
        "as tres setas tem de ouvir «{sinal}»"
    );
    // E o vivo ainda não existe — ele nasce na ponte, no primeiro avanço.
    assert!(sim.world().get::<StateMachineRuntime>(porta).is_none());
}

/// ⚠️ **O roteador nunca devolve acima do tecto que ele declara.**
///
/// ⭐ É este gate que mantém o [`CENAS`] honesto agora que o `montar` não tem `match` (uma cena só,
/// e o `clippy` recusa um braço único). *O número conta-se do produto, nunca de uma nota.*
///
/// **Mutação que deve sangrar:** `CENAS = 2` sem uma segunda cena — o roteador ofereceria um nível
/// que devolve a `=1`, e o artista leria *«a `=2` está partida»*.
#[test]
fn o_roteador_nunca_devolve_acima_do_tecto() {
    let mut vistos = std::collections::BTreeSet::new();
    for nivel in 0..=5u32 {
        let mut sim = SimWorld::new();
        let m = montar(sim.world_mut(), nivel);
        assert!(
            m >= 1 && m <= CENAS,
            "nivel {nivel} devolveu {m}, tecto {CENAS}"
        );
        vistos.insert(m);
    }
    assert_eq!(
        vistos.len() as u32,
        CENAS,
        "o roteador tem de ALCANCAR todas as cenas que declara: {vistos:?}"
    );
}
