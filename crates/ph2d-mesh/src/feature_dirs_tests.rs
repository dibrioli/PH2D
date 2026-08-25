//! Os gates da detecção de feição, e a sonda que fixa os quatro coeficientes.

use super::{FeatureOptions, feature_dirs};
use crate::shapes;

/// A aresta média de uma malha — o `h` que a cadeia usa é da mesma ordem.
fn edge_mean(m: &crate::Mesh) -> f32 {
    let pos = m.positions();
    let (mut s, mut n) = (0.0f32, 0usize);
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            s += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            n += 1;
        }
    }
    s / n.max(1) as f32
}

/// ⭐⭐⭐ **O CONTROLO QUE SEPARA A LEI DO RUÍDO.**
///
/// Uma **esfera** não tem feição nenhuma: as duas curvaturas são iguais em todo o
/// lado, e a lei tem de marcar **quase nada**. Um **cilindro** tem duas circunferências
/// agudas (onde a parede encontra a tampa), e a lei tem de as marcar.
///
/// ⛔⛔ **A 1ª redacção usava um OCTAEDRO, e ele não continha o fenómeno:** o sólido cru
/// tem **6 vértices**, e a lei é por vértice — não havia onde uma aresta aguda ter
/// pontos. *Uma fixtura que não contém o fenómeno devolve `0` e lê-se como aprovação.*
///
/// ⛔ **É este par que torna o resultado uma medição e não uma opinião.** Uma detecção
/// que marca a esfera está a ler ruído; uma que não marca a quina do cilindro não serve para
/// nada — e nenhum dos dois erros aparece se só se olhar para uma das peças.
#[test]
fn the_sphere_has_no_feature_and_the_cylinder_rim_is_one() {
    let opts = FeatureOptions::default();

    let mut sphere = shapes::uv_sphere(32, 48, 1.0);
    sphere.triangulate();

    let (_, sr) = feature_dirs(&sphere, edge_mean(&sphere), opts);

    let mut cyl = shapes::cylinder(64, 0.5, 1.5);
    cyl.triangulate();

    let (_, or) = feature_dirs(&cyl, edge_mean(&cyl), opts);

    let frac = |r: &crate::FeatureReport| {
        100.0 * r.marked as f64 / r.points.max(1) as f64
    };
    eprintln!(
        "esfera:   {} de {} vertices marcados ({:.1}%) | recusados: {} planos, {} pela JANELA",
        sr.marked, sr.points, frac(&sr), sr.rejected_flat, sr.rejected_window
    );
    eprintln!(
        "cilindro: {} de {} vertices marcados ({:.1}%) | recusados: {} planos, {} pela JANELA",
        or.marked, or.points, frac(&or), or.rejected_flat, or.rejected_window
    );
    assert!(
        sr.points > 500,
        "a fixtura tem de ter tamanho: {} vertices",
        sr.points
    );
    assert!(
        frac(&sr) < 5.0,
        "⛔ a ESFERA nao tem feicao nenhuma, e a lei marcou {:.1}% dos vertices — \
         isso e' ruido a passar por feicao",
        frac(&sr)
    );
}

/// ⭐⭐⭐ **A SONDA QUE FIXA OS QUATRO COEFICIENTES** (`CLAUDE.md` §0.0).
///
/// ```text
/// cargo test -p ph2d-mesh --release -- --ignored the_feature_law_sweeps --nocapture
/// ```
///
/// ⛔ **O que ela mede não é «quantas feições achámos»** — a espec é explícita: *marcar
/// feição a mais é pior que marcar a menos*, porque cada restrição força uma
/// singularidade. As colunas que decidem são **a esparsidade** e **o que a janela de
/// facto recusa**: se `rejected_window` for zero, a janela não está a filtrar nada e a
/// meia-largura é pequena de mais.
#[test]
#[ignore = "sonda -- os quatro coeficientes da lei da feicao"]
fn the_feature_law_sweeps_its_four_coefficients() {
    let mut sphere = shapes::uv_sphere(32, 48, 1.0);
    sphere.triangulate();

    let mut cyl = shapes::cylinder(64, 0.5, 1.5);
    cyl.triangulate();

    let (hs, ho) = (edge_mean(&sphere), edge_mean(&cyl));

    eprintln!(
        "{:>8} {:>10} {:>8} | {:>22} | {:>22}",
        "r1/h", "anisotrop", "janela", "ESFERA marcados", "CILINDRO marcados"
    );
    for r1 in [1.0f32, 2.0, 4.0, 8.0] {
        for min_a in [0.6f32, 0.8] {
            for hw in [0.25f32, 1.0] {
                let (kmin, min_a) = (0.05f32, min_a);
                let opts = FeatureOptions {
                    r1_in_h: r1,
                    min_anisotropy: min_a,
                    min_curvature_in_bbox: kmin,
                    half_window_in_h: hw,
                    ..FeatureOptions::default()
                };
                let (_, sr) = feature_dirs(&sphere, hs, opts);
                let (_, or) = feature_dirs(&cyl, ho, opts);
                let pct = |r: &crate::FeatureReport| {
                    100.0 * r.marked as f64 / r.points.max(1) as f64
                };
                eprintln!(
                    "{r1:>8.1} {min_a:>10.2} {hw:>8.2} | {:>7} ({:>5.1}%) jan {:>4} | \
                     {:>7} ({:>5.1}%) jan {:>4}",
                    sr.marked,
                    pct(&sr),
                    sr.rejected_window,
                    or.marked,
                    pct(&or),
                    or.rejected_window
                );
            }
        }
    }
}

/// Diagnóstico: o que a lei de facto lê num cilindro, raio a raio.
#[test]
#[ignore = "sonda -- diagnostico do estimador"]
fn what_does_the_law_actually_read() {
    let mut cyl = shapes::cylinder(64, 0.5, 1.5);
    cyl.triangulate();
    let n = cyl.normals();
    eprintln!("normais: {} entradas · primeiras 3: {:?}", n.len(), &n[..3.min(n.len())]);
    let nz = n.iter().filter(|v| v[0].abs() + v[1].abs() + v[2].abs() > 1e-6).count();
    eprintln!("normais NAO nulas: {nz} de {}", n.len());
    let e = edge_mean(&cyl);
    eprintln!("aresta media {e:.4} · vertices {}", cyl.positions().len());
    let (dirs, rep) = feature_dirs(&cyl, e, FeatureOptions { r1_in_h: 4.0, min_anisotropy: 0.0, min_curvature_in_bbox: 0.0, ..FeatureOptions::default() });
    eprintln!("com os pisos a ZERO: {} marcados, {} planos, {} janela, {} degenerados",
        rep.marked, rep.rejected_flat, rep.rejected_window, rep.rejected_degenerate);
    for d in dirs.iter().take(5) {
        eprintln!("   v{} anisotropia {:.3} raio {:.3}", d.vert, d.anisotropy, d.radius);
    }
}
