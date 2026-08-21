//! Os gates do "quem é quem" — a célula da folha 07.

use super::*;
use crate::PULSE_COL;

fn with_ids(ids: &[f32]) -> Stream {
    Stream::new(ids.len()).with("id", Column::Scalar(ids.to_vec()))
}

/// **UMA PEÇA QUE SOBREVIVE LEVA O DEGRAU DELA, mesmo que as vizinhas tenham
/// morrido.**
///
/// É o defeito inteiro numa linha: ontem viviam os ids `[7, 8, 9]`, hoje vivem
/// `[9, 11]`. Posicionalmente o `9` de hoje leria a linha 0 — o tique do id `7`.
/// FALSIFICADO se o pareamento devolvesse `[Some(0), Some(1)]`.
#[test]
fn a_survivor_keeps_its_own_row_after_the_set_churns() {
    let now = with_ids(&[9.0, 11.0]);
    let before = with_ids(&[7.0, 8.0, 9.0]);
    assert_eq!(
        pairing(&now, &before, 2),
        Some(vec![Some(2), None]),
        "o id 9 estava na linha 2; o id 11 é novo"
    );
}

/// **UMA STREAM SEM `id` E COM A CONTAGEM ESTÁVEL PAREIA LINHA A LINHA** — o
/// caminho que shipava, byte-idêntico.
#[test]
fn a_stable_countless_stream_pairs_row_for_row() {
    assert_eq!(
        pairing(&Stream::new(3), &Stream::new(3), 3),
        Some(vec![Some(0), Some(1), Some(2)])
    );
}

/// **SEM `id` E COM A CONTAGEM MUDADA, TUDO RE-SEMEIA.**
///
/// ⚠️ A lei é a do `motion.integrate`, de propósito. Uma grelha 2×2 que vira 2×3
/// renumera todos os pontos: herdar a linha 1 é herdar o degrau de outra peça, e
/// o resultado seria uma escada que ninguém autorou num sítio que ninguém olhou.
#[test]
fn a_countless_stream_that_changed_size_reseeds_instead_of_pairing_by_index() {
    assert_eq!(pairing(&Stream::new(6), &Stream::new(4), 6), None);
    assert_eq!(pairing(&Stream::new(2), &Stream::new(4), 2), None);
}

/// **NO TIQUE 0 NÃO HÁ ESTADO** — o `pre` cozinha vazio e tudo semeia.
#[test]
fn the_first_tick_has_nothing_to_pair_against() {
    assert_eq!(pairing(&with_ids(&[1.0, 2.0]), &Stream::new(0), 2), None);
    assert_eq!(pairing(&Stream::new(3), &Stream::new(0), 3), None);
}

/// **UM ID REPETIDO NÃO ENTRA EM PÂNICO NEM PERDE PEÇAS** — o `BTreeMap` fica com
/// a ÚLTIMA linha, deterministicamente, e as duas peças de hoje leem-na.
///
/// ⚠️ Uma stream com ids repetidos é corrupta, mas *"corrupta"* não pode ser
/// *"crasha"*: o nó tem de emitir `n` linhas aconteça o que acontecer.
#[test]
fn a_duplicated_id_is_deterministic_and_never_panics() {
    let now = with_ids(&[5.0, 5.0]);
    let before = with_ids(&[5.0, 5.0]);
    assert_eq!(pairing(&now, &before, 2), Some(vec![Some(1), Some(1)]));
}

/// **A ESCADA `0/1/n` DO BATIMENTO** — e o degrau do meio é a cura.
///
/// FALSIFICADO pela lei antiga (esticar com zeros): ali `beat_at(&[1.0], 3)`
/// daria `0.0`, e um relógio global paralisaria tudo menos o elemento 0.
#[test]
fn a_global_beat_reaches_every_element_not_only_the_first() {
    assert_eq!(beat_at(&[], 0), 0.0, "nada ligado");
    assert_eq!(beat_at(&[], 9), 0.0);
    for i in 0..4 {
        assert_eq!(beat_at(&[1.0], i), 1.0, "o elemento {i} ouve o batimento");
    }
    let per_element = [0.0, 1.0, 0.0];
    assert_eq!(beat_at(&per_element, 1), 1.0, "e um campo é por-elemento");
    assert_eq!(beat_at(&per_element, 0), 0.0);
    assert_eq!(beat_at(&per_element, 7), 0.0, "fora do fim: silêncio");
}

/// **`raw_col` NÃO ESTICA** — é isso que faz a escada existir. Um `resize(n)` aqui
/// apagaria os três degraus, porque o comprimento 1 deixaria de ser distinguível.
#[test]
fn the_column_reader_reports_the_real_length() {
    let s = Stream::new(1).with(PULSE_COL, Column::Scalar(vec![1.0]));
    assert_eq!(raw_col(&s, PULSE_COL).len(), 1);
    assert_eq!(raw_col(&s, "nada").len(), 0, "ausente é a lista vazia");
}
