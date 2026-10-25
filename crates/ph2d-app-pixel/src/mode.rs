//! **Pixel Sprite ▸ Edit** — o modo onde o lápis pinta. O seletor do topo oferece Object · Edit, e
//! o `Tab` alterna.

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;

use crate::PixelApp;

/// A família de modo, construída em cada quadro com os Pixel Sprites vivos.
pub struct Family<'a> {
    app: &'a mut PixelApp,
    sprites: Vec<u64>,
}

impl<'a> Family<'a> {
    #[must_use]
    pub fn new(app: &'a mut PixelApp, world: &ph2d_ecs::World) -> Self {
        Self {
            app,
            sprites: sprites(world).into_iter().map(|(bits, _)| bits).collect(),
        }
    }
}

/// Os objectos Pixel Sprite do mundo: `(bits, chave do documento)`, do mais VELHO para o mais novo
/// (`StableId` crescente; sem ele, os bits) — numa cópia, o original vem primeiro.
#[must_use]
pub fn sprites(world: &ph2d_ecs::World) -> Vec<(u64, u32)> {
    let mut v: Vec<(u64, u64, u32)> = world
        .iter_entities()
        .filter_map(|e| {
            let age = e.get::<ph2d_ecs::StableId>().map_or(u64::MAX, |s| s.0);
            e.get::<ph2d_ecs::PixelSprite>()
                .map(|p| (age, e.id().to_bits(), p.0))
        })
        .collect();
    v.sort_unstable();
    v.into_iter().map(|(_, bits, key)| (bits, key)).collect()
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::PixelSprite, ObjectMode::Edit)]
    }
    fn holds(&mut self, mode: ObjectMode, entity: u64, _: &mut ToolRegistry) -> bool {
        mode == ObjectMode::Edit && self.sprites.contains(&entity)
    }
    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        self.holds(mode, entity, tools)
    }
    /// Largar o Edit a meio de um traço deixa-o cair (nunca um passo fantasma).
    fn leave(&mut self, _: ObjectMode, entity: u64, _: &mut ToolRegistry) {
        if self
            .app
            .gesture
            .as_ref()
            .is_some_and(|g| g.entity == entity)
        {
            self.app.gesture = None;
        }
    }
    fn wants(&mut self, _: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        let bits = self.app.wants_edit.take()?;
        self.sprites
            .contains(&bits)
            .then_some((bits, ObjectMode::Edit))
    }
    fn heir(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) -> Option<u64> {
        None
    }
}

/// O objecto em Pixel Sprite ▸ Edit neste quadro, se há.
#[must_use]
pub fn editing(
    world: &ph2d_ecs::World,
    hero: &ph2d_editor_core::screens::hero::HeroScreen,
) -> Option<u64> {
    let a = hero.gizmo.mode.active()?;
    (a.mode == ObjectMode::Edit && crate::input::key_of(world, a.entity).is_some())
        .then_some(a.entity)
}
