//! ⭐⭐⭐ **Smoke do `SignalActions`** (TOP-20 #5). `PH2D_SIGNAL_ACTION_SMOKE=1`.
//!
//! # O que esta cena prova, e porque ela é a mais importante da fila
//!
//! Até 2026-09-09 um sinal deste app só sabia fazer **um toast**. A cena põe na tela o circuito
//! inteiro, **sem uma linha de script**:
//!
//! ```text
//!   Batida (relógio de 1 s, repete)  ──sinal `batida`──►  Piscante:  inverte a visibilidade
//!   Gatilho (relógio de 3 s, uma vez) ──sinal `abre`───►  Porta:     ESCONDE-SE (a porta abre)
//!                                     └────────────────►  Motor:     ARRANCA o relógio dele
//!   Motor (relógio de 1 s, repete, autostart DESLIGADO) ──sinal `motor`──► toast
//! ```
//!
//! | objecto | o que ele é | o que se vê |
//! |---|---|---|
//! | **Batida** | relógio de 1 s que repete | pisca o **Piscante** uma vez por segundo |
//! | **Piscante** | um quadrado verde | aparece e desaparece, para sempre |
//! | **Gatilho** | relógio de 3 s, **uma vez** | aos 3 s: a **Porta** some **e** o **Motor** arranca |
//! | **Porta** | um quadrado laranja | some aos 3 s e **fica** assim |
//! | **Motor** | relógio de 1 s com **`autostart` DESLIGADO** | calado até aos 3 s, e depois fala |
//!
//! # ⚠️ O que provar na tela
//!
//! - **O Piscante pisca a 1 Hz** — um sinal a escrever na cena, não num toast.
//! - **Aos 3 segundos a Porta desaparece**, e não volta: um *one-shot* dispara uma vez.
//! - ⭐⭐⭐ **`Signal: motor` só começa DEPOIS dos 3 s.** É a prova de que um sinal ARRANCA um
//!   relógio — o `Motor` tem `autostart` desligado, e até 2026-09-09 um timer assim era
//!   **inalcançável por gesto nenhum**.
//! - ⭐⭐ **`Ctrl+Z` não desfaz nada disto.** A visibilidade é um componente **do documento**, e
//!   sem o ledger de pré-visualização cada pisca do Piscante seria um passo de `Ctrl+Z` — **60 por
//!   minuto**. Carregue nele à vontade: a fila tem de estar vazia.
//!
//! ⚠️ **Ligue `PH2D_SIGNAL_LOG=1` junto**: o terminal diz a origem de cada sinal e imprime
//! `N accao(oes) aplicada(s)` — o segundo consumidor, com cursor próprio.
//!
//! ⚠️ Se a linha `[signal-action-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Name, SignalAction, SignalActions, SignalVerb, Timer, Timers, Transform, Visibility,
};
use ph2d_render::Sprite;

/// Um relógio nomeado que publica `signal`.
fn relogio(name: &str, secs: u64, repeat: bool, autostart: bool, signal: &str) -> Timers {
    Timers(vec![Timer {
        name: name.to_string(),
        duration_us: secs * 1_000_000,
        repeat,
        autostart,
        signal: signal.to_string(),
    }])
}

fn linha(on: &str, target: &str, verb: SignalVerb, arg: &str) -> SignalAction {
    SignalAction {
        on: on.into(),
        target: target.into(),
        verb,
        arg: arg.into(),
        target_by: ph2d_ecs::SignalTarget::Named,
    }
}

/// No prólogo do quadro, uma vez. No-op sem a env.
pub fn signal_action_smoke(cx: &mut crate::scene_ctx::SceneCtx) {
    {
        let world = cx.sim.world_mut();

        // ⚠️ **Os alvos nascem ANTES de quem os nomeia** — não porque a resolução o exija (ela
        // corre por quadro, e o nome é resolvido na hora), mas porque a cena se lê de cima
        // para baixo e um leitor humano procura o alvo acima da tabela que o cita.
        world.spawn((
            Transform::from_translation(Vec2::new(-2.0, 1.5)),
            Sprite::atlas(0, [1.2, 1.2], [0.2, 0.8, 0.4, 1.0]),
            Name::new("Piscante"),
            Visibility::visible(),
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(2.0, 1.5)),
            Sprite::atlas(0, [1.2, 1.2], [0.9, 0.6, 0.1, 1.0]),
            Name::new("Porta"),
            Visibility::visible(),
        ));

        // ⭐⭐⭐ **O objecto cujo relógio SÓ um sinal arranca.** `autostart: false` — e até esta
        // wave um timer assim era inalcançável por gesto nenhum.
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, -1.5)),
            Sprite::atlas(0, [1.2, 0.6], [0.5, 0.5, 0.55, 1.0]),
            Name::new("Motor"),
            relogio("motor", 1, true, false, "motor"),
        ));

        // O relógio que pisca — e a tabela que faz o pisca acontecer.
        world.spawn((
            Transform::from_translation(Vec2::new(-2.0, 0.0)),
            Sprite::atlas(0, [1.2, 0.4], [0.35, 0.35, 0.4, 1.0]),
            Name::new("Batida"),
            relogio("batida", 1, true, true, "batida"),
            SignalActions(vec![linha(
                "batida",
                "Piscante",
                SignalVerb::ToggleVisibility,
                "",
            )]),
        ));

        // ⭐ **Um sinal, DUAS acções** — a lista é ordenada e as duas correm no mesmo quadro.
        world.spawn((
            Transform::from_translation(Vec2::new(2.0, 0.0)),
            Sprite::atlas(0, [1.2, 0.4], [0.35, 0.35, 0.4, 1.0]),
            Name::new("Gatilho"),
            relogio("gatilho", 3, false, true, "abre"),
            SignalActions(vec![
                linha("abre", "Porta", SignalVerb::Hide, ""),
                linha("abre", "Motor", SignalVerb::StartTimer, ""),
            ]),
        ));
    }
    // ⚠️ **A linha que o doc manda procurar** — uma cena que monta em silêncio é uma cena que
    // o smoke julga errado.
    eprintln!(
        "[signal-action-smoke] Batida->pisca o Piscante · Gatilho(3s)->esconde a Porta E arranca o Motor"
    );
}
