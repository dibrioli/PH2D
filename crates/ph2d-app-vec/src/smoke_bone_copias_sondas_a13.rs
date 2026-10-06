//! Sonda A13 da cena `=6` — as camadas da dobra e o winding do desenho, num irmão das
//! [`super`] pelo tecto de LOC.

use super::*;

/// O número de voltas das polilinhas `pl` em torno de `p` (contornos já fechados).
pub(super) fn voltas(pl: &[Vec<[f64; 2]>], p: [f64; 2]) -> i32 {
    let mut w = 0;
    for l in pl {
        for s in l.windows(2) {
            let (a, b) = (s[0], s[1]);
            let x = (b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1]);
            if a[1] <= p[1] {
                if b[1] > p[1] && x > 0.0 {
                    w += 1;
                }
            } else if b[1] <= p[1] && x < 0.0 {
                w -= 1;
            }
        }
    }
    w
}

/// Só os contornos FECHADOS de `p`.
pub(super) fn fechados_de(p: &ph2d_vec_scene::VecPath) -> ph2d_vec_scene::VecPath {
    let mut f = p.clone();
    f.subpaths.retain(|c| c.closed);
    f
}

/// A distância de `p` à polilinha mais perto.
pub(super) fn dist_pl(pl: &[Vec<[f64; 2]>], p: [f64; 2]) -> f64 {
    pl.iter()
        .flat_map(|l| l.windows(2))
        .map(|s| {
            let ab = [s[1][0] - s[0][0], s[1][1] - s[0][1]];
            let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18);
            let u = (((p[0] - s[0][0]) * ab[0] + (p[1] - s[0][1]) * ab[1]) / l2).clamp(0.0, 1.0);
            (p[0] - s[0][0] - u * ab[0]).hypot(p[1] - s[0][1] - u * ab[1])
        })
        .fold(f64::MAX, f64::min)
}

/// Uma amostra do interior em repouso, posta: onde caiu, `W_repouso · sinal(det J)` e `|det J|`.
struct Posta {
    q: [f64; 2],
    c: i32,
    det: f64,
}

/// ⭐ **SONDA — A13: as camadas e o winding.** Hipótese H: a parte que a dobra vira tem orientação
/// invertida e o `NonZero` dá `0` onde uma camada virada cobre uma direita. A VERDADE é o interior
/// em repouso (a fonte do bake) posto pela MESMA lei do bake (o campo do domínio, `point_corrected`),
/// com o sinal de `det J` por diferenças centrais; o DESENHO é o `forma` do `recook_desenhando`.
#[test]
#[ignore = "sonda: imprime"]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "sonda"
)]
fn diag_a13_as_camadas_e_o_winding() {
    use ph2d_skeleton_live::skin_desenho as sd;
    let (_, t) = crate::smoke_bone_efeitos::PECA;
    let cel = t / 20.0;
    let passo = t / 80.0;
    let h = t * 1e-3;
    for (g1, g2) in [
        (110f32, 110f32),
        (170.0, 110.0),
        (170.0, 140.0),
        (170.0, 150.0),
        (170.0, 170.0),
    ] {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut st = crate::state::VecState::default();
        build(&mut scene, &mut sim, &mut st);
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
        let pend = st.bone_smoke_pend.take().expect("pendentes");
        for (id, raiz) in &pend {
            assert_eq!(
                ph2d_skeleton_live::skin_live::bind(
                    &mut sim,
                    &mut scene,
                    &st.entities,
                    &[*id],
                    *raiz
                ),
                1
            );
            crate::smoke_bone_par::dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
        }
        let d = ph2d_skeleton_live::skin_live::recook_desenhando(&sim, &mut scene.clone());
        let segs = ph2d_skeleton_live::skin_live::bone_segments(&sim);
        for (k, (id, raiz)) in pend.iter().enumerate() {
            let Some(x) = d.get(id) else {
                println!("{g1}/{g2} barra {k}: SEM desenho");
                continue;
            };
            let e = st
                .entities
                .get(id)
                .and_then(|b| Entity::try_from_bits(*b))
                .expect("entidade");
            let bind = sim
                .world()
                .get::<ph2d_skeleton_ecs::SkinBind>(e)
                .expect("bind")
                .clone();
            let pele = ph2d_skeleton_live::skin_live::skin_of(&sim, e).expect("pele");
            let prep = sd::lida(e.to_bits(), &bind).expect("fonte");
            let g = &prep.guardado;
            let fonte = if sd::os_nos_servem(&g.path) {
                g.path.clone()
            } else {
                prep.cozido
                    .as_ref()
                    .map_or_else(|| g.path.cooked().into_owned(), |c| c.0.clone())
            };
            let correcoes = bind.correcoes_resolvidas();
            let fora = std::cell::Cell::new(0usize);
            let derivada = |p: [f64; 2]| {
                let mut w = pele.scratch();
                pele.point(p, &mut w)
            };
            let pose = |p: [f64; 2]| {
                g.campo
                    .as_ref()
                    .and_then(|c| c.linha_com(p, prep.indice.as_ref()))
                    .map_or_else(
                        || {
                            fora.set(fora.get() + 1);
                            derivada(p)
                        },
                        |linha| {
                            let mut w = pele.scratch();
                            pele.point_corrected(p, Some(&linha), &mut w, &correcoes)
                        },
                    )
            };
            // ⭐ O REFERENCIAL: os nós da fonte postos contra o contorno desenhado.
            let desenho_todo = polilinhas(&fechados_de(&x.forma), 64);
            let mut erro = [0.0f64; 2];
            for v in fechados_de(&fonte).verts_all() {
                erro[0] = erro[0].max(dist_pl(&desenho_todo, pose(v.anchor)));
                erro[1] = erro[1].max(dist_pl(&desenho_todo, derivada(v.anchor)));
            }
            // ⭐ A IMAGEM EXACTA do contorno em repouso (grau = Σ sinal det J nas pré-imagens), e a
            // distância de Hausdorff dela ao desenho, nos dois sentidos.
            let posta_pl = |n: usize| -> Vec<Vec<[f64; 2]>> {
                polilinhas(&fechados_de(&fonte), n)
                    .into_iter()
                    .map(|l| l.into_iter().map(pose).collect())
                    .collect()
            };
            let imagem = posta_pl(512);
            // A amostragem do BAKE (`amostras_por_segmento` dos segmentos fechados), sem o ajuste.
            let segs_f: usize = (0..fechados_de(&fonte).contour_count())
                .filter_map(|c| fechados_de(&fonte).contour(c).map(|(v, _)| v.len()))
                .sum();
            let n_bake = sd::amostras_por_segmento(segs_f);
            let do_bake = posta_pl(n_bake);
            let haus_bake = [
                do_bake
                    .iter()
                    .flatten()
                    .map(|p| dist_pl(&imagem, *p))
                    .fold(0.0, f64::max),
                imagem
                    .iter()
                    .flatten()
                    .map(|p| dist_pl(&do_bake, *p))
                    .fold(0.0, f64::max),
                desenho_todo
                    .iter()
                    .flatten()
                    .map(|p| dist_pl(&do_bake, *p))
                    .fold(0.0, f64::max),
            ];
            println!(
                "    bake: {segs_f} segmentos fechados × {n_bake} amostras · Hausdorff assado→imagem {:.2e}, imagem→assado {:.2e}, desenho→assado {:.2e}",
                haus_bake[0], haus_bake[1], haus_bake[2]
            );
            let pior = imagem
                .iter()
                .flatten()
                .map(|p| (*p, dist_pl(&desenho_todo, *p)))
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(p, _)| p);
            let haus = [
                imagem
                    .iter()
                    .flatten()
                    .map(|p| dist_pl(&desenho_todo, *p))
                    .fold(0.0, f64::max),
                desenho_todo
                    .iter()
                    .flatten()
                    .map(|p| dist_pl(&imagem, *p))
                    .fold(0.0, f64::max),
            ];
            fora.set(0);
            // ⭐ A VERDADE: o interior em repouso, posto.
            let repouso = polilinhas(&fechados_de(&fonte), 32);
            let mut cx = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
            for p in repouso.iter().flatten() {
                cx = [
                    cx[0].min(p[0]),
                    cx[1].min(p[1]),
                    cx[2].max(p[0]),
                    cx[3].max(p[1]),
                ];
            }
            let mut postas: Vec<Posta> = Vec::new();
            let (nx, ny) = (
                ((cx[2] - cx[0]) / passo) as usize + 1,
                ((cx[3] - cx[1]) / passo) as usize + 1,
            );
            for i in 0..nx {
                for j in 0..ny {
                    let p = [
                        cx[0] + (i as f64 + 0.5) * passo,
                        cx[1] + (j as f64 + 0.5) * passo,
                    ];
                    let wr = voltas(&repouso, p);
                    if wr == 0 {
                        continue;
                    }
                    let (a, b) = (pose([p[0] + h, p[1]]), pose([p[0] - h, p[1]]));
                    let (c, dd) = (pose([p[0], p[1] + h]), pose([p[0], p[1] - h]));
                    let det = ((a[0] - b[0]) * (c[1] - dd[1]) - (a[1] - b[1]) * (c[0] - dd[0]))
                        / (4.0 * h * h);
                    postas.push(Posta {
                        q: pose(p),
                        c: wr * if det < 0.0 { -1 } else { 1 },
                        det: det.abs(),
                    });
                }
            }
            // ⭐ O DESENHO, numa grelha de `largura / 20`.
            let desenho = polilinhas(&fechados_de(&x.forma), 32);
            let mut bx = cx;
            for p in desenho.iter().flatten().chain(postas.iter().map(|s| &s.q)) {
                bx = [
                    bx[0].min(p[0]),
                    bx[1].min(p[1]),
                    bx[2].max(p[0]),
                    bx[3].max(p[1]),
                ];
            }
            let (x0, y0) = (bx[0] - 3.0 * cel, bx[1] - 3.0 * cel);
            let (gx, gy) = (
                ((bx[2] - x0) / cel) as usize + 4,
                ((bx[3] - y0) / cel) as usize + 4,
            );
            let celula = |q: [f64; 2]| {
                let (i, j) = (((q[0] - x0) / cel) as usize, ((q[1] - y0) / cel) as usize);
                (i < gx && j < gy).then_some(i * gy + j)
            };
            let (mut mais, mut menos, mut media) = (
                vec![0u32; gx * gy],
                vec![0u32; gx * gy],
                vec![0.0f64; gx * gy],
            );
            let viradas = postas.iter().filter(|s| s.c < 0).count();
            for s in &postas {
                let Some(c) = celula(s.q) else { continue };
                if s.c > 0 {
                    mais[c] += 1;
                } else {
                    menos[c] += 1;
                }
                media[c] += f64::from(s.c) * s.det * passo * passo / (cel * cel);
            }
            // A borda: bit 1 a do desenho, bit 2 a da imagem exacta (o ruído de aresta de cada um).
            let mut borda = vec![0u8; gx * gy];
            for (s, bit) in desenho
                .iter()
                .flat_map(|l| l.windows(2))
                .map(|s| (s, 1u8))
                .chain(imagem.iter().flat_map(|l| l.windows(2)).map(|s| (s, 2u8)))
            {
                let n = ((s[1][0] - s[0][0]).hypot(s[1][1] - s[0][1]) / (cel / 4.0)) as usize + 1;
                for m in 0..=n {
                    let u = m as f64 / n as f64;
                    let q = [
                        s[0][0] + u * (s[1][0] - s[0][0]),
                        s[0][1] + u * (s[1][1] - s[0][1]),
                    ];
                    let (i, j) = (((q[0] - x0) / cel) as i64, ((q[1] - y0) / cel) as i64);
                    for di in -2..=2 {
                        for dj in -2..=2 {
                            let (a, b) = (i + di, j + dj);
                            if a >= 0 && b >= 0 && (a as usize) < gx && (b as usize) < gy {
                                borda[a as usize * gy + b as usize] |= bit;
                            }
                        }
                    }
                }
            }
            // As juntas DESTA barra: as pontas partilhadas por dois ossos do esqueleto dela.
            let meus: Vec<u64> = ph2d_skeleton_live::skin_live::skeleton_of(&sim, *raiz)
                .iter()
                .map(|e| e.to_bits())
                .collect();
            let ossos: Vec<_> = segs.iter().filter(|s| meus.contains(&s.0)).collect();
            let juntas: Vec<[f64; 2]> = ossos
                .iter()
                .map(|s| s.2)
                .filter(|b| {
                    ossos
                        .iter()
                        .any(|o| (o.1[0] - b[0]).hypot(o.1[1] - b[1]) < 1e-6)
                })
                .collect();
            let pior = pior.map(|p| {
                let dj: Vec<String> = juntas
                    .iter()
                    .map(|j| format!("{:.2}", (j[0] - p[0]).hypot(j[1] - p[1]) / t))
                    .collect();
                format!(
                    "[{:.3}, {:.3}] a {} larguras das juntas",
                    p[0],
                    p[1],
                    dj.join("/")
                )
            });
            println!("    o pior ponto da imagem fora do desenho: {pior:?}");
            println!(
                "{g1}/{g2} barra {k}: regra {:?} · contornos fonte {} fechados / desenho {} fechados + {} abertos · traço à parte {} · nós da fonte vs desenho: campo {:.2e}, Skin::point {:.2e} · Hausdorff imagem→desenho {:.2e} desenho→imagem {:.2e} · fora do campo {} de {} · amostras viradas {}/{}",
                x.forma.fill_rule,
                fechados_de(&fonte).contour_count(),
                fechados_de(&x.forma).contour_count(),
                x.forma.contour_count() - fechados_de(&x.forma).contour_count(),
                x.traco.is_some(),
                erro[0],
                erro[1],
                haus[0],
                haus[1],
                fora.get(),
                postas.len() * 5,
                viradas,
                postas.len(),
            );
            for modo in [1u8, 3u8] {
                let nonzero = x.forma.fill_rule == ph2d_vec_scene::FillRule::NonZero;
                let (mut cobertas, mut hist) =
                    (0usize, std::collections::BTreeMap::<i32, usize>::new());
                let (mut misto, mut so_mais, mut so_menos, mut espurias) = (0, 0, 0, [0usize; 2]);
                let mut img_desacordo = std::collections::BTreeMap::<(i32, i32), usize>::new();
                let mut pred_def = std::collections::BTreeMap::<i32, usize>::new();
                let mut pred_desacordo = 0usize;
                let mut perto = vec![0usize; juntas.len()];
                let mut dist_def: Vec<f64> = Vec::new();
                for i in 0..gx {
                    for j in 0..gy {
                        let c = i * gy + j;
                        if borda[c] & modo != 0 {
                            continue;
                        }
                        let centro = [x0 + (i as f64 + 0.5) * cel, y0 + (j as f64 + 0.5) * cel];
                        let w = voltas(&desenho, centro);
                        let wi = voltas(&imagem, centro);
                        let pinta = if nonzero { w != 0 } else { w % 2 != 0 };
                        if wi != w {
                            *img_desacordo.entry((wi, w)).or_default() += 1;
                        }
                        let coberta = mais[c] + menos[c] > 0;
                        if !coberta {
                            if pinta {
                                espurias[usize::from(wi != 0)] += 1;
                            }
                            continue;
                        }
                        cobertas += 1;
                        *hist.entry(w).or_default() += 1;
                        if media[c].round() as i64 != i64::from(wi) {
                            pred_desacordo += 1;
                        }
                        if pinta {
                            continue;
                        }
                        *pred_def.entry(wi).or_default() += 1;
                        match (mais[c] > 0, menos[c] > 0) {
                            (true, true) => misto += 1,
                            (true, false) => so_mais += 1,
                            _ => so_menos += 1,
                        }
                        if let Some((m, dm)) = juntas
                            .iter()
                            .map(|jt| (jt[0] - centro[0]).hypot(jt[1] - centro[1]))
                            .enumerate()
                            .min_by(|a, b| a.1.total_cmp(&b.1))
                        {
                            perto[m] += 1;
                            dist_def.push(dm / t);
                        }
                    }
                }
                dist_def.sort_by(f64::total_cmp);
                let med = dist_def
                    .get(dist_def.len() / 2)
                    .copied()
                    .unwrap_or(f64::NAN);
                let maxd = dist_def.last().copied().unwrap_or(f64::NAN);
                println!(
                    "    [borda {}] cobertas {cobertas} · W {hist:?} · DEFEITOS {} (misto {misto}, só+ {so_mais}, só− {so_menos}) · pintadas sem amostra [W_img=0, W_img≠0] {espurias:?} · Σamostras≠W_img {pred_desacordo} · W_img nos defeitos {pred_def:?} · (W_img,W_desenho) em desacordo {img_desacordo:?} · defeitos por junta {perto:?} · dist/largura med {med:.2} max {maxd:.2} · juntas {juntas:?}",
                    if modo == 1 {
                        "só do desenho"
                    } else {
                        "desenho+imagem"
                    },
                    misto + so_mais + so_menos
                );
            }
        }
    }
}
