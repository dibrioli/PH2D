//! ⭐⭐ **IMAGE ▸ PAINT** — o modo que esta família declara e como ele abre e larga o Painter sobre a
//! imagem activa (spec/06 F2 + a F3 da Imagem; D6: *imagem → Object · Paint*).
//!
//! O Painter já é «o desta entidade» (`PainterTool::bound_doc` = a selecção): entrar no modo é pôr
//! o Painter em mãos com a selecção colapsada ao activo (`screens::hero::mode_drive`), e o cadeado
//! da selecção (`ph2d_editor_core::object_mode::decide`) guarda-a lá enquanto o modo durar.
//!
//! ⚠️ **A porta do IMG não se aplica aqui:** o modo É a porta do Painter. O botão dele saiu da
//! barra IMG na mesma fase (dois caminhos para o mesmo módulo divergem — spec/06 §5).

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_painter::PainterTool;

/// ⭐ **O modo que esta família declara, e as portas dele** — a shell junta-a às outras.
pub const FAMILY: ModeFamily = ModeFamily {
    modes: &[(ObjectKind::Image, ObjectMode::Paint)],
    holds: |_, tools| holds_an_image(tools),
    enter: |_, tools| enter(tools),
    leave: |_, tools| leave(tools),
};

/// O id do Painter no registo de ferramentas.
const PAINTER: &str = "painter";

/// O Painter está em mãos **sobre uma imagem** — e não sobre a tela da peça 3D, que é a pintura
/// da escultura (`ph2d_app_sculpt3d::painter_na_malha`, o Paint do Sculpt na F3).
#[must_use]
pub fn holds_an_image(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .is_some_and(|p| !p.on_screen_canvas())
}

/// ⭐ **Entra**: põe o Painter em mãos — o quadro do modo já colapsou a selecção ao activo, que é o
/// documento que ele abre. Devolve `false` se o registo recusou.
pub fn enter(tools: &mut ToolRegistry) -> bool {
    holds_an_image(tools) || tools.set_active(&ToolId::new(PAINTER))
}

/// **Larga**: devolve o canvas à ferramenta de omissão, se o Painter ainda o tem.
pub fn leave(tools: &mut ToolRegistry) {
    if tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new(PAINTER))
    {
        tools.activate_default();
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=1` selecciona a 1.ª imagem da cena e abre o
/// seletor *Mode* sobre ela; `=2` entra logo em *Paint Mode*. É como a foto do passo do smoke o
/// apanha, já que o clique sintético não chega à tela virtual
/// (`docs/Components/ferramentas/fotografa_cena.sh`). Corre uma vez; devolve o pedido a drenar.
pub fn smoke_step(
    sim: &mut ph2d_ecs::SimWorld,
    hero: &mut HeroScreen,
) -> Option<ph2d_editor_core::object_mode::ModeRequest> {
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = abrir o seletor · 2 = entrar em Paint · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    static REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("1") => 1,
                Ok("2") => 2,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    if stage == 9 {
        return None;
    }
    if hero.gizmo.selection.is_none() {
        let first = sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &ph2d_render::Sprite)>()
            .iter(sim.world())
            .map(|(e, _)| e.to_bits())
            .min();
        if first.is_some() {
            hero.gizmo.replace_selection(first);
        } else if !REQUESTED.swap(true, Ordering::Relaxed) {
            // A cena de arranque é vazia: a imagem nasce pela porta do `Ctrl+N`, em branco.
            hero.store.set_new_image_bg(2);
            hero.store.request_new_image();
        }
        return None;
    }
    if stage == 2 {
        STAGE.store(9, Ordering::Relaxed);
        return Some(ph2d_editor_core::object_mode::ModeRequest::Enter(
            ObjectMode::Paint,
        ));
    }
    // O chip só se abre depois de pintado (o pulldown ancora-se no rect dele).
    let chip = ph2d_editor_core::ids::area_menu_button(0);
    if hero.hit_index.rect_for(chip).is_some() {
        STAGE.store(9, Ordering::Relaxed);
        hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(chip));
    }
    None
}

#[cfg(test)]
#[path = "paint_mode_tests.rs"]
mod tests;
