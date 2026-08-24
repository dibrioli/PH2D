//! Os gates da CONSULTA — ver [`super`].

use super::ProfileIndex;
use ph2d_field::{FillRule, Profile};

fn ngon(n: usize, r: f64, c: [f64; 2]) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(c[0] + r * a.cos()) as f32, (c[1] + r * a.sin()) as f32]
        })
        .collect()
}

/// A árvore, avaliada ponto a ponto — o ORÁCULO desta wave.
fn tape_of(p: &Profile) -> impl FnMut(&[f32], &[f32]) -> Vec<f32> {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    let tree = crate::profile::sd_profile(p, &Tree::x(), &Tree::y());
    let shape = crate::Engine::from(tree);
    let mut eval = crate::Engine::new_float_slice_eval();
    let tape = shape.ez_float_slice_tape();
    move |xs: &[f32], ys: &[f32]| {
        let zs = vec![0.0f32; xs.len()];
        eval.eval(&tape, xs, ys, &zs).expect("avalia").to_vec()
    }
}

/// Uma nuvem determinística à volta da caixa do perfil.
fn cloud(n: usize, half: f32) -> (Vec<f32>, Vec<f32>) {
    let mut s = 0x5DEE_CE66u64;
    let mut rnd = move || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((s >> 33) as f32 / u32::MAX as f32) - 0.5
    };
    let (mut xs, mut ys) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        xs.push(rnd() * half * 2.0);
        ys.push(rnd() * half * 2.0);
    }
    (xs, ys)
}

/// ⭐⭐ **O GATE-MÃE: a consulta é a MESMA LEI que a fita.**
///
/// ⚠️ **Dois motores, uma lei — e a lei tem um juiz.** É o mesmo compromisso (e a mesma defesa) do
/// [`crate::hybrid`], onde as booleanas existem como árvore *e* como aritmética `f32`. Aqui a árvore
/// é o **oráculo**: ela shipou desde a W3, foi medida contra o `Cylinder` e o `Torus` analíticos, e
/// é ela que define o que "dentro" quer dizer.
///
/// A barra é `1e-5` em unidades do perfil (raio `0,5`) — o resíduo de somar em `f32` por caminhos
/// diferentes, e não uma folga para a consulta discordar.
#[test]
fn the_query_is_the_same_law_as_the_tape() {
    for (name, contours, fill) in [
        (
            "polígono de 168 lados",
            vec![ngon(168, 0.5, [0.0, 0.0])],
            FillRule::NonZero,
        ),
        (
            "quadrado",
            vec![vec![[-0.4f32, -0.4], [0.4, -0.4], [0.4, 0.4], [-0.4, 0.4]]],
            FillRule::NonZero,
        ),
        (
            "anel (buraco, NonZero)",
            vec![ngon(48, 0.5, [0.0, 0.0]), {
                let mut inner = ngon(32, 0.22, [0.0, 0.0]);
                inner.reverse();
                inner
            }],
            FillRule::NonZero,
        ),
        (
            "anel (buraco, EvenOdd)",
            vec![ngon(48, 0.5, [0.0, 0.0]), ngon(32, 0.22, [0.0, 0.0])],
            FillRule::EvenOdd,
        ),
        (
            "duas ilhas separadas",
            vec![ngon(24, 0.2, [-0.35, 0.0]), ngon(24, 0.2, [0.35, 0.0])],
            FillRule::NonZero,
        ),
    ] {
        let p = Profile::new(contours, fill, 1e-3).expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        let mut tape = tape_of(&p);
        // ⚠️ A nuvem cobre **1,5× a caixa**: metade das amostras cai FORA dela, que é onde a marcha
        // de facto passa o tempo — e é o caminho em que o enrolamento é dispensado por ser zero.
        let (xs, ys) = cloud(20_000, 0.9);
        let want = tape(&xs, &ys);
        let mut worst = 0.0f32;
        let mut at = (0.0f32, 0.0f32);
        for i in 0..xs.len() {
            let got = idx.sd(xs[i], ys[i]);
            let e = (got - want[i]).abs();
            if e > worst {
                worst = e;
                at = (xs[i], ys[i]);
            }
        }
        assert!(
            worst < 1.0e-5,
            "{name}: a consulta discorda da fita em {worst:e} (pior em {at:?}) — dois motores, \
             duas leis"
        );
    }
}

/// ⚠️ **O sinal tem de estar CERTO, e não só a magnitude.**
///
/// Uma consulta que devolvesse sempre `+|d|` passaria num gate de erro absoluto que só olhasse
/// pontos de fora — e a nuvem tem metade deles. Este mede o que a outra metade decide.
#[test]
fn the_query_knows_inside_from_outside() {
    let p = Profile::new(vec![ngon(64, 0.5, [0.0, 0.0])], FillRule::NonZero, 1e-3)
        .expect("perfil válido");
    let idx = ProfileIndex::build(&p);
    assert!(idx.sd(0.0, 0.0) < 0.0, "o centro está dentro");
    assert!(idx.sd(0.9, 0.0) > 0.0, "longe está fora");
    // …e o valor no centro é o raio inscrito, a menos da flecha do polígono.
    let insc = 0.5 * (std::f32::consts::PI / 64.0).cos();
    assert!(
        (idx.sd(0.0, 0.0) + insc).abs() < 1.0e-4,
        "no centro a distância é o raio inscrito: {} contra {insc}",
        idx.sd(0.0, 0.0)
    );
}

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

/// ⭐⭐ **O CORTE NUNCA DEITA FORA A ARESTA MAIS PRÓXIMA** — e o juiz continua a ser a fita.
///
/// ⚠️ **É o gate que o coração desta wave exigia.** Deitar fora uma aresta que podia ser a mais
/// próxima faz a distância sair **maior** que a verdadeira, e uma esfera-marcha que sobre-estima o
/// passo **atravessa a peça** — o defeito não apareceria como um número errado num teste de
/// unidade, apareceria como um buraco na imagem.
///
/// Os lotes são **compactos de propósito** (é o regime em que o corte de facto corta): um corte que
/// só estivesse certo com a pegada do perfil inteiro passaria num gate de lote largo e falharia
/// exactamente onde vai ser usado.
#[test]
fn the_cull_never_drops_the_nearest_edge() {
    for (name, contours, fill) in [
        (
            "polígono de 168 lados",
            vec![ngon(168, 0.5, [0.0, 0.0])],
            FillRule::NonZero,
        ),
        (
            "anel com buraco",
            vec![ngon(64, 0.5, [0.0, 0.0]), {
                let mut i = ngon(48, 0.25, [0.0, 0.0]);
                i.reverse();
                i
            }],
            FillRule::NonZero,
        ),
        (
            "duas ilhas",
            vec![ngon(24, 0.2, [-0.35, 0.0]), ngon(24, 0.2, [0.35, 0.0])],
            FillRule::NonZero,
        ),
    ] {
        let p = Profile::new(contours, fill, 1e-3).expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        let mut tape = tape_of(&p);
        let mut s = 0x9E37_79B9u64;
        let mut rnd = move || {
            s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (s >> 33) as f32 / u32::MAX as f32
        };
        let (mut scratch, mut got) = (Vec::new(), Vec::new());
        let mut worst = 0.0f32;
        // Lotes compactos espalhados por todo o plano — dentro, fora, e colados à casca.
        for _ in 0..400 {
            let c = [(rnd() - 0.5) * 1.8, (rnd() - 0.5) * 1.8];
            let half = 0.5f32.mul_add(rnd(), 0.01);
            let (mut xs, mut ys) = (Vec::new(), Vec::new());
            for _ in 0..64 {
                xs.push((rnd() - 0.5).mul_add(2.0 * half, c[0]));
                ys.push((rnd() - 0.5).mul_add(2.0 * half, c[1]));
            }
            idx.sd_batch_culled(&xs, &ys, &mut scratch, &mut got);
            let want = tape(&xs, &ys);
            for i in 0..xs.len() {
                worst = worst.max((got[i] - want[i]).abs());
            }
        }
        assert!(
            worst < 1.0e-5,
            "{name}: o lote cortado discorda da fita em {worst:e} — o corte deitou fora a aresta \
             mais próxima, e a marcha atravessaria a peça"
        );
    }
}

/// ⭐⭐⭐ **A ÁRVORE ESPECIALIZADA CONCORDA COM A COMPLETA — DENTRO DA REGIÃO DELA.**
///
/// ⚠️ **É o gate que autoriza a wave inteira.** Se a especialização discordar, o traçado ganha uma
/// imagem errada em troca de velocidade — e o modo de falha é o pior: uma distância **maior** que a
/// verdadeira faz a esfera-marcha **atravessar a peça**, o que se lê como um buraco, não como um
/// número errado.
///
/// ⚠️ **E ele mede as DUAS pontas**: dentro da região tem de concordar; e a fixture inclui regiões
/// que atravessam a fronteira do perfil, que é onde o enrolamento pré-somado tem de estar certo.
#[test]
fn the_specialised_tree_agrees_inside_its_region() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    for (name, contours, fill) in [
        (
            "polígono de 168 lados",
            vec![ngon(168, 0.5, [0.0, 0.0])],
            FillRule::NonZero,
        ),
        (
            "anel com buraco (NonZero)",
            vec![ngon(64, 0.5, [0.0, 0.0]), {
                let mut i = ngon(48, 0.25, [0.0, 0.0]);
                i.reverse();
                i
            }],
            FillRule::NonZero,
        ),
        (
            "anel com buraco (EvenOdd)",
            vec![ngon(64, 0.5, [0.0, 0.0]), ngon(48, 0.25, [0.0, 0.0])],
            FillRule::EvenOdd,
        ),
        (
            "duas ilhas",
            vec![ngon(24, 0.2, [-0.35, 0.0]), ngon(24, 0.2, [0.35, 0.0])],
            FillRule::NonZero,
        ),
    ] {
        let p = Profile::new(contours, fill, 1e-3).expect("perfil válido");
        let idx = ProfileIndex::build(&p);
        let mut full = tape_of(&p);
        let mut s = 0xF00D_BEEFu64;
        let mut rnd = move || {
            s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (s >> 33) as f32 / u32::MAX as f32
        };
        let (mut worst, mut nodes_full, mut nodes_cut, mut regions) =
            (0.0f32, 0usize, 0usize, 0usize);
        for _ in 0..120 {
            // Regiões espalhadas: dentro, fora, e a cavalo da fronteira.
            let c = [(rnd() - 0.5) * 1.6, (rnd() - 0.5) * 1.6];
            let half = 0.15f32.mul_add(rnd(), 0.01);
            let lo = [c[0] - half, c[1] - half];
            let hi = [c[0] + half, c[1] + half];
            let cut =
                crate::profile::sd_profile_in_region(&p, &idx, &Tree::x(), &Tree::y(), lo, hi);
            let shape = crate::Engine::from(cut);
            let mut eval = crate::Engine::new_float_slice_eval();
            let tape = shape.ez_float_slice_tape();
            let (mut xs, mut ys) = (Vec::new(), Vec::new());
            for _ in 0..256 {
                xs.push((rnd() - 0.5).mul_add(2.0 * half, c[0]));
                ys.push((rnd() - 0.5).mul_add(2.0 * half, c[1]));
            }
            let zs = vec![0.0f32; xs.len()];
            let got = eval.eval(&tape, &xs, &ys, &zs).expect("avalia").to_vec();
            let want = full(&xs, &ys);
            for i in 0..xs.len() {
                worst = worst.max((got[i] - want[i]).abs());
            }
            nodes_cut += idx.distance_edges(lo, hi).len() + idx.crossing_edges(lo, hi).len();
            nodes_full += idx.edge_count();
            regions += 1;
        }
        assert!(
            worst < 1.0e-5,
            "{name}: a árvore especializada discorda da completa em {worst:e} DENTRO da região — \
             a marcha atravessaria a peça"
        );
        // ⚠️ **A metade que impede a cura degenerada**: uma especialização que guardasse todas as
        // arestas passaria no gate acima e não compraria nada.
        let ratio = nodes_cut as f64 / nodes_full as f64;
        assert!(
            ratio < 0.5,
            "{name}: a especialização guardou {:.0}% das arestas em média ({regions} regiões) — \
             ela concorda e não compra nada",
            ratio * 100.0
        );
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
                let t =
                    crate::profile::sd_profile_in_region(&p, &idx, &Tree::x(), &Tree::y(), lo, hi);
                let shape = crate::Engine::from(t);
                let tape = shape.ez_float_slice_tape();
                build.push(t0.elapsed().as_secs_f64() * 1e3);
                drop(tape);
            }
            build.sort_by(f64::total_cmp);
            let t = crate::profile::sd_profile_in_region(&p, &idx, &Tree::x(), &Tree::y(), lo, hi);
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
