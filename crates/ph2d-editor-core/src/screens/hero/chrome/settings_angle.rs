// ph2d-chrome-sync:z=101 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → **Angle unit** cascade: open submenu + pick Degrees/Radians.
//!
//! # Por que é um ficheiro próprio, irmão do [`super::settings_unit`]
//!
//! Aquele responde *"em que unidade leio um COMPRIMENTO?"*; este, *"…e um ÂNGULO?"*.
//! São duas perguntas independentes — a primeira segue a escala do projecto
//! (`pixels_per_meter`), a segunda segue o hábito do artista — e um artista de pixel
//! art em `Pixels` pode querer graus tanto quanto radianos. ⛔ Juntá-las num
//! selector só obrigaria a um submenu de dois níveis para duas escolhas binárias.
//!
//! # ⚠️ O armazenamento NÃO muda
//!
//! O `ph2d_ecs::Transform` guarda **radianos** (*"rotation (radians, CCW from +X)"*)
//! e toda a matemática os assume. Isto escreve `project.display_angle`, que só é
//! lido na **fronteira de pintura/comissão** — exactamente como o `display_unit`.
//!
//! Pedido do Enio, 2026-08-30: *"devemos ter ambas as opções no app (px e metros,
//! graus e radianos)"*.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::project::DisplayAngle;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_ANGLE {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsAngleSubmenu,
        });
        return true;
    }
    // Angle-unit submenu options — write to `project.display_angle` and close the
    // menu. Inspector / panel angle rows read the project setting on the next paint.
    let pick = if id == ids::CTX_MENU_ANGLE_DEGREES {
        DisplayAngle::Degrees
    } else if id == ids::CTX_MENU_ANGLE_RADIANS {
        DisplayAngle::Radians
    } else {
        return false;
    };
    hero.project.display_angle = pick;
    hero.store.close_context_menu();
    true
}
