//! ⭐⭐⭐ **A SECÇÃO *Component* DO PAINEL VETORIAL, LIGADA AO MECANISMO GERAL** (F4.6c, wave 1).
//!
//! # O que esta wave resolve
//!
//! O app tem **dois** motores para a mesma feature. O geral (ADR-0164) faz de uma cópia uma
//! **sub-árvore de entidades reais**; o vetorial ([`crate::vec_component_edit`]) guarda um
//! **rectângulo de suporte** e **deriva o desenho por quadro**. As diferenças que o artista vê:
//!
//! | | vetorial (velho) | geral (novo) |
//! |---|---|---|
//! | a cópia na Hierarquia | **uma** linha (o suporte) | a sub-árvore, com as peças nomeadas |
//! | editar uma peça da cópia | só pelos slots do painel (`MAX_INSTANCE_PIECES`) | seleccionar a peça e usar qualquer ferramenta |
//! | onde a excepção mora | `VecInstance.overrides` | `ObjectInstance.overrides` |
//!
//! *Dois motores para o mesmo estado é pior que um motor lento* — e é a razão pela qual a F4.6c
//! existe no plano desde o início.
//!
//! # ⛔ Por que ele nasce DESLIGADO, e isto não é timidez
//!
//! Esta secção é um fluxo que **o dono usa** e que **eu não posso smokar** — ela vive no editor
//! vetorial, atrás de uma janela. Trocar o motor por baixo dela e entregar sem rede seria pedir-lhe
//! que descobrisse a regressão por mim. ⇒ `PH2D_VEC_COMPONENT_GENERAL=1` **arma** o modo novo, e
//! sem a variável **nada muda, por construção**: o `armed()` é a única porta, e ela é lida nos
//! **dois** sítios que decidem (o que o painel MOSTRA e o que o clique FAZ).
//!
//! ⚠️ **É o precedente do `PH2D_RETOPO_EXTRACT`**: o motor novo shipa desligado, com a tabela da
//! comparação ao lado, e o caminho de omissão fica **byte-idêntico**.
//!
//! # ⚠️ O que o modo novo ainda NÃO oferece — e por que isso não deixa botão morto
//!
//! Três verbos do motor velho não têm ainda equivalente publicado aqui: o **conta-gotas** do *Swap*,
//! a lista de **peças** (o olho por peça) e a fileira de **variants**. ⇒ no modo novo a shell
//! publica essas listas **VAZIAS**, e o painel simplesmente **não as pinta** — a secção oferece
//! exactamente o que o gesto faz. *Um controlo que aparece e não faz nada é o defeito que este
//! repo caça; um que não aparece é uma feature por construir, e ela está nomeada.*

use ph2d_ecs::{Entity, MasterRoot, SimWorld};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// **O modo geral está armado?** Porta única — ver o cabeçalho.
///
/// ⚠️ **Lida em DOIS sítios** (o que o painel mostra e o que o clique faz) e por isso é uma função,
/// não um `if` copiado: os dois a discordarem dá uma secção que oferece um verbo que o dreno recusa.
#[must_use]
pub(crate) fn armed() -> bool {
    std::env::var("PH2D_VEC_COMPONENT_GENERAL").is_ok_and(|v| v != "0")
}

/// A forma seleccionada (uma só) e a entidade dela — o mesmo critério do motor velho.
///
/// ⚠️ **Uma só, de propósito:** um componente é sobre UMA coisa, e *«criar componente»* a partir de
/// duas selecções é outra operação (agrupar primeiro).
fn subject(map: &VecEntityMap, selected: &[VecPathId]) -> Option<Entity> {
    let [only] = selected else { return None };
    map.get(only).copied().map(Entity::from_bits)
}

/// ⭐⭐ **O `ComponentState` lido do modelo GERAL.**
///
/// ⚠️ **`is_instance` pergunta pelo ELO, e não pela raiz da cópia:** o que o painel oferece é sobre
/// a forma que está na mão. Uma peça de dentro de uma cópia é uma instância para efeito de
/// *Detach*/*Reset*/*Apply* — os três verbos gerais resolvem a raiz sozinhos.
///
/// ⛔ **`main_missing` é a mesma pergunta do motor velho, feita ao modelo novo:** o elo aponta para
/// um `StableId` que já não existe. *Uma instância cuja receita sumiu ainda desenha, e nada na forma
/// diz porquê.*
#[must_use]
pub(crate) fn state_of(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<ph2d_panel_vector::state::ComponentState> {
    let e = subject(map, selected)?;
    if sim.world().get_entity(e).is_err() {
        return None;
    }
    let link = sim.world().get::<ph2d_ecs::InstanceOf>(e).copied();
    let root = crate::instance_verbs::instance_root_of(sim, e);
    let has_overrides = root.is_some_and(|r| {
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(r)
            .is_some_and(|o| !o.overrides.is_empty())
    });
    Some(ph2d_panel_vector::state::ComponentState {
        is_main: sim.world().get::<MasterRoot>(e).is_some(),
        is_instance: link.is_some(),
        has_overrides,
        // ⚠️ O elo existe e o mestre não resolve — é a órfã do modelo geral
        // (`a_dangling_link_is_left_alone`), a mesma pergunta que o produtor vetorial responde
        // pelo lado dele.
        main_missing: link
            .is_some_and(|l| crate::instance_verbs::entity_for_stable_id(sim, l.master).is_none()),
        // ⛔ O conta-gotas ainda não está ligado ao modelo geral — ver o cabeçalho. Publicá-lo
        // armado daria um botão que arma um pick que ninguém consome.
        swap_armed: false,
    })
}

/// **Qual verbo geral este verbo do painel vetorial pede** — `None` quando ainda não há equivalente
/// publicado (e aí o painel também não o pinta; ver o cabeçalho).
#[must_use]
fn general_verb(
    verb: crate::vec_component_edit::ComponentEdit,
) -> Option<crate::instance_verbs::Verb> {
    use crate::instance_verbs::Verb;
    use crate::vec_component_edit::ComponentEdit as E;
    match verb {
        E::Create => Some(Verb::Make),
        E::Place => Some(Verb::Place),
        E::Detach => Some(Verb::Detach),
        E::UpdateMain => Some(Verb::Apply),
        // O *Reset* não é um `Verb` — ele é o *Revert to Master*, que tem porta própria porque
        // devolve a pose de maneira diferente (ver [`crate::instance_revert`]).
        E::Reset | E::Swap | E::PieceVisible(_) | E::Variant(_, _) => None,
    }
}

/// ⭐⭐⭐ **O clique da secção, no modelo GERAL.** `true` quando alguma coisa mudou.
///
/// ⚠️ **Ele passa pelo DRENO geral** (`instance_verbs::drain`), e não chama as portas uma a uma: é
/// lá que vivem a voz de cada recusa, a cascata da cópia e a lei *«a selecção segue para a cópia»*.
/// *Uma segunda montagem esquece sempre um dos três, e cada esquecimento é mudo.*
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch(
    verb: crate::vec_component_edit::ComponentEdit,
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    // ⚠️ **A ENTIDADE, e não o par (mapa, selecção)** — o mapa vive dentro do `OwnedDocs`, que o
    // dreno geral precisa **emprestado mutavelmente**. Resolver o sujeito aqui dentro pediria o
    // mapa duas vezes; quem o resolve é o chamador, **antes** de montar os documentos.
    subject: Entity,
    toasts: &mut ph2d_editor::ToastQueue,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    place_step: [f32; 2],
    select_out: &mut Option<u64>,
) -> bool {
    let e = subject;
    // O *Reset* tem porta própria — ver [`general_verb`].
    if verb == crate::vec_component_edit::ComponentEdit::Reset {
        let Some(r) = crate::instance_revert::revert_all_overrides(sim, echo, e) else {
            toasts.push(ph2d_editor::Toast::warning(
                "That is not a copy of a prefab",
            ));
            return false;
        };
        toasts.push(ph2d_editor::Toast::success(format!(
            "Reverted {} override(s) to the prefab",
            r.count
        )));
        return r.count > 0;
    }
    // ⛔⛔ **O conta-gotas do *Swap* FALA em vez de morrer.** O painel pinta aquele botão para toda
    // instância, e no modo novo ele não tem consumidor — *um controlo que come o clique em silêncio
    // é pior que um ausente*, e é o defeito que esta linha caçou três vezes. ⚠️ A saída que a voz
    // nomeia **existe e é melhor**: o *Replace selection with this* da biblioteca (F5.8) escolhe o
    // mestre por nome e com os **três modos** de emparelhamento, contra o palpite de um clique.
    if verb == crate::vec_component_edit::ComponentEdit::Swap {
        toasts.push(ph2d_editor::Toast::info(
            "Swapping by eyedropper is not in the new component mode \u{2014} use \u{201c}Replace \
             selection with this\u{201d} in the library",
        ));
        return false;
    }
    let Some(v) = general_verb(verb) else {
        return false;
    };
    crate::instance_verbs::drain(
        v,
        sim,
        registry,
        echo,
        e.to_bits(),
        toasts,
        docs,
        place_step,
        select_out,
    )
}

#[cfg(test)]
#[path = "vec_component_general_tests.rs"]
mod tests;
