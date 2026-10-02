//! Os gates da ROTA da patrulha (plano 30, W6): os pontos são os da forma desenhada, em mundo, e a
//! curva parte-se à régua da `arrive_distance` do agente. A fixtura é a meia circunferência de raio
//! `2` do seguidor de caminho (respostas FECHADAS: todo ponto dela está a `2` do centro).

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Transform, VecPathRef};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

const K: f64 = 0.552_284_749_830_793_4 * 2.0;

fn v(anchor: [f64; 2], inh: [f64; 2], outh: [f64; 2], kind: VertexKind) -> VecVertex {
    VecVertex {
        anchor,
        in_handle: inh,
        out_handle: outh,
        kind,
        corner_radius: 0.0,
    }
}

fn arco() -> Vec<VecVertex> {
    vec![
        v([-2.0, 0.0], [-2.0, 0.0], [-2.0, K], VertexKind::Smooth),
        v([0.0, 2.0], [-K, 2.0], [K, 2.0], VertexKind::Smooth),
        v([2.0, 0.0], [2.0, K], [2.0, 0.0], VertexKind::Smooth),
    ]
}

fn quadrado() -> Vec<VecVertex> {
    [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]
        .iter()
        .map(|&p| v(p, p, p, VertexKind::Corner))
        .collect()
}

/// Uma forma `Ronda` em `(10, 0)` e um guarda em patrulha por ela.
fn cena(verts: Vec<VecVertex>, closed: bool, chegada: f32) -> (SimWorld, VecScene, Entity) {
    let mut sim = SimWorld::new();
    let mut c = VecScene::new();
    let id = c.push_path(VecPath {
        verts,
        closed,
        ..VecPath::default()
    });
    sim.world_mut().spawn((
        Name::new("Ronda"),
        Transform::from_translation(Vec2::new(10.0, 0.0)),
        VecPathRef(id),
    ));
    let guarda = sim
        .world_mut()
        .spawn((
            Name::new("Guarda"),
            NavAgent {
                target: NavTarget::Patrol(stable_name_id("Ronda")),
                arrive_distance: chegada,
                ..NavAgent::default()
            },
        ))
        .id();
    (sim, c, guarda)
}

fn rota(sim: &SimWorld, e: Entity) -> Option<NavRoute> {
    sim.world().get::<NavRoute>(e).cloned()
}

#[test]
fn um_poligono_fechado_da_os_seus_cantos_em_mundo() {
    let (mut sim, c, g) = cena(quadrado(), true, 0.1);
    escreve(&mut sim, &c);
    let r = rota(&sim, g).expect("a rota");
    assert!(r.closed);
    assert_eq!(
        r.points,
        vec![[10.0, 0.0], [14.0, 0.0], [14.0, 4.0], [10.0, 4.0]],
        "os cantos, levados pela pose da forma"
    );
}

#[test]
fn a_curva_parte_se_a_regua_da_chegada() {
    let (mut sim, c, g) = cena(arco(), false, 0.05);
    escreve(&mut sim, &c);
    let fina = rota(&sim, g).expect("a rota");
    // Todo ponto está NA curva (a 2 do centro (10, 0)), e o primeiro e o último são as pontas.
    for p in &fina.points {
        let d = ((p[0] - 10.0).powi(2) + p[1].powi(2)).sqrt();
        assert!((d - 2.0).abs() < 1e-3, "um ponto fora da curva: {p:?}");
    }
    assert_eq!(fina.points.first(), Some(&[8.0, 0.0]));
    assert_eq!(fina.points.last(), Some(&[12.0, 0.0]));
    // A flecha de cada corda cabe na régua: corda de comprimento c num círculo de raio 2 tem flecha
    // 2 − √(4 − c²/4).
    for w in fina.points.windows(2) {
        let c2 = (w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2);
        let flecha = 2.0f32 - (4.0f32 - c2 / 4.0).sqrt();
        assert!(
            flecha <= 0.05 + 1e-4,
            "uma corda afasta-se {flecha} da curva"
        );
    }
    // O CONTROLO: com uma régua mais grossa saem MENOS pontos, e com zero só as âncoras.
    let (mut sim2, c2, g2) = cena(arco(), false, 0.5);
    escreve(&mut sim2, &c2);
    let grossa = rota(&sim2, g2).expect("a rota");
    assert!(grossa.points.len() < fina.points.len());
    let (mut sim3, c3, g3) = cena(arco(), false, 0.0);
    escreve(&mut sim3, &c3);
    assert_eq!(
        rota(&sim3, g3).expect("a rota").points.len(),
        3,
        "só as três âncoras"
    );
}

#[test]
fn sem_forma_ou_fora_da_patrulha_nao_ha_rota() {
    let (mut sim, c, g) = cena(quadrado(), true, 0.1);
    escreve(&mut sim, &c);
    assert!(rota(&sim, g).is_some());
    sim.world_mut()
        .get_mut::<NavAgent>(g)
        .expect("o agente")
        .target = NavTarget::Point([0.0, 0.0]);
    escreve(&mut sim, &c);
    assert!(rota(&sim, g).is_none(), "saiu da patrulha e a rota ficou");
    sim.world_mut()
        .get_mut::<NavAgent>(g)
        .expect("o agente")
        .target = NavTarget::Patrol(stable_name_id("Ninguém"));
    escreve(&mut sim, &c);
    assert!(rota(&sim, g).is_none(), "uma forma que não existe deu rota");
}
