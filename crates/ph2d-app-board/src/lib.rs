//! **A família do QUADRO** (MiroClone) — `docs/MiroClone/02_plano.md`.
//!
//! Um quadro é um DOCUMENTO numa aba da barra de cima, nunca um objecto da cena. O modelo é
//! `ph2d-board-model`, o desenho `ph2d-board-render`; as abas e a vista vivem no `ph2d-editor-core`
//! (são pintadas por `paint_hero_screen`). Aqui fica o que a shell chama pelo nome.

/// As cenas de smoke (`PH2D_BOARD_SMOKE`).
pub mod smoke;

/// Os ficheiros do quadro (W8): exportar, importar, o nome de quem escreve.
pub mod io;

/// O quadro ao vivo (W11, ADR-0181): a sessão e a rede local, servidas a cada imagem.
pub mod live;

/// As imagens a entrar no quadro (W14): a porta de entrada e o descodificador do desenho.
pub mod images;

/// «Send to board…» da Hierarquia (W15): o modal e a entrega do conteúdo de um objecto da cena.
pub mod send_to_board;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "board",
    routers: smoke::ROUTERS,
};
