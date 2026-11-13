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

/// ⭐⭐ **Onda 7 (dono 13/11: «não bem alinhada»): UMA linha de centro e UMA altura.** Todo controlo da
/// faixa (o «Menu», o campo, a pílula do jogo, o Play, o Pause, o Stop, o «Controls», − □ ×) tem
/// `MENU_BAR_CTL_H_PX` de alto e o centro no centro da faixa, com a borda no pixel inteiro; o
/// segmento (o Play, a aba escolhida) tem o mesmo centro. A aba das colunas tem a altura da fila de
/// ferramentas e começa na borda da marca «PH2D». Antes eram quatro alturas: 22, 24, 28 e 22.
/// A régua de pixels da foto é `docs/BarraDeMenus/ferramentas/regua_faixa.py`.
#[test]
fn todo_controlo_da_faixa_tem_uma_altura_e_um_centro() {
    use ph2d_tokens::{MENU_BAR_CTL_H_PX, TOOL_ROW_CHIP_PX, TOOL_ROW_H_PX};
    for largura in [1930.0, 1366.0] {
        let bar = Rect::new(0.0, 0.0, largura, MENU_BAR_H);
        let mut ts = TextSystem::without_system_fonts();
        let f = faixa(bar, &mut ts, Some(false));
        let meio = bar.y + bar.h * 0.5;
        let mut caixas = vec![("Menu", f.menu), ("campo", f.find)];
        caixas.extend(f.jogo.iter().map(|(_, _, r)| ("jogo", *r)));
        caixas.extend(f.janela.iter().map(|(_, _, r)| ("janela", *r)));
        assert!(caixas.len() >= 9, "controlo: a régua viu o jogo e a janela");
        if let Some(u) = super::super::transport_bar::pilula(&f.jogo) {
            caixas.push(("pílula do jogo", u));
        }
        for (nome, r) in caixas {
            let c = caixa(r);
            assert_eq!(c.h, MENU_BAR_CTL_H_PX, "{largura}: {nome} com outra altura");
            assert!(
                (c.y + c.h * 0.5 - meio).abs() < 1e-3,
                "{largura}: {nome} fora do centro"
            );
            assert_eq!(
                c.y,
                c.y.round(),
                "{largura}: {nome} com a borda a meio pixel"
            );
            let s = segmento(r);
            assert!(
                (s.y + s.h * 0.5 - meio).abs() < 1e-3,
                "{largura}: segmento de {nome}"
            );
        }
    }
    // A aba das colunas: a altura da fila, o segmento do lado do chip, a borda da marca.
    assert_eq!(super::super::slot_tabs::TAB_BAR_H, TOOL_ROW_H_PX);
    let fila = Rect::new(0.0, MENU_BAR_H, 400.0, TOOL_ROW_H_PX);
    let aba = super::super::slot_tabs::tab_rects(fila, &[90.0])[0];
    let mut ts = TextSystem::without_system_fonts();
    let f = faixa(Rect::new(0.0, 0.0, 1930.0, MENU_BAR_H), &mut ts, None);
    assert_eq!(
        aba.x,
        f.menu.x + Spacing::Xs.px(),
        "a aba começa na borda da marca"
    );
    assert!(aba.h >= TOOL_ROW_CHIP_PX);
}
