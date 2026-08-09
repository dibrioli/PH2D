//! **W0 do [ADR-0157]** — o kill-criterion: *quanto custa compor N dabs por pixel?*
//!
//! O ADR fecha dizendo que este é **o primeiro número da implementação** e que **nenhum passo de grade
//! entra no código antes dele** (`CLAUDE.md` §0: *meça antes de limitar*). Esta sonda é essa medição, e
//! ela é feita pela porta REAL ([`compose_at`] sobre [`DabField`]) — não por uma reescrita local, pelo
//! motivo que esta linha já pagou duas vezes: *uma sonda com laço próprio fica CEGA à porta e segue
//! imprimindo o custo de um código que o produto parou de rodar*.
//!
//! ## Por que o custo do cook tem uma forma diferente do custo de hoje
//!
//! O warp de hoje é **incremental**: cada dab escreve na própria pegada e some — `O(pegada)` por dab,
//! pago uma vez. O cook do ADR-0157 é o oposto: o campo é **derivado**, então todo nó da grade re-anda a
//! lista INTEIRA a cada re-cozimento — `O(nós × N)`. É essa multiplicação que o cache existe para
//! limitar, e é por isso que o número por-avaliação decide o passo da grade em vez de ser detalhe.
//!
//! ## O que é medido
//!
//! * `measure_one_dab_costs` — o ÁTOMO: `DabField::at` por modo, em ns. É ele que multiplica.
//! * `measure_the_cook_is_linear_in_both` — a FORMA: dobrar `N` e dobrar os nós têm de dobrar o custo,
//!   cada um por sua conta. Se a forma não for essa, a tabela de passo não significa nada.
//! * `measure_the_lattice_pitch_table` — a TABELA que o ADR pede, em nós por orçamento de quadro.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1`
//!
//! ⚠️ **`--release` não é preferência** e `--test-threads=1` não é higiene: o perfil `test` mede o
//! compilador, e duas sondas em paralelo disputam os mesmos núcleos e medem uma à outra.
//!
//! [ADR-0157]: ../../../../../../docs/architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use super::field::{DabField, DeformMode, compose_at};
use std::hint::black_box;
use std::time::Instant;

/// A lista de dabs de um Twist parado — o pior caso HONESTO para o cook, porque todo dab cobre o mesmo
/// lugar, então todo nó daquela vizinhança paga a lista inteira. Um traço que ANDA espalha os dabs e
/// cada nó só paga os que o alcançam; medir o espalhado responderia uma pergunta mais fácil.
fn twist_dabs(n: usize, centre: [f32; 2], radius: f32) -> Vec<DabField> {
    (0..n)
        .map(|k| {
            DabField::new(
                DeformMode::Twist,
                centre,
                radius,
                [0.0, 0.0],
                [0.0, 0.0],
                0.0,
                0.8,
                0.0,
                0.0,
                k as u64 + 1,
            )
        })
        .collect()
}

/// Cozinha uma grade `side × side` cobrindo `span` px — a forma exata do laço que o device roda, um nó
/// por invocação, sem estado partilhado.
fn cook(dabs: &[DabField], side: usize, span: f32) -> f32 {
    let step = span / side as f32;
    let mut sink = 0.0_f32;
    for gy in 0..side {
        for gx in 0..side {
            let p = [gx as f32 * step, gy as f32 * step];
            let d = compose_at(black_box(dabs), black_box(p));
            sink += d[0] + d[1];
        }
    }
    sink
}

fn ms(f: impl Fn()) -> f64 {
    let t = Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1e3
}

/// **O ÁTOMO.** `DabField::at` custa quanto, por modo? Tudo o mais nesta wave é este número vezes
/// `nós × N`, então ele é o único que precisa ser pequeno.
///
/// ⚠️ Os modos NÃO custam o mesmo, e a diferença é estrutural em vez de afinação: Push-com-Distortion e
/// **Wrinkle** chamam `value_noise`, que faz **quatro hashes splitmix64** por avaliação — aritmética de
/// **64 bits**, que o WGSL do core não tem. Isso não é um detalhe de velocidade: é a fronteira que
/// decide se o modo pode cozinhar no device sem uma SEGUNDA resposta para *"que ruído é este?"*.
#[test]
#[ignore = "probe: measures, does not assert"]
fn measure_one_dab_costs() {
    const N: usize = 4_000_000;
    println!("\n=== o ÁTOMO do cook: DabField::at ===");
    println!("{:<24} {:>10} {:>12}", "modo", "ms/4M", "ns/eval");
    for (name, mode, distortion) in [
        ("Push", DeformMode::Push, 0.0),
        ("Push + Distortion", DeformMode::Push, 1.0),
        ("Twist", DeformMode::Twist, 0.0),
        ("Pinch", DeformMode::Pinch, 0.0),
        ("Wrinkle (ruído fixo)", DeformMode::Wrinkle, 0.0),
        ("Fold", DeformMode::Fold, 0.0),
    ] {
        let f = DabField::new(
            mode,
            [0.0, 0.0],
            40.0,
            [4.0, 1.0],
            [0.0, 0.0],
            0.0,
            0.8,
            distortion,
            0.0,
            7,
        );
        // Pontos DENTRO do raio: fora dele o `at` sai no primeiro `if` e mediríamos o early-out.
        let t = ms(|| {
            let mut sink = 0.0_f32;
            for i in 0..N {
                let a = (i % 61) as f32 - 30.0;
                let b = (i % 47) as f32 - 23.0;
                let d = black_box(&f).at([a, b]);
                sink += d[0] + d[1];
            }
            black_box(sink);
        });
        println!("{name:<24} {t:>10.2} {:>12.3}", t * 1e6 / N as f64);
    }
}

/// **A FORMA.** O cook é `O(nós × N)` — e as duas metades têm de escalar sozinhas. Se dobrar `N` não
/// dobrar o custo, a lista está sendo podada em algum lugar e a tabela de passo mente; se dobrar os nós
/// não dobrar, alguém está cacheando entre nós e a promessa *"dois nós nunca conversam"* é falsa.
#[test]
#[ignore = "probe: measures, does not assert"]
fn measure_the_cook_is_linear_in_both() {
    const SPAN: f32 = 256.0;
    println!("\n=== a FORMA do cook: O(nós × N) ===");
    println!(
        "{:<10} {:>8} {:>12} {:>14} {:>10}",
        "lado", "N dabs", "ms", "ns/(nó·dab)", "razão"
    );
    let mut prev: Option<(f64, f64)> = None; // (trabalho, ms)
    for (side, n) in [(128, 32), (128, 64), (256, 64), (256, 128)] {
        let dabs = twist_dabs(n, [SPAN * 0.5, SPAN * 0.5], SPAN * 0.45);
        let t = ms(|| {
            black_box(cook(&dabs, side, SPAN));
        });
        let work = (side * side * n) as f64;
        let ratio = prev.map_or(0.0, |(w0, t0)| (t / t0) / (work / w0));
        println!(
            "{side:<10} {n:>8} {t:>12.2} {:>14.3} {:>10}",
            t * 1e6 / work,
            if prev.is_none() {
                "-".to_string()
            } else {
                format!("{ratio:.2}x")
            }
        );
        prev = Some((work, t));
    }
    println!("(razão = custo medido / custo previsto pelo trabalho; 1,00x = linear)");
}

/// **A TABELA que o ADR pede.** Dado o custo por-(nó·dab), quantos nós cabem num quadro?
///
/// ⚠️ O orçamento não é escolhido aqui: **16,6 ms é um quadro de 60 fps**, e o cook divide esse quadro
/// com tudo o mais que o app faz, então a linha honesta é a fração — e é ela que o smoke julga.
#[test]
#[ignore = "probe: measures, does not assert"]
fn measure_the_lattice_pitch_table() {
    const SPAN: f32 = 4096.0;
    const N: usize = 64;
    let side = 128;

    // ⚠️ A 1ª versão desta sonda mediu **6,14 ns** por (nó·dab) onde a irmã da FORMA media 19,0 — o
    // mesmo modo, a mesma lei, 3× de diferença. A causa é fixture, não código: ali o pincel cobria a
    // grade inteira e aqui um disco de 600 px numa extensão de 4096, então a maioria das avaliações
    // saía no PRIMEIRO `if` do `at` (`t2 >= 1.0`) e o que estava sendo cronometrado era o early-out.
    // Um passo de grade derivado daquele número seria otimista por 3×.
    //
    // Então a tabela sai do **PIOR CASO honesto** — a grade inteira dentro do pincel, que é o gesto do
    // report (o artista insiste no lugar) — e a economia do early-out é medida ao lado, como o fato
    // separado que ela é.
    let covered = twist_dabs(N, [SPAN * 0.5, SPAN * 0.5], SPAN); // raio ≥ extensão ⇒ todo nó coberto
    let t_worst = ms(|| {
        black_box(cook(&covered, side, SPAN));
    });
    let per = t_worst * 1e6 / (side * side * N) as f64;

    let sparse = twist_dabs(N, [SPAN * 0.5, SPAN * 0.5], 600.0); // o disco do traço real
    let t_sparse = ms(|| {
        black_box(cook(&sparse, side, SPAN));
    });
    let per_sparse = t_sparse * 1e6 / (side * side * N) as f64;

    println!("\n=== o PASSO da grade, derivado (CPU serial, 1 núcleo) ===");
    println!("pior caso (todo nó dentro do pincel): {per:.3} ns por (nó · dab)");
    println!(
        "traço real (disco de 600 px em {SPAN:.0}): {per_sparse:.3} ns — {:.1}x mais barato, e a razão \
         NÃO é otimização: é o early-out do raio. Um nó só paga os dabs que o ALCANÇAM.",
        per / per_sparse
    );
    println!(
        "\n{:<10} {:>12} {:>10} {:>12} {:>12}",
        "lado", "nós", "passo px", "N=64 (ms)", "N=256 (ms)"
    );
    for side in [64usize, 128, 256, 512, 1024] {
        let nodes = (side * side) as f64;
        println!(
            "{side:<10} {:>12} {:>10.1} {:>12.2} {:>12.2}",
            side * side,
            SPAN / side as f32,
            nodes * 64.0 * per / 1e6,
            nodes * 256.0 * per / 1e6
        );
    }
    println!(
        "\n⚠️ SERIAL, num núcleo. O device é a outra metade da medição e ainda NÃO foi feita — nenhum\n\
           passo entra no código antes dela (ADR-0157 §preço)."
    );
}
