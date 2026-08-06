//! Gates da **persistência da tabela de cor** (plano UI/UX W6, degrau 1).
//!
//! ⚠️ O keystone é o do ESQUECIMENTO: um load que ACRESCENTA em vez de instalar deixa o app com as
//! cores do documento anterior, e nada na tela diz porquê. É a mesma classe do bug que a timeline
//! pagou no W4.T6/B5, e ela não é visível num round-trip — só num SEGUNDO load.

use super::*;

/// Escrever um LITERAL — a forma destes gates antes de o alias existir. O `expect` documenta a
/// propriedade no sítio: um literal TERMINA uma cadeia, então a porta nunca o recusa.
fn put(theme: Theme, token: ColorToken, colour: Option<Color>) {
    ph2d_tokens::overrides::set_color_override(theme, token, colour.map(TokenValue::Literal))
        .expect("um literal nunca fecha um laco");
}

/// **O round-trip devolve exactamente o que saiu.**
#[test]
fn the_table_round_trips() {
    let _ = set_color_overrides(Vec::new());
    let c = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
        a: 0xFF,
    };
    put(Theme::Forge, ColorToken::Accent, Some(c));
    put(Theme::Sunstone, ColorToken::Text1, Some(c));
    let saved = collect();
    assert_eq!(saved.len(), 2);

    let _ = set_color_overrides(Vec::new());
    install(&saved);
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), c);
    assert_eq!(ColorToken::Text1.resolve(Theme::Sunstone), c);
    let _ = set_color_overrides(Vec::new());
}

/// **O load ESQUECE a tabela do documento anterior** — o keystone.
///
/// ⚠️ Um `install` que acrescentasse passaria no round-trip acima e deixaria o app com as cores do
/// projeto A depois de abrir o projeto B, com todos os outros gates verdes.
#[test]
fn loading_forgets_the_previous_document() {
    let _ = set_color_overrides(Vec::new());
    let a = Color {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
        a: 0xFF,
    };
    // O "projeto A": um token autorado.
    put(Theme::Forge, ColorToken::Accent, Some(a));
    let factory = {
        // Guarda a cor de fábrica ANTES de abrir o "projeto B" — é contra ela que se mede.
        let _ = set_color_overrides(Vec::new());
        let f = ColorToken::Accent.resolve(Theme::Forge);
        put(Theme::Forge, ColorToken::Accent, Some(a));
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
    let _ = set_color_overrides(Vec::new());
    let good = SavedToken {
        theme: 0,
        key: ColorToken::Accent.key().to_string(),
        value: SavedValue::Literal([1, 2, 3, 255]),
    };
    let gone = SavedToken {
        theme: 0,
        key: "um-token-que-nunca-existiu".to_string(),
        value: SavedValue::Literal([9, 9, 9, 255]),
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
    let _ = set_color_overrides(Vec::new());
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

/// **Um ELO sobrevive ao arquivo — e viaja pela CHAVE, nunca pelo índice.**
///
/// ⚠️ A segunda metade é a que importa e não é visível num round-trip normal: guardar o índice do
/// variant amarraria todo projeto salvo à ORDEM da lista, e acrescentar um token no meio faria a
/// arte de ontem seguir outro token. O oráculo é o BYTE do arquivo conter a chave.
#[test]
fn a_link_survives_the_file_and_travels_by_key() {
    let _ = set_color_overrides(Vec::new());
    let (a, b) = (ColorToken::Border, ColorToken::Accent);
    ph2d_tokens::overrides::set_color_override(Theme::Forge, a, Some(TokenValue::Alias(b)))
        .expect("a fixture nao fecha laco");

    let saved = collect();
    assert_eq!(saved.len(), 1);
    let bytes = postcard::to_allocvec(&saved).expect("o registro tem de serializar");
    assert!(
        String::from_utf8_lossy(&bytes).contains(b.key()),
        "a CHAVE do alvo nao esta' no arquivo — ele guardou um indice"
    );

    let _ = set_color_overrides(Vec::new());
    assert_ne!(a.resolve(Theme::Forge), b.resolve(Theme::Forge));
    install(&saved);
    assert_eq!(
        a.resolve(Theme::Forge),
        b.resolve(Theme::Forge),
        "o elo nao sobreviveu ao arquivo"
    );
    let _ = set_color_overrides(Vec::new());
}

/// **Um elo cujo ALVO já não existe é DESCARTADO, não recusa o projeto.**
///
/// Mesma lei do token desconhecido: a tabela de fábrica é a autoridade sobre quais tokens existem,
/// e um elo pendurado no vazio não tem valor a devolver.
#[test]
fn a_link_to_a_vanished_token_is_dropped_not_fatal() {
    let _ = set_color_overrides(Vec::new());
    let dangling = SavedToken {
        theme: 0,
        key: ColorToken::Border.key().to_string(),
        value: SavedValue::Alias("um-token-que-nunca-existiu".to_string()),
    };
    assert_eq!(install(&[dangling]), 1, "o elo pendurado nao foi CONTADO");
    assert!(
        color_overrides().is_empty(),
        "o elo pendurado no vazio entrou na tabela"
    );
}

/// **Um arquivo com um LAÇO abre, perdendo só o elo que o fecha.**
///
/// ⚠️ Recusar o projeto inteiro jogaria fora uma re-vestida por causa de duas linhas; aceitar
/// poria na tabela o laço que a porta de escrita promete não ter. O que sobrevive é acíclico.
#[test]
fn a_file_carrying_a_loop_still_opens() {
    let _ = set_color_overrides(Vec::new());
    let key = |t: ColorToken| t.key().to_string();
    let (a, b) = (ColorToken::Border, ColorToken::Accent);
    let dropped = install(&[
        SavedToken {
            theme: 0,
            key: key(a),
            value: SavedValue::Alias(key(b)),
        },
        SavedToken {
            theme: 0,
            key: key(b),
            value: SavedValue::Alias(key(a)),
        },
    ]);
    assert_eq!(color_overrides().len(), 1, "o laco tinha de perder UM elo");
    // ⚠️ E o descarte é CONTADO — uma tabela que encolhe sem dizer quanto le-se como "eu nunca
    // autorei isto", e o artista procuraria a cor onde ela nao esta'.
    assert_eq!(dropped, 1, "o elo descartado nao foi CONTADO");
    // E a tabela resolve — não gira, não entra em pânico.
    assert_eq!(a.resolve(Theme::Forge), b.resolve(Theme::Forge));
    let _ = set_color_overrides(Vec::new());
}

// ═══════════════════════════════════════════════════════════════════════════
// A família NUMÉRICA (plano UI/UX W4c.1) — ela viaja na MESMA lista.
// ═══════════════════════════════════════════════════════════════════════════

use ph2d_tokens::num_overrides::{clear_num_overrides, num_override};

fn wipe() {
    let _ = set_color_overrides(Vec::new());
    clear_num_overrides();
}

/// **Um comprimento autorado sobrevive ao save**, e viaja pela CHAVE.
#[test]
fn a_numeric_token_round_trips() {
    wipe();
    ph2d_tokens::num_overrides::set_num_override(
        Theme::Forge,
        NumToken::ALL[0],
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    let saved = collect();
    assert_eq!(saved.len(), 1);
    assert_eq!(
        saved[0].key,
        NumToken::ALL[0].key(),
        "viajou o indice, nao a chave"
    );

    wipe();
    assert_eq!(install(&saved), 0);
    assert_eq!(NumToken::ALL[0].px(Theme::Forge), 13.0);
    wipe();
}

/// ⚠️ **As DUAS famílias na MESMA lista, e o load devolve cada uma à sua camada.**
///
/// Sem a rota por CHAVE, uma entrada de `spacing.md` cairia no `from_key` da cor, devolveria `None`
/// e seria **contada como descartada** — o artista veria *"1 token descartado"* sobre um arquivo
/// perfeitamente bom, e a escala não voltaria.
#[test]
fn the_two_families_share_one_list_and_each_lands_in_its_own_layer() {
    wipe();
    let c = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
        a: 0xFF,
    };
    put(Theme::Forge, ColorToken::Accent, Some(c));
    ph2d_tokens::num_overrides::set_num_override(
        Theme::Forge,
        NumToken::ALL[0],
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    let saved = collect();
    assert_eq!(saved.len(), 2);

    wipe();
    assert_eq!(
        install(&saved),
        0,
        "uma entrada boa foi contada como descarte"
    );
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), c);
    assert_eq!(NumToken::ALL[0].px(Theme::Forge), 13.0);
    wipe();
}

/// **O load ESQUECE a escala do documento anterior** — o keystone, na outra família.
///
/// ⚠️ Um `install` que só instalasse a família presente no arquivo deixaria a escala do projeto A
/// de pé debaixo das cores do projeto B, e nada na tela diria porquê.
#[test]
fn loading_forgets_the_previous_documents_scale() {
    wipe();
    ph2d_tokens::num_overrides::set_num_override(
        Theme::Forge,
        NumToken::ALL[0],
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    // Um arquivo que NAO tem tokens numericos nenhuns.
    install(&[]);
    assert_eq!(
        num_override(Theme::Forge, NumToken::ALL[0]),
        None,
        "a escala do documento ANTERIOR sobreviveu ao load"
    );
    wipe();
}

/// **Um par mal-emparelhado CAI** — chave de uma família, valor da outra.
///
/// ⚠️ Nenhuma porta emite isto; só um arquivo editado à mão. Dobrá-lo para o que der faria a
/// re-vestida inventar-se sozinha, então ele é descartado e **contado**.
#[test]
fn a_key_and_value_from_different_families_is_dropped_and_counted() {
    wipe();
    let bad = vec![
        SavedToken {
            theme: 0,
            key: NumToken::ALL[0].key().to_string(),
            value: SavedValue::Literal([1, 2, 3, 4]),
        },
        SavedToken {
            theme: 0,
            key: ColorToken::Accent.key().to_string(),
            value: SavedValue::Number(13.0),
        },
    ];
    assert_eq!(install(&bad), 2);
    assert_eq!(num_override(Theme::Forge, NumToken::ALL[0]), None);
    wipe();
}

/// Um token numérico que o design system já não tem é **descartado e contado** — a tabela de
/// fábrica é a autoridade sobre quais tokens existem.
#[test]
fn an_unknown_numeric_key_is_dropped_and_counted() {
    wipe();
    let bad = vec![SavedToken {
        theme: 0,
        key: "spacing.nope".to_string(),
        value: SavedValue::Number(13.0),
    }];
    assert_eq!(install(&bad), 1);
    wipe();
}
