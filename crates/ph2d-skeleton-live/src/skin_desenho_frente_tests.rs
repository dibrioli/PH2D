//! A2 — a parte da FRENTE tapa as riscas da de TRÁS numa dobra forte.
//!
//! ⚠️ **A régua não usa a chave de osso da lei:** a frente é o MEMBRO da ponta (repouso `x > 20 + M`)
//! posado pela pele, e uma risca de trás é a do membro da raiz (`x < 20 − M`); entre os dois fica a
//! zona da junta, que a régua não julga. Uma amostra conta como TINTA quando o desenho tem um
//! contorno ABERTO a menos de [`PERTO`] dela e PARALELO a ela (uma risca da frente que a cruza não
//! a pinta).

use crate::skin_desenho::Leis;
use crate::skin_live::tests::palco;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::VecPath;

/// A junta da fixtura (`palco`: barra `40 × 10`, dois ossos de `20`).
const JUNTA: f64 = 20.0;
/// A meia-largura da zona da junta que a régua não julga.
const M: f64 = 4.0;
/// Distância a que uma amostra se lê pintada.
const PERTO: f64 = 0.1;
/// Folga da fronteira da frente: perto dela a régua não julga.
const FOLGA: f64 = 0.25;

fn cubica(vs: &[ph2d_vec_scene::VecVertex], k: usize) -> [[f64; 2]; 4] {
    [vs[k].anchor, vs[k].out_handle, vs[k + 1].in_handle, vs[k + 1].anchor]
}

fn em(c: &[[f64; 2]; 4], t: f64) -> ([f64; 2], [f64; 2]) {
    let s = 1.0 - t;
    let p = |k: usize| {
        s * s * s * c[0][k] + 3.0 * s * s * t * c[1][k] + 3.0 * s * t * t * c[2][k] + t * t * t * c[3][k]
    };
    let d = |k: usize| {
        3.0 * s * s * (c[1][k] - c[0][k]) + 6.0 * s * t * (c[2][k] - c[1][k]) + 3.0 * t * t * (c[3][k] - c[2][k])
    };
    ([p(0), p(1)], [d(0), d(1)])
}

/// Amostras `(ponto, tangente)` dos contornos ABERTOS (ou dos fechados, `fechados = true`).
fn amostras(p: &VecPath, fechados: bool, por_seg: usize) -> Vec<([f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else { continue };
        if fechado != fechados || vs.len() < 2 {
            continue;
        }
        for k in 0..vs.len() - 1 {
            let cb = cubica(vs, k);
            for i in 0..=por_seg {
                #[expect(clippy::cast_precision_loss, reason = "amostra")]
                out.push(em(&cb, i as f64 / por_seg as f64));
            }
        }
    }
    out
}

fn dentro(pol: &[[f64; 2]], q: [f64; 2]) -> bool {
    let mut d = false;
    for i in 0..pol.len() {
        let (a, b) = (pol[i], pol[(i + 1) % pol.len()]);
        if (a[1] > q[1]) != (b[1] > q[1]) && q[0] < (b[0] - a[0]) * (q[1] - a[1]) / (b[1] - a[1]) + a[0] {
            d = !d;
        }
    }
    d
}

fn dist_pol(pol: &[[f64; 2]], q: [f64; 2]) -> f64 {
    (0..pol.len())
        .map(|i| {
            let (a, b) = (pol[i], pol[(i + 1) % pol.len()]);
            let ab = [b[0] - a[0], b[1] - a[1]];
            let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18);
            let t = (((q[0] - a[0]) * ab[0] + (q[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0);
            (q[0] - a[0] - t * ab[0]).hypot(q[1] - a[1] - t * ab[1])
        })
        .fold(f64::MAX, f64::min)
}

/// `(tapadas pintadas / tapadas, frente pintada / frente, trás à vista pintada / trás à vista,
/// tapadas)` — o desenho com e sem a lei do contacto.
fn mede(graus: f32, contacto: bool) -> (f64, f64, f64, usize) {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: false,
        },
    ))];
    assert_eq!(crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None), 1);
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let campo = g.campo.clone().expect("campo");
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = graus.to_radians();
    let index = crate::skin_live::bone_index(&sim);
    let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
    let posa = |p: [f64; 2]| {
        let mut w = pele.scratch();
        pele.weights_corrected(p, campo.linha(p).as_deref(), &mut w, &[]);
        pele.blend(p, &w)
    };
    let leis = Leis {
        contacto,
        ..Leis::do_ambiente()
    };
    let d = crate::skin_live::recook_leis(&sim, &mut scene.clone(), leis)
        .remove(&id)
        .expect("desenho");
    let tinta = amostras(&d, false, 64);
    let pintada = |(p, t): ([f64; 2], [f64; 2])| {
        let q = posa(p);
        let q2 = posa([p[0] + 1e-4 * t[0], p[1] + 1e-4 * t[1]]);
        let dir = [q2[0] - q[0], q2[1] - q[1]];
        let nd = dir[0].hypot(dir[1]).max(1e-18);
        tinta.iter().any(|(r, u)| {
            let nu = u[0].hypot(u[1]).max(1e-18);
            (r[0] - q[0]).hypot(r[1] - q[1]) < PERTO
                && ((dir[0] * u[0] + dir[1] * u[1]) / (nd * nu)).abs() > 8f64.to_radians().cos()
        })
    };
    // A frente posada: o contorno do repouso `x > JUNTA + M` pela pele.
    let lado: Vec<[f64; 2]> = (0..=100)
        .map(|i| [JUNTA + M + (40.0 - JUNTA - M) * f64::from(i) / 100.0, 0.0])
        .chain((0..=100).map(|i| [40.0, 10.0 * f64::from(i) / 100.0]))
        .chain((0..=100).map(|i| [40.0 - (40.0 - JUNTA - M) * f64::from(i) / 100.0, 10.0]))
        .chain((0..=100).map(|i| [JUNTA + M, 10.0 - 10.0 * f64::from(i) / 100.0]))
        .collect();
    let frente: Vec<[f64; 2]> = lado.iter().map(|&p| posa(p)).collect();
    let riscas = amostras(&g.path, false, 400);
    let (mut tap, mut tap_p, mut fr, mut fr_p, mut vis, mut vis_p) = (0, 0, 0, 0, 0, 0);
    for s in riscas {
        let x = s.0[0];
        if x > JUNTA + M {
            fr += 1;
            fr_p += usize::from(pintada(s));
        } else if x < JUNTA - M {
            let q = posa(s.0);
            if dist_pol(&frente, q) < FOLGA {
                continue;
            }
            if dentro(&frente, q) {
                tap += 1;
                tap_p += usize::from(pintada(s));
            } else {
                vis += 1;
                vis_p += usize::from(pintada(s));
            }
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "contagens")]
    let r = |a: usize, b: usize| a as f64 / b.max(1) as f64;
    (r(tap_p, tap), r(fr_p, fr), r(vis_p, vis), tap)
}

/// ⭐⭐⭐ **GATE — numa dobra forte as riscas de TRÁS não pintam por cima da FRENTE, e nada mais se
/// apaga.** Varrida a `110°`, `130°`, `150°`.
///
/// ⛔ **O CONTROLO:** sem a lei do contacto as mesmas riscas tapadas pintam-se (o defeito
/// FOTOGRAFADO na `=5` a `110°`).
#[test]
fn as_riscas_de_tras_nao_pintam_por_cima_da_frente() {
    for graus in [110.0, 130.0, 150.0] {
        let (tap, fr, vis, n) = mede(graus, true);
        let (tap0, fr0, vis0, n0) = mede(graus, false);
        println!(
            "  {graus}°: tapadas {n} pintadas {tap:.3} (sem a lei {tap0:.3}, {n0}) · frente {fr:.3} \
             ({fr0:.3}) · trás à vista {vis:.3} ({vis0:.3})"
        );
        assert!(n > 20, "a {graus}° a fixtura não tem riscas tapadas ({n})");
        assert!(tap0 > 0.9, "o CONTROLO: sem a lei as tapadas não se pintam ({tap0:.3})");
        assert!(tap < 0.02, "a {graus}° as riscas de trás pintam a frente ({tap:.3})");
        assert!(fr > 0.98, "a {graus}° a frente perdeu riscas ({fr:.3})");
        assert!(vis > 0.98, "a {graus}° a trás à vista perdeu riscas ({vis:.3})");
    }
}

#[test]
fn sonda_bits() {
    let (sim, _s, _m, _id, [raiz, ponta]) = palco();
    println!("raiz {:#x} ponta {:#x}", raiz.to_bits(), ponta.to_bits());
    let mut sim2 = ph2d_ecs::SimWorld::default();
    let a = crate::skin_live::tests::osso(&mut sim2, "a", [0.0, 0.0], 1.0, None);
    let b = crate::skin_live::tests::osso(&mut sim2, "b", [1.0, 0.0], 1.0, Some(a));
    let c = crate::skin_live::tests::osso(&mut sim2, "c", [1.0, 0.0], 1.0, Some(b));
    println!("a {:#x} b {:#x} c {:#x}", a.to_bits(), b.to_bits(), c.to_bits());
    println!("skeleton_of {:?}", crate::skin_live::skeleton_of(&sim, None).iter().map(|e| e.to_bits()).collect::<Vec<_>>());
}
