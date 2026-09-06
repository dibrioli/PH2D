//! ⭐⭐⭐ **O QUE CADA MODO CUSTA, E QUE ESTADO ELE CARREGA** (ciclo 2, W3 — doc 105).
//!
//! ⚠️⚠️ **A W3 abre com a medição e não com o kernel, e a primeira coisa que ela fez foi
//! REFUTAR a minha própria linha do plano.** Eu tinha escrito que este nó *«lê o passado de
//! OUTRO elemento, o que é um gather»*. É **falso**: cada elemento lê o **seu próprio** passado.
//! O bloqueador do dispositivo é outro — o **anel de 32 fatias** ([`super::ring::MAX_LAG`]), que
//! é 32 colunas de estado a deslocar-se por tique.
//!
//! ⭐ E daí sai a pergunta que interessa: o modo **`Blend`** — o default, e aquele *para que o nó
//! existe* — **não usa o anel**. Ele precisa de UMA coluna (`dl_out`). Se ele estiver a pagar o
//! anel que não lê, isso é um ganho muito mais barato que um kernel.
//!
//! ```text
//! cargo test -p ph2d-node-motion-delay --release -- --ignored --nocapture measure_what_each_mode_costs
//! ```

use super::tests::run_any;
use ph2d_nodegraph::attr::{Column, Stream};

fn entrada(n: usize, t: usize) -> Stream {
    Stream::new(n).with(
        "P",
        Column::Vec2(
            (0..n)
                .map(|i| [i as f32 * 0.1, (t as f32 * 0.7 + i as f32).sin()])
                .collect(),
        ),
    )
}

#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE"]
fn measure_what_each_mode_costs() {
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!("\n  modo     | elementos | ms/tique | colunas de estado na saida");
    eprintln!("  ---------|-----------|----------|---------------------------");
    for (nome, m) in [("Delay", 0.0_f32), ("Average", 1.0), ("Blend", 2.0)] {
        for n in [1_000_usize, 100_000] {
            const TIQUES: usize = 12;
            let t0 = std::time::Instant::now();
            let saidas = run_any(&[("mode", m), ("ticks", 8.0)], TIQUES, |_, _, t| {
                entrada(n, t)
            });
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / TIQUES as f64;
            let cols = saidas
                .last()
                .map(|s| {
                    s.columns()
                        .filter(|(nome, _)| super::ring::is_state(nome))
                        .count()
                })
                .unwrap_or(0);
            eprintln!("  {nome:<8} | {n:>9} | {ms:>8.3} | {cols:>25}");
        }
    }
    eprintln!(
        "\n  (o `Blend` precisa de UMA coluna de estado, `dl_out`; o anel de 32 fatias e' das
   outras duas, e e' ele que as prende a` CPU.)\n"
    );
}

/// ⭐⭐⭐ **O MODO COMUM DEIXOU DE PAGAR O ANEL QUE NÃO LÊ** — o achado da W3, medido.
///
/// | modo | 100 000 elementos, antes | depois | colunas de estado |
/// |---|---|---|---|
/// | `Delay` | `12,82 ms` | `12,85 ms` | 34 → 34 |
/// | `Average` | `13,89 ms` | `10,32 ms` | 34 → 34 |
/// | **`Blend`** (o default) | **`10,96 ms`** | **`1,46 ms`** | **34 → 2** |
///
/// FALSIFICADO por o `depth_for` voltar a devolver `MAX_LAG` para o `Blend`: a contagem de
/// colunas sobe de 2 para 34 e o gate diz qual.
#[test]
fn the_blend_mode_stops_paying_for_a_ring_it_never_reads() {
    let contar = |m: f32, canal: f32| {
        let saidas = run_any(
            &[("mode", m), ("ticks", 8.0), ("channel", canal)],
            4,
            |_, _, t| entrada(16, t),
        );
        saidas
            .last()
            .expect("quatro tiques")
            .columns()
            .filter(|(n, _)| super::ring::is_state(n))
            .count()
    };
    // `1` = Y, um canal escalar não-angular.
    assert_eq!(
        contar(2.0, 1.0),
        2,
        "o Blend guarda `dl_out` e a etiqueta do canal"
    );
    assert_eq!(
        contar(0.0, 1.0),
        34,
        "o Delay continua a guardar o anel inteiro"
    );
    assert_eq!(contar(1.0, 1.0), 34, "e o Average tambem");
}

/// ⚠️⚠️ **E O `BLEND` ANGULAR AINDA GUARDA A SUA FATIA** — o desembrulho do ângulo compara com
/// «um tique atrás», e uma varredura que devolvesse zero para todo `Blend` partiria a rotação.
///
/// FALSIFICADO por `depth_for` ignorar o `angular`: a contagem cai de 3 para 2 e o giro que
/// cruza os 180° passa a saltar.
#[test]
fn the_angular_blend_still_keeps_its_one_slice() {
    let saidas = run_any(
        // `2` = Rotation, o canal angular.
        &[("mode", 2.0), ("ticks", 8.0), ("channel", 2.0)],
        4,
        |_, _, t| {
            Stream::new(4).with(
                "rot",
                Column::Scalar((0..4).map(|i| 170.0 + t as f32 * 8.0 + i as f32).collect()),
            )
        },
    );
    let cols = saidas
        .last()
        .expect("quatro tiques")
        .columns()
        .filter(|(n, _)| super::ring::is_state(n))
        .count();
    assert_eq!(cols, 3, "`dl_out` + a etiqueta + a fatia do desembrulho");
    assert!(
        super::ring::depth_for(super::MODE_BLEND, true) == 1
            && super::ring::depth_for(super::MODE_BLEND, false) == 0,
        "a profundidade do Blend segue o canal"
    );
}
