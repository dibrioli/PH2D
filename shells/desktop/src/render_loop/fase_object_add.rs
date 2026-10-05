//! **Fase do quadro: O MENU ADD DE OBJECTOS** (spec/06 F1) — abrir o modal quando o `+` da
//! Hierarquia ou o `Shift+A` o pedem, e entregar o pick à FAMÍLIA dona.
//!
//! ⚠️ **A shell é composição:** cada família declara as entradas e sabe criá-las com os recursos
//! dela; aqui só se junta a lista compilada e se passam os recursos.

use super::*;
use ph2d_editor_core::object_add::{self, AddEntry};

/// ⭐ **As famílias compiladas, na ordem do modal** — um tipo cuja crate saiu do build não deixa
/// item morto.
pub(crate) const FAMILIES: &[&[AddEntry]] = &[
    object_add::CORE,
    ph2d_app_vec::object_add::ENTRIES,
    ph2d_app_flip::object_add::ENTRIES,
    ph2d_app_components::object_add::ENTRIES,
];

/// O que o pick deu: um objecto já na cena, ou nada ainda (a imagem abre o diálogo de tamanho; o
/// desenho vetorial nasce no 1.º traço).
enum Born {
    Entity(u64),
    Dialog,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_object_add(&mut self, open: bool) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            hero_screen,
            component_registry,
            flip,
            ..
        } = FrameGfx::of(gfx);
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        if open {
            hero.store
                .open_command_palette(object_add::build(FAMILIES, &|_| None));
        }
        // ⚠️ O dreno é CONDICIONAL: o canal de pick tem outros quatro consumidores.
        let Some(entry) = hero
            .store
            .take_command_pick_if(|id| object_add::entry_of_pick(FAMILIES, id).is_some())
            .and_then(|id| object_add::entry_of_pick(FAMILIES, id))
        else {
            return;
        };
        let born: Result<Born, String> = if entry == object_add::EMPTY {
            Ok(Born::Entity(
                ph2d_app_components::object_add::spawn_empty_root(sim, entry.key.tr()),
            ))
        } else if entry == object_add::IMAGE {
            // ⭐ O `Ctrl+N` continua a ser o atalho desta entrada (escolha 2 do dono).
            hero.store.open_new_image_dialog();
            Ok(Born::Dialog)
        } else if let Some(r) = ph2d_app_components::object_add::add(
            entry,
            sim,
            component_registry,
            ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
        ) {
            r.map(Born::Entity)
        } else if ph2d_app_vec::object_add::add(entry, &mut self.vec) {
            use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
            let leave = ph2d_editor_core::action_bus::EditorAction::ObjectMode;
            hero.bus.push(leave(ModeRequest::Enter(ObjectMode::Object)));
            Ok(Born::Dialog)
        } else if let Some(r) =
            ph2d_app_flip::object_add::add(entry, sim, flip, &mut self.flip_state)
        {
            r.map(Born::Entity).map_err(String::from)
        } else {
            return;
        };
        let bits = match born {
            Ok(Born::Entity(bits)) => Some(bits),
            Ok(Born::Dialog) => None,
            Err(e) => {
                if let Some(g) = self.gfx.as_mut() {
                    g.toasts.push(Toast::error(ph2d_i18n::tr_with(
                        "object_add.failed",
                        &[("e", &e)],
                    )));
                }
                return;
            }
        };
        let Some(bits) = bits else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let name = gfx
            .sim
            .world()
            .get::<ph2d_ecs::Name>(ph2d_ecs::Entity::from_bits(bits))
            .map_or_else(|| entry.key.tr().to_string(), |n| n.0.clone());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.replace_selection(Some(bits));
        }
        gfx.toasts.push(Toast::success(ph2d_i18n::tr_with(
            "object_add.added",
            &[("name", &name)],
        )));
        self.title_dirty = true;
    }
}

#[cfg(test)]
#[path = "fase_object_add_tests.rs"]
mod tests;
