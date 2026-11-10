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
        let uma = ts.layout(longa, px, f32::INFINITY).height();
        let toda = ts.layout(longa, px, says.w).height();
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
    // A camada do Material 3: translúcida e mais clara que o cartão do Forge (escuro) ⇒ CLAREIA.
    let [r8, g8, b8, _] = ink.to_rgba8().to_u8_array();
    let rgb = ph2d_tokens::Color {
        r: r8,
        g: g8,
        b: b8,
        a: 255,
    };
    let fundo = ph2d_tokens::ColorToken::BgElev.resolve(Theme::Forge);
    assert!((ink.components[3] - ph2d_tokens::HOVER_LAYER_ALPHA).abs() < 1e-3);
    assert!(
        rgb.relative_luminance() > fundo.relative_luminance(),
        "o hover clareia o cartão"
    );
    card.pressed = Some(NodeId(2));
    assert_eq!(
        caminhos(&mut card),
        quieto + 1,
        "carregado = um caminho a mais"
    );
    assert_eq!(card.painted.highlight.map(|h| h.0), Some(p.rows[1].1));
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
