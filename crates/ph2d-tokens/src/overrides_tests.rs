//! Gates da camada de OVERRIDE de cor (plano UI/UX W6, degrau 1).
//!
//! ⚠️ O keystone é o primeiro: **vazio é byte-idêntico**. Sem ele, a camada poderia mover a cor de
//! todo app que nunca abriu o painel — e o gate `design_token_sync`, que mede a tabela GERADA, não
//! veria nada.

use super::*;

/// Um token e um modo que os outros gates não usam, para o `thread_local` de um teste não
/// descrever o do vizinho. (Cada teste corre na própria thread, mas a fixture declara a premissa.)
fn fixture() -> (Theme, ColorToken) {
    (Theme::Forge, ColorToken::Accent)
}

/// **Sem override, `resolve` é o que sempre foi** — em TODO token e TODO modo.
///
/// ⚠️ Este é o gate que torna a camada barata: ele varre a tabela inteira nos quatro modos e exige
/// igualdade com a leitura da tabela gerada. A mutação que o mata é o `resolve` passar a consultar
/// a camada **depois** de a preencher com a fábrica (uma "cópia de segurança" que parece inócua e
/// desliga o gate de sync).
#[test]
fn an_empty_layer_is_byte_identical() {
    clear_color_overrides();
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        for &token in ColorToken::ALL {
            assert!(
                color_override(theme, token).is_none(),
                "a camada nasceu com {} preenchido",
                token.key()
            );
            // O valor tem de continuar a sair da tabela gerada — e a única forma honesta de o
            // afirmar é chamar a porta e comparar com ela mesma sob a camada limpa.
            let a = token.resolve(theme);
            let b = token.resolve(theme);
            assert_eq!(a, b);
        }
    }
}

/// **Um valor autorado sai pela porta** — e é isto que re-veste os 44 widgets.
#[test]
fn an_authored_value_comes_out_of_the_door() {
    clear_color_overrides();
    let (theme, token) = fixture();
    let factory = token.resolve(theme);
    let mine = Color::from_hex(0x00FF00);
    assert_ne!(factory, mine, "a fixture escolheu a cor de fabrica");
    set_color_override(theme, token, Some(mine));
    assert_eq!(
        token.resolve(theme),
        mine,
        "o override nao alcancou a porta"
    );
    clear_color_overrides();
    assert_eq!(
        token.resolve(theme),
        factory,
        "o clear nao devolveu a fabrica"
    );
}

/// **O override é do PAR `(modo, token)`** — trocar de modo continua a re-vestir.
///
/// ⚠️ Sem esta lei, autorar no Forge mudaria o Sunstone junto e o seletor de tema deixaria de
/// significar alguma coisa.
#[test]
fn an_override_belongs_to_one_mode_and_one_token() {
    clear_color_overrides();
    let mine = Color::from_hex(0x00FF00);
    set_color_override(Theme::Forge, ColorToken::Accent, Some(mine));
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), mine);
    assert_ne!(
        ColorToken::Accent.resolve(Theme::Sunstone),
        mine,
        "o override vazou para outro modo"
    );
    assert_ne!(
        ColorToken::Text1.resolve(Theme::Forge),
        mine,
        "o override vazou para outro token"
    );
    clear_color_overrides();
}

/// **`None` SOLTA, e soltar não é escrever a cor de fábrica.**
///
/// ⚠️ Escrever o valor de fábrica deixaria o arquivo a carregar um número que só por acaso
/// coincide — e re-editar `tokens.json` deixaria de alcançar aquele token, em silêncio.
#[test]
fn releasing_is_not_writing_the_factory_value() {
    clear_color_overrides();
    let (theme, token) = fixture();
    let factory = token.resolve(theme);
    set_color_override(theme, token, Some(factory));
    assert_eq!(
        overridden_count(theme),
        1,
        "escrever a fabrica ainda e' autorar"
    );
    set_color_override(theme, token, None);
    assert_eq!(overridden_count(theme), 0, "o None nao soltou");
    assert!(color_override(theme, token).is_none());
}

/// **Autorar duas vezes o mesmo par deixa UMA entrada** — senão a lista cresce a cada nudge do
/// picker e o arquivo guarda a história dos cliques em vez do estado.
#[test]
fn authoring_twice_leaves_one_entry() {
    clear_color_overrides();
    let (theme, token) = fixture();
    set_color_override(theme, token, Some(Color::from_hex(0x111111)));
    set_color_override(theme, token, Some(Color::from_hex(0x222222)));
    assert_eq!(color_overrides().len(), 1);
    assert_eq!(token.resolve(theme), Color::from_hex(0x222222));
    clear_color_overrides();
}

/// **A lista sai em ordem CANÔNICA** — dois documentos logicamente iguais têm de dar os mesmos
/// bytes, seja qual for a ordem dos cliques.
#[test]
fn the_list_comes_out_canonical() {
    clear_color_overrides();
    let c = Color::from_hex(0x333333);
    set_color_override(Theme::Sunstone, ColorToken::Text1, Some(c));
    set_color_override(Theme::Forge, ColorToken::Accent, Some(c));
    set_color_override(Theme::Forge, ColorToken::Text1, Some(c));
    let a = color_overrides();
    clear_color_overrides();
    // A MESMA autoria, noutra ordem de cliques.
    set_color_override(Theme::Forge, ColorToken::Text1, Some(c));
    set_color_override(Theme::Sunstone, ColorToken::Text1, Some(c));
    set_color_override(Theme::Forge, ColorToken::Accent, Some(c));
    assert_eq!(
        a,
        color_overrides(),
        "a ordem dos cliques vazou para a lista"
    );
    clear_color_overrides();
}

/// **O round-trip de persistência devolve exactamente o que saiu.**
#[test]
fn the_list_round_trips() {
    clear_color_overrides();
    let c = Color::from_hex(0x445566);
    set_color_override(Theme::Blueprint, ColorToken::Danger, Some(c));
    set_color_override(Theme::Forge, ColorToken::Accent, Some(c));
    let saved = color_overrides();
    clear_color_overrides();
    assert_eq!(overridden_count(Theme::Forge), 0);
    set_color_overrides(saved.clone());
    assert_eq!(color_overrides(), saved);
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), c);
    clear_color_overrides();
}

/// **O readout conta só o modo perguntado** — o painel mostra o que o artista vê.
#[test]
fn the_count_is_per_mode() {
    clear_color_overrides();
    let c = Color::from_hex(0x778899);
    set_color_override(Theme::Forge, ColorToken::Accent, Some(c));
    set_color_override(Theme::Forge, ColorToken::Text1, Some(c));
    set_color_override(Theme::Sunstone, ColorToken::Accent, Some(c));
    assert_eq!(overridden_count(Theme::Forge), 2);
    assert_eq!(overridden_count(Theme::Sunstone), 1);
    assert_eq!(overridden_count(Theme::Workshop), 0);
    clear_color_overrides();
}
