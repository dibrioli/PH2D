//! ⭐⭐ **Os ícones dos componentes na linha** (doc 01 §7.1) — num relance vê-se o que cada objecto
//! FAZ, e o clique leva ao cartão. O Godot só mostra script e sinais; o Unity, nada.
//!
//! Um ícone por FAMÍLIA presente ([`ph2d_editor_core::hier_rows::component_icon`]), à esquerda das
//! travas, na ordem da paleta: um objecto com três componentes de Lógica mostra um ⚡, não três.

use ph2d_editor_core::hier_rows::component_icon;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{paint_icon, resolve};
use ph2d_editor_core::screens::hero::fixture;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Pinta da direita para a esquerda a partir de `right_x` e devolve onde acabou.
pub(crate) fn paint_component_icons(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
    row_id: Option<ph2d_a11y::NodeId>,
    mut hit_index: Option<&mut HitIndex>,
    mut right_x: f32,
) -> f32 {
    let size = INLINE_ICON_PX;
    let gap = Spacing::Xxs.px();
    let color = resolve(
        if entity.muted {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text3
        },
        theme,
    );
    for &category in entity.components.iter().rev() {
        let Some(icon) = component_icon(category) else {
            continue;
        };
        right_x -= gap;
        let icon_rect = Rect::new(right_x - size, rect.y + (rect.h - size) * 0.5, size, size);
        paint_icon(scene, icon, icon_rect, color, StrokeToken::Default.px());
        if let (Some(row), Some(idx)) = (row_id, hit_index.as_mut()) {
            // A folga é a do intervalo: dois ícones vizinhos nunca partilham um píxel de clique.
            let hit = crate::row::row_tall(rect, icon_rect.x - gap * 0.5, size + gap);
            idx.register(crate::ids::hier_component_icon(row, category), hit);
        }
        right_x -= size;
    }
    right_x
}
