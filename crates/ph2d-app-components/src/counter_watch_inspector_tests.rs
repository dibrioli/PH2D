//! ⭐⭐⭐ **A IDA e a VOLTA do ÂMBITO da vigia** (2026-09-20).
//!
//! ⚠️⚠️ **Este ficheiro nasceu de uma MUTAÇÃO SOBREVIVENTE.** O gate de costura do painel alimenta
//! o instantâneo **à mão**, logo ele entra ABAIXO da rotura: com o `build_info` a cravar
//! `scope_own: false` ele ficava **verde**. *Um gate que monta o instantâneo não pode afirmar nada
//! sobre quem o constrói.*

use ph2d_ecs::{Compare, CounterScope, CounterWatch, CounterWatchRow, SimWorld};
use ph2d_editor_core::counter_watch_edits::CounterWatchFieldEdit as E;

use super::{apply_all, build_info};

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn(CounterWatch(vec![CounterWatchRow {
            counter: "vida".into(),
            compare: Compare::AtMost,
            value: 0,
            signal: "morri".into(),
            once: false,
            scope: CounterScope::World,
        }]))
        .id();
    (sim, e.to_bits())
}

fn scope(sim: &SimWorld, bits: u64) -> CounterScope {
    sim.world()
        .get::<CounterWatch>(ph2d_ecs::Entity::from_bits(bits))
        .expect("a vigia")
        .0[0]
        .scope
}

/// ⭐⭐ **A edição chega ao componente, e o instantâneo lê-a de volta.**
///
/// **Mutação que deve sangrar:** o `build_info` a cravar `scope_own: false` (a VOLTA); ou o braço
/// `E::Scope` do `apply` a não escrever (a IDA).
#[test]
fn o_ambito_da_vigia_faz_a_ida_e_a_volta() {
    let (mut sim, bits) = cena();

    // CONTROLO: ele nasce na cena inteira, dos dois lados.
    let antes = build_info(&sim, bits, false, 1).expect("info");
    assert!(
        !antes.rows[0].scope_own,
        "CONTROLO: nasce no ambito da cena"
    );
    assert_eq!(scope(&sim, bits), CounterScope::World);

    assert!(apply_all(&mut sim, &[(bits, E::Scope(0, true))]));
    assert_eq!(scope(&sim, bits), CounterScope::Own, "IDA");
    let depois = build_info(&sim, bits, false, 1).expect("info");
    assert!(depois.rows[0].scope_own, "VOLTA");

    // E desmarcar volta atrás — senão a caixa é um interruptor de um sentido só.
    assert!(apply_all(&mut sim, &[(bits, E::Scope(0, false))]));
    assert_eq!(scope(&sim, bits), CounterScope::World);
}

/// ⚠️ **Uma edição que não muda nada devolve `false`** — a lei que esta secção já aplica a todos os
/// campos, e sem a qual cada clique inerte viraria um passo de `Ctrl+Z`.
#[test]
fn escrever_o_mesmo_ambito_e_inerte() {
    let (mut sim, bits) = cena();
    assert!(
        !apply_all(&mut sim, &[(bits, E::Scope(0, false))]),
        "ja' estava em World: a edicao tem de ser inerte"
    );
}
