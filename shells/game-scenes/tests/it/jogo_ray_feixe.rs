//! ⭐ **O feixe do Ray Sensor no canvas** (GEF 07 RS-1/RS-2) — o pintor de `ph2d-app-physics` (`probe_marks`)
//! julgado pela geometria em px de tela. Mora aqui porque a `ph2d-app-physics` está no tecto da catraca.

use ph2d_app_physics::overlay::probes::probe_marks;
use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{ProbeKind, ProbeMark};
use ph2d_render::Camera2d;

/// A câmera do harness do pintor: 1000 px / 10 m = 100 px/m.
const JANELA: WindowSize = WindowSize {
    width: 1000,
    height: 1000,
};

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

fn ray(kind: ProbeKind, hit: Option<f32>) -> ProbeMark {
    ProbeMark::ray(kind, [0.0, 0.0], [1.0, 0.0], 2.0, hit, 0.0)
}

/// Os troços do caminho (um por `move_to`), em px de tela.
fn trocos(path: &ph2d_vector::BezPath) -> Vec<Vec<(f64, f64)>> {
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
    for el in path.elements() {
        match *el {
            ph2d_vector::PathEl::MoveTo(q) => out.push(vec![(q.x, q.y)]),
            ph2d_vector::PathEl::LineTo(q) => out.last_mut().expect("troço").push((q.x, q.y)),
            _ => {}
        }
    }
    out
}

/// O ponto de mundo em px de tela, pela câmera do harness (100 px/m).
fn tela(x: f32, y: f32) -> (f64, f64) {
    let (sx, sy) = camera().world_to_screen([x, y], JANELA);
    (f64::from(sx), f64::from(sy))
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// ⭐ **O leque desenha a FATIA e o raio grosso a CÁPSULA** (GEF 07 RS-1/RS-2): o leque sai da origem, vai
/// pelo arco no alcance (de `rumo − meio_cone` a `rumo + meio_cone`) e volta, com um tique centrado em cada
/// ponto visto; a cápsula fica a `largura / 2` do eixo em todo o contorno, e o círculo do acerto também.
///
/// **Mutações que sangram:** o arco do leque no raio `largura` (ou a meio do alcance) · tirar os tiques de
/// `f.pontos()` · a cápsula deslocada por `largura` em vez de `largura / 2`.
#[test]
fn o_leque_e_a_fatia_e_o_raio_grosso_e_a_capsula() {
    let sim = SimWorld::new();
    let meio = std::f32::consts::FRAC_PI_6;
    let mut leque = ray(ProbeKind::Sensor, None);
    leque.feixe.meio_cone = meio;
    leque.feixe.pontos[0] = [1.0, 0.2];
    leque.feixe.pontos[1] = [1.5, -0.3];
    leque.feixe.n = 2;
    let mut grosso = ray(ProbeKind::Sensor, Some(1.0));
    grosso.feixe.largura = 0.4;
    let d = probe_marks(true, &[leque, grosso], &sim, &camera(), JANELA);
    let (o, px_alcance) = (tela(0.0, 0.0), 200.0);
    // O leque: a fatia (origem → arco → origem) e os dois tiques.
    let t = trocos(&d[0].0);
    assert_eq!(t.len(), 3, "a fatia e um tique por ponto visto: {t:?}");
    let fatia = &t[0];
    assert_eq!(
        (fatia[0], *fatia.last().expect("fim")),
        (o, o),
        "sai e volta à origem"
    );
    let arco = &fatia[1..fatia.len() - 1];
    assert!(
        arco.iter().all(|&q| (dist(q, o) - px_alcance).abs() < 0.5),
        "o arco no alcance"
    );
    let ponta = |a: f32| tela(2.0 * a.cos(), 2.0 * a.sin());
    assert!(dist(arco[0], ponta(-meio)) < 0.5 && dist(arco[arco.len() - 1], ponta(meio)) < 0.5);
    for (tique, q) in t[1..].iter().zip([[1.0, 0.2], [1.5, -0.3]]) {
        let mid = (
            (tique[0].0 + tique[1].0) * 0.5,
            (tique[0].1 + tique[1].1) * 0.5,
        );
        assert!(
            dist(mid, tela(q[0], q[1])) < 0.5,
            "o tique senta no ponto visto"
        );
    }
    // O raio grosso: o contorno a 20 px (0,2 m) do eixo, o eixo, e o círculo do acerto.
    let t = trocos(&d[1].0);
    assert_eq!(t.len(), 3, "a cápsula, o eixo e o círculo do acerto");
    let ao_eixo = |q: (f64, f64)| {
        let x = q.0.clamp(o.0, o.0 + px_alcance);
        dist(q, (x, o.1))
    };
    assert!(
        t[0].iter().all(|&q| (ao_eixo(q) - 20.0).abs() < 0.5),
        "{:?}",
        t[0]
    );
    assert_eq!(
        (t[0][0], t[0][1]),
        (tela(0.0, 0.2), tela(2.0, 0.2)),
        "um lado"
    );
    assert!(
        t[0].contains(&tela(2.0, -0.2)) && t[0].contains(&tela(0.0, -0.2)),
        "o outro"
    );
    assert_eq!(t[1], vec![o, tela(2.0, 0.0)], "o eixo");
    let c = tela(1.0, 0.0);
    assert!(t[2].len() > 8 && t[2].iter().all(|&q| (dist(q, c) - 20.0).abs() < 0.5));
}
