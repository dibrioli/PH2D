//! **A TROCA, sozinha** — o gate da [`super::swap_window`], cortado do pai
//! pelo tecto de LOC quando a tinta fina lhe acrescentou o quarto canal.
//!
//! ⚠️ O corte é de RESPONSABILIDADE e não de tamanho: o que está aqui mede
//! a peça que faz desfazer e refazer serem a MESMA operação, e ela é a única
//! coisa do pai que se testa sem uma cena.

use super::swap_window;

/// ⚠️ **A troca devolve o que ESTAVA lá, não o que ela instalou.**
///
/// É o dente do modelo inteiro: se ela devolvesse o valor novo, o desfazer
/// funcionaria (o estado certo é instalado) e o refazer seria um no-op que
/// **consome** a entrada — a forma de "o redo às vezes não faz nada" que
/// nenhum gate de contagem vê.
#[test]
fn the_window_swap_returns_what_was_there_not_what_it_installed() {
    let mut plane = [10.0f32, 11.0, 12.0, 13.0];
    let verts = [3u32, 1];

    let was = swap_window(&mut plane, &verts, &[99.0, 98.0]);
    assert_eq!(
        plane,
        [10.0, 98.0, 12.0, 99.0],
        "instalou nos índices certos"
    );
    assert_eq!(
        was,
        vec![13.0, 11.0],
        "e colheu o que estava lá, na ordem dos índices"
    );

    // E ela é a própria inversa: aplicar o que voltou restaura o começo — que
    // é literalmente o que a fila oposta faz.
    let back = swap_window(&mut plane, &verts, &was);
    assert_eq!(plane, [10.0, 11.0, 12.0, 13.0], "a volta restaura");
    assert_eq!(back, vec![99.0, 98.0], "e devolve o que o refazer precisa");
}
