//! Os gates da porta de entrada.

use super::*;

/// ⭐ **Uma peça importada nasce no tamanho do ENQUADRAMENTO**, venha ela em que unidades vier.
///
/// ⚠️ **Sem isto o artista conclui que a importação falhou.** Um arquivo de 300 unidades ao lado de
/// uma caixa de 1 não aparece grande: ele aparece **como nada**, porque a câmera enquadra a caixa e a
/// escultura fica toda fora do quadro. O sintoma é uma tela igual à de antes do clique.
#[test]
fn an_imported_piece_arrives_at_the_size_of_the_framing() {
    let half = 1.0f32;
    for extent in [0.01f32, 1.0, 7.5, 300.0] {
        let s = framing_scale(extent, half);
        let on_screen = extent * s;
        assert!(
            (on_screen - half * 2.0 * FRAMING_FRACTION).abs() < 1e-3,
            "extensão {extent}: a peça ficou com {on_screen} no mundo, e o enquadramento pede {}",
            half * 2.0 * FRAMING_FRACTION
        );
    }
}

/// ⚠️ **Uma extensão degenerada não vira uma escala degenerada.** Um arquivo com um só vértice, ou
/// com `NaN` numa coordenada, dá extensão zero — e uma divisão por ela poria a peça no infinito, que
/// é a forma mais confusa de um import falhar (nada na tela, nenhum erro).
#[test]
fn a_degenerate_extent_falls_back_to_one() {
    for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY] {
        let s = framing_scale(bad, 1.0);
        assert!(s.is_finite() && s > 0.0, "extensão {bad} deu escala {s}");
    }
    assert!((framing_scale(0.0, 1.0) - 1.0).abs() < 1e-6);
}

/// ⚠️ **A escala acompanha o ENQUADRAMENTO, não é uma constante.** Uma peça importada com a câmera
/// afastada tem de nascer maior — senão ela nasce do tamanho certo para um enquadramento que já não
/// é o que o artista tem à frente.
#[test]
fn the_size_follows_the_camera_not_a_constant() {
    let extent = 2.0;
    let near = framing_scale(extent, 0.5);
    let far = framing_scale(extent, 4.0);
    assert!(
        (far / near - 8.0).abs() < 1e-3,
        "oito vezes o enquadramento tem de dar oito vezes a escala: {near} e {far}"
    );
}
