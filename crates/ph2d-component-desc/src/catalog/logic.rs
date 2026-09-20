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
const SEQUENCE_FIELDS: &[FieldDesc] = &[f(1, "component.field.sequence_fields.1", K::Text)];

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

/// ⭐⭐⭐ **O que o SEGUIDOR DE CAMINHO guarda** (suplente #23) — e o primeiro campo decide tudo.
///
/// ⚠️ **`Path` é o NOME da forma desenhada**, não um id nem um caminho de ficheiro: é a lei do
/// `stable_name_id` desta casa, e é o que faz renomear a curva na Hierarquia levar o seguidor com
/// ela. ⛔ A GEOMETRIA não é campo nenhum — ela vive no documento vectorial, e o `ph2d-ecs` nem o
/// vê (medido).
///
/// ⛔ **Não há campo de DURAÇÃO**, pela mesma razão do tween e da fábrica: o tempo vem do `Timers`.
const PATH_FOLLOW_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.path_follow_fields.1", K::Text),
    f(2, "component.field.path_follow_fields.2", K::Scalar),
    f(3, "component.field.path_follow_fields.3", K::Enum),
    f(4, "component.field.path_follow_fields.4", K::Enum),
    f(5, "component.field.path_follow_fields.5", K::Enum),
    f(6, "component.field.path_follow_fields.6", K::Enum),
    f(7, "component.field.path_follow_fields.7", K::Scalar),
    f(8, "component.field.path_follow_fields.8", K::Toggle),
    f(9, "component.field.path_follow_fields.9", K::Angle),
    f(10, "component.field.path_follow_fields.10", K::Scalar),
];

/// **Os campos de UM tween** (suplente #22) — como o `Timers`, o componente é uma LISTA e isto
/// descreve UM elemento dela.
///
/// ⛔ **Não há campo de DURAÇÃO**, e é a mesma decisão medida que a fábrica declara logo abaixo: o
/// tempo vem do `Timers`, que o `requires` puxa quando o artista acrescenta o tween. Um `duration`
/// aqui seria um **segundo relógio** para a mesma lei.
///
/// ⚠️ **`From` e `To` são `Vec4` porque a ARIDADE é do canal** (`ph2d_tween::Canal::aridade`): o
/// painel pinta um campo ou quatro, e o descritor descreve o que o modelo guarda. ⛔ Dois pares de
/// campos — um escalar e um de cor — seriam duas respostas a *«de onde para onde?»*.
const WEAPON_FIELDS: &[FieldDesc] = &[
    // ⚠️ **Vazio = nunca dispara**, a lei do consumidor de sinal desta casa.
    f(1, "component.field.weapon_fields.1", K::Text),
    f(2, "component.field.weapon_fields.2", K::Scalar),
    // ⭐⭐ **O NOME do contador que É o pente** — vazio = munição infinita. ⛔ A munição não é um
    // campo deste componente, e é isso que a põe no HUD (`LabelSource::Counter`) de graça.
    f(3, "component.field.weapon_fields.3", K::Text),
    f(4, "component.field.weapon_fields.4", K::Scalar),
    f(5, "component.field.weapon_fields.5", K::Text),
    // ⭐ **O fio para a `Factory`**: sem ele a arma dispara e nada nasce.
    f(6, "component.field.weapon_fields.6", K::Text),
    f(7, "component.field.weapon_fields.7", K::Text),
    f(8, "component.field.weapon_fields.8", K::Text),
];

const TWEEN_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.tween_fields.1", K::Enum),
    f(2, "component.field.tween_fields.2", K::Vec4),
    f(3, "component.field.tween_fields.3", K::Vec4),
    f(4, "component.field.tween_fields.4", K::Enum),
    f(5, "component.field.tween_fields.5", K::Enum),
    f(6, "component.field.tween_fields.6", K::Enum),
];

/// ⭐⭐⭐ **Uma REGRA da vigia** — como o `Timers`, o componente é uma LISTA e isto descreve a LINHA.
///
/// ⚠️ **`Counter` é o NOME do contador e não o deste objecto** — a vigia pode viver num objecto
/// «Regras» sem contador nenhum, e é isso que a torna autorável num sítio só.
/// ⭐⭐⭐ **Uma LINHA do gatilho** — como a vigia, o componente é uma LISTA e isto descreve a LINHA.
///
/// ⚠️ **`Action` é o nome da acção do INPUT MAP e não o de uma tecla** — é ele que sobrevive a um
/// remapeamento, e é ele que a fita determinística grava.
const TRIGGER_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.trigger_fields.1", K::Text),
    f(2, "component.field.trigger_fields.2", K::Enum),
    // ⚠️ **Vazio = calada** — a lei da vigia e da §11.
    f(3, "component.field.trigger_fields.3", K::Text),
];

const WATCH_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.watch_fields.1", K::Text),
    f(2, "component.field.watch_fields.2", K::Enum),
    f(3, "component.field.watch_fields.3", K::Int),
    // ⚠️ **Vazio = calada** — a lei da §11, a mesma do `Signal` do relógio logo abaixo.
    f(4, "component.field.watch_fields.4", K::Text),
    f(5, "component.field.watch_fields.5", K::Toggle),
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
        "component.counter_watch.name",
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
    // ⭐⭐⭐ **O SEGUIDOR DE CAMINHO** (suplente #23) — este objecto anda sobre a curva desenhada.
    //
    // ⚠️ **DEPOIS do `Lifetime` e ANTES do `SequencePlayer`:** a lista é procurada por busca
    // binária, e fora de ordem o descritor devolve `None` para um tipo que existe.
    //
    // ⭐ **`requires` os `Timers`**, como o tween e a cutscene: sem relógio ele não é meia feature,
    // é uma feature **INERTE**, e a paleta sabe dizê-lo antes de o artista descobrir.
    //
    // ⚠️ `O::ANY` pela razão do tween: um objecto vazio que seja o pai de um grupo também patrulha.
    D::authored_requiring(
        "ph2d::ecs::PathFollow",
        "component.path_follow.name",
        C::Logic,
        O::ANY,
        PATH_FOLLOW_FIELDS,
        &["ph2d::ecs::Timers"],
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
        "component.sequence_player.name",
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
    // ⭐⭐⭐ **O GATILHO** (suplente #24) — a mão de quem joga, o 14.º produtor de sinal e o
    // primeiro cuja entrada não é o mundo nem o barramento.
    //
    // ⚠️ **`O::ANY` pela mesma razão do relógio e da tabela:** o sítio natural para as teclas de um
    // jogo é um objecto VAZIO chamado «Controlos», que não tem sprite nenhuma.
    //
    // ⛔ **Ele NÃO requer o `SignalActions`, e a ausência é a decisão** — o mesmo argumento que a
    // vigia escreve para o `Counter`: quem ouve o sinal pode ser a FÁBRICA, o cérebro ou uma tabela
    // noutro objecto, e exigi-lo poria uma tabela vazia em toda cena bem montada.
    //
    // ⚠️ **Entre o `SignalActions` e o `StateMachine`** — a lista é procurada por busca binária, e
    // fora de ordem o descritor devolve `None` para um tipo que existe.
    D::authored(
        "ph2d::ecs::SignalOnAction",
        "component.signal_on_action.name",
        C::Logic,
        O::ANY,
        TRIGGER_FIELDS,
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
    // ⚠️ **DEPOIS do `Timers`, e isso NÃO é estilo:** a lista é procurada por busca binária, e fora
    // de ordem o descritor devolve `None` para um tipo que existe — *um descritor que não é
    // encontrado lê-se exactamente como um que não existe*.
    //
    // ⚠️ **`O::ANY`, pela MESMA razão do `Timers`:** um tween de POSE serve a um objecto vazio que
    // seja o pai de um grupo, e restringi-lo a `DRAWABLE` tiraria isso a quem o usa como pivô.
    //
    // ⭐ **`requires` os `Timers`**, como o `SequencePlayer` e a `Factory`: sem relógio ele não é
    // meia feature, é uma feature **INERTE** — e a casa tem como o dizer na paleta em vez de deixar
    // o artista descobrir.
    D::authored_requiring(
        "ph2d::ecs::Tweens",
        "component.tweens.name",
        C::Logic,
        O::ANY,
        TWEEN_FIELDS,
        &["ph2d::ecs::Timers"],
    ),
    // ⭐⭐⭐ **A ARMA DO JOGADOR** — o RITMO, o PENTE e a recarga. `O::ANY` pela razão do `Timers`:
    // uma torreta é um objecto VAZIO com uma arma e uma fábrica.
    //
    // ⚠️ **DEPOIS do `Tweens`, e isso NÃO é estilo:** a lista é procurada por busca binária, e fora
    // de ordem o descritor devolve `None` para um tipo que existe — *um descritor que não é
    // encontrado lê-se exactamente como um que não existe*.
    //
    // ⭐⭐ **Ela REQUER o `Counter`**, e isso é o mecanismo dos *required components* a pagar-se
    // outra vez: o pente É um contador, e a ponte lê-o **nesta entidade** (a soma global é a
    // pergunta certa para um placar e a errada para uma escrita, que precisa de um dono). ⛔ Sem o
    // `requires`, a arma nasceria com munição infinita em silêncio e o artista descobria-o a jogar.
    D::authored_requiring(
        "ph2d::ecs::WeaponFire",
        "component.weapon_fire.name",
        C::Logic,
        O::ANY,
        WEAPON_FIELDS,
        &["ph2d::ecs::Counter"],
    ),
];
