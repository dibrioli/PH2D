//! ⭐⭐⭐ **OS GATES DO G4 — e a sonda que mede o PRODUTO.**

use ph2d_mesh::Mesh;

/// ⭐⭐⭐ **SONDA — o COMPROMISSO do G3, medido no PRODUTO.**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     which_weight_gives_the_best_quads -- --ignored --nocapture
/// ```
///
/// ⭐ O G3 mostrou que fechar as costuras custa o alinhamento (`4,1° → 13,0°`). ⚠️ Qual
/// dos dois importa **ao produto** não se deduz: um mapa mal alinhado dá arcos que
/// serpenteiam nele, e um mapa com costuras frouxas dá dois lados a discordar. *A
/// escolha do `SEAM_WEIGHT` foi feita por raciocínio; esta sonda mede-a.*
#[test]
#[ignore = "sonda LENTA -- um solver completo por peso"]
fn which_weight_gives_the_best_quads() {
    for (name, mesh) in [
        ("ESFERA FINA", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("ESFERA LISA", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
    ] {
        eprintln!("── {name} ──");
        let (mesh, layout, cut, _, h) = upto_map(mesh);
        let dual = ph2d_crossfield::Dual::build(&mesh);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let l2 = ph2d_trace::trace_patches(&mesh, &dual, &field);
        let (combed, _) = crate::comb::comb_patches(&mesh, &l2, &cut);
        let target = ph2d_quadflow::edge_for_detail_with(
            &mesh,
            0.55,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let run = |arc_tau: &Vec<Vec<f32>>, rotulo: &str| {
            let mut l = layout.clone();
            l.arc_tau.clone_from(arc_tau);
            let Ok(spec) = l.to_layout(target) else {
                eprintln!("  {rotulo}: layout RECUSOU");
                return;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                eprintln!("  {rotulo}: quantizacao RECUSOU");
                return;
            };
            match ph2d_quadfill::fill(&mesh, &mesh, &l, &quant, ph2d_quadfill::SMOOTHING_ROUNDS) {
                Ok((_, r)) => eprintln!(
                    "  {rotulo}: ⭐enviesamento p50 {:>4.0}° p99 {:>4.0}° (>60°: {:>4}) \
                     | aspecto p50 {:.2} | dobras {}",
                    r.shape.skew_p50,
                    r.shape.skew_p99,
                    r.shape.skew_over_60,
                    r.shape.aspect_p50,
                    r.folded_local,
                ),
                Err(e) => eprintln!("  {rotulo}: montagem RECUSOU {e:?}"),
            }
        };
        run(&layout.arc_tau, "controlo      ");
        for weight in [1.0f32, 8.0, 64.0, 512.0] {
            let (map, sr) = crate::solve::solve_with(
                &mesh,
                &cut,
                &combed,
                crate::solve::Step::uniform(h),
                weight,
                160_000,
            );
            let (tau, mr) = super::arc_marks(&layout, &cut, &map);
            eprintln!(
                "  ── peso {weight:>5.0}: angulo p50 {:>4.1}° | costura max {:.4} | marcou {}/{} \
                 | desacordo max {:.4}",
                sr.angle_p50, sr.seam_max, mr.marked, mr.arcs, mr.disagree_max
            );
            run(&tau, &format!("  ⭐peso {weight:>5.0}  "));
        }
    }
}

/// A cadeia inteira até ao mapa global, sobre uma malha dada.
fn upto_map(
    mut mesh: Mesh,
) -> (
    Mesh,
    ph2d_trace::PatchLayout,
    crate::cut::CutMesh,
    crate::solve::GridMap,
    f32,
) {
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let pos = mesh.positions();
    let mut edges: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            edges.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    edges.sort_by(f32::total_cmp);
    let h = edges[edges.len() / 2];
    let (map, _) = crate::solve::solve(&mesh, &cut, &combed, h);
    (mesh, layout, cut, map, h)
}

/// ⭐⭐⭐ **A PROMESSA INTEIRA DO MAPA GLOBAL, medida: os dois lados concordam.**
///
/// ⛔⛔ **É este número que separa esta fase das seis curas locais.** Todas elas tinham
/// de *negociar* a marcação entre os dois patches que partilham um arco; aqui os dois
/// leem a **mesma função**, logo o desacordo tem de ser ruído.
///
/// ⚠️ **A barra é `2 %` e sai de MEDIÇÃO**, com o peso que shipa (`8`): o desacordo
/// máximo na esfera fina mede `0,0159`. ⛔ *Ela não é a barra do peso `512`, que dá
/// `0,009` — esse peso foi rejeitado porque o **produto** com ele é pior que o controlo
/// (`22°` contra `18°`), e o desacordo apertado não compra nada.*
///
/// ⚠️ Na esfera **grossa** o mesmo peso dá `0,378`, e isso não é uma reprovação desta
/// fase: ali o F1 **refina** em vez de grosseirar, e o `CLAUDE.md` §5 já marca essa rota
/// como partida por outros motivos.
#[test]
fn the_two_sides_of_an_arc_agree_on_where_the_marks_fall() {
    // ⭐ **A esfera FINA, que é a fixtura canónica desta linha.** ⚠️ A `24×36` é grossa,
    // logo o F1 **refina** em vez de grosseirar — uma rota que o `CLAUDE.md` §5 já marca
    // como separadamente partida (`aspecto 4,38`, `enviesamento 52°`). *Medir a promessa
    // do mapa global sobre ela seria medir o defeito de outra fase.*
    let (_, layout, cut, map, _) = upto_map(ph2d_mesh::shapes::uv_sphere(96, 144, 1.0));
    let (tau, rep) = super::arc_marks(&layout, &cut, &map);
    assert_eq!(
        tau.len(),
        layout.arc_tau.len(),
        "a contagem de arcos mudou: {rep:?}"
    );
    assert!(
        rep.marked > 0,
        "nenhum arco recebeu marcacao do mapa global: {rep:?}"
    );
    assert_eq!(
        rep.marked + rep.gave_up[4],
        rep.arcs,
        "marcou {} de {} arcos -- desistiu [sem costura {}, sem copia {}, percurso nulo {}, \
         tau degenerado {}, serpenteia {}]",
        rep.marked,
        rep.arcs,
        rep.gave_up[0],
        rep.gave_up[1],
        rep.gave_up[2],
        rep.gave_up[3],
        rep.gave_up[4]
    );
    assert!(
        rep.disagree_max < 0.02,
        "os dois lados de um arco discordam ate' {:.4} do comprimento dele (p50 {:.4}) -- \
         o mapa nao esta' a fechar, e sem isso esta fase nao entrega nada que as curas \
         locais nao entregassem",
        rep.disagree_max,
        rep.disagree_p50
    );
}

/// ⭐ **O TOTAL DE CADA ARCO NÃO MUDA, e o `τ` sai monótono.**
///
/// ⚠️ Mexer no total mudaria quantos segmentos o F4 dá a cada arco — outra experiência.
/// E um `τ` que recua faz a reamostragem devolver pontos fora de ordem.
#[test]
fn the_marks_keep_the_total_and_never_go_backwards() {
    let (_, layout, cut, map, _) = upto_map(ph2d_mesh::shapes::uv_sphere(24, 36, 1.0));
    let (tau, _) = super::arc_marks(&layout, &cut, &map);
    for (a, (novo, velho)) in tau.iter().zip(&layout.arc_tau).enumerate() {
        assert_eq!(novo.len(), velho.len(), "o arco {a} mudou de contagem");
        let (x, y) = (
            novo.last().copied().unwrap_or(0.0),
            velho.last().copied().unwrap_or(0.0),
        );
        assert!(
            (x - y).abs() <= y.abs() * 1.0e-4,
            "o arco {a} mudou de TOTAL ({y} -> {x})"
        );
        assert!(
            novo.windows(2).all(|w| w[1] >= w[0]),
            "o arco {a} saiu com `tau` a RECUAR"
        );
    }
}

/// ⭐⭐⭐ **SONDA — O PRODUTO.** A marcação do mapa global contra a de sempre.
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     what_does_the_global_map_do_to_the_quads -- --ignored --nocapture
/// ```
///
/// ⭐ **É o número que decide a semana.** Tudo o resto — ângulo, resíduo de costura,
/// desacordo entre lados — são proxies; *a forma dos quads é o produto*.
#[test]
#[ignore = "sonda -- o produto, com a cadeia inteira"]
fn what_does_the_global_map_do_to_the_quads() {
    for (name, mesh) in [
        ("ESFERA LISA", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
        ("ESFERA FINA", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
    ] {
        eprintln!("── {name} ──");
        let (mesh, layout, cut, map, h) = upto_map(mesh);
        let (tau, rep) = super::arc_marks(&layout, &cut, &map);
        eprintln!(
            "  h={h:.5} · marcou {}/{} arcos · desacordo entre lados p50 {:.5} max {:.5} \
             · monotonia forcada {}",
            rep.marked, rep.arcs, rep.disagree_p50, rep.disagree_max, rep.forced_monotone
        );
        for detail in [0.35f32, 0.55, 0.8] {
            for (rotulo, arc_tau) in [("controlo ", &layout.arc_tau), ("⭐GLOBAL  ", &tau)] {
                let mut l = layout.clone();
                l.arc_tau.clone_from(arc_tau);
                // ⭐ **A MESMA configuração das tabelas publicadas desta linha** — o alvo
                // sai de `edge_for_detail_with`, não de um número escrito aqui. ⚠️ *A
                // primeira versão usava `0,25` e dava `52°` onde as tabelas dizem `18°`:
                // medir noutra configuração e comparar com uma tabela é comparar nada.*
                let target = ph2d_quadflow::edge_for_detail_with(
                    &mesh,
                    detail,
                    ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
                );
                let Ok(spec) = l.to_layout(target) else {
                    eprintln!("  d={detail:.2} {rotulo}: o layout RECUSOU");
                    continue;
                };
                let Ok((quant, _)) =
                    ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
                else {
                    eprintln!("  d={detail:.2} {rotulo}: a quantizacao RECUSOU");
                    continue;
                };
                match ph2d_quadfill::fill(&mesh, &mesh, &l, &quant, ph2d_quadfill::SMOOTHING_ROUNDS)
                {
                    Ok((out, r)) => eprintln!(
                        "  d={detail:.2} {rotulo}: {} quads | ⭐enviesamento p50 {:>4.0}° p99 {:>4.0}° (>60°: {}) \
                     | aspecto p50 {:.2} | dobras {} | DOMINIO rect {:.1}° leque {:.1}°",
                        out.faces().len(),
                        r.shape.skew_p50,
                        r.shape.skew_p99,
                        r.shape.skew_over_60,
                        r.shape.aspect_p50,
                        r.folded_local,
                        r.domain_skew.0,
                        r.domain_skew.1,
                    ),
                    Err(e) => eprintln!("  d={detail:.2} {rotulo}: a montagem RECUSOU {e:?}"),
                }
            }
        }
    }
}

/// ⭐⭐⭐ **SONDA — que arcos discordam, e o que têm em comum?**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     which_arcs_disagree -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- diagnostico dos arcos que discordam"]
fn which_arcs_disagree() {
    let (_, layout, cut, map, _) = upto_map(ph2d_mesh::shapes::uv_sphere(24, 36, 1.0));
    let mut by_arc: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for (s, seam) in cut.seams.iter().enumerate() {
        if let Some(a) = seam.arc {
            by_arc.insert(a, s);
        }
    }
    let (_, rep) = super::arc_marks(&layout, &cut, &map);
    eprintln!(
        "  {} arcos, marcou {}, desacordo p50 {:.5} max {:.5}, monotonia forcada {}",
        rep.arcs, rep.marked, rep.disagree_p50, rep.disagree_max, rep.forced_monotone
    );
    for (a, old) in layout.arc_tau.iter().enumerate() {
        let Ok(aid) = u32::try_from(a) else { continue };
        let Some(&s) = by_arc.get(&aid) else { continue };
        let seam = &cut.seams[s];
        let mut info: Vec<(usize, f32, f32, usize)> = Vec::new();
        let mut fs: Vec<Vec<f32>> = Vec::new();
        for side in &seam.side {
            let p = side.patch as usize;
            let z: Vec<[f32; 2]> = side
                .local
                .iter()
                .filter_map(|l| l.and_then(|l| map.uv.get(p).and_then(|u| u.get(l as usize))))
                .copied()
                .collect();
            if z.len() != old.len() {
                continue;
            }
            let d = [
                (z[z.len() - 1][0] - z[0][0]).abs(),
                (z[z.len() - 1][1] - z[0][1]).abs(),
            ];
            let axis = usize::from(d[1] > d[0]);
            let back = z
                .windows(2)
                .filter(|w| (w[1][axis] - w[0][axis]).signum() != (d[axis]).signum())
                .count();
            info.push((axis, d[0], d[1], back));
            if let Some(f) = super::along(&z) {
                fs.push(f);
            }
        }
        if fs.len() != 2 {
            continue;
        }
        let worst = fs[0]
            .iter()
            .zip(&fs[1])
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        // ⭐ A RECTIDAO: quanto o arco ANDA no mapa contra quanto ele PERCORRE.
        let straight = |p: usize| -> f32 {
            let side = &seam.side[p];
            let z: Vec<[f32; 2]> = side
                .local
                .iter()
                .filter_map(|l| {
                    l.and_then(|l| {
                        map.uv
                            .get(side.patch as usize)
                            .and_then(|u| u.get(l as usize))
                    })
                })
                .copied()
                .collect();
            if z.len() < 2 {
                return 0.0;
            }
            let d = [z[z.len() - 1][0] - z[0][0], z[z.len() - 1][1] - z[0][1]];
            let disp = d[0].mul_add(d[0], d[1] * d[1]).sqrt();
            let walk: f32 = z
                .windows(2)
                .map(|w| {
                    let e = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
                    e[0].mul_add(e[0], e[1] * e[1]).sqrt()
                })
                .sum();
            if walk < 1.0e-12 { 0.0 } else { disp / walk }
        };
        eprintln!(
            "     arco {a:>3}: desacordo {worst:.4} rectidao {:.3}/{:.3}",
            straight(0),
            straight(1)
        );
        if worst > 0.01 {
            eprintln!(
                "  ⛔ arco {a} ({} pontos): desacordo {worst:.4} | lado0 eixo {} percurso \
                 ({:.3},{:.3}) recuos {} | lado1 eixo {} percurso ({:.3},{:.3}) recuos {}",
                old.len(),
                info[0].0,
                info[0].1,
                info[0].2,
                info[0].3,
                info[1].0,
                info[1].1,
                info[1].2,
                info[1].3,
            );
        }
    }
}

/// ⭐⭐⭐ **SONDA — a FRACÇÃO ALCANÇÁVEL de curar o leque.**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     how_much_can_fixing_the_fan_possibly_buy -- --ignored --nocapture
/// ```
///
/// ⛔⛔ **Antes de construir a cura do leque, medir quanto ela pode valer no MÁXIMO.**
/// Se as faces vindas de patches de quatro lados — que não têm leque nenhum — já
/// medirem quase o mesmo que as de leque, então curar o leque não pode levar `18°` a
/// `6°`, e a obra é outra. *É a lei da cura medida numa fixtura que não contém o
/// fenómeno, um nível acima: medir a fracção alcançável ANTES do resultado.*
#[test]
#[ignore = "sonda -- quanto vale curar o leque, no maximo"]
fn how_much_can_fixing_the_fan_possibly_buy() {
    for (name, mesh) in [
        ("ESFERA FINA", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("ESFERA LISA", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
    ] {
        let (mesh, layout, _, _, _) = upto_map(mesh);
        for detail in [0.55f32, 0.8] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &mesh,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let Ok(spec) = layout.to_layout(target) else {
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                continue;
            };
            let Ok((_, r)) = ph2d_quadfill::fill(
                &mesh,
                &mesh,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) else {
                continue;
            };
            eprintln!(
                "  {name} d={detail:.2}: p50 GLOBAL {:>4.0}° | ⭐rectangulo {:>5.1}° LEQUE {:>5.1}° \
                 | por origem {:?}",
                r.shape.skew_p50,
                r.skew_by_fan.0,
                r.skew_by_fan.1,
                ph2d_quadfill::Provenance::NAMES
                    .iter()
                    .zip(r.skew_prov)
                    .map(|(n, v)| format!("{n} {v:.0}°"))
                    .collect::<Vec<_>>()
            );
        }
    }
}
