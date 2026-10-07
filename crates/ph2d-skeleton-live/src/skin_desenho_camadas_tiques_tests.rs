//! A13 (sonda) — os TIQUES SOLTOS da camada do traço na `=6` (as barras da cena reconstruídas
//! aqui: `4,5 × 0,75`, três ossos, *Repeat* de duas cópias e, na `1`, *Hatch*). Cada trecho cortado de
//! um contorno fechado: o comprimento, as pontas antes e depois do encaixe (F57), a distância de cada
//! ponta ao cruzamento desenhado mais perto e se o meio do trecho está TAPADO pelas três réguas
//! (triângulos rectos · malha fina do produto · malha a `1 %`).

use super::super::{
    AMOSTRAS_POR_FORMA, TOLERANCIA_DA_DIAGONAL, amostras_no_orcamento, diagonal, frente, lida,
    os_nos_servem, segmentos,
};
use super::cruza::Bordas;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{ShapeKind, VecPath, VecPathId, VecScene, VecVertex};
use ph2d_vec_skin::curva::{Bake, CampoIndexado};

const L: f64 = 4.5;
const T: f64 = 0.75;
const O: [f64; 2] = [-2.6, -0.6];

/// Uma barra da `=6` presa e dobrada em S a `g1`/`g2` (a `dobra_duas` da cena).
fn barra(hatch: bool, g1: f32, g2: f32) -> (SimWorld, VecScene, Entity, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let mut p = ph2d_vec_scene::cook_tinted(
        ShapeKind::RoundRect,
        [O[0] - L / 2.0, O[1] - T / 2.0],
        [O[0] + L / 2.0, O[1] + T / 2.0],
        &[T / 2.0],
        [230, 170, 90],
    );
    p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(110, 70, 30, 255),
        T * 0.06,
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
    if hatch {
        p.effects.push(FxEntry::new(PathEffect::Hatch(
            ph2d_vec_scene::fx_hatch::HatchSpec {
                angle: 45.0,
                spacing: 8.0,
                cross: false,
            },
        )));
    }
    let id = scene.push_path(p);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let (x0, passo) = (O[0] - L / 2.0 + T / 2.0, (L - T) / 3.0);
    let (mut pai, mut raiz) = (None, None);
    for k in 0..3 {
        let a = [x0 + passo * f64::from(k), O[1]];
        let b = [x0 + passo * f64::from(k + 1), O[1]];
        let bits = crate::bone::create(&mut sim, pai, a, b).expect("osso");
        let e = Entity::try_from_bits(bits).expect("entidade");
        raiz.get_or_insert(e);
        pai = Some(e);
    }
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], raiz),
        1
    );
    let (mut e, mut k) = (raiz.expect("raiz"), 0);
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(e).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<ph2d_skeleton_ecs::Bone>(**c).is_some())
            .copied()
    }) {
        e = f;
        k += 1;
        let (sinal, g) = if k % 2 == 1 { (1.0, g1) } else { (-1.0, g2) };
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
            t.rotation += sinal * g.to_radians();
        }
    }
    let alvo = Entity::try_from_bits(map[&id]).expect("forma");
    (sim, scene, alvo, id)
}

fn bits(p: &VecPath) -> Vec<u64> {
    (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .flat_map(|(v, f)| {
            std::iter::once(u64::from(f)).chain(v.iter().flat_map(|x| {
                [x.anchor, x.in_handle, x.out_handle]
                    .into_iter()
                    .flat_map(|q| q.map(f64::to_bits))
            }))
        })
        .collect()
}

/// O ponto do contorno `w` (fechado, `w[n] = w[0]`) no parâmetro `u = j + t`.
fn em(w: &[VecVertex], u: f64) -> [f64; 2] {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "índice"
    )]
    let j = (u.floor() as usize).min(w.len() - 2);
    #[expect(clippy::cast_precision_loss, reason = "índice")]
    frente::avalia(&frente::cubica(w, j), u - j as f64)
}

/// O comprimento de uma polilinha de nós (32 amostras por cúbica).
fn comprimento(v: &[VecVertex]) -> f64 {
    let mut s = 0.0;
    for k in 0..v.len().saturating_sub(1) {
        let c = frente::cubica(v, k);
        for i in 0..32 {
            let (a, b) = (
                frente::avalia(&c, f64::from(i) / 32.0),
                frente::avalia(&c, f64::from(i + 1) / 32.0),
            );
            s += (b[0] - a[0]).hypot(b[1] - a[1]);
        }
    }
    s
}

/// Os cruzamentos entre troços NÃO vizinhos dos contornos fechados de `d` (16 por segmento).
fn cruzamentos(d: &VecPath) -> Vec<[f64; 2]> {
    let pl: Vec<Vec<[f64; 2]>> = (0..d.contour_count())
        .filter_map(|c| d.contour(c))
        .filter(|(_, f)| *f)
        .map(|(v, _)| {
            let mut w = v.to_vec();
            w.push(v[0]);
            #[expect(clippy::cast_precision_loss, reason = "amostra")]
            let mut l: Vec<[f64; 2]> = (0..v.len() * 16).map(|i| em(&w, i as f64 / 16.0)).collect();
            l.push(l[0]);
            l
        })
        .collect();
    let tr: Vec<(usize, usize, [f64; 2], [f64; 2])> = pl
        .iter()
        .enumerate()
        .flat_map(|(c, l)| {
            l.windows(2)
                .enumerate()
                .map(move |(i, w)| (c, i, w[0], w[1]))
        })
        .collect();
    let mut out = Vec::new();
    for (x, &(ca, ia, a0, a1)) in tr.iter().enumerate() {
        for &(cb, ib, b0, b1) in &tr[x + 1..] {
            let n = pl[ca].len() - 1;
            if ca == cb && (ia.abs_diff(ib) <= 1 || ia.abs_diff(ib) == n - 1) {
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

/// A cópia da `projeta` da [`super::traco_sobre_o_assado`] (a sonda refaz o desenho e confere-o ao
/// bit contra o produto).
fn projeta(w: &[VecVertex], idx: &[usize], m: usize, (u, q, frac): frente::Ponta) -> f64 {
    let n = w.len() - 1;
    #[expect(clippy::cast_precision_loss, reason = "contagem de nós")]
    let um = u.rem_euclid(m as f64);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segmento"
    )]
    let k = (um.floor() as usize).min(m - 1);
    let fim = (if k + 1 < m { idx[k + 1] } else { n }).max(idx[k] + 1);
    let mut pts: Vec<(usize, f64, [f64; 2])> = Vec::new();
    for j in idx[k]..fim {
        let c = frente::cubica(w, j);
        pts.extend((0..32).map(|i| {
            (
                j,
                f64::from(i) / 32.0,
                frente::avalia(&c, f64::from(i) / 32.0),
            )
        }));
    }
    pts.push((fim - 1, 1.0, w[fim].anchor));
    let mut acc = vec![0.0];
    for i in 1..pts.len() {
        let (a, b) = (pts[i - 1].2, pts[i].2);
        acc.push(acc[i - 1] + (b[0] - a[0]).hypot(b[1] - a[1]));
    }
    let total = acc[acc.len() - 1].max(1e-18);
    let dist = |p: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    let (bi, _) = (0..pts.len())
        .filter(|&i| (acc[i] / total - frac).abs() <= 0.05)
        .map(|i| (i, dist(pts[i].2)))
        .min_by(|x, y| x.1.total_cmp(&y.1))
        .unwrap_or((0, 0.0));
    let (bj, bt) = (pts[bi].0, pts[bi].1);
    let d_em = |t: f64| dist(frente::avalia(&frente::cubica(w, bj), t));
    let (mut a, mut b) = ((bt - 1.0 / 32.0).max(0.0), (bt + 1.0 / 32.0).min(1.0));
    for _ in 0..40 {
        let (x, y) = ((2.0 * a + b) / 3.0, (a + 2.0 * b) / 3.0);
        if d_em(x) < d_em(y) {
            b = y;
        } else {
            a = x;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
    let u = bj as f64 + 0.5 * (a + b);
    u
}

/// O recorte `[u0, u1]` do contorno `w` (com volta), como na produção.
fn recorte(w: &[VecVertex], u0: f64, u1: f64) -> Vec<VecVertex> {
    #[expect(clippy::cast_precision_loss, reason = "contagem")]
    let fim = (w.len() - 1) as f64;
    if u0 < u1 {
        return frente::recorta(w, u0, u1)
            .into_iter()
            .map(|(v, _)| v)
            .collect();
    }
    let mut p: Vec<VecVertex> = frente::recorta(w, u0, fim)
        .into_iter()
        .map(|(v, _)| v)
        .collect();
    let resto: Vec<VecVertex> = frente::recorta(w, 0.0, u1)
        .into_iter()
        .map(|(v, _)| v)
        .collect();
    if let (Some(j), Some(r)) = (p.last_mut(), resto.first()) {
        j.out_handle = r.out_handle;
    }
    p.extend(resto.into_iter().skip(1));
    p
}

/// A medida de uma barra numa pose, com `por_forma` amostras por forma — uma linha por trecho cortado curto (`< 1,5`
/// larguras) e o resumo.
#[expect(clippy::too_many_lines, reason = "sonda")]
fn mede(hatch: bool, g1: f32, g2: f32, por_forma: usize) -> String {
    use std::fmt::Write as _;
    let (sim, scene, e, id) = barra(hatch, g1, g2);
    let produto = crate::skin_live::recook_leis(
        &sim,
        &mut scene.clone(),
        crate::skin_desenho::Leis::do_ambiente(),
    );
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(e)
        .expect("bind")
        .clone();
    let pele = crate::skin_live::skin_of(&sim, e).expect("pele");
    let index = crate::skin_live::bone_index(&sim);
    let ordem = crate::esqueletos::profundidades(&sim, &skin, &index);
    let prep = lida(e.to_bits(), &skin).expect("fonte");
    let g = &prep.guardado;
    let pesos = skin.pesos_do_quadro(if g.valida() { &g.pesos } else { &[] });
    let correcoes = skin.correcoes_resolvidas();
    let campo = g.campo.as_ref().expect("campo");
    let indice = prep.indice.as_ref();
    let (f0, t0) = if os_nos_servem(&g.path) {
        (g.path.clone(), pesos.to_vec())
    } else {
        let (c, t) = prep.cozido.as_ref().expect("cozido");
        (c.clone(), skin.pesos_do_quadro(t).to_vec())
    };
    let (f, t) =
        frente::so_o_que_se_ve(&f0, &t0, (campo, indice), (&pele, &correcoes, true), &ordem)
            .unwrap_or((f0, t0));
    let (d, nos) = ph2d_vec_skin::curva::assa_a_pele_com_nos(
        &pele,
        &f,
        &t,
        &correcoes,
        true,
        CampoIndexado {
            campo: Some(campo),
            indice,
            suave: None,
        },
        Bake {
            amostras: amostras_no_orcamento(segmentos(&f), por_forma),
            tolerancia: TOLERANCIA_DA_DIAGONAL * diagonal(&f),
        },
    );
    let Some(cortes) =
        frente::cortes_dos_fechados(&f, (campo, indice), (&pele, &correcoes, true), &ordem)
    else {
        return format!(
            "  {g1}/{g2} barra {} {por_forma}: nada tapado\n",
            u8::from(hatch)
        );
    };
    let traco = super::traco_sobre_o_assado(&d, &nos, &f, &cortes).expect("traço");
    let vis = &produto[&id];
    if por_forma == AMOSTRAS_POR_FORMA {
        assert_eq!(
            bits(&vis.forma),
            bits(&d),
            "a sonda refaz a FORMA do produto"
        );
        assert_eq!(
            bits(vis.traco.as_ref().expect("camada")),
            bits(&traco),
            "a sonda refaz o TRAÇO do produto"
        );
    }
    let largura = d.stroke.as_ref().map_or(0.0, |s| s.width);
    let bordas = Bordas::de(&d, &f, largura);
    let cruz = cruzamentos(&d);
    let ao_cruz = |p: [f64; 2]| {
        cruz.iter()
            .map(|c| (c[0] - p[0]).hypot(c[1] - p[1]))
            .fold(f64::MAX, f64::min)
            / largura
    };
    let posa = |p: [f64; 2]| {
        let linha = campo.linha_com(p, indice).unwrap_or_default();
        let mut w = pele.scratch();
        pele.point_corrected(p, Some(&linha), &mut w, &correcoes)
    };
    let mut s = String::new();
    let (mut todos, mut curtos) = (0, 0);
    let mut base = 0;
    for c in 0..f.contour_count() {
        let ((fv, fechado), (ov, _)) = (f.contour(c).expect("c"), d.contour(c).expect("c"));
        let m = fv.len();
        base += m;
        let Some(trechos) = cortes.get(c).and_then(Option::as_ref).filter(|_| fechado) else {
            continue;
        };
        let mut idx = Vec::with_capacity(m);
        let mut de = 0;
        for k in 0..m {
            let alvo = nos[base - m + k];
            let i = (de..ov.len()).find(|&i| ov[i].anchor == alvo).expect("nó");
            idx.push(i);
            de = i;
        }
        let n = ov.len();
        let mut w = ov.to_vec();
        w.push(ov[0]);
        let mut wf = fv.to_vec();
        wf.push(fv[0]);
        #[expect(clippy::cast_precision_loss, reason = "contagem")]
        let (fim, mf) = (n as f64, m as f64);
        for &(a, b) in trechos {
            todos += 1;
            let (u0, u1) = (projeta(&w, &idx, m, a), projeta(&w, &idx, m, b));
            let cru = comprimento(&recorte(&w, u0, u1)) / largura;
            let enc = bordas.encaixa_trecho(c, (u0, u1), fim, largura);
            let (e0, e1) = enc.unwrap_or((u0, u1));
            let fin = if enc.is_some() {
                comprimento(&recorte(&w, e0, e1)) / largura
            } else {
                0.0
            };
            if std::env::var("SONDA_TODOS").is_err() && (fin >= 1.5 || enc.is_none() && cru >= 1.5)
            {
                continue;
            }
            curtos += 1;
            let (p0, p1) = (em(&w, e0), em(&w, e1));
            // O meio do trecho na FONTE (o parâmetro em repouso), e as três réguas nele e a 1/4.
            let (sa, sb) = (a.0, if b.0 < a.0 { b.0 + mf } else { b.0 });
            let rest: Vec<[f64; 2]> = [0.25, 0.5, 0.75]
                .iter()
                .map(|x| em(&wf, (sa + (sb - sa) * x).rem_euclid(mf)))
                .collect();
            let tres = frente::tests::cobre::tapado_nas_tres(
                &f,
                (campo, indice),
                (&pele, &correcoes),
                &ordem,
                &rest,
            );
            let meio = posa(rest[1]);
            let _ = writeln!(
                s,
                "    c{c} fonte [{:.3},{:.3}] · cru {cru:.2} → final {} · pontas ({:.3},{:.3})–({:.3},{:.3}) · \
                 ao cruzamento {:.2}/{:.2} (cru {:.2}/{:.2}) · encaixe {}/{} · meio posado ({:.3},{:.3}) · \
                 tapado ¼/½/¾ [grossa,produto,1%] {:?}",
                a.0,
                b.0,
                if enc.is_some() {
                    format!("{fin:.2}")
                } else {
                    "SAIU".into()
                },
                p0[0],
                p0[1],
                p1[0],
                p1[1],
                ao_cruz(p0),
                ao_cruz(p1),
                ao_cruz(em(&w, u0.rem_euclid(fim))),
                ao_cruz(em(&w, u1.rem_euclid(fim))),
                u8::from((e0 - u0.rem_euclid(fim)).abs() > 1e-12),
                u8::from((e1 - u1.rem_euclid(fim)).abs() > 1e-12),
                meio[0],
                meio[1],
                tres.iter()
                    .map(|(x, y, z)| [u8::from(*x), u8::from(*y), u8::from(*z)])
                    .collect::<Vec<_>>()
            );
            // ⭐ Cada ponta: as três réguas a ¼…1 largura para FORA (o lado que o corte diz tapado) e
            // para DENTRO, e onde o encaixe a pôs (em larguras ao longo da fonte; < 0 = para fora).
            for (nome, u, sinal, pf) in [("início", sa, -1.0, p0), ("fim", sb, 1.0, p1)] {
                let rp = |x: f64| em(&wf, x.rem_euclid(mf));
                let (h, q0) = (1e-4, posa(rp(u)));
                let q1 = posa(rp(u + sinal * h));
                let vel = (q1[0] - q0[0]).hypot(q1[1] - q0[1]) / h;
                let du = largura / vel.max(1e-12);
                let lados: Vec<[f64; 2]> = [-1.0, -0.75, -0.5, -0.25, 0.25, 0.5, 0.75, 1.0]
                    .iter()
                    .map(|j| rp(u - sinal * j * du))
                    .collect();
                let tr = frente::tests::cobre::tapado_nas_tres(
                    &f,
                    (campo, indice),
                    (&pele, &correcoes),
                    &ordem,
                    &lados,
                );
                let txt: Vec<String> = tr
                    .iter()
                    .map(|(x, y, z)| format!("{}{}{}", u8::from(*x), u8::from(*y), u8::from(*z)))
                    .collect();
                // O parâmetro de repouso da ponta FINAL (a mais perto, numa janela de ±2 larguras).
                let (mut melhor, mut ux) = (f64::MAX, u);
                for i in -400..=400 {
                    let x = u + f64::from(i) / 200.0 * du;
                    let p = posa(rp(x));
                    let dd = (p[0] - pf[0]).hypot(p[1] - pf[1]);
                    if dd < melhor {
                        (melhor, ux) = (dd, x);
                    }
                }
                let dentro = rp(u - sinal * 0.25 * du);
                let quem = frente::tests::cobre::quem_tapa(
                    &f,
                    (campo, indice),
                    (&pele, &correcoes),
                    &ordem,
                    &[rp(u + sinal * 0.25 * du), dentro],
                );
                let resumo = |x: &Option<(
                    frente::tests::cobre::Dono,
                    Vec<frente::tests::cobre::Dono>,
                )>| {
                    x.as_ref().map_or("fora da malha".to_string(), |((_, v, c), cob)| {
                        let virados = cob.iter().filter(|x| x.1).count();
                        format!(
                            "dono chave {c:.2}{} · {} cobridores ({virados} virados, chaves {:?})",
                            if *v { " VIRADO" } else { "" },
                            cob.len(),
                            cob.iter().map(|x| format!("{:.2}", x.2)).collect::<std::collections::BTreeSet<_>>()
                        )
                    })
                };
                let _ = writeln!(
                    s,
                    "        ¼ para fora: {} | ¼ para dentro: {}",
                    resumo(&quem[0]),
                    resumo(&quem[1])
                );
                let _ = writeln!(
                    s,
                    "      {nome}: dentro→fora −1…−¼ | ¼…1 = {} · a ponta final está a {:+.2} larguras da                      decidida (resíduo {:.3})",
                    txt.join(" "),
                    (ux - u) / du * sinal,
                    melhor / largura
                );
            }
        }
    }
    // As polilinhas do traço À VISTA (32 por cúbica), com o arco acumulado.
    let vistos: Vec<Vec<([f64; 2], f64)>> = (0..traco.contour_count())
        .filter_map(|c| traco.contour(c))
        .map(|(v, fechado)| {
            let mut w = v.to_vec();
            if fechado {
                w.push(v[0]);
            }
            let mut out = vec![(w[0].anchor, 0.0)];
            for k in 0..w.len() - 1 {
                let cb = frente::cubica(&w, k);
                for i in 1..=32 {
                    let p = frente::avalia(&cb, f64::from(i) / 32.0);
                    let (q, s0) = out[out.len() - 1];
                    out.push((p, s0 + (p[0] - q[0]).hypot(p[1] - q[1])));
                }
            }
            out
        })
        .collect();
    // A distância de uma ponta ao traço À VISTA mais perto que não seja ela mesma (o seu trecho a
    // mais de 3 larguras de arco dela).
    let ao_traco = |c: usize, inicio: bool| {
        let l = &vistos[c];
        let (p, s0) = if inicio { l[0] } else { l[l.len() - 1] };
        vistos
            .iter()
            .enumerate()
            .flat_map(|(k, m)| m.iter().map(move |x| (k, x)))
            .filter(|(k, (_, s))| *k != c || (s - s0).abs() > 3.0 * largura)
            .map(|(_, (q, _))| (q[0] - p[0]).hypot(q[1] - p[1]))
            .fold(f64::MAX, f64::min)
            / largura
    };
    for c in 0..traco.contour_count() {
        let (v, fechado) = traco.contour(c).expect("c");
        let (a, z) = (v[0].anchor, v[v.len() - 1].anchor);
        if !fechado {
            let _ = writeln!(
                s,
                "    peça {c}: ponta início a {:.2} larguras de outro traço à vista, fim a {:.2}",
                ao_traco(c, true),
                ao_traco(c, false)
            );
        }
        let _ = writeln!(
            s,
            "    peça {c}: {} · {:.2} larguras · ({:.3},{:.3})–({:.3},{:.3})",
            if fechado { "fechada" } else { "aberta" },
            comprimento(&{
                let mut w = v.to_vec();
                if fechado {
                    w.push(v[0]);
                }
                w
            }) / largura,
            a[0],
            a[1],
            z[0],
            z[1]
        );
    }
    format!(
        "  {g1}/{g2} barra {} {por_forma} amostras: {todos} trechos, {curtos} curtos (< 1,5 larguras)\n{s}",
        u8::from(hatch)
    )
}

/// ⭐ **SONDA — A13: os tiques soltos do traço** a `110/110` (controlo) e `170/{110,140,150,170}`,
/// nas duas barras, com a amostragem do produto e a de metade.
#[test]
#[ignore = "sonda: imprime"]
fn diag_a13_os_tiques_soltos_do_traco() {
    for (g1, g2) in [
        (110f32, 110f32),
        (170.0, 110.0),
        (170.0, 140.0),
        (170.0, 150.0),
        (170.0, 170.0),
    ] {
        for hatch in [false, true] {
            for por_forma in [AMOSTRAS_POR_FORMA, AMOSTRAS_POR_FORMA / 2] {
                let linha = std::thread::scope(|s| {
                    s.spawn(|| mede(hatch, g1, g2, por_forma))
                        .join()
                        .expect("thread")
                });
                print!("{linha}");
            }
        }
    }
}
