//! **Fase do quadro: O MODO DO OBJECTO ACTIVO** (spec/06 F2) — os modos do tipo do activo, a rede
//! de segurança do modo em curso, o pedido do quadro (seletor, `Tab`, aba de cima) e o seletor.
//!
//! ⚠️ **A shell é composição:** as leis são `ph2d_editor_core::object_mode`; cada família declara
//! os seus modos (`MODES`) e sabe abrir e largar o módulo dela. O `match` sobre o modo é exaustivo
//! de propósito: um modo novo no vocabulário não compila sem a família que o abre.

use super::*;
use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::{self, ModeRequest, ObjectMode, Step};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_i18n::tr_with;

/// ⭐ **As famílias que declaram modos**, compiladas.
pub(crate) const MODE_FAMILIES: &[&[(ObjectKind, ObjectMode)]] =
    &[ph2d_app_painter::paint_mode::MODES];

/// O módulo do modo tem a entidade em mãos?
fn holds(mode: ObjectMode, tools: &mut ToolRegistry) -> bool {
    match mode {
        ObjectMode::Object => true,
        ObjectMode::Paint => ph2d_app_painter::paint_mode::holds_an_image(tools),
    }
}

/// Abre o módulo do modo sobre o activo.
fn enter(mode: ObjectMode, tools: &mut ToolRegistry, hero: &mut HeroScreen) -> bool {
    match mode {
        ObjectMode::Object => true,
        ObjectMode::Paint => ph2d_app_painter::paint_mode::enter(tools, hero),
    }
}

/// Larga o módulo do modo.
fn leave(mode: ObjectMode, tools: &mut ToolRegistry) {
    match mode {
        ObjectMode::Object => {}
        ObjectMode::Paint => ph2d_app_painter::paint_mode::leave(tools),
    }
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_object_mode(&mut self, request: Option<ModeRequest>) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            tools,
            toasts,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let request = request.or_else(|| ph2d_app_painter::paint_mode::smoke_step(sim, hero));
        if drive(sim.world(), tools, hero, toasts, request) {
            self.title_dirty = true;
        }
    }
}

/// **O quadro do modo**, sem o `App` — o que a fase corre e o que os gates dirigem. Devolve se o
/// modo mudou.
pub(crate) fn drive(
    world: &ph2d_ecs::World,
    tools: &mut ToolRegistry,
    hero: &mut HeroScreen,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
    request: Option<ModeRequest>,
) -> bool {
    let mut changed = false;
    // 1. O activo e os modos que o TIPO dele declara — o tipo pergunta-se ao marcador.
    let active = object_mode::active_of(&hero.gizmo);
    let entity = active.map(ph2d_ecs::Entity::from_bits);
    let modes: Vec<ObjectMode> = entity
        .map(|e| ph2d_app_components::component_attach::kind_of(world, e))
        .map(|k| {
            MODE_FAMILIES
                .iter()
                .flat_map(|f| f.iter())
                .filter(|(fk, _)| *fk == k)
                .map(|(_, m)| *m)
                .collect()
        })
        .unwrap_or_default();
    hero.gizmo.mode.publish(active, &modes);
    // 2. A rede de segurança: quem perdeu a entidade (por qualquer porta) volta a Object.
    if let Some(current) = hero.gizmo.mode.active() {
        let held = holds(current.mode, tools);
        if !hero.gizmo.mode.still_holds(
            hero.gizmo.selection,
            hero.gizmo.extra_selection.len(),
            held,
        ) {
            hero.gizmo.mode.leave();
            if held {
                leave(current.mode, tools);
            }
            changed = true;
        }
    }
    // 3. O pedido do quadro.
    if let Some(req) = request {
        let name = || {
            entity
                .and_then(|e| world.get::<ph2d_ecs::Name>(e))
                .map_or_else(String::new, |n| n.0.clone())
        };
        match hero.gizmo.mode.resolve(req) {
            Step::Stay => {}
            Step::Enter(m) => {
                if let Some(prev) = hero.gizmo.mode.leave() {
                    leave(prev.mode, tools);
                }
                if let Some(bits) = active
                    && enter(m, tools, hero)
                {
                    hero.gizmo.mode.enter(bits, m);
                    toasts.push(Toast::info(tr_with(
                        "object_mode.entered",
                        &[("mode", &m.label_key().tr())],
                    )));
                }
            }
            Step::Leave => {
                if let Some(prev) = hero.gizmo.mode.leave() {
                    leave(prev.mode, tools);
                }
                toasts.push(Toast::info(ph2d_i18n::tr("object_mode.left").to_string()));
            }
            Step::LeaveToDefault => {
                if let Some(prev) = hero.gizmo.mode.leave() {
                    leave(prev.mode, tools);
                }
                tools.activate_default();
            }
            Step::Refuse => {
                toasts.push(Toast::info(if active.is_none() {
                    ph2d_i18n::tr("object_mode.nothing_selected").to_string()
                } else {
                    tr_with("object_mode.only_object", &[("name", &name())])
                }));
            }
        }
        changed = true;
    }
    // 4. O seletor — em todo quadro, vazio incluído.
    let menu = hero.gizmo.mode.menu(&mut hero.store);
    hero.store.set_mode_menu(menu);
    changed
}

#[cfg(test)]
#[path = "fase_object_mode_tests.rs"]
mod tests;
