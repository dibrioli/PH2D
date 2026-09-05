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
    let noop = BoundStyle {
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
    let drawn = p.painted(Some(&BoundStyle {
        path: p.id,
        fill: Some(tok),
        stroke: None,
        alpha: None,
        width: None,
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
    let drawn = p.painted(Some(&BoundStyle {
        path: p.id,
        fill: Some(tok),
        stroke: None,
        alpha: None,
        width: None,
    }));
    assert_eq!(drawn.fill, Some(Paint::Solid(tok)));
}

/// **E bindar o traço de uma forma SEM traço NÃO cria um** — faltaria a largura, que o artista não
/// escreveu. O gate mede as duas metades: o traço que existe é colorido, e o que não existe segue
/// não existindo (e nem sequer clona).
#[test]
fn binding_a_stroke_colours_an_existing_stroke_and_never_invents_one() {
    let tok = Rgba8::new(7, 8, 9, 255);
    let b = |p: &VecPath| BoundStyle {
        path: p.id,
        fill: None,
        stroke: Some(tok),
        alpha: None,
        width: None,
    };

    let mut with = shape();
    with.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.05));
    let drawn = with.painted(Some(&b(&with)));
    assert_eq!(drawn.stroke.as_ref().map(|s| s.color()), Some(tok));
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

/// ⭐⭐⭐ **QUALQUER opacidade é a identidade PARA A TINTA, e nem clona** (v19).
///
/// ⚠️ **Era *"opacidade CHEIA é a identidade"*, e o gate mudou com a lei** (2026-09-05): a
/// opacidade deixou de escalar a tinta e passou a ser a do OBJECTO, aplicada como camada por quem
/// desenha ([`crate::object_alpha`]). O que o `painted` faz com ela hoje é **nada**, e é isso que
/// este gate pina — a metade que sobra da frase antiga (*não paga um clone por forma por frame*)
/// passou a valer para toda a faixa do slider, e não só para o topo dela.
///
/// ⭐ **E é uma economia medida na wave anterior:** a chave do memo de FX é feita desta forma
/// pintada, então desvanecer uma forma **filtrada** deixou de a re-cozinhar 60 vezes por segundo.
#[test]
fn a_live_opacity_never_touches_the_paint_and_costs_nothing() {
    let p = shape();
    for a in [255_u8, 128, 0] {
        let live = BoundStyle {
            path: p.id,
            alpha: Some(a),
            ..BoundStyle::default()
        };
        let drawn = p.painted(Some(&live));
        assert!(
            matches!(drawn, std::borrow::Cow::Borrowed(_)),
            "alpha={a}: a opacidade viva nao pode clonar a forma — ela nao e' tinta"
        );
    }

    // E pela rota que CLONA (um token junto), a tinta tem de sair com o alfa AUTORADO: o clone
    // acontece pelo `fill`, e o que se prova aqui é que o alfa vivo não entra na cor.
    let with_token = BoundStyle {
        path: p.id,
        fill: Some(Rgba8::new(9, 9, 9, 255)),
        alpha: Some(51),
        ..BoundStyle::default()
    };
    let drawn = p.painted(Some(&with_token));
    assert_eq!(drawn.fill, Some(Paint::Solid(Rgba8::new(9, 9, 9, 255))));
}

/// ⭐⭐⭐ **A OPACIDADE VIVA SOBREPÕE A AUTORADA** — a lei dos outros três campos do [`BoundStyle`],
/// agora também na opacidade (v19).
///
/// ⚠️ Mutação que tem de sangrar: `object_alpha` MULTIPLICAR em vez de sobrepor. Multiplicar
/// parece inofensivo e é a segunda resposta à mesma pergunta: uma forma autorada a `0,5` com um
/// estado de UI a pedir `1,0` ficaria a meio — *o controlo no topo da barra deixaria de significar
/// «opaca»*.
#[test]
fn the_live_opacity_overrides_the_authored_one() {
    let mut p = shape();
    p.opacity = crate::Opacity::new(0.5);
    assert!(
        (crate::object_alpha(&p, None) - 0.5).abs() < 1e-6,
        "sem valor vivo, manda o documento"
    );
    let live = BoundStyle {
        path: p.id,
        alpha: Some(u8::MAX),
        ..BoundStyle::default()
    };
    assert!(
        (crate::object_alpha(&p, Some(&live)) - 1.0).abs() < 1e-6,
        "o vivo COBRE o autorado — nao o multiplica"
    );
    let quarter = BoundStyle {
        path: p.id,
        alpha: Some(64),
        ..BoundStyle::default()
    };
    assert!(
        (crate::object_alpha(&p, Some(&quarter)) - 64.0 / 255.0).abs() < 1e-6,
        "e o valor vivo chega inteiro"
    );
}

/// **O `fade` arredonda ao mais próximo, e não trunca.**
///
/// ⚠️ O gate mede onde as duas contas DIFEREM (`100 * 130/255 = 50,98`), porque numa forma opaca
/// elas coincidem — foi a mutação que tirou o `+127` e sobreviveu que nomeou este buraco. Truncar
/// erra sempre para baixo, e uma cadeia de desvanecimentos escureceria meio nível de cada vez.
///
/// ⚠️ **Ele mede o [`crate::paint_bind::fade`] DIRECTAMENTE desde a v19**, e não o `painted`: a
/// opacidade viva saiu da tinta e virou camada, mas a função ficou — com um consumidor,
/// `brush_copies`, que desvanece a ARTE de um traço (tinta de verdade). *A lei não morreu; mudou
/// de dono, e o gate seguiu-a.*
#[test]
fn the_fade_rounds_to_nearest_instead_of_always_down() {
    let mut p = shape();
    p.fill = Some(Paint::Solid(Rgba8::new(9, 9, 9, 100)));
    crate::paint_bind::fade(&mut p, 130);
    assert_eq!(
        p.fill,
        Some(Paint::Solid(Rgba8::new(9, 9, 9, 51))),
        "100 * 130/255 = 50,98 — arredonda para 51; truncar daria 50"
    );
}

/// **O `fade` ESCALA o alfa e preserva a ESPÉCIE da tinta.**
///
/// ⚠️ O gradiente é o oráculo, e não o sólido: o atalho que esta wave recusou — trocar o `fill` por
/// uma cor com alfa — passaria no sólido e achataria todo gradiente em silêncio. Aqui cada parada
/// desvanece junto, e a rampa continua uma rampa.
///
/// ⚠️ **Mede o `fade` directamente desde a v19** (ver o irmão do arredondamento): a arte de um
/// pincel pode ser um gradiente ou um padrão, e é ela que passa por aqui agora.
#[test]
fn the_fade_dims_every_species_of_paint() {
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
    crate::paint_bind::fade(&mut p, 128);
    let drawn = &p;
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
        drawn.stroke.as_ref().map(|s| s.color().a),
        Some(128),
        "o traco desvanece com o resto — a forma inteira e' que fica translucida"
    );
}

/// ⛔⛔ **A ORDEM «opacidade DEPOIS do token» DISSOLVEU** (v19), e a nota fica porque a pergunta
/// era boa.
///
/// O gate antigo media que o slider não ficava inerte numa forma bindada a um token — um risco
/// real enquanto as duas coisas escreviam o MESMO campo (a cor). Com a opacidade a ser uma camada
/// sobre o desenho, ela desvanece **o que quer que tenha sido desenhado**: não há ordem a acertar,
/// e nenhuma tinta futura pode escapar-lhe por a tabela de espécies a esquecer.
///
/// *Uma lei que a representação torna verdadeira por construção não precisa de gate; precisa de
/// nota, para ninguém a reconstruir como código.* O que sobra de testável é o CONTROLE: o token
/// entra, a opacidade não toca na cor — e isso é o
/// [`a_live_opacity_never_touches_the_paint_and_costs_nothing`] acima.
#[test]
fn the_token_paints_and_the_live_opacity_stays_out_of_the_colour() {
    let p = shape();
    let both = BoundStyle {
        path: p.id,
        fill: Some(Rgba8::new(1, 2, 3, 255)),
        alpha: Some(51),
        ..BoundStyle::default()
    };
    let drawn = p.painted(Some(&both));
    assert_eq!(
        drawn.fill,
        Some(Paint::Solid(Rgba8::new(1, 2, 3, 255))),
        "o token pinta com o alfa DELE; a opacidade do objecto vive na camada"
    );
    assert!(
        (crate::object_alpha(&p, Some(&both)) - 51.0 / 255.0).abs() < 1e-6,
        "e o numero da camada e' o do valor vivo"
    );
}
