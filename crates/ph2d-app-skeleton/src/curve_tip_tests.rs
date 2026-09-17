//! Os gates da porta que lista **quem pode mandar na ponta**.

use super::{MAX_TIP_CHILDREN, choice_at, options};
use ph2d_ecs::SimWorld;
use ph2d_skeleton_ecs::CurveTip;

/// A bifurcação com os ids duráveis atribuídos, e o osso ramificado em `From Chain`.
fn cena() -> (SimWorld, ph2d_ecs::Entity, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let b = ph2d_skeleton_demo::bifurcacao(&mut sim).expect("a cena monta");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, b.com_dois_filhos, b.com_um_filho)
}

/// ⭐⭐⭐ **A LISTA MOSTRA OS FILHOS, e a escolha de cada linha é a que a lista promete** — as duas
/// metades saem da MESMA porta, que é o ponto dela existir.
#[test]
fn the_list_offers_the_two_fixed_choices_and_one_line_per_bone_child() {
    let (sim, ramificado, um) = cena();
    let (o, escolhas) = options(&sim, ramificado).expect("o osso esta' em From Chain");
    assert_eq!(
        o.rotulos.len(),
        escolhas.len(),
        "rotulos e escolhas alinhados"
    );
    assert_eq!(
        o.rotulos.len(),
        4,
        "duas fixas + dois filhos: {:?}",
        o.rotulos
    );
    assert_eq!(escolhas[0], CurveTip::Chain);
    assert_eq!(escolhas[1], CurveTip::Straight);
    assert!(
        matches!(escolhas[2], CurveTip::Bone(_)) && matches!(escolhas[3], CurveTip::Bone(_)),
        "as duas ultimas sao os filhos: {escolhas:?}"
    );
    assert_eq!(o.ligado, 0, "o nascimento e' a corrente");
    assert_eq!(o.escondidos, 0);
    // ⭐ E a `choice_at` devolve exactamente o que a lista prometeu, linha a linha.
    for (i, esperado) in escolhas.iter().enumerate() {
        assert_eq!(choice_at(&sim, ramificado, i), Some(*esperado), "linha {i}");
    }
    assert_eq!(choice_at(&sim, ramificado, escolhas.len()), None);
    // O osso de UM filho tem três linhas.
    let (o1, _) = options(&sim, um).expect("tambem esta' em From Chain");
    assert_eq!(o1.rotulos.len(), 3, "{:?}", o1.rotulos);
}

/// ⛔⛔ **COM AS ALÇAS AUTORADAS NÃO HÁ PERGUNTA** — ali ninguém deriva tangente de vizinho nenhum,
/// e um selector seria um controlo morto (a espécie que este módulo já pagou no botão `Smooth`).
#[test]
fn a_bone_with_authored_handles_has_no_tip_question() {
    let (mut sim, ramificado, _) = cena();
    sim.world_mut()
        .get_mut::<ph2d_skeleton_ecs::Bone>(ramificado)
        .expect("osso")
        .handles = ph2d_skeleton::bend::Handles::Authored;
    assert!(options(&sim, ramificado).is_none());
    assert!(choice_at(&sim, ramificado, 2).is_none());
}

/// ⭐⭐ **A ESCOLHA LIGADA É A QUE A LISTA DE AGORA CONHECE** — e uma que já não resolve cai no
/// `Chain`, que é exactamente o que a lei faz com ela.
#[test]
fn the_selected_row_falls_back_to_the_chain_when_the_choice_no_longer_resolves() {
    let (mut sim, ramificado, _) = cena();
    let (_, escolhas) = options(&sim, ramificado).expect("lista");
    sim.world_mut()
        .get_mut::<ph2d_skeleton_ecs::Bone>(ramificado)
        .expect("osso")
        .curve_tip = escolhas[3];
    assert_eq!(options(&sim, ramificado).expect("lista").0.ligado, 3);
    sim.world_mut()
        .get_mut::<ph2d_skeleton_ecs::Bone>(ramificado)
        .expect("osso")
        .curve_tip = CurveTip::Bone(ph2d_ecs::StableId(u64::MAX));
    assert_eq!(
        options(&sim, ramificado).expect("lista").0.ligado,
        0,
        "uma escolha que nao resolve tem de ler-se como a corrente"
    );
}

/// ⛔ **UM OSSO COM MAIS FILHOS QUE O POOL NÃO MENTE** — ele diz quantos ficaram de fora.
///
/// ⚠️ *Uma lista truncada em silêncio é um painel a esconder o que existe* — e o pool é fixo porque
/// o chrome não cunha ids em tempo de execução.
#[test]
fn a_bone_with_more_children_than_the_pool_says_how_many_are_hidden() {
    let mut sim = SimWorld::default();
    let raiz = ph2d_ecs::Entity::from_bits(
        ph2d_skeleton_live::bone::create(&mut sim, None, [0.0, 0.0], [1.0, 0.0]).expect("raiz"),
    );
    let extra = 3;
    for i in 0..MAX_TIP_CHILDREN + extra {
        let y = (i as f64) * 0.1;
        ph2d_skeleton_live::bone::create(&mut sim, Some(raiz), [1.0, 0.0], [2.0, y])
            .expect("filho");
    }
    sim.world_mut()
        .get_mut::<ph2d_skeleton_ecs::Bone>(raiz)
        .expect("osso")
        .handles = ph2d_skeleton::bend::Handles::Auto;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (o, escolhas) = options(&sim, raiz).expect("lista");
    assert_eq!(escolhas.len(), 2 + MAX_TIP_CHILDREN);
    assert_eq!(
        o.escondidos, extra,
        "os que nao couberam tem de ser contados"
    );
}

/// ⭐⭐⭐ **O POOL DE IDS ALCANÇA TODA LINHA QUE A LEI OFERECE** — a igualdade entre o chrome e a
/// família, que nenhuma das duas crates pode medir sozinha.
///
/// ⚠️ **A `ph2d-editor-core` está ABAIXO desta crate** e não pode nomear o [`MAX_TIP_CHILDREN`];
/// esta vê as duas, e é por isso que o gate mora aqui (o mesmo desenho do
/// `the_action_picker_reaches_every_clip_the_document_can_hold`).
///
/// ⛔ **Um pool menor esconderia filhos que EXISTEM** (o artista veria uma lista que mente) e um
/// maior deixaria ids **mortos** — pintados por ninguém.
#[test]
fn the_tip_picker_reaches_every_child_the_law_offers() {
    assert_eq!(
        ph2d_editor_core::ids::MAX_TIP_OPTIONS,
        2 + MAX_TIP_CHILDREN,
        "o pool de ids do selector de ponta e a lista que a lei oferece discordam — ou ha' filho \
         sem linha (inalcancavel) ou linha sem filho (morta)"
    );
}
