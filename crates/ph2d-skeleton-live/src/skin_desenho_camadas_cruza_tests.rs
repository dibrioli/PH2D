//! A9 — a ponta de um corte do traço acerta no cruzamento desenhado.
//!
//! ⚠️ **A régua não usa a malha nem a chave de osso:** só a geometria DESENHADA — a ponta de um
//! trecho do traço que mora sobre o contorno fecha-se a que distância do cruzamento mais perto dos
//! contornos FECHADOS desenhados (a borda onde o traço de trás passa por baixo da frente)? Entre
//! `0,05` e `1` largura é um tique; mais longe é outra coisa (a ponta do vinco, aberto A10).

use super::super::SkinDesenhado;
use crate::skin_desenho::Leis;
use crate::skin_live::tests::palco;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

const LARGURA: f64 = 0.5;

/// Duas cópias da barra `40 × 10` (a 2.ª girada `5°`), traço `0,5`, presas e a ponta a `graus`.
fn desenho(graus: f32) -> SkinDesenhado {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    {
        let p = scene.path_mut(id).expect("path");
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            LARGURA,
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
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = graus.to_radians();
    crate::skin_live::recook_leis(&sim, &mut scene.clone(), Leis::do_ambiente())
}

fn polilinhas(p: &VecPath, so_fechados: bool) -> Vec<Vec<[f64; 2]>> {
    (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .filter(|(v, f)| v.len() > 1 && (*f || !so_fechados))
        .map(|(v, fechado)| {
            let m = if fechado { v.len() } else { v.len() - 1 };
            let mut out: Vec<[f64; 2]> = (0..m)
                .flat_map(|k| {
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                    (0..64)
                        .map(move |i| super::super::super::frente::avalia(&c, f64::from(i) / 64.0))
                })
                .collect();
            out.push(if fechado {
                v[0].anchor
            } else {
                v[v.len() - 1].anchor
            });
            out
        })
        .collect()
}

/// Os cruzamentos entre troços não vizinhos: `(contorno, troço, fracção no troço)`, os dois lados.
fn cruzamentos(pl: &[Vec<[f64; 2]>]) -> Vec<(usize, usize, f64)> {
    let t: Vec<(usize, usize, [f64; 2], [f64; 2])> = pl
        .iter()
        .enumerate()
        .flat_map(|(c, l)| {
            l.windows(2)
                .enumerate()
                .map(move |(i, w)| (c, i, w[0], w[1]))
        })
        .collect();
    let mut out = Vec::new();
    for (x, &(ca, ia, a0, a1)) in t.iter().enumerate() {
        for &(cb, ib, b0, b1) in &t[x + 1..] {
            let n = pl[ca].len() - 1;
            if ca == cb && (ia.abs_diff(ib) <= 1 || ia.abs_diff(ib) >= n - 1) {
                continue;
            }
            let (r, s) = (
                [a1[0] - a0[0], a1[1] - a0[1]],
                [b1[0] - b0[0], b1[1] - b0[1]],
            );
            let den = r[0] * s[1] - r[1] * s[0];
            if den.abs() < 1e-18 {
                continue;
            }
            let q = [b0[0] - a0[0], b0[1] - a0[1]];
            let (u, v) = (
                (q[0] * s[1] - q[1] * s[0]) / den,
                (q[0] * r[1] - q[1] * r[0]) / den,
            );
            if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                out.push((ca, ia, u));
                out.push((cb, ib, v));
            }
        }
    }
    out
}

/// A distância AO LONGO do contorno fechado (em larguras) de cada ponta de trecho do traço sobre ele
/// ao cruzamento mais perto, até `3` larguras (além disso, `3`). ⚠️ Ao longo e não em linha recta:
/// numa dobra em grampo o cruzamento do OUTRO braço fica perto pelo ar (lia `0,14` uma ponta de vinco).
fn pontas(d: &SkinDesenhado) -> Vec<f64> {
    let mut out = Vec::new();
    for x in d.values() {
        let Some(t) = &x.traco else { continue };
        let contorno = polilinhas(&x.forma, true);
        let cruz = cruzamentos(&contorno);
        let perto = |p: [f64; 2]| -> (usize, usize, f64, f64) {
            let mut m = (0, 0, 0.0, f64::MAX);
            for (c, l) in contorno.iter().enumerate() {
                for (i, s) in l.windows(2).enumerate() {
                    let ab = [s[1][0] - s[0][0], s[1][1] - s[0][1]];
                    let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18);
                    let f = (((p[0] - s[0][0]) * ab[0] + (p[1] - s[0][1]) * ab[1]) / l2)
                        .clamp(0.0, 1.0);
                    let dd = (p[0] - s[0][0] - f * ab[0]).hypot(p[1] - s[0][1] - f * ab[1]);
                    if dd < m.3 {
                        m = (c, i, f, dd);
                    }
                }
            }
            m
        };
        // Só os trechos ABERTOS (um fechado inteiro à vista não tem ponta: a «ponta» é a emenda).
        let mut abertos = t.clone();
        abertos.subpaths.retain(|c| !c.closed);
        if abertos.closed {
            abertos.verts.clear();
        }
        for l in polilinhas(&abertos, false) {
            if l.len() < 3 || perto(l[l.len() / 2]).3 > 1e-4 {
                continue;
            }
            for p in [l[0], l[l.len() - 1]] {
                let (c, i, f, _) = perto(p);
                let pl = &contorno[c];
                let n = pl.len() - 1;
                let comp = |j: usize| (pl[j + 1][0] - pl[j][0]).hypot(pl[j + 1][1] - pl[j][1]);
                let mut melhor = 3.0 * LARGURA;
                for &(cc, j, g) in cruz.iter().filter(|x| x.0 == c) {
                    let _ = cc;
                    // O arco de (i, f) a (j, g) pelos dois sentidos da volta.
                    let (mut ida, mut k, mut a) = (0.0, i, f);
                    for _ in 0..n {
                        if k == j && g >= a {
                            ida += (g - a) * comp(k);
                            break;
                        }
                        ida += (1.0 - a) * comp(k);
                        (k, a) = ((k + 1) % n, 0.0);
                    }
                    let (mut volta, mut k, mut a) = (0.0, i, f);
                    for _ in 0..n {
                        if k == j && g <= a {
                            volta += (a - g) * comp(k);
                            break;
                        }
                        volta += a * comp(k);
                        (k, a) = ((k + n - 1) % n, 1.0);
                    }
                    melhor = melhor.min(ida.min(volta));
                }
                out.push(melhor / LARGURA);
            }
        }
    }
    out
}

/// ⭐⭐⭐ **GATE — nenhuma ponta de corte fica a um TIQUE do cruzamento** (`0,05`–`1` largura), de
/// `110°` a `170°`. ⛔ **O CONTROLO:** sem o encaixe há tiques (MEDIDO na `=6`: `0,13`–`0,98`).
#[test]
fn nenhuma_ponta_de_corte_fica_a_um_tique_do_cruzamento() {
    let tiques = |v: &[f64]| v.iter().filter(|d| **d > 0.05 && **d < 1.0).count();
    let (mut com, mut sem, mut no_cruzamento) = (0, 0, 0);
    for graus in [110f32, 130.0, 150.0, 170.0] {
        super::SEM_ENCAIXE.with(|c| c.set(true));
        let antes = pontas(&desenho(graus));
        super::SEM_ENCAIXE.with(|c| c.set(false));
        let depois = pontas(&desenho(graus));
        println!(
            "  {graus}°: tiques sem o encaixe {} · com {} · pontas {} {depois:.2?}",
            tiques(&antes),
            tiques(&depois),
            depois.len()
        );
        sem += tiques(&antes);
        com += tiques(&depois);
        no_cruzamento += depois.iter().filter(|d| **d <= 0.05).count();
    }
    assert!(sem > 0, "o CONTROLO: sem o encaixe não há tique nenhum");
    assert!(
        no_cruzamento > 0,
        "o CONTROLO: nenhuma ponta cai num cruzamento"
    );
    assert_eq!(
        com, 0,
        "{com} pontas de corte ficaram a um tique do cruzamento"
    );
}

/// SONDA — a foto SVG desta fixtura a `SONDA_G` graus (`target/prova/cruza.svg`): o contorno a azul
/// fino, a camada do traço a vermelho.
#[test]
#[ignore = "sonda: escreve SVG"]
fn diag_a_foto_da_fixtura() {
    use std::fmt::Write as _;
    let g: f32 = std::env::var("SONDA_G")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(130.0);
    let d = desenho(g);
    let mut s = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-30 -40 90 70' width='2400' height='1867'><rect x='-30' y='-40' width='90' height='70' fill='white'/>",
    );
    for x in d.values() {
        for (p, cor, w) in [
            (Some(&x.forma), "#1e66ff", 0.08),
            (x.traco.as_ref(), "#8b0000", LARGURA),
        ] {
            let Some(p) = p else { continue };
            for l in polilinhas(p, false) {
                let pts: Vec<String> = l
                    .iter()
                    .map(|q| format!("{:.4},{:.4}", q[0], -q[1]))
                    .collect();
                let _ = write!(
                    s,
                    "<polyline points='{}' fill='none' stroke='{cor}' stroke-width='{w}' stroke-opacity='0.8'/>",
                    pts.join(" ")
                );
            }
        }
    }
    s.push_str("</svg>");
    std::fs::create_dir_all("target/prova").expect("pasta");
    std::fs::write("target/prova/cruza.svg", s).expect("svg");
}
