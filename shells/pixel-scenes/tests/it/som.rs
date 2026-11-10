//! P13 T pelo caminho da família: o **Add Audio…** do painel (o pedido no canal) e o diálogo
//! respondido chegam ao documento como UMA faixa, num passo; Ctrl+Z tira-a.

use ph2d_app_pixel::mode;
use ph2d_pixel_audio::canal::{self, Pedido};

use crate::comum::born;

#[test]
fn add_audio_chega_ao_documento_num_passo_e_o_desfazer_tira() {
    let (mut app, sim, _cam, _bits) = born();
    let (_, key) = mode::sprites(sim.world())[0];
    let _ = app.foca(key);
    let passos = app.doc(key).expect("doc").history.undo_len();
    let ficheiro = std::env::temp_dir().join("ph2d-pixel-som-nao-existe.wav");
    ph2d_app_host::modal::responde_o_proximo(ficheiro.clone());
    canal::pede(Pedido::Escolher);
    app.som();
    let d = app.doc(key).expect("doc");
    assert_eq!(d.history.undo_len(), passos + 1, "um passo");
    assert_eq!(d.sprite.audio().len(), 1);
    assert_eq!(d.sprite.audio()[0].asset, ficheiro.to_string_lossy());
    // A linha publicada para o painel (o ficheiro não existe: sem duração, sem pânico).
    let l = canal::linhas();
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].dur_ms, None);
    assert!(app.undo(key), "Ctrl+Z");
    assert!(app.doc(key).expect("doc").sprite.audio().is_empty());
    app.som();
    assert!(canal::linhas().is_empty());
}
