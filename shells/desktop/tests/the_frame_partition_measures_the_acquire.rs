//! **A ESPERA DO GPU É MEDIDA, NÃO SUBTRAÍDA.**
//!
//! ⚠️ **O número que este gate protege enganou o autor do próprio instrumento.**
//! A linha `[frame]` publicava `present/acquire-stall = total - encode`, uma
//! SUBTRAÇÃO com nome de medição, e o `encode` só começa em `cpu_start` — depois
//! do `tool-tick`, do flush de carimbo e do pump de eventos. Logo o resíduo
//! continha **trabalho de CPU** com um rótulo que diz *espera de GPU*.
//!
//! Medido no smoke de 2026-08-01: `stall 7,91` com `tool-tick 3,31` ⇒ a espera
//! real é ~4,6 ms e a CPU trabalha ~12 ms de um quadro de 16,6 — não os 8,25 que
//! a linha sugeria. **Eu li o 8,35 como ociosidade e reportei ao Enio que "a CPU
//! passa metade do quadro esperando o GPU".** Era o rótulo, não a máquina.
//!
//! *Um número derivado por subtração absorve tudo que ninguém mediu, e herda o
//! nome de quem o publicou.*
//!
//! Arch-gate sobre o fonte porque o `[frame]` só existe com janela — nenhum
//! teste de unidade alcança aquele `eprintln`.

const SRC: &str = include_str!("../src/render_loop/mod.rs");
const PRESENT: &str = include_str!("../src/render_loop/present.rs");

/// A espera é cronometrada NO SÍTIO do `acquire_frame`.
///
/// Mutação que sangra: tirar o `note_acquire_wait` — a linha volta a não ter
/// nenhuma medida da espera, e o resíduo volta a ser a única fonte.
#[test]
fn the_acquire_wait_is_clocked_where_it_happens() {
    let at = PRESENT
        .find("match surface.acquire_frame() {")
        .expect("o present adquire o frame");
    // Janela ESTRUTURAL: o braço `Ok(frame) =>` até a linha seguinte que volta à
    // indentação dele. Nunca uma distância em bytes — a lição que o gate irmão
    // do divisor pagou quando o próprio comentário do fix o quebrou.
    let rest = &PRESENT[at..];
    let arm_end = rest.find("\n            Err(").unwrap_or(rest.len());
    let arm = &rest[..arm_end];
    assert!(
        arm.contains("note_acquire_wait"),
        "a espera do acquire nao e' cronometrada no braco que a contem — a linha \
         `[frame]` so pode derivá-la por subtracao, e a subtracao absorve o \
         `tool-tick` com o nome de espera de GPU"
    );
    assert_eq!(
        PRESENT.matches("note_acquire_wait").count(),
        1,
        "ha mais de um sitio cronometrando o acquire — a media sairia sobre \
         quadros contados duas vezes"
    );
}

/// A linha imprime a espera MEDIDA, e o resíduo desconta as DUAS parcelas.
///
/// ⚠️ A segunda metade é a que impede a mentira de voltar: se `outside_ms`
/// deixar de subtrair o `acq_ms`, a partição volta a somar errado **em
/// silêncio** — os três números continuam sendo impressos, e um deles passa a
/// conter o outro.
///
/// Mutação que sangra: `outside_ms = total - encode` (sem o acquire).
#[test]
fn the_frame_line_prints_a_partition_that_actually_partitions() {
    assert!(
        SRC.contains("acquire(medido)={acq_ms:"),
        "o `[frame]` nao imprime a espera MEDIDA — quem le so tem o residuo, e \
         foi assim que `stall 8,35` virou 'a CPU espera metade do quadro'"
    );
    let at = SRC
        .find("let outside_ms =")
        .expect("o residuo do quadro e' calculado");
    let expr: String = SRC[at..].chars().take(160).collect();
    let line = expr.lines().next().unwrap_or_default();
    assert!(
        line.contains("encode") && line.contains("acq_ms"),
        "o residuo nao desconta as DUAS parcelas ({line}): a particao volta a \
         somar errado em silencio, com um dos numeros contendo o outro"
    );
}
