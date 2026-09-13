//! ⭐⭐⭐ **O `SignalActions`** — a tabela **nome → acção**, e o item que o levantamento chama de
//! *«o mais importante»* do TOP-20 (#5).
//!
//! # O que ele fecha
//!
//! Até 2026-09-09 o `ph2d-runtime` publicava sinais de **cinco** origens (timeline · contacto ·
//! sensor · animação · **relógio**) e tinha **três** consumidores, todos de diagnóstico: um toast,
//! uma linha de terminal e a máquina de estados de UI. *Um sinal não fazia nada acontecer na cena.*
//!
//! Este componente é o consumidor que faltava: uma lista de linhas `(sinal, alvo, verbo)` que o
//! artista escreve no Inspector. Com ela, **um sinal vira jogo sem uma linha de script** — que é a
//! frase com que o levantamento justifica o bloco 2–5 inteiro.
//!
//! # ⚠️ As leis que este módulo herda, e onde cada uma foi paga
//!
//! - ⭐⭐ **O alvo é o NOME, nunca os bits.** É a lei do `stable_name_id` escrita no CLAUDE.md §5:
//!   *o undo respawna tudo com bits novos, e bits dentro dos bytes de um componente envenenam o
//!   próprio undo*. ⇒ o alvo viaja como `String`, e a resolução é por [`crate::stable_id`].
//!
//!   ⭐ E há um segundo ganho, medido no desenho: **uma cópia de um prefab funciona sem rewiring**
//!   quando o alvo é vazio (= *este objecto*), que é o caso comum de uma porta, de uma armadilha
//!   ou de um inimigo.
//! - **Um sinal sem nome nunca dispara** — o espelho exacto da lei do produtor (*«um produtor sem
//!   nome não fala, em vez de falar com um nome vazio»*). Aqui: um consumidor sem nome não escuta,
//!   em vez de escutar tudo.
//! - ⛔ **NÃO há verbo que EMITA um sinal**, e a ausência é a decisão: seria a classe inteira dos
//!   laços (`a` dispara `b` dispara `a`), e não existe hoje consumidor que a peça. Quando existir,
//!   ela entra com um orçamento de profundidade, não com um `if`.
//!
//! # ⚠️ O que a RESOLUÇÃO faz, e o que ela deliberadamente NÃO faz
//!
//! [`resolve`] é **pura sobre o mundo**: ela lê, casa nomes e devolve a lista de efeitos. Quem os
//! **aplica** é a shell, e a razão é o undo — escrever a [`crate::Visibility`] de um objecto é
//! escrever um componente **registado**, logo um passo de `Ctrl+Z` por cada porta que abre. A shell
//! declara essas escritas ao ledger de pré-visualização (*o documento é o valor AUTORADO; o que um
//! motor escreve agora é pré-visualização*), e essa maquinaria não vive nesta crate.
//!
//! ⇒ **duas metades, duas casas**, e a fronteira é exactamente a que o `tick_timers` já usa: a lei
//! devolve factos, a ponte publica-os.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{Entity, Name, World};
use ph2d_tags::{TagId, TagTree};

/// Quantas acções uma entidade pode ter.
///
/// ⚠️ **O número sai do PAINEL**, como o [`crate::TIMERS_MAX`] e pela mesma lei: *um modelo que
/// aceita o que o painel não mostra produz estado inalcançável*. A secção do Inspector desenha uma
/// linha por acção dentro do dock, e `16` é o que cabe sem a secção sozinha passar a altura útil da
/// coluna.
pub const SIGNAL_ACTIONS_MAX: usize = 16;

/// **O que uma acção FAZ.**
///
/// ⛔ **A lista é o que tem SINK hoje, e não o que seria bonito ter.** Um verbo cujo consumidor não
/// existe é um controlo morto com cara de feature — o defeito que a caça de 2026-08-30 mediu em 34
/// controlos. Cada entrada abaixo escreve num componente que existe e que alguém lê.
///
/// # ⏳ Os que ficam de FORA, com o motivo
///
/// - ✅ **Tocar um som DEIXOU de estar aqui** — a recusa dizia *«não existe `AudioSource2D` na
///   árvore»* e ela dissolveu no dia seguinte, porque **esta linha construiu-o** (TOP-20 #4). É o
///   §0.0 outra vez: *quem move o número que tornava algo inalcançável tem de reconferir a nota.*
///   ⇒ os verbos `PlaySound`/`StopSound` estão na lista abaixo.
/// - **Tocar uma animação nomeada.** ⏳ O sink existe (`SpriteAnimator::current`/`playing`), e o
///   que o barra é uma **colisão de granularidade**: o ledger já conduz aquele componente pelo
///   `Driver::SpriteAnim`, cujo recorte são os três campos do RELÓGIO mais a célula. Um segundo
///   motor sobre os campos autorados do mesmo componente precisa do seu próprio recorte, e isso é
///   desenho, não uma linha.
/// - **Nascer um objecto** (*spawn*). ⏳ É o item **#11** da fila e precisa de uma referência a uma
///   receita, que é outro campo e outro picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignalVerb {
    /// **Arranca um relógio do alvo.** O `arg` é o NOME do timer; vazio = **todos** os do alvo.
    ///
    /// ⭐ É o verbo que fecha o buraco que a W3 do Timer deixou nomeado: até aqui o `autostart` era
    /// o único caminho para um timer começar, e um timer com ele desligado era inalcançável.
    #[default]
    StartTimer,
    /// **Pára um relógio do alvo**, guardando o progresso — é o *Pause* do Godot, não o *Stop*, e a
    /// escolha é a que a lei pura do timer já declara (`running = false` não zera o `elapsed`).
    StopTimer,
    /// Mostra o alvo.
    Show,
    /// Esconde o alvo.
    Hide,
    /// Inverte o que o alvo está a fazer agora.
    ToggleVisibility,
    /// **Toca o som do alvo** (o `AudioSource2D` dele). ⛔ Um alvo sem fonte de som não faz nada —
    /// não é erro, é a mesma lei de um alvo que não existe.
    ///
    /// ⚠️ **Ele NÃO lê o `arg`, e é deliberado:** o ficheiro é do componente, não da linha da
    /// tabela. Pôr um caminho aqui daria **duas** respostas a *«que som é este objecto?»*, e a que
    /// o artista vê no Inspector seria a que envelhece.
    PlaySound,
    /// **Cala o som do alvo** — pára as vozes que ele tem a soar agora.
    StopSound,
}

impl SignalVerb {
    /// Todos, em ordem — **a fonte da iteração**. ⛔ Nunca escreva a lista uma segunda vez.
    /// ⚠️ **APPEND-ONLY**: a posição é a tag e ela viaja no ficheiro. Um verbo novo entra no FIM.
    pub const ALL: [SignalVerb; 7] = [
        SignalVerb::StartTimer,
        SignalVerb::StopTimer,
        SignalVerb::Show,
        SignalVerb::Hide,
        SignalVerb::ToggleVisibility,
        SignalVerb::PlaySound,
        SignalVerb::StopSound,
    ];

    /// O rótulo que o artista lê. Inglês (HR-15).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SignalVerb::StartTimer => "Start Timer",
            SignalVerb::StopTimer => "Stop Timer",
            SignalVerb::Show => "Show",
            SignalVerb::Hide => "Hide",
            SignalVerb::ToggleVisibility => "Toggle Visibility",
            SignalVerb::PlaySound => "Play Sound",
            SignalVerb::StopSound => "Stop Sound",
        }
    }

    /// **Este verbo LÊ o `arg`?** — é o que decide se o painel pinta o campo.
    ///
    /// ⚠️ **Derivado do verbo, nunca uma segunda lista.** Um painel que mostra um campo que o verbo
    /// não lê é um controlo morto; um que o esconde onde o verbo o lê é uma feature inalcançável.
    #[must_use]
    pub const fn uses_arg(self) -> bool {
        matches!(self, SignalVerb::StartTimer | SignalVerb::StopTimer)
    }

    /// A posição em [`Self::ALL`] — a tag que o painel usa nos segmentados.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O verbo desta posição, ou o primeiro. ⚠️ **A POSIÇÃO NO ARRAY É A TAG**, e reordenar
    /// [`Self::ALL`] faria um clique escrever outro verbo — e compila.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }
}

/// ⭐⭐ **A QUEM a acção se aplica** — pelo nome, ou a todos os que pertencem a uma tag (TOP-20 #9,
/// `docs/Components/08_plano_tags.md` §2.3).
///
/// ⚠️ **Ele é APENDADO ao fim do [`SignalAction`]** e o default é o de sempre, então uma linha que
/// nunca escolheu tag resolve exactamente como antes. O postcard é posicional, e é por isso que o
/// campo custa o degrau `128 -> 129` do `PROJECT_SCHEMA` e a migração em `signal_actions_v1.rs`.
///
/// ⛔ **Não é um segundo campo de texto ao lado do `target`**: com os dois, *«a quem?»* teria duas
/// respostas escritas ao mesmo tempo e o painel teria de escolher uma. Aqui a variante escolhe, e o
/// `target` só é lido pelo `Named`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalTarget {
    /// O objecto chamado [`SignalAction::target`] — vazio = **este objecto**. O de sempre, e o
    /// default: toda linha de um ficheiro v128 migra para aqui.
    #[default]
    Named,
    /// **Todos os que pertencem à tag**, com a subárvore dela, pela ordem da identidade
    /// ([`crate::tags::tagged`]). ⚠️ Inclui quem reage, se pertencer — o `call_group` do Godot
    /// (medido). Uma tag que já não existe = **ninguém** (a lei do alvo que não existe).
    ///
    /// ⚠️ `u64` e não [`TagId`] porque a folha das tags não fala `serde` (de propósito).
    Tagged(u64),
}

impl SignalTarget {
    /// A tag escolhida, se o alvo for por tag.
    #[must_use]
    pub const fn tag(self) -> Option<TagId> {
        match self {
            Self::Tagged(id) => Some(TagId(id)),
            Self::Named => None,
        }
    }
}

/// **Uma linha da tabela** — *quando o sinal `on` chegar, faz `verb` em `target`*.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalAction {
    /// O nome do sinal que a dispara. **Vazio = nunca** (ver o doc do módulo).
    pub on: String,
    /// O NOME do objecto que sofre a acção. **Vazio = este objecto.**
    ///
    /// ⚠️ **Nunca bits** — ver o doc do módulo. E o vazio é o caso comum, não uma abreviatura: é
    /// ele que faz uma cópia de prefab funcionar sem rewiring.
    pub target: String,
    pub verb: SignalVerb,
    /// O parâmetro do verbo — hoje, o nome do timer. Vazio quando o verbo não o lê
    /// ([`SignalVerb::uses_arg`]), e **vazio também significa «todos»** para os verbos de timer.
    pub arg: String,
    /// ⭐ **Por nome ou por tag** — ver [`SignalTarget`]. ⚠️ O ÚLTIMO campo, de propósito: é o que
    /// torna a migração de um v128 uma leitura com um tipo congelado e um re-encode.
    pub target_by: SignalTarget,
}

/// **A tabela de uma entidade** — o componente registado.
///
/// ⚠️ **Uma LISTA**, como o [`crate::Timers`] e o `NamedAnchorList`: um componente ECS é único por
/// entidade, e um objecto tem tipicamente várias reacções (*abre com `botao`, fecha com `alarme`*).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalActions(pub Vec<SignalAction>);

/// **O efeito que uma acção decidiu**, com o alvo já resolvido para uma entidade.
///
/// ⚠️ **A resolução vem feita**, e é isso que separa esta metade da outra: a shell recebe *«faz
/// isto NESTA entidade»* e não tem de saber o que é um nome. Um alvo que não existe **não produz
/// efeito nenhum** — ver [`resolve`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalEffect {
    /// Quem sofre.
    pub target: Entity,
    pub verb: SignalVerb,
    /// O `arg` da linha, copiado. Vazio = «todos», para os verbos que o lêem.
    pub arg: String,
    /// Quem reagiu — a entidade que carrega a tabela. ⚠️ **Não é o alvo**, e a distinção é a que
    /// torna um diagnóstico legível: *«a Porta reagiu ao sinal `botao` e escondeu a Parede»*.
    pub source: Entity,
}

/// ⭐⭐⭐ **A RESOLUÇÃO** — que efeitos os sinais deste quadro produzem.
///
/// # ⚠️ A ordem é DETERMINISTA, e não é a da query
///
/// Duas entidades que reajam ao mesmo sinal e escrevam no mesmo alvo têm de o fazer sempre na
/// mesma ordem, senão o replay diverge. A ordem da query do `bevy_ecs` é a do arquétipo, que muda
/// quando um componente é inserido — ⇒ a saída é ordenada pelo [`crate::StableId`] de quem reage,
/// que é a identidade que sobrevive ao respawn do undo por construção.
///
/// ⚠️ **Dentro de uma entidade, a ordem é a que o artista ESCREVEU.** Ela é visível na lista do
/// painel, e reordená-la seria o painel a mentir sobre o que acontece.
///
/// # ⛔ Um alvo que não existe é SILÊNCIO, e é deliberado
///
/// Um nome que não casa devolve zero efeitos — nunca um efeito sobre quem reagiu. *Cair no próprio
/// objecto quando o alvo desapareceu faria uma porta esconder-se a si mesma no dia em que alguém
/// renomeasse a parede*, e o artista leria isso como um defeito do motor.
///
/// ⚠️ **`&mut World` porque a resolução de nome ATRIBUI ids em falta** (`stable_id_for_name`), e
/// isso é uma escrita. Ela é idempotente e o `assign_missing_stable_ids` já corre no quadro.
///
/// ⚠️ **A árvore de tags entra por parâmetro**, e é o documento do PROJECTO (a shell guarda-a no
/// `AppGfx`): uma acção por tag pergunta quem pertence à subárvore, e a pertença de um objecto é só
/// uma lista de ids. Uma acção por nome nunca a lê.
#[must_use]
pub fn resolve(world: &mut World, tree: &TagTree, fired: &[&str]) -> Vec<SignalEffect> {
    if fired.is_empty() {
        return Vec::new();
    }
    // Quem reage, na ordem da identidade — nunca a da query.
    let mut reactors: Vec<(u64, Entity, SignalActions)> = world
        .query::<(Entity, &SignalActions, &crate::StableId)>()
        .iter(world)
        .filter(|(_, a, _)| !a.0.is_empty())
        .map(|(e, a, s)| (s.0, e, a.clone()))
        .collect();
    if reactors.is_empty() {
        return Vec::new();
    }
    reactors.sort_unstable_by_key(|(id, _, _)| *id);

    let mut out = Vec::new();
    for (_, source, table) in reactors {
        for action in &table.0 {
            // Um consumidor sem nome não escuta — o espelho da lei do produtor.
            if action.on.is_empty() || !fired.contains(&action.on.as_str()) {
                continue;
            }
            // ⚠️ A ordem dentro de uma linha é a dos ALVOS (pela identidade), depois da ordem dos
            // reactores e da ordem das linhas — as três, deterministas.
            for target in targets_of(world, tree, source, action) {
                out.push(SignalEffect {
                    target,
                    verb: action.verb,
                    arg: action.arg.clone(),
                    source,
                });
            }
        }
    }
    out
}

/// ⭐⭐ **Quem sofre esta acção** — a porta ÚNICA da pergunta *«a quem?»* (plano de Tags §2.2).
///
/// - [`SignalTarget::Named`]: o objecto com aquele nome, ou `source` com o nome vazio — zero ou um.
/// - [`SignalTarget::Tagged`]: todos os que pertencem à subárvore da tag, pela ordem do
///   [`crate::StableId`]; uma tag que já não existe dá **ninguém**.
///
/// ⛔ Resolver o alvo na shell seria a segunda resposta, e é a que envelhece.
#[must_use]
pub fn targets_of(
    world: &mut World,
    tree: &TagTree,
    source: Entity,
    action: &SignalAction,
) -> Vec<Entity> {
    match action.target_by {
        SignalTarget::Named => target_of(world, source, &action.target)
            .into_iter()
            .collect(),
        SignalTarget::Tagged(id) => crate::tags::tagged(world, tree, TagId(id)),
    }
}

/// O alvo de uma acção: `source` quando o nome é vazio, senão quem tiver aquele [`Name`].
///
/// ⚠️ **Ele confere que a entidade AINDA existe.** Entre o quadro em que o artista escreveu o nome
/// e este, um `Ctrl+Z` pode tê-la levado.
fn target_of(world: &mut World, source: Entity, name: &str) -> Option<Entity> {
    if name.is_empty() {
        return world.get_entity(source).ok().map(|_| source);
    }
    let id = crate::stable_id_for_name(world, name);
    crate::entity_of_stable_id(world, crate::StableId(id))
}

/// **O nome do objecto que este efeito atinge**, para um diagnóstico legível. `None` = sem `Name`.
#[must_use]
pub fn name_of(world: &World, entity: Entity) -> Option<String> {
    world.get::<Name>(entity).map(|n| n.as_str().to_string())
}

/// O `SignalAction` como o `PROJECT_SCHEMA` 128 o gravava, congelado para a migração.
#[path = "signal_actions_v1.rs"]
mod v1;
pub use v1::migrate_v1_blob;

#[cfg(test)]
#[path = "signal_actions_tests.rs"]
mod tests;
