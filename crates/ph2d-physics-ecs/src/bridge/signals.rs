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

use ph2d_platformer::{JumpKind, PlayerEvent};

use crate::{PlayerSignals, SignalOnHit, SignalOnLeave};

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
        // ── E O PLAYER (`W-PlayerOut`, A3) ───────────────────────────────────
        // ⚠️ **A TERCEIRA fonte da MESMA porta, e não uma segunda porta.** Este
        // método já funde contatos e sensores; um canal paralelo obrigaria a
        // shell a drenar duas coisas e a decidir a ordem entre elas — e a shell
        // que já drena esta não muda uma linha para receber aquilo.
        //
        // ⚠️ **Opt-in por-player** (`PlayerSignals`): sem ele toda cena de smoke
        // com um personagem cuspiria toasts, e o custo cairia sobre waves que
        // nada têm com esta. É o mesmo idioma dos dois irmãos acima.
        for (source, ev) in self.player_events() {
            if sim.world().get::<PlayerSignals>(*source).is_none() {
                continue;
            }
            out.push(SignalEvent {
                name: player_signal_name(ev).to_owned(),
                source: *source,
                // ⚠️ **Um evento de player tem UMA parte.** O campo existe porque
                // um contato tem duas, e repetir a fonte é a resposta honesta:
                // inventar um segundo corpo (o chão em que aterrou?) seria um
                // fato que o canal não carrega.
                other: *source,
            });
        }
        out
    }
}

/// **O nome estável de cada transição de player** — o contrato do A3.
///
/// ⚠️ **Os três pulos são três NOMES, não um nome com um campo**, e é a lei que
/// o [`SignalOnLeave`] já enuncia palavra por palavra: o contrato é o NOME
/// (ADR-0143), quem escuta casa numa string, e um campo de tipo obrigaria todo
/// consumidor a perguntar duas coisas para saber uma. Um som de pulo de parede
/// não é o som de um pulo do chão — são dois contratos, e é assim que se lê.
///
/// ⚠️ **Porta ÚNICA:** o dia em que um segundo lugar traduzir isto, os dois
/// divergem e o consumidor deixa de casar com metade dos eventos.
#[must_use]
fn player_signal_name(ev: &PlayerEvent) -> &'static str {
    match ev {
        PlayerEvent::Landed { .. } => "player.landed",
        PlayerEvent::Jumped { kind } => match kind {
            JumpKind::Ground => "player.jumped.ground",
            JumpKind::Air => "player.jumped.air",
            JumpKind::Wall => "player.jumped.wall",
        },
        PlayerEvent::Apex => "player.apex",
        PlayerEvent::Dashed => "player.dashed",
        PlayerEvent::LedgeGrabbed => "player.ledge_grabbed",
        PlayerEvent::EnteredWater => "player.entered_water",
        PlayerEvent::LeftWater => "player.left_water",
    }
}
