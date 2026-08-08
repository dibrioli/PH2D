//! Gates da AUTORIA dos estados de UI (plano UI/UX W7).

use super::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_vec_scene::{VecScene, rectangle};

/// Um mundo com uma forma-hospedeiro e um filho, ligados na hierarquia.
///
/// ⚠️ A fixture tem **filho de propósito**: um botão não é uma forma, e um estado que só gravasse
/// o hospedeiro deixaria de fora justamente o que se move num hover. Sem o filho, a lei
/// *"o estado é da SUB-ÁRVORE"* seria verde por vácuo.
fn scene_with_host_and_child() -> (SimWorld, VecScene, VecEntityMap, VecPathId, VecPathId) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::default();

    let host_id = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let child_id = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));

    let he = sim
        .world_mut()
        .spawn((
            Name("Host".into()),
            Transform::IDENTITY,
            ph2d_ecs::VecPathRef(host_id),
        ))
        .id();
    let ce = sim
        .world_mut()
        .spawn((
            Name("Child".into()),
            {
                let mut t = Transform::IDENTITY;
                t.translation.x = 5.0;
                t
            },
            ph2d_ecs::VecPathRef(child_id),
        ))
        .id();
    sim.world_mut().entity_mut(ce).insert(ph2d_ecs::ChildOf(he));

    let mut map = VecEntityMap::new();
    map.insert(host_id, he.to_bits());
    map.insert(child_id, ce.to_bits());
    (sim, scene, map, host_id, child_id)
}

/// **O estado grava a SUB-ÁRVORE, não só o hospedeiro.**
#[test]
fn recording_captures_the_whole_subtree() {
    let (mut sim, mut scene, map, host, child) = scene_with_host_and_child();
    let mut states = StateSets::default();

    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );

    let s = states.role(host, StateRole::Default).expect("gravado");
    let ids: Vec<_> = s.objects.iter().map(|p| p.id).collect();
    assert!(
        ids.contains(&host) && ids.contains(&child),
        "o estado deixou de fora um membro da sub-arvore: {ids:?}"
    );
    let c = s.objects.iter().find(|p| p.id == child).unwrap();
    assert!(
        (c.translation[0] - 5.0).abs() < 1e-9,
        "a pose do filho nao foi lida do Transform LOCAL dele"
    );
}

/// **A pose é a LOCAL, e é isso que faz o botão sobreviver a ser movido.**
///
/// ⚠️ O gate move o HOSPEDEIRO depois de gravar e exige que a pose do filho **não** mude: com a
/// pose de mundo os dois números discordariam no instante em que alguém arrastasse o botão para
/// outro canto da tela, e os estados dele passariam a descrever um lugar que ele já não ocupa.
#[test]
fn the_pose_is_local_so_moving_the_host_does_not_invalidate_it() {
    let (mut sim, mut scene, map, host, child) = scene_with_host_and_child();
    let mut states = StateSets::default();

    let before = {
        apply(
            &mut sim,
            &mut scene,
            &map,
            &[host],
            &mut states,
            UiStateEdit::Record(StateRole::Default),
        );
        states.role(host, StateRole::Default).unwrap().objects[..]
            .iter()
            .find(|p| p.id == child)
            .unwrap()
            .translation
    };

    // O artista arrasta o botão inteiro.
    let he = Entity::from_bits(*map.get(&host).unwrap());
    sim.world_mut()
        .get_mut::<Transform>(he)
        .unwrap()
        .translation
        .x = 100.0;

    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Hover),
    );
    let after = states.role(host, StateRole::Hover).unwrap().objects[..]
        .iter()
        .find(|p| p.id == child)
        .unwrap()
        .translation;

    assert!(
        (before[0] - after[0]).abs() < 1e-9,
        "mover o hospedeiro mudou a pose gravada do filho ({before:?} -> {after:?}) -- a pose \
         esta a ser lida em MUNDO"
    );
}

/// **O Show NÃO escreve pose: ele devolve o pedido.**
///
/// ⚠️ É a fronteira que impede a segunda porta para *"pôr a cena nesta pose"*. Uma escrita direta
/// aqui daria um salto instantâneo ao lado da máquina que anima — e a diferença entre as duas é
/// exactamente o tween que o artista autorou.
#[test]
fn show_asks_the_machine_instead_of_writing_the_pose() {
    let (mut sim, mut scene, map, host, child) = scene_with_host_and_child();
    let mut states = StateSets::default();
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    // Uma pose de Hover longe do repouso.
    let ce = Entity::from_bits(*map.get(&child).unwrap());
    sim.world_mut()
        .get_mut::<Transform>(ce)
        .unwrap()
        .translation
        .x = 42.0;
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Hover),
    );
    // Volta a cena ao repouso e PEDE o hover.
    sim.world_mut()
        .get_mut::<Transform>(ce)
        .unwrap()
        .translation
        .x = 5.0;

    let asked = apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Apply(StateRole::Hover),
    );
    assert_eq!(asked, Some((host, StateRole::Hover)));
    let x = sim.world().get::<Transform>(ce).unwrap().translation.x;
    assert!(
        (x - 5.0).abs() < 1e-6,
        "o Show escreveu a pose direto (x={x}) -- ele devia so' PEDIR, e quem mostra e' a maquina"
    );
}

/// **Seleção múltipla não tem hospedeiro.**
///
/// ⚠️ *"Gravar o estado destas três formas"* teria de escolher qual delas é o assunto, e escolher
/// em silêncio é como um estado nasce pendurado no objeto errado.
#[test]
fn a_multi_selection_has_no_host() {
    let (mut sim, mut scene, map, host, child) = scene_with_host_and_child();
    let mut states = StateSets::default();
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host, child],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    assert!(states.is_empty(), "uma selecao multipla gravou um estado");
    assert!(publish(&[host, child], &states, None, false).is_none());
}

/// **A seção é oferecida a qualquer forma única, com estados ou sem.**
///
/// ⚠️ A face VAZIA é a importante: uma seção que só existisse onde já há estados tornaria a
/// feature alcançável apenas onde ela já foi usada, ou seja em lugar nenhum. É a mesma lei da
/// seção de física.
#[test]
fn the_section_is_offered_before_there_is_anything_to_show() {
    let states = StateSets::default();
    let v =
        publish(&[1], &states, None, false).expect("a secao e' oferecida numa forma sem estados");
    assert_eq!(v.recorded, [false; 4]);
    assert!(
        (v.duration_s - 0.15).abs() < 1e-6,
        "o default nao chegou ao painel"
    );
}

/// **Os três verbos decodificam, e nada mais decodifica.**
#[test]
fn the_router_reads_the_three_verbs_and_only_them() {
    for (i, &role) in StateRole::ALL.iter().enumerate() {
        assert_eq!(
            ui_state_edit_for_id(ph2d_editor::ids::vector_state_record_id(i)),
            Some(UiStateEdit::Record(role))
        );
        assert_eq!(
            ui_state_edit_for_id(ph2d_editor::ids::vector_state_clear_id(i)),
            Some(UiStateEdit::Clear(role))
        );
        assert_eq!(
            ui_state_edit_for_id(ph2d_editor::ids::vector_state_apply_id(i)),
            Some(UiStateEdit::Apply(role))
        );
    }
    // CONTROLE POSITIVO: um id de outra seção não é um verbo de estado.
    assert_eq!(
        ui_state_edit_for_id(ph2d_editor::ids::VECTOR_WIDGET_WEAR),
        None
    );
}

/// **A tabela de ids cobre TODO papel do catálogo.**
///
/// ⚠️ `>=` e não igualdade, e a razão é a mesma do `MAX_WIDGET_KINDS`: um papel além do teto seria
/// pintado e **inalcançável** — não há conta-gotas por trás.
#[test]
fn every_role_has_a_row_of_ids() {
    assert!(
        ph2d_editor::ids::MAX_STATE_ROLES >= StateRole::ALL.len(),
        "um papel do catalogo ficou sem ids: {} < {}",
        ph2d_editor::ids::MAX_STATE_ROLES,
        StateRole::ALL.len()
    );
}

/// **Todo papel tem nome traduzido, e o painel recebe o do CATÁLOGO.**
///
/// ⚠️ A metade que importa é a segunda: os rótulos são publicados por esta porta, então um papel
/// novo aparece nomeado sem ninguém tocar na UI. A metade da tradução pega o oposto — uma chave
/// que ninguém pôs no catálogo volta como a própria chave, e a linha do painel diria
/// `panel.vector.states.role.…` ao artista.
#[test]
fn every_role_reaches_the_panel_with_a_translated_name() {
    let v = publish(&[1], &StateSets::default(), None, false).expect("a secao e' oferecida");
    for (i, &role) in StateRole::ALL.iter().enumerate() {
        let key = role.i18n_key();
        assert_ne!(
            v.role_labels[i], key,
            "o papel {role:?} nao tem traducao — a linha mostraria a chave crua"
        );
        assert_eq!(
            v.role_labels[i],
            ph2d_i18n::tr(key),
            "o rotulo publicado nao veio do catalogo"
        );
    }
}

/// **A régua da duração é UMA.** O teto que o painel usa para encher o trilho e o que o modelo
/// usa para clampar têm de ser o mesmo número.
///
/// ⚠️ Duas réguas fariam o artista arrastar o trilho até ao fim e ler um número que o clamp
/// recusa — o slider cheio mostrando um valor que o documento não tem.
#[test]
fn the_duration_ruler_is_one_number() {
    // O painel divide por 2.0 para encher o trilho; a shell multiplica por este valor ao
    // converter o track de volta.
    assert!(
        (ph2d_ui_state::MAX_DURATION_S - 2.0).abs() < 1e-9,
        "a regua do modelo mudou e a do painel nao: o trilho vai encher no valor errado"
    );
    // ⚠️ O default é o TOKEN do design system (`Duration::Fast`, *"button press"*), e não um
    // literal: a pergunta *"quanto tempo leva a reação de um controle neste app?"* já tinha dono.
    // O modelo carrega o mesmo número porque uma crate-folha de dados não depende do sistema de
    // tokens — e é este gate que impede os dois de divergirem.
    assert!(
        (ph2d_ui_state::DEFAULT_DURATION_S - f64::from(ph2d_tokens::Duration::Fast.secs())).abs()
            < 1e-6,
        "o default do modelo e o token do design system divergiram: o slider nasce num valor e o \
         documento noutro"
    );
}

/// **O interruptor da PREVIEW só é oferecido onde há o que pré-visualizar** (W7r).
///
/// ⚠️ A pergunta é sobre a CENA (*algum hospedeiro tem pose?*) dentro de uma seção da SELEÇÃO, e
/// é deliberado: a preview entrega o rato a todos os hospedeiros. O `None` é o que impede um
/// botão que não faz nada — [`crate::render_loop::ui_preview::UiPreview::enter`] recusa
/// exactamente a mesma condição, e um botão pintado sobre ela seria um clique que o artista não
/// tem como diagnosticar.
#[test]
fn the_preview_switch_is_offered_only_where_there_is_something_to_preview() {
    let states = StateSets::default();
    let v = publish(&[1], &states, None, false).expect("a secao e' oferecida");
    assert_eq!(
        v.preview, None,
        "o interruptor foi oferecido numa cena SEM pose nenhuma — ligar nao faria nada"
    );

    let (mut sim, mut scene, map, host, _child) = scene_with_host_and_child();
    let mut states = StateSets::default();
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    assert_eq!(
        publish(&[host], &states, None, false).unwrap().preview,
        Some(false),
        "com uma pose gravada o interruptor tem de existir, desligado"
    );
    assert_eq!(
        publish(&[host], &states, None, true).unwrap().preview,
        Some(true),
        "o estado LIGADO tem de chegar ao painel — senao o botao nunca acende"
    );
    // ⚠️ E a oferta é da CENA: um hospedeiro OUTRO que não o selecionado já basta.
    assert_eq!(
        publish(&[999], &states, None, false).unwrap().preview,
        Some(false),
        "a oferta seguiu a SELECAO em vez da cena — a preview dirige todos os hospedeiros"
    );
}

/// **A FORMA no estado** — o que a autoria grava de geometria, e como ela viaja. Módulo FILHO
/// (e não irmão) de propósito: a fixture `scene_with_host_and_child` continua a ser **uma
/// porta**, e um irmão obrigaria a duplicá-la ou a torná-la pública para fora do assunto.
#[path = "vec_ui_state_shape_tests.rs"]
mod shape;
