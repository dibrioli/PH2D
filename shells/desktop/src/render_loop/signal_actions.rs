//! ⭐⭐⭐ **A PONTE do `SignalActions`** — onde um sinal deixa de ser um toast e vira jogo
//! (TOP-20 #5).
//!
//! # O que ela fecha
//!
//! O `ph2d-runtime` publica sinais de **cinco** origens (timeline · contacto · sensor · animação ·
//! relógio) e, até 2026-09-09, tinha **três** consumidores — um toast, uma linha de terminal e a
//! máquina de estados de UI. Todos de diagnóstico. *Nada na cena reagia a um sinal.*
//!
//! # ⚠️ As DUAS metades, e porque a fronteira está onde está
//!
//! A [`ph2d_ecs::resolve_signal_actions`] é pura sobre o mundo: ela casa nomes, resolve alvos e
//! devolve efeitos. **Este** ficheiro aplica-os, e a razão de ele existir separado é o **undo**:
//!
//! - **Um relógio** vive no `TimerRuntime`, que **não é componente registado** ⇒ arrancá-lo não é
//!   um passo de `Ctrl+Z`, e não precisa de declaração nenhuma.
//! - **A visibilidade** vive na [`ph2d_ecs::Visibility`], que **é** registada ⇒ sem o ledger,
//!   **cada porta que abre seria um passo de undo**. É a mesma lei que o `preview_drive` já
//!   escreve: *o documento é o valor AUTORADO; o que um motor escreve agora é pré-visualização*.
//!
//! ⇒ os dois verbos escrevem por portas diferentes de propósito, e a diferença **não é
//! arbitrária** — ela é a de o componente estar ou não no registo.
//!
//! # ⚠️ QUANDO, no quadro
//!
//! Depois do dreno do outbox (senão os sinais deste quadro chegariam ao próximo) e antes do
//! `post_frame_undo` (senão a escrita não seria fotografada nem declarada). É a mesma janela que o
//! toast usa, e é por isso que ela é lida **do mesmo cursor de leitura** — um segundo `SignalReader`
//! ao lado, com o seu próprio cursor.

use ph2d_ecs::{SignalEffect, SignalVerb, SimWorld, TimerRuntime, Timers, Visibility};

use ph2d_preview_drive::{Driven, PreviewDrive};

/// O que uma aplicação fez — para o log de diagnóstico, e para os gates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActionReport {
    /// Quantos efeitos chegaram a mexer em alguma coisa.
    pub(crate) applied: usize,
    /// Quantos não tinham onde pegar (o alvo não tem o componente que o verbo escreve).
    ///
    /// ⚠️ **Não é um erro, e é por isso que ele é CONTADO e não gritado:** ligar um sinal a um
    /// objecto sem relógio é uma configuração a meio, não uma avaria — e o painel é quem tem de o
    /// dizer, não um toast por quadro.
    pub(crate) inert: usize,
}

/// ⭐⭐ **Aplica os efeitos deste quadro.**
///
/// ⚠️ **O `drive` é obrigatório na assinatura**, e não um `Option`: uma função-irmã «sem ledger»
/// seria a segunda porta pela qual o defeito volta — exactamente a lei que a `ProjectState::capture`
/// já escreve para si mesma.
///
/// ⚠️ **O `audio` é um `Option` e o `drive` não**, e a assimetria é honesta: um editor sem
/// dispositivo de som corre em silêncio de propósito (o `AudioSystem::new` devolve `None` e a casa
/// degrada como faz com o `gilrs`), enquanto uma escrita no documento sem ledger é sempre um
/// defeito. *Um `Option` que nomeia uma ausência real não é o mesmo que um que nomeia uma
/// conveniência.*
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    sim: &mut SimWorld,
    effects: &[SignalEffect],
    drive: &mut PreviewDrive,
    mut audio: Option<&mut ph2d_app_audio::AudioSystem>,
) -> ActionReport {
    let mut report = ActionReport::default();
    for fx in effects {
        let ok = match fx.verb {
            SignalVerb::StartTimer => set_timers(sim, fx, true),
            SignalVerb::StopTimer => set_timers(sim, fx, false),
            SignalVerb::Show => set_visible(sim, drive, fx, Some(false)),
            SignalVerb::Hide => set_visible(sim, drive, fx, Some(true)),
            SignalVerb::ToggleVisibility => set_visible(sim, drive, fx, None),
            // ⭐⭐⭐ **O SOM** (TOP-20 #4) — o verbo que a recusa deste enum nomeava como
            // inalcançável até 2026-09-09. ⚠️ `as_deref_mut` porque o laço passa por aqui N vezes
            // e um `Option<&mut _>` não é `Copy`.
            SignalVerb::PlaySound => {
                super::audio_2d::play_target(sim, audio.as_deref_mut(), fx.target)
            }
            SignalVerb::StopSound => {
                super::audio_2d::stop_target(sim, audio.as_deref_mut(), fx.target)
            }
        };
        if ok {
            report.applied += 1;
        } else {
            report.inert += 1;
        }
    }
    report
}

/// Arranca ou pára os timers do alvo. `arg` vazio = **todos**; senão, os que têm aquele nome.
///
/// ⚠️ **A lei de arrancar e de parar vive no `ph2d-ecs`** ([`ph2d_ecs::timer_start`] /
/// [`ph2d_ecs::timer_stop`]), e não aqui: *arrancar é do princípio, parar guarda o progresso*, e
/// escrevê-la neste ficheiro daria uma segunda resposta ao lado da que o `autostart` já usa.
///
/// ⚠️ **Um nome que não casa com timer nenhum devolve `false`** — é a configuração a meio que o
/// [`ActionReport::inert`] conta.
fn set_timers(sim: &mut SimWorld, fx: &SignalEffect, start: bool) -> bool {
    let Some(timers) = sim.world().get::<Timers>(fx.target).cloned() else {
        return false;
    };
    // ⚠️ **Os índices saem da CONFIG antes de tocar no relógio**: o `Timers` e o `TimerRuntime`
    // ligam-se pelo índice, e ler os dois ao mesmo tempo pediria dois empréstimos do mundo.
    let alvos: Vec<usize> = timers
        .0
        .iter()
        .enumerate()
        .filter(|(_, t)| fx.arg.is_empty() || t.name == fx.arg)
        .map(|(i, _)| i)
        .collect();
    if alvos.is_empty() {
        return false;
    }
    let Some(mut rt) = sim.world_mut().get_mut::<TimerRuntime>(fx.target) else {
        // O relógio ainda não nasceu — ele nasce na reconciliação do quadro seguinte, e o sinal
        // deste quadro perde-se. ⚠️ Só acontece no quadro em que o componente é anexado.
        return false;
    };
    let mut tocou = false;
    for i in alvos {
        let Some(s) = rt.0.get_mut(i) else { continue };
        if start {
            ph2d_ecs::timer_start(s);
        } else {
            ph2d_ecs::timer_stop(s);
        }
        tocou = true;
    }
    tocou
}

/// Escreve a visibilidade do alvo **pelo ledger**. `None` = inverter o que está lá.
///
/// ⚠️⚠️ **As DUAS metades são obrigatórias**, e esquecer a segunda é o defeito silencioso: escrever
/// sem `driven` faria a `settle` do fim do quadro esquecer o facto, e o valor de pré-visualização
/// viraria **documento** — que é exactamente o passo de undo que este caminho existe para não ter.
fn set_visible(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    fx: &SignalEffect,
    hide: Option<bool>,
) -> bool {
    let Some(antes) = sim.world().get::<Visibility>(fx.target).map(|v| v.hidden) else {
        return false;
    };
    let depois = hide.unwrap_or(!antes);
    // ⚠️ **Declara-se mesmo sem mudança**, e é deliberado: a `settle` esquece quem **não** foi
    // declarado neste quadro, então calar-se num quadro em que o valor coincide devolveria o
    // facto ao documento no meio de uma corrida.
    drive.driven(fx.target, Driven::Visible(antes), Driven::Visible(depois));
    Driven::Visible(depois).write(sim, fx.target);
    true
}

/// Os gates desta ponte — módulo irmão, pelo teto de 600 LOC da shell.
#[cfg(test)]
#[path = "signal_actions_tests.rs"]
mod tests;
