//! ⭐⭐⭐ **EDITAR A RECEITA é um MODO, e o gesto que entra nele é selecioná-la** (F4.6, o §14).
//!
//! # As duas verdades que se contradiziam
//!
//! 1. *«Uma receita não está na cena»* — o gesto *Criar componente* esconde-a, senão o artista vê
//!    **dois objetos empilhados**, um que cai e outro que não (F4.5, e o smoke do Enio confirmou).
//! 2. *«O artista tem de conseguir mudar a forma da receita»* — foi assim que a cena 2 do smoke
//!    passou: com a receita **desenhada**, longe das cópias, mover um nó dela chega às três.
//!
//! Esconder sempre mata a (2); desenhar sempre mata a (1). ⇒ **a receita sai da cena, e volta
//! enquanto está selecionada.** É o *Prefab Mode* do Unity reduzido ao que esta casa já tem: a
//! Hierarquia é onde a biblioteca vive, e escolher uma linha lá é dizer *«é nisto que estou a
//! mexer»*.
//!
//! # ⚠️ Porque é uma MARCA DERIVADA, e não um argumento
//!
//! A pergunta *«esta entidade está na cena?»* é feita em dois sítios muito distantes — o extract
//! de sprites ([`super::off_canvas`]) e a cadeia de visibilidade do vetor
//! ([`ph2d_vec_entities::entities`]) — e nenhum dos dois tem a selecção à mão. Enfiar um `Option<Entity>`
//! nos dois caminhos seria a mesma resposta a viajar por duas estradas, e elas divergiriam.
//!
//! ⇒ um passe carimba [`ph2d_ecs::MasterEditing`] na sub-árvore da receita seleccionada e
//! **desmarca todo o resto**; as duas perguntas leem o mundo. ⚠️ **As duas metades são
//! obrigatórias** — marcar sem desmarcar deixa uma receita visível para sempre depois de o artista
//! mudar de selecção, que é o defeito (1) de volta pela porta do lado.
//!
//! ⚠️ **Não é registada**, como o `MasterPiece` e pela mesma razão: é derivada da selecção, que é
//! vista e não documento. Um valor derivado no arquivo envenena o undo.

use ph2d_ecs::{Entity, MasterEditing, SimWorld};

/// ⭐ **Marca as receitas que estão a ser editadas.** Devolve `true` quando mexeu em alguma coisa.
///
/// `selection` são os bits de **toda** a selecção — a primária e os extras. Uma selecção que não
/// toque receita nenhuma desmarca tudo, que é o caso comum e o mais barato (e não aloca: um
/// `collect` de um iterador vazio não pede memória).
///
/// ⚠️⚠️ **O parâmetro era um `Option<u64>` — só o primário — e isso era o defeito §1.6 da
/// auditoria de 2026-08-27.** Duas rotas correntes deixam uma receita seleccionada sem ser
/// primária: o Shift/Ctrl-clique (`add_to_selection`/`toggle_in_selection`) e o atalho
/// `preserves_multi` do ramo `Replace`. ⇒ a linha ficava realçada na Hierarquia e o canvas
/// continuava vazio, que é o report *«cliquei nela e não aconteceu nada»*. ⛔ **E o gate não podia
/// vê-lo**: um `Option<u64>` torna o defeito inexprimível na assinatura, então
/// `the_recipe_comes_back_while_it_is_being_edited` ficava verde por construção. *Uma assinatura
/// que não consegue exprimir o caso é um gate que nunca o mede.*
///
/// ⚠️ **N receitas ao mesmo tempo é o comportamento certo, não uma tolerância:** seleccionar duas
/// linhas de biblioteca e ver as duas é o que a multi-selecção promete em todo o resto do app.
///
/// ⭐⭐⭐ **A TRAVA** (Enio, 2026-09-07: *«só permita sair da edição apertando Done ou a tecla
/// Enter»*): enquanto `latch` aponta uma receita, ela fica aberta **independentemente da
/// selecção**. Sem ela, clicar no vazio fechava a sessão — a saída era um acidente, e o artista não
/// tinha como saber que o clique lá custava o modo.
///
/// ⚠️ **Ela é a UNIÃO com o derivado da selecção, e não uma substituição:** é o derivado que ABRE
/// (as quatro superfícies do verbo seleccionam a raiz), e a trava que SEGURA. Trocar a união por
/// *«só a trava»* tornaria impossível abrir uma segunda receita sem sair da primeira.
///
/// ⚠️ **E ela solta-se sozinha quando a receita MORRE** — apagá-la a meio da sessão deixaria o
/// artista num modo sem barra (a barra é derivada do mundo) e portanto sem saída.
///
/// ⛔⛔⛔ **A trava é um `StableId`, e não os bits da entidade** — a 1.ª versão eram os bits, e por
/// isso um `Ctrl+Z` dentro da sessão **expulsava o artista dela**: o undo respawna tudo com bits
/// novos, e a trava passava a apontar para uma entidade morta. É a lei escrita do módulo do editor
/// (*«referência durável entre objectos é o `StableId`, nunca os bits — o undo respawna tudo»*), e
/// eu escrevi-a ao contrário; o gate `the_session_survives_an_undo_step` é a reprodução.
pub fn mark(
    sim: &mut SimWorld,
    selection: impl IntoIterator<Item = u64>,
    latch: &mut Option<u64>,
) -> Marked {
    // A trava resolve-se pela identidade DURÁVEL a cada quadro; ela larga quando a receita deixa de
    // existir (ou deixa de ser uma receita).
    let latched: Option<u64> = latch.and_then(|id| {
        crate::instance_verbs_walk::entity_for_stable_id(sim, id).filter(|&bits| {
            sim.world()
                .get::<ph2d_ecs::MasterRoot>(Entity::from_bits(bits))
                .is_some()
        })
    });
    if latched.is_none() {
        *latch = None;
    }
    let editing: Vec<Entity> = selection
        .into_iter()
        .chain(latched)
        .map(Entity::from_bits)
        .filter(|&e| sim.world().get_entity(e).is_ok())
        .filter_map(|e| ph2d_ecs::master_root_of(sim.world(), e))
        .collect();
    let mut want: std::collections::BTreeSet<Entity> = std::collections::BTreeSet::new();
    for &root in &editing {
        want.extend(subtree(sim, root));
    }
    let have: std::collections::BTreeSet<Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<Entity, bevy_ecs::query::With<MasterEditing>>();
        q.iter(sim.world()).collect()
    };
    // ⭐⭐⭐ **QUEM ACABOU DE ABRIR** — a raiz que ainda não estava marcada. É esta a transição que
    // o enquadramento da câmera espera ([`crate::prefab_framing`]), e ela mora AQUI de propósito:
    // as quatro superfícies que abrem uma receita (o painel, a Hierarquia, o cartão da biblioteca,
    // o cartão do Inspector) passam todas por esta marca, e uma delas que não passasse já não
    // acenderia a receita. *Um evento derivado da mesma lei não pode ficar por fora de uma porta.*
    let opened: Vec<Entity> = editing
        .iter()
        .copied()
        .filter(|r| !have.contains(r))
        .collect();
    let mut touched = false;
    for &e in want.difference(&have) {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(MasterEditing);
            touched = true;
        }
    }
    // ⚠️ A metade que se esquece: sem ela a receita fica visível para sempre.
    for &e in have.difference(&want) {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.remove::<MasterEditing>();
            touched = true;
        }
    }
    Marked { touched, opened }
}

/// O que o passe fez neste quadro.
///
/// ⚠️ **`opened` não é «o que está aberto», é «o que ABRIU AGORA»** — as duas leem-se igual numa
/// chamada e são coisas diferentes: a primeira é verdade em todo quadro enquanto a receita estiver
/// seleccionada, e enquadrar a câmera com ela prenderia a vista à receita para sempre.
pub struct Marked {
    /// Alguma marca foi posta ou tirada — o mundo mudou.
    ///
    /// ⚠️ **Só os gates o leem**, e é deliberado: o quadro carimba e segue, porque quem consome a
    /// marca (o extract e a cadeia de visibilidade do vetor) lê o MUNDO, não este retorno. *Um
    /// campo que o produto não lê e o gate lê é a assinatura de uma pergunta que só o teste faz —
    /// e apagá-lo tiraria aos gates a única forma de perguntar «o passe fez alguma coisa?».*
    #[cfg_attr(not(test), allow(dead_code))]
    pub touched: bool,
    /// As raízes que passaram de fechadas a abertas NESTE quadro.
    pub opened: Vec<Entity>,
}

/// ⭐⭐⭐ **Há ALGUMA receita aberta neste quadro?** — a pergunta que liga o vidro jateado.
///
/// ⚠️⚠️ **Ela pergunta ao MUNDO, e não à vista do vetor, e a diferença é uma família inteira de
/// prefabs:** o `VecViewState::isolated` é enchido a partir das FORMAS vectoriais marcadas, então
/// uma receita feita só de imagens (um ragdoll, um cartão de sprite) deixava-o vazio — e o vidro
/// nunca subia para o único caso em que só as peças raster precisavam de ser levantadas. *Uma
/// pergunta respondida pelo lado que tem a resposta mais ESTREITA é um modo que não liga para
/// metade dos sujeitos dele.*
pub fn any_open(sim: &mut SimWorld) -> bool {
    let mut q = sim
        .world_mut()
        .query_filtered::<Entity, bevy_ecs::query::With<MasterEditing>>();
    q.iter(sim.world()).next().is_some()
}

/// ⭐⭐⭐ **O que a BARRA DO MODO mostra** — qual receita está aberta, e quantas cópias a seguem.
///
/// ⚠️ **Irmã do [`any_open`], uma pergunta mais fina, e há gate a prendê-las:** a barra tem de
/// aparecer **exactamente** quando o vidro sobe. Duas respostas diferentes a *«há receita aberta?»*
/// dariam um canvas borrado sem barra (ou uma barra sobre um canvas nítido), e nenhum dos dois é
/// diagnosticável a olho.
///
/// ⚠️ **A primeira, e não todas:** a barra tem um sítio só. Com duas receitas abertas ela nomeia a
/// de ordem mais baixa — a mesma que o palco levanta.
///
/// ⛔ `None` sem receita aberta, e aí o quadro não paga a varredura das cópias.
pub fn open_view(
    sim: &mut SimWorld,
) -> Option<ph2d_editor_core::screens::hero::prefab_bar::PrefabEditView> {
    let root = {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &ph2d_ecs::StableId), (
                bevy_ecs::query::With<MasterEditing>,
                bevy_ecs::query::With<ph2d_ecs::MasterRoot>,
            )>();
        q.iter(sim.world()).map(|(e, s)| (e, s.0)).min()?
    };
    let (entity, id) = root;
    let _ = entity;
    let copies = {
        let mut q = sim.world_mut().query::<&ph2d_ecs::InstanceOf>();
        q.iter(sim.world()).filter(|i| i.master == id).count()
    };
    let name = crate::instance_verbs::master_named(sim, id)?;
    Some(ph2d_editor_core::screens::hero::prefab_bar::PrefabEditView { name, copies })
}

/// A sub-árvore de `root`, ela incluída.
fn subtree(sim: &SimWorld, root: Entity) -> std::collections::BTreeSet<Entity> {
    let mut out = std::collections::BTreeSet::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if !out.insert(e) {
            continue;
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out
}

#[cfg(test)]
#[path = "master_editing_tests.rs"]
mod tests;
