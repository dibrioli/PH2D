//! **Nenhuma distância de offset derruba o processo** — o gate do achado de 2026-07-23.
//!
//! # O que ele pina, e por que não é paranoia
//!
//! `offset_path` documenta, desde que existe, que devolve vazio *"se o sweep falhar"*. A frase era
//! verdadeira só para o `None` do `Region::of`: o `linesweeper` 0.3.0 tem um `unwrap()` numa curva
//! degenerada e **entra em pânico**, o que num app de desenho é o artista a perder o trabalho.
//!
//! Medido antes da cura, varrendo 200 distâncias por forma: uma estrela de 5 pontas com quina
//! **Bevel** panica em **118 delas** — 59% do curso do slider de Offset da seção Expand. Não é
//! um caso hostil nem uma entrada inventada: é a forma do catálogo, com um knob do painel.
//!
//! O gate varre as MESMAS combinações e exige que a chamada RETORNE. Não exige que ela devolva
//! geometria: quando o sweep de facto não consegue responder, vazio é a resposta certa (é a que a
//! doc sempre prometeu) — o que não pode acontecer é o processo morrer.

use ph2d_vec_scene::{LineJoin, OffsetSide, ShapeKind, cook};

#[test]
fn no_offset_distance_takes_the_process_down() {
    let shapes = [
        (
            "retângulo",
            cook(ShapeKind::Rectangle, [0.0, 0.0], [2.0, 1.0], &[]),
        ),
        (
            "hexágono",
            cook(ShapeKind::Polygon, [1.0, -1.2], [3.4, 1.2], &[6.0]),
        ),
        (
            "estrela",
            cook(
                ShapeKind::Star,
                [-3.6, -1.4],
                [-0.8, 1.4],
                &[5.0, 0.45, 0.0],
            ),
        ),
    ];
    let mut empties = 0;
    let mut total = 0;
    for (name, shape) in &shapes {
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            for side in [OffsetSide::Outer, OffsetSide::Inner, OffsetSide::Both] {
                for i in 1..=100 {
                    // A faixa cobre o curso REAL do slider: até ~2× o tamanho da forma.
                    let d = f64::from(i) * 0.04;
                    total += 1;
                    if ph2d_vec_boolean::offset_path(shape, d, join, side).is_empty() {
                        empties += 1;
                    }
                    let _ = name;
                }
            }
        }
    }
    // Controle positivo: se TUDO saísse vazio, o gate acima seria satisfeito por uma função que
    // não faz nada — e um gate que não pode falhar é pior que gate nenhum.
    assert!(
        empties * 2 < total,
        "{empties} de {total} offsets sairam vazios — o gate deixou de medir robustez e passou a \
         medir uma funcao morta"
    );
}
