//! R11a: a prévia da aba é a tela REAL do jogador — o painel inteiro no tamanho natural (todas as linhas, o «Reset to
//! defaults» e a dica, sem indicador de rolagem), escalado para caber na tela emoldurada e centrado nela.

use ph2d_editor_core::zones::Rect;
use ph2d_input::InputMap;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::{Layout, paint_preview, rows};

/// O céu da tela emoldurada da C4 (a 1600 × 1000: 16 linhas de 28 px de largo, 9/16 disso menos o chão).
const SKY: Rect = Rect::new(40.0, 30.0, 448.0, 224.0); // LITERAL-PX-OK: o céu medido da foto C4

/// Todas as acções na tela do jogador, para o painel passar da altura do céu.
fn every_row() -> Vec<crate::Row> {
    let mut map = InputMap::with_player_defaults();
    let ids: Vec<_> = map.actions().iter().map(|a| a.id).collect();
    for id in ids {
        map.controls_mut().set_shown_in_game(id, true);
    }
    rows(&map)
}

#[test]
fn a_previa_mostra_o_painel_inteiro_escalado_para_caber_no_ceu() {
    let r = every_row();
    let mut ts = TextSystem::new();
    let real = Layout::of(SKY, &r, &mut ts);
    assert!(
        real.bar.is_some() && real.rows.len() < r.len(),
        "a este tamanho a tela de verdade rolaria — é por isso que a prévia escala ({} de {})",
        real.rows.len(),
        r.len()
    );
    let nat = Layout::natural(0.0, 0.0, &r, &mut ts);
    assert_eq!(nat.rows.len(), r.len(), "todas as linhas");
    assert!(nat.bar.is_none(), "sem indicador de rolagem");
    assert!(
        nat.hint.y + nat.hint.h <= nat.panel.y + nat.panel.h,
        "a dica dentro do painel"
    );
    let mut scene = VectorScene::new();
    let at = paint_preview(SKY, &r, &mut scene, &mut ts, Theme::default());
    assert!(
        at.h < nat.panel.h,
        "escalado para baixo: {at:?} × {:?}",
        nat.panel
    );
    assert!(
        at.x >= SKY.x
            && at.y >= SKY.y
            && at.x + at.w <= SKY.x + SKY.w
            && at.y + at.h <= SKY.y + SKY.h,
        "nada cortado: {at:?} dentro de {SKY:?}"
    );
    let aspect = |r: Rect| r.w / r.h;
    assert!(
        (aspect(at) - aspect(nat.panel)).abs() < 1e-3,
        "a mesma proporção: {at:?} × {:?}",
        nat.panel
    );
    let mid = |a: f32, b: f32| a + b * 0.5;
    assert!(
        (mid(at.x, at.w) - mid(SKY.x, SKY.w)).abs() < 0.5
            && (mid(at.y, at.h) - mid(SKY.y, SKY.h)).abs() < 0.5,
        "centrado no céu"
    );
}

#[test]
fn a_previa_nunca_aumenta_o_painel() {
    let r = every_row();
    let mut ts = TextSystem::new();
    let big = Rect::new(0.0, 0.0, 1600.0, 1000.0); // LITERAL-PX-OK: a janela do smoke
    let nat = Layout::natural(0.0, 0.0, &r, &mut ts);
    let at = paint_preview(big, &r, &mut VectorScene::new(), &mut ts, Theme::default());
    assert!(
        (at.w - nat.panel.w).abs() < 1e-3 && (at.h - nat.panel.h).abs() < 1e-3,
        "num céu grande fica no tamanho natural: {at:?} × {:?}",
        nat.panel
    );
}

/// R13 · foto P1: a reserva da dica maior (para a caixa não saltar ao escutar) ficava como faixa vazia por baixo
/// da última linha. Fechada e à escuta, o espaço por cima do título é o MESMO que por baixo da última linha da dica,
/// e a caixa não muda.
#[test]
fn o_painel_tem_o_mesmo_espaco_em_cima_e_em_baixo_e_nao_salta_ao_escutar() {
    use crate::ControlsScreen;
    use crate::layout::{FOOT_PX, TITLE_PX};
    use ph2d_input::{Key, PlayerKeys};
    let mut g = InputMap::with_player_defaults();
    g.controls_mut().players_can_change_keys = true;
    let r = rows(&g);
    let mut ts = TextSystem::new();
    let area = Rect::new(0.0, 0.0, 1280.0, 720.0); // LITERAL-PX-OK: a janela do plano §8
    let mut listening = ControlsScreen::default();
    let mut keys = PlayerKeys::new();
    listening.handle_key(Key(0xF704), true, &g, &mut keys); // F1 abre
    listening.handle_key(Key(0x0D), true, &g, &mut keys); // Enter escuta
    assert!(listening.listening(), "Enter escuta");
    let rest = Layout::scrolled(area, &r, &ControlsScreen::default(), &mut ts);
    let heard = Layout::scrolled(area, &r, &listening, &mut ts);
    assert_eq!(rest.panel, heard.panel, "a caixa não salta ao escutar");
    for (at, l) in [("fechada", &rest), ("à escuta", &heard)] {
        let above = l.title.y + (l.title.h - TITLE_PX) * 0.5 - l.panel.y;
        let last = l.hint.y + l.hint.h - l.reset.h * 0.5 + FOOT_PX * 0.5;
        let below = l.panel.y + l.panel.h - last;
        assert!(
            (above - below).abs() < 0.5,
            "{at}: {above} por cima do título × {below} por baixo da dica"
        );
    }
}
