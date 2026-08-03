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

use crate::{SignalOnHit, SignalOnLeave};

use super::PhysicsBridge;
use super::contacts::ContactPhase;

/// **Um sinal que a física emitiu neste dispatch.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalEvent {
    /// O nome autorado — já aparado, e nunca vazio (ver [`SignalOnHit::name`]).
    ///
    /// ⚠️ **É o contrato inteiro.** Chegada e saída não se distinguem por um campo
    /// aqui: distinguem-se por serem NOMES diferentes, autorados em duas rows
    /// (W-SignalLeave). Ver o porquê em [`PhysicsBridge::signal_events`].
    pub name: String,
    /// Quem GRITOU: a entidade que carrega o [`SignalOnHit`] ou o
    /// [`SignalOnLeave`].
    pub source: Entity,
    /// Quem CHEGOU — ou quem SAIU. Um consumidor que pergunta *"quem abriu a
    /// porta?"* quer este.
    pub other: Entity,
}

impl PhysicsBridge {
    /// **Os sinais que as chegadas e as saídas deste dispatch emitem.**
    ///
    /// # Derivado dos canais que já existem, nunca acumulado à parte
    ///
    /// Lê [`contact_events`](Self::contact_events) (as transições sólidas, por
    /// TICK) e [`trigger_events`](Self::trigger_events)/[`trigger_exits`] (os
    /// dois extremos de um sensor, por dispatch). Manter uma lista própria seria
    /// uma **segunda resposta** a *o que aconteceu neste quadro* — e o dia em que
    /// as duas discordassem, o sinal descreveria uma colisão que a tela não
    /// desenhou.
    ///
    /// [`trigger_exits`]: Self::trigger_exits
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
    /// # CHEGADA e SAÍDA, cada uma com o SEU nome (W-SignalLeave)
    ///
    /// `Began` + entrada em sensor leem o [`SignalOnHit`]; `Ended` + saída de
    /// sensor leem o [`SignalOnLeave`]. ⚠️ **O evento NÃO carrega fase**, e é
    /// deliberado: o contrato é o NOME (ADR-0143), quem escuta casa numa string,
    /// e esta mesma outbox recebe os sinais da timeline, que não têm fase
    /// nenhuma. Um campo de fase obrigaria todo consumidor a perguntar duas
    /// coisas para saber uma — `door_open` e `door_close` são dois contratos.
    ///
    /// ⚠️ **Um extremo sem o componente dele é SILÊNCIO, não o outro nome.** Marcar
    /// só a chegada é o mundo que já existia, e ele tem de continuar
    /// byte-idêntico.
    ///
    /// Aloca só quando há o que emitir: a varredura é sobre listas que estão
    /// vazias em quase todo quadro.
    #[must_use]
    pub fn signal_events(&self, sim: &SimWorld) -> Vec<SignalEvent> {
        let mut out = Vec::new();
        let on_hit = |e: Entity| -> Option<String> {
            sim.world()
                .get::<SignalOnHit>(e)
                .and_then(|s| s.name().map(str::to_owned))
        };
        let on_leave = |e: Entity| -> Option<String> {
            sim.world()
                .get::<SignalOnLeave>(e)
                .and_then(|s| s.name().map(str::to_owned))
        };
        for ev in self.contact_events() {
            // O `Ended` é honesto mesmo com um dos corpos DELETADO (o doc do
            // `ContactPhase` já o declara): o `get` de uma entidade que não
            // existe devolve `None`, então um corpo morto simplesmente não grita
            // — e o vizinho VIVO que o perdeu de vista grita a saída dele, que é
            // exatamente o que uma porta deve fazer.
            let named: &dyn Fn(Entity) -> Option<String> = match ev.phase {
                ContactPhase::Began => &on_hit,
                ContactPhase::Ended => &on_leave,
            };
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
        for (evs, named) in [
            (
                self.trigger_events(),
                &on_hit as &dyn Fn(Entity) -> Option<String>,
            ),
            (self.trigger_exits(), &on_leave),
        ] {
            for ev in evs {
                if let Some(name) = named(ev.sensor) {
                    out.push(SignalEvent {
                        name,
                        source: ev.sensor,
                        other: ev.other,
                    });
                }
            }
        }
        out
    }
}
