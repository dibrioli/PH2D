//! P13 T: as edições das faixas de som são um passo do desfazer cada.

use ph2d_pixel_audio::faixas::{Edicao, asset_de, op};
use ph2d_pixel_core::{History, NewSprite, Sprite};

fn faz(s: &mut Sprite, h: &mut History, e: Edicao) {
    let o = op(s, &e).expect("muda");
    h.commit(s, o).expect("aplica");
}

/// **Add Audio ⇒ a faixa no documento, num passo; Ctrl+Z volta.** E as outras edições também.
#[test]
fn add_audio_poe_a_faixa_num_passo_e_o_desfazer_tira() {
    let mut s = Sprite::new(NewSprite::default()).expect("sprite");
    let mut h = History::default();
    let antes = s.clone();
    faz(&mut s, &mut h, Edicao::Adiciona("sfx/step.ogg".into()));
    assert_eq!(h.undo_len(), 1);
    assert_eq!(s.audio().len(), 1);
    assert_eq!(s.audio()[0].asset, "sfx/step.ogg");
    assert!(h.undo(&mut s));
    assert_eq!(s, antes);
    assert!(h.redo(&mut s));
    let um = s.clone();
    for e in [
        Edicao::Mudo(0, true),
        Edicao::Move(0, -120),
        Edicao::Ganho(0, 0.5),
        Edicao::Remove(0),
    ] {
        let n = h.undo_len();
        faz(&mut s, &mut h, e.clone());
        assert_eq!(h.undo_len(), n + 1, "{e:?}: um passo");
    }
    assert!(s.audio().is_empty());
    for _ in 0..4 {
        assert!(h.undo(&mut s));
    }
    assert_eq!(s, um);
    // O que não muda nada não é passo.
    assert!(op(&s, &Edicao::Mudo(0, false)).is_none());
    assert!(op(&s, &Edicao::Remove(7)).is_none());
    assert!(op(&s, &Edicao::Adiciona(String::new())).is_none());
    assert!(op(&s, &Edicao::Ganho(0, f32::NAN)).is_none());
}

/// O asset dentro do projeto é relativo a ele; fora, o caminho.
#[test]
fn o_asset_e_relativo_ao_projeto() {
    let p = std::path::Path::new("/proj");
    assert_eq!(
        asset_de(std::path::Path::new("/proj/sfx/a.wav"), Some(p)),
        "sfx/a.wav"
    );
    assert_eq!(
        asset_de(std::path::Path::new("/outro/a.wav"), Some(p)),
        "/outro/a.wav"
    );
    assert_eq!(
        asset_de(std::path::Path::new("/outro/a.wav"), None),
        "/outro/a.wav"
    );
}
