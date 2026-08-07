//! Os gates da POLÍTICA do interop DTCG — a metade que decide, dirigida sem tocar num arquivo.

use super::*;
use ph2d_tokens::num::NumToken;
use ph2d_tokens::num_overrides::{NumValue, num_override, set_num_override};
use ph2d_tokens::overrides::{TokenValue, color_override};
use ph2d_tokens::spacing::Spacing;
use ph2d_tokens::{ColorToken, color::Color};

const RED: Color = Color {
    r: 220,
    g: 30,
    b: 40,
    a: 255,
};
const GREEN: Color = Color {
    r: 10,
    g: 200,
    b: 30,
    a: 255,
};

fn clear() {
    ph2d_tokens::overrides::clear_color_overrides();
    let _ = set_num_overrides(Vec::new());
}

/// Um `Imported` montado à mão — os gates da POLÍTICA não passam pelo parser.
fn imported(colours: Vec<(Theme, ColorToken, Color)>) -> Imported {
    Imported {
        colours: colours
            .into_iter()
            .map(|(theme, token, c)| ph2d_tokens::overrides::ColorOverride {
                theme,
                token,
                value: TokenValue::Literal(c),
            })
            .collect(),
        ..Imported::default()
    }
}

/// ⭐ **O modo vigente é SUBSTITUÍDO, e os outros três FICAM.**
///
/// ⚠️ As duas metades num gate só, porque o modo de falha de cada uma é o oposto do da outra:
/// somar em vez de substituir deixa de pé um token que o arquivo não menciona; substituir demais
/// apaga trabalho num modo que o artista não está a olhar.
#[test]
fn the_current_mode_is_replaced_and_the_other_modes_survive() {
    clear();
    // Antes: uma cor autorada no Forge e outra no Workshop.
    ph2d_tokens::overrides::set_color_override(
        Theme::Forge,
        ColorToken::Accent,
        Some(TokenValue::Literal(RED)),
    )
    .expect("literal");
    ph2d_tokens::overrides::set_color_override(
        Theme::Workshop,
        ColorToken::Accent,
        Some(TokenValue::Literal(RED)),
    )
    .expect("literal");

    // Um import no Forge que menciona OUTRO token.
    let changed = install(
        Theme::Forge,
        imported(vec![(Theme::Forge, ColorToken::BorderStrong, GREEN)]),
    );
    assert!(changed);

    assert_eq!(
        color_override(Theme::Forge, ColorToken::Accent),
        None,
        "o token que o arquivo NAO menciona tem de SAIR — um import e' 'a tabela deste modo passa \
         a ser esta'"
    );
    assert_eq!(
        color_override(Theme::Forge, ColorToken::BorderStrong),
        Some(TokenValue::Literal(GREEN))
    );
    assert_eq!(
        color_override(Theme::Workshop, ColorToken::Accent),
        Some(TokenValue::Literal(RED)),
        "o OUTRO modo nao pode ser tocado"
    );
    clear();
}

/// **As duas famílias são instaladas, mesmo quando o arquivo só traz uma.**
///
/// ⚠️ Pular a família que este arquivo não usa deixaria a escala do arquivo ANTERIOR de pé sob as
/// cores do novo — a mesma classe do load de projeto que não esquece.
/// ⚠️ **As DUAS metades, e a segunda foi achada por uma mutação que SOBREVIVEU:** o gate acima
/// prova *"os outros modos sobrevivem"* só para a família de COR, e apagar o filtro de modo do
/// lado NUMÉRICO passava por ele — duas famílias são duas camadas, e camadas em série querem um
/// gate cada.
#[test]
fn a_colour_only_file_still_clears_the_scale_of_this_mode_and_only_of_this_mode() {
    clear();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Md),
        Some(NumValue::Literal(42.0)),
    )
    .expect("um comprimento");
    set_num_override(
        Theme::Workshop,
        NumToken::Spacing(Spacing::Md),
        Some(NumValue::Literal(7.0)),
    )
    .expect("um comprimento");

    install(
        Theme::Forge,
        imported(vec![(Theme::Forge, ColorToken::Accent, RED)]),
    );

    assert_eq!(
        num_override(Theme::Forge, NumToken::Spacing(Spacing::Md)),
        None,
        "a escala do modo tinha de ser substituida junto"
    );
    assert_eq!(
        num_override(Theme::Workshop, NumToken::Spacing(Spacing::Md)),
        Some(NumValue::Literal(7.0)),
        "a escala do OUTRO modo nao pode ser tocada"
    );
    clear();
}

/// **Um import que não muda nada devolve `false`** — e é isso que impede o título de ficar sujo
/// por o artista ter aberto um arquivo e o fechado.
#[test]
fn an_import_that_changes_nothing_reports_no_change() {
    clear();
    assert!(
        !install(Theme::Forge, Imported::default()),
        "de fabrica para fabrica nao mudou nada"
    );

    install(
        Theme::Forge,
        imported(vec![(Theme::Forge, ColorToken::Accent, RED)]),
    );
    assert!(
        !install(
            Theme::Forge,
            imported(vec![(Theme::Forge, ColorToken::Accent, RED)])
        ),
        "o MESMO arquivo duas vezes nao muda nada na segunda"
    );
    clear();
}

/// **O que o artista lê diz os três fatos, e só os que existem.**
///
/// ⚠️ Uma linha que diz sempre *"0 desconhecidos"* é uma linha que se aprende a não ler.
#[test]
fn the_report_names_each_fact_only_when_it_happened() {
    let plain = report(&Imported {
        colours: vec![ph2d_tokens::overrides::ColorOverride {
            theme: Theme::Forge,
            token: ColorToken::Accent,
            value: TokenValue::Literal(RED),
        }],
        ..Imported::default()
    });
    assert!(plain.contains('1'));
    assert!(!plain.contains("unknown"), "{plain}");
    assert!(!plain.contains("factory"), "{plain}");

    let rich = report(&Imported {
        unknown: 2,
        dropped: 3,
        at_factory: 4,
        ..Imported::default()
    });
    for word in ["unknown", "unusable", "factory"] {
        assert!(rich.contains(word), "{rich} nao diz {word}");
    }
}

/// **O nome proposto carrega o MODO** — um arquivo é de um modo, e quatro arquivos com o mesmo
/// nome na pasta de Downloads são quatro tabelas que ninguém distingue.
#[test]
fn the_proposed_file_name_carries_the_mode() {
    for (theme, want) in [
        (Theme::Forge, "forge"),
        (Theme::Workshop, "workshop"),
        (Theme::Sunstone, "sunstone"),
        (Theme::Blueprint, "blueprint"),
    ] {
        let n = default_name(theme);
        assert!(n.contains(want), "{n} nao nomeia o modo {want}");
        assert!(n.ends_with(".tokens.json"), "{n}");
    }
}
