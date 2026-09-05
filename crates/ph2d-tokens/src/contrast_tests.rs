//! Gates da **lei de contraste** (plano UI/UX W4b).
//!
//! ⚠️ O keystone não é nenhum dos números: é o gate que prova que a checagem de COMPILAÇÃO é cega
//! ao valor que o artista autora. Sem ele, esta wave inteira parece redundante com os quatro
//! testes que ela substituiu.

use super::*;
use crate::color::Color;
use crate::overrides::{TokenValue, clear_color_overrides, set_color_override};

fn put(theme: Theme, token: ColorToken, colour: Color) {
    set_color_override(theme, token, Some(TokenValue::Literal(colour)))
        .expect("um literal nunca fecha um laco");
}

const ALL_THEMES: [Theme; 8] = [
    Theme::Forge,
    Theme::Workshop,
    Theme::Sunstone,
    Theme::Blueprint,
    // ⭐ A família moderna entra na mesma régua: um tema DERIVADO que não cumpra a WCAG é um
    //    defeito da derivação, e é aqui que ele reprova.
    Theme::Dark,
    Theme::Gray,
    Theme::Light,
    Theme::Oled,
];

/// **A tabela de FÁBRICA cumpre a WCAG nos quatro modos** — o gate que as quatro cópias eram.
///
/// ⚠️ Ele agora percorre a LISTA, então um par novo nasce gateado sem ninguém escrever um quinto
/// laço. A mutação que o mata é esvaziar a tabela: com zero pares ele fica verde por vácuo, e é
/// por isso que o gate ao lado conta quantos pares existem.
#[test]
fn the_factory_table_meets_wcag_in_all_modes() {
    clear_color_overrides();
    for theme in ALL_THEMES {
        for p in CONTRAST_PAIRS {
            let r = p.ratio(theme);
            assert!(
                p.passes(theme),
                "{theme:?}: {} sobre {} = {r:.2}:1, precisa de >= {} ({})",
                p.fg.key(),
                p.bg.key(),
                p.min_ratio,
                p.criterion
            );
        }
    }
}

/// **A lista não pode encolher em silêncio.**
///
/// ⚠️ Um gate que percorre uma lista fica VERDE quando a lista esvazia — é o verde por vácuo, e
/// aqui ele apagaria quatro promessas de acessibilidade sem uma linha vermelha. Os quatro pares
/// que este arquivo herdou das quatro cópias antigas são nomeados um a um.
#[test]
fn the_table_still_carries_the_pairs_it_replaced() {
    let has = |fg: ColorToken, bg: ColorToken, min: f64| {
        CONTRAST_PAIRS
            .iter()
            .any(|p| p.fg == fg && p.bg == bg && (p.min_ratio - min).abs() < f64::EPSILON)
    };
    assert!(has(ColorToken::Text1, ColorToken::Bg1, 4.5), "text-1 caiu");
    assert!(has(ColorToken::Text2, ColorToken::Bg1, 4.5), "text-2 caiu");
    assert!(
        has(ColorToken::BorderEmph, ColorToken::Bg1, 3.0),
        "border-emph caiu"
    );
    assert!(has(ColorToken::Accent, ColorToken::Bg1, 3.0), "accent caiu");
}

/// **O KEYSTONE: a checagem de COMPILAÇÃO é cega ao valor autorado.**
///
/// ⚠️ Este gate não mede o produto — ele mede o **alcance** do gate irmão, e é a razão inteira de
/// existir um readout. Um teste de unidade corre com a camada vazia, então ele afirma a tabela de
/// FÁBRICA; a cor que o artista escolhe move o valor efetivo em runtime, onde nenhum teste está a
/// olhar. Se ele um dia falhar, o readout tornou-se redundante e esta wave pode ser retirada.
#[test]
fn the_compile_time_check_cannot_see_an_authored_break() {
    clear_color_overrides();
    let theme = Theme::Forge;
    // Um cinzento que quase iguala o fundo — texto ilegível, e nenhuma constante mudou.
    let bg = ColorToken::Bg1.resolve(theme);
    put(theme, ColorToken::Text1, bg);

    // A tabela GERADA continua conforme: é isso que o gate de compilação mede.
    let factory_ratio = {
        let saved = crate::overrides::color_overrides();
        clear_color_overrides();
        let r = CONTRAST_PAIRS[0].ratio(theme);
        assert_eq!(crate::overrides::set_color_overrides(saved), 0);
        r
    };
    assert!(
        factory_ratio >= 4.5,
        "a fabrica ja' estava fora de conformidade — a fixture nao prova nada"
    );

    // E o mundo que o artista está a ver, não.
    assert!(
        !CONTRAST_PAIRS[0].passes(theme),
        "o par mediu a FABRICA — o readout descreveria um app que ninguem esta' a ver"
    );
    clear_color_overrides();
}

/// **Um valor autorado que quebra a WCAG é REPORTADO.**
#[test]
fn an_authored_break_is_reported() {
    clear_color_overrides();
    let theme = Theme::Forge;
    assert!(
        failing_pairs(theme).is_empty(),
        "o CONTROLE falhou: a fabrica ja' reporta problemas"
    );

    put(theme, ColorToken::Text1, ColorToken::Bg1.resolve(theme));
    let failing = failing_pairs(theme);
    assert_eq!(failing.len(), 1, "esperava exactamente um par a falhar");
    assert_eq!(failing[0].fg, ColorToken::Text1);
    clear_color_overrides();
}

/// **Um ALIAS que herda uma cor ilegível também é reportado.**
///
/// ⚠️ Isto sai de graça — o readout mede pelo `resolve`, que segue a cadeia — e é por isso que
/// tem gate: a alternativa (ler o SLOT) pareceria funcionar em todo teste de literal e ficaria
/// cega exactamente na feature que a W4b.1 acabou de shipar.
#[test]
fn a_link_that_inherits_an_unreadable_colour_is_reported() {
    clear_color_overrides();
    let theme = Theme::Forge;
    // `bg-2` recebe a cor do fundo, e `text-1` passa a SEGUIR `bg-2`.
    put(theme, ColorToken::Bg2, ColorToken::Bg1.resolve(theme));
    set_color_override(
        theme,
        ColorToken::Text1,
        Some(TokenValue::Alias(ColorToken::Bg2)),
    )
    .expect("a fixture nao fecha laco");

    assert!(
        failing_pairs(theme)
            .iter()
            .any(|p| p.fg == ColorToken::Text1),
        "o par nao viu a cor que o token HERDOU"
    );
    clear_color_overrides();
}

/// **A marca da linha aparece nos DOIS lados do par, e só neles.**
///
/// ⚠️ O par é uma relação: escurecer o FUNDO quebra a legibilidade do texto, e marcar só o texto
/// mandaria o artista consertar o token que ele não mexeu.
#[test]
fn both_sides_of_a_failing_pair_are_marked() {
    clear_color_overrides();
    let theme = Theme::Forge;
    put(theme, ColorToken::Text1, ColorToken::Bg1.resolve(theme));

    assert!(token_is_in_a_failing_pair(theme, ColorToken::Text1));
    assert!(
        token_is_in_a_failing_pair(theme, ColorToken::Bg1),
        "o FUNDO do par nao foi marcado — ele e' metade da relacao"
    );
    assert!(
        !token_is_in_a_failing_pair(theme, ColorToken::Danger),
        "um token fora de qualquer par foi marcado"
    );
    clear_color_overrides();
}

/// **O readout é do MODO VIGENTE** — quebrar o Forge não acusa o Sunstone.
///
/// A mesma decisão de escopo do *Reset This Mode*: um override é do par `(modo, token)`, e um
/// aviso sobre uma tela que o artista não está a ver é um aviso que ele aprende a ignorar.
#[test]
fn the_readout_is_per_mode() {
    clear_color_overrides();
    put(
        Theme::Forge,
        ColorToken::Text1,
        ColorToken::Bg1.resolve(Theme::Forge),
    );
    assert_eq!(failing_pairs(Theme::Forge).len(), 1);
    assert!(
        failing_pairs(Theme::Sunstone).is_empty(),
        "o aviso vazou para um modo que ninguem tocou"
    );
    clear_color_overrides();
}

/// **Sem nada autorado, o readout fica CALADO** — o controle.
///
/// ⚠️ Sem esta metade, um readout que reportasse tudo sempre passaria em todos os gates acima.
#[test]
fn a_factory_table_reports_nothing() {
    clear_color_overrides();
    for theme in ALL_THEMES {
        assert!(
            failing_pairs(theme).is_empty(),
            "{theme:?}: a fabrica reportou um problema"
        );
        for &token in ColorToken::ALL {
            assert!(
                !token_is_in_a_failing_pair(theme, token),
                "{theme:?}: {} foi marcado numa tabela de fabrica",
                token.key()
            );
        }
    }
}
