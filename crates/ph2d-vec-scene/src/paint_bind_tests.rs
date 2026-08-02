//! Os gates da tinta resolvida.

use super::*;
use crate::{StrokeSpec, rectangle};

fn shape() -> VecPath {
    let mut p = rectangle([0.0, 0.0], [2.0, 1.0]);
    p.fill = Some(Paint::Solid(Rgba8::new(10, 20, 30, 255)));
    p
}

/// **Sem binding o desenho é o MESMO ponteiro.**
///
/// Não é higiene: é o que torna seguro chamar a porta em toda forma da cena, e o que garante que
/// todo documento que já existe desenha byte-idêntico ao mundo pré-token.
#[test]
fn a_shape_with_no_binding_is_the_very_same_path() {
    let p = shape();
    let borrowed = p.painted(None);
    assert!(
        matches!(borrowed, std::borrow::Cow::Borrowed(_)),
        "sem binding tem de emprestar, nao clonar"
    );
    assert!(std::ptr::eq(&*borrowed, &p), "e o MESMO ponteiro");

    // Uma entrada VAZIA (a forma está na tabela, mas nada resolveu) também não clona.
    let noop = BoundPaint {
        path: p.id,
        ..Default::default()
    };
    assert!(matches!(
        p.painted(Some(&noop)),
        std::borrow::Cow::Borrowed(_)
    ));
}

/// O token COBRE o literal no desenho — e o literal continua no documento.
#[test]
fn the_token_covers_the_literal_without_erasing_it() {
    let p = shape();
    let tok = Rgba8::new(200, 40, 90, 255);
    let drawn = p.painted(Some(&BoundPaint {
        path: p.id,
        fill: Some(tok),
        stroke: None,
    }));
    assert_eq!(drawn.fill, Some(Paint::Solid(tok)), "desenha o token");
    assert_eq!(
        p.fill,
        Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))),
        "o LITERAL do documento nao foi tocado — desbindar devolve a cor de antes"
    );
}

/// **Bindar o preenchimento de uma forma SEM preenchimento cria um** — uma cor descreve um
/// preenchimento por inteiro, então não há número inventado.
#[test]
fn binding_a_fill_where_there_is_none_paints_it() {
    let mut p = shape();
    p.fill = None;
    let tok = Rgba8::new(1, 2, 3, 255);
    let drawn = p.painted(Some(&BoundPaint {
        path: p.id,
        fill: Some(tok),
        stroke: None,
    }));
    assert_eq!(drawn.fill, Some(Paint::Solid(tok)));
}

/// **E bindar o traço de uma forma SEM traço NÃO cria um** — faltaria a largura, que o artista não
/// escreveu. O gate mede as duas metades: o traço que existe é colorido, e o que não existe segue
/// não existindo (e nem sequer clona).
#[test]
fn binding_a_stroke_colours_an_existing_stroke_and_never_invents_one() {
    let tok = Rgba8::new(7, 8, 9, 255);
    let b = |p: &VecPath| BoundPaint {
        path: p.id,
        fill: None,
        stroke: Some(tok),
    };

    let mut with = shape();
    with.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.05));
    let drawn = with.painted(Some(&b(&with)));
    assert_eq!(drawn.stroke.as_ref().map(|s| s.color), Some(tok));
    assert!(
        (drawn.stroke.as_ref().expect("tem traco").width - 0.05).abs() < 1e-12,
        "a largura AUTORADA sobrevive: o token e' de cor"
    );

    let without = shape();
    assert!(without.stroke.is_none(), "premissa da fixture");
    let drawn = without.painted(Some(&b(&without)));
    assert!(drawn.stroke.is_none(), "nao inventa traco");
    assert!(
        matches!(drawn, std::borrow::Cow::Borrowed(_)),
        "e nem clona: um binding que nao muda pixel nao paga copia"
    );
}
