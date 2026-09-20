//! ⭐⭐⭐ **A PERGUNTA DE ANTES DA 1.ª LINHA da MUNIÇÃO DE RESERVA** (item aberto do handoff da
//! ARMA): *a composição de hoje já a exprime?*
//!
//! Um pente é um [`Counter`] e a recarga repõe-no ao `start`. A reserva é *«e de onde vêm essas
//! balas?»* — um segundo depósito de que a recarga TIRA, e que acaba.
//!
//! # ⚠️ O que se mede, e porquê é esta a pergunta certa
//!
//! Os verbos da tabela do #5 sabem **somar um delta fixo** a um contador (`AddToCounter`), e a
//! [`ph2d_ecs::CounterWatch`] sabe **falar quando um número cruza um limiar**. O que nenhum dos
//! dois sabe é ***tirar o que falta, até ao que há*** — e é exactamente isso que uma reserva é.
//!
//! ⇒ a sonda deixa uma arma recarregar sem parar e conta quantas balas saíram: se a composição
//! exprimisse a reserva, esse número teria um TECTO.
//!
//! ⚠️ **Sonda, não gate.** Ela corre com `--ignored` e IMPRIME.

use ph2d_ecs::{Counter, CounterRuntime, SimWorld, WeaponFire, WeaponRuntime, weapon};

const DT_US: u64 = 16_667;

/// Uma arma com pente de `pente` e uma recarga de `recarga_ms`.
fn arma(sim: &mut SimWorld, pente: i64, recarga_ms: u64) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            WeaponFire {
                on_signal: "fogo".into(),
                cooldown_ms: 0,
                ammo_counter: "pente".into(),
                reload_ms: recarga_ms,
                on_fire: "saiu".into(),
                on_empty: "seca".into(),
                ..WeaponFire::default()
            },
            weapon::born(),
            Counter {
                name: "pente".into(),
                start: pente,
                ..Counter::default()
            },
            CounterRuntime { value: pente },
        ))
        .id()
}

/// Puxa o gatilho `tiques` vezes e devolve quantas balas saíram.
fn quantas_saem(sim: &mut SimWorld, e: ph2d_ecs::Entity, tiques: u32) -> u32 {
    let mut saidas = 0;
    for _ in 0..tiques {
        let (cfg, mun) = {
            let w = sim.world();
            let cfg = w.get::<WeaponFire>(e).unwrap().clone();
            let c = w.get::<Counter>(e).unwrap();
            let rt = w.get::<CounterRuntime>(e).unwrap();
            (
                cfg,
                weapon::Municao {
                    tem: rt.value,
                    cheio: c.start,
                    existe: true,
                    ..weapon::Municao::default()
                },
            )
        };
        let mut st = *sim.world().get::<WeaponRuntime>(e).unwrap();
        let tiro = weapon::avanca(&cfg, &mut st, mun, DT_US, true, false);
        sim.world_mut().entity_mut(e).insert(st);
        sim.world_mut().get_mut::<CounterRuntime>(e).unwrap().value = tiro.municao;
        if tiro.disparou {
            saidas += 1;
        }
    }
    saidas
}

#[test]
#[ignore = "sonda: imprime a medicao que decide se a wave se constroi"]
fn mede_o_que_a_composicao_ja_da_a_reserva() {
    println!("== A) uma arma com pente de 5 e recarga de 100 ms, gatilho segurado 10 s ==");
    let mut sim = SimWorld::new();
    let e = arma(&mut sim, 5, 100);
    let saidas = quantas_saem(&mut sim, e, 600);
    let restante = sim.world().get::<CounterRuntime>(e).unwrap().value;
    println!(
        "   balas que sairam: {saidas} · pente no fim: {restante}\n   \
         => a recarga repoe o pente ao `start` sem tirar de lado nenhum: o deposito e' INFINITO."
    );

    println!("\n== B) e a composicao consegue limita-lo? ==");
    println!(
        "   o `AddToCounter` soma um DELTA FIXO e a `CounterWatch` fala quando um numero cruza\n   \
         um limiar. Nenhum dos dois sabe «tirar O QUE FALTA, ate' ao que HA'» — que e' a lei\n   \
         inteira de uma reserva. Uma tabela com `AddToCounter(-5)` tiraria 5 mesmo com 2 no\n   \
         deposito, e deixava-o NEGATIVO."
    );

    println!("\n== veredito ==");
    println!(
        "   a composicao NAO exprime a reserva: falta-lhe o MINIMO entre o que falta e o que ha'.\n   \
         => a wave constroi-se, e o que ela acrescenta e' UM campo mais essa transferencia na\n   \
         lei — nunca um segundo motor de contagem."
    );
}
