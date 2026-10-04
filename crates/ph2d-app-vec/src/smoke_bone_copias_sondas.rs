//! Sondas da cena `=6` — a FOTO sem a placa: o desenho de cada barra presa em SVG, a uma dobra
//! qualquer (`SONDA_G1`/`SONDA_G2`, as duas juntas; omissão `170`/`110`, A9). Saída em
//! `SONDA_SAIDA` (omissão `target/prova/copias`); `magick <f>.svg <f>.png` para ver.

use super::*;
use std::fmt::Write as _;

fn env_f32(nome: &str, omissao: f32) -> f32 {
    std::env::var(nome)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(omissao)
}

/// Os contornos de `p` como um `d` de SVG (y para baixo).
pub(crate) fn d_de(p: &ph2d_vec_scene::VecPath) -> String {
    let mut d = String::new();
    for c in 0..p.contour_count() {
        let Some((v, fechado)) = p.contour(c) else {
            continue;
        };
        if v.is_empty() {
            continue;
        }
        let _ = write!(d, "M{:.5} {:.5} ", v[0].anchor[0], -v[0].anchor[1]);
        let n = if fechado { v.len() } else { v.len() - 1 };
        for k in 0..n {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let _ = write!(
                d,
                "C{:.5} {:.5} {:.5} {:.5} {:.5} {:.5} ",
                a.out_handle[0],
                -a.out_handle[1],
                b.in_handle[0],
                -b.in_handle[1],
                b.anchor[0],
                -b.anchor[1]
            );
        }
        if fechado {
            d.push('Z');
        }
    }
    d
}

/// O SVG do desenho: o preenchimento cinzento, o traço da forma a preto e a CAMADA do traço a
/// vermelho escuro (para se ver qual das duas desenha cada risco).
pub(crate) fn svg_do_desenho(
    d: &ph2d_skeleton_live::skin_desenho::SkinDesenhado,
    caixa: [f64; 4],
) -> String {
    let [x0, y0, x1, y1] = caixa;
    let mut s = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{x0} {} {} {}' width='2400' \
         height='{:.0}'><rect x='{x0}' y='{}' width='{}' height='{}' fill='white'/>",
        -y1,
        x1 - x0,
        y1 - y0,
        2400.0 * (y1 - y0) / (x1 - x0),
        -y1,
        x1 - x0,
        y1 - y0
    );
    for x in d.values() {
        let w = [Some(&x.forma), x.traco.as_ref()]
            .into_iter()
            .flatten()
            .filter_map(|p| p.stroke.as_ref())
            .map(|s| s.width)
            .fold(0.0, f64::max);
        let regra = if x.forma.fill_rule == ph2d_vec_scene::FillRule::EvenOdd {
            "evenodd"
        } else {
            "nonzero"
        };
        let traco_da_forma = if x.traco.is_none() {
            format!("stroke='black' stroke-width='{w}'")
        } else {
            String::new()
        };
        let _ = write!(
            s,
            "<path d='{}' fill='#bbbbbb' fill-rule='{regra}' {traco_da_forma} stroke-linejoin='round'/>",
            d_de(&x.forma)
        );
        if let Some(t) = &x.traco {
            let _ = write!(
                s,
                "<path d='{}' fill='none' stroke='#8b0000' stroke-width='{w}' stroke-linecap='round' \
                 stroke-linejoin='round'/>",
                d_de(t)
            );
        }
        if std::env::var("SONDA_CONTORNO").is_ok() {
            let _ = write!(
                s,
                "<path d='{}' fill='none' stroke='#1e66ff' stroke-width='{}'/>",
                d_de(&x.forma),
                w / 5.0
            );
        }
    }
    s.push_str("</svg>");
    s
}

/// ⭐ **SONDA — a FOTO da `=6`** a `SONDA_G1`/`SONDA_G2` (as juntas de cada barra, em S), inteira e
/// a caixa de cada barra.
#[test]
#[ignore = "sonda: escreve SVG"]
fn diag_a_foto_das_copias() {
    let (g1, g2) = (env_f32("SONDA_G1", 170.0), env_f32("SONDA_G2", 110.0));
    let saida = std::env::var("SONDA_SAIDA").unwrap_or_else(|_| "target/prova/copias".into());
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
    for (id, raiz) in st.bone_smoke_pend.take().expect("pendentes") {
        assert_eq!(
            ph2d_skeleton_live::skin_live::bind(&mut sim, &mut scene, &st.entities, &[id], raiz),
            1
        );
        crate::smoke_bone_par::dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
    }
    let d = ph2d_skeleton_live::skin_live::recook_desenhando(&sim, &mut scene.clone());
    std::fs::create_dir_all(&saida).expect("pasta");
    let caixa = |x: &ph2d_vec_scene::VecPath| {
        let mut c = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        for v in x.verts_all() {
            c = [
                c[0].min(v.anchor[0]),
                c[1].min(v.anchor[1]),
                c[2].max(v.anchor[0]),
                c[3].max(v.anchor[1]),
            ];
        }
        [c[0] - 0.2, c[1] - 0.2, c[2] + 0.2, c[3] + 0.2]
    };
    for (k, (id, x)) in d.iter().enumerate() {
        let so: ph2d_skeleton_live::skin_desenho::SkinDesenhado = [(*id, x.clone())].into();
        let f = format!("{saida}/copias_{g1}_{g2}_barra{k}.svg");
        std::fs::write(&f, svg_do_desenho(&so, caixa(&x.forma))).expect("svg");
        println!("  {f} · camada do traço: {}", x.traco.is_some());
    }
}

/// A polilinha de cada contorno de `p` (`n` amostras por segmento).
pub(crate) fn polilinhas(p: &ph2d_vec_scene::VecPath, n: usize) -> Vec<Vec<[f64; 2]>> {
    (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .filter(|(v, _)| v.len() > 1)
        .map(|(v, fechado)| {
            let m = if fechado { v.len() } else { v.len() - 1 };
            let mut out = Vec::new();
            for k in 0..m {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                for i in 0..n {
                    #[expect(clippy::cast_precision_loss, reason = "amostra")]
                    let t = i as f64 / n as f64;
                    let s = 1.0 - t;
                    out.push([0, 1].map(|j| {
                        s * s * s * c[0][j]
                            + 3.0 * s * s * t * c[1][j]
                            + 3.0 * s * t * t * c[2][j]
                            + t * t * t * c[3][j]
                    }));
                }
            }
            out.push(if fechado {
                v[0].anchor
            } else {
                v[v.len() - 1].anchor
            });
            out
        })
        .collect()
}

/// Os pontos onde dois troços NÃO vizinhos das polilinhas se cruzam.
pub(crate) fn cruzamentos(pl: &[Vec<[f64; 2]>]) -> Vec<[f64; 2]> {
    let trocos: Vec<(usize, usize, [f64; 2], [f64; 2])> = pl
        .iter()
        .enumerate()
        .flat_map(|(c, l)| {
            l.windows(2)
                .enumerate()
                .map(move |(i, w)| (c, i, w[0], w[1]))
        })
        .collect();
    let mut out = Vec::new();
    for (x, &(ca, ia, a0, a1)) in trocos.iter().enumerate() {
        let (lo, hi) = (
            [a0[0].min(a1[0]), a0[1].min(a1[1])],
            [a0[0].max(a1[0]), a0[1].max(a1[1])],
        );
        for &(cb, ib, b0, b1) in &trocos[x + 1..] {
            if ca == cb && ia.abs_diff(ib) <= 1 {
                continue;
            }
            if b0[0].max(b1[0]) < lo[0]
                || b0[0].min(b1[0]) > hi[0]
                || b0[1].max(b1[1]) < lo[1]
                || b0[1].min(b1[1]) > hi[1]
            {
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
            let (t, u) = (
                (q[0] * s[1] - q[1] * s[0]) / den,
                (q[0] * r[1] - q[1] * r[0]) / den,
            );
            if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                out.push([a0[0] + t * r[0], a0[1] + t * r[1]]);
            }
        }
    }
    out
}

/// ⭐ **SONDA — A9: onde acaba cada corte do traço de um FECHADO.** Na dobra, o traço de trás só pode
/// deixar de se ver onde o contorno desenhado CRUZA outra borda (ou na ponta de uma dobra): para
/// cada ponta de um trecho cortado, a distância ao cruzamento mais perto do contorno desenhado, em
/// larguras de traço. A régua não usa a chave de osso nem a malha.
#[test]
#[ignore = "sonda: imprime"]
fn diag_as_pontas_dos_cortes() {
    for (g1, g2) in [
        (110f32, 110f32),
        (130.0, 130.0),
        (150.0, 110.0),
        (170.0, 110.0),
        (170.0, 170.0),
    ] {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut st = crate::state::VecState::default();
        build(&mut scene, &mut sim, &mut st);
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
        for (id, raiz) in st.bone_smoke_pend.take().expect("pendentes") {
            assert_eq!(
                ph2d_skeleton_live::skin_live::bind(
                    &mut sim,
                    &mut scene,
                    &st.entities,
                    &[id],
                    raiz
                ),
                1
            );
            crate::smoke_bone_par::dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
        }
        let d = ph2d_skeleton_live::skin_live::recook_desenhando(&sim, &mut scene.clone());
        for (k, x) in d.values().enumerate() {
            let Some(t) = &x.traco else { continue };
            let w = t.stroke.as_ref().map_or(1.0, |s| s.width);
            let contorno = polilinhas(&x.forma, 64);
            let cruz = cruzamentos(&contorno);
            let perto_do_contorno = |p: [f64; 2]| {
                contorno.iter().flat_map(|l| l.windows(2)).any(|s| {
                    let ab = [s[1][0] - s[0][0], s[1][1] - s[0][1]];
                    let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18);
                    let u = (((p[0] - s[0][0]) * ab[0] + (p[1] - s[0][1]) * ab[1]) / l2)
                        .clamp(0.0, 1.0);
                    (p[0] - s[0][0] - u * ab[0]).hypot(p[1] - s[0][1] - u * ab[1]) < 1e-4
                })
            };
            let mut dist: Vec<f64> = Vec::new();
            for l in polilinhas(t, 64) {
                if l.len() < 3 || !perto_do_contorno(l[l.len() / 2]) || l[0] == l[l.len() - 1] {
                    continue;
                }
                for p in [l[0], l[l.len() - 1]] {
                    let dmin = cruz
                        .iter()
                        .map(|c| (c[0] - p[0]).hypot(c[1] - p[1]))
                        .fold(f64::MAX, f64::min);
                    dist.push(dmin / w);
                }
            }
            dist.sort_by(f64::total_cmp);
            let txt: Vec<String> = dist.iter().map(|v| format!("{v:.2}")).collect();
            println!(
                "  {g1}°/−{g2}° barra {k}: {} pontas, em larguras: [{}]",
                dist.len(),
                txt.join(" ")
            );
        }
    }
}
