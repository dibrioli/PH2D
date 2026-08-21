//! **SONDA — de que ESPÉCIE são os rasgos que o Enio fotografou?**
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
//!     --test artifacts -- --ignored --nocapture
//! ```
//!
//! ⭐ **A foto de 2026-08-21 mostra uma grade BOA com fendas escuras** ao longo do
//! vinco de uma orelha esculpida. E o relatório diz `casca FECHADA` — zero arestas
//! de bordo, característica de Euler exacta. As duas coisas são verdade, então a
//! fenda **não é um buraco topológico**.
//!
//! Restam três espécies, e elas pedem curas diferentes:
//!
//! | espécie | assinatura | cura |
//! |---|---|---|
//! | **face DEGENERADA** | área ≈ 0, ou razão de aspecto enorme | amostragem |
//! | **face DOBRADA** | normal a apontar contra a superfície por baixo | projeção |
//! | **salto de PROJEÇÃO** | dois vértices vizinhos na grade a aterrar em bancos opostos de um vinco côncavo | *feature lines* |
//!
//! ⚠️ **A terceira é a hipótese de trabalho**, e o mecanismo é conhecido na
//! literatura: o ponto mais próximo de uma superfície é **descontínuo** num vinco
//! côncavo. O interior de um patch sai por Coons — uma interpolação que **corta** o
//! vinco — e a reprojecção agarra cada ponto ao banco mais próximo. Dois vizinhos
//! da grade caem em bancos opostos, e o quad entre eles atravessa a fenda.
//!
//! ⛔ **E nenhuma das três é vista pelas réguas que existem hoje**: comprimento de
//! aresta mediano/máximo não muda quando uma face dobra sobre si mesma.

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill};
use ph2d_quantize::{Budget, quantize_within};
use ph2d_trace::trace_patches;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}
fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// A área e a razão de aspecto de um quad, pelos dois triângulos.
fn area_and_aspect(p: &[[f32; 3]], v: &[u32]) -> (f32, f32) {
    let mut area = 0.0f32;
    for k in 1..v.len() - 1 {
        let (a, b, c) = (p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
        area += 0.5 * norm(cross(sub(b, a), sub(c, a)));
    }
    let mut lo = f32::INFINITY;
    let mut hi = 0.0f32;
    for k in 0..v.len() {
        let l = norm(sub(p[v[(k + 1) % v.len()] as usize], p[v[k] as usize]));
        lo = lo.min(l);
        hi = hi.max(l);
    }
    (area, if lo > 1.0e-9 { hi / lo } else { f32::INFINITY })
}

/// A normal de uma face, pelo primeiro triângulo.
fn face_normal(p: &[[f32; 3]], v: &[u32]) -> [f32; 3] {
    let (a, b, c) = (p[v[0] as usize], p[v[1] as usize], p[v[2] as usize]);
    cross(sub(b, a), sub(c, a))
}

fn probe(name: &str, mesh: Mesh, detail_edge: f32, smoothing: usize) {
    let reference = mesh.clone();
    let mut work = mesh;
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = Dual::build(&work);
    let (field, _) = solve_miq(&dual);
    let layout = trace_patches(&work, &dual, &field);
    let Ok(spec) = layout.to_layout(detail_edge) else {
        println!("{name:<20} layout invalido");
        return;
    };
    let Ok((quant, _)) = quantize_within(&spec, Budget::new(256, 512)) else {
        println!("{name:<20} nao quantiza");
        return;
    };
    // ⭐ **O ALVO de cada arco contra o que o F4 lhe deu.** Um arco longo
    // quantizado em poucos segmentos produz uma aresta gigante, e a grade de Coons
    // ao lado dela estica — que é o candidato a explicar `max 1,10` numa esfera de
    // raio 1,0 com mediana 0,047.
    let mut worst: Vec<(f32, usize, u32, f32)> = layout
        .arc_length
        .iter()
        .enumerate()
        .map(|(a, &l)| {
            let want = l / detail_edge;
            let got = quant.arc[a];
            #[allow(clippy::cast_precision_loss)]
            (want / (got as f32).max(1.0), a, got, l)
        })
        .collect();
    worst.sort_by(|x, y| y.0.total_cmp(&x.0));
    if smoothing == 0 {
        let over = worst.iter().filter(|w| w.0 > 2.0).count();
        println!(
            "  [arcos] {} de {} com o alvo > 2x o que o F4 deu | os 5 piores: {}",
            over,
            worst.len(),
            worst
                .iter()
                .take(5)
                .map(|(r, a, g, l)| format!(
                    "#{a}: pedia {:.1} deu {g} (len {l:.3}, {r:.1}x)",
                    l / detail_edge
                ))
                .collect::<Vec<_>>()
                .join(" · ")
        );
    }
    let Ok((out, r)) = fill(&work, &reference, &layout, &quant, smoothing) else {
        println!("{name:<20} a montagem recusou");
        return;
    };

    let pos = out.positions();
    let mut slivers = 0usize;
    let mut worst_aspect = 0.0f32;
    let mut folded = 0usize;
    // ⚠️ **A normal da referência no ponto mais próximo do centróide** — é contra
    // ela que se pergunta se a face dobrou. `face_normals` da saída sozinha não
    // responde: uma malha inteiramente ao contrário é consistente consigo mesma.
    let ref_normals = reference.face_normals();
    let mut hits = Vec::new();
    let rb = reference.bounds();
    let seed = norm(sub(rb.max, rb.min)) * 0.05;
    for f in out.faces() {
        let v = f.verts();
        let (_area, aspect) = area_and_aspect(pos, v);
        if aspect > 8.0 {
            slivers += 1;
        }
        worst_aspect = worst_aspect.max(aspect);

        let mut c = [0.0f32; 3];
        for &i in v {
            let q = pos[i as usize];
            for k in 0..3 {
                c[k] += q[k] / v.len() as f32;
            }
        }
        let mut best = (f32::INFINITY, usize::MAX);
        let mut radius = seed;
        while best.1 == usize::MAX && radius < seed * 64.0 {
            reference.octree().faces_in_sphere(c, radius, &mut hits);
            for &fi in &hits {
                let rv = reference.faces()[fi as usize].verts();
                let mut rc = [0.0f32; 3];
                for &i in rv {
                    let q = reference.positions()[i as usize];
                    for k in 0..3 {
                        rc[k] += q[k] / rv.len() as f32;
                    }
                }
                let d = norm(sub(rc, c));
                if d < best.0 {
                    best = (d, fi as usize);
                }
            }
            radius *= 2.0;
        }
        if best.1 != usize::MAX {
            let n = face_normal(pos, v);
            if dot(n, ref_normals[best.1]) < 0.0 {
                folded += 1;
            }
        }
    }
    println!(
        "{name:<24} alis={smoothing:<2} {:<5} quads | IRREG {:<5} LASCAS {slivers:<4} \
         DOBRADAS {folded:<4} RADIAL {:<4} | med {:.3} max {:.3}",
        r.quads,
        r.irregular,
        inward_faces(&out),
        r.edge_median,
        r.edge_max
    );
}

/// Conta as faces cuja normal aponta para DENTRO, num sólido estrelado.
///
/// ⭐ **É o controle do detector, e ele é inequívoco numa esfera:** a normal de
/// toda face tem de concordar com o raio. *Uma régua que julga dobras tem de provar
/// primeiro que não acusa uma malha que não tem nenhuma.*
fn inward_faces(mesh: &Mesh) -> usize {
    let pos = mesh.positions();
    let b = mesh.bounds();
    let c = [
        (b.min[0] + b.max[0]) * 0.5,
        (b.min[1] + b.max[1]) * 0.5,
        (b.min[2] + b.max[2]) * 0.5,
    ];
    mesh.faces()
        .iter()
        .filter(|f| {
            let v = f.verts();
            let mut m = [0.0f32; 3];
            #[allow(clippy::cast_precision_loss)]
            for &i in v {
                let q = pos[i as usize];
                for k in 0..3 {
                    m[k] += q[k] / v.len() as f32;
                }
            }
            dot(face_normal(pos, v), sub(m, c)) < 0.0
        })
        .count()
}

#[test]
#[ignore = "sonda -- o CONTROLE do detector de dobra, antes de se acreditar nele"]
fn the_fold_detector_does_not_accuse_a_mesh_without_folds() {
    for (name, mesh) in [
        ("esfera lisa CRUA", shapes::uv_sphere(96, 144, 1.0)),
        ("esfera esculpida CRUA", shapes::sculpt_sphere(1.0)),
    ] {
        let mut iso = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut iso, ph2d_remesh_iso::ALPHA);
        println!(
            "{name:<24} crua: {} faces para dentro de {} | apos F1: {} de {}",
            inward_faces(&mesh),
            mesh.face_count(),
            inward_faces(&iso),
            iso.face_count()
        );
    }
}

#[test]
#[ignore = "sonda -- classifica os artefactos, nao afirma um limite"]
fn what_kind_of_artefact_survives_the_chain() {
    // ⭐ **A varredura do ALISAMENTO isola a fase.** Se as dobras nascem da
    // construção (leque + Coons), elas já estão lá com zero rondas; se nascem do
    // Laplaciano, elas crescem com o número de rondas.
    for smoothing in [0usize, 1, 3, 6, 12] {
        probe(
            "esfera lisa",
            shapes::uv_sphere(96, 144, 1.0),
            0.05,
            smoothing,
        );
    }
    println!("──");
    for smoothing in [0usize, 6] {
        probe(
            "esfera esculpida",
            shapes::sculpt_sphere(1.0),
            0.05,
            smoothing,
        );
        probe(
            "esfera ruidosa",
            shapes::uv_sphere_noisy(96, 144, 1.0, 0.02),
            0.05,
            smoothing,
        );
    }
    let _ = SMOOTHING_ROUNDS;
}

/// ⭐ **A VARREDURA DAS LEIS DE PESO** — o custo de um arco desviado do alvo.
///
/// ⚠️ **O peso decide o que o solver sacrifica**, e a primeira escolha (uniforme)
/// deixava-o indiferente entre esmagar um arco que pedia 24 segmentos e esticar
/// vinte curtos em 1. O ótimo escolhia a primeira, e a malha saía com uma aresta de
/// 6× o alvo.
///
/// ⛔ E a correção "óbvia" — deviação RELATIVA, `|x−t|/t` — tem o incentivo ao
/// contrário: ela torna esmagar um arco LONGO ainda mais barato (custo ≤ 1) e
/// esticar um CURTO caríssimo. *Medir as quatro é mais barato que raciocinar sobre
/// uma.*
#[test]
#[ignore = "sonda -- qual lei de peso protege a grade"]
fn which_arc_weight_law_protects_the_grid() {
    for (name, mesh) in [
        ("esfera lisa", shapes::uv_sphere(96, 144, 1.0)),
        ("esfera esculpida", shapes::sculpt_sphere(1.0)),
    ] {
        let target = 0.05f32;
        let reference = mesh.clone();
        let mut work = mesh;
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq(&dual);
        let layout = trace_patches(&work, &dual, &field);
        let sides = layout.sides();
        println!("── {name} ──");
        for (law, w) in [
            ("uniforme  |x-t|", 0.0f32),
            ("relativa  /t", -1.0),
            ("proporcional *t", 1.0),
            ("raiz  *sqrt(t)", 0.5),
        ] {
            let arcs: Vec<ph2d_quantize::ArcSpec> = layout
                .arc_length
                .iter()
                .map(|&l| {
                    let t = f64::from(l / target);
                    let mut a = ph2d_quantize::ArcSpec::new(t);
                    a.weight = f64::from(t.max(1.0) as f32).powf(f64::from(w));
                    a
                })
                .collect();
            let patches = sides
                .iter()
                .cloned()
                .map(|s| ph2d_quantize::PatchSpec { sides: s })
                .collect();
            let Ok(spec) = ph2d_quantize::Layout::new(arcs, patches) else {
                println!("  {law:<18} layout invalido");
                continue;
            };
            let Ok((quant, _)) = quantize_within(&spec, Budget::new(256, 512)) else {
                println!("  {law:<18} nao quantiza");
                continue;
            };
            let Ok((out, r)) = fill(&work, &reference, &layout, &quant, SMOOTHING_ROUNDS) else {
                println!("  {law:<18} a montagem recusou");
                continue;
            };
            #[allow(clippy::cast_precision_loss)]
            let worst = layout
                .arc_length
                .iter()
                .enumerate()
                .map(|(a, &l)| (l / target) / (quant.arc[a] as f32).max(1.0))
                .fold(0.0f32, f32::max);
            println!(
                "  {law:<18} {:<5} quads | DOBRADAS {:<4} | pior arco {worst:>5.1}x | \
                 med {:.3} max {:.3}",
                r.quads,
                inward_faces(&out),
                r.edge_median,
                r.edge_max
            );
        }
    }
}

/// ⭐ **A DOBRA CONTRA A RAZÃO grão-de-quad ÷ grão-da-REFERÊNCIA.**
///
/// ⚠️ **É o eixo que nunca foi testado, e ele explode.** A reprojecção agarra cada
/// ponto à FACETA mais próxima da malha de referência; quando o quad fica do
/// tamanho da faceta, dois vizinhos da grade aterram em facetas com normais
/// diferentes e o quad entre eles vira. *A referência não é uma superfície lisa: é
/// um poliedro.*
#[test]
#[ignore = "sonda -- a dobra contra o grao da referencia"]
fn how_fine_may_the_quad_be_against_the_reference_facet() {
    for (rings, segs) in [(48usize, 72usize), (96, 144), (192, 288)] {
        let reference = shapes::uv_sphere(rings, segs, 1.0);
        // A aresta média da referência: o "grão" dela.
        let rp = reference.positions();
        let mut n = 0.0f64;
        let mut sum = 0.0f64;
        for f in reference.faces() {
            let v = f.verts();
            for k in 0..v.len() {
                sum += f64::from(norm(sub(
                    rp[v[(k + 1) % v.len()] as usize],
                    rp[v[k] as usize],
                )));
                n += 1.0;
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        let facet = (sum / n) as f32;
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq(&dual);
        let layout = trace_patches(&work, &dual, &field);
        for target in [0.06f32] {
            let Ok(spec) = layout.to_layout(target) else {
                continue;
            };
            let Ok((quant, _)) = quantize_within(&spec, Budget::new(256, 512)) else {
                continue;
            };
            // ⭐ **O EXPERIMENTO CONTROLADO: o MESMO layout, a MESMA quantização,
            // e só a superfície de projeção muda.** É a única forma de separar
            // "o layout é mau" de "a projeção é má", porque a referência decide as
            // duas coisas ao mesmo tempo (ela alimenta o F1).
            let Ok((out, r)) = fill(&work, &reference, &layout, &quant, SMOOTHING_ROUNDS) else {
                continue;
            };
            // ⭐ A varredura do alisamento sobre a configuração RUIM.
            for rounds in [6usize, 12, 24, 48, 96] {
                let Ok((a, ra)) = fill(&work, &reference, &layout, &quant, rounds) else {
                    continue;
                };
                #[allow(clippy::cast_precision_loss)]
                let p = 100.0 * inward_faces(&a) as f64 / ra.quads.max(1) as f64;
                println!(
                    "    ref {rings}x{segs} alis={rounds:<3} DOBRADAS {p:.1} % | med {:.3} max {:.3}",
                    ra.edge_median, ra.edge_max
                );
            }
            let Ok((alt, _)) = fill(&work, &work, &layout, &quant, SMOOTHING_ROUNDS) else {
                continue;
            };
            #[allow(clippy::cast_precision_loss)]
            let pct = 100.0 * inward_faces(&out) as f64 / r.quads.max(1) as f64;
            #[allow(clippy::cast_precision_loss)]
            let pct_alt = 100.0 * inward_faces(&alt) as f64 / r.quads.max(1) as f64;
            println!(
                "  ref {rings}x{segs} (faceta {facet:.3}) alvo {target:.3} quad/faceta \
                 {:.2}x | {:<5} quads | DOBRADAS: sobre a REFERENCIA {pct:.1} % · \
                 sobre a ISO {pct_alt:.1} %",
                target / facet,
                r.quads
            );
        }
        println!("  ──");
    }
}
