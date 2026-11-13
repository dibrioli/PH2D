//! A geometria da faixa (versão C): as peças não se pisam, em janela larga e estreita.

use super::*;
use crate::screens::hero::menu_bar::MENU_BAR_H;

fn faixa_de(largura: f32) -> (Rect, Faixa) {
    let bar = Rect::new(0.0, 0.0, largura, MENU_BAR_H);
    let mut ts = TextSystem::without_system_fonts();
    (bar, faixa(bar, &mut ts, None))
}

/// «Menu» · abas · campo · jogo, nesta ordem, sem sobreposição e dentro da barra; o campo no
/// centro quando a janela é larga.
#[test]
fn as_pecas_da_faixa_nao_se_sobrepoem() {
    for largura in [1930.0, 1366.0, 1000.0] {
        let (bar, f) = faixa_de(largura);
        assert!(f.menu.w > 0.0 && f.menu.x >= bar.x);
        assert!(!f.lupa, "{largura}: o campo cabe e virou lupa");
        assert!(
            f.docs.0 >= f.menu.x + f.menu.w,
            "{largura}: abas sob o Menu"
        );
        assert!(f.docs.1 <= f.find.x, "{largura}: abas sob o campo");
        assert!(
            f.docs.1 - f.docs.0 >= f.find.w * 0.5,
            "{largura}: abas sem vão"
        );
        let jogo = f.jogo.first().expect("o jogo cabe").2;
        assert!(f.find.x + f.find.w <= jogo.x, "{largura}: campo sob o Play");
        let fim = f.jogo.last().expect("o jogo cabe").2;
        assert!(
            fim.x + fim.w <= bar.x + bar.w,
            "{largura}: o jogo transborda"
        );
        for par in f.jogo.windows(2) {
            assert!(par[0].2.x + par[0].2.w <= par[1].2.x + f32::EPSILON);
        }
    }
    let (bar, f) = faixa_de(1930.0);
    let centro = f.find.x + f.find.w * 0.5;
    assert!(
        (centro - (bar.x + bar.w * 0.5)).abs() < 1.0,
        "o campo é o centro"
    );
}

/// Numa janela estreita (tablet) o campo vira a lupa, antes do jogo, e nada se pisa.
#[test]
fn na_janela_estreita_o_campo_vira_lupa() {
    let (_, f) = faixa_de(430.0);
    assert!(f.lupa);
    assert!(f.find.w > 0.0, "a lupa tem alvo");
    let jogo = f.jogo.first().expect("o jogo cabe a 430").2;
    assert!(f.find.x + f.find.w <= jogo.x);
    assert!(f.find.x >= f.menu.x + f.menu.w);
}

/// O Play da faixa leva o nome: é mais largo que alto, e a seta do palco vem logo a seguir.
#[test]
fn o_play_tem_nome_e_a_seta_ao_lado() {
    let (_, f) = faixa_de(1366.0);
    let ids: Vec<NodeId> = f.jogo.iter().map(|(id, ..)| *id).collect();
    assert_eq!(
        ids,
        [
            ids::TOPBAR_PLAY_BUTTON,
            ids::MENUBAR_PLAY_MENU,
            ids::TOPBAR_PAUSE,
            ids::TOPBAR_RESET,
            ids::TOPBAR_PLAYER_CONTROLS,
        ]
    );
    let play = f.jogo[0].2;
    assert!(play.w > play.h, "o Play com nome é mais largo que um ícone");
}
