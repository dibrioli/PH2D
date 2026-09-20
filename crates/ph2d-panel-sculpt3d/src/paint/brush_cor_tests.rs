//! **A PONTE ENTRE O `f32` DO MOTOR E O `u8` DO ARTISTA** — as duas direcções,
//! e o erro que a quantização custa, MEDIDO em vez de suposto.
//!
//! ⚠️ Os gates de GESTO — a amostra é pintada, o clique abre o selector, a cor
//! escolhida chega ao pincel — vivem na costura (`tests/it/seam_cor.rs`), porque
//! precisam do `MockPanelHost` e do `Down` real.

// ⚠️ `super` já É o [`super::super::brush_cor`]: este módulo é declarado DENTRO
// dele por `#[path]`. Escrever `super::brush_cor` é o `E0432` que esta casa já
// pagou duas vezes.
use super::{para_f32, para_u8};

/// ⭐⭐⭐ **GATE — a cor que o artista ESCOLHE é a cor que a amostra MOSTRA.**
///
/// A ida-e-volta que interessa ao produto é `u8 → f32 → u8`, porque é a que o
/// gesto percorre: o selector entrega bytes, o pincel guarda `f32`, e a amostra
/// do quadro seguinte volta a mostrar bytes. Ela é **exacta nos 256 valores** —
/// e isso não é sorte, é o `+ 0,5` da [`para_u8`] a desfazer exactamente a
/// divisão da [`para_f32`].
///
/// ⛔ Sem esta metade, um erro de arredondamento faria a amostra mostrar `194`
/// depois de o artista escolher `195`, e — pior — a guarda do *«mudou?»* leria
/// diferença **todo quadro** e publicaria um passo de undo por quadro.
#[test]
fn a_cor_que_o_artista_escolhe_e_a_que_a_amostra_mostra() {
    for b in 0u8..=255 {
        let volta = para_u8(para_f32([b, b, b]));
        assert_eq!(
            volta,
            [b, b, b],
            "o byte {b} nao sobrevive a' ida-e-volta: a amostra mostraria \
             {volta:?} depois de o artista escolher {b}"
        );
    }
}

/// ⭐⭐⭐ **GATE — o valor de FÁBRICA sobrevive ao bit.**
///
/// A cor de fábrica é o `_color` do `Paint.js:13` e **não é representável em 8
/// bits** (`0,766 × 255 = 195,33`). Se a amostra escrevesse de volta todo
/// quadro, a primeira coisa que ela faria era mudar a cor do pincel sem
/// ninguém tocar em nada.
///
/// ⚠️ **A régua é o PRODUTO e não a aritmética:** o que este gate afirma é que
/// o byte que a amostra mostra é o mesmo em que a guarda do *«mudou?»* compara,
/// logo a comparação dá IGUAL e nenhum `SetUi` é publicado. *A mesma conta nos
/// dois lados é o que torna a fábrica um ponto fixo.*
#[test]
fn a_cor_de_fabrica_e_um_ponto_fixo_da_guarda() {
    let fabrica = ph2d_sculpt3d::Brush::default().color;
    // O CONTROLO: ela de facto não é representável, senão este gate seria
    // trivialmente verdadeiro e não afirmaria nada.
    assert_ne!(
        para_f32(para_u8(fabrica)),
        fabrica,
        "a cor de fabrica passou a ser representavel em 8 bits — este gate \
         deixou de conter o fenomeno e a premissa dele morreu"
    );
    // E a guarda do produto compara BYTES, que é onde ela é um ponto fixo.
    assert_eq!(
        para_u8(para_f32(para_u8(fabrica))),
        para_u8(fabrica),
        "a guarda do «mudou?» leria diferenca sobre a cor de fabrica — um passo \
         de undo por quadro sobre uma cor que ninguem mexeu"
    );
}

/// ⭐⭐ **GATE — o preço da quantização, MEDIDO e não suposto.**
///
/// A partir da primeira cor escolhida, o canal passa a ser quantizado a 8 bits.
/// O erro máximo é meio passo (`1/510 ≈ 0,00196`), que sobre um ecrã de 8 bits
/// é **meio byte** — invisível por construção.
///
/// ⚠️ **E a troca AUMENTA o que o artista alcança:** arrastar o chip de uma
/// pista de passo `0,05` dava `21` valores por canal; a amostra dá `256`. Este
/// gate prende o número que essa frase cita.
#[test]
fn a_quantizacao_custa_meio_byte_e_compra_doze_vezes_mais_valores() {
    let mut pior = 0.0f32;
    for i in 0..=1000 {
        let v = i as f32 / 1000.0;
        let erro = (para_f32(para_u8([v, v, v]))[0] - v).abs();
        pior = pior.max(erro);
    }
    assert!(
        pior <= 1.0 / 510.0 + f32::EPSILON,
        "o erro da quantizacao mede {pior:.6} e o meio passo e' {:.6}",
        1.0 / 510.0
    );
    // Os `21` da pista: `(max − min) / step + 1` com o passo que a `Row` de cor
    // declarava (`0,05` sobre `[0, 1]`).
    let valores_da_pista = (1.0f32 / 0.05).round() as u32 + 1;
    assert_eq!(valores_da_pista, 21);
    assert!(
        256 > valores_da_pista * 12,
        "a amostra deixou de comprar 12x o que a pista comprava"
    );
}

/// ⚠️ **GATE — a conversão SATURA em vez de dar a volta.**
///
/// Um `f32` fora de `[0, 1]` chega aqui por qualquer caminho que escreva a cor
/// sem passar pela amostra (o `SetUi` de um teste, um projecto gravado por uma
/// versão futura). ⛔ Sem o `clamp`, `1,5 × 255 + 0,5 = 383` truncado para `u8`
/// em Rust **satura** em `255` — o que dá a resposta certa por acidente —, mas
/// `-0,5 × 255 + 0,5 = -127` satura em `0`, e um valor `NaN` daria `0`. *Uma
/// cerca que repete o que a linguagem garante ainda é a cerca que diz o que ela
/// não garante*: o `clamp` é o que torna os três casos a MESMA lei.
#[test]
fn uma_cor_fora_da_faixa_satura_nos_extremos() {
    assert_eq!(para_u8([1.5, -0.5, f32::NAN]), [255, 0, 0]);
    assert_eq!(para_u8([1.0, 0.0, 0.5]), [255, 0, 128]);
}
