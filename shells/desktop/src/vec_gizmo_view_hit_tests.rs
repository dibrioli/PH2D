//! **O que se VE, se CLICA** — os gates da area de captura de um traco.
//!
//! Enio: *"selecionar a seta esta dificil pois a area da seta e fina"*. A causa nao era o raio
//! de captura: era que METADE DO DESENHO nao existia para o mouse. A ponta de seta e construida
//! a partir do caminho + do `StrokeSpec` (`stroke_head`) e nunca entrou no `VecPath`, entao o
//! hit-test jamais a viu — clicar no triangulo, que e a parte gorda e a que o olho mira, nao
//! selecionava nada.
//!
//! **A calibragem que estes testes exigiram, e que vale registrar:** a cabeca e proporcional a
//! LARGURA do traco. Com uma linha curta e um traco grosso, o triangulo cobre metade da cena — e
//! a primeira versao destes testes ficou vermelha nas asserções NEGATIVAS, porque os pontos que
//! eu supunha "longe da seta" caiam dentro dela. O bug era do teste. Uma linha longa e um traco
//! fino poem cada coisa no seu lugar.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Transform};
use ph2d_vec_scene::{Marker, Rgba8, StrokeSpec, VecPathId, line, rectangle};

/// Uma linha horizontal LONGA, de (0,0) a (100,0), com traco de largura `width` e a ponta
/// `marker` no fim.
fn scene_with_line(
    width: f64,
    marker: Marker,
    scale: f64,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut p = line([0.0, 0.0], [100.0, 0.0]);
    let mut s = StrokeSpec::new(Rgba8::new(255, 255, 255, 255), width);
    s.marker_end = marker;
    s.marker_scale = scale;
    p.stroke = Some(s);
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(0.0, 0.0),
                ..Default::default()
            },
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    (sim, scene, map, id)
}

fn hits(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap, p: [f32; 2], r: f64) -> bool {
    let id = *map.keys().next().expect("um path");
    let e = Entity::from_bits(map[&id]);
    contains_path(
        sim,
        scene,
        &Default::default(),
        &Default::default(),
        e,
        id,
        p,
        r,
    )
}

/// **O GATE.** O corpo da ponta de seta pega o clique.
///
/// O raio de captura usado aqui e MINUSCULO (0,01) e a linha e fina: nem a folga nem a tinta
/// explicam um acerto a 0,7 do eixo. So a cabeca explica — e ela so existe se o hit-test a
/// CONSTROI, que era exatamente o que faltava.
#[test]
fn the_arrowhead_itself_is_clickable_not_just_the_thin_line() {
    let (sim, scene, map, _) = scene_with_line(1.0, Marker::Triangle, 1.0);
    let tiny = 0.01;

    // Dentro do corpo do triangulo (que se estende para TRAS da ponta em (100, 0)), acima do
    // eixo da linha — fora do alcance da folga minuscula e da meia-tinta (0,5).
    assert!(
        hits(&sim, &scene, &map, [98.5_f32, 0.7], tiny),
        "clicar DENTRO da cabeca da seta nao selecionou nada — a parte gorda do desenho \
         continua invisivel para o mouse"
    );

    // NAO-VACUO: o MESMO afastamento do eixo, mas no MEIO da linha (longe da cabeca), nao pega.
    // Se pegasse, o verde acima seria acidente da folga, nao da cabeca.
    assert!(
        !hits(&sim, &scene, &map, [50.0_f32, 0.7], tiny),
        "teste VACUO: um ponto igualmente afastado do eixo, mas longe da cabeca, tambem pega — \
         entao o verde acima nao prova que a cabeca entrou no hit-test"
    );
}

/// **A TINTA QUE SE VE CONTA.** Uma linha grossa e pegavel em toda a largura com que ela e
/// PINTADA — nao so nos poucos pixels centrais.
///
/// Sem a meia-largura no raio, o hit-test media a distancia ate a CURVA e ignorava a espessura:
/// o usuario clicava visivelmente EM CIMA do traco e nada acontecia. Sem ponta de seta, para
/// isolar a tinta.
#[test]
fn a_thick_line_is_clickable_across_the_ink_it_actually_paints() {
    // Largura 4 => a tinta vai de y = -2 a y = +2.
    let (sim, scene, map, _) = scene_with_line(4.0, Marker::None, 1.0);
    let tiny = 0.01; // folga quase nula: so a TINTA pode explicar um acerto

    assert!(
        hits(&sim, &scene, &map, [50.0_f32, 1.8], tiny),
        "clicar dentro da tinta da linha (y = 1.8, meia-largura 2.0) nao pegou: o hit-test \
         ignora a espessura com que a linha e desenhada"
    );
    // Fora da tinta E fora da folga: nao pega. Sem esta metade, bastaria devolver `true` sempre.
    assert!(
        !hits(&sim, &scene, &map, [50.0_f32, 3.0], tiny),
        "pegou FORA da tinta e fora da folga — o raio virou generoso demais"
    );
}

/// Uma LINHA ganha mais folga que a borda de uma forma fechada: ela nao tem interior, entao a
/// folga *e* a area clicavel. Na borda de uma forma, a folga e so o fio — e uma folga grande ali
/// roubaria cliques do que esta atras.
#[test]
fn an_open_path_gets_a_more_generous_slop_than_a_closed_ones_border() {
    let (sim, scene, map, _) = scene_with_line(0.0, Marker::None, 1.0);
    let r = 1.0;
    // 1,4 esta FORA da folga base (1,0) e DENTRO da ampliada (1,75).
    assert!(
        hits(&sim, &scene, &map, [50.0_f32, 1.4], r),
        "a linha nao ganhou a folga ampliada: a {OPEN_PATH_HIT_K}x de {r} deveria pegar"
    );
    // E o teto continua existindo — a folga ampliada nao pode virar um ima.
    assert!(
        !hits(&sim, &scene, &map, [50.0_f32, 2.5], r),
        "a folga ampliada nao pode virar um ima que pega qualquer clique"
    );
}

/// A borda de uma forma FECHADA NAO ganha a folga ampliada — ela tem interior para mirar, e uma
/// folga grande ali roubaria cliques de quem esta atras dela.
#[test]
fn a_closed_shape_keeps_the_tight_border_slop() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [10.0, 10.0]));
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(0.0, 0.0),
                ..Default::default()
            },
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());

    let r = 1.0;
    // Dentro: o interior pega, sempre.
    assert!(hits(&sim, &scene, &map, [5.0_f32, 5.0], r));
    // 0,5 FORA da borda: dentro da folga base.
    assert!(hits(&sim, &scene, &map, [10.5_f32, 5.0], r));
    // 1,4 fora: uma LINHA pegaria (folga ampliada), uma forma fechada NAO.
    assert!(
        !hits(&sim, &scene, &map, [11.4_f32, 5.0], r),
        "a borda de uma forma fechada ganhou a folga ampliada — ela vai roubar cliques do que \
         esta atras dela"
    );
}

/// **DENTRO DE UMA MOLDURA, O CLIQUE PEGA O FILHO** (Enio 2026-08-02: *"quando dentro do Frame
/// não consigo selecionar as formas"*).
///
/// A moldura é o ÚLTIMO membro da própria sub-árvore na pilha de z — é o que emparelha o push e
/// o pop da camada de recorte —, logo a mais À FRENTE. O renderer sabe disso e antecipa o
/// desenho dela; o apontar não sabia, e o retângulo dela ganhava todo clique dos filhos. Duas
/// respostas para *"onde nesta pilha está esta moldura?"*: desenhada no fundo, apontada na
/// frente.
///
/// ⚠️ **A premissa da fixture é o intervalo** (`clips`): é ele que diz *quem foi antecipado*, e
/// é dele que a demoção lê. Uma moldura sem intervalo não é demovida, e é isso que impede o
/// apontar de discordar do desenho pelo outro lado.
///
/// Nasceu VERMELHO com a moldura em primeiro.
#[test]
fn a_click_inside_a_frame_lands_on_the_child_not_the_frame() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();

    // ⚠️ **O FILHO entra na cena PRIMEIRO, e isso é a premissa.** A pilha de z que o produto
    // projeta (`vec_zorder::z_order`) põe o contêiner por ÚLTIMO na sub-árvore dele — logo à
    // FRENTE —, e é dessa posição que vem o defeito. Empurrar a moldura primeiro deixaria o
    // filho na frente por acidente da fixture, e o gate ficaria verde sem conter o fenómeno.
    let kid = scene.push_path(rectangle([2.0, 2.0], [4.0, 4.0]));
    let frame = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    let fe = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(frame)))
        .id();
    let ke = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(kid), ChildOf(fe)))
        .id();
    map.insert(frame, fe.to_bits());
    map.insert(kid, ke.to_bits());

    let vs = VecViewState {
        parent_spans: vec![ph2d_vec_scene::VecParentSpan {
            parent: frame,
            first: kid,
            clip: false, // uma moldura de LAYOUT: e' fundo, nao recorta
        }],
        ..Default::default()
    };
    // Um ponto que os DOIS contêm — o interior do filho está dentro da moldura por construção.
    let hits = pick_all_at_world(
        &sim,
        &scene,
        &Default::default(),
        &vs,
        &map,
        [3.0, 3.0],
        0.0,
    );
    assert_eq!(
        hits.first(),
        Some(&ke.to_bits()),
        "o clique pegou a MOLDURA — dentro dela nenhuma forma e' selecionavel: {hits:?}"
    );
    assert!(
        hits.contains(&fe.to_bits()),
        "e a moldura tem de continuar na lista, atras dele (o clique-ciclico a alcanca)"
    );
}

/// **Uma forma que NÃO foi antecipada continua na frente dos filhos dela** — o apontar segue o
/// desenho, e o desenho só antecipa quem tem intervalo.
///
/// ⚠️ É a metade que impede a cura de virar o defeito espelhado: sem ela, uma demoção cega faria
/// toda forma-pai perder o clique para quem está dentro, inclusive as que pintam por cima.
#[test]
fn a_parent_that_was_not_hoisted_still_wins_the_click() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();

    // A mesma premissa do gate irmão: o pai é o mais à frente na ordem crua.
    let kid = scene.push_path(rectangle([2.0, 2.0], [4.0, 4.0]));
    let parent = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    let pe = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(parent)))
        .id();
    let ke = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(kid), ChildOf(pe)))
        .id();
    map.insert(parent, pe.to_bits());
    map.insert(kid, ke.to_bits());

    // Sem `clips`: ninguem foi antecipado.
    let vs = VecViewState::default();
    let hits = pick_all_at_world(
        &sim,
        &scene,
        &Default::default(),
        &vs,
        &map,
        [3.0, 3.0],
        0.0,
    );
    assert_eq!(
        hits.first(),
        Some(&pe.to_bits()),
        "o pai desenha na frente e tem de ser apontado na frente: {hits:?}"
    );
}

/// **UMA FORMA QUE O LAYOUT COLOCOU É APONTADA ONDE ELA ESTÁ** (Enio 2026-08-02: *"os Path das
/// formas aparecem no lugar de origem e talvez por isso não consigo selecioná-las"*).
///
/// O passe do auto layout assa o resultado na `LiveGeometry`, e é por isso que a forma APARECE no
/// lugar certo. Quem não desenha geometria — o hit-test, as âncoras do modo Node, a caixa do
/// gizmo — lê a pose AUTORADA, e ela não se mexeu: o clique procurava a forma onde ela saiu.
///
/// ⚠️ **O oráculo tem as DUAS metades**, e é o par que o torna capaz de falhar: onde a forma
/// ESTÁ agora tem de pegar, e onde ela ESTAVA não pode mais.
#[test]
fn a_shape_the_layout_moved_is_picked_where_it_now_is() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());

    // A moldura empurrou-a 10 para a direita.
    let vs = VecViewState {
        poses: vec![(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 10.0, 0.0]))],
        ..Default::default()
    };
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [11.0, 1.0],
            0.0
        ),
        Some(e.to_bits()),
        "o clique ONDE A FORMA ESTA' nao a pegou"
    );
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &vs,
            &map,
            [1.0, 1.0],
            0.0
        ),
        None,
        "e ela continua clicavel no lugar de ORIGEM — o hit-test ficou nos dois sitios"
    );

    // O controle: sem pose, ela e' apontada onde foi autorada, como sempre.
    let bare = VecViewState::default();
    assert_eq!(
        pick_at_world(
            &sim,
            &scene,
            &Default::default(),
            &bare,
            &map,
            [1.0, 1.0],
            0.0
        ),
        Some(e.to_bits())
    );
}
