//! **As tabelas do PIXEL ART** (ADR-0196), numa só entrada da `lib.rs` (que vive no tecto de 700
//! linhas). Os ficheiros continuam em `src/` com o mesmo nome — os censos `every_word_…` dos painéis
//! leem-nos pelo caminho. `pixel_slices` e `pixel_teclas` ficam na `lib.rs`: têm submódulos em pasta.

/// O menu Add, as ferramentas e os avisos da família.
#[path = "pixel.rs"]
mod pixel;
/// P13 U-CRT: a caixa CRT e o cartão da Preview.
#[path = "pixel_crt.rs"]
mod pixel_crt;
/// P6: os diálogos Color Range, Modify, Canvas Size e Sprite Size.
#[path = "pixel_dialogos.rs"]
mod pixel_dialogos;
/// P13: Layer Properties ▸ Effects ▸ Outline.
#[path = "pixel_efeitos.rs"]
mod pixel_efeitos;
#[path = "pixel_export.rs"]
mod pixel_export;
/// P13: View ▸ Guides (perspectiva e iso 2:1).
#[path = "pixel_guias.rs"]
mod pixel_guias;
/// P6: a selecção, a moldura solta e os menus Select/Edit/Sprite do painel.
#[path = "pixel_pincel.rs"]
mod pixel_pincel;
#[path = "pixel_selecao.rs"]
mod pixel_selecao;
#[path = "pixel_tiles.rs"]
mod pixel_tiles;

/// A tradução de uma chave destas tabelas, ou `None` — a mesma ordem que a `lib.rs` tinha.
pub(crate) fn tr(k: &str) -> Option<&'static str> {
    pixel::tr(k)
        .or_else(|| pixel_selecao::tr(k))
        .or_else(|| pixel_pincel::tr(k))
        .or_else(|| pixel_dialogos::tr(k))
        .or_else(|| pixel_efeitos::tr(k))
        .or_else(|| pixel_export::tr(k))
        .or_else(|| pixel_crt::tr(k))
        .or_else(|| pixel_tiles::tr(k))
        .or_else(|| pixel_guias::tr(k))
}
