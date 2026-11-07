//! Os botões do jogo: no centro, sem pisar nos vizinhos, e acesos pelo estado da corrida.

use super::*;
use crate::interaction::{HitIndex, WidgetStore};

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

/// Sem estes botões, a tela cheia não tinha como parar. Mutação: tirar o Stop da lista ⇒ reprova.
#[test]
fn a_tela_cheia_tem_jogar_pausar_e_parar_no_topo_ao_centro() {
    let r = rects_palco(BARRA);
    let ids: Vec<_> = r.iter().map(|(id, ..)| *id).collect();
    assert_eq!(
        ids,
        [
            ids::MENUBAR_RUN_FULLSCREEN,
            ids::TOPBAR_PAUSE,
            ids::TOPBAR_RESET
        ]
    );
    let (inicio, fim) = (r[0].2.x, r[2].2.x + r[2].2.w);
    assert!(((inicio + fim) * 0.5 - 500.0).abs() < 0.01);
}

/// ⭐ Dono 07/11: a área clicável é o desenho. Cada botão responde até a borda do seu quadrado, com o ícone no centro,
/// e um ponto fora não é de ninguém. Mutação: registar o `glifo` em vez de `r` ⇒ a borda deixa de responder.
#[test]
fn a_area_clicavel_e_o_quadrado_que_o_botao_desenha() {
    for (botoes, run) in [
        (rects(BARRA, 100.0, 800.0), GameRun::default()),
        (
            rects_palco(BARRA),
            GameRun {
                stage: Some(GameStage::Fullscreen),
                playing: false,
            },
        ),
    ] {
        let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
        paint(
            &botoes,
            run,
            &mut scene,
            Theme::Dark,
            &mut hits,
            &WidgetStore::default(),
        );
        for (id, _, r) in &botoes {
            let meio = r.y + r.h * 0.5;
            assert_eq!(hits.hit(r.x + r.w * 0.5, meio), Some(*id), "centro");
            assert_eq!(hits.hit(r.x + 0.5, meio), Some(*id), "borda esquerda");
            assert_eq!(hits.hit(r.x + r.w - 0.5, meio), Some(*id), "borda direita");
            assert_eq!(hits.hit(r.x + 0.5, r.y + 0.5), Some(*id), "canto");
        }
        let ultimo = botoes[botoes.len() - 1].2;
        assert_eq!(hits.hit(ultimo.x + ultimo.w + 1.0, ultimo.y + 1.0), None);
    }
}
