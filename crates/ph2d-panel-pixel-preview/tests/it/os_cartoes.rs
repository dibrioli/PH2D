//! ⭐ **A Preview na moldura da lista**: os verbos pelo clique real pedem o mesmo que `PH2D_CARDS=0`, e a imagem não
//! perde altura. A roda e o arrasto do título estão em `seam.rs`, que corre na cara nova.

use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_preview::{
    PixelPreviewPanel, PixelPreviewPanelState, PreviewFrame, ids, publish,
};
use ph2d_ui_testkit::mesa::Mesa;
use std::sync::Arc;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

fn mesa(cartoes: bool) -> Mesa<PixelPreviewPanel> {
    force_cards_on_this_thread(Some(cartoes));
    publish(Some(PreviewFrame {
        rgba: Arc::new(vec![255; 16 * 8 * 4]),
        w: 16,
        h: 8,
        seq: 1,
        frame: 7,
        playing: false,
        checker: Default::default(),
    }));
    let mut m0 = Mesa::nova(PixelPreviewPanelState::default());
    m0.host.set_panel_visible(PixelPreviewPanel::ID, true);
    m0.assenta_em(JANELA);
    m0
}

/// ⭐⭐ O tocar e o «×» fazem o mesmo com e sem cartões. *Mutação: não registar o `PLAY` ⇒ «não pintado».*
#[test]
fn os_verbos_fazem_o_mesmo() {
    let toca = |cartoes| format!("{:?}", mesa(cartoes).clica(ids::PLAY));
    assert_eq!(toca(false), toca(true));
    assert_ne!(toca(true), "[]", "vácuo: o tocar não pediu nada");
    for cartoes in [false, true] {
        let mut m0 = mesa(cartoes);
        let _ = m0.clica(ids::PREVIEW_CLOSE);
        assert!(
            !m0.host.panel_visible(PixelPreviewPanel::ID),
            "o «×» não fechou"
        );
    }
}

/// ⭐ Os verbos moram na linha do título da lista. *Mutação: tirar `no_meio` ⇒ vermelho.*
#[test]
fn os_verbos_estao_na_linha_do_titulo() {
    let m0 = mesa(true);
    let janela = m0
        .host
        .store()
        .panel_rect(PixelPreviewPanel::NODE_ID)
        .expect("rect");
    for id in [ids::PLAY, ids::PREVIEW_CLOSE] {
        let r: Rect = m0.rect(id).expect("pintado");
        let meio = r.y + r.h * 0.5 - janela.y;
        assert!((meio - m::LIST_HEAD_H * 0.5).abs() < 0.5, "{id:?} a {meio}");
    }
}

/// ⭐⭐ A imagem não perde altura: o título não fica mais alto. *Mutação: `header_h` maior ⇒ vermelho.*
#[test]
fn a_imagem_nao_perde_altura() {
    let titulo = |cartoes| mesa(cartoes).rect(ids::DRAG).expect("alça").h;
    let (antes, depois) = (titulo(false), titulo(true));
    eprintln!("altura do título: antes {antes}, cartões {depois}");
    assert!(depois <= antes + 0.5, "{antes} → {depois}");
}
