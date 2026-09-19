//! Os gates do [`super`] — a costura entre o relógio e a lei, medida no MUNDO.

use super::*;
use crate::{Timer, TimerRuntime, Timers, World};
use ph2d_tween::{AoAcabar, Canal};

/// Um objecto com `n` timers de um segundo (o primeiro a correr) e os tweens dados.
fn cena(tweens: Vec<Tween>, n_timers: usize) -> (World, bevy_ecs::entity::Entity) {
    let mut w = World::new();
    let cfg = Timers(
        (0..n_timers)
            .map(|_| Timer {
                duration_us: 1_000_000,
                ..Timer::default()
            })
            .collect(),
    );
    let rt = TimerRuntime(cfg.0.iter().map(crate::timer::born).collect());
    let e = w.spawn((Tweens(tweens), cfg, rt)).id();
    (w, e)
}

fn fade() -> Tween {
    Tween::linear(Canal::Opacity, 1.0, 0.0)
}

/// **O tween corre no relógio, e a meio do período escreve metade.**
#[test]
fn o_tween_corre_no_relogio_do_mesmo_indice() {
    let (mut w, e) = cena(vec![fade()], 1);
    // Meio segundo.
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].elapsed_us = 500_000;
    }
    let escritas = a_escrever(&mut w);
    assert_eq!(escritas.len(), 1);
    assert_eq!(escritas[0].entity, e);
    assert_eq!(escritas[0].canal, Canal::Opacity);
    assert!((escritas[0].valor[0] - 0.5).abs() < 1e-6);
}

/// ⭐⭐⭐ **O ÍNDICE liga os dois, e o gate mede-o com os relógios em instantes DIFERENTES** —
/// *dois números iguais não distinguem duas leis.*
#[test]
fn o_indice_e_que_liga_o_tween_ao_timer() {
    let (mut w, e) = cena(vec![fade(), Tween::linear(Canal::ScaleX, 1.0, 2.0)], 2);
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].elapsed_us = 250_000; // o fade a 25 %
        rt.0[1].running = true;
        rt.0[1].elapsed_us = 750_000; // a escala a 75 %
    }
    let escritas = a_escrever(&mut w);
    assert_eq!(escritas.len(), 2);
    let op = escritas.iter().find(|x| x.canal == Canal::Opacity).unwrap();
    let sx = escritas.iter().find(|x| x.canal == Canal::ScaleX).unwrap();
    assert!(
        (op.valor[0] - 0.75).abs() < 1e-6,
        "o fade leu {}",
        op.valor[0]
    );
    assert!(
        (sx.valor[0] - 1.75).abs() < 1e-6,
        "a escala leu {}",
        sx.valor[0]
    );
}

/// ⚠️ **Um tween SEM timer no mesmo índice sai da lista** — e ⛔ *não* cai no timer `0`: correr no
/// relógio errado lê-se como um defeito do motor.
#[test]
fn um_tween_sem_relogio_sai_da_lista() {
    let (mut w, e) = cena(vec![fade(), fade()], 1);
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].elapsed_us = 500_000;
    }
    let escritas = a_escrever(&mut w);
    assert_eq!(
        escritas.len(),
        1,
        "o segundo tween nao tem relogio e escreveu na mesma"
    );
}

/// **Parado = ausente da lista** — o que faz o objecto voltar à cena sem uma linha a repô-lo.
#[test]
fn parado_e_ausente_da_lista() {
    let (mut w, e) = cena(vec![fade()], 1);
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].running = false;
        rt.0[0].elapsed_us = 0;
    }
    assert!(a_escrever(&mut w).is_empty());
}

/// ⭐⭐⭐ **REBOBINAR JÁ FUNCIONA, e sem uma linha nova** — o tween é função pura do relógio, e o
/// [`crate::rewind_runtime`] já repõe os timers pela porta `born`.
///
/// ⚠️ **O CONTROLO é metade do gate:** sem a primeira asserção, um tween que nunca tivesse escrito
/// nada passaria por vacuidade.
#[test]
fn rebobinar_ja_funciona_porque_o_tween_nao_guarda_nada() {
    let (mut w, e) = cena(vec![fade()], 1);
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].elapsed_us = 900_000;
    }
    assert_eq!(
        a_escrever(&mut w).len(),
        1,
        "controlo: ele estava a escrever"
    );

    crate::rewind_runtime::rewind_runtime_state(&mut w);
    // O timer de fabrica tem `autostart`, logo renasce A CORRER e no zero.
    let escritas = a_escrever(&mut w);
    assert!(
        escritas.is_empty() || (escritas[0].valor[0] - 1.0).abs() < 1e-6,
        "renascer tem de devolver o inicio, leu {escritas:?}"
    );
}

/// ⭐ **O fim com `Hold` continua a escrever; com `Rewind` cala-se** — as duas metades, pela porta
/// do produto e com o [`crate::TimerState::finished`] a vir de um `advance` REAL.
#[test]
fn o_fim_tem_duas_respostas_pela_porta_do_produto() {
    for (ao_acabar, espera) in [(AoAcabar::Hold, 1), (AoAcabar::Rewind, 0)] {
        let t = Tween {
            ao_acabar,
            ..fade()
        };
        let (mut w, e) = cena(vec![t], 1);
        {
            let cfg = w.get::<Timers>(e).unwrap().0[0].clone();
            let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
            let out = crate::timer::advance(&cfg, &mut rt.0[0], 1_000_000);
            assert!(out.finished, "controlo: o relogio TEM de ter acabado");
        }
        let escritas = a_escrever(&mut w);
        assert_eq!(
            escritas.len(),
            espera,
            "{ao_acabar:?} escreveu o numero errado"
        );
        if espera == 1 {
            assert!(escritas[0].valor[0].abs() < 1e-6, "o `Hold` ficou no meio");
        }
    }
}

/// ⚠️ **O tecto é DERIVADO do dos timers**, e não escolhido: um tween depois do último timer não
/// tem relógio, logo é inerte.
#[test]
fn o_tecto_e_derivado_do_dos_timers() {
    assert_eq!(TWEENS_MAX, crate::TIMERS_MAX);
}

/// ⭐ **Um tween no índice `0` PARTILHA o relógio com um `SequencePlayer`, e isso é a leitura
/// certa** — as duas coisas correm juntas, que é o que alguém que as ponha no mesmo objecto quer.
///
/// ⚠️ O gate existe para a decisão ser VISÍVEL: quem a mudar tem de o reprovar.
#[test]
fn um_tween_e_uma_cutscene_no_mesmo_objecto_partilham_o_relogio() {
    let (mut w, e) = cena(vec![fade()], 1);
    w.entity_mut(e).insert(crate::SequencePlayer {
        container: "Porta".into(),
    });
    {
        let mut rt = w.get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].elapsed_us = 500_000;
    }
    // O tween le' o timer 0…
    let escritas = a_escrever(&mut w);
    assert!((escritas[0].valor[0] - 0.5).abs() < 1e-6);
    // …e a cutscene TAMBE'M, no mesmo instante.
    let corridas = crate::sequence::em_corrida(&mut w, &["Porta"]);
    assert_eq!(corridas.len(), 1);
    assert!(
        (corridas[0].t - 0.5).abs() < 1e-6,
        "a cutscene leu outro instante: {}",
        corridas[0].t
    );
}
