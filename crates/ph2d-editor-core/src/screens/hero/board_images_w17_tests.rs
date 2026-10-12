//! W17 — a imagem que NÃO se lê (o cabeçalho bom, os dados estragados) avisa na tela do quadro.

use std::sync::Arc;

use super::*;
use crate::screens::hero::board_images::insert_images;
use ph2d_board_model::ImageFile;

/// Pinta até nenhuma imagem do quadro estar a descodificar.
fn settle(t: &mut T) {
    let t0 = std::time::Instant::now();
    loop {
        t.paint();
        if !t.hero.documents.live.render_cache.images.pending() {
            t.paint();
            return;
        }
        assert!(t0.elapsed().as_secs() < 20, "ficou a descodificar");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// ⭐ Pelo ecrã REAL: a imagem chega, o descodificador não a lê, e a faixa de avisos do quadro diz-o —
/// uma vez por ficheiro (pintar outra vez não repete; outro ficheiro estragado avisa de novo).
#[test]
fn an_unreadable_picture_raises_the_board_notice_once_per_file() {
    let mut t = T::new();
    t.hero
        .documents
        .live
        .render_cache
        .images
        .set_decoder(Arc::new(|_: &[u8], _| None));
    let f = ImageFile::new(vec![7; 4], [120, 80]);
    assert_eq!(insert_images(&mut t.hero, vec![f], None), 1);
    settle(&mut t);
    assert_eq!(
        t.hero.documents.live.notice.map(|(k, _)| k),
        Some("board.images.unreadable"),
        "a imagem ilegível não avisou"
    );
    t.hero.documents.live.notice = None;
    settle(&mut t);
    assert_eq!(t.hero.documents.live.notice, None, "avisou duas vezes");
    let g = ImageFile::new(vec![9; 4], [60, 40]);
    assert_eq!(insert_images(&mut t.hero, vec![g], None), 1);
    settle(&mut t);
    assert_eq!(
        t.hero.documents.live.notice.map(|(k, _)| k),
        Some("board.images.unreadable"),
        "o 2.º ficheiro estragado não avisou"
    );
}
