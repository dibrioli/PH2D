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
use ph2d_ecs::Transform;
use ph2d_vec_scene::{Marker, Rgba8, StrokeSpec, VecPathId, line};

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
    contains_path(sim, scene, &Default::default(), e, id, p, r)
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
