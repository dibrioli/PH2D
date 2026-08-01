//! **O nome que este objeto GRITA quando algo chega nele** (W-Signal).
//!
//! # O vão, e por que ele era o maior do módulo
//!
//! A física publica quatro canais de leitura desde o W7 — *quem está dentro de um
//! sensor*, *quem toca quem*, *as transições*, *o pico do impacto* — todos
//! gateados, e **nenhum deles faz nada acontecer**. O que faltava era o
//! consumidor de GAMEPLAY, e a nota dele dizia *"cross-line, decisão do Enio"*
//! desde então.
//!
//! ⚠️ **A decisão já estava tomada no produto, e escrita.** O `render_loop`
//! declara, no ponto onde drena os sinais da timeline (ADR-0143):
//!
//! > *"v1 consumer: a toast — the visible proof the decoupled channel
//! > round-trips. **Audio/gameplay/Luau are the deferred cross-line consumers of
//! > the SAME outbox**; the timeline emits an event and never calls any of them
//! > (ADR-0075)."*
//!
//! Então o consumidor **existe**; quem faltava era o **publicador**. Esta wave
//! não inventa um segundo barramento — ela põe a física a emitir no que já há.
//!
//! # Um nome, não uma chamada
//!
//! O componente carrega uma **string**, e a física nunca sabe quem escuta. É o
//! ADR-0143 ao pé da letra (*"um marker EMITE um evento desacoplado, não uma
//! chamada"*) e o ADR-0075 (*"systems não se chamam"*): a porta é um nome, e o
//! dia em que houver um script Luau ou uma pista de áudio ouvindo, nada aqui
//! muda.
//!
//! # Um nome só, e quando ele dispara
//!
//! ⚠️ **CHEGADA, nunca saída.** Emitir o MESMO nome nos dois extremos tornaria o
//! sinal ambíguo — quem escuta não saberia se a porta abriu ou fechou —, e um
//! segundo nome é uma segunda pergunta com uma segunda row. Deferido com o
//! motivo, não esquecido.
//!
//! ⚠️ **E uma entidade é sólida OU sensor**, então as duas fontes de chegada (um
//! contato que COMEÇA · algo que ENTRA num sensor) são mutuamente exclusivas na
//! prática: um nome por entidade responde às duas sem um knob que escolha entre
//! elas.

use bevy_ecs::prelude::Component;
use ph2d_ecs::SimComponent;
use serde::{Deserialize, Serialize};

/// **O sinal que esta entidade emite quando algo chega nela.**
///
/// Ausente = não emite nada, que é o default de toda cena que já existe.
///
/// **Por que um componente próprio e não um campo do [`Collider`](super::Collider):**
/// o `Collider` é serializado POSICIONALMENTE pelo postcard, então apendar nele é
/// um bump de `PROJECT_SCHEMA` (o que `layer`/`is_sensor`/`offset` custaram cada
/// um). Um componente recém-**registrado** é chaveado pelo hash do próprio nome
/// de tipo e é puramente aditivo — **sem bump** —, o precedente do
/// `PhysicsJoint`/W3 e de todos os overrides desde o W8.
///
/// ⚠️ **É CONFIG, nunca estado vivo** (a lei do módulo): o nome é autorado e não
/// muda por conta própria, então o `canonicalize` do undo — que ordena pelos
/// BYTES do componente — não vê um passo espúrio por frame.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalOnHit(pub String);

impl SignalOnHit {
    /// O nome, ou `None` se estiver em branco.
    ///
    /// ⚠️ **Um nome em branco NÃO é um sinal**, e a regra é emprestada do
    /// `set_marker_signal` da timeline, palavra por palavra: *"um sinal sem nome
    /// não é um contrato que alguém possa casar, então ele não pode ler como
    /// 'tem sinal'"*. Duas respostas para *o que conta como um sinal?* é como as
    /// duas metades passam a discordar sobre uma string com um espaço.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        let t = self.0.trim();
        (!t.is_empty()).then_some(t)
    }
}

impl SimComponent for SignalOnHit {}
