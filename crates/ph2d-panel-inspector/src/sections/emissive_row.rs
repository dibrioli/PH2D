//! **A linha `Emissive`** — a sprite como fonte de luz, no Inspector.
//! Plano [`docs/Sprite_projeto/18`](../../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W8, por ordem do Enio (2026-08-21).
//!
//! # Por que um ficheiro irmão em vez de mais dez linhas no `render_source`
//!
//! ⚠️ O [`super::render_source`] está a **550** linhas contra um tecto de 600
//! (`architecture_panel_loc_cap`). Enfiar a linha lá dentro cabia hoje e rebentava o tecto na
//! próxima. *Um tecto de ficheiro e um tecto de função medem grandezas diferentes*, e a cura de
//! ambos é a mesma: cortar para o irmão.
//!
//! # Onde ela aparece, e por que ali
//!
//! Logo abaixo do par `Format`, na secção **Render Source**. ⚠️ **É a vizinhança que explica a
//! feature sem uma palavra:** a emissão é a única coisa no app que precisa da folga acima de 1.0 que
//! os 16 bits dão. Ela **funciona** numa sprite de 8 bits (o multiplicador empurra a cor para cima
//! na hora de desenhar); o que os 16 bits acrescentam é poder **guardar** o brilho na textura — um
//! EXR importado emite sozinho, sem tocar neste slider.

use super::*;
use ph2d_editor_core::widget::{SliderState, paint_slider_with_chip};

/// Altura do campo, igual à das outras linhas com chip.
const FIELD_H: f32 = 24.0; // LITERAL-PX-OK: mesma altura de campo do resto do Inspector

/// Pinta a linha e devolve o `cur_y` a seguir a ela.
///
/// ⚠️ **O slider guarda `0..1` normalizado e a chip mostra a intensidade real.** É o mesmo par que
/// a Opacidade usa (armazenamento contínuo, leitura humana), e a razão é a mesma: um slider cujo
/// curso fosse `0..64` daria ao artista 63/64 do percurso para valores que ele nunca usa. O
/// mapeamento vive no [`crate::populate`], num sítio só.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_emissive_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    cur_y: f32,
) -> f32 {
    let (_, value) = store
        .slider(ids::INSP_SPRITE_EMISSIVE)
        .unwrap_or((SliderState::Normal, 0.0));
    let h = paint_slider_with_chip(
        Rect::new(x, cur_y, w, FIELD_H),
        "Emissive",
        value,
        ids::INSP_SPRITE_EMISSIVE,
        ids::INSP_SPRITE_EMISSIVE_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    cur_y + h
}
