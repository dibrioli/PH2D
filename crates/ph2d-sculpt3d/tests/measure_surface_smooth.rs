//! **O QUE O SURFACE SMOOTH COMPRA, E O QUE ELE COBRA** — a sonda que decide os
//! dois defaults do HC.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_surface_smooth \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⚠️ **Os dois números do Blender NÃO são legíveis** (`.blend` binário desde o
//! 4.3 — a §7.0 do plano 21 mediu essa parede para a W1 inteira) e o paper não
//! está nesta máquina. Herdar um valor de memória e escrevê-lo com a autoridade
//! de uma fonte que não o declara é exactamente a linha que a §4 do plano
//! proíbe. ⇒ **medir**, que é o que o §0 do `CLAUDE.md` manda fazer com todo
//! número que alguém ia escolher.
//!
//! ⚠️ **A fixture é uma esfera RUGOSA, e a irmã lisa não serve:** o HC existe
//! para *alisar ruído sem encolher*, e numa esfera exacta a primeira metade não
//! tem sobre o que agir. É a mesma lição que a `uv_sphere_shuffled` pagou no
//! Slide Relax, do outro lado — *a fixture tem de conter o fenômeno*.
//!
//! ⚠️ **E o [`Verb::Smooth`] é o CONTROLE em toda tabela.** Sozinho, *"o HC
//! alisou e o raio quase não mexeu"* também é verdade para um pincel que não faz
//! nada; o que decide é a RAZÃO entre as duas colunas nos dois verbos.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// A esfera com 2 % de ruído radial — a *noisy surface mesh* do título do paper.
fn noisy() -> Mesh {
    ph2d_mesh::shapes::uv_sphere_noisy(48, 96, 1.0, 0.02)
}

/// A MESMA esfera sem ruído — o **PISO** da rugosidade.
///
/// ⚠️ **Ele existe porque a régua não chega a zero, e sem ele a tabela mente
/// para baixo:** uma `uv_sphere` LIMPA já tem `|p − média do anel|` ≠ 0 (a
/// curvatura mais o espaçamento desigual dos anéis), então *"resta 71 %"* lido
/// contra zero é uma frase sobre a tesselação e não sobre o ruído. O que o
/// pincel pode remover é o que está ACIMA deste piso.
fn clean() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(48, 96, 1.0)
}

/// **A FORMA** — o raio médio. `1,0` numa esfera unitária intacta, e o ruído
/// radial é simétrico, então ele também vale ~`1,0` na fixture suja: todo desvio
/// é encolhimento.
fn mean_radius(mesh: &Mesh) -> f64 {
    let p = mesh.positions();
    p.iter()
        .map(|v| f64::from(v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2]))).sqrt())
        .sum::<f64>()
        / p.len() as f64
}

/// **O RUÍDO** — a rugosidade de ALTA frequência: o RMS de `|p − média do
/// anel|`, que é a magnitude do próprio deslocamento laplaciano.
///
/// ⚠️ **A primeira versão desta função era o desvio-padrão do RAIO, e ela
/// MENTIU:** com 32 passadas de Smooth ela subia de volta a 96,1 % da base
/// enquanto a esfera encolhia 5 %, e a leitura óbvia — *"o Smooth deixou de
/// alisar"* — é falsa. O que acontece é que uma `uv_sphere` tem anéis de
/// espaçamento desigual, então o laplaciano encolhe os polos muito mais que o
/// equador: o desvio do raio passou a medir **deformação de baixa frequência**,
/// que é a coisa que a coluna do lado já mede.
///
/// *Uma régua de ruído que responde a mudança de FORMA não separa as duas
/// perguntas que este verbo existe para separar.*
fn roughness(mesh: &Mesh) -> f64 {
    let adj = mesh.adjacency();
    let p = mesh.positions();
    let mut acc = 0.0f64;
    for (i, q) in p.iter().enumerate() {
        let avg = ph2d_mesh::ring_average(adj, i as u32, *q, |nb| p[nb as usize]);
        let d = [avg[0] - q[0], avg[1] - q[1], avg[2] - q[2]];
        acc += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])));
    }
    (acc / p.len() as f64).sqrt()
}

/// Um dab que cobre a esfera INTEIRA — o *Filter Layer*, onde o encolhimento é
/// o efeito e não um detalhe de borda.
fn whole_sphere_dab() -> Dab {
    Dab::at([0.0, 0.0, 0.0], 4.0, [0.0, 0.0, -1.0])
}

fn run(verb: Verb, alpha: f32, beta: f32, dabs: usize) -> (f64, f64) {
    let mut mesh = noisy();
    let b = Brush {
        verb,
        radius: 4.0,
        strength: 1.0,
        // `Constant` de propósito: com curva macia o peso cairia com a distância
        // e o número falaria do FALLOFF junto. O que se mede é o operador.
        falloff: Falloff::Constant,
        hc_shape: alpha,
        hc_vertex: beta,
        ..Brush::default()
    };
    // ⚠️ **UM traço, N dabs** — o `b` é do TRAÇO, e recomeçá-lo a cada passada
    // mediria um verbo que não existe (o `α` pesa contra o pen-down, e um
    // pen-down por passada seria `α` inerte).
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for _ in 0..dabs {
        s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
    }
    (mean_radius(&mesh), roughness(&mesh))
}

/// **A VARREDURA que escolhe os dois defaults.**
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_what_the_two_hc_knobs_buy_and_cost() {
    let base = noisy();
    let floor = roughness(&clean());
    let (r0, n0) = (mean_radius(&base), roughness(&base));
    println!("\n  base: raio {r0:.6}  rugosidade {n0:.6}");
    println!(
        "  piso (esfera LIMPA): {floor:.6}  =>  ruido removivel = {:.6}",
        n0 - floor
    );
    let left = |n: f64| ((n - floor) / (n0 - floor) * 100.0).max(0.0);

    for dabs in [1usize, 8, 32] {
        println!("\n  === {dabs} dab(s) ===");
        println!("     verbo         alfa   beta    raio     encolhe    ruido    resta");
        println!("     -----------   ----   ----   -------   -------   -------   -----");
        let (r, n) = run(Verb::Smooth, 0.0, 0.0, dabs);
        println!(
            "     Smooth          -      -    {r:.5}   {:>6.3}%   {n:.5}   {:>4.1}%",
            (r0 - r) / r0 * 100.0,
            left(n)
        );
        for alpha in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            for beta in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
                let (r, n) = run(Verb::SurfaceSmooth, alpha, beta, dabs);
                println!(
                    "     Surface       {alpha:.2}   {beta:.2}   {r:.5}   {:>6.3}%   {n:.5}   {:>4.1}%",
                    (r0 - r) / r0 * 100.0,
                    left(n)
                );
            }
        }
    }
}

/// **O CANTO DEGENERADO, medido em vez de deduzido:** `α = 0` com `β = 1`
/// subtrai exactamente o que o passo laplaciano somou.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_the_no_op_corner() {
    let base = noisy();
    let before: Vec<[f32; 3]> = base.positions().to_vec();
    let mut mesh = noisy();
    let b = Brush {
        verb: Verb::SurfaceSmooth,
        radius: 4.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        hc_shape: 0.0,
        hc_vertex: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
    let worst = before
        .iter()
        .zip(mesh.positions())
        .map(|(a, c)| {
            let d = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))).sqrt()
        })
        .fold(0.0f64, f64::max);
    println!("\n  alfa 0 / beta 1: pior deslocamento = {worst:.9}");
}

/// **O CUSTO** — quantas vezes um dab de HC custa um dab de Smooth.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_what_the_hc_costs() {
    use std::time::Instant;

    // ⚠️ **Leia a RAZÃO, nunca os milissegundos.** Esta workstation partilha 32
    // núcleos com as outras linhas, e a MESMA corrida deu 0,53 e 1,52 ms/dab
    // para o mesmo binário com `load average` de 2 e de 17. O que sobrevive à
    // carga é a comparação feita DENTRO da corrida, porque ali ela é fator
    // comum.
    //
    // ⚠️ **ALTERNADO e por MEDIANA, e as duas metades são a correção.** A 1ª
    // versão cronometrava um verbo e depois o outro, uma vez cada, e reportou o
    // HC como **1,4× mais RÁPIDO** que o irmão — impossível por construção: ele
    // faz tudo o que o Smooth faz (a média do anel) e mais uma passada de
    // preparação e uma segunda média por vértice. *Um número que contradiz a
    // contagem de trabalho é a máquina a falar, não o código*, e é a mesma
    // lição que a `line/Painter` escreveu ao medir duas rotas: cronometre-as
    // costas-com-costas DENTRO da corrida, sobre o mesmo estado, para a carga
    // ser um fator comum.
    const ROUNDS: usize = 7;
    const DABS: usize = 20;
    let mut samples: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut passes = [0usize; 2];
    for _ in 0..ROUNDS {
        for (slot, verb) in [Verb::Smooth, Verb::SurfaceSmooth].into_iter().enumerate() {
            let mut mesh = noisy();
            let b = Brush {
                verb,
                radius: 4.0,
                strength: 1.0,
                falloff: Falloff::Constant,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            // Aquece: a captura da pegada inteira acontece no primeiro dab.
            s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            let t = Instant::now();
            for _ in 0..DABS {
                s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            }
            samples[slot].push(t.elapsed().as_secs_f64() * 1000.0 / DABS as f64);
            passes[slot] = b.passes().len();
        }
    }
    let median = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let smooth = median(&mut samples[0]);
    let hc = median(&mut samples[1]);
    println!(
        "\n  {} vertices, mediana de {ROUNDS} rodadas alternadas",
        noisy().vert_count()
    );
    println!("  {:>14}: {smooth:.3} ms/dab", Verb::Smooth.label());
    println!("  {:>14}: {hc:.3} ms/dab", Verb::SurfaceSmooth.label());
    println!("  passes: smooth {} hc {}", passes[0], passes[1]);
    println!("  o HC cobra {:.2}x o irmao", hc / smooth);
}

/// **A TRAJETÓRIA, dab a dab** — a tabela agregada diz *onde parou*, e isso não
/// distingue *convergiu* de *oscila com a mesma amplitude*.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_the_roughness_dab_by_dab() {
    let floor = roughness(&clean());
    let n0 = roughness(&noisy());
    println!("\n  piso {floor:.6}   base {n0:.6}");
    for (verb, alpha, beta) in [
        (Verb::Smooth, 0.0f32, 0.0f32),
        (Verb::SurfaceSmooth, 0.0, 0.50),
        (Verb::SurfaceSmooth, 0.0, 0.75),
        (Verb::SurfaceSmooth, 0.0, 0.90),
        (Verb::SurfaceSmooth, 0.0, 0.60),
    ] {
        let mut mesh = noisy();
        let b = Brush {
            verb,
            radius: 4.0,
            strength: 1.0,
            falloff: Falloff::Constant,
            hc_shape: alpha,
            hc_vertex: beta,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        print!("\n  {:>14} a={alpha:.2} b={beta:.2} :", verb.label());
        for d in 1..=12 {
            s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            if d <= 6 || d % 3 == 0 {
                print!(" {:.4}", (roughness(&mesh) - floor) / (n0 - floor));
            }
        }
        println!();
    }
}

/// **O PESO que o dab de facto entrega** — antes de culpar o kernel.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_the_weight_the_dab_delivers() {
    let d = whole_sphere_dab();
    for verb in [Verb::Smooth, Verb::SurfaceSmooth] {
        let b = Brush {
            verb,
            radius: 4.0,
            strength: 1.0,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        println!(
            "\n  {:>14}: weight()={:.4}  pressure={:.4}  modo={:?}",
            verb.label(),
            b.weight(),
            d.pressure,
            b.mode
        );
    }
}

/// **A FRONTEIRA DE ESTABILIDADE do β** — onde o operador deixa de contrair.
///
/// A tabela agregada mostrou `β = 0` a explodir (16 534 % da rugosidade base) e
/// `β = 0,5` a convergir; entre os dois há uma linha, e um slider que a alcança
/// é um slider que rebenta a malha.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_where_the_beta_stops_contracting() {
    let floor = roughness(&clean());
    let n0 = roughness(&noisy());
    println!("\n  beta    rug@4     rug@16    veredito");
    println!("  ----   -------   -------   --------");
    for i in 0u8..=14 {
        let beta = 0.30 + f32::from(i) * 0.025;
        let mut mesh = noisy();
        let b = Brush {
            verb: Verb::SurfaceSmooth,
            radius: 4.0,
            strength: 1.0,
            falloff: Falloff::Constant,
            hc_shape: 0.0,
            hc_vertex: beta,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        let mut r4 = 0.0;
        for d in 1..=16 {
            s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            if d == 4 {
                r4 = (roughness(&mesh) - floor) / (n0 - floor);
            }
        }
        let r16 = (roughness(&mesh) - floor) / (n0 - floor);
        println!(
            "  {beta:.3}   {r4:>7.4}   {r16:>7.4}   {}",
            if r16 > r4 { "DIVERGE" } else { "contrai" }
        );
    }
}

/// **O QUE O α CONTROLA** — e a régua não é o raio.
///
/// ⚠️ Na tabela agregada o α move a coluna de encolhimento de `−0,022 %` para
/// `−0,006 %`: ruído. É o oráculo errado — o `α` ancora o `b` na pose do
/// **pen-down**, logo o que ele governa é **a DERIVA em relação a ela**, e num
/// traço longo é isso que o artista vê.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_what_the_alpha_holds_on_to() {
    let floor = roughness(&clean());
    let n0 = roughness(&noisy());
    let o: Vec<[f32; 3]> = noisy().positions().to_vec();
    println!("\n  alfa    deriva media   deriva pior   rugosidade resta");
    println!("  ----   ------------   -----------   ----------------");
    for i in 0u8..=4 {
        let alpha = f32::from(i) * 0.25;
        let mut mesh = noisy();
        let b = Brush {
            verb: Verb::SurfaceSmooth,
            radius: 4.0,
            strength: 1.0,
            falloff: Falloff::Constant,
            hc_shape: alpha,
            hc_vertex: 0.5,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for _ in 0..32 {
            s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
        }
        let d: Vec<f64> = o
            .iter()
            .zip(mesh.positions())
            .map(|(a, c)| {
                let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                f64::from(v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2]))).sqrt()
            })
            .collect();
        println!(
            "  {alpha:.2}   {:>12.6}   {:>11.6}   {:>15.1}%",
            d.iter().sum::<f64>() / d.len() as f64,
            d.iter().copied().fold(0.0f64, f64::max),
            (roughness(&mesh) - floor) / (n0 - floor) * 100.0
        );
    }
}

/// **O CUSTO DE UM DAB MUDA COM O DAB, E A PEGADA DIZ POR QUÊ** — a sonda que
/// desqualificou um controle e deixou um número sem atribuição.
///
/// ⚠️ **O [`Verb::Sharpen`] parecia o controle perfeito** (o MESMO anel, a mesma
/// forma, só o sinal trocado) e mediu **3× mais barato** que o irmão. A pegada
/// impressa ao lado mostra por quê: ela cai **4514 → 3961 → 2007 → 761 → 194 →
/// 12 → 0**. Ele *diverge*: empurra os vértices para fora do dab, e a partir do
/// 6º carimbo **deixa de trabalhar**. *Um controle que fica mais barato porque
/// parou de fazer o trabalho não é um controle* — e foi a coluna da pegada, não
/// o relógio, que o disse.
///
/// ⚠️ **E o que fica MEDIDO e NÃO ATRIBUÍDO:** com a pegada idêntica (4514 nos
/// dois, todo dab) e o mesmo número de passes, o HC mede **~0,37 contra ~0,53
/// ms/dab** do [`Verb::Smooth`] — reproduzível, plano desde o primeiro carimbo e
/// **independente da ordem** (medido com o irmão duas vezes na mesma lista). Isso
/// CONTRADIZ a contagem de trabalho: o braço do HC contém o do Smooth e ainda
/// percorre o anel mais duas vezes. Ablar a média dos `b` tira só **0,04 ms**
/// (o `hc_b` é indexado por slot e denso, ao contrário do `positions()`), então
/// a diferença **não é o código desta wave**. *Uma ablação atribui a um BLOCO;
/// dizer qual linha dentro dele é inferência de segunda ordem* — e a causa do
/// braço do Smooth ser mais caro que o braço que o contém segue **em aberto,
/// nomeada em vez de explicada**.
#[test]
#[ignore = "sonda: roda com --ignored --nocapture"]
fn measure_the_cost_dab_by_dab() {
    use std::time::Instant;
    for verb in [Verb::Smooth, Verb::Sharpen, Verb::SurfaceSmooth] {
        let mut mesh = noisy();
        let b = Brush {
            verb,
            radius: 4.0,
            strength: 1.0,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        print!("\n  {:>14}:", verb.label());
        for k in 0..12 {
            let t = Instant::now();
            s.dab(&mut mesh, &b, &whole_sphere_dab(), Symmetry::default());
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            if k > 0 {
                print!(" {ms:.2}/{}", s.footprint_len());
            }
        }
    }
    println!();
}
