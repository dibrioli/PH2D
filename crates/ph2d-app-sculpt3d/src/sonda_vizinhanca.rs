//! 🔎 **SONDA (não é gate) — o preço do desfoque NA SUPERFÍCIE** (`docs/3D/30`
//! §7, o critério de desistência da W6: *«blur de raio razoável a `32x` passar
//! de `100 ms` ⇒ vira aplicar, não vivo»*).
//!
//! A peça da lição nos quatro degraus: o laplaciano da retícula
//! (`ph2d_mesh_colors::difusao::Difusao`) e o desfoque `[f32; 4]` da peça
//! inteira para raios em fracção da diagonal da peça.

use std::time::Instant;

use ph2d_mesh_colors::difusao::Difusao;

#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_preco_do_desfoque_na_superficie() {
    let mesh = crate::scenes::tinta_fina::peca();
    let b = mesh.bounds();
    let diag = ((b.max[0] - b.min[0]).powi(2)
        + (b.max[1] - b.min[1]).powi(2)
        + (b.max[2] - b.min[2]).powi(2))
    .sqrt();
    eprintln!(
        "peça: {} vértices · {} faces · diagonal {diag:.3}",
        mesh.vert_count(),
        mesh.faces().len()
    );
    for k in 3u8..=6 {
        let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
        let tinta = ph2d_mesh_colors::Tinta::nova(mesh.vert_count(), faces(), k);
        let n = tinta.amostras().len();
        let t = Instant::now();
        let d = Difusao::nova(&tinta, |f| mesh.faces()[f].verts(), mesh.positions());
        let construir = t.elapsed().as_secs_f64() * 1e3;
        let mut lam: Vec<f32> = (0..n).map(|a| d.lambda_da_amostra(a)).collect();
        lam.sort_by(f32::total_cmp);
        let q = |p: f64| lam[((n - 1) as f64 * p) as usize];
        eprintln!(
            "degrau {k} ({}x) · {n} amostras · laplaciano {construir:.1} ms · {:.1} MB · λ por amostra: \
             mediana {:.3e} · p99 {:.3e} · p99,9 {:.3e} · máx {:.3e}",
            1u32 << k,
            d.footprint_bytes() as f64 / 1e6,
            q(0.5),
            q(0.99),
            q(0.999),
            lam[n - 1]
        );
        // Onde moram as amostras rígidas (λ acima de 4× a mediana)?
        let xs = crate::vizinhanca_da_peca::tests::posicoes(&tinta, &mesh);
        let (mediana, mut rigidas, mut nos_polos) = (q(0.5), 0usize, 0usize);
        for (a, x) in xs.iter().enumerate() {
            if d.lambda_da_amostra(a) > 4.0 * mediana {
                rigidas += 1;
                let r = (x[0] * x[0] + x[1] * x[1] + x[2] * x[2]).sqrt();
                if (x[1] / r).abs()
                    > (std::f32::consts::PI / 2.0 - 2.0 * std::f32::consts::PI / 24.0).sin()
                {
                    nos_polos += 1;
                }
            }
        }
        eprintln!(
            "    λ: p90 {:.2}× · p95 {:.2}× · p98 {:.2}× a mediana · {rigidas} acima de 4× ({nos_polos} nas duas primeiras filas dos pólos)",
            q(0.9) / mediana,
            q(0.95) / mediana,
            q(0.98) / mediana
        );
        for frac in [0.005f32, 0.01, 0.02, 0.05, 0.1] {
            let raio = frac * diag;
            let sigma = raio / 3.0;
            let grau = d.polinomio(sigma).len();
            let mut buf: Vec<[f32; 4]> = (0..n)
                .map(|i| [(i as f32 * 0.37).sin().abs(), 0.5, 0.25, 1.0])
                .collect();
            let mut tempos = Vec::new();
            for _ in 0..3 {
                let t = Instant::now();
                d.desfoca(sigma, &mut buf);
                tempos.push(t.elapsed().as_secs_f64() * 1e3);
            }
            tempos.sort_by(f64::total_cmp);
            // Um canal só — a ALTURA do relevo (`docs/3D/30` §20).
            let mut alt: Vec<f32> = (0..n).map(|i| (i as f32 * 0.37).sin().abs()).collect();
            let mut t1 = Vec::new();
            for _ in 0..3 {
                let t = Instant::now();
                d.desfoca(sigma, &mut alt);
                t1.push(t.elapsed().as_secs_f64() * 1e3);
            }
            t1.sort_by(f64::total_cmp);
            eprintln!(
                "    raio {:.1} % da diagonal ({raio:.4}) · grau {grau} · desfoque {:.1} ms (pior {:.1}) · \
                 um canal {:.1} ms (pior {:.1})",
                frac * 100.0,
                tempos[1],
                tempos[2],
                t1[1],
                t1[2]
            );
        }
    }
}
