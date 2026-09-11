//! **SONDA — de que ESPÉCIE é a não-variedade que o F3 denunciou?**
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-remesh-iso --release \
//!     --test manifold_probe -- --ignored --nocapture
//! ```
//!
//! ⚠️ *"Anel aberto"* é o SINTOMA e ele tem três causas distintas, que pedem
//! curas distintas: aresta de **borda** (uma face só), aresta **não-variedade**
//! (três ou mais), e aresta dirigida **repetida** (duas faces a percorrerem a
//! mesma aresta no mesmo sentido — orientação inconsistente ou face duplicada).
//! Esta sonda separa as três, e mede em que RODADA cada uma nasce.

use std::collections::BTreeMap;

use ph2d_mesh::{Mesh, shapes};
use ph2d_remesh_iso::{ALPHA, remesh_isotropic};

/// Os três números que separam as causas.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Diag {
    border: usize,
    nonmanifold: usize,
    repeated: usize,
    open_rings: usize,
}

fn diagnose(mesh: &Mesh) -> Diag {
    let mut undirected: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    let mut directed: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *directed.entry((a, b)).or_default() += 1;
            *undirected.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let mut d = Diag {
        border: undirected.values().filter(|&&n| n == 1).count(),
        nonmanifold: undirected.values().filter(|&&n| n > 2).count(),
        repeated: directed.values().filter(|&&n| n > 1).count(),
        open_rings: 0,
    };
    // O anel, exatamente como o F3 o constrói.
    let mut half: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            half.insert((v[k], v[(k + 1) % v.len()]), u32::try_from(fi).unwrap());
        }
    }
    for v in 0..mesh.vert_count() {
        if ph2d_trace_ring(mesh, &half, u32::try_from(v).unwrap()).is_none() {
            d.open_rings += 1;
        }
    }
    d
}

/// Uma cópia local do pivô do F3 — a sonda não pode depender da crate que ela
/// está a diagnosticar (`ph2d-trace` depende do campo, que depende disto).
fn ph2d_trace_ring(mesh: &Mesh, half: &BTreeMap<(u32, u32), u32>, v: u32) -> Option<()> {
    let faces = mesh.faces();
    let incident = mesh.adjacency().vert_faces.neighbours(v as usize);
    let first = *incident.first()?;
    let mut f = first;
    let mut n = 0usize;
    for _ in 0..incident.len() {
        let t = faces.get(f as usize)?.verts();
        let k = t.iter().position(|&x| x == v)?;
        let c = t[(k + 1) % t.len()];
        n += 1;
        f = *half.get(&(c, v))?;
        if f == first {
            break;
        }
    }
    if n != incident.len() || f != first {
        return None;
    }
    Some(())
}

fn report(name: &str, mesh: &Mesh) {
    let d = diagnose(mesh);
    println!(
        "{name:<28} v {:<7} f {:<7} | borda {:<4} NAO-VARIEDADE {:<4} dirigida-repetida {:<4} \
         aneis-abertos {:<4}",
        mesh.vert_count(),
        mesh.face_count(),
        d.border,
        d.nonmanifold,
        d.repeated,
        d.open_rings
    );
}

/// ⚠️ **A réplica do laço do passe**, com as MESMAS constantes — é a única forma
/// de perguntar *"qual das três operações a criou"* sem tornar público um detalhe
/// interno. O controle positivo é a linha `REPLICA`: se ela não bate com o passe
/// de verdade, a resposta desta sonda não vale nada.
fn per_operation(name: &str, mesh: &mut Mesh) {
    mesh.triangulate();
    let reference = mesh.clone();
    let target = ph2d_remesh_iso::target_edge(mesh, ALPHA);
    let (mut scratch, mut births, mut remap) = (
        ph2d_mesh::RegionScratch::default(),
        Vec::<ph2d_mesh::Birth>::new(),
        ph2d_mesh::Remap::default(),
    );
    let mut last = usize::MAX;
    let mut prev = Diag::default();
    for round in 1..=ph2d_remesh_iso::MAX_ROUNDS {
        for (op, run) in [
            ("PARTIR  ", 0u8),
            ("COLAPSAR", 1),
            ("FLIPAR  ", 2),
            ("ALISAR  ", 3),
        ] {
            let (centre, radius) = whole(mesh);
            match run {
                0 => {
                    ph2d_mesh::refine_in_sphere(
                        mesh,
                        centre,
                        radius,
                        target * 4.0 / 3.0,
                        &mut births,
                        &mut scratch,
                    );
                }
                1 => {
                    ph2d_mesh::collapse_in_sphere(
                        mesh,
                        centre,
                        radius,
                        target * 4.0 / 5.0,
                        &mut remap,
                        &mut scratch,
                    );
                }
                2 => {
                    ph2d_mesh::relax_valence(mesh, &mut scratch);
                }
                _ => relax_like_the_pass(mesh, &reference, target),
            }
            let d = diagnose(mesh);
            if d != prev && (d.nonmanifold > 0 || d.repeated > 0 || d.border > 0) {
                println!(
                    "  {name} r{round} apos {op}: borda {} NAO-VAR {} dirigida-repetida {} \
                     (era {} / {} / {})",
                    d.border,
                    d.nonmanifold,
                    d.repeated,
                    prev.border,
                    prev.nonmanifold,
                    prev.repeated
                );
            }
            prev = d;
        }
        let now = mesh.vert_count();
        #[allow(clippy::cast_precision_loss)]
        if last != usize::MAX && (now as f32 - last as f32).abs() <= 0.01 * last as f32 {
            break;
        }
        last = now;
    }
    report(&format!("{name} REPLICA"), mesh);
}

/// A esfera que cobre a malha — a mesma lei do passe.
fn whole(mesh: &Mesh) -> ([f32; 3], f32) {
    let b = mesh.bounds();
    let c = [
        (b.min[0] + b.max[0]) * 0.5,
        (b.min[1] + b.max[1]) * 0.5,
        (b.min[2] + b.max[2]) * 0.5,
    ];
    let d = [b.max[0] - c[0], b.max[1] - c[1], b.max[2] - c[2]];
    let r = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    (c, r * 1.5 + 1.0e-4)
}

/// O alisamento não muda topologia — basta reprojetar para a réplica seguir a
/// mesma trajetória geométrica.
fn relax_like_the_pass(mesh: &mut Mesh, reference: &Mesh, target: f32) {
    let n = mesh.vert_count();
    let mut moved = Vec::with_capacity(n);
    for v in 0..n {
        let mut sum = [0.0f32; 3];
        let mut k = 0.0f32;
        for &w in mesh.adjacency().vert_verts.neighbours(v) {
            for (acc, q) in sum.iter_mut().zip(mesh.positions()[w as usize]) {
                *acc += q;
            }
            k += 1.0;
        }
        let p = mesh.positions()[v];
        if k < 1.0 {
            moved.push(p);
            continue;
        }
        let avg = [sum[0] / k, sum[1] / k, sum[2] / k];
        let mid = [
            p[0] + 0.5 * (avg[0] - p[0]),
            p[1] + 0.5 * (avg[1] - p[1]),
            p[2] + 0.5 * (avg[2] - p[2]),
        ];
        moved.push(ph2d_remesh_iso::project_onto(reference, mid, target * 4.0));
    }
    mesh.positions_mut().copy_from_slice(&moved);
}

/// As arestas não-variedade, com quantas faces cada uma tem.
fn offenders(mesh: &Mesh) -> BTreeMap<(u32, u32), usize> {
    let mut e: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *e.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    e.retain(|_, n| *n > 2);
    e
}

#[test]
#[ignore = "sonda -- o MECANISMO: uma aresta com QUATRO faces e' a mesma diagonal criada duas vezes"]
fn the_offending_edge_is_a_diagonal_created_twice() {
    // ⭐ **A pergunta decisiva.** Se o flip criasse a diagonal por cima de uma
    // que já existia, a aresta ofensora teria **três** faces (a velha + as duas
    // novas). Se dois flips DISTINTOS da mesma rodada criassem a mesma diagonal,
    // ela teria **quatro** — e nenhum dos dois a veria na adjacência de ENTRADA.
    let mut mesh = shapes::uv_sphere_shuffled(96, 144, 1.0);
    mesh.triangulate();
    let before = offenders(&mesh);
    assert!(before.is_empty(), "a entrada tem de chegar limpa");
    let mut edges_before: std::collections::BTreeSet<(u32, u32)> =
        std::collections::BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            edges_before.insert((a.min(b), a.max(b)));
        }
    }
    let mut scratch = ph2d_mesh::RegionScratch::default();
    let flips = ph2d_mesh::relax_valence(&mut mesh, &mut scratch);
    let after = offenders(&mesh);
    println!(
        "um relax_valence: {flips} trocas, {} arestas ofensoras",
        after.len()
    );
    for (e, n) in &after {
        println!(
            "  aresta {e:?}: {n} faces | ja' existia antes? {}",
            edges_before.contains(e)
        );
    }
}

#[test]
#[ignore = "sonda -- localiza a OPERACAO que cria a nao-variedade"]
fn which_operation_creates_it() {
    per_operation("cube", &mut shapes::cube(1.0));
    per_operation(
        "sphere_shuffled",
        &mut shapes::uv_sphere_shuffled(96, 144, 1.0),
    );
}

#[test]
#[ignore = "sonda -- classifica a nao-variedade, nao afirma um limite"]
fn which_kind_of_non_manifold_does_the_remesh_produce() {
    for (name, mut mesh) in [
        ("cube", shapes::cube(1.0)),
        ("sphere_uv_96x144", shapes::uv_sphere(96, 144, 1.0)),
        ("torus_64x32", shapes::torus(64, 32, 1.0, 0.35)),
        ("sphere_shuffled", shapes::uv_sphere_shuffled(96, 144, 1.0)),
    ] {
        mesh.triangulate();
        report(&format!("{name} ANTES"), &mesh);
        let r = remesh_isotropic(&mut mesh, ALPHA);
        report(&format!("{name} DEPOIS ({} rodadas)", r.rounds), &mesh);
    }
}
