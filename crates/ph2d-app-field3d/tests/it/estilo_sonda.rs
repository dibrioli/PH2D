//! ⏱️ **AS SONDAS DA AUDITORIA DA CAMADA DE ESTILO** (report do dono, 2026-09-19):
//! *«Edge tint e Cavity tint com bordas muito duras sem ajustes finos»* · *«Zone pivot não sei para
//! que serve mas parece morto»*.
//!
//! ⚠️ **Elas IMPRIMEM uma tabela e não afirmam nada** — são sondas, não gates (a mesma fronteira do
//! `studio_probe_tests`). Todas correm pelo caminho de REFERÊNCIA (`shade_render`), com o
//! olhar do PRODUTO (`ph2d_app_field3d::shading::OPENING_LOOK`), a câmera de omissão e a luz de abertura.
//!
//! ⛔ **Nenhuma toca na placa.**

#![allow(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::float_cmp,
    clippy::needless_range_loop
)]

use ph2d_field_render::{Gbuffer, Lighting, Orbit, Presentation, curvatura, shade_render, trace};
use ph2d_style::{Curvature, Rim, Style, Zones};
use ph2d_view_transform::{Look, ViewTransform};

const BG: [u8; 4] = [0, 0, 0, 0];

/// A cena `=35` e o que o produto deriva dela.
fn cena() -> (
    ph2d_field::FieldDoc,
    ph2d_field_eval::hybrid::Registry,
    Orbit,
    f32,
) {
    let doc = ph2d_app_field3d::smoke::scene(35);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let raio = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).map_or(1.0, |b| b.radius);
    (doc, reg, cam, raio)
}

/// A luz que a cena abre — a mesma porta do produto ([`ph2d_app_field3d::lights::opening_light`]).
fn luz(cam: &Orbit) -> Vec<ph2d_field_render::PointLamp> {
    let (world, fl) = ph2d_app_field3d::lights::opening_light(cam);
    vec![ph2d_field_render::PointLamp {
        world,
        radiance_at_one: ph2d_app_field3d::lights::radiance_at_one(fl),
    }]
}

fn perc(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

fn ordena(v: &mut [f64]) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
}

/// ⭐ **A largura da banda de transição, em PÍXEIS** — `0,8 / |∇w|` sobre os píxeis em que o peso
/// está entre `10 %` e `90 %`.
///
/// ⚠️ **Contar quantos píxeis estão tintados não serve**: um campo que salta de `0` a `1` num pixel
/// tinge o mesmo número de píxeis que um que sobe devagar. O que o olho lê é o GRADIENTE.
fn banda(g: &Gbuffer, w: &[f32]) -> (usize, f64, f64, f64) {
    let (wd, ht) = (g.width as usize, g.height as usize);
    let mut larguras: Vec<f64> = Vec::new();
    for y in 1..ht - 1 {
        for x in 1..wd - 1 {
            let i = y * wd + x;
            if !g.hit[i] {
                continue;
            }
            let viz = [i - 1, i + 1, i - wd, i + wd];
            if viz.iter().any(|&j| !g.hit[j]) {
                continue;
            }
            if !(w[i] > 0.10 && w[i] < 0.90) {
                continue;
            }
            let dx = f64::from(w[i + 1] - w[i - 1]) * 0.5;
            let dy = f64::from(w[i + wd] - w[i - wd]) * 0.5;
            let gm = dx.hypot(dy);
            if gm > 0.0 {
                larguras.push(0.8 / gm);
            }
        }
    }
    ordena(&mut larguras);
    (
        larguras.len(),
        perc(&larguras, 0.1),
        perc(&larguras, 0.5),
        perc(&larguras, 0.9),
    )
}

/// O salto de `kr` por PIXEL — a régua que decide se a dureza é do CLAMP ou do CAMPO.
fn salto_por_pixel(g: &Gbuffer, kr: &[f32]) -> (f64, f64, f64) {
    let (wd, ht) = (g.width as usize, g.height as usize);
    let mut v: Vec<f64> = Vec::new();
    for y in 1..ht - 1 {
        for x in 1..wd - 1 {
            let i = y * wd + x;
            if !g.hit[i] {
                continue;
            }
            let viz = [i - 1, i + 1, i - wd, i + wd];
            if viz.iter().any(|&j| !g.hit[j]) {
                continue;
            }
            let dx = f64::from(kr[i + 1] - kr[i - 1]) * 0.5;
            let dy = f64::from(kr[i + wd] - kr[i - wd]) * 0.5;
            v.push(dx.hypot(dy));
        }
    }
    ordena(&mut v);
    (perc(&v, 0.5), perc(&v, 0.99), perc(&v, 1.0))
}

fn curvatura_da_cena(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    g: &Gbuffer,
    eps: f32,
) -> Vec<f32> {
    let mut eval = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    curvatura::do_gbuffer(&mut eval, g, eps)
}

/// O `ε` que corresponde a uma fracção `f` do tamanho da peça, **pela porta do produto**.
///
/// ⚠️ A [`curvatura::eps_para`] é `escala · 0,0064` com piso de precisão; pedir `escala = raio·f/0,0064`
/// devolve `ε = raio·f` e herda o piso. ⛔ Um `max(1e-6)` escrito aqui seria a segunda cópia dele.
fn eps_fraccao(raio: f32, f: f32) -> f32 {
    curvatura::eps_para(raio * f / 0.0064)
}

/// ⏱️ **SONDA A1 — A DUREZA DA BORDA DA TINTA POR CURVATURA.**
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn sonda_a1_a_dureza_da_borda_por_curvatura() {
    let (doc, reg, cam, raio) = cena();
    println!("\n== A1 — a dureza da borda da tinta por curvatura ==");
    println!(
        "cena =35 · piece_radius = {raio:.6} · eps de produto = {:.6}",
        curvatura::eps_para(raio)
    );

    // (1) A DISTRIBUIÇÃO de H·R, e o salto por pixel.
    for (w, h) in [(320_u32, 240_u32), (640, 480), (1280, 960)] {
        let g = trace(&doc, &reg, &cam, w, h);
        let k = curvatura_da_cena(&doc, &reg, &g, curvatura::eps_para(raio));
        let hits: Vec<usize> = (0..g.hit.len()).filter(|&i| g.hit[i]).collect();
        let kr: Vec<f32> = k.iter().map(|v| v * raio).collect();
        let mut hs: Vec<f64> = hits.iter().map(|&i| f64::from(k[i])).collect();
        let mut krs: Vec<f64> = hits.iter().map(|&i| f64::from(kr[i])).collect();
        ordena(&mut hs);
        ordena(&mut krs);
        let (s50, s99, smax) = salto_por_pixel(&g, &kr);
        println!("\n-- {w}x{h} · {} px da peça --", hits.len());
        println!(
            "H      (1/mundo): min {:.3} p01 {:.3} p05 {:.3} p25 {:.3} p50 {:.3} p75 {:.3} p95 {:.3} p99 {:.3} max {:.3}",
            perc(&hs, 0.0),
            perc(&hs, 0.01),
            perc(&hs, 0.05),
            perc(&hs, 0.25),
            perc(&hs, 0.5),
            perc(&hs, 0.75),
            perc(&hs, 0.95),
            perc(&hs, 0.99),
            perc(&hs, 1.0)
        );
        println!(
            "H·R (o do estilo): min {:.3} p01 {:.3} p05 {:.3} p25 {:.3} p50 {:.3} p75 {:.3} p95 {:.3} p99 {:.3} max {:.3}",
            perc(&krs, 0.0),
            perc(&krs, 0.01),
            perc(&krs, 0.05),
            perc(&krs, 0.25),
            perc(&krs, 0.5),
            perc(&krs, 0.75),
            perc(&krs, 0.95),
            perc(&krs, 0.99),
            perc(&krs, 1.0)
        );
        let covas = krs.iter().filter(|v| **v < -0.5).count();
        let plano = krs.iter().filter(|v| v.abs() < 0.10).count();
        let filete = krs.iter().filter(|v| **v > 3.0).count();
        println!(
            "populações: cova (<-0,5) {covas} ({:.1} %) · plano (|H·R|<0,1) {plano} ({:.1} %) · filete (>3) {filete} ({:.1} %)",
            100.0 * covas as f64 / krs.len() as f64,
            100.0 * plano as f64 / krs.len() as f64,
            100.0 * filete as f64 / krs.len() as f64
        );
        println!("salto de H·R POR PIXEL: p50 {s50:.4} · p99 {s99:.4} · max {smax:.4}");
        // (2) A BANDA, por posição do knob.
        println!(
            "nitidez | satur.(|c|>=1) | banda ARESTA px (p10/p50/p90, n) | banda COVA px (p10/p50/p90, n)"
        );
        for s in [0.05_f32, 0.2, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
            let c: Vec<f32> = kr.iter().map(|v| (v * s).clamp(-1.0, 1.0)).collect();
            let wc: Vec<f32> = c.iter().map(|v| v.max(0.0)).collect();
            let wv: Vec<f32> = c.iter().map(|v| (-v).max(0.0)).collect();
            let sat = hits.iter().filter(|&&i| (kr[i] * s).abs() >= 1.0).count();
            let (na, a10, a50, a90) = banda(&g, &wc);
            let (nc, c10, c50, c90) = banda(&g, &wv);
            println!(
                "{s:>7.2} | {:>13.1}% | {a10:>6.2}/{a50:>6.2}/{a90:>7.2} n={na:<6} | {c10:>6.2}/{c50:>6.2}/{c90:>7.2} n={nc}",
                100.0 * sat as f64 / hits.len() as f64
            );
        }
    }

    // (3) O ε como ESCALA ESPACIAL — o candidato a knob que falta.
    println!("\n-- ε como escala espacial (640x480, nitidez 1) --");
    let g = trace(&doc, &reg, &cam, 640, 480);
    let hits: Vec<usize> = (0..g.hit.len()).filter(|&i| g.hit[i]).collect();
    println!(
        "ε/raio  |    ε     | H·R p05/p50/p95 | salto/px p50/p99 | banda ARESTA px p50 | banda COVA px p50"
    );
    for f in [0.0064_f32, 0.0128, 0.0256, 0.0512, 0.1024, 0.2048, 0.4096] {
        let eps = eps_fraccao(raio, f);
        let k = curvatura_da_cena(&doc, &reg, &g, eps);
        let kr: Vec<f32> = k.iter().map(|v| v * raio).collect();
        let mut krs: Vec<f64> = hits.iter().map(|&i| f64::from(kr[i])).collect();
        ordena(&mut krs);
        let (s50, s99, _) = salto_por_pixel(&g, &kr);
        let c: Vec<f32> = kr.iter().map(|v| v.clamp(-1.0, 1.0)).collect();
        let wc: Vec<f32> = c.iter().map(|v| v.max(0.0)).collect();
        let wv: Vec<f32> = c.iter().map(|v| (-v).max(0.0)).collect();
        let (_, _, a50, _) = banda(&g, &wc);
        let (_, _, c50, _) = banda(&g, &wv);
        println!(
            "{f:>7.4} | {eps:>8.5} | {:>5.2}/{:>5.2}/{:>5.2} | {s50:>7.4}/{s99:>7.4} | {a50:>18.2} | {c50:>16.2}",
            perc(&krs, 0.05),
            perc(&krs, 0.5),
            perc(&krs, 0.95)
        );
    }

    // (4) O PREÇO em precisão: uma esfera SOZINHA, onde H é conhecido.
    println!("\n-- o preço do ε: esfera de raio 0,45 sozinha (H verdadeiro = 2,2222) --");
    let esfera = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.45 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg2 = ph2d_field_eval::hybrid::Registry::new();
    let ge = trace(&esfera, &reg2, &cam, 320, 240);
    let hits_e: Vec<usize> = (0..ge.hit.len()).filter(|&i| ge.hit[i]).collect();
    println!("ε/raio  | H medido p50 | erro relativo p50 | erro relativo p95");
    for f in [0.0064_f32, 0.0128, 0.0256, 0.0512, 0.1024, 0.2048, 0.4096] {
        let eps = eps_fraccao(0.45, f);
        let k = curvatura_da_cena(&esfera, &reg2, &ge, eps);
        let mut e: Vec<f64> = hits_e
            .iter()
            .map(|&i| (f64::from(k[i]) / (1.0 / 0.45) - 1.0).abs())
            .collect();
        let mut m: Vec<f64> = hits_e.iter().map(|&i| f64::from(k[i])).collect();
        ordena(&mut e);
        ordena(&mut m);
        println!(
            "{f:>7.4} | {:>12.4} | {:>16.4} | {:>17.4}",
            perc(&m, 0.5),
            perc(&e, 0.5),
            perc(&e, 0.95)
        );
    }

    // (5) ⛔ O JOELHO SUAVE — a cura que a leitura ingénua prescreve. Ela actua no domínio do
    // VALOR (`c`), e a banda é `janela_em_w / |∇w|`: um joelho só pode mudar `dw/dc`, nunca `|∇c|`.
    println!("\n-- o joelho suave contra o `clamp` (640x480) --");
    let k = curvatura_da_cena(&doc, &reg, &g, curvatura::eps_para(raio));
    let kr: Vec<f32> = k.iter().map(|v| v * raio).collect();
    println!("nitidez | lei              | banda ARESTA px p10/p50/p90 (n)");
    for s in [0.5_f32, 1.0, 2.0, 4.0] {
        let duro: Vec<f32> = kr
            .iter()
            .map(|v| (v * s).clamp(-1.0, 1.0).max(0.0))
            .collect();
        let suave: Vec<f32> = kr
            .iter()
            .map(|v| {
                let t = (v * s).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            })
            .collect();
        let racional: Vec<f32> = kr
            .iter()
            .map(|v| {
                let c = (v * s).max(0.0);
                c / (1.0 + c)
            })
            .collect();
        for (nome, w) in [
            ("clamp (hoje)", &duro),
            ("smoothstep   ", &suave),
            ("c/(1+c)      ", &racional),
        ] {
            let (n, a10, a50, a90) = banda(&g, w);
            println!("{s:>7.2} | {nome} | {a10:>6.2}/{a50:>6.2}/{a90:>8.2} (n={n})");
        }
    }
}

/// ⭐⭐⭐ **O DEGRAU DE BYTE entre píxeis VIZINHOS** — a régua que o OLHO lê.
///
/// ⚠️ Ela mede a imagem FINAL, e por isso tem de vir com o CONTROLO ao lado: um sombreamento
/// honesto já tem degraus (a silhueta, o realce, a costura das peças). *Uma régua de dureza sem o
/// lado sem-tinta mede a dureza do render, não a da tinta.*
fn degrau_de_byte(g: &Gbuffer, img: &[u8]) -> (f64, f64, f64) {
    let (wd, ht) = (g.width as usize, g.height as usize);
    let c = img.as_chunks::<4>().0;
    let mut v: Vec<f64> = Vec::new();
    for y in 1..ht - 1 {
        for x in 1..wd - 1 {
            let i = y * wd + x;
            if !g.hit[i] {
                continue;
            }
            let viz = [i - 1, i + 1, i - wd, i + wd];
            if viz.iter().any(|&j| !g.hit[j]) {
                continue;
            }
            let d = viz
                .iter()
                .map(|&j| (0..3).map(|k| c[i][k].abs_diff(c[j][k])).max().unwrap_or(0))
                .max()
                .unwrap_or(0);
            v.push(f64::from(d));
        }
    }
    ordena(&mut v);
    (perc(&v, 0.5), perc(&v, 0.99), perc(&v, 1.0))
}

/// O mesmo degrau, **sem** os píxeis que o anti-serrilhado re-sombreia — o controlo da hipótese.
fn degrau_sem_borda(g: &Gbuffer, img: &[u8], de_borda: &std::collections::BTreeSet<usize>) -> f64 {
    let (wd, ht) = (g.width as usize, g.height as usize);
    let c = img.as_chunks::<4>().0;
    let mut v: Vec<f64> = Vec::new();
    for y in 1..ht - 1 {
        for x in 1..wd - 1 {
            let i = y * wd + x;
            if !g.hit[i] || de_borda.contains(&i) {
                continue;
            }
            let viz = [i - 1, i + 1, i - wd, i + wd];
            if viz.iter().any(|&j| !g.hit[j] || de_borda.contains(&j)) {
                continue;
            }
            let d = viz
                .iter()
                .map(|&j| (0..3).map(|k| c[i][k].abs_diff(c[j][k])).max().unwrap_or(0))
                .max()
                .unwrap_or(0);
            v.push(f64::from(d));
        }
    }
    ordena(&mut v);
    perc(&v, 0.99)
}

/// ⏱️ **SONDA A5 — A CURA MEDIDA: o `ε` como escala espacial, no PIXEL.**
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn sonda_a5_a_cura_medida_no_pixel() {
    let (doc, reg, cam, raio) = cena();
    let (w, h) = (640_u32, 480_u32);
    let mut g = trace(&doc, &reg, &cam, w, h);
    let points = luz(&cam);
    let produto = ph2d_app_field3d::shading::OPENING_LOOK;
    println!("\n== A5 — a cura medida, no pixel ==");
    println!("cena =35 · {w}x{h}");

    // O CONTROLO: a mesma imagem SEM tinta nenhuma.
    let k0 = curvatura_da_cena(&doc, &reg, &g, curvatura::eps_para(raio));
    g.curvature.clone_from(&k0);
    let limpo = pinta(&g, &cam, &points, Style::default(), raio, produto);
    let (l50, l99, lmax) = degrau_de_byte(&g, &limpo);
    println!(
        "CONTROLO (sem tinta nenhuma): degrau de byte entre vizinhos p50 {l50:.1} · p99 {l99:.1} · max {lmax:.0}"
    );

    let tinta = |sharp: f32| Style {
        curvature: Curvature {
            convex: ph2d_app_field3d::materials::colour_from_srgb8([255, 150, 40]),
            concave: ph2d_app_field3d::materials::colour_from_srgb8([30, 60, 200]),
            edge_sharpness: sharp,
            cavity_sharpness: sharp,
            ..Curvature::default()
        },
        ..Style::default()
    };

    // ⛔ A hipótese do ANTI-SERRILHADO: o passe de borda re-sombreia com QUATRO normais e **uma**
    // curvatura (`shade_render`, o laço dos `g.edges`) ⇒ a tinta por curvatura é o único canal que
    // ele nunca suaviza. Se fosse essa a causa, o degrau cairia ao excluir os píxeis de borda.
    let de_borda: std::collections::BTreeSet<usize> =
        g.edges.iter().map(|e| e.pixel as usize).collect();
    println!(
        "píxeis de borda (silhueta + quina) no G-buffer: {} ({:.1} % da peça)",
        de_borda.len(),
        100.0 * de_borda.len() as f64 / g.hit.iter().filter(|h| **h).count() as f64
    );

    println!("\n-- com o ε do PRODUTO, por posição da nitidez --");
    println!(
        "nitidez | satur. | degrau de byte p50 / p99 / max | vs CONTROLO | p99 SEM os px de borda"
    );
    let kr0: Vec<f32> = k0.iter().map(|v| v * raio).collect();
    for s in [0.0625_f32, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0] {
        let img = pinta(&g, &cam, &points, tinta(s), raio, produto);
        let (d50, d99, dmax) = degrau_de_byte(&g, &img);
        let sem = degrau_sem_borda(&g, &img, &de_borda);
        let sat = (0..g.hit.len())
            .filter(|&i| g.hit[i] && (kr0[i] * s).abs() >= 1.0)
            .count();
        println!(
            "{s:>7.4} | {:>5.1}% | {d50:>6.1} / {d99:>6.1} / {dmax:>5.0} | {:>10.2}x | {sem:>6.1}",
            100.0 * sat as f64 / g.hit.iter().filter(|h| **h).count() as f64,
            d99 / l99
        );
    }

    println!("\n-- a CURA: o ε como escala espacial (nitidez 1) --");
    println!(
        "ε/raio  | degrau de byte p50 / p99 / max | vs controlo | imagem vs o ε de produto (>8 px, pior)"
    );
    let base_eps = pinta(&g, &cam, &points, tinta(1.0), raio, produto);
    for f in [0.0064_f32, 0.0128, 0.0256, 0.0512, 0.1024, 0.2048] {
        let k = curvatura_da_cena(&doc, &reg, &g, eps_fraccao(raio, f));
        g.curvature.clone_from(&k);
        let img = pinta(&g, &cam, &points, tinta(1.0), raio, produto);
        let (d50, d99, dmax) = degrau_de_byte(&g, &img);
        let (_, _, n8, pior) = delta(&g, &img, &base_eps);
        println!(
            "{f:>7.4} | {d50:>6.1} / {d99:>6.1} / {dmax:>5.0} | {:>10.2}x | {n8:>7} px, pior {pior}",
            d99 / l99
        );
    }
    g.curvature.clone_from(&k0);

    println!("\n-- e o JOELHO SUAVE, no pixel: `smoothstep` no lugar do `clamp` --");
    println!("(medido fora da lei, com o mesmo campo: a tinta escrita à mão sobre a imagem limpa)");
    println!("nitidez | lei         | degrau de byte p50 / p99 / max");
    let kr: Vec<f32> = k0.iter().map(|v| v * raio).collect();
    let conv = ph2d_app_field3d::materials::colour_from_srgb8([255, 150, 40]);
    let conc = ph2d_app_field3d::materials::colour_from_srgb8([30, 60, 200]);
    for s in [0.5_f32, 1.0, 2.0] {
        for (nome, dura) in [("clamp     ", true), ("smoothstep", false)] {
            let mut img = limpo.clone();
            for (i, px) in img.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                if !g.hit[i] {
                    continue;
                }
                let c = kr[i] * s;
                let (wc, wv) = if dura {
                    let c = c.clamp(-1.0, 1.0);
                    (c.max(0.0), (-c).max(0.0))
                } else {
                    let f = |t: f32| {
                        let t = t.clamp(0.0, 1.0);
                        t * t * (3.0 - 2.0 * t)
                    };
                    (f(c), f(-c))
                };
                for ch in 0..3 {
                    let lin = ph2d_color::srgb::srgb_to_linear_byte(px[ch]);
                    let tint = 1.0 + (conv[ch] - 1.0) * wc + (conc[ch] - 1.0) * wv;
                    px[ch] = ph2d_color::srgb::linear_to_srgb_byte(lin * tint);
                }
            }
            let (d50, d99, dmax) = degrau_de_byte(&g, &img);
            println!("{s:>7.2} | {nome}  | {d50:>6.1} / {d99:>6.1} / {dmax:>5.0}");
        }
    }
}

/// ⏱️ **SONDA A4 — O CENSO DOS DEZ BOTÕES: qual deles move um pixel, e a partir de que estado.**
///
/// ⚠️ **A população é DERIVADA do produto** ([`ph2d_app_field3d::estilo::rows`]) e a escrita passa
/// pelas portas dele ([`estilo::with_number`], [`estilo::with_colour`]) — o molde do
/// `censo_dos_knobs_tests` da família da escultura.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn sonda_a4_o_censo_dos_dez_botoes() {
    use ph2d_app_field3d::estilo;
    use ph2d_field::{Bound, Param};

    let (doc, reg, cam, raio) = cena();
    let (w, h) = (640_u32, 480_u32);
    let mut g = trace(&doc, &reg, &cam, w, h);
    let k = curvatura_da_cena(&doc, &reg, &g, curvatura::eps_para(raio));
    g.curvature.clone_from(&k);
    let points = luz(&cam);
    let produto = ph2d_app_field3d::shading::OPENING_LOOK;
    println!("\n== A4 — o censo dos dez botões ==");
    println!("cena =35 · {w}x{h} · duas posições de cada controlo, pelo caminho do produto");

    // O estado ARMADO — o que o roteiro da cena manda o artista construir.
    let armado = Style {
        rim: Rim {
            color: [1.0; 3],
            strength: 1.5,
            width: 3.0,
        },
        curvature: Curvature {
            convex: ph2d_app_field3d::materials::colour_from_srgb8([255, 150, 40]),
            concave: ph2d_app_field3d::materials::colour_from_srgb8([30, 60, 200]),
            edge_sharpness: 1.0,
            cavity_sharpness: 1.0,
            ..Curvature::default()
        },
        zones: Zones {
            shadow: ph2d_app_field3d::materials::colour_from_srgb8([40, 70, 180]),
            highlight: ph2d_app_field3d::materials::colour_from_srgb8([255, 200, 120]),
            pivot: 0.18,
        },
        indirect_saturation: 1.0,
    };

    for (nome, base) in [("FÁBRICA", Style::default()), ("ARMADO", armado)] {
        println!("\n-- a partir do estado {nome} --");
        println!(
            "controlo                             | A→B                  | movidos>0 | movidos>8 | pior byte"
        );
        for r in estilo::rows(base, true) {
            let Param::Style(slot) = r.param else {
                continue;
            };
            let (Bound::Soft(teto) | Bound::Hard(teto) | Bound::Wrap(teto)) = r.bound;
            let (a, b, rotulo) = if r.swatch.is_some() {
                (
                    estilo::with_colour(base, slot, [255, 255, 255]),
                    estilo::with_colour(base, slot, [255, 40, 0]),
                    "branco → vermelho".to_string(),
                )
            } else {
                (
                    estilo::with_number(base, slot, 0.0),
                    estilo::with_number(base, slot, teto),
                    format!("0 → {teto}"),
                )
            };
            let ia = pinta(&g, &cam, &points, a, raio, produto);
            let ib = pinta(&g, &cam, &points, b, raio, produto);
            let (n0, _, n8, pior) = delta(&g, &ia, &ib);
            println!("{:<36} | {rotulo:<20} | {n0:>9} | {n8:>9} | {pior}", r.key);
        }
    }
}

/// A imagem do produto com um estilo.
fn pinta(
    g: &Gbuffer,
    cam: &Orbit,
    points: &[ph2d_field_render::PointLamp],
    style: Style,
    raio: f32,
    look: Look,
) -> Vec<u8> {
    let padrao = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &padrao,
        owners: None,
    };
    let lamps: [ph2d_field_render::Lamp; 0] = [];
    let light = Lighting {
        lamps: &lamps,
        points,
        sky: &ph2d_app_field3d::render_light::StudioSky,
        shadows: None,
    };
    shade_render(
        g,
        cam,
        &surfaces,
        &light,
        &Presentation {
            look,
            style: style.sanitized(),
            piece_radius: raio,
            bloom: ph2d_field_render::Bloom::default(),
        },
        BG,
    )
}

/// `(movidos > 0, movidos > 1, movidos > 8, pior byte)` sobre os píxeis da PEÇA.
fn delta(g: &Gbuffer, a: &[u8], b: &[u8]) -> (usize, usize, usize, u8) {
    let (ca, cb) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
    let (mut n0, mut n1, mut n8, mut pior) = (0, 0, 0, 0u8);
    for i in 0..ca.len() {
        if !g.hit[i] {
            continue;
        }
        let d = (0..3)
            .map(|c| ca[i][c].abs_diff(cb[i][c]))
            .max()
            .unwrap_or(0);
        if d > 0 {
            n0 += 1;
        }
        if d > 1 {
            n1 += 1;
        }
        if d > 8 {
            n8 += 1;
        }
        pior = pior.max(d);
    }
    (n0, n1, n8, pior)
}

/// ⏱️ **SONDA A2 — O `Zone Pivot` ESTÁ MORTO?**
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn sonda_a2_o_zone_pivot_esta_morto() {
    let (doc, reg, cam, raio) = cena();
    let (w, h) = (640_u32, 480_u32);
    let g = trace(&doc, &reg, &cam, w, h);
    let points = luz(&cam);
    let hits: Vec<usize> = (0..g.hit.len()).filter(|&i| g.hit[i]).collect();
    println!("\n== A2 — o Zone Pivot ==");
    println!(
        "cena =35 · {w}x{h} · {} px da peça · sem sombra (declarado)",
        hits.len()
    );

    // (1) A LUMINÂNCIA DE CENA que a `graded` lê — medida pelo olhar `Standard`, que é a
    // identidade para luz em `0..=1`: o byte devolve o linear pela curva sRGB.
    let leitura = Look {
        exposure_stops: 0.0,
        view: ViewTransform::Standard,
    };
    let base = pinta(&g, &cam, &points, Style::default(), raio, leitura);
    let bc = base.as_chunks::<4>().0;
    let mut ls: Vec<f64> = hits
        .iter()
        .map(|&i| {
            let lin = [0, 1, 2].map(|c| ph2d_color::srgb::srgb_to_linear_byte(bc[i][c]));
            f64::from(
                ph2d_style::LUMA[0] * lin[0]
                    + ph2d_style::LUMA[1] * lin[1]
                    + ph2d_style::LUMA[2] * lin[2],
            )
        })
        .collect();
    ordena(&mut ls);
    println!(
        "luminância de cena `l` (antes do olhar): min {:.5} p05 {:.5} p25 {:.5} p50 {:.5} p75 {:.5} p95 {:.5} max {:.5}",
        perc(&ls, 0.0),
        perc(&ls, 0.05),
        perc(&ls, 0.25),
        perc(&ls, 0.5),
        perc(&ls, 0.75),
        perc(&ls, 0.95),
        perc(&ls, 1.0)
    );
    let saturados = ls.iter().filter(|v| **v >= 0.99).count();
    println!(
        "⚠️ píxeis onde o olhar `Standard` satura (l>=0,99, logo a leitura é um PISO): {saturados} ({:.2} %)",
        100.0 * saturados as f64 / ls.len() as f64
    );
    println!("h = l/(l+pivô) nos percentis, por posição do pivô:");
    println!("  pivô |  h(p05)  |  h(p50)  |  h(p95)  | amplitude de h");
    for p in [0.001_f64, 0.01, 0.05, 0.18, 0.5, 1.0, 2.0, 4.0] {
        let hq = |q: f64| {
            let l = perc(&ls, q);
            l / (l + p)
        };
        println!(
            "{p:>6.3} | {:>8.4} | {:>8.4} | {:>8.4} | {:>8.4}",
            hq(0.05),
            hq(0.5),
            hq(0.95),
            hq(0.95) - hq(0.05)
        );
    }

    // (2) O PRODUTO: as tintas da FOTO do dono (sombra BRANCA, luz VERDE).
    let verde = ph2d_app_field3d::materials::colour_from_srgb8([0, 200, 0]);
    let com = |pivot: f32| Style {
        zones: Zones {
            shadow: [1.0; 3],
            highlight: verde,
            pivot,
        },
        ..Style::default()
    };
    let produto = ph2d_app_field3d::shading::OPENING_LOOK;
    println!("\n-- o produto (olhar Neutral, sombra BRANCA + luz VERDE sRGB(0,200,0)) --");
    println!("  pivô | vs pivô de FÁBRICA (0,18): movidos>0 / >1 / >8 · pior byte");
    let fabrica = pinta(&g, &cam, &points, com(0.18), raio, produto);
    for p in [0.001_f32, 0.01, 0.05, 0.18, 0.5, 1.0, 2.0, 4.0] {
        let img = pinta(&g, &cam, &points, com(p), raio, produto);
        let (n0, n1, n8, pior) = delta(&g, &img, &fabrica);
        println!(
            "{p:>6.3} | {n0:>7} / {n1:>6} / {n8:>5} · pior {pior}  ({:.1}% / {:.1}% / {:.1}%)",
            100.0 * n0 as f64 / hits.len() as f64,
            100.0 * n1 as f64 / hits.len() as f64,
            100.0 * n8 as f64 / hits.len() as f64
        );
    }
    println!("  degrau a degrau (cada um contra o anterior):");
    let escada = [0.001_f32, 0.01, 0.05, 0.18, 0.5, 1.0, 2.0, 4.0];
    let mut ant = pinta(&g, &cam, &points, com(escada[0]), raio, produto);
    for p in &escada[1..] {
        let img = pinta(&g, &cam, &points, com(*p), raio, produto);
        let (n0, n1, n8, pior) = delta(&g, &img, &ant);
        println!("  →{p:>6.3} | {n0:>7} / {n1:>6} / {n8:>5} · pior {pior}");
        ant = img;
    }

    // (3) ⭐ O CONTRAFACTUAL do defeito das cinco cores: com `shadow == highlight` o pivô é
    // INERTE AO BIT, por construção (o parêntesis da `graded` é exactamente zero).
    println!(
        "\n-- contrafactual: as DUAS tintas iguais (o que o defeito das cinco cores fazia) --"
    );
    let iguais = |pivot: f32| Style {
        zones: Zones {
            shadow: verde,
            highlight: verde,
            pivot,
        },
        ..Style::default()
    };
    let a = pinta(&g, &cam, &points, iguais(0.001), raio, produto);
    let b = pinta(&g, &cam, &points, iguais(4.0), raio, produto);
    let (n0, _, _, pior) = delta(&g, &a, &b);
    println!("pivô 0,001 contra 4,0 com as duas tintas VERDES: movidos {n0} · pior byte {pior}");
    let a2 = pinta(&g, &cam, &points, com(0.001), raio, produto);
    let b2 = pinta(&g, &cam, &points, com(4.0), raio, produto);
    let (n0b, _, n8b, piorb) = delta(&g, &a2, &b2);
    println!(
        "o mesmo par com as tintas DIFERENTES: movidos {n0b} (>8 bytes: {n8b}) · pior byte {piorb}"
    );

    // (4) E com as tintas mais separadas que o painel permite.
    println!("\n-- com o contraste MÁXIMO que o painel permite (sombra AZUL, luz LARANJA) --");
    let azul = ph2d_app_field3d::materials::colour_from_srgb8([20, 40, 160]);
    let laranja = ph2d_app_field3d::materials::colour_from_srgb8([255, 170, 60]);
    let duro = |pivot: f32| Style {
        zones: Zones {
            shadow: azul,
            highlight: laranja,
            pivot,
        },
        ..Style::default()
    };
    let f2 = pinta(&g, &cam, &points, duro(0.18), raio, produto);
    for p in [0.001_f32, 0.01, 0.05, 0.18, 0.5, 1.0, 2.0, 4.0] {
        let img = pinta(&g, &cam, &points, duro(p), raio, produto);
        let (n0, n1, n8, pior) = delta(&g, &img, &f2);
        println!(
            "{p:>6.3} | {n0:>7} / {n1:>6} / {n8:>5} · pior {pior}  ({:.1}% dos px com >8 bytes)",
            100.0 * n8 as f64 / hits.len() as f64
        );
    }
}

/// ⏱️ **SONDA A3 — OS CINCO TECTOS.**
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn sonda_a3_os_cinco_tectos() {
    let (doc, reg, cam, raio) = cena();
    let (w, h) = (640_u32, 480_u32);
    let mut g = trace(&doc, &reg, &cam, w, h);
    // ⚠️ A curvatura entra no G-buffer **uma vez**, e ela é inerte para os outros três botões: com as
    // tintas brancas a `curvature_tinted` multiplica por `1` exactamente, e o material de omissão tem
    // `subsurface_weight = 0`, logo o `at_curvature` não muda um bit.
    let k = curvatura_da_cena(&doc, &reg, &g, curvatura::eps_para(raio));
    g.curvature.clone_from(&k);
    let points = luz(&cam);
    let hits: Vec<usize> = (0..g.hit.len()).filter(|&i| g.hit[i]).collect();
    let produto = ph2d_app_field3d::shading::OPENING_LOOK;
    let perimetro = {
        let (wd, ht) = (w as usize, h as usize);
        let mut n = 0;
        for y in 1..ht - 1 {
            for x in 1..wd - 1 {
                let i = y * wd + x;
                if g.hit[i] && [i - 1, i + 1, i - wd, i + wd].iter().any(|&j| !g.hit[j]) {
                    n += 1;
                }
            }
        }
        n
    };
    println!("\n== A3 — os cinco tectos ==");
    println!(
        "cena =35 · {w}x{h} · {} px da peça · perímetro {perimetro} px",
        hits.len()
    );

    // --- Rim Strength (tecto 4) ---
    println!("\n-- Rim Strength (tecto de hoje: 4) · largura 3 --");
    let rim = |strength: f32, width: f32| Style {
        rim: Rim {
            color: [1.0; 3],
            strength,
            width,
        },
        ..Style::default()
    };
    let zero = pinta(&g, &cam, &points, Style::default(), raio, produto);
    println!(" força | vs o degrau anterior: >0 / >1 / >8 · pior | vs força 0: >8 · pior");
    let escada_f = [
        0.0_f32, 0.25, 0.5, 1.0, 1.5, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0,
    ];
    let mut ant = zero.clone();
    for f in escada_f {
        let img = pinta(&g, &cam, &points, rim(f, 3.0), raio, produto);
        let (n0, n1, n8, pior) = delta(&g, &img, &ant);
        let (_, _, z8, zpior) = delta(&g, &img, &zero);
        println!("{f:>6.2} | {n0:>7} / {n1:>6} / {n8:>5} · {pior:>3} | {z8:>7} · {zpior}");
        ant = img;
    }

    // --- Rim Width (tecto MAX_WIDTH = 64) ---
    println!(
        "\n-- Rim Width (tecto de hoje: {}) · força 2,0 --",
        Rim::MAX_WIDTH
    );
    println!("a espessura do fio = (px movidos >8 bytes) / perímetro");
    println!("largura | movidos>1 | movidos>8 | espessura px (>8) | pior byte");
    for wd in [1.0_f32, 2.0, 3.0, 4.32, 8.0, 16.0, 32.0, 64.0, 128.0] {
        let img = pinta(&g, &cam, &points, rim(2.0, wd), raio, produto);
        let (_, n1, n8, pior) = delta(&g, &img, &zero);
        println!(
            "{wd:>7.2} | {n1:>9} | {n8:>9} | {:>17.2} | {pior}",
            n8 as f64 / perimetro as f64
        );
    }

    // O mesmo a 1080p — a resolução em que o doc do `MAX_WIDTH` faz a afirmação dele.
    println!("\n-- Rim Width a 1920x1080 (a resolução que o doc do MAX_WIDTH cita) · força 2,0 --");
    let g2 = trace(&doc, &reg, &cam, 1920, 1080);
    let per2 = {
        let (wd, ht) = (1920_usize, 1080_usize);
        let mut n = 0;
        for y in 1..ht - 1 {
            for x in 1..wd - 1 {
                let i = y * wd + x;
                if g2.hit[i] && [i - 1, i + 1, i - wd, i + wd].iter().any(|&j| !g2.hit[j]) {
                    n += 1;
                }
            }
        }
        n
    };
    let zero2 = pinta(&g2, &cam, &points, Style::default(), raio, produto);
    println!("largura | movidos>8 | espessura px (>8) | pior byte  (perímetro {per2} px)");
    for wd in [3.0_f32, 8.0, 16.0, 32.0, 64.0, 128.0] {
        let img = pinta(&g2, &cam, &points, rim(2.0, wd), raio, produto);
        let (_, _, n8, pior) = delta(&g2, &img, &zero2);
        println!(
            "{wd:>7.2} | {n8:>9} | {:>17.3} | {pior}",
            n8 as f64 / per2 as f64
        );
    }

    // --- Curvature Sharpness (tecto 8) ---
    println!("\n-- Curvature Sharpness (tecto de hoje: 8) · aresta LARANJA, cova AZUL --");
    let tinta = |sharp: f32| Style {
        curvature: Curvature {
            convex: ph2d_app_field3d::materials::colour_from_srgb8([255, 150, 40]),
            concave: ph2d_app_field3d::materials::colour_from_srgb8([30, 60, 200]),
            edge_sharpness: sharp,
            cavity_sharpness: sharp,
            ..Curvature::default()
        },
        ..Style::default()
    };
    let neutro = pinta(&g, &cam, &points, Style::default(), raio, produto);
    println!("nitidez | satur. | vs degrau anterior: >1 / >8 · pior | vs neutro: >8 · pior");
    let escada_s = [
        0.0_f32, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0,
    ];
    let mut ant = neutro.clone();
    for s in escada_s {
        let img = pinta(&g, &cam, &points, tinta(s), raio, produto);
        let (_, n1, n8, pior) = delta(&g, &img, &ant);
        let (_, _, z8, zpior) = delta(&g, &img, &neutro);
        let sat = hits
            .iter()
            .filter(|&&i| (k[i] * raio * s).abs() >= 1.0)
            .count();
        println!(
            "{s:>7.3} | {:>5.1}% | {n1:>6} / {n8:>6} · {pior:>3} | {z8:>7} · {zpior}",
            100.0 * sat as f64 / hits.len() as f64
        );
        ant = img;
    }

    // --- Indirect Saturation (tecto 4) ---
    println!("\n-- Indirect Saturation (tecto de hoje: 4) --");
    let sat = |s: f32| Style {
        indirect_saturation: s,
        ..Style::default()
    };
    let um = pinta(&g, &cam, &points, sat(1.0), raio, produto);
    println!("  sat. | vs degrau anterior: >1 / >8 · pior | vs 1,0: >8 · pior");
    let escada_sat = [0.0_f32, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
    let mut ant = pinta(&g, &cam, &points, sat(escada_sat[0]), raio, produto);
    for s in escada_sat {
        let img = pinta(&g, &cam, &points, sat(s), raio, produto);
        let (_, n1, n8, pior) = delta(&g, &img, &ant);
        let (_, _, u8_, upior) = delta(&g, &img, &um);
        println!("{s:>6.2} | {n1:>6} / {n8:>6} · {pior:>3} | {u8_:>7} · {upior}");
        ant = img;
    }
    println!(
        "⚠️ nota: o material de omissão é CINZENTO e o céu é o `StudioSky` — uma lei que redistribui\n\
         saturação ENTRE canais tem pouco que fazer aqui. Repetido com uma base VERDE:"
    );
    let padrao_verde = [ph2d_material::OpenPbr {
        base_color: [0.08, 0.45, 0.12],
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    let surfaces_v = ph2d_field_render::Surfaces {
        all: &padrao_verde,
        owners: None,
    };
    let lamps: [ph2d_field_render::Lamp; 0] = [];
    let pinta_v = |s: f32| {
        let light = Lighting {
            lamps: &lamps,
            points: &points,
            sky: &ph2d_app_field3d::render_light::StudioSky,
            shadows: None,
        };
        shade_render(
            &g,
            &cam,
            &surfaces_v,
            &light,
            &Presentation {
                look: produto,
                style: sat(s).sanitized(),
                piece_radius: raio,
                bloom: ph2d_field_render::Bloom::default(),
            },
            BG,
        )
    };
    let um_v = pinta_v(1.0);
    println!("  sat. | vs 1,0 (base verde): >1 / >8 · pior");
    for s in escada_sat {
        let img = pinta_v(s);
        let (_, n1, n8, pior) = delta(&g, &img, &um_v);
        println!("{s:>6.2} | {n1:>6} / {n8:>6} · {pior}");
    }
}

/// ⭐⭐⭐ **A CURA: a suavidade AMACIA a borda da tinta por curvatura, e não mata a tinta.**
///
/// Report do dono (2026-09-19, com foto): *«Edge tint e Cavity tint com bordas muito duras sem
/// ajustes finos, não me parece certo.»*
///
/// # ⛔ Porque este gate mede o PIXEL e não a lei
///
/// A lei ([`ph2d_style::Style::curvature_tinted`]) está **ilibada**: ela é uma função por ponto de
/// uma grandeza constante por troço, e *uma função por ponto de um campo constante por troço é
/// constante por troço*. ⇒ nenhum gate na `ph2d-style` pode ver esta cura — ela vive na DISTÂNCIA a
/// que a curvatura é medida, a montante, e só o pixel a mostra.
///
/// # As três metades, e cada uma fecha um buraco diferente
///
/// 1. **A borda AMACIA** — o degrau de byte entre píxeis vizinhos cai, e a régua tem o CONTROLO ao
///    lado (a mesma imagem sem estilo nenhum). Sem o controlo, «169 bytes» não quer dizer nada.
/// 2. **A tinta SOBREVIVE** — as covas continuam côncavas. É a cerca que o
///    [`Curvature::MAX_SOFTNESS`] existe para pôr: medir de longe demais apaga a feição, e um botão
///    que amacia até a tinta desaparecer não é um ajuste fino, é um interruptor.
/// 3. **A omissão do PRODUTO está do lado macio** — senão a cura existiria e o artista não a
///    receberia, que é o defeito que o `CLAUDE.md` §0.0 chama de *o caminho lento a definir o produto*.
#[test]
fn a_suavidade_amacia_a_borda_e_nao_mata_a_tinta() {
    let (doc, reg, cam, raio) = cena();
    let g = trace(&doc, &reg, &cam, 320, 240);
    let points = luz(&cam);
    // Um estilo que ACENDE as duas tintas — sem elas a curvatura nem é lida.
    let tinta = |softness: f32| Style {
        curvature: Curvature {
            convex: [1.0, 0.35, 0.2],
            concave: [0.2, 0.4, 1.0],
            softness,
            ..Curvature::default()
        },
        ..Style::default()
    };
    // ⚠️ **O `eps` sai da MESMA porta que o produto usa** — ver
    // [`ph2d_field_render::Presentation::curvature_eps`]. ⛔ Uma fracção multiplicada à mão aqui
    // seria a segunda resposta, e ela divergiria no dia em que o piso mudasse.
    //
    // ⚠️ **O canal é trocado NO SÍTIO** e não por uma cópia do G-buffer: o [`Gbuffer`] não é `Clone`
    // de propósito (ele é o quadro, não um valor), e torná-lo clonável para servir um teste seria
    // mudar o produto para caber no arnês.
    let mut g = g;
    let pinta_com = |g: &mut Gbuffer, softness: f32| {
        let pres = Presentation {
            look: Look::default(),
            style: tinta(softness).sanitized(),
            piece_radius: raio,
            bloom: ph2d_field_render::Bloom::default(),
        };
        g.curvature_style = curvatura_da_cena(&doc, &reg, g, pres.curvature_eps());
        let img = pinta(g, &cam, &points, pres.style, raio, Look::default());
        let kr: Vec<f32> = g.curvature_style.iter().map(|v| v * raio).collect();
        (degrau_de_byte(g, &img).1, kr)
    };

    // O CONTROLO: a mesma cena sem estilo nenhum. É ele que dá escala ao número.
    let sem_estilo = degrau_de_byte(
        &g,
        &pinta(&g, &cam, &points, Style::default(), raio, Look::default()),
    )
    .1;

    let (duro, kr_duro) = pinta_com(&mut g, Curvature::MIN_SOFTNESS);
    let (macio, kr_macio) = pinta_com(&mut g, Curvature::SOFTNESS);
    println!(
        "\n== a cura ==\ncontrolo (sem estilo) p99 = {sem_estilo:.0}\n\
         piso   ({:.4}) p99 = {duro:.0}  ⇒ {:.2}× o controlo\n\
         fábrica({:.4}) p99 = {macio:.0}  ⇒ {:.2}× o controlo",
        Curvature::MIN_SOFTNESS,
        duro / sem_estilo,
        Curvature::SOFTNESS,
        macio / sem_estilo,
    );

    // (1) ⭐ A borda AMACIA, e a barra é uma FRACÇÃO do que o piso entrega — não um número escolhido.
    assert!(
        macio < duro * 0.75,
        "a suavidade de fábrica não amacia a borda: {macio:.0} contra {duro:.0}"
    );
    // ⚠️ **Piso de população da régua**: com a tinta apagada os dois lados leriam o controlo e a
    // desigualdade acima seria trivialmente falsa OU trivialmente verdadeira por ruído.
    assert!(
        duro > sem_estilo * 2.0,
        "a régua não vê o fenómeno: o piso lê {duro:.0} contra um controlo de {sem_estilo:.0}"
    );

    // (2) ⛔ A tinta SOBREVIVE — as covas continuam côncavas nas duas pontas do botão.
    for (nome, kr) in [("piso", &kr_duro), ("fábrica", &kr_macio)] {
        let mut v: Vec<f64> = (0..kr.len())
            .filter(|&i| g.hit[i])
            .map(|i| f64::from(kr[i]))
            .collect();
        ordena(&mut v);
        let p05 = perc(&v, 0.05);
        println!("{nome}: p05 de H·R = {p05:.3}");
        assert!(
            p05 < -0.5,
            "com a suavidade em «{nome}» as covas deixaram de ser côncavas (p05 = {p05:.3}): a \
             Cavity Tint morreu"
        );
    }
}

/// ⭐⭐ **A OMISSÃO DO PRODUTO ESTÁ DO LADO MACIO — e é erro de COMPILAÇÃO, não um teste.**
///
/// Sem isto a cura existiria e o artista não a receberia, que é o defeito que o `CLAUDE.md` §0.0
/// chama de *o caminho lento a definir o produto*. ⚠️ Um `assert!` sobre constantes é dobrado pelo
/// compilador — ver o irmão em `ph2d-style/src/tests_botoes.rs`.
const _: () = assert!(Curvature::SOFTNESS >= Curvature::MIN_SOFTNESS * 4.0);
