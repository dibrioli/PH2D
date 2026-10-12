//! W15 — **«Send to board…»** da Hierarquia: a metade do QUADRO. A shell (composição) lê o conteúdo do
//! objecto na família dele — os píxeis da sprite, o SVG do vector, o quadro actual do Flip — e chama
//! aqui: [`ask`] abre o modal que escolhe o quadro, [`deliver`] põe o conteúdo nele.
//!
//! ⚠️ Tudo entra pela porta de entrada do quadro ([`crate::images::ingest`]): o que o soltar aceita,
//! isto aceita, com o mesmo tamanho natural.

use ph2d_board_model::BoardId;
use ph2d_board_render::Pixels;
use ph2d_editor_core::Toast;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_images::insert_images;
use ph2d_editor_core::screens::hero::document_tabs::register_board;
use ph2d_editor_core::screens::hero::send_to_board::open;
use ph2d_i18n::{tr, tr_with};

/// O conteúdo de um objecto da cena, lido pela família dele.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    /// RGBA de 8 bits com alfa RETO (a sprite, o quadro do Flip) — entra como PNG.
    Pixels(Pixels),
    /// Um documento SVG (o vector) — entra como SVG, nítido a qualquer zoom.
    Svg(String),
}

impl Content {
    /// Os píxeis RGBA de 8 bits, alfa reto, `size` em píxeis.
    #[must_use]
    pub fn rgba(size: [u32; 2], rgba: Vec<u8>) -> Self {
        Self::Pixels(Pixels { size, rgba })
    }

    /// Os bytes do ficheiro que o quadro guarda.
    fn bytes(self) -> Option<Vec<u8>> {
        match self {
            Self::Pixels(px) => crate::images::encode_png(&px),
            Self::Svg(s) => Some(s.into_bytes()),
        }
    }
}

/// O pedido da linha (`HierRequest::SendToBoard`): abre o modal se o objecto é de um tipo que um
/// quadro recebe (`goes`, pela lei `send_to_board::goes_to_board`); senão, o aviso que o diz.
pub fn ask(hero: &mut HeroScreen, row: ph2d_editor_core::NodeId, goes: bool) -> Option<Toast> {
    if !goes {
        return Some(Toast::warning(tr("board.send.wrong_kind")));
    }
    open(&mut hero.store, row);
    None
}

/// ⭐ A escolha do modal (`HierRequest::SendToBoardChosen`): `board` = um quadro aberto, `None` = um
/// quadro NOVO (o nome por omissão das abas, como o «+»). O quadro fica ACTIVO (a aba dele abre-se) e
/// o conteúdo nasce no meio da vista dele, seleccionado, num passo de desfazer. `name` é o do objecto,
/// para o aviso de quando o conteúdo não chega (`None`) ou o quadro o recusa.
///
/// ⚠️ Nada muda antes de o conteúdo ser aceite: um conteúdo recusado não cria um quadro vazio nem troca
/// a aba.
pub fn deliver(
    hero: &mut HeroScreen,
    board: Option<u64>,
    name: &str,
    content: Option<Content>,
) -> Result<BoardId, Toast> {
    let say = |key: &str| Toast::warning(tr_with(key, &[("name", &name)]));
    let content = content.ok_or_else(|| say("board.send.empty"))?;
    let file = content
        .bytes()
        .and_then(|b| crate::images::ingest(&b))
        .ok_or_else(|| say("board.send.refused"))?;
    let target = match board.map(BoardId) {
        Some(b) if hero.documents.boards().get(b).is_some() => b,
        Some(_) => return Err(Toast::warning(tr("board.send.no_board"))),
        None => {
            let n = hero.documents.boards().boards().len() + 1;
            let fresh = tr_with("board.tab.default_name", &[("n", &n)]);
            let b = hero.documents.create(fresh);
            register_board(&mut hero.store, b);
            b
        }
    };
    hero.documents.activate(Some(target));
    if insert_images(hero, vec![file], None) == 0 {
        return Err(say("board.send.refused"));
    }
    Ok(target)
}

#[cfg(test)]
#[path = "send_to_board_tests.rs"]
mod tests;
