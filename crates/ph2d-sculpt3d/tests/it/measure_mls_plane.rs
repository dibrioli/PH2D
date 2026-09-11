//! **QUANTO UMA SUPERFÍCIE MLS DIFERE DO PLANO** — a sonda que decide se a W7
//! tem conteúdo, antes de uma linha dela ser escrita.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_mls_plane \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! O `l-mode` que o plano 21 §4 promete para Flatten/Fill/Scrape/Clay é a
//! **projeção MLS** (Alexa et al. 2003): em vez de projectar num PLANO, o alvo
//! é uma superfície polinomial local ajustada à pegada. A pergunta que decide a
//! wave é uma só — **de quanto ela se afasta do plano?** —, e ela tem duas
//! metades opostas:
//!
//! * numa superfície **CURVA** a diferença tem de ser grande o bastante para o
//!   artista ver (senão é um chip sem conteúdo, o que o §4 recusa);
//! * numa superfície **PLANA** ela tem de ser ~zero (senão o `l-mode` mexeria
//!   onde o `s-mode` não mexe, e a lei do modo deixaria de ser *outra lei sobre
//!   a mesma coisa*).
//!
//! ⚠️ **O PLANO vem da porta do produto** ([`SculptStroke::probe_plane`]) e
//! nunca de um ajuste próprio: a comparação tem de ser contra o plano que o dab
//! de facto usa, senão ela mede a minha cópia. O **quadric** é ajustado aqui
//! porque é a coisa que está a ser AVALIADA — ele ainda não existe no produto, e
//! é exactamente essa a pergunta.

use ph2d_mesh::{Mesh, QueryScratch, shapes};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Verb};

/// Resolve `A x = b` para um sistema simétrico pequeno — eliminação de Gauss com
/// pivotamento parcial, em `f64`.
///
/// ⚠️ **`f64` e não `f32`:** as equações normais de um ajuste quadrático elevam
/// as coordenadas à QUARTA potência, e é por isso que o `u`/`v` chegam
/// normalizados pelo raio (abaixo) — as duas coisas juntas mantêm o sistema
/// resolúvel; só uma delas não basta.
fn solve(mut a: [[f64; 6]; 6], mut b: [f64; 6]) -> Option<[f64; 6]> {
    for col in 0..6 {
        let mut piv = col;
        for r in col + 1..6 {
            if a[r][col].abs() > a[piv][col].abs() {
                piv = r;
            }
        }
        if a[piv][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, piv);
        b.swap(col, piv);
        for r in col + 1..6 {
            let f = a[r][col] / a[col][col];
            let (lo, hi) = a.split_at_mut(r);
            for (dst, src) in hi[0][col..].iter_mut().zip(&lo[col][col..]) {
                *dst -= f * src;
            }
            b[r] -= f * b[col];
        }
    }
    let mut x = [0.0f64; 6];
    for i in (0..6).rev() {
        let mut s = b[i];
        for j in i + 1..6 {
            s -= a[i][j] * x[j];
        }
        x[i] = s / a[i][i];
    }
    Some(x)
}

/// `h ≈ c0 + c1·u + c2·v + c3·u² + c4·uv + c5·v²`, mínimos quadrados.
fn fit_quadric(samples: &[(f64, f64, f64)]) -> Option<[f64; 6]> {
    let mut ata = [[0.0f64; 6]; 6];
    let mut atb = [0.0f64; 6];
    for &(u, v, h) in samples {
        let row = [1.0, u, v, u * u, u * v, v * v];
        for i in 0..6 {
            for j in 0..6 {
                ata[i][j] += row[i] * row[j];
            }
            atb[i] += row[i] * h;
        }
    }
    solve(ata, atb)
}

fn eval_quadric(c: &[f64; 6], u: f64, v: f64) -> f64 {
    c[0] + c[1] * u + c[2] * v + c[3] * u * u + c[4] * u * v + c[5] * v * v
}

/// Uma base ortonormal cujo Z é `n`.
fn frame(n: [f32; 3]) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let n = [f64::from(n[0]), f64::from(n[1]), f64::from(n[2])];
    let seed = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let mut t = [
        seed[1] * n[2] - seed[2] * n[1],
        seed[2] * n[0] - seed[0] * n[2],
        seed[0] * n[1] - seed[1] * n[0],
    ];
    let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
    for x in &mut t {
        *x /= len;
    }
    let b = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    (t, b, n)
}

struct Row {
    plane_rms: f64,
    quad_rms: f64,
    max_g: f64,
    verts: usize,
    /// A RUGA de facto presente na pegada — `| |p| − raio_da_esfera |`, que é a
    /// grandeza que o Flatten existe para remover.
    ///
    /// ⚠️ **Sem ela o CONTROLE 2 lê o número ao contrário:** a minha primeira
    /// versão reportava *"o quadric capturou 41,6% do desvio"* e concluía que
    /// ele estava a comer a ruga — e o desvio ao plano numa esfera RUGOSA é
    /// **curvatura + ruga**, com a curvatura a ser exactamente o que ele deve
    /// capturar. Só comparando o resíduo com a ruga MEDIDA se separa as duas.
    noise_rms: f64,
}

/// Mede a pegada de UM dab: quanto a superfície se afasta do plano, e quanto do
/// quadric.
fn measure(
    mesh: &mut Mesh,
    center: [f32; 3],
    view: [f32; 3],
    radius: f32,
    sphere_r: f64,
) -> Option<Row> {
    let brush = Brush {
        verb: Verb::Flatten,
        radius,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let dab = Dab::at(center, radius, view);

    let mut s = SculptStroke::default();
    s.begin(mesh);
    let (p, n) = s.probe_plane(mesh, &brush, &dab);

    let (tx, ty, tz) = frame(n);
    let mut scratch = QueryScratch::default();
    let mut ids = Vec::new();
    mesh.verts_in_sphere(center, radius, &mut scratch, &mut ids);
    if ids.len() < 12 {
        return None;
    }

    let pos = mesh.positions();
    let inv_r = 1.0 / f64::from(radius);
    let samples: Vec<(f64, f64, f64)> = ids
        .iter()
        .map(|&i| {
            let q = pos[i as usize];
            let d = [
                f64::from(q[0] - p[0]),
                f64::from(q[1] - p[1]),
                f64::from(q[2] - p[2]),
            ];
            let u = (d[0] * tx[0] + d[1] * tx[1] + d[2] * tx[2]) * inv_r;
            let v = (d[0] * ty[0] + d[1] * ty[1] + d[2] * ty[2]) * inv_r;
            let h = d[0] * tz[0] + d[1] * tz[1] + d[2] * tz[2];
            (u, v, h)
        })
        .collect();

    let c = fit_quadric(&samples)?;
    let n_s = samples.len() as f64;
    let plane_rms = (samples.iter().map(|&(_, _, h)| h * h).sum::<f64>() / n_s).sqrt();
    let quad_rms = (samples
        .iter()
        .map(|&(u, v, h)| {
            let e = h - eval_quadric(&c, u, v);
            e * e
        })
        .sum::<f64>()
        / n_s)
        .sqrt();
    let max_g = samples
        .iter()
        .map(|&(u, v, _)| eval_quadric(&c, u, v).abs())
        .fold(0.0f64, f64::max);

    let noise_rms = (ids
        .iter()
        .map(|&i| {
            let q = pos[i as usize];
            let len = f64::from(q[0])
                .hypot(f64::from(q[1]))
                .hypot(f64::from(q[2]));
            let d = len - sphere_r;
            d * d
        })
        .sum::<f64>()
        / n_s)
        .sqrt();

    Some(Row {
        plane_rms,
        quad_rms,
        max_g,
        verts: samples.len(),
        noise_rms,
    })
}

/// **A MEDIÇÃO.** Numa esfera (curva) e num plano (o CONTROLE).
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn how_far_a_local_surface_sits_from_the_plane() {
    println!("\n  ESFERA de raio 1 — o dab olha para o polo +Z\n");
    println!("     raio   verts   |h| do PLANO   |h| do QUADRIC   pior |g|   |g| / raio");
    println!("  -------   -----   ------------   --------------   -------   ----------");
    for &r in &[0.1f32, 0.2, 0.3, 0.4, 0.6] {
        let mut m = shapes::uv_sphere(64, 96, 1.0);
        let Some(row) = measure(&mut m, [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], r, 1.0) else {
            println!("  {r:>7.2}   (pegada pequena demais)");
            continue;
        };
        println!(
            "  {r:>7.2}   {:>5}   {:>12.6}   {:>14.6}   {:>7.4}   {:>10.4}",
            row.verts,
            row.plane_rms,
            row.quad_rms,
            row.max_g,
            row.max_g / f64::from(r)
        );
    }

    println!(
        "\n  CONTROLE 1 — esfera de raio 20 (localmente PLANA). O quadric tem\n  \
         de quase desaparecer, senao o l-mode mexeria onde o s-mode nao mexe.\n"
    );
    println!("     raio   verts   |h| do PLANO   |h| do QUADRIC   pior |g|   |g| / raio");
    println!("  -------   -----   ------------   --------------   -------   ----------");
    for &r in &[0.4f32, 0.8] {
        // ⚠️ A tesselação sobe com o RAIO: a 64×96 uma esfera de raio 20 tem os
        // vértices a ~1 unidade uns dos outros, e um dab de 0,4 não apanha
        // NENHUM — a primeira corrida devolveu *"pegada pequena demais"* nas
        // duas linhas, que é a fixture a não conter o fenómeno.
        let mut m = shapes::uv_sphere(256, 384, 20.0);
        let Some(row) = measure(&mut m, [0.0, 0.0, 20.0], [0.0, 0.0, -1.0], r, 20.0) else {
            println!("  {r:>7.2}   (pegada pequena demais)");
            continue;
        };
        println!(
            "  {r:>7.2}   {:>5}   {:>12.8}   {:>14.8}   {:>7.5}   {:>10.5}",
            row.verts,
            row.plane_rms,
            row.quad_rms,
            row.max_g,
            row.max_g / f64::from(r)
        );
    }

    println!(
        "\n  CONTROLE 2 — o RISCO da wave: uma esfera com RUGA. Um Flatten existe\n  \
         para REMOVER detalhe; se a superficie MLS seguisse a ruga, ele deixaria\n  \
         de achatar. Um quadric e' de grau 2 sobre a pegada inteira -- a pergunta\n  \
         e' quanto da ruga ele captura.\n"
    );
    println!(
        "  ⇒ a regua e' a RUGA MEDIDA, nao o desvio ao plano: o quadric DEVE\n  \
           capturar a curvatura, e o que ele NAO pode e' comer a ruga.\n"
    );
    println!("     raio   verts   ruga MEDIDA   |h| do QUADRIC   da ruga sobra");
    println!("  -------   -----   -----------   --------------   -------------");
    for &r in &[0.2f32, 0.4] {
        let mut m = shapes::uv_sphere_noisy(64, 96, 1.0, 0.03);
        let Some(row) = measure(&mut m, [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], r, 1.0) else {
            println!("  {r:>7.2}   (pegada pequena demais)");
            continue;
        };
        println!(
            "  {r:>7.2}   {:>5}   {:>11.6}   {:>14.6}   {:>12.1}%",
            row.verts,
            row.noise_rms,
            row.quad_rms,
            row.quad_rms / row.noise_rms * 100.0
        );
    }
}

/// **A NORMALIZAÇÃO POR RAIO É LOAD-BEARING?** — a mutação que a remove
/// sobreviveu ao gate de invariância de escala, e uma cerca que nenhum oráculo
/// separa tem de ser MEDIDA em vez de defendida por prosa.
///
/// As equações normais de um quadric elevam as coordenadas à QUARTA potência,
/// então a pergunta é a partir de que raio o `f64` do solver deixa de absorver
/// isso.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn where_the_unnormalised_fit_starts_to_lie() {
    println!(
        "\n  A MESMA calote, ajustada com e sem normalizar por raio.\n  \
              A regua e' a altura avaliada, em fracao do raio do dab.\n"
    );
    println!("      escala   raio do dab   pior |g| norm.   pior |g| cru   desvio rel.");
    println!("   ---------   -----------   --------------   ------------   -----------");
    for &scale in &[1.0f32, 100.0, 10_000.0, 1_000_000.0] {
        let mut m = shapes::uv_sphere(64, 96, scale);
        let r = 0.4 * scale;
        let brush = Brush {
            verb: Verb::Flatten,
            radius: r,
            strength: 1.0,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        let dab = Dab::at([0.0, 0.0, scale], r, [0.0, 0.0, -1.0]);
        let mut s = SculptStroke::default();
        s.begin(&m);
        let (p, n) = s.probe_plane(&mut m, &brush, &dab);
        let (tx, ty, tz) = frame(n);

        let mut scratch = QueryScratch::default();
        let mut ids = Vec::new();
        m.verts_in_sphere(dab.center, r, &mut scratch, &mut ids);
        let pos = m.positions();
        let raw: Vec<(f64, f64, f64)> = ids
            .iter()
            .map(|&i| {
                let q = pos[i as usize];
                let d = [
                    f64::from(q[0] - p[0]),
                    f64::from(q[1] - p[1]),
                    f64::from(q[2] - p[2]),
                ];
                (
                    d[0] * tx[0] + d[1] * tx[1] + d[2] * tx[2],
                    d[0] * ty[0] + d[1] * ty[1] + d[2] * ty[2],
                    d[0] * tz[0] + d[1] * tz[1] + d[2] * tz[2],
                )
            })
            .collect();
        let inv = 1.0 / f64::from(r);
        let normed: Vec<(f64, f64, f64)> =
            raw.iter().map(|&(u, v, h)| (u * inv, v * inv, h)).collect();

        let (Some(cn), Some(cr)) = (fit_quadric(&normed), fit_quadric(&raw)) else {
            println!("   {scale:>9.0}   (singular)");
            continue;
        };
        let mut gn = 0.0f64;
        let mut worst = 0.0f64;
        for (&(u, v, _), &(ur, vr, _)) in normed.iter().zip(&raw) {
            let a = eval_quadric(&cn, u, v);
            let b = eval_quadric(&cr, ur, vr);
            gn = gn.max(a.abs());
            worst = worst.max((a - b).abs());
        }
        let mut gr = 0.0f64;
        for &(ur, vr, _) in &raw {
            gr = gr.max(eval_quadric(&cr, ur, vr).abs());
        }
        println!(
            "   {scale:>9.0}   {r:>11.1}   {:>14.6}   {:>12.6}   {:>11.3e}",
            gn / f64::from(r),
            gr / f64::from(r),
            worst / f64::from(r)
        );
    }
}
