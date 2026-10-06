//! ⭐⭐ **IMAGE ▸ EDIT** — as ferramentas de imagem (Bg Removal, Padding, Real Size, Make Square…)
//! são um MODO da imagem, ao lado do Paint (decisão do dono, 05/10: o botão IMG da barra *«vira
//! modo da imagem»*). O seletor do topo oferece Object · Paint · Edit.
//!
//! - **O modo É o interruptor**: o `image_edit.mode_on` (que a fila das ferramentas de imagem, o
//!   `activation_gate` e o canvas lêem) passou a ser o ESPELHO dele ([`mirror`]); o botão IMG saiu.
//! - ⭐ **O Edit é do TIPO** (dono, 05/10: *«o modo de edição significa que todos os objetos daquele
//!   tipo estão em modo de edição»*): toda imagem é parte dele — o *Equalize Sizes* trabalha sobre
//!   várias —, e escolher um objecto de outro tipo volta a Object.

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;

/// ⭐ **A família**, construída em cada quadro com as imagens vivas.
pub struct Family {
    images: Vec<u64>,
}

impl Family {
    /// As imagens do mundo (o marcador do tipo Image é o `Sprite`).
    #[must_use]
    pub fn new(world: &ph2d_ecs::World) -> Self {
        Self::of(images(world))
    }

    /// A família sobre uma lista dada de imagens.
    #[must_use]
    pub fn of(images: Vec<u64>) -> Self {
        Self { images }
    }
}

/// Os bits de toda entidade do tipo Image.
#[must_use]
pub fn images(world: &ph2d_ecs::World) -> Vec<u64> {
    world
        .iter_entities()
        .filter(|e| e.contains::<ph2d_render::Sprite>())
        .map(|e| e.id().to_bits())
        .collect()
}

impl ModeFamily for Family {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Image, ObjectMode::Edit)]
    }
    fn holds(&mut self, mode: ObjectMode, entity: u64, _: &mut ToolRegistry) -> bool {
        mode == ObjectMode::Edit && self.images.contains(&entity)
    }
    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        self.holds(mode, entity, tools)
    }
    /// ⚠️ Não larga a ferramenta: com o espelho a `false`, quem a larga é a fase
    /// `fase_image_tools_mode_and_pills` da shell — e é ela que limpa a pré-visualização do Bg
    /// Removal (largar aqui deixaria a pré-visualização a arder).
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {}
    fn joins(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }
    fn holds_the_whole_kind(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }
    /// ⭐ A imagem em Edit mantém o gizmo (report do dono, 05/10) — como com o antigo botão IMG.
    fn parts_take_the_object_gizmo(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }
    fn parts(&mut self, entity: u64) -> Option<Vec<u64>> {
        if !self.images.contains(&entity) {
            return None;
        }
        let others: Vec<u64> = self
            .images
            .iter()
            .copied()
            .filter(|b| *b != entity)
            .collect();
        (!others.is_empty()).then_some(others)
    }
    /// A imagem trancada apagada passa o Edit a outra (o Edit é do tipo).
    fn heir(&mut self, mode: ObjectMode, locked: u64, _: &mut ToolRegistry) -> Option<u64> {
        (mode == ObjectMode::Edit && !self.images.contains(&locked))
            .then(|| self.images.last().copied())
            .flatten()
    }
}

/// ⭐ **O espelho**: o `image_edit.mode_on` é `true` exactamente no Edit de uma imagem. Corre depois
/// do quadro do modo; `is_image` responde pelo tipo de uma entidade.
pub fn mirror(hero: &mut HeroScreen, is_image: impl Fn(u64) -> bool) {
    hero.image_edit.mode_on = hero
        .gizmo
        .mode
        .active()
        .is_some_and(|a| a.mode == ObjectMode::Edit && is_image(a.entity));
}

#[cfg(test)]
#[path = "image_edit_mode_tests.rs"]
mod tests;
