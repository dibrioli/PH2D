//! **A física GRITA um nome** — o publicador que faltava (W-Signal).
//!
//! Quatro canais de leitura existem desde o W7 e **nenhum deles faz nada
//! acontecer**. O que faltava era o consumidor de GAMEPLAY, e a decisão estava
//! escrita no produto o tempo todo — no ponto onde o `render_loop` drena os
//! sinais da timeline (ADR-0143):
//!
//! > *"Audio/gameplay/Luau are the deferred cross-line consumers of the SAME
//! > outbox; the timeline emits an event and never calls any of them (ADR-0075)."*
//!
//! Então o consumidor **existe** (hoje um toast, *"a prova visível de que o canal
//! desacoplado fecha a volta"*), e esta porta é o publicador. Ela **não inventa um
//! segundo barramento**.
//!
//! # A física não conhece a timeline, e é isso que a mantém desacoplada
//!
//! O tipo do sinal da timeline (`TimelineSignal`) mora na crate DELA. Importá-lo
//! aqui faria o motor de física depender do editor de animação para responder
//! *"algo bateu"* — o oposto do ADR-0075. Esta porta devolve o **nome** e as duas
//! entidades; quem funde as duas fontes numa saída é o SHELL, que já é o dono do
//! consumidor e já drena a outra.

use ph2d_ecs::{Entity, SimWorld};

use crate::SignalOnHit;

use super::PhysicsBridge;
use super::contacts::ContactPhase;

/// **Um sinal que a física emitiu neste dispatch.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalEvent {
    /// O nome autorado — já aparado, e nunca vazio (ver [`SignalOnHit::name`]).
    pub name: String,
    /// Quem GRITOU: a entidade que carrega o [`SignalOnHit`].
    pub source: Entity,
    /// Quem CHEGOU. Um consumidor que pergunta *"quem abriu a porta?"* quer este.
    pub other: Entity,
}

impl PhysicsBridge {
    /// **Os sinais que as chegadas deste dispatch emitem.**
    ///
    /// # Derivado dos canais que já existem, nunca acumulado à parte
    ///
    /// Lê [`contact_events`](Self::contact_events) (as transições sólidas, por
    /// TICK) e [`trigger_events`](Self::trigger_events) (as entradas em sensor,
    /// por dispatch). Manter uma terceira lista própria seria uma **segunda
    /// resposta** a *o que aconteceu neste quadro* — e o dia em que as duas
    /// discordassem, o sinal descreveria uma colisão que a tela não desenhou.
    ///
    /// Por isso ele também herda de graça toda a disciplina que aqueles canais
    /// pagaram: um scrub não vira tempestade, uma descontinuidade re-baseliza em
    /// silêncio, e um toque mais curto que um tick ainda grita.
    ///
    /// # Os DOIS lados de um contato podem gritar
    ///
    /// Um contato é uma relação simétrica, então cada ponta é perguntada por si:
    /// uma bola marcada caindo num chão marcado emite os dois nomes, cada um com
    /// o `other` certo. Escolher um lado exigiria uma regra que ninguém autorou.
    ///
    /// ⚠️ **Só a CHEGADA** — `Began` e a entrada em sensor. O `Ended` sob o mesmo
    /// nome tornaria o sinal ambíguo para quem escuta; um nome de saída é uma
    /// segunda pergunta, e está deferida com o motivo no doc de [`SignalOnHit`].
    ///
    /// Aloca só quando há o que emitir: a varredura é sobre listas que estão
    /// vazias em quase todo quadro.
    #[must_use]
    pub fn signal_events(&self, sim: &SimWorld) -> Vec<SignalEvent> {
        let mut out = Vec::new();
        let named = |e: Entity| -> Option<String> {
            sim.world()
                .get::<SignalOnHit>(e)
                .and_then(|s| s.name().map(str::to_owned))
        };
        for ev in self.contact_events() {
            if ev.phase != ContactPhase::Began {
                continue;
            }
            for (source, other) in [(ev.a, ev.b), (ev.b, ev.a)] {
                if let Some(name) = named(source) {
                    out.push(SignalEvent {
                        name,
                        source,
                        other,
                    });
                }
            }
        }
        for ev in self.trigger_events() {
            if let Some(name) = named(ev.sensor) {
                out.push(SignalEvent {
                    name,
                    source: ev.sensor,
                    other: ev.other,
                });
            }
        }
        out
    }
}
