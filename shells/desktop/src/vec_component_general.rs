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
//! # ⭐⭐⭐ Ele é o caminho de OMISSÃO desde 2026-09-06, e a data importa
//!
//! Ele **nasceu desligado**, e isso não era timidez: esta secção é um fluxo que **o dono usa** e
//! que **eu não posso smokar** — ela vive no editor vetorial, atrás de uma janela. Trocar o motor
//! por baixo dela e entregar sem rede seria pedir-lhe que descobrisse a regressão por mim.
//!
//! ⇒ o precedente é **o do `PH2D_RETOPO_EXTRACT`, inteiro**: shipa desligado, o dono smoka, e
//! **quando ele aprova, o motor novo passa a ser o caminho de omissão**. Aqui foram **três** smokes
//! aprovados em sequência (o modo · o conta-gotas do *Swap* · a cascata + o *Make Variant*), e a
//! última coisa que faltava — os três controlos que o motor velho tinha a mais — está medida na
//! tabela abaixo. *A lei «tudo o que é novo shipa desligado» vale enquanto houver algo por fechar;
//! ela não é um estado permanente.*
//!
//! ⛔⛔ **E O INTERRUPTOR MORREU COM O OUTRO MOTOR** (F4.6c wave 3, 2026-09-07). Aqui viveu um
//! `armed()` — `PH2D_VEC_COMPONENT_GENERAL=0` voltava ao motor `VecInstance` — lido nos **três**
//! sítios que decidiam. O bloqueio daquela wave era **prático e não técnico** (apagar ~2 961 LOC
//! em 27 ficheiros com a `line/Vector` viva era catástrofe de merge), e dissolveu-se no dia em
//! que aquela linha integrou. *Não há segundo motor, logo não há o que escolher.*
//!
//! ⚠️ **Uma instância vetorial GRAVADA antes de 2026-09-06 lê-se como forma comum** (o painel
//! oferece-lhe *Make Prefab*), e a partir de então não havia como criar uma. Não há projectos
//! gravados — decisão do Enio, registada no §5 —, e o `PROJECT_SCHEMA` sobe com este corte para
//! que um ficheiro que carregasse aquele componente **recuse em voz alta** em vez de ser lido
//! errado em silêncio.
//!
//! # ⭐⭐⭐ Os TRÊS controlos que o motor velho tinha a mais, e onde cada um foi parar
//!
//! A F4.6c media *«um porte MENOS três features»*. Medidas uma a uma em 2026-09-06, **duas já
//! existiam noutra superfície do modelo geral** e a terceira foi construída:
//!
//! | controlo do painel vetorial | no modelo geral |
//! |---|---|
//! | **conta-gotas do *Swap*** | ⭐ **construído aqui** ([`swap_by_pick`]) — o mesmo gesto de duas mãos, com o alvo resolvido pelo `master_subject` e re-key determinístico, que o *Swap* do vetor nunca teve |
//! | **lista de peças** (o olho · a cor por peça) | ⛔ **não se porta: as peças são ENTIDADES REAIS.** O olho é o da Hierarquia e a cor é qualquer ferramenta sobre a peça seleccionada — as duas portas estão medidas em `ph2d_app_components::instance_structure::instance_piece_override_tests`. *A lista existia porque naquele modelo a peça não tinha endereço; aqui tem.* |
//! | **fileira de variants** | ⛔ **não se porta: ela já é pintada**, no cartão *Properties* do Inspector (`variant_axes::axes_for`, alimentado pelo `family_members`), derivada pelo MESMO mapa que a troca usa. Uma segunda fileira noutro painel seria a mesma pergunta com duas respostas. |
//!
//! ⚠️ **E é por isso que a shell publica as duas listas VAZIAS** neste modo — não por elas ficarem
//! vazias sozinhas (o `VecInstance` viaja na cópia profunda), mas por DECLARAÇÃO. *Um controlo que
//! aparece e não faz nada é o defeito que este repo caça.*

use ph2d_ecs::{Entity, MasterRoot, SimWorld};
use ph2d_i18n::{tr, tr_with};
use ph2d_vec_scene::VecPathId;

use ph2d_vec_entities::entities::VecEntityMap;

/// ⭐⭐⭐ **O OBJECTO sobre o qual a secção fala** — e ele não é forçosamente um traço.
///
/// ⚠️ **Uma coisa só, de propósito:** um prefab é sobre UM objecto, e *«fazer prefab»* a partir de
/// duas selecções é outra operação (agrupar primeiro).
///
/// # ⛔⛔ O GRUPO não é um path (report do Enio, 2026-09-07)
///
/// *«Com a pasta do grupo seleccionada não aparecem as opções de prefab; aparecem ao clicar nos
/// filhos.»* A secção perguntava **só à caneta** (`vec_pen.selected_paths()`), e um grupo é uma
/// entidade **comum com filhos** — ele não tem `VecPathRef`, logo não é path nenhum e a lista vinha
/// vazia (ou com os dois filhos). ⇒ *a lente do painel era mais estreita que a do verbo*, pela
/// **quarta** vez nesta secção: os verbos gerais trabalham sobre ENTIDADES, e o menu da Hierarquia
/// já aceitava o grupo — que é, aliás, o único sítio por onde se faz um prefab de várias formas.
///
/// ⇒ duas fontes, nesta ordem: o path seleccionado (o caminho exacto de sempre) e, quando ele não
/// responde, o **objecto único** que o gizmo tem na mão. ⚠️ **A ordem importa:** com um path
/// seleccionado as duas concordam, e manter o path à frente deixa o caminho de omissão intocado.
///
/// ⚠️ **A MESMA função serve o que a secção MOSTRA e o que o clique FAZ** — duas resoluções dariam
/// uma secção a oferecer um verbo sobre o grupo e um clique a agir sobre um filho, que é a espécie
/// de defeito mais cara deste ficheiro.
pub(crate) fn subject_of(
    map: &VecEntityMap,
    selected: &[VecPathId],
    single_selected: Option<u64>,
) -> Option<Entity> {
    if let [only] = selected
        && let Some(&bits) = map.get(only)
    {
        return Some(Entity::from_bits(bits));
    }
    single_selected.map(Entity::from_bits)
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
    // ⭐ **O objecto único na mão do gizmo** — a segunda fonte do sujeito; ver [`subject_of`].
    single_selected: Option<u64>,
    // ⭐ **O conta-gotas está armado?** — o mesmo dado que o produtor vetorial recebe. Ele vive no
    // `app.vec.path_pick`, que a shell guarda: publicá-lo é o que faz o botão trocar de rótulo para
    // *Click a copy of the prefab* enquanto o gesto está aberto.
    pick_armed: bool,
) -> Option<ph2d_panel_vector::state::ComponentState> {
    let e = subject_of(map, selected, single_selected)?;
    if sim.world().get_entity(e).is_err() {
        return None;
    }
    let link = sim.world().get::<ph2d_ecs::InstanceOf>(e).copied();
    let root = ph2d_app_components::instance_verbs::instance_root_of(sim, e);
    let has_overrides = root.is_some_and(|r| {
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(r)
            .is_some_and(|o| !o.overrides.is_empty())
    });
    // ⭐⭐⭐ **`is_main` pergunta *«há receita para instanciar?»*, e não *«isto É a receita?»***
    // (report do Enio, 2026-09-06: *«não existe mais a opção de instanciar»*).
    //
    // # ⛔⛔ Duas decisões certas deste ficheiro desfaziam-se uma à outra
    //
    // O *Make Prefab* **move a selecção para a cópia**, de propósito ([`ph2d_app_components::instance_verbs`],
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
        let subject = ph2d_app_components::instance_verbs_walk::master_subject(sim, e);
        sim.world().get::<MasterRoot>(subject).is_some()
    };
    Some(ph2d_panel_vector::state::ComponentState {
        is_main: has_recipe,
        is_instance: link.is_some(),
        has_overrides,
        // ⚠️ O elo existe e o mestre não resolve — é a órfã do modelo geral
        // (`a_dangling_link_is_left_alone`), a mesma pergunta que o produtor vetorial responde
        // pelo lado dele.
        main_missing: link.is_some_and(|l| {
            ph2d_app_components::instance_verbs::entity_for_stable_id(sim, l.master).is_none()
        }),
        // ⭐⭐⭐ **Uma cópia pode virar uma VERSÃO NOVA do prefab** (report do Enio, 2026-09-06:
        // *«Make Prefab só aparece no menu da hierarchy e não no painel vector»*). É o `Verb::Make`
        // sobre uma cópia — a lei da F5 critério 2 —, e o menu da Hierarquia já o oferecia porque a
        // tabela dele é PLANA. *A terceira vez, nesta secção, em que a lente do painel era mais
        // estreita que a do verbo.*
        can_make_variant: link.is_some(),
        // ⭐⭐⭐ **A receita está ESCONDIDA no canvas** (`is_unedited_recipe`), então chegar a ela é
        // um gesto — e até 2026-09-07 o único era o cartão da biblioteca. Ver [`Verb::Edit`].
        can_edit_prefab: link.is_some(),
        swap_armed: pick_armed,
    })
}

/// **Qual verbo geral este verbo do painel vetorial pede** — `None` quando ainda não há equivalente
/// publicado (e aí o painel também não o pinta; ver o cabeçalho).
#[must_use]
fn general_verb(
    verb: crate::vec_component_edit::ComponentEdit,
) -> Option<ph2d_app_components::instance_verbs::Verb> {
    use crate::vec_component_edit::ComponentEdit as E;
    use ph2d_app_components::instance_verbs::Verb;
    match verb {
        E::Create => Some(Verb::Make),
        E::Edit => Some(Verb::Edit),
        E::Place => Some(Verb::Place),
        E::PlaceLinked => Some(Verb::PlaceLinked),
        E::Detach => Some(Verb::Detach),
        E::UpdateMain => Some(Verb::Apply),
        // O *Reset* não é um `Verb` — ele é o *Revert to Master*, que tem porta própria porque
        // devolve a pose de maneira diferente (ver [`ph2d_app_components::instance_revert`]).
        E::Reset | E::Swap => None,
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
    echo: &mut ph2d_app_components::instance_sync::MasterEcho,
    // ⚠️ **A ENTIDADE, e não o par (mapa, selecção)** — o mapa vive dentro do `OwnedDocs`, que o
    // dreno geral precisa **emprestado mutavelmente**. Resolver o sujeito aqui dentro pediria o
    // mapa duas vezes; quem o resolve é o chamador, **antes** de montar os documentos.
    subject: Entity,
    toasts: &mut ph2d_editor_core::ToastQueue,
    docs: &mut ph2d_app_components::instance_docs::OwnedDocs<'_>,
    place_step: [f32; 2],
    select_out: &mut Option<u64>,
    // ⭐ Sai `true` quando o verbo abre o gesto de duas mãos do conta-gotas — ver o braço do `Swap`.
    arm_pick: &mut bool,
) -> bool {
    let e = subject;
    // O *Reset* tem porta própria — ver [`general_verb`].
    if verb == crate::vec_component_edit::ComponentEdit::Reset {
        let Some(r) = ph2d_app_components::instance_revert::revert_all_overrides(sim, echo, e)
        else {
            toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.vec_component_general.that_is_not_a_copy_of",
            )));
            return false;
        };
        toasts.push(ph2d_editor_core::Toast::success(tr_with(
            "shell.vec_component_general.reverted_override_s_to",
            &[("r", &(r.count))],
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
    ph2d_app_components::instance_verbs::drain(
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
/// ([`ph2d_app_components::instance_verbs_walk::master_subject`]): clicar uma cópia do prefab B quer dizer
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
    echo: &mut ph2d_app_components::instance_sync::MasterEcho,
    toasts: &mut ph2d_editor_core::ToastQueue,
    source: Entity,
    clicked: Entity,
) -> bool {
    let Some(root) = ph2d_app_components::instance_verbs::instance_root_of(sim, source) else {
        toasts.push(ph2d_editor_core::Toast::warning(tr(
            "shell.vec_component_general.that_is_not_a_copy_of",
        )));
        return false;
    };
    let target = ph2d_app_components::instance_verbs_walk::master_subject(sim, clicked);
    let Some(id) = sim
        .world()
        .get::<ph2d_ecs::StableId>(target)
        .map(|s| s.0)
        .filter(|_| sim.world().get::<MasterRoot>(target).is_some())
    else {
        // ⚠️ **A recusa NOMEIA o que fazer** — o artista clicou numa forma comum, e o gesto fica
        // armado de propósito (desarmar aqui faria um clique fora do alvo parecer uma troca).
        toasts.push(ph2d_editor_core::Toast::warning(tr(
            "shell.vec_component_general.that_shape_is_not_a",
        )));
        return false;
    };
    match ph2d_app_components::instance_variant::swap(
        sim,
        echo,
        root,
        id,
        ph2d_app_components::instance_swap_match::WhenUnrelated::Refuse,
    ) {
        Ok(r) => {
            let name = ph2d_app_components::instance_verbs::master_named(sim, id)
                .unwrap_or_else(|| "prefab".to_string());
            let mut say = tr_with(
                "shell.vec_component_general.now_a_copy_of_override",
                &[("name", &name), ("overrides_kept", &(r.overrides_kept))],
            );
            // ⚠️ **O que se PERDEU é dito no mesmo fôlego.** O motor velho escrevia isto num
            // `eprintln!`, que o artista não vê — e o que ele vê é uma peça a desaparecer.
            if r.dropped > 0 {
                say.push_str(&tr_with(
                    "shell.vec_component_general.piece_s_the_new_prefab",
                    &[("dropped", &(r.dropped))],
                ));
            }
            toasts.push(ph2d_editor_core::Toast::success(say));
            true
        }
        Err(ph2d_app_components::instance_variant::SwapRefusal::Already) => {
            toasts.push(ph2d_editor_core::Toast::info(tr(
                "shell.vec_component_general.it_is_already_a_copy",
            )));
            false
        }
        Err(ph2d_app_components::instance_variant::SwapRefusal::Unrelated) => {
            toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.vec_component_general.those_two_prefabs_are",
            )));
            false
        }
        Err(_) => {
            toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.vec_component_general.that_copy_cannot",
            )));
            false
        }
    }
}

#[cfg(test)]
#[path = "vec_component_general_tests.rs"]
mod tests;

/// ⭐ **Os gestos de DUAS MÃOS** — irmão por responsabilidade, e o corte que o tecto de LOC impôs.
#[cfg(test)]
#[path = "vec_component_general_pick_tests.rs"]
mod pick_tests;

/// ⭐ **O CENSO do motor único** — irmão por responsabilidade: ele mede o que a ÁRVORE contém, e
/// não o que a secção faz.
#[cfg(test)]
#[path = "vec_component_general_census_tests.rs"]
mod census_tests;
