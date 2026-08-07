//! Os gates do IMPORT — *o que existe entra, o que não serve cai e é CONTADO*, e o round-trip.

use super::*;
use crate::export::{VENDOR_KEY, export};
use ph2d_tokens::num::NumToken;
use ph2d_tokens::num_overrides::{NumValue, num_overrides, set_num_override, set_num_overrides};
use ph2d_tokens::overrides::{
    TokenValue, color_overrides, set_color_override, set_color_overrides,
};
use ph2d_tokens::spacing::Spacing;
use ph2d_tokens::{ColorToken, Theme};

fn one(body: &str) -> Imported {
    import(body, Theme::Forge).expect("o corpo dos gates e' JSON valido")
}

/// **Instala o parser de math** — sem ele a porta de escrita RECUSA uma fórmula, e um gate de
/// round-trip não conseguiria sequer criar a que ele quer ver voltar. É a MESMA porta do boot.
fn with_math() {
    ph2d_token_math::install();
}

/// **Um arquivo que não é JSON DIZ porquê** — um import que não acontece sem nada na tela é
/// indistinguível de um botão quebrado.
#[test]
fn a_file_that_is_not_usable_says_why() {
    assert!(matches!(
        import("{ nao sou json", Theme::Forge),
        Err(DtcgError::NotJson(_))
    ));
    assert_eq!(import("[]", Theme::Forge), Err(DtcgError::NotAGroup));
}

/// **Uma chave que este design system não tem é CONTADA, não instalada.**
#[test]
fn an_unknown_key_is_counted_not_installed() {
    let r = one(r##"{"chartreuse":{"$type":"color","$value":"#123456"}}"##);
    assert_eq!(r.unknown, 1);
    assert_eq!(r.authored(), 0);
}

/// **As DUAS formas de cor são lidas** — a de 2025.10 e a string de hex dos rascunhos anteriores.
///
/// ⚠️ Recusar a antiga faria o interop falhar exactamente nos arquivos que há para importar hoje.
#[test]
fn both_colour_forms_are_read() {
    let legacy = one(r##"{"accent":{"$type":"color","$value":"#0a0b0c"}}"##);
    assert_eq!(legacy.authored(), 1);
    assert_eq!(
        legacy.colours[0].value,
        TokenValue::Literal(Color {
            r: 10,
            g: 11,
            b: 12,
            a: 255
        })
    );

    let modern = one(
        r##"{"accent":{"$type":"color","$value":{"colorSpace":"srgb",
           "components":[0.039216,0.043137,0.047059],"alpha":1,"hex":"#0a0b0c"}}}"##,
    );
    assert_eq!(modern.colours[0].value, legacy.colours[0].value);
}

/// **`#abc` é `#aabbcc`** — cada dígito REPETE, e não é multiplicar por 16.
#[test]
fn the_three_digit_hex_repeats_each_digit() {
    let r = one(r##"{"accent":{"$value":"#abc"}}"##);
    assert_eq!(
        r.colours[0].value,
        TokenValue::Literal(Color {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 255
        })
    );
}

/// **Os `components` vencem o `hex`; num espaço que não sabemos ler, o `hex` é a ponte.**
#[test]
fn the_components_win_but_a_foreign_colour_space_falls_back_to_the_hex() {
    // Os dois discordam de propósito: quem ganha é observável.
    let r = one(
        r##"{"accent":{"$value":{"colorSpace":"srgb","components":[1,0,0],"hex":"#000000"}}}"##,
    );
    assert_eq!(
        r.colours[0].value,
        TokenValue::Literal(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255
        })
    );

    let wide = one(
        r##"{"accent":{"$value":{"colorSpace":"display-p3","components":[1,0,0],"hex":"#010203"}}}"##,
    );
    assert_eq!(
        wide.colours[0].value,
        TokenValue::Literal(Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255
        }),
        "fora do sRGB o hex e' a unica coisa que sabemos ler — e' para isso que ele existe"
    );

    // E sem nenhum dos dois, cai e é contado.
    let neither = one(r#"{"accent":{"$value":{"colorSpace":"lab","components":[1,0,0]}}}"#);
    assert_eq!(neither.authored(), 0);
    assert_eq!(neither.dropped, 1);
}

/// **As formas de dimensão que o ecossistema emite entram; `rem` NÃO.**
///
/// ⚠️ Converter `rem` exige um tamanho de fonte-raiz que este app não tem, e escrever `16` seria
/// inventar um número que ninguém autorou. Ele cai **e é contado**, que é a diferença entre um
/// limite e um silêncio.
#[test]
fn every_px_form_is_read_and_rem_is_refused_and_counted() {
    for body in [
        r#"{"spacing":{"md":{"$value":{"value":13,"unit":"px"}}}}"#,
        r#"{"spacing":{"md":{"$value":"13px"}}}"#,
        r#"{"spacing":{"md":{"$value":13}}}"#,
        r#"{"spacing":{"md":{"$value":{"value":13}}}}"#,
    ] {
        let r = one(body);
        assert_eq!(r.authored(), 1, "{body}");
        assert_eq!(r.nums[0].value, NumValue::Literal(13.0), "{body}");
    }

    let rem = one(r#"{"spacing":{"md":{"$value":{"value":1,"unit":"rem"}}}}"#);
    assert_eq!(rem.authored(), 0);
    assert_eq!(rem.dropped, 1, "o rem tem de ser CONTADO, nao ignorado");
}

/// **Uma referência vira um ALIAS, nas duas famílias.**
#[test]
fn a_reference_becomes_an_alias() {
    let c = one(r#"{"border-strong":{"$value":"{accent}"}}"#);
    assert_eq!(c.colours[0].value, TokenValue::Alias(ColorToken::Accent));

    let n = one(r#"{"radius":{"md":{"$value":"{spacing.md}"}}}"#);
    assert_eq!(
        n.nums[0].value,
        NumValue::Alias(NumToken::Spacing(Spacing::Md))
    );

    // Um elo pendurado no vazio cai e é contado.
    let dead = one(r#"{"border-strong":{"$value":"{chartreuse}"}}"#);
    assert_eq!(dead.authored(), 0);
    assert_eq!(dead.dropped, 1);
}

/// **A FÓRMULA do `$extensions` vence o `$value`.**
///
/// ⚠️ O `$value` de uma linha com fórmula é o número que ela DEU — tomá-lo seria assar a fórmula
/// num literal, a mesma perda que achatar um alias.
#[test]
fn the_formula_extension_beats_the_resolved_value() {
    let body = format!(
        r#"{{"spacing":{{"lg":{{"$value":{{"value":999,"unit":"px"}},
           "$extensions":{{"{VENDOR_KEY}":{{"formula":"{{spacing.md}} * 2"}}}}}}}}}}"#
    );
    let r = one(&body);
    assert_eq!(r.nums[0].value, NumValue::Expr("{spacing.md} * 2".into()));
}

/// **O `$type` NÃO decide a família** — a chave decide, e ele é só uma anotação.
///
/// ⚠️ Várias ferramentas escrevem `"number"` onde a spec diz `"dimension"`. Cair por causa disso
/// seria recusar o arquivo por uma anotação em vez de por um valor.
#[test]
fn the_type_annotation_does_not_decide_the_family() {
    let r = one(r#"{"spacing":{"md":{"$type":"number","$value":13}}}"#);
    assert_eq!(r.authored(), 1);
    let none = one(r#"{"spacing":{"md":{"$value":13}}}"#);
    assert_eq!(none.authored(), 1);
}

/// **Grupos aninham por PONTO, em qualquer profundidade.**
#[test]
fn groups_nest_by_dots() {
    let r = one(r#"{"spacing":{"md":{"$value":13},"lg":{"$value":21}}}"#);
    assert_eq!(r.authored(), 2);
}

/// ⭐ **UM VALOR QUE JÁ É O DE FÁBRICA NÃO É AUTORADO** — a lei desta crate.
///
/// ⚠️ Sem ela, reimportar um export de um projeto intocado autoraria os ~80 tokens de uma vez, e a
/// partir daí re-editar o `docs/design/tokens.json` deixaria de alcançar o app **em silêncio**.
#[test]
fn a_value_that_already_is_the_factory_authors_nothing() {
    let md = NumToken::Spacing(Spacing::Md).factory_px();
    let r = one(&format!(r#"{{"spacing":{{"md":{{"$value":{md}}}}}}}"#));
    assert_eq!(r.authored(), 0);
    assert_eq!(r.at_factory, 1);

    let c = ColorToken::Accent.factory(Theme::Forge);
    let hex = format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
    let r = one(&format!(r#"{{"accent":{{"$value":"{hex}"}}}}"#));
    assert_eq!(r.authored(), 0);
    assert_eq!(r.at_factory, 1);
}

/// **Mas um ALIAS e uma FÓRMULA autoram sempre** — são estruturais.
///
/// ⚠️ O artista autorou o *vínculo*; o número que ele por acaso dá hoje não o desfaz.
#[test]
fn an_alias_or_a_formula_authors_even_when_it_resolves_to_the_factory() {
    // ⚠️ A fixture precisa de dois tokens que **já valem o mesmo** — o elo entre eles não move um
    // pixel, e mesmo assim é uma decisão que o arquivo carrega. Ela ACHA o par em vez de o nomear:
    // um par escrito à mão depende de dois números do `tokens.json` que ninguém prometeu manter.
    let a = NumToken::Radius(ph2d_tokens::radius::Radius::Md);
    let b = NumToken::ALL
        .iter()
        .copied()
        .find(|&t| t != a && t.factory_px() == a.factory_px())
        .expect("a escala tem de ter dois degraus de mesmo valor para esta fixture existir");
    let r = one(&format!(
        r#"{{"radius":{{"md":{{"$value":"{{{}}}"}}}}}}"#,
        b.key()
    ));
    assert_eq!(r.authored(), 1, "um elo autora mesmo valendo a fabrica");
    assert_eq!(r.at_factory, 0);
}

// ─────────────────────────── O ROUND-TRIP ───────────────────────────

/// ⭐ **O round-trip de uma tabela INTOCADA autora ZERO.**
///
/// É a metade que torna o export-da-tabela-inteira seguro: sem ela, um `Export` seguido de
/// `Import` transformaria os ~80 tokens em overrides e a tabela de fábrica ficaria inalcançável.
#[test]
fn the_round_trip_of_an_untouched_table_authors_nothing() {
    let file = export(Theme::Forge);
    let r = import(&file, Theme::Forge).expect("o nosso proprio export");
    assert_eq!(
        r.authored(),
        0,
        "{} tokens voltaram como AUTORADOS de uma tabela de fabrica",
        r.authored()
    );
    assert_eq!(
        r.unknown, 0,
        "o nosso export nao pode ter chaves que nao temos"
    );
    assert_eq!(
        r.dropped, 0,
        "o nosso export nao pode ter valores que nao lemos"
    );
    assert_eq!(r.at_factory, ColorToken::ALL.len() + NumToken::ALL.len());
}

/// ⭐ **O que o artista autorou volta como ele o autorou** — literal, elo e fórmula.
#[test]
fn the_round_trip_preserves_literals_aliases_and_formulas() {
    with_math();
    let red = Color {
        r: 220,
        g: 30,
        b: 40,
        a: 255,
    };
    set_color_override(
        Theme::Forge,
        ColorToken::Accent,
        Some(TokenValue::Literal(red)),
    )
    .expect("literal");
    set_color_override(
        Theme::Forge,
        ColorToken::BorderStrong,
        Some(TokenValue::Alias(ColorToken::Accent)),
    )
    .expect("elo");
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Lg),
        Some(NumValue::Expr("{spacing.md} * 2".into())),
    )
    .expect("formula");
    set_num_override(
        Theme::Forge,
        NumToken::Stroke(ph2d_tokens::stroke::StrokeToken::Thick),
        Some(NumValue::Literal(7.25)),
    )
    .expect("numero");

    let file = export(Theme::Forge);
    let want_c = color_overrides();
    let want_n = num_overrides();

    // A camada é limpa: o import tem de a reconstruir sozinho, do arquivo.
    set_color_overrides(Vec::new());
    set_num_overrides(Vec::new());

    let r = import(&file, Theme::Forge).expect("o nosso proprio export");
    assert_eq!(r.colours, want_c);
    assert_eq!(r.nums, want_n);
    assert_eq!(r.unknown + r.dropped, 0);
}

/// **O modo de destino é o do CHAMADOR, nunca o que o arquivo diz.**
///
/// ⚠️ O artista está a olhar para um modo e o painel nomeia-o; re-vestir um que ele não vê é a
/// mesma falha que o *Reset This Mode* evita ao só resetar o vigente.
#[test]
fn the_target_mode_is_the_callers_not_the_files() {
    let file = export(Theme::Forge); // o `$description` diz "forge"
    let r = import(&file, Theme::Blueprint).expect("o nosso proprio export");
    // Nada é autorado (a tabela é de fábrica), mas o que ele COMPAROU foi o Blueprint: as cores do
    // Forge não são as do Blueprint, então elas contam como autoradas ali.
    assert!(
        r.authored() > 0,
        "importar um arquivo do Forge para o Blueprint tem de autorar as cores que diferem"
    );
    assert!(r.colours.iter().all(|o| o.theme == Theme::Blueprint));
}
