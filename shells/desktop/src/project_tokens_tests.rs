//! Gates da **persistência da tabela de cor** (plano UI/UX W6, degrau 1).
//!
//! ⚠️ O keystone é o do ESQUECIMENTO: um load que ACRESCENTA em vez de instalar deixa o app com as
//! cores do documento anterior, e nada na tela diz porquê. É a mesma classe do bug que a timeline
//! pagou no W4.T6/B5, e ela não é visível num round-trip — só num SEGUNDO load.

use super::*;

/// **O round-trip devolve exactamente o que saiu.**
#[test]
fn the_table_round_trips() {
    set_color_overrides(Vec::new());
    let c = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
        a: 0xFF,
    };
    ph2d_tokens::overrides::set_color_override(Theme::Forge, ColorToken::Accent, Some(c));
    ph2d_tokens::overrides::set_color_override(Theme::Sunstone, ColorToken::Text1, Some(c));
    let saved = collect();
    assert_eq!(saved.len(), 2);

    set_color_overrides(Vec::new());
    install(&saved);
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), c);
    assert_eq!(ColorToken::Text1.resolve(Theme::Sunstone), c);
    set_color_overrides(Vec::new());
}

/// **O load ESQUECE a tabela do documento anterior** — o keystone.
///
/// ⚠️ Um `install` que acrescentasse passaria no round-trip acima e deixaria o app com as cores do
/// projeto A depois de abrir o projeto B, com todos os outros gates verdes.
#[test]
fn loading_forgets_the_previous_document() {
    set_color_overrides(Vec::new());
    let a = Color {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
        a: 0xFF,
    };
    // O "projeto A": um token autorado.
    ph2d_tokens::overrides::set_color_override(Theme::Forge, ColorToken::Accent, Some(a));
    let factory = {
        // Guarda a cor de fábrica ANTES de abrir o "projeto B" — é contra ela que se mede.
        set_color_overrides(Vec::new());
        let f = ColorToken::Accent.resolve(Theme::Forge);
        ph2d_tokens::overrides::set_color_override(Theme::Forge, ColorToken::Accent, Some(a));
        f
    };
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), a);

    // O "projeto B": um arquivo de fábrica (sem tokens autorados).
    install(&[]);
    assert_eq!(
        ColorToken::Accent.resolve(Theme::Forge),
        factory,
        "abrir um projeto de fabrica deixou as cores do documento ANTERIOR"
    );
}

/// **Um token que o design system já não tem é DESCARTADO, não recusa o projeto.**
///
/// ⚠️ A tabela de fábrica é a autoridade sobre quais tokens existem; o `PROJECT_SCHEMA` é quem
/// recusa FORMATO. Um arquivo perfeitamente legível não pode morrer porque um token foi renomeado.
#[test]
fn an_unknown_token_key_is_dropped_not_fatal() {
    set_color_overrides(Vec::new());
    let good = SavedToken {
        theme: 0,
        key: ColorToken::Accent.key().to_string(),
        rgba: [1, 2, 3, 255],
    };
    let gone = SavedToken {
        theme: 0,
        key: "um-token-que-nunca-existiu".to_string(),
        rgba: [9, 9, 9, 255],
    };
    install(&[good, gone]);
    assert_eq!(color_overrides().len(), 1, "o desconhecido nao caiu");
    assert_eq!(
        ColorToken::Accent.resolve(Theme::Forge),
        Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255
        }
    );
    set_color_overrides(Vec::new());
}

/// **O modo viaja pelo byte, e o byte volta ao mesmo modo.**
///
/// ⚠️ Um byte que o enum não tem cai no default em vez de recusar o arquivo — o `PROJECT_SCHEMA`
/// é quem recusa formato.
#[test]
fn the_mode_round_trips_through_its_byte() {
    for (b, expected) in [
        (0u8, Theme::Forge),
        (1, Theme::Workshop),
        (2, Theme::Sunstone),
        (3, Theme::Blueprint),
        (250, Theme::Forge),
    ] {
        assert_eq!(
            theme_from_u8(b),
            expected,
            "o byte {b} nao deu o modo certo"
        );
    }
    // E a direção de ida concorda com a de volta em TODO modo — sem isto as duas poderiam
    // divergir e um projeto salvo no Sunstone reabriria no Blueprint.
    for t in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        assert_eq!(theme_from_u8(t as u8), t);
    }
}
