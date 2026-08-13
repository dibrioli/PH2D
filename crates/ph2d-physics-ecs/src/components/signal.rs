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
//! # Dois nomes, porque são duas perguntas (W-SignalLeave)
//!
//! ⚠️ **Emitir o MESMO nome nos dois extremos tornaria o sinal ambíguo** — quem
//! escuta não saberia se a porta abriu ou fechou. O W-Signal deferiu a saída com
//! esse motivo; esta wave a constrói do jeito que o motivo prescreve: um
//! **segundo nome**, num **segundo componente**, com uma **segunda row**.
//!
//! ⚠️ **E o par NÃO é `(nome, fase)`.** O contrato é o NOME (ADR-0143), e quem
//! escuta casa numa string — a mesma outbox recebe os sinais da timeline, que
//! não têm fase nenhuma. Um campo de fase seria uma segunda resposta a *o que
//! aconteceu*, e obrigaria todo consumidor a perguntar duas coisas para saber
//! uma. `door_open` e `door_close` são dois contratos, e é assim que se lê.
//!
//! ⚠️ **E uma entidade é sólida OU sensor**, então as duas fontes de cada extremo
//! (um contato que COMEÇA/TERMINA · algo que ENTRA/SAI de um sensor) são
//! mutuamente exclusivas na prática: um nome por extremo responde às duas sem um
//! knob que escolha entre elas.
//!
//! # ⚠️ Dois componentes, e a razão é o custo de um BUMP
//!
//! `SignalOnHit` é uma tupla serializada **posicionalmente** pelo postcard, então
//! apendar o segundo nome nela seria um bump de `PROJECT_SCHEMA` — e **um bump
//! RECUSA todo projeto já salvo**. Um componente recém-*registrado* é chaveado
//! pelo hash do próprio nome de tipo e é puramente aditivo. É o mesmo trade que o
//! W-AreaDrag pagou (`AreaEffector` + `AreaDrag` separados pelo mesmo motivo),
//! escrito aqui para ninguém "arrumar" os dois num struct só.

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
        signal_name(&self.0)
    }
}

impl SimComponent for SignalOnHit {}

/// **O sinal que esta entidade emite quando algo SAI dela** (W-SignalLeave).
///
/// O gêmeo exato do [`SignalOnHit`], e as duas fontes espelham as dele: um
/// contato que TERMINA e algo que sai de um sensor. Ausente = não emite nada.
///
/// ⚠️ **Componente próprio e não um campo do irmão** — ver o cabeçalho do módulo:
/// apendar num tipo serializado posicionalmente é um bump de `PROJECT_SCHEMA`, e
/// um bump recusa todo projeto já salvo.
///
/// ⚠️ **Um `Ended` é honesto mesmo quando o outro corpo foi DELETADO** — é o que
/// o [`ContactPhase::Ended`] já declara (*"o contato terminou, e **por quê** é a
/// pergunta de quem chama"*). Uma porta que fecha quando o jogador é destruído
/// dentro dela é o comportamento certo, não um caso especial.
///
/// [`ContactPhase::Ended`]: crate::ContactPhase::Ended
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalOnLeave(pub String);

impl SignalOnLeave {
    /// O nome, ou `None` se estiver em branco — a MESMA regra do irmão, pela
    /// mesma função: duas cópias de *o que conta como um sinal?* discordariam
    /// sobre uma string com um espaço.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        signal_name(&self.0)
    }
}

impl SimComponent for SignalOnLeave {}

/// **O que conta como um sinal** — a porta ÚNICA dos dois componentes.
///
/// Aparada, e nunca vazia. Não é um detalhe de implementação partilhado por
/// conveniência: é a resposta a uma pergunta que os dois lados TÊM de responder
/// igual, e é por isso que ela não é escrita duas vezes.
fn signal_name(raw: &str) -> Option<&str> {
    let t = raw.trim();
    (!t.is_empty()).then_some(t)
}

/// **Este player publica os eventos dele como SINAIS** (`W-PlayerOut`, A3).
///
/// Ausente = silêncio, que é o default de toda cena que já existe — e o default
/// é DESLIGADO de propósito: sem ele toda cena de smoke com um personagem
/// passaria a cuspir toasts, e o custo cairia sobre waves que nada têm com esta.
///
/// # ⚠️ Um MARCADOR, e não um campo com um nome
///
/// Os irmãos [`SignalOnHit`]/[`SignalOnLeave`] carregam uma `String` porque o
/// nome deles é **autorado** — a porta que o artista batiza. Os nomes de um
/// player não são: eles descrevem o que a LEI fez (*aterrou*, *saltou de uma
/// parede*), e são fixos. Um campo de nome aqui seria um controle a pedir uma
/// escolha que não existe, e a presença é o booleano inteiro — o idioma do
/// [`Ccd`](super::Ccd) e do [`LockRotation`](super::LockRotation).
///
/// ⚠️ **Componente próprio e não um campo do [`PlatformPlayer`](super::PlatformPlayer):**
/// aquele é serializado POSICIONALMENTE pelo postcard, então apendar nele é um
/// bump de `PROJECT_SCHEMA`, e **um bump recusa todo projeto já salvo**. Um
/// componente recém-registrado é chaveado pelo hash do próprio nome de tipo e é
/// puramente aditivo — **sem bump**.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerSignals;

impl SimComponent for PlayerSignals {}
