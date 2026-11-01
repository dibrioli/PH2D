//! ⭐ **O ícone da MACRO no canvas** (dono 01/11, plano GEF 19 E6): no lugar do anel do vazio, o glifo do painel
//! Macros, com o «Icon Size» do projecto — como o da câmera (`ph2d_camera_frame::tinta`). A tinta e o dedo leem a
//! MESMA conta ([`icones`]), em px de tela.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

const TRACO_PX: f32 = 2.0; // LITERAL-PX-OK: chrome de overlay, o traço do ícone da câmera

/// É uma macro? (o anel do vazio dá-lhe o lugar ao ícone).
#[must_use]
pub fn e_macro(world: &ph2d_ecs::World, e: ph2d_ecs::Entity) -> bool {
    world.get::<ph2d_ecs::Macro>(e).is_some()
}

/// O rectângulo do ícone de toda macro visível, em px da vista.
#[must_use]
pub fn icones(
    world: &ph2d_ecs::World,
    lado: f32,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(u64, Rect)> {
    let Some(mut q) = world.try_query::<(ph2d_ecs::Entity, &ph2d_ecs::Macro)>() else {
        return Vec::new();
    };
    let mut scratch = Vec::new();
    q.iter(world)
        .map(|(e, _)| e)
        .filter(|&e| !ph2d_ecs::is_hidden_in_tree(world, e))
        .filter_map(|e| {
            let t = ph2d_ecs::world_transform_into(world, e, &mut scratch)?;
            let (x, y) = camera.world_to_screen([t.translation.x, t.translation.y], window);
            Some((
                e.to_bits(),
                Rect::new(x - lado * 0.5, y - lado * 0.5, lado, lado),
            ))
        })
        .collect()
}

/// ⭐ **O ícone pega**: as macros cujo ícone contém `tela`.
#[must_use]
pub fn icones_em(
    world: &ph2d_ecs::World,
    tela: (f32, f32),
    lado: f32,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<u64> {
    icones(world, lado, camera, window)
        .into_iter()
        .filter(|(_, r)| r.contains(tela.0, tela.1))
        .map(|(b, _)| b)
        .collect()
}

/// Pinta o ícone de toda macro visível.
pub fn desenhar(
    world: &ph2d_ecs::World,
    lado: f32,
    theme: Theme,
    camera: &Camera2d,
    window: WindowSize,
    cena: &mut VectorScene,
) {
    let cor = ph2d_editor_core::paint::resolve(ColorToken::Selection, theme);
    for (_, r) in icones(world, lado, camera, window) {
        ph2d_editor_core::paint_icon(cena, IconId::Macro, r, cor, TRACO_PX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ O ícone da macro está no sítio dela, do lado do «Icon Size», e a macro escondida não pega.
    #[test]
    fn o_icone_da_macro_pega_no_sitio_dela_e_escondida_some() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let m = sim
            .world_mut()
            .spawn((
                ph2d_ecs::Transform::from_translation(ph2d_core::Vec2::new(2.0, 1.0)),
                ph2d_ecs::Macro::default(),
            ))
            .id();
        let (cam, janela) = (Camera2d::default(), WindowSize::new(1280, 720));
        let (x, y) = cam.world_to_screen([2.0, 1.0], janela);
        let lado = ph2d_editor_core::game_format::CAMERA_ICON_PX;
        assert_eq!(
            icones_em(sim.world(), (x + lado * 0.5 - 1.0, y), lado, &cam, janela),
            vec![m.to_bits()]
        );
        assert!(icones_em(sim.world(), (x + lado * 0.5 + 1.0, y), lado, &cam, janela).is_empty());
        sim.world_mut()
            .entity_mut(m)
            .insert(ph2d_ecs::Visibility::hidden());
        assert!(icones_em(sim.world(), (x, y), lado, &cam, janela).is_empty());
    }
}
