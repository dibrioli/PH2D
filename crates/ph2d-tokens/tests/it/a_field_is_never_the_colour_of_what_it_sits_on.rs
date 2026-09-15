//! ⭐⭐⭐ **UM CAMPO NUNCA PINTA A COR DO QUE ESTÁ POR BAIXO DELE.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14:** *«caixas de input numérico sem cor de fundo»*.
//!
//! Medido nesse dia: o `number_input` e a `text_area` enchiam com `ColorToken::Bg1`, que é
//! **exactamente** o token de um cartão de secção — e é sobre cartões que o Inspector põe as
//! linhas dele. A distância era **`0/255` nos oito temas**, e num tema moderno a moldura de
//! repouso é ZERO: *não havia caixa nenhuma*, só o número pousado no cartão.
//!
//! | tema | campo vs painel | vs cartão | vs subcartão | moldura? |
//! |---|---|---|---|---|
//! | Dark (antes) | 12 | **0** | 10 | não |
//! | Gray (antes) | 19 | **0** | 14 | não |
//! | Light (antes) | 13 | **0** | 11 | não |
//! | Dark (hoje) | 10 | 22 | 32 | não |
//! | Gray (hoje) | 10 | 29 | 43 | não |
//! | Light (hoje) | 35 | 22 | 11 | não |
//!
//! # ⚠️ A barra é o degrau que esta casa já mediu, e é a MESMA pergunta
//!
//! [`ph2d_tokens::derive::SURFACE_STEP`] (`0,04` ≈ `10/255`) nasceu para separar o CHÃO do painel
//! e responde aqui a *«duas superfícies que se tocam lêem-se como duas?»* — a mesma pergunta, nos
//! mesmos quatro temas. ⛔ Não é herdar a resposta do par *cartão-contra-painel* (`12/255`), que é
//! a cerca que o `a_card_stands_off_its_panel` planta por escrito.
//!
//! # ⛔ A cláusula da MOLDURA, e porque ela não é uma isenção
//!
//! Onde o tema traça uma moldura de repouso (a família clássica, e o OLED com *Draw Extra
//! Borders*), a caixa lê-se pela borda e a distância de tom pode ser zero — é o que a família
//! clássica sempre fez, e é o que o gate dos cartões já declara para o OLED, cuja base preta
//! colapsa a escada inteira.

use ph2d_tokens::derive::SURFACE_STEP;
use ph2d_tokens::{Color, ColorToken, Theme, visuals::Chrome};

/// A distância entre dois tons, no pior canal, em unidades de `255`.
fn dist(a: Color, b: Color) -> f32 {
    let f = |x: u8, y: u8| (f32::from(x) - f32::from(y)).abs();
    f(a.r, b.r).max(f(a.g, b.g)).max(f(a.b, b.b))
}

/// **As três superfícies em que um campo pode assentar neste app.**
///
/// ⚠️ Elas são NOMEADAS, e não varridas: um cartão novo que alguém invente entra aqui à mão, e é
/// isso que faz o gate falhar em vez de passar a medir menos.
fn surfaces(theme: Theme) -> [(&'static str, Color); 3] {
    [
        ("painel", ColorToken::PanelBg.resolve(theme)),
        ("cartao de seccao", ColorToken::Bg1.resolve(theme)),
        ("cartao de subseccao", ColorToken::Bg2.resolve(theme)),
    ]
}

#[test]
fn a_field_is_never_the_colour_of_what_it_sits_on() {
    let barra = SURFACE_STEP * 255.0;
    let mut medidos = 0;
    let mut sem_moldura = 0;
    for theme in Theme::ALL {
        let chrome = Chrome::of(theme);
        if chrome.field_border.is_visible() {
            // A caixa lê-se pela borda — ver o cabeçalho.
            continue;
        }
        sem_moldura += 1;
        for (nome, sup) in surfaces(theme) {
            let d = dist(chrome.field_fill, sup);
            assert!(
                d + 0.51 >= barra,
                "{theme:?}: um campo sem moldura fica a {d:.0}/255 do {nome} (barra {barra:.0}) — \
                 sem moldura e sem degrau de tom NAO HA CAIXA. Foi este o report do dono em \
                 2026-09-14, com o campo a `0/255` do cartao de seccao."
            );
            medidos += 1;
        }
    }
    // ⚠️ **Piso de população:** um gate que salte todos os temas passa trivialmente. Os temas
    // modernos sem *Draw Extra Borders* são três (`Dark`, `Gray`, `Light`) — o OLED tem-nas.
    assert!(
        sem_moldura >= 3 && medidos >= 9,
        "a varredura mediu {medidos} pares em {sem_moldura} temas sem moldura — \
         a população encolheu por baixo do gate"
    );
}

/// ⚠️ **A metade que a primeira redacção não tinha: o TEXTO ainda se lê no campo.**
///
/// Afundar o fundo de um campo é barato até alguém afundar demais — e a régua do contraste vive
/// noutra crate-folha, então um tom que não é token nenhum **não entra no censo
/// `contrast_tests`**. *Um valor derivado fora da tabela de tokens é invisível às réguas da
/// tabela.*
#[test]
fn the_text_in_a_field_still_reads() {
    for theme in Theme::ALL {
        let fill = Chrome::of(theme).field_fill;
        let texto = ColorToken::Text1.resolve(theme);
        let razao = fill.contrast_ratio(&texto);
        assert!(
            razao >= 4.5,
            "{theme:?}: o texto de um campo lê {razao:.2}:1 sobre o fundo dele (WCAG AA pede 4,5)"
        );
    }
}
