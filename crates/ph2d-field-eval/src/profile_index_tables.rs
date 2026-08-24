//! ⭐ **As TABELAS da consulta de perfil** — medição, não gate.
//!
//! Irmão do [`super::tests`] por responsabilidade (teto de LOC): ali ficam as afirmações que têm de
//! ser verdade, aqui as varreduras de relógio que escolheram cada número. Todas `#[ignore]`, porque
//! relógio sob carga não vale nada (`CLAUDE.md` §5.0).

use super::ProfileIndex;
use super::tests::{cloud, ngon};
use ph2d_field::{FillRule, Profile};

/// ⭐⭐ **QUANTO A CONSULTA COMPRA** — a tabela que decide se esta wave vale o que custa.
///
/// ⚠️ `#[ignore]` porque mede relógio — máquina calma:
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     profile_index::tests::the_table_of_what_the_query_buys --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_what_the_query_buys() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    const N: usize = 200_000;
    // ⚠️ A nuvem cobre 1,5× a caixa: metade FORA, que é a proporção que uma marcha de facto vê.
    let (xs, ys) = cloud(N, 0.9);
    let zs = vec![0.0f32; N];
    println!("arestas | fita (ns/pt) | consulta (ns/pt) | ganho | construir");
    for n in [56usize, 168, 332, 664, 940] {
        let p = Profile::new(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero, 1e-3)
            .expect("perfil válido");

        let tree = crate::profile::sd_profile(&p, &Tree::x(), &Tree::y());
        let shape = crate::Engine::from(tree);
        let mut eval = crate::Engine::new_float_slice_eval();
        let tape = shape.ez_float_slice_tape();
        let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
        let mut a = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
            a.push(t0.elapsed().as_secs_f64() * 1e9 / N as f64);
        }
        a.sort_by(f64::total_cmp);

        let mut build = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let idx = ProfileIndex::build(&p);
            build.push(t0.elapsed().as_secs_f64() * 1e3);
            drop(idx);
        }
        build.sort_by(f64::total_cmp);
        let idx = ProfileIndex::build(&p);
        let mut got = Vec::new();
        idx.sd_batch(&xs, &ys, &mut got);
        let mut acc: f32 = got.iter().sum();
        let mut b = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            idx.sd_batch(&xs, &ys, &mut got);
            b.push(t0.elapsed().as_secs_f64() * 1e9 / N as f64);
            acc += got[0];
        }
        b.sort_by(f64::total_cmp);
        assert!(
            acc.is_finite(),
            "o acumulador existe para o laço não ser optimizado para fora"
        );
        // As duas metades, separadas: distância (BVH) e sinal (grelha).
        let half = |f: &dyn Fn()| {
            let mut t = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                f();
                t.push(t0.elapsed().as_secs_f64() * 1e9 / N as f64);
            }
            t.sort_by(f64::total_cmp);
            t[2]
        };
        let both = half(&|| {
            let _ = idx.probe_dist_only(&xs, &ys);
        });
        println!(
            "{n:>7} | {:>12.1} | {:>16.1} | {:>4.1}x | {:>7.2} ms | metades juntas {both:>6.1}",
            a[2],
            b[2],
            a[2] / b[2],
            build[2]
        );
    }
}

/// ⚠️ **Onde os nanossegundos estão** — a sonda que separa a distância do sinal.
#[test]
#[ignore]
fn the_table_of_which_half_pays() {
    const N: usize = 200_000;
    let (xs, ys) = cloud(N, 0.9);
    println!("arestas | distância (ns/pt) | sinal (ns/pt)");
    for n in [56usize, 168, 664] {
        let p = Profile::new(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero, 1e-3)
            .expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        let time = |f: &dyn Fn() -> f32| {
            let mut t = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let v = f();
                t.push(t0.elapsed().as_secs_f64() * 1e9 / N as f64);
                assert!(v.is_finite());
            }
            t.sort_by(f64::total_cmp);
            t[2]
        };
        let d = time(&|| idx.probe_dist_only(&xs, &ys));
        let w = time(&|| idx.probe_sign_only(&xs, &ys));
        println!("{n:>7} | {d:>17.1} | {w:>13.1}");
    }
}

/// ⚠️ **A NUVEM UNIFORME NÃO CONTÉM O FENÓMENO** — uma marcha não amostra o interior assim.
///
/// Uma esfera-marcha caminha **de fora para dentro** e pára na superfície: ela pede muitos pontos
/// afastados, alguns colados à casca, e quase nenhum no miolo. E o miolo é exactamente onde uma
/// busca do segmento mais próximo é patológica — no centro de um círculo **todas** as arestas estão
/// à mesma distância, e nenhuma estrutura poda o que é equidistante.
#[test]
#[ignore]
fn the_table_of_where_the_distance_is_expensive() {
    const N: usize = 100_000;
    let mut s = 0x1234_5678u64;
    let mut rnd = move || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (s >> 33) as f32 / u32::MAX as f32
    };
    // Três dietas, todas em torno do círculo de raio 0,5.
    let ring = |lo: f32, hi: f32, rnd: &mut dyn FnMut() -> f32| {
        let (mut xs, mut ys) = (Vec::with_capacity(N), Vec::with_capacity(N));
        for _ in 0..N {
            let a = rnd() * std::f32::consts::TAU;
            let r = lo + (hi - lo) * rnd();
            xs.push(r * a.cos());
            ys.push(r * a.sin());
        }
        (xs, ys)
    };
    let diets = [
        ("longe (1,0..2,0 R)", ring(0.5, 1.0, &mut rnd)),
        (
            "colado à casca (0,95..1,05 R)",
            ring(0.475, 0.525, &mut rnd),
        ),
        ("no miolo (0..0,5 R)", ring(0.0, 0.25, &mut rnd)),
    ];
    println!(
        "arestas | {:>20} | {:>20} | {:>20}",
        diets[0].0, diets[1].0, diets[2].0
    );
    for n in [56usize, 168, 664] {
        let p = Profile::new(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero, 1e-3)
            .expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        let mut cols = Vec::new();
        for (_, (xs, ys)) in &diets {
            let mut t = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let v = idx.probe_dist_only(xs, ys);
                t.push(t0.elapsed().as_secs_f64() * 1e9 / N as f64);
                assert!(v.is_finite());
            }
            t.sort_by(f64::total_cmp);
            cols.push(t[2]);
        }
        println!(
            "{n:>7} | {:>17.1} ns | {:>17.1} ns | {:>17.1} ns",
            cols[0], cols[1], cols[2]
        );
    }
}

/// ⭐⭐⭐ **O QUE O CORTE COMPRA, por compacidade do lote** — a tabela que escolhe o tamanho do lote.
///
/// ⚠️ O corte mede **quem o chamou**: uma linha inteira de ecrã tem pegada larga e não corta nada.
/// A tabela varre a pegada (em frações do raio da peça) e diz quantas arestas sobram e quanto custa.
#[test]
#[ignore]
fn the_table_of_what_culling_buys_by_batch_compactness() {
    const N: usize = 100_000;
    let mut s = 0xABCD_1234u64;
    let mut rnd = move || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (s >> 33) as f32 / u32::MAX as f32
    };
    println!("arestas | pegada | arestas após o corte | ns/ponto | vs fita");
    for n in [168usize, 664] {
        let p = Profile::new(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero, 1e-3)
            .expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        // A fita, para a mesma dieta.
        let tape_ns = match n {
            168 => 154.2,
            _ => 614.3,
        };
        for foot in [1.0f32, 0.5, 0.25, 0.125, 0.0625] {
            // Lotes compactos colados à casca: o que uma marcha por ladrilho de facto pede.
            let per = 1024usize;
            let mut chunks: Vec<(Vec<f32>, Vec<f32>)> = Vec::new();
            let mut kept = 0usize;
            while chunks.len() * per < N {
                let a = rnd() * std::f32::consts::TAU;
                let (cx, cy) = (0.5 * a.cos(), 0.5 * a.sin());
                let half = 0.5 * foot;
                let (mut xs, mut ys) = (Vec::with_capacity(per), Vec::with_capacity(per));
                for _ in 0..per {
                    xs.push(cx + (rnd() - 0.5) * half * 2.0);
                    ys.push(cy + (rnd() - 0.5) * half * 2.0);
                }
                kept += idx.probe_cull(
                    [
                        xs.iter().copied().fold(f32::INFINITY, f32::min),
                        ys.iter().copied().fold(f32::INFINITY, f32::min),
                    ],
                    [
                        xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
                        ys.iter().copied().fold(f32::NEG_INFINITY, f32::max),
                    ],
                );
                chunks.push((xs, ys));
            }
            let mut scratch = Vec::new();
            let mut out = Vec::new();
            let mut t = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let mut acc = 0.0f32;
                for (xs, ys) in &chunks {
                    idx.sd_batch_culled(xs, ys, &mut scratch, &mut out);
                    acc += out[0];
                }
                t.push(t0.elapsed().as_secs_f64() * 1e9 / (chunks.len() * per) as f64);
                assert!(acc.is_finite());
            }
            t.sort_by(f64::total_cmp);
            println!(
                "{n:>7} | {foot:>6.3} | {:>20.1} | {:>8.1} | {:>6.1}x",
                kept as f64 / chunks.len() as f64,
                t[2],
                tape_ns / t[2]
            );
        }
    }
}

/// ⭐⭐⭐ **O QUE A ESPECIALIZAÇÃO COMPRA** — a tabela que decide o desenho da wave.
///
/// ⚠️ `#[ignore]` porque mede relógio — máquina calma:
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     profile_index::tests::the_table_of_what_specialising_the_tree_buys --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_what_specialising_the_tree_buys() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    const N: usize = 200_000;
    let mut s = 0x0BAD_F00Du64;
    let mut rnd = move || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (s >> 33) as f32 / u32::MAX as f32
    };
    println!("arestas | pegada | dist+cruz | montar | ns/ponto | vs fita completa");
    for n in [168usize, 664] {
        let p = Profile::new(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero, 1e-3)
            .expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        let full_ns = if n == 168 { 155.0 } else { 636.0 };
        for foot in [0.25f32, 0.125, 0.0625] {
            let half = 0.5 * foot * 0.5;
            // Uma região colada à casca — onde a marcha de facto pára.
            let a = rnd() * std::f32::consts::TAU;
            let c = [0.5 * a.cos(), 0.5 * a.sin()];
            let lo = [c[0] - half, c[1] - half];
            let hi = [c[0] + half, c[1] + half];
            let (near, cross) = (
                idx.distance_edges(lo, hi).len(),
                idx.crossing_edges(lo, hi).len(),
            );
            let mut build = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let t = crate::profile::sd_profile_in_region(
                    &p,
                    &idx,
                    &Tree::x(),
                    &Tree::y(),
                    lo,
                    hi,
                    false,
                );
                let shape = crate::Engine::from(t);
                let tape = shape.ez_float_slice_tape();
                build.push(t0.elapsed().as_secs_f64() * 1e3);
                drop(tape);
            }
            build.sort_by(f64::total_cmp);
            let t = crate::profile::sd_profile_in_region(
                &p,
                &idx,
                &Tree::x(),
                &Tree::y(),
                lo,
                hi,
                false,
            );
            let shape = crate::Engine::from(t);
            let mut eval = crate::Engine::new_float_slice_eval();
            let tape = shape.ez_float_slice_tape();
            let (mut xs, mut ys) = (Vec::with_capacity(N), Vec::with_capacity(N));
            for _ in 0..N {
                xs.push((rnd() - 0.5).mul_add(2.0 * half, c[0]));
                ys.push((rnd() - 0.5).mul_add(2.0 * half, c[1]));
            }
            let zs = vec![0.0f32; N];
            let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
            let mut ms = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
                ms.push(t0.elapsed().as_secs_f64() * 1e9 / N as f64);
            }
            ms.sort_by(f64::total_cmp);
            println!(
                "{n:>7} | {foot:>6.3} | {near:>4}+{cross:<4} | {:>5.2} ms | {:>8.1} | {:>16.1}x",
                build[2],
                ms[2],
                full_ns / ms[2]
            );
        }
    }
}
