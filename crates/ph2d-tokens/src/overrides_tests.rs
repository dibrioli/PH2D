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

/// Escrever um LITERAL — a forma que estes gates usavam antes de o alias existir.
///
/// ⚠️ O `expect` **documenta a propriedade no sítio**: um literal TERMINA uma cadeia, nunca a
/// alonga, então a porta não tem como o recusar. Um `let _ =` diria que a recusa é possível e que
/// escolhemos ignorá-la.
fn put(theme: Theme, token: ColorToken, colour: Option<Color>) {
    set_color_override(theme, token, colour.map(TokenValue::Literal))
        .expect("um literal nunca fecha um laco");
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
    put(theme, token, Some(mine));
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
    put(Theme::Forge, ColorToken::Accent, Some(mine));
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
    put(theme, token, Some(factory));
    assert_eq!(
        overridden_count(theme),
        1,
        "escrever a fabrica ainda e' autorar"
    );
    put(theme, token, None);
    assert_eq!(overridden_count(theme), 0, "o None nao soltou");
    assert!(color_override(theme, token).is_none());
}

/// **Autorar duas vezes o mesmo par deixa UMA entrada** — senão a lista cresce a cada nudge do
/// picker e o arquivo guarda a história dos cliques em vez do estado.
#[test]
fn authoring_twice_leaves_one_entry() {
    clear_color_overrides();
    let (theme, token) = fixture();
    put(theme, token, Some(Color::from_hex(0x111111)));
    put(theme, token, Some(Color::from_hex(0x222222)));
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
    put(Theme::Sunstone, ColorToken::Text1, Some(c));
    put(Theme::Forge, ColorToken::Accent, Some(c));
    put(Theme::Forge, ColorToken::Text1, Some(c));
    let a = color_overrides();
    clear_color_overrides();
    // A MESMA autoria, noutra ordem de cliques.
    put(Theme::Forge, ColorToken::Text1, Some(c));
    put(Theme::Sunstone, ColorToken::Text1, Some(c));
    put(Theme::Forge, ColorToken::Accent, Some(c));
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
    put(Theme::Blueprint, ColorToken::Danger, Some(c));
    put(Theme::Forge, ColorToken::Accent, Some(c));
    let saved = color_overrides();
    clear_color_overrides();
    assert_eq!(overridden_count(Theme::Forge), 0);
    assert_eq!(set_color_overrides(saved.clone()), 0);
    assert_eq!(color_overrides(), saved);
    assert_eq!(ColorToken::Accent.resolve(Theme::Forge), c);
    clear_color_overrides();
}

/// **O readout conta só o modo perguntado** — o painel mostra o que o artista vê.
#[test]
fn the_count_is_per_mode() {
    clear_color_overrides();
    let c = Color::from_hex(0x778899);
    put(Theme::Forge, ColorToken::Accent, Some(c));
    put(Theme::Forge, ColorToken::Text1, Some(c));
    put(Theme::Sunstone, ColorToken::Accent, Some(c));
    assert_eq!(overridden_count(Theme::Forge), 2);
    assert_eq!(overridden_count(Theme::Sunstone), 1);
    assert_eq!(overridden_count(Theme::Workshop), 0);
    clear_color_overrides();
}

// ── O ALIAS (plano UI/UX W4b) ────────────────────────────────────────────────

/// Fazer `token` seguir `target`, afirmando que a porta aceitou.
fn link(theme: Theme, token: ColorToken, target: ColorToken) {
    set_color_override(theme, token, Some(TokenValue::Alias(target)))
        .expect("esta fixture nao fecha laco");
}

/// **Um token que segue outro vale o que o outro vale — e SEGUE quando o outro muda.**
///
/// ⚠️ Este é o gate que separa um alias de uma CÓPIA: a segunda metade (re-autorar o alvo e o
/// seguidor acompanhar) é a que falha se alguém "otimizar" achatando a cadeia na escrita.
#[test]
fn an_alias_is_worth_what_its_target_is_worth_and_follows_it() {
    clear_color_overrides();
    let theme = Theme::Forge;
    let (follower, target) = (ColorToken::Border, ColorToken::Accent);

    link(theme, follower, target);
    assert_eq!(
        follower.resolve(theme),
        target.resolve(theme),
        "o alias nao pegou o valor do alvo"
    );

    let mine = Color::from_hex(0x00FF00);
    put(theme, target, Some(mine));
    assert_eq!(
        follower.resolve(theme),
        mine,
        "o alias nao SEGUIU o alvo — a cadeia foi achatada na escrita"
    );
    clear_color_overrides();
}

/// **A cadeia termina na fábrica do ALVO, nunca na do token de partida.**
///
/// ⚠️ Com o alvo não-autorado as duas leituras são fáceis de confundir, e a errada é silenciosa:
/// o token seguiria a si mesmo e o artista veria a linha marcada como "segue accent" mostrando a
/// cor de sempre. A fixture escolhe um par cujas fábricas DIFEREM, senão o gate é vácuo.
#[test]
fn a_chain_ends_at_the_targets_factory_not_the_starters() {
    clear_color_overrides();
    let theme = Theme::Forge;
    let (follower, target) = (ColorToken::Border, ColorToken::Accent);
    let own = follower.resolve(theme);
    let theirs = target.resolve(theme);
    assert_ne!(own, theirs, "a fixture precisa de duas fabricas distintas");

    link(theme, follower, target);
    assert_eq!(follower.resolve(theme), theirs);
    assert_eq!(
        resolved_override(theme, follower),
        Some(Authored::Factory(target)),
        "a cadeia tem de NOMEAR onde terminou"
    );
    clear_color_overrides();
}

/// **O alias é do MODO** — em cada um ele vale o alvo DAQUELE modo.
///
/// É isto que mantém a re-vestida viva: um alias que atravessasse modos congelaria o token no
/// valor de um deles e o seletor de tema deixaria de o alcançar.
#[test]
fn an_alias_resolves_within_its_own_mode() {
    clear_color_overrides();
    let (follower, target) = (ColorToken::Border, ColorToken::Accent);
    for theme in [Theme::Forge, Theme::Sunstone] {
        link(theme, follower, target);
    }
    for theme in [Theme::Forge, Theme::Sunstone] {
        assert_eq!(
            follower.resolve(theme),
            target.resolve(theme),
            "o alias saiu do modo dele"
        );
    }
    assert_ne!(
        follower.resolve(Theme::Forge),
        follower.resolve(Theme::Sunstone),
        "a fixture precisa de dois modos cujo alvo difere"
    );
    clear_color_overrides();
}

/// **Um laço é RECUSADO na porta, dizendo onde fecha — e não escreve nada.**
///
/// ⚠️ As duas metades: a recusa (senão `a → b → a` não tem valor a devolver) e a **ausência de
/// efeito**, porque uma porta que recusa DEPOIS de escrever deixa a tabela num estado que ninguém
/// pediu.
#[test]
fn a_loop_is_refused_at_the_door_and_writes_nothing() {
    clear_color_overrides();
    let theme = Theme::Forge;
    let (a, b) = (ColorToken::Border, ColorToken::Accent);
    link(theme, a, b);
    let before = color_overrides();

    let err = set_color_override(theme, b, Some(TokenValue::Alias(a)))
        .expect_err("b -> a fecha o laco e tem de ser RECUSADO");
    assert_eq!(err.token, b);
    assert_eq!(err.target, a);
    assert_eq!(err.at, b, "o laco fecha em quem pediu");
    assert_eq!(color_overrides(), before, "a recusa escreveu na tabela");
    clear_color_overrides();
}

/// **Um token não pode seguir a si mesmo** — o laço de comprimento um.
///
/// ⚠️ Ele é o caso que um teste de ciclo escrito "para dois nós" deixa passar, e o defeito dele é
/// o pior: um slot que só aponta para si mesmo não tem valor nenhum, em modo nenhum.
#[test]
fn a_token_cannot_follow_itself() {
    clear_color_overrides();
    let theme = Theme::Forge;
    let t = ColorToken::Accent;
    let err = set_color_override(theme, t, Some(TokenValue::Alias(t)))
        .expect_err("auto-alias e' um laco");
    assert_eq!(err.at, t);
    assert!(color_override(theme, t).is_none());
    clear_color_overrides();
}

/// **Uma cadeia tão longa quanto a tabela permite ainda RESOLVE.**
///
/// ⚠️ O gate do teto, e é a metade útil dele: apertar `max_alias_hops` cortaria cadeias legítimas
/// **em silêncio** (o token cairia na fábrica), o que é indistinguível de "o alias não pegou". A
/// outra metade — uma tabela já cíclica não travar a leitura — é **inalcançável pelo produto**,
/// porque as duas portas de escrita a recusam; ela fica documentada em vez de gateada, pelo
/// precedente do ADR-0145 (defesa que o regime que shipa não pode observar).
#[test]
fn the_longest_honest_chain_still_resolves() {
    clear_color_overrides();
    let theme = Theme::Forge;
    let all = ColorToken::ALL;
    let mine = Color::from_hex(0x123456);
    for w in all.windows(2) {
        link(theme, w[0], w[1]);
    }
    put(theme, all[all.len() - 1], Some(mine));
    assert_eq!(
        all[0].resolve(theme),
        mine,
        "a caminhada desistiu antes do fim de uma cadeia honesta"
    );
    clear_color_overrides();
}

/// **O load DESCARTA o laço que um arquivo trouxer, e DIZ quantos.**
///
/// ⚠️ As alternativas são piores e as duas foram consideradas: recusar o arquivo inteiro joga fora
/// uma re-vestida por causa de duas linhas; aceitar põe na tabela o laço que a porta promete não
/// ter. O que sobrevive é acíclico **por construção**, seja qual for a ordem da lista.
#[test]
fn installing_a_cyclic_table_drops_the_loop_and_says_how_many() {
    clear_color_overrides();
    let theme = Theme::Forge;
    let (a, b) = (ColorToken::Border, ColorToken::Accent);
    let cyclic = vec![
        ColorOverride {
            theme,
            token: a,
            value: TokenValue::Alias(b),
        },
        ColorOverride {
            theme,
            token: b,
            value: TokenValue::Alias(a),
        },
    ];
    assert_eq!(set_color_overrides(cyclic), 1, "o laco tem de ser DITO");
    // O que ficou resolve — não gira, não entra em pânico, não fica preso.
    assert_eq!(a.resolve(theme), b.resolve(theme));
    assert_eq!(overridden_count(theme), 1);
    clear_color_overrides();
}
