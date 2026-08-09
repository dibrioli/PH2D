//! **A FASE 0 DO EXTRACT** — a forma do problema, medida antes de escolher
//! qualquer constante.
//!
//! Duas perguntas, e cada uma decide um número que o kernel teria de inventar:
//!
//! 1. **Qual é a escala de comprimento intrínseca da superfície?** O extract
//!    ergue a casca uma fração acima do original para ela não brigar por
//!    profundidade com a peça de onde saiu, e um valor absoluto seria certo numa
//!    malha e errado na seguinte. A única régua que a geometria tem é a **aresta
//!    dela**.
//! 2. **Quanto custa a SELEÇÃO?** O extract varre a máscara, expande pelas
//!    faces e reconstrói uma malha — se o custo for do documento e não da
//!    região, ele fica pesado exatamente na peça grande onde ele é útil.
//!
//! Rodar: `cargo test -p ph2d-mesh --release --test measure_extract -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes};
use std::time::Instant;

/// A mediana das arestas da malha — a régua que o erguimento usa.
fn median_edge(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let mut len: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
            if a < b {
                let d = [p[b][0] - p[a][0], p[b][1] - p[a][1], p[b][2] - p[a][2]];
                len.push(d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt());
            }
        }
    }
    len.sort_by(|a, b| a.partial_cmp(b).unwrap());
    len[len.len() / 2]
}

/// Mascara o hemisfério `y > 0` — a fixture do extract: uma calota com uma
/// fronteira longa, que é onde a costura vive.
fn mask_upper(mesh: &mut Mesh) -> usize {
    let n = mesh.vert_count();
    let up: Vec<bool> = (0..n).map(|i| mesh.positions()[i][1] > 0.0).collect();
    let m = mesh.masks_mut();
    let mut count = 0;
    for i in 0..n {
        m[i] = if up[i] {
            count += 1;
            1.0
        } else {
            0.0
        };
    }
    count
}

/// O conjunto que o extract vai copiar: os mascarados, expandidos às faces que
/// os tocam e de volta aos vértices dessas faces (a franja de um anel).
fn selection(mesh: &Mesh) -> (usize, usize, usize) {
    let masks = mesh.masks().unwrap();
    let sel: Vec<bool> = masks.iter().map(|&m| m >= 0.5).collect();
    let mut faces = 0usize;
    let mut seen = vec![false; mesh.vert_count()];
    for f in mesh.faces() {
        if f.verts().iter().any(|&v| sel[v as usize]) {
            faces += 1;
            for &v in f.verts() {
                seen[v as usize] = true;
            }
        }
    }
    let verts = seen.iter().filter(|&&s| s).count();
    (sel.iter().filter(|&&s| s).count(), faces, verts)
}

#[test]
#[ignore = "sonda"]
fn measure_the_shape_of_an_extract() {
    println!("\n== a REGUA: mediana das arestas ==");
    for (name, m) in [
        ("uv_sphere(24,32)", shapes::uv_sphere(24, 32, 1.0)),
        ("uv_sphere(64,96)", shapes::uv_sphere(64, 96, 1.0)),
        ("uv_sphere(128,192)", shapes::uv_sphere(128, 192, 1.0)),
    ] {
        println!(
            "{name:20} {:6} verts  aresta mediana {:.5}",
            m.vert_count(),
            median_edge(&m)
        );
    }

    println!("\n== a SELECAO: hemisferio mascarado ==");
    for (name, mut m) in [
        ("uv_sphere(24,32)", shapes::uv_sphere(24, 32, 1.0)),
        ("uv_sphere(64,96)", shapes::uv_sphere(64, 96, 1.0)),
        ("uv_sphere(128,192)", shapes::uv_sphere(128, 192, 1.0)),
    ] {
        let masked = mask_upper(&mut m);
        let t = Instant::now();
        let (sel, faces, verts) = selection(&m);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        assert_eq!(sel, masked);
        println!(
            "{name:20} malha {:6}v  mascarados {:6}  faces {:6}  verts+franja {:6}  ({:.3} ms)",
            m.vert_count(),
            sel,
            faces,
            verts,
            ms
        );
    }
}

/// **O QUE CADA PASSADA DE RELAXAMENTO CUSTA** — o número que decide a faixa do
/// slider.
///
/// A costura relaxa por laplaciano, e a borda de uma calota é um círculo: a
/// média de um ponto com os dois vizinhos dele num círculo cai na CORDA. É o
/// encurtamento de curva, e ele é MONOTÔNICO — a pergunta não é *se* a beira
/// encolhe, é a partir de quantas passadas isso deixa de ser "acalmou o
/// serrilhado" e passa a ser "a peça ficou menor que a máscara".
#[test]
#[ignore = "sonda"]
fn measure_what_a_relaxation_pass_costs() {
    use ph2d_mesh::{Extract, extract_masked};
    let mut src = shapes::uv_sphere(24, 32, 1.0);
    mask_upper(&mut src);
    let reach = |m: &Mesh| {
        m.positions()
            .iter()
            .map(|p| p[0].mul_add(p[0], p[2] * p[2]).sqrt())
            .fold(0.0_f32, f32::max)
    };
    let base = reach(
        &extract_masked(
            &src,
            Extract {
                thickness: 0.0,
                smooth: 0,
            },
        )
        .expect("ha' o que extrair"),
    );
    println!("\n== o preco de relaxar a costura (calota, aresta mediana 0,13081) ==");
    for passes in [0u32, 1, 2, 3, 4, 6, 8, 12, 16, 24] {
        let r = reach(
            &extract_masked(
                &src,
                Extract {
                    thickness: 0.0,
                    smooth: passes,
                },
            )
            .expect("ha' o que extrair"),
        );
        println!(
            "{passes:3} passe(s)  alcance {r:.4}  ({:+.2}% da mascara)",
            (r / base - 1.0) * 100.0
        );
    }
}

/// **QUANTAS PASSADAS UMA COSTURA SERRILHADA PRECISA** — e é ela, não a lisa,
/// que decide a faixa do slider.
///
/// ⚠️ A calota tem a beira já limpa, então ela converge em duas passadas e
/// esconde a pergunta. O caso do artista é uma máscara pintada à mão: a
/// fronteira entra e sai um anel a cada poucos vértices, e o serrilhado é
/// exatamente o que o relaxamento existe para acalmar.
///
/// A grandeza é a **rugosidade da beira** — o desvio-padrão da altura dos
/// vértices dela. Uma beira lisa nesta fixture está toda na mesma latitude.
#[test]
#[ignore = "sonda"]
fn measure_how_many_passes_a_jagged_seam_needs() {
    use ph2d_mesh::{Extract, extract_masked};
    let mut src = shapes::uv_sphere(24, 32, 1.0);
    // Uma máscara de fronteira SERRILHADA: a latitude de corte oscila com a
    // longitude, o que é o que uma mão desenha.
    let n = src.vert_count();
    let jag: Vec<bool> = (0..n)
        .map(|i| {
            let p = src.positions()[i];
            let ang = p[2].atan2(p[0]);
            let cut = 0.15 * (ang * 9.0).sin();
            p[1] > cut
        })
        .collect();
    {
        let m = src.masks_mut();
        for i in 0..n {
            m[i] = f32::from(u8::from(jag[i]));
        }
    }
    let roughness = |m: &Mesh| {
        let adj = m.adjacency();
        let ys: Vec<f32> = (0..m.vert_count())
            .filter(|&i| adj.is_border(i))
            .map(|i| m.positions()[i][1])
            .collect();
        let mean = ys.iter().sum::<f32>() / ys.len() as f32;
        (ys.iter().map(|y| (y - mean) * (y - mean)).sum::<f32>() / ys.len() as f32).sqrt()
    };
    println!("\n== quantas passadas uma costura SERRILHADA precisa ==");
    let mut prev = f32::MAX;
    for passes in [0u32, 1, 2, 3, 4, 6, 8, 12, 16] {
        let out = extract_masked(
            &src,
            Extract {
                thickness: 0.0,
                smooth: passes,
            },
        )
        .expect("ha' o que extrair");
        let r = roughness(&out);
        println!(
            "{passes:3} passe(s)  rugosidade da beira {r:.5}  (ganho {:+.2}%)",
            if prev == f32::MAX {
                0.0
            } else {
                (r / prev - 1.0) * 100.0
            }
        );
        prev = r;
    }
}
