//! A CRT da Preview (P13 U-CRT): desligada = os bytes de hoje; o clique REAL na caixa CRT liga a
//! preferência da vista e a imagem passa pelo passe; o documento nunca muda.

use std::sync::Arc;

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_preview::{
    PixelPreviewPanel, PixelPreviewPanelState, PreviewFrame, crt, ids, publish,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

fn quadro() -> Arc<Vec<u8>> {
    Arc::new((0..16 * 8 * 4).map(|i| (i * 7 % 251) as u8).collect())
}

fn palco(rgba: Arc<Vec<u8>>) -> (MockPanelHost, PixelPreviewPanelState) {
    crt::define(crt::VistaCrt::default());
    publish(Some(PreviewFrame {
        rgba,
        w: 16,
        h: 8,
        seq: 1,
        frame: 7,
        playing: false,
        checker: Default::default(),
    }));
    (
        MockPanelHost::with_panel::<PixelPreviewPanel>(),
        PixelPreviewPanelState::default(),
    )
}

#[test]
fn a_crt_desligada_e_a_previa_de_hoje() {
    let rgba = quadro();
    let (mut host, mut st) = palco(rgba.clone());
    let _ = host.paint::<PixelPreviewPanel>(&mut st, VIEWPORT);
    let (passes, mostrada) = st.crt_mostrada();
    assert_eq!(passes, 0, "desligada, o passe CRT não corre");
    let m = mostrada.expect("a Preview mostrou uma imagem");
    assert!(
        Arc::ptr_eq(&m, &rgba),
        "desligada mostra os bytes publicados"
    );
    assert_eq!(*m, *rgba);
}

#[test]
fn a_caixa_crt_liga_a_vista_e_a_previa_passa_pelo_passe() {
    let rgba = quadro();
    let (mut host, mut st) = palco(rgba.clone());
    let rects = host.paint::<PixelPreviewPanel>(&mut st, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == ids::CRT)
        .map(|(_, r)| *r)
        .expect("a caixa CRT pinta-se");
    let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ligou = false;
    for e in ev {
        if matches!(e, WidgetEvent::Click(c) | WidgetEvent::Toggled(c) if c == ids::CRT) {
            let _ = host.apply_panel_event::<PixelPreviewPanel>(&mut st, e);
            ligou = true;
        }
    }
    assert!(ligou, "o clique no meio da caixa CRT não chegou a ela");
    assert!(crt::vista().ligada, "a caixa liga a preferência da vista");
    let _ = host.paint::<PixelPreviewPanel>(&mut st, VIEWPORT);
    let (passes, mostrada) = st.crt_mostrada();
    assert_eq!(passes, 1, "ligada, a imagem passa pelo passe CRT uma vez");
    let m = mostrada.expect("a Preview mostrou uma imagem");
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let k = (st.escala().floor().max(1.0)) as u32;
    let esperada = ph2d_pixel_crt::crt_cpu(&rgba, 16, 8, k, &crt::vista().params);
    assert_eq!(*m, esperada, "a imagem mostrada é a da lei CRT a {k}×");
    assert_ne!(*m, ph2d_pixel_crt::amplia_nearest(&rgba, 16, 8, k));
    let _ = host.paint::<PixelPreviewPanel>(&mut st, VIEWPORT);
    assert_eq!(st.crt_mostrada().0, 1, "o mesmo quadro não refaz o passe");
    crt::define(crt::VistaCrt::default());
}

#[test]
fn a_crt_ligada_nao_muda_o_documento() {
    use ph2d_pixel_core::{NewSprite, Sprite, codec::write_entry};
    let s = Sprite::new(NewSprite {
        width: 16,
        height: 8,
        ..NewSprite::default()
    })
    .expect("sprite");
    let doc = || write_entry(1, &s);
    let antes = doc();
    let frame = s.frames()[0].id;
    let mut out = Vec::new();
    ph2d_pixel_core::compose_rgba(&s, frame, None, &mut out);
    let (mut host, mut st) = palco(Arc::new(out));
    crt::define(crt::VistaCrt {
        ligada: true,
        ..crt::VistaCrt::default()
    });
    let _ = host.paint::<PixelPreviewPanel>(&mut st, VIEWPORT);
    assert_eq!(st.crt_mostrada().0, 1);
    assert_eq!(doc(), antes, "a CRT é vista: o documento gravado não muda");
    crt::define(crt::VistaCrt::default());
}
