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

const fn f(field_id: u16, name: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
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
    f(1, "Name", K::Text),
    // ⚠️ **Em SEGUNDOS no painel, microssegundos no modelo.** A conversão vive nas duas pontas do
    // `render_loop::inspector_timer`, e há gate de ida-e-volta.
    f(2, "Duration", K::Scalar),
    f(3, "Repeat", K::Toggle),
    // ⚠️ **É este o campo autorado** — o «está a correr agora» é vivo e não chega ao Inspector.
    f(4, "Autostart", K::Toggle),
    // ⚠️ **Vazio = calado** — a lei da §11: um produtor sem nome não fala, em vez de falar com um
    // nome vazio.
    f(5, "Signal", K::Text),
];

/// **Os campos de UMA linha da tabela `SignalActions`** — *quando o sinal `on` chegar, faz `verb`
/// em `target`*.
///
/// ⚠️ **O `target` é `Text` e não `Ref`**, e a distinção é a lei do CLAUDE.md §5: *referência
/// durável entre objectos é o NOME, nunca os bits*. Um `FieldKind::Ref` prometeria um picker que
/// guarda uma identidade — e o undo respawna tudo com bits novos.
const ACTION_FIELDS: &[FieldDesc] = &[
    f(1, "On Signal", K::Text),
    f(2, "Target", K::Text),
    f(3, "Action", K::Enum),
    f(4, "Timer", K::Text),
];

/// Os descritores da família.
pub const DESCS: &[ComponentDesc] = &[
    // ⭐⭐⭐ **O consumidor que faltava aos sinais** — e `O::ANY` pela mesma razão do relógio: quem
    // reage a um sinal é tantas vezes um objecto VAZIO («o cérebro da cena») quanto uma sprite.
    D::authored(
        "ph2d::ecs::SignalActions",
        "Signal Actions",
        C::Logic,
        O::ANY,
        ACTION_FIELDS,
    ),
    // ⚠️ **`O::ANY`, e é a decisão**: um relógio serve a um sprite, a uma forma, a um objecto
    // VAZIO e a um grupo. Restringi-lo a `DRAWABLE` faria o objecto vazio — a entidade que o
    // artista usa como *«o cérebro da cena»* — não poder ter um.
    D::authored(
        "ph2d::ecs::Timers",
        "Timers",
        C::Logic,
        O::ANY,
        TIMER_FIELDS,
    ),
];
