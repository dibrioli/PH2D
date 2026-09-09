//! Os gates do [`super`] — a lei pura do `Timer`, medida caso a caso.
//!
//! ⚠️ **Cada gate morde uma recusa NOMEADA no doc do [`super::advance`]**, e o doc do módulo diz
//! qual lei da casa a paga. Se você mexer aqui, refaça as mutações do handoff.

use super::*;

/// Um timer de um segundo, a correr, sem repetição.
fn one_shot() -> (Timer, TimerState) {
    (
        Timer {
            name: "recarga".into(),
            duration_us: 1_000_000,
            signal: "arma_pronta".into(),
            ..Timer::default()
        },
        TimerState {
            elapsed_us: 0,
            running: true,
        },
    )
}

/// **Um tique curto não dispara nada, e o progresso anda.**
#[test]
fn a_short_tick_only_moves_the_progress() {
    let (t, mut s) = one_shot();
    let out = advance(&t, &mut s, 250_000);
    assert_eq!(out, TimerOutcome::default(), "disparou antes da hora");
    let p = t.progress(&s);
    assert!((p - 0.25).abs() < 1e-6, "progresso {p}");
    assert!(s.running, "um tique curto parou o timer");
}

/// **Chegar ao fim dispara UMA vez e PÁRA** — e zera, para que o próximo *start* seja um período
/// inteiro. ⚠️ Deixá-lo cheio faria o disparo seguinte ser imediato.
#[test]
fn a_one_shot_fires_once_stops_and_rewinds() {
    let (t, mut s) = one_shot();
    let out = advance(&t, &mut s, 1_000_000);
    assert_eq!(out.fires, 1);
    assert!(
        out.finished,
        "um one-shot que acaba tem de dizer que acabou"
    );
    assert!(
        !s.running,
        "o one-shot continuou a correr depois de disparar"
    );
    assert_eq!(
        s.elapsed_us, 0,
        "ele ficou cheio — o proximo start dispararia de imediato"
    );
    assert_eq!(t.progress(&s), 0.0);
}

/// ⭐⭐ **Um tique ATRASADO colapsa num evento só, com a contagem dentro.**
///
/// A lei do `cycles` da §11 e do `rows` do Motion: *o colapso é lossy, e o número é o que ele
/// descarta.* ⚠️ **Mutação que deve sangrar:** publicar um `fires` por período em vez de contar.
#[test]
fn a_late_tick_collapses_into_one_event_carrying_the_count() {
    let (base, mut s) = one_shot();
    let t = Timer {
        repeat: true,
        ..base
    };
    // A janela esteve parada 3,5 segundos: três periodos inteiros fecharam.
    let out = advance(&t, &mut s, 3_500_000);
    assert_eq!(
        out.fires, 3,
        "a contagem perdeu-se — dez disparos que ninguem viu sao ruido"
    );
    assert!(!out.finished, "um repetidor nao acaba");
    assert!(s.running);
    assert_eq!(s.elapsed_us, 500_000, "o resto do periodo perdeu-se");
}

/// ⛔ **Duração `0` NUNCA dispara.**
///
/// A alternativa — disparar a cada tique — faria um campo por preencher no painel virar uma rajada
/// de sinais. *Um valor por preencher não pode ser um gesto destrutivo.*
#[test]
fn a_zero_duration_never_fires() {
    let (base, mut s) = one_shot();
    let t = Timer {
        duration_us: 0,
        repeat: true,
        ..base
    };
    for _ in 0..1_000 {
        assert_eq!(
            advance(&t, &mut s, 16_666).fires,
            0,
            "uma duracao de zero disparou"
        );
    }
    assert_eq!(
        t.progress(&s),
        0.0,
        "um timer que nunca dispara nao esta' CHEIO, esta' PARADO"
    );
}

/// ⛔ **Pausado guarda o progresso** — é o *Pause* do Godot, não o *Stop*.
#[test]
fn pausing_keeps_the_progress_and_resuming_continues() {
    let (t, mut s) = one_shot();
    advance(&t, &mut s, 400_000);
    s.running = false;
    advance(&t, &mut s, 5_000_000);
    assert_eq!(s.elapsed_us, 400_000, "uma pausa acumulou tempo");
    s.running = true;
    let out = advance(&t, &mut s, 600_000);
    assert_eq!(out.fires, 1, "retomar nao continuou de onde estava");
}

/// ⛔⛔ **A REDE do laço**: uma duração mínima com um tique enorme não congela o app.
///
/// ⚠️ Ela é uma rede, não uma lei do produto — o doc do [`FIRES_GUARD`] diz de que recurso é.
#[test]
fn a_minimal_period_with_a_huge_tick_does_not_hang() {
    let (base, mut s) = one_shot();
    let t = Timer {
        duration_us: 1,
        repeat: true,
        ..base
    };
    let out = advance(&t, &mut s, u64::MAX / 2);
    assert!(
        out.fires <= FIRES_GUARD,
        "o laco passou a rede: {}",
        out.fires
    );
    assert_eq!(
        s.elapsed_us, 0,
        "a rede deixou o relogio num estado que dispara de novo"
    );
}

/// **O `autostart` é IDEMPOTENTE e não pisa quem já corre.**
///
/// ⚠️ Se reescrevesse por quadro, o diff do undo veria o componente mudar e cada quadro com
/// entrada viraria um passo espúrio — a lei que o `assign_missing_*` já pagou.
#[test]
fn arming_autostart_is_idempotent_and_spares_a_running_timer() {
    let (base, _) = one_shot();
    let ts = Timers(vec![
        Timer {
            autostart: true,
            ..base.clone()
        },
        Timer {
            autostart: false,
            ..base.clone()
        },
        Timer {
            autostart: true,
            ..base
        },
    ]);
    let mut rt = TimerRuntime(vec![
        TimerState::default(),
        TimerState::default(),
        TimerState {
            elapsed_us: 700_000,
            running: true,
        },
    ]);
    arm_autostart(&ts, &mut rt);
    assert!(rt.0[0].running, "o autostart nao armou");
    assert!(!rt.0[1].running, "o autostart armou quem nao pediu");
    assert_eq!(
        rt.0[2].elapsed_us, 700_000,
        "o autostart REZEROU um timer que ja' corria — cada quadro viraria um passo de undo"
    );
    let antes = rt.clone();
    arm_autostart(&ts, &mut rt);
    assert_eq!(rt, antes, "o autostart nao e' idempotente");
}

/// ⚠️ **O nome do TIMER e o nome do SINAL são coisas diferentes** — e o gate existe porque
/// confundi-los obrigaria a renomear o componente para mudar o contrato.
#[test]
fn the_timer_name_and_the_signal_name_are_separate() {
    let (t, _) = one_shot();
    assert_eq!(t.name, "recarga");
    assert_eq!(t.signal, "arma_pronta");
    assert_ne!(t.name, t.signal);
}

/// **Um timer sem nome de sinal é CALADO** — a lei da §11: um produtor sem nome não fala, em vez de
/// falar com um nome vazio. ⚠️ O gate mede o CAMPO; quem o honra é a ponte (gate próprio na shell).
#[test]
fn a_timer_with_no_signal_name_is_mute_by_construction() {
    let (base, _) = one_shot();
    let t = Timer {
        signal: String::new(),
        ..base
    };
    assert!(t.signal.is_empty());
}
