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
        alpha: None,
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
        alpha: None,
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
        alpha: None,
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

/// **Opacidade cheia é a IDENTIDADE, e nem clona.**
///
/// ⚠️ O gate do CONTROLE da W8b.3, e ele mede as duas metades da mesma frase: um slider no topo
/// não muda um byte da arte E não paga um clone por forma por frame.
#[test]
fn a_full_opacity_is_the_identity_and_costs_nothing() {
    let p = shape();
    let full = BoundPaint {
        path: p.id,
        alpha: Some(255),
        ..BoundPaint::default()
    };
    let drawn = p.painted(Some(&full));
    assert!(
        matches!(drawn, std::borrow::Cow::Borrowed(_)),
        "opacidade cheia nao pode custar um clone"
    );

    // E pela rota que CLONA (um token junto), o alfa tem de sair intacto: aqui o early-out não
    // salva — o clone acontece pelo `fill`, e o que prova a identidade é o `fades` ser falso.
    let with_token = BoundPaint {
        path: p.id,
        fill: Some(Rgba8::new(9, 9, 9, 255)),
        alpha: Some(255),
        ..BoundPaint::default()
    };
    let drawn = p.painted(Some(&with_token));
    assert_eq!(drawn.fill, Some(Paint::Solid(Rgba8::new(9, 9, 9, 255))));
}

/// **A opacidade arredonda ao mais próximo, e não trunca.**
///
/// ⚠️ O gate mede onde as duas contas DIFEREM (`100 * 130/255 = 50,98`), porque numa forma opaca
/// elas coincidem — foi a mutação que tirou o `+127` e sobreviveu que nomeou este buraco. Truncar
/// erra sempre para baixo, e uma cadeia de desvanecimentos escureceria meio nível de cada vez.
#[test]
fn the_opacity_rounds_to_nearest_instead_of_always_down() {
    let mut p = shape();
    p.fill = Some(Paint::Solid(Rgba8::new(9, 9, 9, 100)));
    let b = BoundPaint {
        path: p.id,
        alpha: Some(130),
        ..BoundPaint::default()
    };
    let drawn = p.painted(Some(&b));
    assert_eq!(
        drawn.fill,
        Some(Paint::Solid(Rgba8::new(9, 9, 9, 51))),
        "100 * 130/255 = 50,98 — arredonda para 51; truncar daria 50"
    );
}

/// **A opacidade ESCALA o alfa e preserva a ESPÉCIE da tinta.**
///
/// ⚠️ O gradiente é o oráculo, e não o sólido: o atalho que esta wave recusou — trocar o `fill` por
/// uma cor com alfa — passaria no sólido e achataria todo gradiente em silêncio. Aqui cada parada
/// desvanece junto, e a rampa continua uma rampa.
#[test]
fn the_opacity_fades_every_species_of_paint() {
    let mut p = shape();
    p.fill = Some(Paint::Linear {
        stops: vec![
            crate::GradientStop::new(0.0, Rgba8::new(255, 0, 0, 200)),
            crate::GradientStop::new(1.0, Rgba8::new(0, 0, 255, 100)),
        ],
        start: [0.0, 0.0],
        end: [1.0, 0.0],
    });
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.05));
    let half = BoundPaint {
        path: p.id,
        alpha: Some(128),
        ..BoundPaint::default()
    };
    let drawn = p.painted(Some(&half));
    let Some(Paint::Linear { stops, .. }) = drawn.fill.as_ref() else {
        panic!("a especie da tinta MUDOU — o gradiente foi achatado");
    };
    assert_eq!(stops[0].color.a, 100, "200 * 128/255");
    assert_eq!(stops[1].color.a, 50, "100 * 128/255");
    assert_eq!(
        stops[0].color.r, 255,
        "so' o ALFA desvanece; a cor autorada fica"
    );
    assert_eq!(
        drawn.stroke.as_ref().map(|s| s.color.a),
        Some(128),
        "o traco desvanece com o resto — a forma inteira e' que fica translucida"
    );
}

/// **A opacidade entra DEPOIS do token** — ela desvanece o que de fato vai ser desenhado.
///
/// Se entrasse antes, o token cobriria a cor já desvanecida com o alfa dele e o slider ficaria
/// inerte em toda forma bindada — inerte só ALI, que é a pior forma de um controle falhar.
#[test]
fn the_opacity_fades_what_the_token_put_there() {
    let p = shape();
    let both = BoundPaint {
        path: p.id,
        fill: Some(Rgba8::new(1, 2, 3, 255)),
        alpha: Some(51),
        ..BoundPaint::default()
    };
    let drawn = p.painted(Some(&both));
    assert_eq!(drawn.fill, Some(Paint::Solid(Rgba8::new(1, 2, 3, 51))));
}
