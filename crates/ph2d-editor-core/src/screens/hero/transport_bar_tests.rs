//! Os botões do jogo: no centro, sem pisar nos vizinhos, e acesos pelo estado da corrida.

use super::*;

const BARRA: Rect = Rect::new(0.0, 0.0, 1000.0, 24.0);

#[test]
fn ficam_no_centro_da_barra() {
    let r = rects(BARRA, 100.0, 800.0);
    assert_eq!(r.len(), 5);
    let inicio = r[0].2.x;
    let fim = r[4].2.x + r[4].2.w;
    assert!(((inicio + fim) * 0.5 - 500.0).abs() < 0.01);
}

/// Mutação: tirar o `clamp` ⇒ os botões caem sobre as abas da direita.
#[test]
fn saem_do_centro_antes_de_pisar_nos_vizinhos() {
    let r = rects(BARRA, 100.0, 520.0);
    assert_eq!(r.len(), 5);
    assert!(r[4].2.x + r[4].2.w <= 520.0);
    assert!(r[0].2.x >= 100.0);
}

#[test]
fn sem_espaco_nao_aparecem() {
    assert!(rects(BARRA, 100.0, 150.0).is_empty());
}

#[test]
fn a_corrida_acende_o_botao_certo() {
    let parado = GameRun::default();
    assert!(BOTOES.iter().all(|(id, _)| !aceso(*id, parado)));
    let janela = GameRun {
        stage: Some(GameStage::Window),
        playing: true,
    };
    assert!(aceso(ids::MENUBAR_RUN_WINDOW, janela));
    assert!(!aceso(ids::MENUBAR_RUN_EDITOR, janela));
    assert!(!aceso(ids::TOPBAR_PAUSE, janela));
    let pausado = GameRun {
        stage: Some(GameStage::Window),
        playing: false,
    };
    assert!(aceso(ids::TOPBAR_PAUSE, pausado));
    assert!(!aceso(ids::MENUBAR_RUN_WINDOW, pausado));
    assert!(
        !BOTOES
            .iter()
            .any(|(id, _)| *id == ids::TOPBAR_RESET && aceso(*id, janela))
    );
}
