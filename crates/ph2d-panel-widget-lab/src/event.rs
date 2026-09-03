//! O despacho do laboratório.
//!
//! ⚠️ **Cada braço aqui é o CONSUMIDOR de um chip da bancada**, e é isso que os impede de serem
//! knobs mortos — a espécie que o `CLAUDE.md` §5.0 descreve e que esta linha apanhou no trilho de
//! ferramentas. O gate `every_lab_control_moves_the_study` prova-o por mutação.
//!
//! ⭐ **E os três primeiros chips mudam o APP, não a bancada:** eles escrevem no
//! [`SliderStyle`](ph2d_tokens::SliderStyle), que o `paint` publica por quadro.

// ⚠️ Os comprimentos vêm das TABELAS de `ph2d-tokens` e do `study`, nunca de constantes escritas
// aqui. A 1.ª redacção deste ficheiro declarava `ACCENT_COUNT = 6` a "espelhar" a tabela, com um
// gate a provar que os dois números concordavam. ⛔ Um gate que compara duas cópias é a admissão de
// que há duas cópias. *A cura de um espelho não é vigiá-lo — é não o ter.*

use crate::WidgetLabPanel;
use crate::state::WidgetLabState;
use crate::study::ACCENTS;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_tokens::{SLIDER_DENSITIES, SLIDER_RADII};

/// Anda uma posição numa tabela, dando a volta. ⚠️ Devolve o **elemento**, não o índice: guardar o
/// índice foi o que fez a 1.ª versão precisar de contadores espelhados.
fn cycle<T: Copy + PartialEq>(table: &[T], current: T) -> T {
    let i = table.iter().position(|x| *x == current).unwrap_or(0);
    table[(i + 1) % table.len()]
}

pub(crate) fn apply_event(
    state: &mut WidgetLabState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let WidgetEvent::Click(id) = ev else {
        return EventOutcome::Ignored;
    };
    if id == ids::TOPBAR_WIDGET_LAB || id == ids::LAB_CLOSE {
        let next = !host.panel_visible(WidgetLabPanel::ID);
        host.set_panel_visible(WidgetLabPanel::ID, next);
        return EventOutcome::Consumed;
    }
    if id == ids::LAB_VARIANT_NEXT {
        state.style.design = state.style.design.next();
    } else if id == ids::LAB_VARIANT_PREV {
        state.style.design = state.style.design.prev();
    } else if id == ids::LAB_RADIUS_CYCLE {
        state.style.radius = cycle(&SLIDER_RADII, state.style.radius);
    } else if id == ids::LAB_DENSITY_CYCLE {
        state.style.density = cycle(&SLIDER_DENSITIES, state.style.density);
    } else if id == ids::LAB_ACCENT_CYCLE {
        state.accent = (state.accent + 1) % ACCENTS.len();
    } else if id == ids::LAB_DECORATOR_TOGGLE {
        state.decorator = !state.decorator;
    } else if id == ids::LAB_COMPARE_TOGGLE {
        state.compare = !state.compare;
    } else {
        return EventOutcome::Ignored;
    }
    EventOutcome::Consumed
}
