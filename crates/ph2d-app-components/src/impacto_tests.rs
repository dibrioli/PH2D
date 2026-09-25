//! Gates do IMPACTO (plano 28, W5) — a pausa no golpe medida no NOSSO relógio, o pedido que a
//! decide, e os números de dano.

use super::*;
use ph2d_core::{FixedStep, Vec2};
use ph2d_ecs::Transform;
use ph2d_physics_ecs::{Damage, Health, HealthEvent, HealthEventKind};

/// Quantos tiques do passo fixo uma pausa de `pausa_s` COME, com o jogo a correr a `fps`.
///
/// ⚠️ **A corrida acaba MEIO TIQUE depois de dois segundos**, com um último quadro dessa duração:
/// assim nenhuma das duas contagens cai numa fronteira de tique, e a diferença mede a pausa e não
/// um arredondamento. ⛔ Sem isto, a `30 fps` cada quadro é EXACTAMENTE dois tiques, as contagens
/// caem todas em fronteiras e o erro de `f64` come um tique a mais (medido: `4` onde a lei dá `3`).
fn tiques_comidos(fps: f64, pausa_s: f64) -> u64 {
    let dt = 1.0 / fps;
    let meio_tique = 0.5 * FixedStep::default().fixed_dt();
    let quadros = (2.0 * fps).round() as usize;
    let corre = |pede: bool| {
        let mut fs = FixedStep::default();
        let mut imp = ImpactoState::default();
        let mut total = 0_u64;
        for q in 0..quadros {
            if pede && q == 10 {
                imp.pausa.pede(pausa_s);
            }
            let w = imp.retem(dt, true);
            total += u64::from(fs.advance(w).ticks);
        }
        total + u64::from(fs.advance(imp.retem(meio_tique, true)).ticks)
    };
    corre(false) - corre(true)
}

/// ⭐⭐⭐ **Os três números da pesquisa, medidos no NOSSO relógio** (a abertura da W5): o golpe
/// comum `~0,05 s`, o flash `~0,1 s`, o golpe final `~0,15 s` ⇒ **3 / 6 / 9 tiques** do passo fixo de
/// 60 Hz, a QUALQUER cadência de quadros — ⭐ porque a pausa é tempo de parede retido antes do
/// acumulador, e não um número de quadros (a `30`, a `60` e a `144` fps come o mesmo tempo de jogo).
///
/// **Mutação que deve sangrar:** o `retem` devolver o `wall_dt` sem consumir.
#[test]
fn a_pausa_come_o_mesmo_tempo_de_jogo_a_qualquer_cadencia() {
    let hz = 1.0 / FixedStep::default().fixed_dt();
    assert!(
        (hz - 60.0).abs() < 1e-9,
        "o passo fixo da casa mudou: {hz} Hz"
    );
    for fps in [30.0, 60.0, 144.0] {
        for (pausa, esperado) in [(0.05, 3), (0.1, 6), (0.15, 9)] {
            assert_eq!(
                tiques_comidos(fps, pausa),
                esperado,
                "a {fps} fps uma pausa de {pausa} s tinha de comer {esperado} tiques"
            );
        }
    }
}

/// ⚠️ **Com o relógio PARADO não há jogo a congelar** — a pausa não consome, e o passo manual
/// recebe o tempo inteiro. O CONTROLO da mesma função a correr come o tempo.
#[test]
fn com_o_relogio_parado_a_pausa_nao_come_nada() {
    let mut imp = ImpactoState::default();
    imp.pausa.pede(0.1);
    assert_eq!(imp.retem(0.02, false), 0.02);
    assert!(
        (imp.pausa.resta_s() - 0.1).abs() < 1e-12,
        "parado, a pausa não anda"
    );
    assert_eq!(imp.retem(0.02, true), 0.0, "a correr, ela come o quadro");
}

fn mundo_com(dano_s: f32, morte_s: f32) -> (World, Entity, Entity, Entity) {
    let mut w = World::new();
    let espada = w
        .spawn(Damage {
            hitstop_s: dano_s,
            ..Damage::default()
        })
        .id();
    let adaga = w
        .spawn(Damage {
            hitstop_s: dano_s * 0.5,
            ..Damage::default()
        })
        .id();
    let alvo = w
        .spawn(Health {
            death_hitstop_s: morte_s,
            ..Health::default()
        })
        .id();
    (w, espada, adaga, alvo)
}

fn ev(target: Entity, source: Entity, kind: HealthEventKind) -> HealthEvent {
    HealthEvent {
        target,
        source,
        kind,
    }
}

/// ⭐⭐ **O pedido é o MAIOR, e cada peso vem de quem o autorou** — o golpe pela ESPADA, a morte
/// pela VIDA; esquiva e cura não pesam.
///
/// **Mutações que devem sangrar:** somar em vez de `max` · ler o `death_hitstop_s` num golpe comum
/// · deixar a esquiva pesar.
#[test]
fn o_pedido_e_o_maior_e_vem_de_quem_o_autorou() {
    let (w, espada, adaga, alvo) = mundo_com(0.05, 0.15);
    let dmg = HealthEventKind::Damaged { amount: 10.0 };
    let esc = HealthEventKind::Shielded { amount: 10.0 };
    assert!((pausa_pedida(&w, &[ev(alvo, espada, dmg)]) - 0.05).abs() < 1e-7);
    // Dois golpes no mesmo dispatch pesam como o MAIOR, nunca como a soma.
    let dois = [ev(alvo, espada, dmg), ev(alvo, adaga, esc)];
    assert!((pausa_pedida(&w, &dois) - 0.05).abs() < 1e-7);
    // A morte pesa pela VIDA.
    let morte = [
        ev(alvo, espada, dmg),
        ev(alvo, espada, HealthEventKind::Died),
    ];
    assert!((pausa_pedida(&w, &morte) - 0.15).abs() < 1e-7);
    // Esquiva e cura não pesam nada.
    assert_eq!(
        pausa_pedida(&w, &[ev(alvo, espada, HealthEventKind::Dodged)]),
        0.0
    );
    let cura = HealthEventKind::Healed { amount: 5.0 };
    assert_eq!(pausa_pedida(&w, &[ev(alvo, espada, cura)]), 0.0);
}

/// O texto de um número é o dano arredondado, e **nunca `0`** (um golpe que entrou e diz zero
/// lê-se como «não entrou»).
#[test]
fn o_numero_diz_o_dano_arredondado_e_nunca_zero() {
    assert_eq!(texto_do_dano(12.4), "12");
    assert_eq!(texto_do_dano(12.5), "13");
    assert_eq!(texto_do_dano(0.3), "1");
}

fn sim_com_alvo(numeros: bool) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(2.0, 1.0)),
            Health {
                numbers: numeros,
                numbers_size: 0.5,
                ..Health::default()
            },
        ))
        .id();
    (sim, e)
}

/// ⭐⭐⭐ **Um golpe que ENTRA faz nascer o número por cima do alvo, e ele sobe, desvanece e sai.**
///
/// **Mutações que devem sangrar:** ignorar o `numbers` · nascer no centro em vez de por cima ·
/// o `anda` não esquecer os mortos.
#[test]
fn um_golpe_que_entra_faz_nascer_o_numero_que_sobe_e_sai() {
    let (sim, alvo) = sim_com_alvo(true);
    let mut imp = ImpactoState::default();
    imp.ouve(
        &sim,
        &[
            ev(alvo, alvo, HealthEventKind::Damaged { amount: 17.2 }),
            ev(alvo, alvo, HealthEventKind::Dodged),
        ],
    );
    assert_eq!(imp.numeros().len(), 1, "UM golpe entrou, UM número");
    let n = imp.numeros()[0].clone();
    assert_eq!(n.texto, "17");
    assert_eq!(
        n.nasceu,
        [2.0, 1.5],
        "nasce uma altura dele ACIMA do centro"
    );
    // Sobe sem parar e desvanece só no fim.
    imp.anda(0.2);
    let a = imp.numeros()[0].clone();
    imp.anda(0.3);
    let b = imp.numeros()[0].clone();
    assert!(b.onde()[1] > a.onde()[1] && a.onde()[1] > n.onde()[1]);
    assert_eq!(a.alfa(), 1.0, "a meio da vida ainda é opaco");
    imp.anda(0.25);
    assert!(imp.numeros()[0].alfa() < 1.0, "no fim desvanece");
    imp.anda(0.1);
    assert!(imp.numeros().is_empty(), "e sai quando a vida dele acaba");

    // O CONTROLO: a mesma vida com os números DESLIGADOS fica calada.
    let (sim, alvo) = sim_com_alvo(false);
    let mut imp = ImpactoState::default();
    imp.ouve(
        &sim,
        &[ev(alvo, alvo, HealthEventKind::Damaged { amount: 17.2 })],
    );
    assert!(imp.numeros().is_empty());
}

/// ⭐ **Rebobinar é renascer** — a pausa acaba e os números saem.
#[test]
fn rebobinar_acaba_a_pausa_e_tira_os_numeros() {
    let (sim, alvo) = sim_com_alvo(true);
    let mut imp = ImpactoState::default();
    imp.pausa.pede(0.1);
    imp.ouve(
        &sim,
        &[ev(alvo, alvo, HealthEventKind::Damaged { amount: 5.0 })],
    );
    assert_eq!(imp.rewind(), 2);
    assert!(imp.numeros().is_empty());
    assert_eq!(imp.pausa.resta_s(), 0.0);
}
