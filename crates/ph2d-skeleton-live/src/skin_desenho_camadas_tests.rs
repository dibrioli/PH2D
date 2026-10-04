//! A6 — numa dobra forte SEM união, o traço dos contornos FECHADOS de trás não pinta por cima da
//! frente. ⚠️ A régua é a do A2 (`frente::tests`), sem a chave da lei: a frente é o membro da ponta
//! (`x > 24`), a trás o da raiz (`x < 16`); tinta = um troço do TRAÇO desenhado a `< 0,1` e PARALELO.

use crate::skin_desenho::Leis;
use crate::skin_desenho::frente::tests::{amostras, cubica, dentro, dist_pol, em};
use crate::skin_live::tests::palco;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

const JUNTA: f64 = 20.0;
const M: f64 = 4.0;
const PERTO: f64 = 0.1;
const FOLGA: f64 = 0.25;

/// O que pinta TRAÇO: a camada do traço quando a há, senão a forma inteira (com traço).
fn tinta(d: &crate::skin_desenho::Desenhado) -> Vec<[[f64; 2]; 2]> {
    let p = d.traco.as_ref().unwrap_or(&d.forma);
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else { continue };
        let mut w = vs.to_vec();
        if fechado && !vs.is_empty() {
            w.push(vs[0]);
        }
        for k in 0..w.len().saturating_sub(1) {
            let cb = cubica(&w, k);
            let pts: Vec<[f64; 2]> = (0..=64).map(|i| em(&cb, f64::from(i) / 64.0).0).collect();
            out.extend(pts.windows(2).map(|q| [q[0], q[1]]));
        }
    }
    out
}

/// `(tapadas pintadas, frente pintada, trás à vista pintada, tapadas, há camada?)` — duas cópias
/// da barra sobrepostas `40 %`, a 2.ª girada `5°` (os contornos CRUZAM-se em repouso ⇒ a união não
/// corre; ⚠️ sem o giro os lados ficam colineares, a pergunta `overlaps_itself` diz «não» e a união
/// funde as cópias — MEDIDO), a ponta a `graus`.
fn mede(graus: f32, frente: bool) -> (f64, f64, f64, usize, bool) {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    {
        let p = scene.path_mut(id).expect("path");
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            0.5,
        ));
        p.effects = vec![FxEntry::new(PathEffect::Repeat(
            ph2d_vec_scene::fx_repeat::RepeatSpec {
                copies_x: 1.0,
                move_x: 0.0,
                copies_y: 2.0,
                move_y: 60.0,
                spin: 5.0,
                orbit: 0.0,
            },
        ))];
    }
    assert_eq!(crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None), 1);
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    assert_eq!(g.path.contour_count(), 2, "o CONTROLO: duas cópias");
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
        frente,
        ..Leis::do_ambiente()
    };
    let d = crate::skin_live::recook_leis(&sim, &mut scene.clone(), leis)
        .remove(&id)
        .expect("desenho");
    let troco = tinta(&d);
    let pintada = |(p, t): ([f64; 2], [f64; 2])| {
        let q = posa(p);
        let q2 = posa([p[0] + 1e-4 * t[0], p[1] + 1e-4 * t[1]]);
        let dir = [q2[0] - q[0], q2[1] - q[1]];
        let nd = dir[0].hypot(dir[1]).max(1e-18);
        troco.iter().any(|&[a, b]| {
            let u = [b[0] - a[0], b[1] - a[1]];
            let nu = u[0].hypot(u[1]).max(1e-18);
            dist_pol(&[a, b], q) < PERTO
                && ((dir[0] * u[0] + dir[1] * u[1]) / (nd * nu)).abs() > 8f64.to_radians().cos()
        })
    };
    // ⚠️ A região posada é uma NUVEM (cada ponto do interior em repouso, posado), não o polígono do
    // contorno posado: na dobra esse polígono cruza-se a si mesmo e o par-ímpar erra (MEDIDO: `6,7 %`
    // da trás «à vista» estava debaixo da borda da frente).
    let aneis: Vec<Vec<[f64; 2]>> = (0..g.path.contour_count())
        .filter_map(|c| g.path.contour(c))
        .filter(|(_, f)| *f)
        .map(|(v, _)| {
            let mut w = v.to_vec();
            w.push(v[0]);
            (0..w.len() - 1)
                .flat_map(|k| {
                    let cb = cubica(&w, k);
                    (0..16).map(move |i| em(&cb, f64::from(i) / 16.0).0)
                })
                .collect()
        })
        .collect();
    let na_forma = |p: [f64; 2]| aneis.iter().any(|a| dentro(a, p));
    // `funda`: só o INTERIOR da arte (a `0,3` da borda) — a borda da nuvem passava da arte e lia como
    // «tapado» o traço logo ao lado dela (MEDIDO: `2,9 %` a `130°`).
    let nuvem = |x0: f64, funda: bool| -> Vec<[f64; 2]> {
        (0..=200)
            .flat_map(|i| (-40..=120).map(move |j| [f64::from(i) * 0.2, f64::from(j) * 0.2]))
            .filter(|p| p[0] > x0 && na_forma(*p))
            .filter(|p| {
                !funda
                    || [[0.3, 0.0], [-0.3, 0.0], [0.0, 0.3], [0.0, -0.3]]
                        .iter()
                        .all(|d| na_forma([p[0] + d[0], p[1] + d[1]]))
            })
            .map(&posa)
            .collect()
    };
    let (frente_n, adiante_n) = (nuvem(JUNTA + M, true), nuvem(JUNTA - M, false));
    let coberto =
        |n: &[[f64; 2]], q: [f64; 2], r: f64| n.iter().any(|p| (p[0] - q[0]).hypot(p[1] - q[1]) < r);
    let (mut tap, mut tap_p, mut fr, mut fr_p, mut vis, mut vis_p) = (0, 0, 0, 0, 0, 0);
    for s in amostras(&g.path, true, 200) {
        let x = s.0[0];
        if s.1[0].hypot(s.1[1]) < 1e-9 {
            continue;
        }
        // ⚠️ A frente julgada começa a `2M` da junta: entre `M` e `2M` a zona de mistura dobra-se
        // sobre si mesma e a ordem ali é a da lei (MEDIDO: `0,3 %` em `x ≈ 24`).
        if x > JUNTA + 2.0 * M {
            fr += 1;
            fr_p += usize::from(pintada(s));
        } else if x > JUNTA + M {
        } else if x < JUNTA - M {
            let q = posa(s.0);
            if coberto(&frente_n, q, 0.15) {
                tap += 1;
                tap_p += usize::from(pintada(s));
            } else if x < JUNTA - 2.0 * M && !coberto(&adiante_n, q, FOLGA + 0.2) {
                vis += 1;
                vis_p += usize::from(pintada(s));
            }
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "contagens")]
    let r = |a: usize, b: usize| a as f64 / b.max(1) as f64;
    (r(tap_p, tap), r(fr_p, fr), r(vis_p, vis), tap, d.traco.is_some())
}

/// ⭐⭐⭐ **GATE — sem união, o traço dos FECHADOS de trás não pinta por cima da frente, e nada mais
/// se apaga** (a `110°`, `130°`, `150°`). ⛔ **O CONTROLO:** sem a lei da frente o mesmo traço pinta-se
/// (FOTOGRAFADO em SVG: duas cópias a `110°`, contornos a riscar a frente nas duas juntas).
#[test]
fn o_traco_dos_fechados_de_tras_nao_pinta_por_cima_da_frente() {
    for graus in [110.0, 130.0, 150.0] {
        let (tap, fr, vis, n, camada) = mede(graus, true);
        let (tap0, fr0, vis0, _, camada0) = mede(graus, false);
        println!(
            "  {graus}°: tapadas {n} pintadas {tap:.3} (sem a lei {tap0:.3}) · frente {fr:.3} \
             ({fr0:.3}) · trás à vista {vis:.3} ({vis0:.3}) · camada {camada} ({camada0})"
        );
        assert!(n > 20, "a {graus}° a fixtura não tem traço tapado ({n})");
        assert!(tap0 > 0.9 && !camada0, "o CONTROLO: sem a lei o traço tapado não se pinta ({tap0:.3})");
        assert!(camada, "a {graus}° não saiu camada de traço");
        assert!(tap < 0.02, "a {graus}° o traço de trás pinta a frente ({tap:.3})");
        assert!(fr > 0.95 && fr >= fr0 - 1e-3, "a {graus}° a frente perdeu traço ({fr:.3} × {fr0:.3})");
        assert!(vis > 0.95 && vis >= vis0 - 1e-3, "a {graus}° a trás perdeu traço ({vis:.3} × {vis0:.3})");
    }
}

/// ⭐⭐ **GATE — sem dobra que tape, NÃO há camada** (a forma sai como antes, ao bit) — a `0°` e a
/// `40°`. ⛔ **O CONTROLO** é o gate de cima: a `110°` a camada existe.
#[test]
fn sem_dobra_que_tape_nao_ha_camada() {
    for graus in [0.0, 40.0] {
        let (.., camada) = mede(graus, true);
        assert!(!camada, "a {graus}° saiu camada de traço");
    }
}
