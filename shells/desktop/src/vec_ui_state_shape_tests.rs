//! Gates da **FORMA** nos estados de UI (plano UI/UX W7) — o que um estado grava de geometria e
//! como ela atravessa a transição.
//!
//! ⚠️ O assunto é um só: *mudar o desenho entre dois estados anima*. As ferramentas que o report
//! do Enio nomeia entram todas por aqui — modo Node e Fillet/Chamfer pelo `VecPath`, o Width Tool
//! pelo componente ECS, e o Cut pela fronteira que ele expõe (ele destrói o id).

use super::*;

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

/// **O que o CUT faz aos estados, medido — e onde está a fronteira.**
///
/// A faca **destrói o id** (`scene.remove_path` + peças novas), e um estado guarda poses
/// chaveadas por `VecPathId`. Daí as duas metades, e as duas são honestas:
///
/// - **Gravar os dois estados DEPOIS do corte funciona como qualquer outra forma:** a peça é uma
///   forma normal, e editá-la morfa. É este o caminho que a wave entrega.
/// - **Um estado gravado ANTES do corte não ressuscita o que a faca consumiu.** O membro sai do
///   documento, e a transição fá-lo desvanecer (`Leaving`) em vez de o trazer de volta.
///
/// ⚠️ **A fronteira tem nome:** um estado sabe repor uma POSE, não CRIAR um objeto. Ressuscitar
/// exigiria que ele fosse dono do conjunto de objetos (criar path, entidade, hierarquia e um id
/// novo — que o estado já não referencia), e isso é outra feature, não um campo a mais.
#[test]
fn the_cut_destroys_the_id_and_the_state_animates_the_pieces_not_the_ghost() {
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

    // A faca: o filho some do documento e no lugar dele ficam duas peças novas.
    scene.remove_path(child);
    let piece = scene.push_path(rectangle([0.0, 0.0], [0.5, 1.0]));
    let map2 = {
        let mut m = map.clone();
        m.remove(&child);
        let e = sim
            .world_mut()
            .spawn((
                Name("Piece".into()),
                Transform::IDENTITY,
                ph2d_ecs::VecPathRef(piece),
            ))
            .id();
        sim.world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::ChildOf(ph2d_ecs::Entity::from_bits(map[&host])));
        m.insert(piece, e.to_bits());
        m
    };
    apply(
        &mut sim,
        &mut scene,
        &map2,
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
    let tr = ph2d_ui_state::Transition::new(&a, &b);
    let mid = tr.at(0.5);
    let ghost = mid
        .iter()
        .find(|p| p.id == child)
        .expect("o membro consumido");
    assert!(
        ghost.opacity > 0.0 && ghost.opacity < 1.0,
        "o membro que a faca consumiu tinha de DESVANECER, e nao de saltar: {}",
        ghost.opacity
    );
    let born = mid.iter().find(|p| p.id == piece).expect("a peca nova");
    assert!(
        born.opacity > 0.0 && born.opacity < 1.0,
        "a peca nova tinha de ENTRAR desvanecendo: {}",
        born.opacity
    );

    // E a metade que a wave ENTREGA: dois estados gravados depois do corte morfam a peça.
    let mut after = StateSets::default();
    apply(
        &mut sim,
        &mut scene,
        &map2,
        &[host],
        &mut after,
        UiStateEdit::Record(StateRole::Default),
    );
    scene
        .paths_mut()
        .iter_mut()
        .find(|p| p.id == piece)
        .unwrap()
        .verts[1]
        .anchor = [3.0, 0.0];
    apply(
        &mut sim,
        &mut scene,
        &map2,
        &[host],
        &mut after,
        UiStateEdit::Record(StateRole::Hover),
    );
    let tr = ph2d_ui_state::Transition::new(
        &after.role(host, StateRole::Default).unwrap().objects,
        &after.role(host, StateRole::Hover).unwrap().objects,
    );
    assert_eq!(
        tr.plans_built(),
        1,
        "editar uma peca DEPOIS do corte tinha de morfar como qualquer outra forma"
    );
}
