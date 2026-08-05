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
    assert!(publish(&[host, child], &states, None).is_none());
}

/// **A seção é oferecida a qualquer forma única, com estados ou sem.**
///
/// ⚠️ A face VAZIA é a importante: uma seção que só existisse onde já há estados tornaria a
/// feature alcançável apenas onde ela já foi usada, ou seja em lugar nenhum. É a mesma lei da
/// seção de física.
#[test]
fn the_section_is_offered_before_there_is_anything_to_show() {
    let states = StateSets::default();
    let v = publish(&[1], &states, None).expect("a secao e' oferecida numa forma sem estados");
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
    let v = publish(&[1], &StateSets::default(), None).expect("a secao e' oferecida");
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

/// A maior distância do centro do retângulo a um vértice — a assinatura de APARÊNCIA de uma
/// quina: afiada ela é o ponto mais longe; arredondada, a quina foi recuada.
fn corner_reach(g: &ph2d_vec_scene::VecPath) -> f64 {
    g.verts
        .iter()
        .map(|v| v.anchor[0].hypot(v.anchor[1]))
        .fold(0.0_f64, f64::max)
}

/// **REPRO (Enio, 2026-08-05): mudar a FORMA entre dois estados não animava.**
///
/// *"A animação não funciona para mudanças nos nós da shape, nem para mudanças feitas com as
/// tools Fillet, Chamfer, Width e Cut."*
///
/// ⚠️ E a causa é a que um gate nunca vê sozinho: o [`ObjectPose::geometry`] **existia**, a
/// [`ph2d_ui_state::Transition`] **sabia** construir o `Plan`, o `install` **sabia** escrever os
/// verts — e **ninguém preenchia o campo**. Uma capacidade sem PORTA passa em todos os gates:
/// eles leem quem CONSOME o campo, e o defeito estava em quem o ESCREVE.
#[test]
fn a_node_edit_between_two_states_morphs() {
    let (mut sim, mut scene, map, host, _child) = scene_with_host_and_child();
    let mut states = StateSets::default();

    let rec = |sim: &mut SimWorld, scene: &mut VecScene, states: &mut StateSets, role| {
        apply(sim, scene, &map, &[host], states, UiStateEdit::Record(role));
    };

    rec(&mut sim, &mut scene, &mut states, StateRole::Default);
    // O artista puxa um nó no modo Node.
    scene
        .paths_mut()
        .iter_mut()
        .find(|p| p.id == host)
        .unwrap()
        .verts[1]
        .anchor = [6.0, 0.0];
    rec(&mut sim, &mut scene, &mut states, StateRole::Hover);

    let a = states.role(host, StateRole::Default).unwrap();
    let b = states.role(host, StateRole::Hover).unwrap();
    let tr = ph2d_ui_state::Transition::new(&a.objects, &b.objects);
    assert_eq!(
        tr.plans_built(),
        1,
        "a mudanca de forma nao produziu casamento nenhum: o estado nao gravou a geometria"
    );

    let mid = tr.at(0.5);
    let g = mid
        .iter()
        .find(|p| p.id == host)
        .and_then(|p| p.geometry.as_ref())
        .expect("a pose do meio tem de carregar a forma interpolada");
    let far = g.verts.iter().map(|v| v.anchor[0]).fold(0.0_f64, f64::max);
    assert!(
        far > 2.5 && far < 5.5,
        "o meio do caminho nao esta ENTRE as duas formas (2.0 e 6.0): {far}"
    );
}

/// **O Fillet e o Chamfer também morfam — porque o casamento é feito na geometria COZIDA.**
///
/// ⚠️ Este gate é o que decide *fonte ou cozido* (ADR-0121). O raio de quina vive **dentro do
/// vértice**, então casar as FONTES daria dois quadrados de vértices idênticos — nenhum `Plan`,
/// e a quina apareceria de uma vez só no fim. O artista vê o COZIDO, e é ele que tem de viajar.
#[test]
fn a_fillet_between_two_states_rounds_along_the_way() {
    let (mut sim, mut scene, map, host, _child) = scene_with_host_and_child();
    let mut states = StateSets::default();

    // Um quadrado centrado, para a distância ao centro medir a quina.
    *scene.paths_mut().iter_mut().find(|p| p.id == host).unwrap() = ph2d_vec_scene::VecPath {
        id: host,
        ..ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0])
    };
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );

    // O Fillet: mesma curva autorada, raio novo em cada quina.
    for v in &mut scene
        .paths_mut()
        .iter_mut()
        .find(|p| p.id == host)
        .unwrap()
        .verts
    {
        v.corner_radius = 0.6;
    }
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Hover),
    );

    let a = states.role(host, StateRole::Default).unwrap();
    let b = states.role(host, StateRole::Hover).unwrap();
    let tr = ph2d_ui_state::Transition::new(&a.objects, &b.objects);
    assert_eq!(
        tr.plans_built(),
        1,
        "o Fillet nao produziu casamento: os dois lados foram comparados na FONTE, onde eles \
         sao o mesmo quadrado"
    );

    let sharp = corner_reach(
        a.objects
            .iter()
            .find(|p| p.id == host)
            .unwrap()
            .geometry
            .as_ref()
            .unwrap(),
    );
    let mid = tr.at(0.5);
    let round = corner_reach(
        mid.iter()
            .find(|p| p.id == host)
            .and_then(|p| p.geometry.as_ref())
            .expect("forma no meio"),
    );
    assert!(
        round < sharp - 1e-6,
        "a quina do meio do caminho nao recuou nada: {round} contra {sharp} da afiada"
    );
}

/// **A geometria AUTORADA sobrevive ao Show — as alças de quina e a pilha de efeitos voltam.**
///
/// ⚠️ Este é o gate de PERDA DE TRABALHO, e ele existe porque a transição passa pelo documento:
/// a pose do meio do caminho é geometria já **cozida** (o raio realizado, os efeitos aplicados),
/// e escrevê-la sem a devolver deixaria o artista sem as alças que ele acabou de arrastar. A
/// chegada instala a pose AUTORADA, e é ela que faz a passagem ser transitória.
///
/// ⚠️ E a metade do MEIO é a que impede a dobra: a pilha tem de chegar ao documento **VAZIA**
/// enquanto a forma está cozida, senão o render a aplica outra vez sobre uma geometria que já a
/// tem.
#[test]
fn a_show_gives_the_authored_shape_back_stack_and_all() {
    use ph2d_vec_scene::effect::{FxEntry, PathEffect};
    use ph2d_vec_scene::fx_trim::TrimSpec;

    let (mut sim, mut scene, map, host, _child) = scene_with_host_and_child();
    let mut states = StateSets::default();

    let authored = {
        let p = scene.paths_mut().iter_mut().find(|p| p.id == host).unwrap();
        for v in &mut p.verts {
            v.corner_radius = 0.25;
        }
        p.effects
            .push(FxEntry::new(PathEffect::Trim(TrimSpec::default())));
        p.clone()
    };
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    scene
        .paths_mut()
        .iter_mut()
        .find(|p| p.id == host)
        .unwrap()
        .verts[1]
        .anchor = [7.0, 0.0];
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Hover),
    );

    let a = states
        .role(host, StateRole::Default)
        .unwrap()
        .objects
        .clone();
    let b = states.role(host, StateRole::Hover).unwrap().objects.clone();
    let tr = ph2d_ui_state::Transition::new(&b, &a);

    // O MEIO: cozida, e a pilha tem de sair do documento enquanto ela lá está.
    for p in tr.at(0.5) {
        install(&mut sim, &mut scene, &map, &p);
    }
    let mid = scene.paths().iter().find(|p| p.id == host).unwrap();
    assert!(
        mid.effects.is_empty(),
        "a forma do meio ja esta cozida e a pilha ficou no documento: o render vai aplica-la \
         DUAS vezes"
    );

    // A CHEGADA: a pose autorada, ao bit.
    for p in &a {
        install(&mut sim, &mut scene, &map, p);
    }
    let back = scene.paths().iter().find(|p| p.id == host).unwrap();
    assert_eq!(
        back.effects, authored.effects,
        "a pilha de efeitos nao voltou depois do Show"
    );
    assert!(
        back.verts
            .iter()
            .all(|v| (v.corner_radius - 0.25).abs() < 1e-12),
        "os raios de quina foram ASSADOS pela transicao: as alcas do artista sumiram"
    );
}

/// **Dois estados que diferem SÓ na pilha de efeitos também morfam.**
///
/// ⚠️ Este gate existe por causa de uma mutação que sobreviveu: `same_shape` comparando apenas
/// `verts`+`closed` passava em tudo o resto, porque um Fillet mexe nos verts (o raio mora dentro
/// do vértice) e uma edição de nó também. **O único par que separa os dois testes é aquele em
/// que a fonte é idêntica e só a pilha muda** — e ali a forma desenhada difere sem que um único
/// vértice se mexa.
#[test]
fn two_states_that_differ_only_in_the_effect_stack_morph() {
    use ph2d_vec_scene::effect::{FxEntry, PathEffect};
    use ph2d_vec_scene::fx_warp::BloatSpec;

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
    scene
        .paths_mut()
        .iter_mut()
        .find(|p| p.id == host)
        .unwrap()
        .effects
        .push(FxEntry::new(PathEffect::Bloat(BloatSpec::default())));
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Hover),
    );

    let a = states.role(host, StateRole::Default).unwrap();
    let b = states.role(host, StateRole::Hover).unwrap();
    assert_eq!(
        a.objects
            .iter()
            .find(|p| p.id == host)
            .unwrap()
            .geometry
            .as_ref()
            .unwrap()
            .verts,
        b.objects
            .iter()
            .find(|p| p.id == host)
            .unwrap()
            .geometry
            .as_ref()
            .unwrap()
            .verts,
        "a fixture nao contem o fenomeno: os verts tinham de ser IDENTICOS nos dois estados"
    );
    let tr = ph2d_ui_state::Transition::new(&a.objects, &b.objects);
    assert_eq!(
        tr.plans_built(),
        1,
        "a pilha de efeitos mudou a forma desenhada e ninguem a casou"
    );
}

/// **O Width Tool também anima — o perfil é pose, e ele viaja.**
///
/// ⚠️ Ele é o único canal de forma que **não** mora no `VecPath`: é um componente ECS
/// (ADR-0148). Sem campo próprio na pose ele seria a única das ferramentas de desenho cuja
/// edição não animaria, e a ausência não teria nome nenhum — o traço trocaria de perfil de uma
/// vez no fim do Show, ou não trocaria de todo.
///
/// ⚠️ E a metade que este gate protege duas vezes: **uniforme é a AUSÊNCIA do componente**. Um
/// perfil inerte guardado no documento é uma relação que não desenha nada, e o painel passaria
/// a ver um perfil onde o artista não autorou nenhum.
#[test]
fn the_live_width_profile_is_pose_and_it_travels() {
    use ph2d_ecs::VecStrokeProfile;
    use ph2d_vec_scene::{WidthProfile, WidthStops};

    let (mut sim, mut scene, map, host, _child) = scene_with_host_and_child();
    let mut states = StateSets::default();
    let he = ph2d_ecs::Entity::from_bits(map[&host]);

    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    let bulge = WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.2,
        position: 0.5,
    }
    .to_stops();
    sim.world_mut().entity_mut(he).insert(VecStrokeProfile {
        stops: bulge.clone(),
    });
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Hover),
    );

    let a = states
        .role(host, StateRole::Default)
        .unwrap()
        .objects
        .clone();
    let b = states.role(host, StateRole::Hover).unwrap().objects.clone();
    assert!(
        a.iter().find(|p| p.id == host).unwrap().width.is_none()
            && b.iter().find(|p| p.id == host).unwrap().width.as_ref() == Some(&bulge),
        "a autoria nao leu o perfil do componente"
    );

    // No MEIO do caminho o traço está entre o uniforme e o bojo — nem um nem outro.
    let mid = ph2d_ui_state::Transition::new(&a, &b).at(0.5);
    for p in &mid {
        install(&mut sim, &mut scene, &map, p);
    }
    let live = sim
        .world()
        .get::<VecStrokeProfile>(he)
        .map(|w| w.stops.at(0.5))
        .expect("o perfil do meio nao chegou ao mundo");
    assert!(
        live > 1.05 && live < 1.75,
        "o pico do meio do caminho nao esta entre 1.0 (uniforme) e 1.8 (o bojo): {live}"
    );

    // E a volta ao Default REMOVE o componente, em vez de guardar um perfil inerte.
    for p in &a {
        install(&mut sim, &mut scene, &map, p);
    }
    assert!(
        sim.world().get::<VecStrokeProfile>(he).is_none(),
        "o traco voltou ao uniforme e o componente ficou no documento"
    );
    let _ = WidthStops::default();
}
