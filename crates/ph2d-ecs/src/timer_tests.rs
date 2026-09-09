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

/// **Um slot que NASCE é armado; um que já existia não é tocado.**
///
/// ⚠️ É a lei inteira do [`reconcile`] num caso só: o relógio nasce vazio (é assim que uma
/// entidade chega do ficheiro, da paleta ou de uma cópia), e depois de reconciliar o `autostart`
/// está aplicado — e **só** a quem o pediu.
#[test]
fn a_slot_that_is_born_is_armed_and_only_if_it_asked() {
    let (base, _) = one_shot();
    let ts = Timers(vec![
        Timer {
            autostart: true,
            ..base.clone()
        },
        Timer {
            autostart: false,
            ..base
        },
    ]);
    let mut rt = TimerRuntime::default();
    assert!(reconcile(&ts, &mut rt), "reconciliar nao relatou o trabalho");
    assert_eq!(rt.0.len(), 2, "o relogio nao ficou do tamanho da config");
    assert!(
        rt.0[0].running,
        "o slot nasceu por armar — um Timers anexado pela paleta ficaria INERTE para sempre"
    );
    assert!(!rt.0[1].running, "armou quem nao pediu autostart");
    let antes = rt.clone();
    assert!(
        !reconcile(&ts, &mut rt),
        "reconciliar relatou trabalho sem nada por nascer — quem chama marcaria o componente \
         como alterado a cada quadro"
    );
    assert_eq!(rt, antes, "reconciliar nao e' idempotente");
}

/// ⛔⛔ **UM *ONE-SHOT* QUE TERMINOU NÃO RENASCE** — o gate que separa esta lei da anterior.
///
/// A redacção anterior armava *«todo slot que não está a correr»*, e um one-shot terminado põe
/// `running = false` — que é exactamente essa condição. Chamada por quadro (que é o que o produto
/// precisa, senão nada arma), ela fá-lo-ia **disparar para sempre**.
///
/// ⚠️ **Quem defende a propriedade AQUI é a saída antecipada**, não o `skip` — e isso foi medido:
/// com o comprimento parado, `skip(nascidos)` e `if !s.running` concordam, porque a função nem
/// chega ao laço. As duas guardas são redundantes **neste** caso e separam-se quando um slot é
/// acrescentado, que é o gate
/// [`a_timer_appended_to_a_live_object_is_armed_and_no_neighbour_is_disturbed`].
///
/// **Mutação que deve sangrar:** apagar o `if nascidos == timers.0.len() { return false; }` **e**
/// o `.skip(nascidos)` — que juntos são a lei antiga («arma todo slot que não corre»).
#[test]
fn reconcile_never_re_arms_a_finished_one_shot() {
    let (t, _) = one_shot();
    let ts = Timers(vec![Timer {
        autostart: true,
        repeat: false,
        ..t
    }]);
    let mut rt = TimerRuntime::default();
    reconcile(&ts, &mut rt);
    let out = advance(&ts.0[0], &mut rt.0[0], ts.0[0].duration_us);
    assert_eq!(out.fires, 1);
    assert!(out.finished, "a fixtura nao terminou o one-shot");
    // Cem quadros de reconciliacao, como o produto faz.
    for _ in 0..100 {
        assert!(!reconcile(&ts, &mut rt), "reconciliar mexeu num slot velho");
    }
    assert!(
        !rt.0[0].running,
        "o one-shot terminado foi RE-ARMADO — ele dispararia uma vez por periodo, para sempre"
    );
    assert_eq!(
        advance(&ts.0[0], &mut rt.0[0], ts.0[0].duration_us * 4).fires,
        0,
        "um one-shot terminado voltou a falar"
    );
}

/// ⭐ **Um timer ACRESCENTADO a um objecto que já tinha timers começa a correr** — e **nenhum**
/// vizinho é tocado, nem o que corre nem o que já terminou.
///
/// ⚠️ É a segunda granularidade da mesma lei: o `+` do painel faz nascer **um slot**, não um
/// relógio. Sem esta metade, o segundo timer de um objecto seria inerte enquanto o primeiro corre.
///
/// ⚠️⚠️ **O vizinho TERMINADO é o que dá dentes a este gate, e a 1.ª redacção não o tinha.** Com só
/// um vizinho a correr, a mutação `skip(nascidos)` → `if !s.running` **SOBREVIVIA**: um timer vivo
/// satisfaz `s.running`, logo os dois braços concordam sobre ele. É o slot **parado por ter
/// acabado** que os separa — e é ele que um objecto real tem, porque o `Recarga` da cena de smoke
/// termina ao fim de 3 s e o artista acrescenta um segundo timer depois disso.
///
/// **Mutação que deve sangrar:** `.skip(nascidos)` → `if t.autostart && !s.running`.
#[test]
fn a_timer_appended_to_a_live_object_is_armed_and_no_neighbour_is_disturbed() {
    let (base, _) = one_shot();
    let vivo = Timer {
        autostart: true,
        repeat: true,
        ..base.clone()
    };
    let acabado = Timer {
        autostart: true,
        repeat: false,
        ..base.clone()
    };
    let ts = Timers(vec![vivo.clone(), acabado.clone()]);
    let mut rt = TimerRuntime::default();
    reconcile(&ts, &mut rt);
    rt.0[0].elapsed_us = 700_000;
    // O slot 1 chega ao fim: um one-shot terminado fica `running == false`, que é exactamente a
    // condição que a lei ANTIGA lia como «por armar».
    let out = advance(&ts.0[1], &mut rt.0[1], ts.0[1].duration_us);
    assert!(out.finished, "a fixtura nao terminou o vizinho one-shot");

    let ts2 = Timers(vec![
        vivo,
        acabado,
        Timer {
            autostart: true,
            ..base
        },
    ]);
    assert!(reconcile(&ts2, &mut rt), "o slot novo nao foi reconciliado");
    assert!(rt.0[2].running, "o timer acrescentado nasceu inerte");
    assert_eq!(
        rt.0[0].elapsed_us, 700_000,
        "reconciliar REZEROU o vizinho que ja' corria"
    );
    assert!(
        !rt.0[1].running,
        "acrescentar um timer RESSUSCITOU o one-shot do vizinho — ele dispararia outra vez sem \
         que o artista lhe tivesse tocado"
    );
}

/// **Encolher deita fora o estado dos slots que já não existem.**
///
/// ⚠️ Senão remover um timer e repô-lo devolvia-o a correr **a meio** do período anterior — e o
/// artista não tem por onde ver esse estado.
#[test]
fn shrinking_the_config_drops_the_state_of_the_timers_that_left() {
    let (base, _) = one_shot();
    let ts = Timers(vec![
        Timer {
            autostart: true,
            ..base.clone()
        },
        Timer {
            autostart: true,
            ..base.clone()
        },
    ]);
    let mut rt = TimerRuntime::default();
    reconcile(&ts, &mut rt);
    rt.0[1].elapsed_us = 900_000;

    let so_um = Timers(vec![ts.0[0].clone()]);
    assert!(reconcile(&so_um, &mut rt), "encolher nao foi relatado");
    assert_eq!(rt.0.len(), 1);
    // E repor o segundo dá-lhe um periodo INTEIRO, nao o resto do antigo.
    reconcile(&ts, &mut rt);
    assert_eq!(
        rt.0[1].elapsed_us, 0,
        "o timer reposto herdou o relogio do que foi removido"
    );
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
