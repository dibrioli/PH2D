//! Os gates dos componentes de layout.

use super::*;

/// **O neutro de um filho é INERTE.**
///
/// ⚠️ O gate existe porque o número óbvio é o do CSS (`flex-shrink: 1`), e ele espremeria as formas
/// do artista no instante em que a moldura ficasse pequena demais — sem ninguém pedir. Aqui
/// `Default` significa *não faz nada*, e crescer/encolher é opt-in (é o default do Figma).
#[test]
fn a_child_that_authored_nothing_neither_grows_nor_shrinks() {
    let it = VecLayoutItem::default();
    assert_eq!(it.grow, 0.0);
    assert_eq!(it.shrink, 0.0, "encolher por omissao espreme o desenho");
    assert_eq!(it.basis, None);
}

/// **O neutro de uma moldura é uma linha justa** — sem vão nem recuo, tudo no começo. É o que
/// deixa o artista ver o efeito de cada número que ele escreve a seguir.
#[test]
fn a_frame_that_authored_nothing_is_a_tight_row() {
    let l = VecLayout::default();
    assert_eq!(l.dir, LayoutDir::Row);
    assert_eq!(l.gap, [0.0, 0.0]);
    assert_eq!(l.pad, [0.0; 4]);
    assert_eq!(l.align, LayoutAlign::Start);
    assert_eq!(l.justify, LayoutJustify::Start);
}

/// **Os dois componentes sobrevivem ao round-trip** — é o que o save e o undo fazem com eles.
#[test]
fn both_components_survive_the_round_trip() {
    let l = VecLayout {
        dir: LayoutDir::RowWrap,
        gap: [4.0, 2.0],
        pad: [1.0, 2.0, 3.0, 4.0],
        align: LayoutAlign::Center,
        justify: LayoutJustify::SpaceBetween,
    };
    let bytes = postcard::to_allocvec(&l).expect("serializa");
    assert_eq!(postcard::from_bytes::<VecLayout>(&bytes).expect("le"), l);

    let it = VecLayoutItem {
        grow: 2.0,
        shrink: 1.0,
        basis: Some(12.5),
    };
    let bytes = postcard::to_allocvec(&it).expect("serializa");
    assert_eq!(
        postcard::from_bytes::<VecLayoutItem>(&bytes).expect("le"),
        it
    );
}

/// **A ordem das variantes é FORMATO DE ARQUIVO.**
///
/// ⚠️ O postcard grava o ÍNDICE da variante. Reordenar `LayoutDir` (ou espetar uma no meio) faria
/// todo documento salvo abrir com a direção errada — sem erro, sem aviso, com a moldura a empilhar
/// para o lado errado. Variante nova entra no FIM.
#[test]
fn the_variant_order_is_file_format() {
    for (v, want) in [
        (LayoutDir::Row, 0u8),
        (LayoutDir::Column, 1),
        (LayoutDir::RowWrap, 2),
    ] {
        assert_eq!(postcard::to_allocvec(&v).expect("ser")[0], want, "{v:?}");
    }
    for (v, want) in [
        (LayoutAlign::Start, 0u8),
        (LayoutAlign::Center, 1),
        (LayoutAlign::End, 2),
        (LayoutAlign::Stretch, 3),
    ] {
        assert_eq!(postcard::to_allocvec(&v).expect("ser")[0], want, "{v:?}");
    }
    for (v, want) in [
        (LayoutJustify::Start, 0u8),
        (LayoutJustify::Center, 1),
        (LayoutJustify::End, 2),
        (LayoutJustify::SpaceBetween, 3),
        (LayoutJustify::SpaceAround, 4),
    ] {
        assert_eq!(postcard::to_allocvec(&v).expect("ser")[0], want, "{v:?}");
    }
}
