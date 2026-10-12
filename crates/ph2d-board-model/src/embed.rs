//! W18 ([ADR-0185]) — **um vídeo no quadro**: o quadro guarda o LINK e o que ele é; quem toca é o navegador
//! (no PH2D de mesa, o da pessoa, projectado no canvas; no PH2D web, um `<iframe>`) — o modelo não sabe de
//! motor. Ao vivo viaja o link: cada par carrega o seu tocador e a reprodução não se sincroniza.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use serde::{Deserialize, Serialize};

use crate::Style;

/// O que o link mostra. ⚠️ Append-only (o discriminante vai para o ficheiro).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Video {
    /// Um vídeo do YouTube: o id (11 caracteres), o começo em segundos e se é ao alto (um `/shorts/`).
    YouTube {
        id: String,
        start: u32,
        vertical: bool,
    },
}

/// Um vídeo do quadro: estica-se à caixa `x, y, w, h` do elemento.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Embed {
    /// O link como chegou (o `.excalidraw` grava-o; «Abrir no navegador» abre-o).
    pub link: String,
    pub video: Video,
    /// A opacidade, os cantos redondos e a moldura, como numa imagem.
    pub style: Style,
}
