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
/// P13 L: a linha *Light* (Smooth · Palette) do cartão Pixel Sprite.
#[path = "pixel_luz_rampa.rs"]
mod pixel_luz_rampa;
/// P6: a selecção, a moldura solta e os menus Select/Edit/Sprite do painel.
#[path = "pixel_pincel.rs"]
mod pixel_pincel;
/// P13 U-Paleta: as rampas sob a paleta, New Ramp… e o Cycle.
#[path = "pixel_rampas.rs"]
mod pixel_rampas;
#[path = "pixel_selecao.rs"]
mod pixel_selecao;
/// P13 T: as linhas de som da Timeline.
#[path = "pixel_som.rs"]
mod pixel_som;
#[path = "pixel_tiles.rs"]
mod pixel_tiles;
/// P13 T: Cel ▸ Tween… (os quadros do meio).
#[path = "pixel_tween.rs"]
mod pixel_tween;

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
        .or_else(|| pixel_rampas::tr(k))
        .or_else(|| pixel_luz_rampa::tr(k))
        .or_else(|| pixel_som::tr(k))
        .or_else(|| pixel_tween::tr(k))
}
