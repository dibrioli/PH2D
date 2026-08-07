//! Os gates da porta de roteamento — *a chave decide a família, e o valor tem de caber nela*.

use super::*;
use crate::radius::Radius;
use crate::spacing::Spacing;

const RED: Color = Color {
    r: 255,
    g: 0,
    b: 0,
    a: 255,
};

/// **Uma chave de COR com uma cor entra como cor.**
#[test]
fn a_colour_key_with_a_colour_routes_to_the_colour_family() {
    let r = route(Theme::Forge, "accent", AuthoredValue::Colour(RED));
    assert!(matches!(
        r,
        Some(Routed::Colour(ColorOverride {
            token: ColorToken::Accent,
            value: TokenValue::Literal(RED),
            ..
        }))
    ));
}

/// **Uma chave NUMÉRICA com um número entra como número.**
#[test]
fn a_numeric_key_with_a_length_routes_to_the_numeric_family() {
    let r = route(Theme::Forge, "spacing.md", AuthoredValue::Px(13.0));
    let Some(Routed::Num(o)) = r else {
        panic!("esperava a familia numerica, veio {r:?}");
    };
    assert_eq!(o.token, NumToken::Spacing(Spacing::Md));
    assert_eq!(o.value, NumValue::Literal(13.0));
}

/// **O par TROCADO cai** — e é a metade que impede a re-vestida de se inventar sozinha.
///
/// ⚠️ Nenhuma porta deste app emite uma cor sob `spacing.md`; só um arquivo editado à mão o
/// produz. Dobrá-lo para o que der faria um `spacing` mal-emparelhado pintar uma cor.
#[test]
fn a_value_from_the_other_family_is_dropped() {
    assert_eq!(
        route(Theme::Forge, "spacing.md", AuthoredValue::Colour(RED)),
        None
    );
    assert_eq!(route(Theme::Forge, "accent", AuthoredValue::Px(4.0)), None);
    assert_eq!(
        route(
            Theme::Forge,
            "accent",
            AuthoredValue::Formula("{spacing.md}")
        ),
        None
    );
}

/// **Uma chave que o design system não tem cai.**
#[test]
fn an_unknown_key_is_dropped() {
    assert_eq!(
        route(Theme::Forge, "spacing.enormous", AuthoredValue::Px(4.0)),
        None
    );
    assert_eq!(
        route(Theme::Forge, "chartreuse", AuthoredValue::Colour(RED)),
        None
    );
}

/// **Um elo pendurado no vazio cai** — nas duas famílias.
#[test]
fn an_alias_to_a_key_that_does_not_exist_is_dropped() {
    assert_eq!(
        route(Theme::Forge, "border", AuthoredValue::Alias("chartreuse")),
        None
    );
    assert_eq!(
        route(
            Theme::Forge,
            "radius.md",
            AuthoredValue::Alias("spacing.enormous")
        ),
        None
    );
}

/// **Um alias ATRAVESSA famílias dentro da numérica** (px é px) e **nunca** entre cor e px.
#[test]
fn an_alias_crosses_scales_but_never_the_colour_boundary() {
    let r = route(
        Theme::Forge,
        "radius.md",
        AuthoredValue::Alias("spacing.md"),
    );
    let Some(Routed::Num(o)) = r else {
        panic!("radius.md -> spacing.md e' legal: as duas medem px");
    };
    assert_eq!(o.token, NumToken::Radius(Radius::Md));
    assert_eq!(o.value, NumValue::Alias(NumToken::Spacing(Spacing::Md)));

    // Uma cor a seguir um comprimento não tem valor a devolver.
    assert_eq!(
        route(Theme::Forge, "accent", AuthoredValue::Alias("spacing.md")),
        None
    );
    assert_eq!(
        route(Theme::Forge, "spacing.md", AuthoredValue::Alias("accent")),
        None
    );
}

/// **O modo atravessa a porta intacto** — o roteamento não tem opinião sobre ele.
#[test]
fn the_mode_is_carried_through_untouched() {
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let Some(Routed::Colour(o)) = route(theme, "accent", AuthoredValue::Colour(RED)) else {
            panic!("accent e' uma cor em todo modo");
        };
        assert_eq!(o.theme, theme);
    }
}

/// **A FÁBRICA responde pelas duas famílias, e só para chaves que existem.**
#[test]
fn the_factory_answers_for_both_families_and_only_for_keys_that_exist() {
    assert!(matches!(
        factory(Theme::Forge, "accent"),
        Some(Factory::Colour(_))
    ));
    assert_eq!(
        factory(Theme::Forge, "spacing.md"),
        Some(Factory::Px(NumToken::Spacing(Spacing::Md).factory_px()))
    );
    assert_eq!(factory(Theme::Forge, "chartreuse"), None);
}

/// **A fábrica de uma COR muda com o modo; a de um comprimento NÃO.**
///
/// ⚠️ Não é assimetria acidental: o `tokens.json` guarda `spacing.*` no topo, fora de `themes`, e
/// é isso que faz um export por modo trazer a mesma escala nos quatro. Um import que assumisse o
/// contrário escreveria a escala do modo errado.
#[test]
fn the_colour_factory_is_per_mode_and_the_scale_factory_is_not() {
    // ⚠️ A propriedade é *"as duas tabelas DIFEREM"*, e não *"este token difere"*: nomear um
    // token faz a fixture depender de um valor do `tokens.json` que ninguém prometeu manter — o
    // `bg-0` do Forge e o do Workshop são hoje a MESMA cor, e o gate falhava sobre produto certo.
    assert!(
        ColorToken::ALL
            .iter()
            .any(|t| factory(Theme::Forge, t.key()) != factory(Theme::Workshop, t.key())),
        "nenhum token de cor difere entre dois modos — as tabelas por modo estao a colapsar"
    );

    for t in NumToken::ALL {
        assert_eq!(
            factory(Theme::Forge, t.key()),
            factory(Theme::Workshop, t.key()),
            "{} mudou de valor com o modo — a escala de fabrica nao tem modo",
            t.key()
        );
    }
}

/// **A fábrica de uma cor é CEGA à camada de override** — é essa a diferença para o `resolve`.
///
/// ⚠️ Sem esta propriedade o import DTCG deixaria de ser idempotente na segunda passagem: ele
/// compara o que chega contra a fábrica, e se a "fábrica" já carregasse o override o segundo
/// import diria *"isto é igual"* sobre a escolha do artista e a soltaria.
#[test]
fn the_factory_does_not_see_the_override_layer() {
    let before = ColorToken::Accent.factory(Theme::Forge);
    crate::overrides::set_color_override(
        Theme::Forge,
        ColorToken::Accent,
        Some(TokenValue::Literal(RED)),
    )
    .expect("um literal nunca fecha um laco");
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), RED);
    assert_eq!(ColorToken::Accent.factory(Theme::Forge), before);
    crate::overrides::clear_color_overrides();
}
