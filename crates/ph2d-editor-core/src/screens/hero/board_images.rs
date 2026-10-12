//! W14 — **pôr IMAGENS no quadro activo** (soltar, colar, o botão): os bytes entram no quadro (uma vez
//! por conteúdo) e as imagens nascem pelo editor ([`ph2d_board_edit::Editor::insert_images`]) — no
//! ponto do ecrã onde se largou, ou no meio da vista; cada uma a caber em METADE da área à vista (uma
//! fotografia grande não tapa o quadro). UM passo de desfazer.

use super::HeroScreen;
use super::board_view::{area, area_of, editor};
use ph2d_board_model::ImageFile;

/// Um `Ctrl+V` com uma imagem na área de transferência é para pôr a IMAGEM? Só com um quadro activo,
/// sem estar a escrever num texto, e quando a área de transferência NÃO traz texto: uma folha de
/// cálculo copiada traz as células E uma imagem delas (as células viram notas), e a cópia do próprio
/// quadro traz o texto dela (cola-se a si). Uma captura de ecrã ou a imagem de um navegador não trazem.
#[must_use]
pub fn takes_image_paste(hero: &HeroScreen, text: Option<&str>) -> bool {
    if hero.documents.active().is_none() || text.is_some_and(|t| !t.trim().is_empty()) {
        return false;
    }
    hero.documents
        .live
        .editor
        .as_ref()
        .is_none_or(|ed| !ed.is_editing_text())
}

/// Põe `files` no quadro activo, à volta do píxel de ecrã `at` (ou do meio da vista). Devolve
/// quantas entraram — `0` sem quadro activo.
///
/// ⚠️ ANTES do primeiro desenho (sem área — um quadro criado e escrito no mesmo passo, o «Send to
/// board…» para um quadro novo) a imagem nasce no centro da CÂMARA do quadro, que é o meio da vista
/// quando ele se desenhar, no tamanho natural (não há vista a que caber); o `at` de ecrã não tem
/// tradução e é ignorado.
pub fn insert_images(hero: &mut HeroScreen, files: Vec<ImageFile>, at: Option<[f64; 2]>) -> usize {
    let view = hero.last_layout.map(|l| area_of(area(&l)));
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return 0;
    };
    let middle = [board.camera.center_x, board.camera.center_y];
    let center = match (at, view) {
        (Some(p), Some(v)) => board.camera.to_world(v, p),
        _ => middle,
    };
    let z = board.camera.zoom;
    let fit = view.map_or([f64::INFINITY; 2], |v| [v[2] / z / 2.0, v[3] / z / 2.0]);
    let items: Vec<_> = files
        .into_iter()
        .map(|f| {
            let size = f.size;
            (board.put_file(f), size)
        })
        .collect();
    let history = live.histories.entry(board.id).or_default();
    let ed = editor(&mut live.editor, theme);
    ed.insert_images(&mut board.doc, history, &items, center, fit)
        .len()
}
