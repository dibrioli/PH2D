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

/// O que pinta TRAÇO: a camada do traço e a forma quando ELA tem traço (as duas, se as duas o
/// tiverem — um traço que ficasse na forma pintaria por cima do recorte).
fn tinta(d: &crate::skin_desenho::Desenhado) -> Vec<[[f64; 2]; 2]> {
    let mut out = Vec::new();
    for p in d
        .traco
        .iter()
        .chain(Some(&d.forma).filter(|f| f.stroke.is_some()))
    {
        tinta_de(p, &mut out);
    }
    out
}

fn tinta_de(p: &ph2d_vec_scene::VecPath, out: &mut Vec<[[f64; 2]; 2]>) {
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else {
            continue;
        };
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
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
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
    let coberto = |n: &[[f64; 2]], q: [f64; 2], r: f64| {
        n.iter().any(|p| (p[0] - q[0]).hypot(p[1] - q[1]) < r)
    };
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
    (
        r(tap_p, tap),
        r(fr_p, fr),
        r(vis_p, vis),
        tap,
        d.traco.is_some(),
    )
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
        assert!(
            tap0 > 0.9 && !camada0,
            "o CONTROLO: sem a lei o traço tapado não se pinta ({tap0:.3})"
        );
        assert!(camada, "a {graus}° não saiu camada de traço");
        assert!(
            tap < 0.02,
            "a {graus}° o traço de trás pinta a frente ({tap:.3})"
        );
        assert!(
            fr > 0.95 && fr >= fr0 - 1e-3,
            "a {graus}° a frente perdeu traço ({fr:.3} × {fr0:.3})"
        );
        assert!(
            vis > 0.95 && vis >= vis0 - 1e-3,
            "a {graus}° a trás perdeu traço ({vis:.3} × {vis0:.3})"
        );
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

/// ⭐ **SONDA — o preço da camada do traço** (duas cópias, pose a mudar `110°` ↔ `150°`), com e sem a
/// lei da frente. Imprime; corra em `--release` com o `loadavg` ao lado.
#[test]
#[ignore = "sonda de preço: --release, máquina calma"]
fn diag_o_preco_da_camada_do_traco() {
    use std::time::Instant;
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
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    let mut n = 0_u32;
    for (pa, pb) in [(110f32, 150f32), (0.0, 30.0)] {
        let mut quadro = |frente: bool| {
            let leis = Leis {
                frente,
                ..Leis::do_ambiente()
            };
            let mut sc = scene.clone();
            let t = Instant::now();
            let mut k = 0_u32;
            while t.elapsed().as_millis() < 300 {
                n += 1;
                sim.world_mut()
                    .get_mut::<ph2d_ecs::Transform>(ponta)
                    .expect("Transform")
                    .rotation = if n.is_multiple_of(2) { pa } else { pb }.to_radians();
                let _ = crate::skin_live::recook_leis(&sim, &mut sc, leis);
                k += 1;
            }
            t.elapsed().as_secs_f64() * 1e6 / f64::from(k)
        };
        let (com, sem) = (quadro(true), quadro(false));
        println!(
            "  {pa}°↔{pb}°: µs/forma/quadro: com a lei {com:.1} · sem ela {sem:.1} · loadavg {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}

/// ⭐⭐ **GATE — as riscas ABERTAS vão para a camada do traço** (a forma vai sem traço: sem elas na
/// camada, sumiam). Duas cópias com *Hatch* por cima, a `110°`. ⛔ **O CONTROLO:** há riscas e há
/// camada.
#[test]
fn as_riscas_abertas_vao_para_a_camada_do_traco() {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    {
        let p = scene.path_mut(id).expect("path");
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            0.5,
        ));
        p.effects = vec![
            FxEntry::new(PathEffect::Repeat(ph2d_vec_scene::fx_repeat::RepeatSpec {
                copies_x: 1.0,
                move_x: 0.0,
                copies_y: 2.0,
                move_y: 60.0,
                spin: 5.0,
                orbit: 0.0,
            })),
            FxEntry::new(PathEffect::Hatch(ph2d_vec_scene::fx_hatch::HatchSpec {
                angle: 45.0,
                spacing: 8.0,
                cross: false,
            })),
        ];
    }
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = 110f32.to_radians();
    let d = crate::skin_live::recook_leis(&sim, &mut scene.clone(), Leis::do_ambiente())
        .remove(&id)
        .expect("desenho");
    let abertos = |p: &ph2d_vec_scene::VecPath| {
        (0..p.contour_count())
            .filter_map(|c| p.contour(c))
            .filter(|(v, f)| !f && v.len() > 1)
            .count()
    };
    let traco = d
        .traco
        .as_ref()
        .expect("o CONTROLO: a 110° não saiu camada");
    assert!(abertos(&d.forma) > 0, "o CONTROLO: a forma não tem riscas");
    assert!(
        d.forma.stroke.is_none(),
        "a forma levou o traço com a camada"
    );
    assert!(
        abertos(traco) >= abertos(&d.forma),
        "as riscas não foram para a camada ({} de {})",
        abertos(traco),
        abertos(&d.forma)
    );
}

/// ⭐⭐ **GATE — a camada do traço chega à geometria viva, DEPOIS da forma** (o traço por cima do
/// preenchimento). ⛔ **O CONTROLO:** sem camada, um caminho só.
#[test]
fn a_camada_do_traco_chega_ao_mundo_depois_da_forma() {
    let mut forma = ph2d_vec_scene::VecPath::default();
    forma
        .verts
        .push(ph2d_vec_scene::VecVertex::corner([1.0, 0.0]));
    let mut traco = ph2d_vec_scene::VecPath::default();
    traco
        .verts
        .push(ph2d_vec_scene::VecVertex::corner([2.0, 0.0]));
    let xf = ph2d_vec_scene::VecXforms::default();
    for (camada, esperado) in [(None, 1), (Some(traco.clone()), 2)] {
        let mut d = crate::skin_desenho::SkinDesenhado::new();
        d.insert(
            7,
            crate::skin_desenho::Desenhado {
                forma: forma.clone(),
                traco: camada,
            },
        );
        let mut vivo = std::collections::BTreeMap::new();
        crate::skin_desenho::funde(&d, &xf, &mut vivo);
        let v = &vivo[&7];
        assert_eq!(v.len(), esperado, "caminhos no mundo");
        assert_eq!(v[0].verts[0].anchor, [1.0, 0.0], "a forma vem primeiro");
    }
}

/// ⭐⭐⭐ **GATE — o TRAÇO fica sobre a BORDA do preenchimento**, numa grelha de dobras fortes (até
/// `170°`) — report do dono de 2026-10-04 (foto: o contorno descolado do preenchimento numa dobra
/// agressiva; assados à parte os dois afastavam-se até `0,28`, MEDIDO). ⛔ **O CONTROLO:** a grelha
/// tem camada em quase todas as poses.
#[test]
fn o_traco_fica_sobre_a_borda_do_preenchimento() {
    use crate::barra_da_cena_tests_support::osso;
    let (mut com_camada, mut pior) = (0, 0.0_f64);
    for a in [110f32, 140.0, 170.0] {
        for b in [-110f32, 110.0, 150.0, 180.0] {
            let mut sim = ph2d_ecs::SimWorld::default();
            let mut scene = ph2d_vec_scene::VecScene::new();
            let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
            let mut barra = ph2d_vec_scene::cook(
                ph2d_vec_scene::ShapeKind::RoundRect,
                [-2.25, -0.375],
                [2.25, 0.375],
                &[0.375],
            );
            barra.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
                ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
                0.045,
            ));
            barra.effects = vec![FxEntry::new(PathEffect::Repeat(
                ph2d_vec_scene::fx_repeat::RepeatSpec {
                    copies_x: 1.0,
                    move_x: 0.0,
                    copies_y: 2.0,
                    move_y: 60.0,
                    spin: 5.0,
                    orbit: 0.0,
                },
            ))];
            let id = scene.push_path(barra);
            ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
            let passo = (4.5 - 0.75) / 3.0;
            #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
            let p32 = passo as f32;
            let b1 = osso(&mut sim, "b1", [-1.875, 0.0], passo, None);
            let b2 = osso(&mut sim, "b2", [p32, 0.0], passo, Some(b1));
            let b3 = osso(&mut sim, "b3", [p32, 0.0], passo, Some(b2));
            assert_eq!(
                crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], Some(b1)),
                1
            );
            for (o, g) in [(b2, a), (b3, b)] {
                sim.world_mut()
                    .get_mut::<ph2d_ecs::Transform>(o)
                    .expect("Transform")
                    .rotation = g.to_radians();
            }
            let d = crate::skin_live::recook_leis(&sim, &mut scene.clone(), Leis::do_ambiente())
                .remove(&id)
                .expect("desenho");
            let Some(t) = d.traco.as_ref() else { continue };
            com_camada += 1;
            let mut borda = Vec::new();
            for c in 0..d.forma.contour_count() {
                let Some((v, true)) = d.forma.contour(c) else {
                    continue;
                };
                let mut w = v.to_vec();
                w.push(v[0]);
                for k in 0..w.len() - 1 {
                    let cb = cubica(&w, k);
                    borda.extend((0..64).map(|i| em(&cb, f64::from(i) / 64.0).0));
                }
            }
            let troco: Vec<[[f64; 2]; 2]> = borda.windows(2).map(|q| [q[0], q[1]]).collect();
            for c in 0..t.contour_count() {
                let Some((v, false)) = t.contour(c) else {
                    continue;
                };
                for k in 0..v.len().saturating_sub(1) {
                    let cb = cubica(v, k);
                    for i in 0..=16 {
                        let q = em(&cb, f64::from(i) / 16.0).0;
                        let dmin = troco
                            .iter()
                            .map(|s| dist_pol(s, q))
                            .fold(f64::MAX, f64::min);
                        pior = pior.max(dmin);
                    }
                }
            }
        }
    }
    println!("  {com_camada} poses com camada · o traço longe da borda até {pior:.2e}");
    assert!(
        com_camada >= 8,
        "o CONTROLO: só {com_camada} poses de 12 com camada"
    );
    assert!(
        pior < 1e-3,
        "o traço descolou da borda do preenchimento ({pior})"
    );
}
