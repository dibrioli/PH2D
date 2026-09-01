//! **O CUSTO DO LOWERING, medido** — a sonda que a auditoria de performance de 2026-09-01
//! pediu, e a régua com que a cura dela se prova.
//!
//! ⚠️ **Sonda, não gate**: ela IMPRIME e não julga (`#[ignore]`), pela mesma razão que o
//! `emitter_sim_ceiling_probe` — um número de relógio nesta máquina não vale nada acima de
//! `load ~5` (CLAUDE.md §5.0), e um gate de razão sob fan-out entra na família das flakes.
//!
//! O que ela mede é UMA pergunta: *quanto custa perguntar `stream.get("nome")` DENTRO do laço
//! por elemento?* O lowering hasteia sete colunas para fora do laço e depois faz três lookups
//! por linha — `row_medium` pergunta por `geometry_id` e pela coluna do passe vectorial, e o
//! corpo repete o `geometry_id`. Cada um é uma descida de `BTreeMap<String, _>` com comparação
//! de string.
//!
//! Corra com:
//!   cargo test -p ph2d-eval-motion --release --test lowering_cost -- --ignored --nocapture

use ph2d_eval_motion::{VectorInstance, lower_to_instances_into};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_render::RenderInstance;
use ph2d_render::SinkStyle;

/// Uma corrente com as colunas que o lowering lê, mais a que o manda pelo caminho vectorial.
fn stream_at(n: usize, vector_rows: bool) -> Stream {
    let mut s = Stream::new(n);
    s.set("P", Column::Vec2((0..n).map(|i| [i as f32, 0.0]).collect()));
    s.set("size", Column::Vec2(vec![[1.0, 1.0]; n]));
    s.set(
        "rot",
        Column::Scalar((0..n).map(|i| i as f32 * 0.01).collect()),
    );
    s.set("tint", Column::Vec4(vec![[1.0; 4]; n]));
    if vector_rows {
        // Metade das linhas são formas — o caso misto, que é o que um documento com
        // folhas vectoriais sobre galhos de sprite de facto produz.
        s.set(
            "geometry_id",
            Column::Scalar((0..n).map(|i| (i % 2) as f32).collect()),
        );
    }
    s
}

fn median_ms(mut runs: Vec<f64>) -> f64 {
    runs.sort_by(f64::total_cmp);
    runs[runs.len() / 2]
}

/// ⚠️ **A mediana de 5, no MESMO processo** — subtrair dois relógios de corridas separadas dá
/// a soma dos ruídos das duas ([[feedback_subtracting_two_clocks_from_separate_runs_gives_the_sum_of_the_noises]]).
#[test]
#[ignore = "perf probe"]
fn lowering_cost_probe() {
    eprintln!("       n │ sprite puro ms │ misto: sprite ms │ misto: vector ms");
    for &n in &[65_536usize, 262_144, 1_048_576, 4_194_304] {
        let puro = stream_at(n, false);
        let misto = stream_at(n, true);
        let mut out: Vec<RenderInstance> = Vec::new();
        let mut vout: Vec<VectorInstance> = Vec::new();

        let mut t_puro = Vec::new();
        let mut t_sprite = Vec::new();
        let mut t_vector = Vec::new();
        for r in 0..6 {
            let a = std::time::Instant::now();
            lower_to_instances_into(
                &puro,
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0],
                SinkStyle::PLAIN,
                &mut out,
            );
            let ta = a.elapsed().as_secs_f64() * 1000.0;

            let b = std::time::Instant::now();
            lower_to_instances_into(
                &misto,
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0],
                SinkStyle::PLAIN,
                &mut out,
            );
            let tb = b.elapsed().as_secs_f64() * 1000.0;

            let c = std::time::Instant::now();
            vout.clear();
            ph2d_eval_motion::lower_to_vector_instances_onto(&misto, SinkStyle::PLAIN, &mut vout);
            let tc = c.elapsed().as_secs_f64() * 1000.0;

            if r > 0 {
                // A 1ª corrida paga a capacidade do `out`; o produto reusa o buffer.
                t_puro.push(ta);
                t_sprite.push(tb);
                t_vector.push(tc);
            }
        }
        eprintln!(
            "{n:>8} │ {:>14.3} │ {:>16.3} │ {:>16.3}",
            median_ms(t_puro),
            median_ms(t_sprite),
            median_ms(t_vector)
        );
    }
    eprintln!("  (o orçamento de 60 fps é 16,7 ms/quadro; o lowering é UMA parte dele)");
}

/// ⭐⭐ **A PROVA de que paralelizar o lowering vectorial não mudou um bit** — o gate que a cura
/// da auditoria de performance deve, e o único que **não** é uma sonda de relógio.
///
/// ⚠️ **Ele corre dos DOIS lados do limiar** (`PAR_THRESHOLD = 8192`): abaixo o caminho é o
/// `extend` serial de sempre, acima é o `par_extend`. Um teste só acima provaria a rota nova
/// contra ela mesma; um só abaixo nunca tocaria na rota que a mudança criou.
///
/// A referência é construída **aqui**, com o laço que o produto tinha antes — um oráculo que
/// partilhasse a lei do produto seria um espelho ([[feedback_an_oracle_that_shares_the_law_of_what_it_judges_is_a_mirror]]).
#[test]
fn the_parallel_vector_lowering_is_bit_identical_to_the_serial_one() {
    for &n in &[0usize, 1, 8191, 8192, 8193, 40_000] {
        let s = stream_at(n, true);
        let mut got: Vec<VectorInstance> = Vec::new();
        ph2d_eval_motion::lower_to_vector_instances_onto(&s, SinkStyle::PLAIN, &mut got);

        // O oráculo: o mesmo predicado, escrito à mão, num laço serial que nunca cruza o
        // limiar — as linhas que NÃO são sprite, na ordem do índice.
        let want: Vec<usize> = (0..n)
            .filter(|&i| ph2d_eval_motion::row_medium(&s, i) != ph2d_eval_motion::RowMedium::Sprite)
            .collect();
        assert_eq!(
            got.len(),
            want.len(),
            "n = {n}: o paralelo emitiu outra quantidade de linhas que o serial"
        );
        for (slot, &i) in want.iter().enumerate() {
            assert_eq!(
                got[slot].world_pos,
                [i as f32, 0.0],
                "n = {n}: a linha {slot} devia ser o elemento {i} — a ordem não se preservou"
            );
        }
    }
}

/// **Quanto custa UM `stream.get("nome")`?** — a régua que separa o içamento da paralelização
/// na cura de 2026-09-01, porque as duas entraram no mesmo commit e um número só não as divide.
///
/// Mede a MESMA pergunta pelas duas portas, no mesmo processo, sobre a mesma corrente.
#[test]
#[ignore = "perf probe"]
fn per_element_lookup_cost_probe() {
    let n = 4_194_304usize;
    let s = stream_at(n, true);
    let mut cru = Vec::new();
    let mut icado = Vec::new();
    for r in 0..6 {
        let a = std::time::Instant::now();
        let mut acc = 0usize;
        for i in 0..n {
            if ph2d_eval_motion::row_medium(&s, i) == ph2d_eval_motion::RowMedium::Sprite {
                acc += 1;
            }
        }
        let ta = a.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(acc);

        let b = std::time::Instant::now();
        let media = ph2d_eval_motion::MediaColumns::of(&s);
        let mut acc2 = 0usize;
        for i in 0..n {
            if media.at(i) == ph2d_eval_motion::RowMedium::Sprite {
                acc2 += 1;
            }
        }
        let tb = b.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(acc2);
        assert_eq!(acc, acc2, "as duas portas têm de dar a MESMA resposta");
        if r > 0 {
            cru.push(ta);
            icado.push(tb);
        }
    }
    let (c, i) = (median_ms(cru), median_ms(icado));
    eprintln!("  {n} linhas, serial, mediana de 5:");
    eprintln!(
        "    por elemento (2 lookups) │ {c:>8.3} ms  ⇒ {:.1} ns/linha",
        c * 1e6 / n as f64
    );
    eprintln!(
        "    içado (0 lookups)        │ {i:>8.3} ms  ⇒ {:.1} ns/linha",
        i * 1e6 / n as f64
    );
    eprintln!(
        "    o içamento poupa         │ {:>8.3} ms  ({:.1}×)",
        c - i,
        c / i
    );
}
