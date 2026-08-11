//! Os gates da varredura de corpo (`W-ShapeCast`).
//!
//! ⚠️ **O ORÁCULO desta wave é o RAIO**, não um literal: a varredura existe para
//! ver o que os raios deixam passar, então o gate central põe as duas portas
//! lado a lado sobre a MESMA geometria e afirma que elas **discordam** onde a
//! largura importa — e que **concordam** onde não importa.
//!
//! Módulo FILHO por `#[path]`, então `super::*` alcança o que o pai não exporta.

use super::*;
use crate::world::layers::LayerMatrix;
use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyType};
use rapier2d::geometry::ColliderBuilder;

/// Um corpo dinâmico **CAPSULA** em `(x, y)` — a forma de um personagem, e a
/// única em que *caixa envolvente* e *corpo* diferem de facto.
///
/// ⚠️ **Todo mundo aqui corre com GRAVIDADE ZERO**, e isso é fixture, não
/// conforto: uma varredura é uma CONSULTA, e o `step()` que indexa o BVH também
/// integra o corpo. Medido, um tique de queda vale `½·g·dt² = 1,4 mm` — o
/// suficiente para um gate que compara a varredura com um raio lançado da pose
/// ANTIGA falhar por 0,0014 e mandar procurar o defeito na porta.
fn capsule_body(w: &mut PhysicsWorld, x: f32, y: f32, half_h: f32, radius: f32) -> RigidBodyHandle {
    w.set_gravity(0.0, 0.0);
    let body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
        .translation(Vector2::new(x, y))
        .build();
    let h = w.bodies.insert(body);
    w.stamp_defaults(h);
    let c = ColliderBuilder::capsule_y(half_h, radius)
        .density(1.0)
        .build();
    let ch = w.colliders.insert_with_parent(c, h, &mut w.bodies);
    w.stamp_layer(ch, 0);
    h
}

/// **O QUE PASSA ENTRE DOIS RAIOS** — a razão de esta porta existir.
///
/// A cápsula mede 0,20 de meia-largura, então o sensor do agachar nasce com os
/// raios em `−0,20 · 0,00 · +0,20`. Um pilar de 8 cm posto em `+0,10` cai no
/// meio de duas amostras: **os três raios não o veem** e a varredura vê.
///
/// ⚠️ **Nasceu VERMELHO no produto, não aqui:** a `measure_the_gap_between_rays`
/// mede a consequência com o personagem inteiro — cabeça a 1,267 contra pedra em
/// 1,25. Este gate é a metade que pina o MECANISMO.
#[test]
fn the_sweep_sees_what_fits_between_two_rays() {
    const RADIUS: f32 = 0.2;
    const HALF_H: f32 = 0.3;
    const RISE: f32 = 0.5;
    let mut w = PhysicsWorld::new();
    // Um pilar de 8 cm de largura, face de baixo em y = 0,9.
    w.add_static_cuboid(0.10, 1.9, 0.04, 1.0);
    let body = capsule_body(&mut w, 0.0, 0.0, HALF_H, RADIUS);
    w.step();

    let top = HALF_H + RADIUS;
    for off in [-RADIUS, 0.0, RADIUS] {
        assert!(
            w.cast_ray([off, top], [0.0, 1.0], RISE, Some(body), 0)
                .is_none(),
            "controle: o raio em {off:+.2} tem de passar ao lado do pilar"
        );
    }

    let hit = w
        .sweep_body(body, [0.0, 1.0], RISE, 0)
        .expect("a varredura tem de ver o pilar que os tres raios perdem");
    // ⚠️ **O contacto é na quina MAIS PRÓXIMA do pilar (x = 0,06), não no centro
    // dele** — e é exactamente isto que uma amostra não sabe fazer: a varredura
    // acha o primeiro ponto de encontro sobre o perfil INTEIRO das duas formas.
    // A cápsula em x = 0,06 alcança `0,3 + sqrt(0,2² − 0,06²)` do centro.
    const NEAR_EDGE: f32 = 0.06;
    let reach = HALF_H + (RADIUS * RADIUS - NEAR_EDGE * NEAR_EDGE).sqrt();
    assert!(
        (hit.distance - (0.9 - reach)).abs() < 5.0e-3,
        "a varredura para onde o CORPO encosta: {} (esperado ~{:.4})",
        hit.distance,
        0.9 - reach
    );
}

/// **E onde a largura NÃO importa, as duas portas concordam** — a outra metade,
/// sem a qual o gate acima seria satisfeito por uma porta que responde qualquer
/// coisa.
///
/// Um teto largo e plano acima da cabeça: o raio central e a varredura medem a
/// MESMA distância, porque a cápsula toca o teto pelo próprio topo.
#[test]
fn over_a_flat_ceiling_the_sweep_and_the_ray_agree() {
    const RADIUS: f32 = 0.2;
    const HALF_H: f32 = 0.3;
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 3.0, 8.0, 1.0); // face de baixo em y = 2,0
    let body = capsule_body(&mut w, 0.0, 0.0, HALF_H, RADIUS);
    w.step();

    let top = HALF_H + RADIUS;
    let ray = w
        .cast_ray([0.0, top], [0.0, 1.0], 5.0, Some(body), 0)
        .expect("o raio central tem de ver o teto");
    let sweep = w
        .sweep_body(body, [0.0, 1.0], 5.0, 0)
        .expect("a varredura tem de ver o teto");
    assert!(
        (ray.distance - sweep.distance).abs() < 1.0e-3,
        "num teto plano as duas portas medem o mesmo: raio {} vs varredura {}",
        ray.distance,
        sweep.distance
    );
}

/// **O ponto e a normal saem em MUNDO** — medido, não herdado de um doc.
///
/// ⚠️ Os dois docs do parry discordam: o do `ShapeCastHit` diz *"local-space"* e
/// o do `QueryPipeline::cast_shape` diz *"in world space"*. Publicar um ponto
/// local como se fosse de mundo daria coordenadas plausíveis e erradas — então a
/// pergunta é feita à geometria: uma parede cuja face está em `x = 2,0` tem de
/// devolver um ponto ali, e uma normal a apontar de volta para quem varreu.
#[test]
fn the_witness_point_and_the_normal_come_out_in_world_space() {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(3.0, 0.0, 1.0, 5.0); // face esquerda em x = 2,0
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2);
    w.step();

    let hit = w
        .sweep_body(body, [1.0, 0.0], 5.0, 0)
        .expect("a varredura tem de achar a parede");
    assert!(
        (hit.point[0] - 2.0).abs() < 1.0e-3,
        "o ponto de impacto esta' na FACE da parede (x = 2,0): {:?}",
        hit.point
    );
    assert!(
        hit.normal[0] < -0.9,
        "a normal da parede aponta de volta para quem varreu: {:?}",
        hit.normal
    );
    // E a distância é o vão até a face, menos o raio da cápsula.
    assert!(
        (hit.distance - 1.8).abs() < 1.0e-3,
        "distancia ate' encostar: {}",
        hit.distance
    );
}

/// **UM CORPO PODE TER VÁRIAS FORMAS**, e a varredura vê a que encosta primeiro.
///
/// ⚠️ Este gate existe porque a frase *"um corpo tem exactamente um collider"*
/// já morreu quatro vezes nesta linha (`W-PartFace`). A peça de cima está mais
/// perto do teto: uma varredura que só olhasse a primeira forma mediria a
/// distância errada, e nada mais no produto acusaria.
#[test]
fn a_compound_body_sweeps_every_shape_it_has() {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 4.0, 8.0, 1.0); // face de baixo em y = 3,0
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2); // topo em 0,5
    // Uma segunda peça, mais alta: topo em 1,5.
    let arm = ColliderBuilder::cuboid(0.1, 0.5)
        .translation(Vector2::new(0.0, 1.0))
        .density(1.0)
        .build();
    let ch = w.colliders.insert_with_parent(arm, body, &mut w.bodies);
    w.stamp_layer(ch, 0);
    w.step();

    let hit = w
        .sweep_body(body, [0.0, 1.0], 5.0, 0)
        .expect("a varredura tem de ver o teto");
    assert!(
        (hit.distance - 1.5).abs() < 1.0e-2,
        "a peca MAIS ALTA e' que decide: {} (esperado ~1,5)",
        hit.distance
    );
}

/// **O corpo não encontra a si mesmo.** Sem a exclusão, a forma nasce em cima do
/// próprio collider e toda varredura devolveria impacto em zero — num raio a
/// exclusão é higiene, aqui é aritmética.
#[test]
fn the_sweeper_never_finds_itself() {
    let mut w = PhysicsWorld::new();
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2);
    w.step();
    assert!(
        w.sweep_body(body, [0.0, 1.0], 5.0, 0).is_none(),
        "num mundo com um corpo so' nao ha' nada a encontrar"
    );
}

/// **As CAMADAS valem** — a metade que impede o sensor e o solver de discordarem
/// sobre o que é sólido. Espelho exacto do gate irmão do raio.
#[test]
fn a_layer_the_sweeper_does_not_collide_with_is_invisible_to_it() {
    let wall = |w: &mut PhysicsWorld, layer: usize| {
        let b = w.bodies.insert(
            RigidBodyBuilder::new(RigidBodyType::Fixed)
                .translation(Vector2::new(3.0, 0.0))
                .build(),
        );
        let c = ColliderBuilder::cuboid(1.0, 5.0).build();
        let ch = w.colliders.insert_with_parent(c, b, &mut w.bodies);
        w.stamp_layer(ch, layer);
    };

    // Controle: mesma camada, a varredura a vê.
    let mut same = PhysicsWorld::new();
    wall(&mut same, 0);
    let a = capsule_body(&mut same, 0.0, 0.0, 0.3, 0.2);
    same.step();
    assert!(
        same.sweep_body(a, [1.0, 0.0], 5.0, 0).is_some(),
        "controle: na mesma camada a parede e' visivel"
    );

    // Experimento: a parede na camada 1, que não colide com a 0.
    let mut split = PhysicsWorld::new();
    let mut m = LayerMatrix::default();
    m.set(0, 1, false);
    split.set_layer_matrix(m);
    wall(&mut split, 1);
    let b = capsule_body(&mut split, 0.0, 0.0, 0.3, 0.2);
    split.step();
    assert!(
        split.sweep_body(b, [1.0, 0.0], 5.0, 0).is_none(),
        "uma camada que nao colide comigo e' invisivel a' minha varredura"
    );
}

/// **UM SENSOR NÃO É MATÉRIA**, e esta porta pergunta por matéria — a mesma
/// frase que o `cast_ray` escreve e que o `buoyancy` escreve do outro lado.
///
/// Sem isto, um personagem recusaria levantar-se dentro de um volume de gatilho.
#[test]
fn a_sensor_is_not_something_to_bump_into() {
    let mut w = PhysicsWorld::new();
    let zone = w.bodies.insert(
        RigidBodyBuilder::new(RigidBodyType::Fixed)
            .translation(Vector2::new(0.0, 3.0))
            .build(),
    );
    let c = ColliderBuilder::cuboid(8.0, 1.0).sensor(true).build();
    let ch = w.colliders.insert_with_parent(c, zone, &mut w.bodies);
    w.stamp_layer(ch, 0);
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2);
    w.step();

    assert!(
        w.sweep_body(body, [0.0, 1.0], 5.0, 0).is_none(),
        "um sensor nao e' um teto"
    );
}

/// **O alcance é em METROS, seja qual for o comprimento da direção.**
///
/// ⚠️ Sem a normalização desta porta, `dir = [0, 10]` com `max_dist = 1` varreria
/// 10 m — o chamador pediria 1 e receberia dez vezes isso, em silêncio. É a
/// mesma armadilha que o raio já documenta, e aqui o `max_time_of_impact` do
/// parry a torna literal.
#[test]
fn the_reach_is_in_metres_whatever_the_direction_length() {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 3.0, 8.0, 1.0); // face de baixo em 2,0; a cabeça em 0,5
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2);
    w.step();
    assert!(
        w.sweep_body(body, [0.0, 10.0], 1.0, 0).is_none(),
        "um dir longo nao pode esticar o alcance"
    );
    assert!(
        w.sweep_body(body, [0.0, 10.0], 2.0, 0).is_some(),
        "com alcance suficiente ele acha"
    );
}

/// **Entrada degenerada devolve `None`**, e um corpo que não existe também.
#[test]
fn degenerate_input_yields_nothing() {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 3.0, 8.0, 1.0);
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2);
    w.step();
    assert!(w.sweep_body(body, [0.0, 0.0], 5.0, 0).is_none());
    assert!(w.sweep_body(body, [f32::NAN, 1.0], 5.0, 0).is_none());
    assert!(w.sweep_body(body, [0.0, 1.0], f32::NAN, 0).is_none());
    assert!(w.sweep_body(body, [0.0, 1.0], -5.0, 0).is_none());
}

/// **Começar dentro de alguma coisa dá `distance == 0` com normal viva** — o
/// contrato que o [`CastHit`] publica, e a diferença honesta em relação ao raio.
#[test]
fn starting_inside_reports_zero_with_a_live_normal() {
    let mut w = PhysicsWorld::new();
    // Uma laje que envolve o corpo.
    w.add_static_cuboid(0.0, 0.0, 2.0, 2.0);
    let body = capsule_body(&mut w, 0.0, 0.0, 0.3, 0.2);
    w.step();
    let hit = w
        .sweep_body(body, [0.0, 1.0], 1.0, 0)
        .expect("penetracao e' REPORTADA, nao atravessada");
    assert!(hit.distance.abs() < 1.0e-6, "toi = {}", hit.distance);
    let n2 = hit.normal[0] * hit.normal[0] + hit.normal[1] * hit.normal[1];
    assert!(
        n2 > 0.5,
        "a normal continua util em penetracao: {:?}",
        hit.normal
    );
}
