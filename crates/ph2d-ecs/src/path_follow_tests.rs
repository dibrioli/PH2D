//! Os gates do [`super`] — a costura entre o relógio e a fracção do percurso, medida no MUNDO.
//!
//! ⚠️ **Nenhum destes gates conhece uma curva**, e é isso que se está a afirmar: a fundação
//! responde *«em que fracção do percurso este objecto está»* e nada mais. A geometria é da ponte.

use super::*;
use crate::{Timer, TimerRuntime, Timers, World};

/// Um objecto com `n` relógios de um segundo (todos a correr) e o seguidor dado.
fn cena(pf: PathFollow, n_timers: usize) -> (World, bevy_ecs::entity::Entity) {
    let mut w = World::new();
    let cfg = Timers(
        (0..n_timers)
            .map(|_| Timer {
                duration_us: 1_000_000,
                autostart: true,
                ..Timer::default()
            })
            .collect(),
    );
    let rt = TimerRuntime(cfg.0.iter().map(crate::timer::born).collect());
    let e = w.spawn((pf, cfg, rt)).id();
    (w, e)
}

fn trilho() -> PathFollow {
    PathFollow {
        caminho: "Trilho".into(),
        ..PathFollow::default()
    }
}

fn adianta(w: &mut World, e: bevy_ecs::entity::Entity, i: usize, us: u64) {
    let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
    rt.0[i].elapsed_us = us;
}

/// **Meio período, meio percurso.**
#[test]
fn a_fraccao_e_o_progresso_do_relogio() {
    let (mut w, e) = cena(trilho(), 1);
    adianta(&mut w, e, 0, 500_000);
    let pedidos = a_seguir(&mut w);
    assert_eq!(pedidos.len(), 1);
    assert_eq!(pedidos[0].entity, e);
    assert_eq!(pedidos[0].caminho, "Trilho");
    assert!((pedidos[0].fraccao - 0.5).abs() < 1e-9, "{:?}", pedidos[0]);
}

/// ⭐⭐⭐ **O DESLOCAMENTO dá a volta** — a lei que o `motion.path` desta casa já declara.
///
/// ⚠️ A fixtura usa `0,75 + 0,5 = 1,25`, que **tem** de dar `0,25`: um `clamp` leria `1,0` e o
/// objecto ficaria pregado no fim da pista, que é exactamente o defeito que o `rem_euclid` evita.
#[test]
fn o_deslocamento_da_a_volta() {
    let (mut w, e) = cena(
        PathFollow {
            deslocamento: 0.75,
            ..trilho()
        },
        1,
    );
    adianta(&mut w, e, 0, 500_000);
    let p = a_seguir(&mut w);
    assert!((p[0].fraccao - 0.25).abs() < 1e-9, "{:?}", p[0]);
}

/// ⚠️ **Deslocamento `0` é BYTE-IDÊNTICO ao andamento cru** — sem isto, o campo novo mudaria toda
/// cena que já existe.
#[test]
fn o_deslocamento_zero_nao_mexe_num_bit() {
    let (mut w, e) = cena(trilho(), 1);
    for us in [0_u64, 1, 333_333, 500_000, 999_999] {
        adianta(&mut w, e, 0, us);
        let p = a_seguir(&mut w);
        let cru = ph2d_tween::andamento(
            Relogio {
                a_correr: true,
                acabou: false,
                progresso: (us as f64 / 1_000_000.0) as f32,
            },
            Ciclo::Reinicia,
            Easing::LINEAR,
            AoAcabar::Hold,
        )
        .unwrap();
        assert_eq!(p[0].fraccao.to_bits(), cru.to_bits(), "em {us} µs");
    }
}

/// ⭐⭐⭐ **O VAI-E-VOLTA é a dobra da W8**, e a fixtura mede-a a TRÊS QUARTOS do período — onde a
/// ida e a volta se distinguem (a meio elas coincidem no topo, e no zero coincidem no princípio).
#[test]
fn o_ciclo_ping_pong_volta_pela_mesma_curva() {
    let (mut w, e) = cena(
        PathFollow {
            ciclo: Ciclo::PingPong,
            ..trilho()
        },
        1,
    );
    adianta(&mut w, e, 0, 250_000);
    let ida = a_seguir(&mut w)[0].fraccao;
    adianta(&mut w, e, 0, 750_000);
    let volta = a_seguir(&mut w)[0].fraccao;
    assert!((ida - 0.5).abs() < 1e-9, "ida {ida}");
    assert!((volta - 0.5).abs() < 1e-9, "volta {volta}");
    adianta(&mut w, e, 0, 500_000);
    let topo = a_seguir(&mut w)[0].fraccao;
    assert!((topo - 1.0).abs() < 1e-9, "topo {topo}");
}

/// ⛔⛔⛔ **O FIM DA PISTA É O FIM DA PISTA** — o gate que apanhou o `rem_euclid` cru.
///
/// Um *one-shot* que acaba com [`AoAcabar::Hold`] entrega andamento `1,0`, e `1,0.rem_euclid(1,0)`
/// é **`0,0`**: o objecto **teletransportava-se para o princípio** em vez de descansar na ponta.
/// *Uma volta que apanha o próprio fim não é uma volta, é um salto.*
#[test]
fn quem_acaba_descansa_no_fim_e_nao_salta_para_o_principio() {
    let (mut w, e) = cena(trilho(), 1);
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].running = false;
        rt.0[0].finished = true;
        rt.0[0].elapsed_us = 0;
    }
    let p = a_seguir(&mut w);
    assert_eq!(p.len(), 1);
    assert!(
        (p[0].fraccao - 1.0).abs() < 1e-12,
        "acabou em {}",
        p[0].fraccao
    );
}

/// ⭐⭐⭐ **O ÍNDICE liga o seguidor ao relógio, e o gate mede-o com DOIS relógios em instantes
/// DIFERENTES.**
///
/// ⚠️⚠️ **É a lição que esta linha pagou TRÊS vezes** (a W6, a W9 e a W10 do suplente #22): *uma
/// fixtura com um elemento não pode testar um índice* — com um só, `get(i)` e `first()` devolvem a
/// mesma coisa e a mutação que troca `i` por `0` passa VERDE.
#[test]
fn o_indice_e_que_liga_o_seguidor_ao_timer() {
    let (mut w, e) = cena(
        PathFollow {
            relogio: 1,
            ..trilho()
        },
        2,
    );
    adianta(&mut w, e, 0, 250_000);
    adianta(&mut w, e, 1, 750_000);
    let p = a_seguir(&mut w);
    assert_eq!(p.len(), 1);
    assert!(
        (p[0].fraccao - 0.75).abs() < 1e-9,
        "leu o relógio {} em vez do 1",
        p[0].fraccao
    );
}

/// ⛔ **Um seguidor apontado a um índice sem relógio é INERTE** — e ⛔ **não** cai no relógio `0`:
/// correr no relógio errado lê-se como um defeito do motor, e não como uma lista curta.
#[test]
fn um_seguidor_sem_relogio_no_indice_dele_sai_da_lista() {
    let (mut w, e) = cena(
        PathFollow {
            relogio: 3,
            ..trilho()
        },
        1,
    );
    adianta(&mut w, e, 0, 500_000);
    assert!(a_seguir(&mut w).is_empty());
}

/// ⛔ **Sem NOME não há curva**, e a lista não o carrega: deixá-lo entrar faria a ponte pagar uma
/// busca por nome em toda entidade acabada de anexar.
#[test]
fn um_caminho_vazio_sai_da_lista() {
    let (mut w, e) = cena(PathFollow::default(), 1);
    adianta(&mut w, e, 0, 500_000);
    assert!(a_seguir(&mut w).is_empty());
    // …e o CONTROLO: com nome, a mesma cena pede.
    w.get_mut::<PathFollow>(e).unwrap().caminho = "Trilho".into();
    assert_eq!(a_seguir(&mut w).len(), 1);
}

/// ⚠️ **Um relógio PARADO no zero não escreve** — é a mesma lei do [`crate::tween`], e é ela que
/// faz o objecto ficar na pose da cena até alguém carregar no play.
#[test]
fn um_relogio_parado_no_zero_nao_pede_nada() {
    let mut w = World::new();
    // ⚠️ **`autostart: false` EXPLÍCITO** — o `Timer::default()` nasce a correr (o `born` honra-o), e
    // sem esta linha a fixtura mediria o contrário do que o nome promete.
    let cfg = Timers(vec![Timer {
        duration_us: 1_000_000,
        autostart: false,
        ..Timer::default()
    }]);
    let rt = TimerRuntime(cfg.0.iter().map(crate::timer::born).collect());
    let e = w.spawn((trilho(), cfg, rt)).id();
    assert!(!w.get::<TimerRuntime>(e).unwrap().0[0].running);
    assert!(a_seguir(&mut w).is_empty());
}

/// ⚠️ **Um seguidor acabado de anexar ALINHA-SE** — um que deslizasse de lado lê-se como partido.
#[test]
fn o_valor_de_fabrica_alinha_com_o_caminho() {
    assert!(PathFollow::default().alinha);
    assert_eq!(PathFollow::default().angulo, 0.0);
}
