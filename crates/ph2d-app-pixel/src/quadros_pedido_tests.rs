use ph2d_app_host::quadro::{PRAZO, no_laco};
use ph2d_editor_core::pixel_panel::{PixelFrameAction as A, PixelRequest};

fn acorda(app: &crate::PixelApp, agora: std::time::Instant) -> Option<std::time::Instant> {
    app.pede_o_quadro(true, agora);
    no_laco(|l| l.acorda(&[], agora))
}

#[test]
fn o_pixel_a_tocar_e_as_formigas_pedem_quadro_e_parado_deixa_dormir() {
    let (mut app, agora) = (crate::PixelApp::default(), std::time::Instant::now());
    let key = app
        .create(ph2d_pixel_core::NewSprite::default())
        .expect("sprite");
    app.chave = Some(key);
    assert_eq!(acorda(&app, agora), Some(agora + PRAZO), "parado ⇒ dorme");
    assert!(app.aplica_quadro(&PixelRequest::FrameAction(A::Play)));
    app.tick(25);
    assert_eq!(acorda(&app, agora), None, "a tocar ⇒ desenha já");
    assert!(app.aplica_quadro(&PixelRequest::FrameAction(A::Play)));
    let op = ph2d_pixel_core::selection::select_all(&app.docs[&key].sprite).expect("op");
    app.commit(key, op).expect("commit");
    let passo = std::time::Duration::from_millis(crate::draw::ANTS_STEP_MS - 25);
    assert_eq!(
        acorda(&app, agora),
        Some(agora + passo),
        "formigas ⇒ o passo delas"
    );
}
