//! ⭐⭐⭐ **O GATILHO — a mão do artista vira um SINAL** (suplente #24 do TOP-20, a metade que o
//! `WeaponFire` pedia).
//!
//! # Porque ele existe: o censo dos PRODUTORES
//!
//! Medido em 2026-09-18, o [`ph2d_runtime::SignalOrigin`] tinha **treze** variantes — a timeline, o
//! contacto, o controlo, o movimento, a animação, o relógio, o nascimento, a morte, o cérebro, o
//! script, as partículas, o botão do HUD e a vigia do contador — e **nenhuma era a mão de quem
//! joga**. ⇒ *a tabela de acções do #5 sabia reagir a tudo menos a uma tecla.*
//!
//! ⚠️ **E a ausência não era do substrato:** a [`ph2d_input::Input`] já resolve acções **nomeadas**
//! com as três leituras que uma lei de gatilho precisa (`pressed` · `just_pressed` ·
//! `just_released`), já as torna religáveis pelo Input Map e já grava **a acção resolvida** na fita
//! determinística. O que faltava era **um componente que as ouça**.
//!
//! # ⭐⭐ O que esta lei NÃO tem, e porquê
//!
//! ⛔ **Não há `SignalOnActionRuntime`**, e isso não é economia — é a medição: *a aresta já é
//! trabalho do INPUT*. A [`ph2d_input::ActionState`] guarda **um tique atrás** de propósito (é o que
//! paga o `just_pressed` dela), logo um estado vivo aqui seria **a segunda resposta** a *«ela já
//! estava premida?»* — e as duas divergiriam no primeiro quadro em que a fita reproduzisse um
//! passado diferente do presente.
//!
//! ⇒ as cinco irmãs registadas desta linha (`Timer` · `Factory` · `CounterWatch` · `StateMachine` ·
//! `GameCamera`) têm todas um `*Runtime` não-registado e uma entrada no
//! [`crate::rewind_runtime`]. **Esta não tem nenhum dos dois, e o gate
//! `o_gatilho_nao_guarda_estado` afirma-o** — senão a ausência lê-se como esquecimento.
//!
//! # ⚠️ A lei da acção que NÃO EXISTE
//!
//! Uma linha que nomeie uma acção ausente do mapa fica **calada**, e isso é de graça: as três
//! leituras da `Input` são `is_some_and`, logo um nome desconhecido devolve `false` nas três. É a
//! mesma lei que a vigia do contador escreve para um contador inexistente (*«um contador ausente
//! leria zero, e uma regra `AtMost 0` anunciaria «morreste» no arranque»*) — aqui o defeito mudo
//! equivalente seria um `Release` a disparar **em todo quadro** sobre uma acção que ninguém ligou.
//!
//! # ⛔ E o que ela deixa de fora, com o número
//!
//! Não há filtro por jogador. O `Input` deste app é **um** (o *override* por-jogador em `~/.ph2d/`
//! é item aberto do Input Map desde 24/08), e um campo `player` aqui seria um knob morto — a
//! espécie que o `CLAUDE.md` §5.0 nomeia.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Quantos gatilhos uma entidade pode ter.**
///
/// ⚠️ O número é o da superfície, não um palpite: esta secção do Inspector é a do
/// [`crate::CounterWatch`] com outros campos (lista + editor por baixo), logo o tecto é o mesmo
/// [`crate::WATCHES_MAX`] — *um modelo que aceita o que o painel não mostra produz estado
/// inalcançável* (a lei que a §11 Animation pagou com o `ANIM_TAGS_MAX`).
pub const ACTION_TRIGGERS_MAX: usize = 16;

/// **QUANDO a linha fala.**
///
/// ⚠️ **`Press` é o valor de fábrica e é o único que dá UM tiro por toque.** O `Hold` fala em
/// **todo** quadro em que a tecla está em baixo — é o que um lança-chamas quer e o que uma pistola
/// não quer, e é por isso que ele existe e não é o padrão.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionEdge {
    /// A tecla passou de solta a premida — **uma** vez por toque.
    #[default]
    Press,
    /// Passou de premida a solta.
    Release,
    /// Está premida **agora** — logo fala a cada quadro.
    Hold,
}

impl ActionEdge {
    /// As três, pela ordem em que o chip as oferece. ⚠️ A posição **é** a tag serializada.
    pub const ALL: [Self; 3] = [Self::Press, Self::Release, Self::Hold];
}

/// **A amostra de uma acção neste quadro** — as três leituras que a lei consome.
///
/// ⚠️ Ela existe para a lei ser **pura**: a `ph2d-ecs` não depende da `ph2d-input`, e quem constrói
/// isto é a ponte da shell, que tem o mapa e o estado na mão. *Um `use ph2d_input` aqui ligaria o
/// modelo de dados do projecto ao vocabulário de um dispositivo.*
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Amostra {
    /// Está premida agora.
    pub pressed: bool,
    /// Passou de solta a premida **neste** tique.
    pub just_pressed: bool,
    /// Passou de premida a solta **neste** tique.
    pub just_released: bool,
}

/// **Uma linha:** *quando a acção `action` `edge`, diz `signal`.*
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionTriggerRow {
    /// **Qual acção — pelo NOME.** ⚠️ A lei da casa: o nome viaja. E aqui ela é mais forte do que
    /// o costume, porque o [`ph2d_input::ActionId`] é um **contador estável** de que renomear e
    /// reordenar não mexem — o nome é o que o artista escreve no Input Map e o que a fita grava.
    pub action: String,
    /// Quando.
    pub edge: ActionEdge,
    /// **O que emitir.** Vazio = a linha segue a tecla e fica **calada** — uma configuração a meio,
    /// nunca um erro (a lei do `inert` da tabela de acções e do `signal` da vigia).
    pub signal: String,
}

/// **Os gatilhos de uma entidade** — o componente registado. ⚠️ CONFIG, como o `Timers`.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalOnAction(pub Vec<ActionTriggerRow>);

impl SimComponent for SignalOnAction {}

/// ⭐⭐⭐ **A LEI.** Devolve `true` sse esta linha fala **agora**.
///
/// ⚠️ **O nome vazio é decidido AQUI e não no chamador**, porque é isso que faz a linha calada
/// continuar a ser uma linha: ela aparece no painel, aceita edição e não publica nada. Um `if` do
/// lado da ponte poria a mesma decisão em dois sítios no dia em que um segundo consumidor da lei
/// aparecesse.
#[must_use]
pub fn fala(row: &ActionTriggerRow, a: Amostra) -> bool {
    if row.signal.trim().is_empty() {
        return false;
    }
    match row.edge {
        ActionEdge::Press => a.just_pressed,
        ActionEdge::Release => a.just_released,
        ActionEdge::Hold => a.pressed,
    }
}

/// **Um disparo:** quem falou, por que linha, e o quê.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionFired {
    /// A entidade que carrega o gatilho.
    pub source: Entity,
    /// **Qual** das linhas falou, pela posição — a mesma forma que a vigia do contador usa, e pela
    /// mesma razão: o [`ph2d_runtime::SignalOrigin`] é `Copy` e um nome lá dentro tirava o `Copy` a
    /// **todas** as origens.
    pub row: u16,
    /// O nome autorado.
    pub signal: String,
}

/// **Todos os disparos deste quadro**, pela ordem da identidade.
///
/// ⚠️ **A ordem é por [`crate::StableId`] e não pela iteração do mundo** — a mesma cerca que a
/// fábrica escreve: *entre arquétipos diferentes a ordem de iteração é indefinida por escrito*, e
/// dois sinais publicados por ordem trocada leem-se como dois jogos diferentes numa fita.
///
/// O `lida` é a ponte: ela recebe o **nome** da acção e devolve a amostra dela.
pub fn dispara(world: &mut World, lida: &dyn Fn(&str) -> Amostra) -> Vec<ActionFired> {
    let mut candidatas: Vec<(Entity, SignalOnAction)> = world
        .query::<(Entity, &SignalOnAction)>()
        .iter(world)
        .map(|(e, g)| (e, g.clone()))
        .collect();
    {
        let mundo: &World = world;
        candidatas.sort_by_key(|(e, _)| mundo.get::<crate::StableId>(*e).map_or(u64::MAX, |s| s.0));
    }
    let mut out = Vec::new();
    for (e, g) in candidatas {
        for (i, row) in g.0.iter().enumerate().take(ACTION_TRIGGERS_MAX) {
            if fala(row, lida(&row.action)) {
                out.push(ActionFired {
                    source: e,
                    row: u16::try_from(i).unwrap_or(u16::MAX),
                    signal: row.signal.clone(),
                });
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "signal_on_action_tests.rs"]
mod tests;
