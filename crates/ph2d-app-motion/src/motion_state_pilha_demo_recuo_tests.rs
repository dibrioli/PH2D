//! ⭐⭐⭐ Os gates do DESENHO do mundo de contacto na `=114` (doc 121 §9.20) — pela porta do app
//! (`advance_or_scrub_scoped`, a da ponte): o mundo do rapier é a MEMÓRIA do `sim.step`, e viaja no
//! ponto de recuo; um recuo seguido de Play dá os bits da 1.ª passagem, e a 2.ª volta do `Loop` dá os
//! da 1.ª.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

fn cena() -> (MotionState, Vec<NodeId>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    (state, sinks)
}

/// Leva a cena ao tique `k` pela porta do app (para a frente ou por recuo) e devolve os bits das
/// posições da taça da DIREITA (a que colide).
fn em(state: &mut MotionState, sinks: &[NodeId], k: u64) -> Vec<u32> {
    let scopes = ph2d_node_motion_time_remap::time_scopes(&state.doc.graph, &state.registry);
    state.pump.advance_or_scrub_scoped(
        &state.doc.graph,
        &state.registry,
        sinks,
        k,
        |x| x as f64 / 60.0,
        state.default_uv_rect,
        state.default_size,
        &scopes,
    );
    match state
        .pump
        .cook
        .peek(sinks[1])
        .and_then(|o| o.first())
        .map(|o| o.as_stream().clone())
        .and_then(|s| s.get("P").cloned())
    {
        Some(Column::Vec2(p)) => p
            .iter()
            .flat_map(|x| [x[0].to_bits(), x[1].to_bits()])
            .collect(),
        _ => Vec::new(),
    }
}

/// ⭐⭐⭐ **Um RECUO seguido de Play dá os bits da 1.ª passagem** — sem o mundo no ponto de recuo, a
/// queda de depois do recuo seria OUTRA (o rapier aquece cada contacto com a memória dele).
#[test]
fn a_scrub_back_then_play_gives_the_bits_of_the_first_pass() {
    let (mut state, sinks) = cena();
    let mut primeira = Vec::new();
    for k in 0..=150 {
        let b = em(&mut state, &sinks, k);
        if k >= 100 {
            primeira.push(b);
        }
    }
    assert!(
        state.pump.cook.has_memo(),
        "o sim.step tem de guardar o mundo de contacto"
    );
    assert!(primeira[0].len() >= 40, "a taca da direita tem pecas");
    // O recuo (para um tique que NÃO é ponto de recuo: o anel regista de 8 em 8), e Play outra vez.
    let _ = em(&mut state, &sinks, 99);
    for (i, k) in (100..=150).enumerate() {
        assert_eq!(
            em(&mut state, &sinks, k),
            primeira[i],
            "depois do recuo o tique {k} tem de dar os bits da 1.a passagem"
        );
    }
}

/// ⭐⭐ **Na PAUSA do `Loop` não há mundo de contacto, e a queda seguinte nasce num mundo NOVO** —
/// a zona fica sem peças entre as quedas, e um stream sem colisor apaga a memória do `sim.step`.
///
/// ⛔ A régua de antes («a 2.ª volta fica a `2e-3` da 1.ª») NÃO distinguia o defeito que devia
/// pegar: com o mundo velho forçado a continuar (a mutação `m3`) a diferença era a MESMA
/// (`9,15e-2`), porque o mundo já morria na pausa; e a diferença em si é o relógio `f32` (`3,6 + k/60`
/// contra `k/60`) amplificado pelo caos da pilha — `1,06e-3` com a taça em polilinha, `9,15e-2` em
/// segmentos. O mundo novo ao bit tem gate na crate (`a_stream_that_does_not_continue…`).
#[test]
fn the_loop_pause_drops_the_world_and_the_next_fall_starts_a_new_one() {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos → tiques"
    )]
    let (queda, volta) = (
        (DURACAO * 60.0).round() as u64,
        ((DURACAO + PAUSA) * 60.0).round() as u64,
    );
    let (mut state, sinks) = cena();
    let mut na_pausa = Vec::new();
    for k in 0..=(volta + 30) {
        let _ = em(&mut state, &sinks, k);
        if k == 30 || k == volta + 30 {
            assert!(
                state.pump.cook.has_memo(),
                "a cair (tique {k}) ha' mundo de contacto"
            );
        }
        if k > queda + 5 && k + 5 < volta {
            na_pausa.push(state.pump.cook.has_memo());
        }
    }
    assert!(
        !na_pausa.is_empty() && na_pausa.iter().all(|m| !m),
        "na pausa o mundo some: {na_pausa:?}"
    );
}

/// A sonda da régua retirada acima: a maior diferença entre as duas voltas no tique `150`. ⚠️ A lei de antes
/// (`1,5e-2`) correu aqui atrás de uma chave por fio em `e78b2c096` e saiu com o código dela.
#[test]
#[ignore = "sonda"]
fn probe_a_diferenca_entre_as_voltas() {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos → tiques"
    )]
    let volta = ((DURACAO + PAUSA) * 60.0).round() as u64;
    let (mut state, sinks) = cena();
    let mut uma = Vec::new();
    for k in 0..=(volta + 150) {
        let b = em(&mut state, &sinks, k);
        if k == 150 {
            uma = b;
        } else if k == volta + 150 {
            let d = uma
                .iter()
                .zip(&b)
                .map(|(a, b)| (f32::from_bits(*a) - f32::from_bits(*b)).abs())
                .fold(0.0_f32, f32::max);
            eprintln!("VOLTAS iguais ao bit {} · maior diferenca {d:e}", uma == b);
        }
    }
}
