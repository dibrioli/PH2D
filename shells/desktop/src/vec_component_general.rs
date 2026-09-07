//! ⭐⭐⭐ **A SECÇÃO *Prefab* DO PAINEL VETORIAL, LIGADA AO MECANISMO GERAL** (F4.6c, waves 1 e 2).
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
//! **três** sítios que decidem — o que o painel MOSTRA, o que o clique do botão FAZ, e o **segundo
//! clique do conta-gotas** (que vive no `input_dispatch`; ver [`swap_by_pick`]).
//!
//! ⚠️ **É o precedente do `PH2D_RETOPO_EXTRACT`**: o motor novo shipa desligado, com a tabela da
//! comparação ao lado, e o caminho de omissão fica **byte-idêntico**.
//!
//! # ⭐⭐⭐ Os TRÊS controlos que o motor velho tinha a mais, e onde cada um foi parar
//!
//! A F4.6c media *«um porte MENOS três features»*. Medidas uma a uma em 2026-09-06, **duas já
//! existiam noutra superfície do modelo geral** e a terceira foi construída:
//!
//! | controlo do painel vetorial | no modelo geral |
//! |---|---|
//! | **conta-gotas do *Swap*** | ⭐ **construído aqui** ([`swap_by_pick`]) — o mesmo gesto de duas mãos, com o alvo resolvido pelo `master_subject` e re-key determinístico, que o *Swap* do vetor nunca teve |
//! | **lista de peças** (o olho · a cor por peça) | ⛔ **não se porta: as peças são ENTIDADES REAIS.** O olho é o da Hierarquia e a cor é qualquer ferramenta sobre a peça seleccionada — as duas portas estão medidas em [`crate::instance_piece_override_tests`]. *A lista existia porque naquele modelo a peça não tinha endereço; aqui tem.* |
//! | **fileira de variants** | ⛔ **não se porta: ela já é pintada**, no cartão *Properties* do Inspector (`variant_axes::axes_for`, alimentado pelo `family_members`), derivada pelo MESMO mapa que a troca usa. Uma segunda fileira noutro painel seria a mesma pergunta com duas respostas. |
//!
//! ⚠️ **E é por isso que a shell publica as duas listas VAZIAS** neste modo — não por elas ficarem
//! vazias sozinhas (o `VecInstance` viaja na cópia profunda), mas por DECLARAÇÃO. *Um controlo que
//! aparece e não faz nada é o defeito que este repo caça.*

use ph2d_ecs::{Entity, MasterRoot, SimWorld};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// **O modo geral está armado?** Porta única — ver o cabeçalho.
///
/// ⚠️ **Lida em TRÊS sítios** (o que o painel mostra · o que o clique do botão faz · o segundo
/// clique do conta-gotas) e por isso é uma função, não um `if` copiado: dois deles a discordarem dá
/// uma secção que oferece um verbo que o dreno recusa, ou um gesto que arma num motor e resolve no
/// outro.
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
    // ⭐ **O conta-gotas está armado?** — o mesmo dado que o produtor vetorial recebe. Ele vive no
    // `App::vec_path_pick`, que é da shell: publicá-lo é o que faz o botão trocar de rótulo para
    // *Click a copy of the prefab* enquanto o gesto está aberto.
    pick_armed: bool,
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
    // ⭐⭐⭐ **`is_main` pergunta *«há receita para instanciar?»*, e não *«isto É a receita?»***
    // (report do Enio, 2026-09-06: *«não existe mais a opção instanciate»*).
    //
    // # ⛔⛔ Duas decisões certas deste ficheiro desfaziam-se uma à outra
    //
    // O *Make Prefab* **move a selecção para a cópia**, de propósito ([`crate::instance_verbs`],
    // o doc do `select_out`: é o que o Figma e o Unity fazem, e é a forma que o artista continua
    // a editar). Só que a condição que PINTA o *Instantiate* foi herdada do motor vetorial, onde
    // criar **nunca** tirava a selecção do mestre — logo, no modo geral, o gesto seguinte da fila
    // ficava **inalcançável**: a seleção era a cópia, `is_main` era `false`, e o botão nunca mais
    // era pintado. ⚠️ **E a receita fica invisível no canvas** (`is_unedited_recipe`), então nem
    // clicando nela o artista a reavia — só pela linha da Hierarquia.
    //
    // ⚠️ **A cerca não caiu, ela alinha-se com o CONSUMIDOR:** o `Verb::Place` já resolve a
    // receita a partir de uma cópia desde 2026-08-31 (`instance_verbs_walk::master_subject`,
    // curado no fluxo da Hierarquia pelo report *«me mostre o fluxo inteiro de criar variações»*).
    // ⇒ *a lente do PAINEL era mais estreita que a do consumidor*, que é o inverso do knob morto:
    // um verbo vivo sem controlo pintado. Uma forma que não é nem receita nem cópia continua a
    // não oferecer nada, porque aí o `master_subject` devolve a própria entidade.
    //
    // ⛔ E a órfã continua de fora: com o elo pendurado o `master_subject` não resolve, logo
    // `is_main` é `false` — *instanciar a partir de uma cópia cuja receita sumiu não tem sujeito*.
    let has_recipe = {
        let subject = crate::instance_verbs_walk::master_subject(sim, e);
        sim.world().get::<MasterRoot>(subject).is_some()
    };
    Some(ph2d_panel_vector::state::ComponentState {
        is_main: has_recipe,
        is_instance: link.is_some(),
        has_overrides,
        // ⚠️ O elo existe e o mestre não resolve — é a órfã do modelo geral
        // (`a_dangling_link_is_left_alone`), a mesma pergunta que o produtor vetorial responde
        // pelo lado dele.
        main_missing: link
            .is_some_and(|l| crate::instance_verbs::entity_for_stable_id(sim, l.master).is_none()),
        // ⭐⭐⭐ **Uma cópia pode virar uma VERSÃO NOVA do prefab** (report do Enio, 2026-09-06:
        // *«Make Prefab só aparece no menu da hierarchy e não no painel vector»*). É o `Verb::Make`
        // sobre uma cópia — a lei da F5 critério 2 —, e o menu da Hierarquia já o oferecia porque a
        // tabela dele é PLANA. *A terceira vez, nesta secção, em que a lente do painel era mais
        // estreita que a do verbo.*
        can_make_variant: link.is_some(),
        swap_armed: pick_armed,
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
    // ⭐ Sai `true` quando o verbo abre o gesto de duas mãos do conta-gotas — ver o braço do `Swap`.
    arm_pick: &mut bool,
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
    // ⭐⭐⭐ **O conta-gotas ARMA e sai** — o clique seguinte no canvas é dele
    // ([`swap_by_pick`]). ⚠️ **Quem escreve o `PathPick` é a shell**, porque ele vive no `App`; o
    // que este módulo devolve é a DECISÃO, para a lei do modo ficar toda aqui. *Um `if verb ==
    // Swap` escrito lá em cima seria a terceira leitura do interruptor a decidir sozinha.*
    if verb == crate::vec_component_edit::ComponentEdit::Swap {
        *arm_pick = true;
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

/// ⭐⭐⭐ **O SEGUNDO clique do conta-gotas, no modelo geral** — a cópia `source` passa a ser uma
/// cópia do prefab de `clicked`.
///
/// # ⚠️ O alvo é «uma CÓPIA do prefab que eu quero», e não o prefab
///
/// No motor vetorial o mestre era uma forma **visível** no canvas, e o conta-gotas clicava-a. No
/// modelo geral a receita está **escondida** (`is_unedited_recipe`) — clicar nela é impossível.
/// ⇒ o alvo resolve-se pela porta que os outros verbos já usam
/// ([`crate::instance_verbs_walk::master_subject`]): clicar uma cópia do prefab B quer dizer
/// *«esta também passa a ser um B»*, e clicar a receita, quando ela está aberta, também serve.
/// *É a mesma lei que devolveu o `Instantiate` ao painel — a lente do gesto acompanha a do verbo.*
///
/// ⛔ **`WhenUnrelated::Refuse`**: sem antepassado comum não há mapa determinístico, e um clique não
/// é um sítio para escolher modo de emparelhamento — os três modos vivem no item de menu que os
/// nomeia (F5.8). *Um palpite silencioso num gesto de duas mãos é a pior das duas coisas.*
///
/// Devolve `true` quando o mundo mudou (e aí o pick desarma-se).
pub(crate) fn swap_by_pick(
    sim: &mut SimWorld,
    echo: &mut crate::instance_sync::MasterEcho,
    toasts: &mut ph2d_editor::ToastQueue,
    source: Entity,
    clicked: Entity,
) -> bool {
    let Some(root) = crate::instance_verbs::instance_root_of(sim, source) else {
        toasts.push(ph2d_editor::Toast::warning(
            "That is not a copy of a prefab",
        ));
        return false;
    };
    let target = crate::instance_verbs_walk::master_subject(sim, clicked);
    let Some(id) = sim
        .world()
        .get::<ph2d_ecs::StableId>(target)
        .map(|s| s.0)
        .filter(|_| sim.world().get::<MasterRoot>(target).is_some())
    else {
        // ⚠️ **A recusa NOMEIA o que fazer** — o artista clicou numa forma comum, e o gesto fica
        // armado de propósito (desarmar aqui faria um clique fora do alvo parecer uma troca).
        toasts.push(ph2d_editor::Toast::warning(
            "That shape is not a copy of a prefab \u{2014} click one, or the open prefab",
        ));
        return false;
    };
    match crate::instance_variant::swap(
        sim,
        echo,
        root,
        id,
        crate::instance_swap_match::WhenUnrelated::Refuse,
    ) {
        Ok(r) => {
            let name = crate::instance_verbs::master_named(sim, id)
                .unwrap_or_else(|| "prefab".to_string());
            let mut say = format!(
                "Now a copy of \u{201c}{name}\u{201d} \u{2014} {} override(s) kept",
                r.overrides_kept
            );
            // ⚠️ **O que se PERDEU é dito no mesmo fôlego.** O motor velho escrevia isto num
            // `eprintln!`, que o artista não vê — e o que ele vê é uma peça a desaparecer.
            if r.dropped > 0 {
                say.push_str(&format!(
                    " \u{b7} {} piece(s) the new prefab does not have were removed",
                    r.dropped
                ));
            }
            toasts.push(ph2d_editor::Toast::success(say));
            true
        }
        Err(crate::instance_variant::SwapRefusal::Already) => {
            toasts.push(ph2d_editor::Toast::info(
                "It is already a copy of that prefab",
            ));
            false
        }
        Err(crate::instance_variant::SwapRefusal::Unrelated) => {
            toasts.push(ph2d_editor::Toast::warning(
                "Those two prefabs are unrelated \u{2014} use \u{201c}Replace selection with \
                 this\u{201d} in the library to choose how to match the pieces",
            ));
            false
        }
        Err(_) => {
            toasts.push(ph2d_editor::Toast::warning(
                "That copy cannot become this prefab",
            ));
            false
        }
    }
}

#[cfg(test)]
#[path = "vec_component_general_tests.rs"]
mod tests;
