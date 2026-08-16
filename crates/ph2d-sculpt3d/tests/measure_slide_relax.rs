//! **O QUE O RELAX FAZ, E O QUE ELE NÃO FAZ** — a sonda que separa
//! *redistribuir a malha* de *mudar a forma*, e que decide as barras dos gates.
//!
//! ⚠️ **O [`Verb::Smooth`] é o CONTROLE, e ele é obrigatório aqui:** os dois
//! verbos caminham para a MESMA média do anel, e a única linha que os separa é a
//! subtração da componente normal. Uma sonda que medisse só o relax diria
//! *"vértices andaram e o raio ficou"* — que é verdade também para um verbo que
//! não faz nada de útil (`w = 0`). O que prova a ferramenta é a **razão** entre
//! as duas colunas.
//!
//! ⚠️ **E o oráculo é a GEOMETRIA:** o raio é lido de volta dos vértices, e a
//! uniformidade das arestas também. Nada aqui cita a constante do kernel.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_slide_relax -- --ignored --nocapture`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// ⚠️ **A fixture mora na `ph2d-mesh`, não aqui** — o gate do verbo precisa dela
/// também, e um `tests/` não alcança um `#[cfg(test)]` da crate. O doc dela
/// carrega a medição que a torna necessária (a esfera lisa não contém o
/// fenômeno) e o gate irmão afirma as duas metades: forma exacta, grade torta.
fn sphere_shuffled() -> Mesh {
    ph2d_mesh::shapes::uv_sphere_shuffled(48, 72, 1.0)
}

/// O polo `+z`; o olho olha para `−z`.
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.45;

/// Um traço PARADO no polo — o relax não precisa de percurso para redistribuir,
/// e um traço parado isola a lei do verbo do transporte do caminho.
fn hold(verb: Verb, dabs: usize, strength: f32) -> Mesh {
    let mut mesh = sphere_shuffled();
    let b = Brush {
        verb,
        radius: R,
        strength,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for _ in 0..dabs {
        let d = Dab::pulling(TIP, R, EYE, [0.0; 3]);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    mesh
}

/// Os índices dentro da pegada — quem o traço de facto tocou.
fn footprint(mesh: &Mesh) -> Vec<usize> {
    let r2 = R * R;
    mesh.positions()
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let d = [p[0] - TIP[0], p[1] - TIP[1]];
            p[2] > 0.0 && d[0] * d[0] + d[1] * d[1] <= r2
        })
        .map(|(i, _)| i)
        .collect()
}

/// **A FORMA** — o pior desvio do raio 1, sobre a pegada.
fn radius_drift(mesh: &Mesh, idx: &[usize]) -> f64 {
    idx.iter()
        .map(|&i| {
            let p = mesh.positions()[i];
            let r = f64::from(p[0])
                .hypot(f64::from(p[1]))
                .hypot(f64::from(p[2]));
            (r - 1.0).abs()
        })
        .fold(0.0f64, f64::max)
}

/// **A REDISTRIBUIÇÃO** — o coeficiente de variação do comprimento das arestas
/// da pegada. Quanto menor, mais uniforme é a malha.
fn edge_cv(mesh: &Mesh, idx: &[usize]) -> f64 {
    let inside: std::collections::BTreeSet<usize> = idx.iter().copied().collect();
    let adj = mesh.adjacency();
    let mut lens = Vec::new();
    for &i in idx {
        for &nb in adj.vert_verts.neighbours(i) {
            if !inside.contains(&(nb as usize)) || (nb as usize) < i {
                continue;
            }
            let a = mesh.positions()[i];
            let b = mesh.positions()[nb as usize];
            let d = [
                f64::from(b[0] - a[0]),
                f64::from(b[1] - a[1]),
                f64::from(b[2] - a[2]),
            ];
            lens.push((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
    }
    let n = lens.len() as f64;
    let mean = lens.iter().sum::<f64>() / n;
    let var = lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f64>() / n;
    var.sqrt() / mean
}

/// Quanto o vértice mais deslocado andou.
fn moved(a: &Mesh, b: &Mesh, idx: &[usize]) -> f64 {
    idx.iter()
        .map(|&i| {
            let p = a.positions()[i];
            let q = b.positions()[i];
            f64::from(q[0] - p[0])
                .hypot(f64::from(q[1] - p[1]))
                .hypot(f64::from(q[2] - p[2]))
        })
        .fold(0.0f64, f64::max)
}

#[test]
#[ignore = "sonda"]
fn measure_what_the_relax_does_to_the_shape_and_to_the_mesh() {
    let base = sphere_shuffled();
    let idx = footprint(&base);
    println!("pegada: {} vértices", idx.len());
    println!(
        "base: raio-desvio {:.6}  cv-arestas {:.6}",
        radius_drift(&base, &idx),
        edge_cv(&base, &idx)
    );
    println!();
    println!("verbo         dabs  força   andou     raio-desvio   cv-arestas");
    for verb in [Verb::SlideRelax, Verb::Smooth] {
        for dabs in [1usize, 8, 32] {
            for strength in [0.5f32, 1.0] {
                let m = hold(verb, dabs, strength);
                println!(
                    "{:12}  {dabs:4}  {strength:5.2}  {:8.5}  {:12.6}  {:10.6}",
                    verb.label(),
                    moved(&base, &m, &idx),
                    radius_drift(&m, &idx),
                    edge_cv(&m, &idx),
                );
            }
        }
    }
}

/// **A BISSETRIZ DA BEIRA** — vale a pena numa fixture aberta?
///
/// ⚠️ **Esta sonda existe para decidir se o gate da borda é CONSTRUÍVEL hoje:**
/// a bissetriz só difere da normal da superfície onde a curva de borda **não** é
/// perpendicular à superfície. Num tubo as duas são radiais, então a fixture
/// pode não conter o fenômeno — e é isso que se mede aqui antes de escrever um
/// gate que passaria por vácuo.
#[test]
#[ignore = "sonda"]
fn measure_whether_the_open_fixture_can_tell_the_two_normals_apart() {
    let mesh = ph2d_mesh::shapes_open::open_tube3();
    let adj = mesh.adjacency();
    let mut worst: f64 = 0.0;
    let mut border = 0usize;
    for v in 0..mesh.positions().len() {
        if !adj.is_border(v) {
            continue;
        }
        border += 1;
        let at = mesh.positions()[v];
        // A bissetriz das arestas de borda, como o produto a computa.
        let mut acc = [0.0f64; 3];
        let mut n = 0;
        for &nb in adj.vert_verts.neighbours(v) {
            if !adj.is_border(nb as usize) {
                continue;
            }
            let p = mesh.positions()[nb as usize];
            let d = [
                f64::from(p[0] - at[0]),
                f64::from(p[1] - at[1]),
                f64::from(p[2] - at[2]),
            ];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            for k in 0..3 {
                acc[k] += d[k] / l;
            }
            n += 1;
        }
        assert_eq!(
            n, 2,
            "borda manifold tem exactamente dois vizinhos de borda"
        );
        let l = (acc[0] * acc[0] + acc[1] * acc[1] + acc[2] * acc[2]).sqrt();
        let bis = [acc[0] / l, acc[1] / l, acc[2] / l];
        let sn = mesh.normals()[v];
        let dot = bis[0] * f64::from(sn[0]) + bis[1] * f64::from(sn[1]) + bis[2] * f64::from(sn[2]);
        worst = worst.max(dot.abs().min(1.0).acos().to_degrees());
    }
    println!("open_tube3: {border} vértices de borda");
    println!("ângulo MÁXIMO entre a bissetriz e a normal da superfície: {worst:.3}°");
    println!(
        "⇒ {}",
        if worst < 5.0 {
            "as duas COINCIDEM — esta fixture NÃO contém o fenômeno"
        } else {
            "as duas DIFEREM — a fixture serve"
        }
    );
}
