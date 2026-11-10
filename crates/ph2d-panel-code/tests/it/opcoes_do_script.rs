//! Plano 14 Q4 passo D: o cartão «Script options» — uma linha por opção (título, caixa de marcar, frase),
//! a linha inteira é o alvo, a caixa diz o estado, o alfinete troca-se no carregar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::options_card::{OptionRow, OptionsCard, OptionsFrame, OptionsPainted};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const ABA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 700.0,
};

const ROWS: [OptionRow<'static>; 3] = [
    OptionRow {
        cmd: NodeId(1),
        title: "Strict types",
        says: "Adds --!strict on line 1",
        on: true,
    },
    OptionRow {
        cmd: NodeId(2),
        title: "Format on save",
        says: "Tidy spacing",
        on: false,
    },
    OptionRow {
        cmd: NodeId(3),
        title: "Type hints",
        says: "Show in grey",
        on: true,
    },
];

fn pinta(card: &mut OptionsCard) -> OptionsPainted {
    let f = OptionsFrame {
        id: NodeId(9),
        script: "player.luau",
        rows: &ROWS,
        assist: None,
        restart: NodeId(4),
    };
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    card.paint(ABA, &f, &mut scene, &mut ts, &Theme::Forge);
    card.painted.clone()
}

fn aberto() -> (OptionsCard, OptionsPainted) {
    let mut card = OptionsCard {
        open: true,
        ..OptionsCard::default()
    };
    let p = pinta(&mut card);
    (card, p)
}

fn dentro(fora: Rect, r: Rect) -> bool {
    r.x >= fora.x && r.y >= fora.y && r.x + r.w <= fora.x + fora.w && r.y + r.h <= fora.y + fora.h
}

/// Cada opção tem uma caixa que mostra o estado dela e uma frase por baixo, ambos dentro da linha.
#[test]
fn cada_opcao_tem_caixa_com_o_estado_e_frase_por_baixo() {
    let (_, p) = aberto();
    assert_eq!(
        p.on,
        ROWS.map(|o| o.on).to_vec(),
        "a caixa reflecte o estado"
    );
    assert_eq!((p.marks.len(), p.says.len()), (ROWS.len(), ROWS.len()));
    for i in 0..ROWS.len() {
        let (row, mark, says) = (p.rows[i].1, p.marks[i], p.says[i]);
        assert!(
            dentro(row, mark) && dentro(row, says),
            "{row:?} {mark:?} {says:?}"
        );
        assert!(
            says.y > mark.y,
            "a frase fica por baixo do título e da caixa"
        );
    }
}

/// A linha INTEIRA é o alvo: carregar na frase, longe da caixa, dá o comando da opção; a mão
/// sobre ela; largar apaga a moldura de clique.
#[test]
fn a_linha_inteira_da_opcao_e_o_alvo_com_mao_e_moldura() {
    let (mut card, p) = aberto();
    let says = p.says[1];
    let (x, y) = (says.x + 1.0, says.y + says.h * 0.5);
    assert!(card.hover(x, y), "o hover mudou");
    assert_eq!(card.hover, Some(NodeId(2)));
    assert_eq!(card.look(x, y), Some(PointerLook::Hand));
    assert_eq!(card.press(x, y), Some(NodeId(2)));
    assert_eq!(card.pressed, Some(NodeId(2)), "a moldura de clique");
    assert!(card.release());
    assert_eq!(card.pressed, None);
    let card_r = p.card.expect("o cartão");
    assert_eq!(
        card.look(card_r.x + 1.0, card_r.y + 1.0),
        Some(PointerLook::Arrow)
    );
    assert_eq!(card.look(card_r.x - 1.0, card_r.y), None);
}

/// Dono 10/11: «o padrão do app é caixa, não interruptor» — cada opção leva a caixa de marcar do
/// cartão (o tamanho dela, quadrada), dentro da linha, encostada à direita dela e centrada na altura;
/// `on` espelha o estado.
#[test]
fn cada_opcao_leva_a_caixa_de_marcar_do_cartao_a_direita_e_centrada() {
    let (_, p) = aberto();
    let caixa = ph2d_editor_core::widget::card_check_rect(0.0, 0.0, 0.0);
    assert_eq!(
        p.on,
        ROWS.map(|o| o.on).to_vec(),
        "a caixa espelha o estado"
    );
    let restart = p.rows.last().expect("o Restart").1;
    for (i, &m) in p.marks.iter().enumerate() {
        let row = p.rows[i].1;
        assert_eq!(
            (m.w, m.h),
            (caixa.w, caixa.h),
            "opção {i}: a caixa do cartão, {m:?}"
        );
        assert!(dentro(row, m), "opção {i}: {m:?} fora de {row:?}");
        let (fim, meio) = (m.x + m.w, row.y + row.h * 0.5);
        assert!(
            (fim - (restart.x + restart.w)).abs() < 0.5,
            "opção {i}: à direita"
        );
        assert!(m.x > row.x + row.w * 0.5, "opção {i}: na metade direita");
        assert!((m.y + m.h * 0.5 - meio).abs() < 0.5, "opção {i}: centrada");
    }
}

/// Dono 09/11: a frase nunca se corta — num cartão estreito ela quebra, e a caixa dela guarda o texto
/// todo dentro da linha; num largo fica numa linha só.
#[test]
fn a_frase_da_opcao_quebra_e_nunca_se_corta() {
    let longa = "Tidy spacing, indents and line breaks when you press Ctrl+S";
    let rows = [OptionRow {
        says: longa,
        ..ROWS[1]
    }];
    use ph2d_panel_code::peso::CORPO;
    let mut ts = TextSystem::without_system_fonts();
    for (largura, quebra) in [(900.0, false), (240.0, true)] {
        let mut card = OptionsCard {
            open: true,
            ..OptionsCard::default()
        };
        let f = OptionsFrame {
            id: NodeId(9),
            script: "player.luau",
            rows: &rows,
            assist: None,
            restart: NodeId(4),
        };
        let under = Rect { w: largura, ..ABA };
        card.paint(under, &f, &mut VectorScene::new(), &mut ts, &Theme::Forge);
        let p = &card.painted;
        let says = p.says[0];
        let px = ph2d_tokens::TypeToken::Xs.px();
        let uma = ts
            .layout_with_weight(longa, px, f32::INFINITY, CORPO)
            .height();
        let toda = ts.layout_with_weight(longa, px, says.w, CORPO).height();
        assert!(
            says.h + 0.5 >= toda,
            "{largura}: a caixa {says:?} corta o texto ({toda})"
        );
        assert_eq!(
            says.h > uma * 1.5,
            quebra,
            "{largura}: {says:?}, uma linha = {uma}"
        );
        assert!(dentro(p.rows[0].1, says) && dentro(p.card.unwrap(), says));
    }
}

/// Dono 09/11: o hover é UM fundo na linha inteira — nenhuma moldura, a caixa não se mexe: pintar o
/// cartão com o rato numa linha acrescenta um só caminho à cena; carregado, outro só (mais forte).
#[test]
fn o_hover_e_um_fundo_unico_na_linha_e_a_caixa_nao_se_mexe() {
    let caminhos = |card: &mut OptionsCard| {
        let f = OptionsFrame {
            id: NodeId(9),
            script: "player.luau",
            rows: &ROWS,
            assist: None,
            restart: NodeId(4),
        };
        let mut ts = TextSystem::without_system_fonts();
        let mut scene = VectorScene::new();
        card.paint(ABA, &f, &mut scene, &mut ts, &Theme::Forge);
        scene.inner().encoding().n_paths
    };
    let (mut card, p) = aberto();
    let quieto = caminhos(&mut card);
    assert_eq!(card.painted.highlight, None);
    card.hover = Some(NodeId(2));
    assert_eq!(caminhos(&mut card), quieto + 1, "hover = um caminho a mais");
    let (r, ink) = card.painted.highlight.expect("o hover pintou");
    assert_eq!(r, p.rows[1].1, "o fundo é a linha inteira");
    assert_eq!(card.painted.marks, p.marks, "a caixa não se mexe");
    assert!((ink.components[3] - ph2d_tokens::HOVER_LAYER_ALPHA).abs() < 1e-3);
    card.pressed = Some(NodeId(2));
    assert_eq!(
        caminhos(&mut card),
        quieto + 1,
        "carregado = um caminho a mais"
    );
    assert_eq!(card.painted.highlight.map(|h| h.0), Some(p.rows[1].1));
}

/// O fundo que o popover pinta: a lista suspensa dos cartões, ou o `BgElev` no modo de antes.
fn superficie(theme: Theme, cartoes: bool) -> ph2d_vector::Color {
    use ph2d_editor_core::widget::card_ink::CardInk;
    use ph2d_tokens::ColorToken;
    if cartoes {
        CardInk::of_list(theme).color(ColorToken::KanbanCardRaised)
    } else {
        ph2d_editor_core::paint::resolve(ColorToken::BgElev, theme)
    }
}

fn luz(c: ph2d_vector::Color) -> f64 {
    let [r, g, b, _] = c.to_rgba8().to_u8_array();
    ph2d_tokens::Color { r, g, b, a: 255 }.relative_luminance()
}

/// Pinta o cartão aberto em `theme`, com a tinta de um cartão de tema próprio publicada à volta (o
/// cartão do código onde o popover se abre) e a linha 2 debaixo do rato ou carregada.
fn pinta_sobre_cartao(theme: Theme, cartoes: bool, carregada: bool) -> OptionsPainted {
    use ph2d_editor_core::widget::card_ink::{CardInk, com};
    ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread(Some(cartoes));
    let mut card = OptionsCard {
        open: true,
        hover: Some(NodeId(2)),
        pressed: carregada.then_some(NodeId(2)),
        ..OptionsCard::default()
    };
    let f = OptionsFrame {
        id: NodeId(9),
        script: "player.luau",
        rows: &ROWS,
        assist: None,
        restart: NodeId(4),
    };
    let outro = if theme == Theme::Light {
        Theme::Forge
    } else {
        Theme::Light
    };
    let (mut ts, mut scene) = (TextSystem::without_system_fonts(), VectorScene::new());
    com(CardInk::of_card(Some(outro), theme), || {
        card.paint(ABA, &f, &mut scene, &mut ts, &theme);
    });
    ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread(None);
    card.painted
}

/// A caixa e o realce tomam a tinta da SUPERFÍCIE do popover, nunca a do cartão aberto à volta dele:
/// nos dois modos e num tema escuro e num claro, o hover clareia o fundo escuro e escurece o claro, e
/// o carregado é o `AccentSoft` dessa mesma superfície.
#[test]
fn a_caixa_e_o_realce_tomam_a_tinta_da_superficie_do_popover() {
    use ph2d_editor_core::widget::card_ink::CardInk;
    use ph2d_editor_core::widget::lista_flutuante::Superficie;
    for cartoes in [true, false] {
        for theme in [Theme::Forge, Theme::Light] {
            let p = pinta_sobre_cartao(theme, cartoes, false);
            assert_eq!(
                p.check_ink,
                Some(CardInk::of_list(theme)),
                "{theme:?} {cartoes}"
            );
            let (_, hover) = p.highlight.expect("o hover pintou");
            let fundo = superficie(theme, cartoes);
            let a = hover.components[3];
            let sobre = |f: f32, h: f32| f * (1.0 - a) + h * a;
            let [fr, fg, fb, _] = fundo.components;
            let [hr, hg, hb, _] = hover.components;
            let mistura =
                ph2d_vector::Color::new([sobre(fr, hr), sobre(fg, hg), sobre(fb, hb), 1.0]);
            let escuro = luz(fundo) < 0.5;
            assert_eq!(
                luz(mistura) > luz(fundo),
                escuro,
                "{theme:?} cartões={cartoes}: o hover clareia o escuro e escurece o claro"
            );
            let p = pinta_sobre_cartao(theme, cartoes, true);
            ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread(Some(cartoes));
            let quer = Superficie::de(theme).cor(ph2d_tokens::ColorToken::AccentSoft);
            ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread(None);
            assert_eq!(
                p.highlight.map(|h| h.1),
                Some(quer),
                "{theme:?} {cartoes}: carregado"
            );
        }
    }
}

/// O alfinete: carregar nele segura o cartão (um clique fora já não o fecha) e não corre comando.
#[test]
fn o_alfinete_do_cartao_de_opcoes_o_segura_aberto() {
    let (mut card, p) = aberto();
    let pin = p.pin.expect("o alfinete");
    let card_r = p.card.expect("o cartão");
    assert!(dentro(card_r, pin));
    assert_eq!(
        card.look(pin.x + pin.w * 0.5, pin.y + pin.h * 0.5),
        Some(PointerLook::Hand)
    );
    assert_eq!(card.press(pin.x + pin.w * 0.5, pin.y + pin.h * 0.5), None);
    assert!(card.pinned);
    card.click_outside();
    assert!(card.open, "com alfinete fica");
    card.pinned = false;
    card.click_outside();
    assert!(!card.open, "sem alfinete fecha");
}

/// Num corpo baixo o cartão corta-se: a linha que ficou de fora (e o «Restart») não toma o clique
/// nem a mão por baixo do cartão.
#[test]
fn num_corpo_baixo_as_linhas_cortadas_nao_tomam_o_clique() {
    let mut card = OptionsCard {
        open: true,
        ..OptionsCard::default()
    };
    let f = OptionsFrame {
        id: NodeId(9),
        script: "player.luau",
        rows: &ROWS,
        assist: None,
        restart: NodeId(4),
    };
    let mut ts = TextSystem::without_system_fonts();
    let baixo = Rect::new(0.0, 0.0, ABA.w, 120.0);
    card.paint(baixo, &f, &mut VectorScene::new(), &mut ts, &Theme::Forge);
    let p = card.painted.clone();
    let c = p.card.expect("o cartão");
    let fora = p
        .rows
        .iter()
        .find(|(_, r)| r.y + r.h * 0.5 > c.y + c.h)
        .copied()
        .expect("controlo: uma linha cortada abaixo do cartão");
    let (x, y) = (fora.1.x + fora.1.w * 0.5, fora.1.y + fora.1.h * 0.5);
    assert!(!c.contains(x, y), "controlo: o ponto está fora do cartão");
    assert_eq!(card.press(x, y), None, "{fora:?}");
    assert_eq!(card.pressed, None);
    assert_eq!(card.look(x, y), None);
}

/// Com a letra Large e a faixa de cima como a linha mais larga, o nome do script cabe inteiro: o
/// cartão mede o título no peso em que se pinta (o `600`).
#[test]
fn com_a_letra_grande_o_nome_do_script_na_faixa_de_cima_nao_se_corta() {
    use ph2d_tokens::{TextRendering, UiTextSize, UiTextStyle};
    let antes = ph2d_text::active_text_style();
    ph2d_text::set_active_text_style(UiTextStyle {
        size: UiTextSize::Large,
        ..antes
    });
    ph2d_editor_core::paint::set_text_rendering(TextRendering::CrispHeavyPlus);
    let script = "the_main_character_movement_and_collisions_controller.luau";
    let f = OptionsFrame {
        id: NodeId(9),
        script,
        rows: &ROWS,
        assist: None,
        restart: NodeId(4),
    };
    let mut ts = TextSystem::without_system_fonts();
    let mut card = OptionsCard {
        open: true,
        ..OptionsCard::default()
    };
    card.paint(ABA, &f, &mut VectorScene::new(), &mut ts, &Theme::Forge);
    let font = ph2d_tokens::TypeToken::Sm.px();
    let inteiro =
        ph2d_panel_code::peso::largura_em(&mut ts, script, font, ph2d_panel_code::peso::CORPO);
    ph2d_editor_core::paint::set_text_rendering(TextRendering::Default);
    ph2d_text::set_active_text_style(antes);
    let name = card.painted.name.expect("o nome pintado");
    let cartao = card.painted.card.expect("o cartão");
    assert!(
        cartao.w < ABA.w,
        "o cartão não bateu na aba (a faixa é que manda)"
    );
    assert!(
        name.w >= inteiro - 0.5,
        "o nome coube em {} de {inteiro}: cortado",
        name.w
    );
}
