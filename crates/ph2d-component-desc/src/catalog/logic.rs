//! ⭐⭐⭐ **A família LÓGICA** — o que faz um jogo acontecer **sem uma linha de script**.
//!
//! # Porque ela é um módulo próprio, e não uma linha no `core`
//!
//! É a lei declarada no [`super`]: **uma família, um módulo** (isolamento, DIRETRIZ §1.5.2.1).
//! O `core` descreve o que todo objecto **É** — identidade, pose, ordem; esta descreve o que ele
//! **FAZ**. E o corte paga-se já: a fila do
//! [levantamento](https://github.com/dibrioli/PH2D/blob/main/docs/Components/00_levantamento_componentes.md)
//! §7 traz mais **catorze** para aqui (`SensorZone` · `SignalActions` · `Spawner` · `Lifetime` ·
//! `Tags` · `StateMachine` · `ProjectileMotion` · …), e cada uma apende **sem encostar em mais
//! nada**.
//!
//! ⛔ **A alternativa era a categoria `Scripting`, e seria mentir**: o valor inteiro desta família
//! é *acontecer sem script*. Um artista que procure o relógio na secção de scripts conclui que
//! precisa de programar para o ter.
//!
//! ⚠️ **A lista está ORDENADA por `canonical_name`** — há gate (`the_catalog_is_sorted_and_unique`),
//! e fora de ordem a busca binária devolve `None` para um tipo que existe. *Um descritor que não é
//! encontrado lê-se exactamente como um descritor que não existe.*

use crate::{
    ComponentCategory as C, ComponentDesc, ComponentDesc as D, FieldDesc, FieldKind as K,
    ObjectKinds as O, Propagation,
};

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// **Os campos de UM timer** — o que a secção do Inspector edita.
///
/// ⚠️⚠️ **O `Timers` é uma LISTA, e este descritor descreve UM elemento dela.** É a mesma forma que
/// o `NamedAnchorList` e o `SpriteAnimations` têm, e a razão de ela ser aceitável aqui é o que o
/// descritor SERVE: rótulos e o `field_id` de um override por-campo. ⛔ O override por-campo de uma
/// lista é uma pergunta em aberto para toda a família (a F4 declarou-a assim), não uma dívida deste
/// componente.
///
/// ⛔ **Não há campo para o `TimerRuntime`**, e a ausência é a decisão inteira desta wave: o
/// relógio vivo não é componente registado, o undo não o fotografa, e descrevê-lo aqui seria
/// prometer ao Inspector um valor que ele não deve mostrar nem editar.
const TIMER_FIELDS: &[FieldDesc] = &[
    // ⚠️ **O nome do TIMER, que não é o nome do sinal** — confundi-los obrigaria a renomear o
    // componente para mudar o contrato.
    f(1, "component.field.timer_fields.1", K::Text),
    // ⚠️ **Em SEGUNDOS no painel, microssegundos no modelo.** A conversão vive nas duas pontas do
    // `render_loop::inspector_timer`, e há gate de ida-e-volta.
    f(2, "component.field.timer_fields.2", K::Scalar),
    f(3, "component.field.timer_fields.3", K::Toggle),
    // ⚠️ **É este o campo autorado** — o «está a correr agora» é vivo e não chega ao Inspector.
    f(4, "component.field.timer_fields.4", K::Toggle),
    // ⚠️ **Vazio = calado** — a lei da §11: um produtor sem nome não fala, em vez de falar com um
    // nome vazio.
    f(5, "component.field.timer_fields.5", K::Text),
];

/// **Os campos de um `StateMachine`** — os três de um ESTADO, os três de uma SETA, e o inicial.
///
/// ⚠️ **Duas listas num descritor só**, como o `Timers` descreve um elemento da dele: o que isto
/// SERVE são rótulos e o `field_id` de um override por-campo. Os ids não se reordenam.
///
/// ⛔ **Não há campo para o `StateMachineRuntime`** — o estado corrente é vivo, o undo não o
/// fotografa, e descrevê-lo aqui seria prometer ao Inspector um valor que ele não deve editar.
const STATE_MACHINE_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.state_machine_fields.1", K::Text),
    // ⚠️ **Vazio = calado**, a lei do produtor de sinal desta casa.
    f(2, "component.field.state_machine_fields.2", K::Text),
    f(3, "component.field.state_machine_fields.3", K::Text),
    // ⚠️ Os dois extremos de uma seta são ÍNDICES e não nomes — um nome ligaria a seta ao texto
    // que o artista pode reescrever a meio, e a seta saltaria de estado.
    f(4, "component.field.state_machine_fields.4", K::Scalar),
    // ⚠️ **Este é um NOME**, e é o único: ele é o contrato com o resto do mundo.
    f(5, "component.field.state_machine_fields.5", K::Text),
    f(6, "component.field.state_machine_fields.6", K::Scalar),
    f(7, "component.field.state_machine_fields.7", K::Scalar),
];

/// **Os campos de UMA linha da tabela `SignalActions`** — *quando o sinal `on` chegar, faz `verb`
/// em `target`*.
///
/// ⚠️ **O `target` é `Text` e não `Ref`**, e a distinção é a lei do CLAUDE.md §5: *referência
/// durável entre objectos é o NOME, nunca os bits*. Um `FieldKind::Ref` prometeria um picker que
/// guarda uma identidade — e o undo respawna tudo com bits novos.
const ACTION_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.action_fields.1", K::Text),
    f(2, "component.field.action_fields.2", K::Text),
    f(3, "component.field.action_fields.3", K::Enum),
    f(4, "component.field.action_fields.4", K::Text),
];

/// **Os campos de uma cópia que MORRE SOZINHA** (TOP-20 #12).
///
/// ⚠️ **Só dois, e o painel tem uma terceira linha que NÃO é campo:** a metade honesta —
/// *«isto só corre em cópias que uma fábrica pôs na cena»* — é derivada, não autorada.
/// A cutscene deste objecto — o NOME do container da timeline (TOP-20 #19).
///
/// ⚠️ **UM campo, e a lista curta é a wave:** duração, repetir e *«está a correr»* são do `Timer`,
/// que este componente EXIGE. Um `Duration` aqui seria um segundo relógio (plano 16 §1-bis).
const SEQUENCE_FIELDS: &[FieldDesc] = &[f(1, "Container", K::Text)];


const LIFETIME_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.lifetime_fields.1", K::Scalar),
    f(2, "component.field.lifetime_fields.2", K::Text),
];

/// O campo do fora-do-ecrã: a folga, em metros, antes de a morte valer.
const OUTSIDE_FIELDS: &[FieldDesc] = &[f(1, "component.field.outside_fields.1", K::Scalar)];

/// **Os campos da FÁBRICA** (TOP-20 #11) — *o quê · onde · quando · quanto*.
///
/// ⚠️ **Não há campo de RITMO**, e é a decisão medida do plano §2.4: a cadência vem do `Timers`,
/// que o `requires` puxa quando o artista acrescenta a fábrica. Um `rate` aqui seria um **segundo
/// relógio** para a mesma lei.
const FACTORY_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.factory_fields.1", K::Text),
    f(2, "component.field.factory_fields.2", K::Text),
    f(3, "component.field.factory_fields.3", K::Enum),
    f(4, "component.field.factory_fields.4", K::Vec2),
    f(5, "component.field.factory_fields.5", K::Text),
    f(6, "component.field.factory_fields.6", K::Enum),
    f(7, "component.field.factory_fields.7", K::Int),
    f(8, "component.field.factory_fields.8", K::Int),
    f(9, "component.field.factory_fields.9", K::Int),
    f(10, "component.field.factory_fields.10", K::Text),
    f(11, "component.field.factory_fields.11", K::Text),
    f(12, "component.field.factory_fields.12", K::Seed),
];

/// ⭐⭐⭐ **Uma REGRA da vigia** — como o `Timers`, o componente é uma LISTA e isto descreve a LINHA.
///
/// ⚠️ **`Counter` é o NOME do contador e não o deste objecto** — a vigia pode viver num objecto
/// «Regras» sem contador nenhum, e é isso que a torna autorável num sítio só.
const WATCH_FIELDS: &[FieldDesc] = &[
    f(1, "Counter", K::Text),
    f(2, "Compare", K::Enum),
    f(3, "Value", K::Int),
    // ⚠️ **Vazio = calada** — a lei da §11, a mesma do `Signal` do relógio logo abaixo.
    f(4, "Signal", K::Text),
    f(5, "Only Once", K::Toggle),
];

/// Os descritores da família. ⚠️ **ORDENADOS por `canonical_name`** — há gate.
pub const DESCS: &[ComponentDesc] = &[
    // ⭐⭐⭐ **A VIGIA DO CONTADOR** — o elo que faz um NÚMERO fazer acontecer alguma coisa, e o
    // último buraco que a medição da composição encontrou depois de o TOP-20 fechar.
    //
    // ⛔⛔ **Ela NÃO requer o `Counter`, e a ausência é a decisão.** O alvo é o NOME, logo o sítio
    // natural para as regras de um jogo é um objecto VAZIO chamado «Regras», que não tem contador
    // nenhum — exigi-lo poria um contador órfão em toda cena bem montada, e o painel deixaria de
    // poder distinguir *«falta-te o contador»* de *«este é o que eu quis»*.
    //
    // ⚠️ **E não há descritor para o `CounterWatchRuntime`**, pela linha que o `CounterRuntime` já
    // escreve: a ARESTA é viva, o undo não a fotografa, e descrevê-la aqui prometeria ao Inspector
    // um valor que ele não deve mostrar nem editar.
    D::authored(
        "ph2d::ecs::CounterWatch",
        "Counter Watch",
        C::Logic,
        O::ANY,
        WATCH_FIELDS,
    ),
    // ⭐⭐ **A HIGIENE do ciclo de vida** (TOP-20 #12) — sem ela a fábrica e o projéctil VAZAM.
    //
    // ⚠️ **Estas duas vivem na RECEITA e correm nas CÓPIAS**: um mestre está escondido por
    // construção, então o sítio onde se autoram é exactamente o sítio onde fazem efeito. Num
    // objecto solto elas são inertes, e é o painel que o diz.
    D::authored(
        "ph2d::ecs::DestroyOutside",
        "component.destroy_outside.name",
        C::Logic,
        O::ANY,
        OUTSIDE_FIELDS,
    ),
    // ⭐⭐⭐ **A FÁBRICA** (TOP-20 #11) — a categoria que quase nenhuma engine grande tem como
    // componente. `O::ANY` pela razão do relógio: quem fabrica é quase sempre um objecto VAZIO.
    //
    // ⭐⭐ **Ela REQUER o `Timers`**, e isso é o mecanismo do levantamento §1.4 (*required
    // components*, o meta-matador de UX) a pagar-se: acrescentar a fábrica traz o relógio que lhe
    // dá ritmo, já ligado. ⛔ É o que torna honesto **não** ter um `rate` próprio.
    D::authored_requiring(
        "ph2d::ecs::Factory",
        "component.factory.name",
        C::Logic,
        O::ANY,
        FACTORY_FIELDS,
        &["ph2d::ecs::Timers"],
    ),
    D::authored(
        "ph2d::ecs::Lifetime",
        "component.lifetime.name",
        C::Logic,
        O::ANY,
        LIFETIME_FIELDS,
    ),
    // ⭐⭐⭐ **A CUTSCENE** (TOP-20 #19) — este objecto toca um container da timeline.
    //
    // ⭐⭐ **Ele REQUER os `Timers`**, e é o mesmo mecanismo da fábrica: o relógio de corrida JÁ
    // existe (duração · repetir · o sinal a cada disparo · o decorrido vivo), e um sinal já o
    // arranca com o `StartTimer`. ⛔ É isso que torna honesto este descritor ter **um** campo: um
    // `Duration` aqui seria um segundo relógio, e a cutscene correria num tempo e anunciar-se-ia
    // noutro.
    //
    // ⚠️ `O::ANY` pela razão do relógio e da fábrica: quem toca uma cutscene é quase sempre um
    // objecto VAZIO («o realizador da cena»).
    D::authored_requiring(
        "ph2d::ecs::SequencePlayer",
        "Sequence Player",
        C::Logic,
        O::ANY,
        SEQUENCE_FIELDS,
        &["ph2d::ecs::Timers"],
    ),
    // ⭐⭐⭐ **O consumidor que faltava aos sinais** — e `O::ANY` pela mesma razão do relógio: quem
    // reage a um sinal é tantas vezes um objecto VAZIO («o cérebro da cena») quanto uma sprite.
    D::authored(
        "ph2d::ecs::SignalActions",
        "component.signal_actions.name",
        C::Logic,
        O::ANY,
        ACTION_FIELDS,
    ),
    // ⭐⭐⭐ **O CÉREBRO AUTORÁVEL** (TOP-20 #15) — `O::ANY` pela mesma razão do relógio e da
    // tabela: quem pensa é tantas vezes um objecto VAZIO quanto uma sprite.
    //
    // ⚠️ **Entre o `SignalActions` e o `Timers`, e isso NÃO é estilo:** a lista é procurada por
    // busca binária, e fora de ordem o descritor devolve `None` para um tipo que existe — *um
    // descritor que não é encontrado lê-se exactamente como um que não existe*.
    D::authored(
        "ph2d::ecs::StateMachine",
        "component.state_machine.name",
        C::Logic,
        O::ANY,
        STATE_MACHINE_FIELDS,
    ),
    // ⚠️ **`O::ANY`, e é a decisão**: um relógio serve a um sprite, a uma forma, a um objecto
    // VAZIO e a um grupo. Restringi-lo a `DRAWABLE` faria o objecto vazio — a entidade que o
    // artista usa como *«o cérebro da cena»* — não poder ter um.
    D::authored(
        "ph2d::ecs::Timers",
        "component.timers.name",
        C::Logic,
        O::ANY,
        TIMER_FIELDS,
    ),
];
