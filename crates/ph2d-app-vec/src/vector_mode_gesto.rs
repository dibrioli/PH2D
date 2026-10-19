//! ⭐ **A cena 35** (`PH2D_OBJECT_MODE_SMOKE=35`, F22 — dono 19/10): O GESTO VENCE O SHOW
//! (`docs/UI_New_and_Simple/vetor_nova_ui/33_plano_F22_o_gesto_vence_a_maquina.md` §7). Começa pela
//! 25 inteira (o botão azul feito componente, Default azul e Hover laranja gravados, o Show do Default
//! assente), põe a Duration no máximo (2 s), pede o Show do Hover e, A MEIO da transição, escolhe
//! verde no Fill pelo seletor ⇒ o botão fica verde, parado onde estava, e nenhum papel acende.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_vector::ids;
use ph2d_vec_scene::{Paint, Rgba8, VecPathId};

const AZUL: Rgba8 = Rgba8::new(40, 120, 240, 255);
pub(crate) const VERDE: Rgba8 = Rgba8::new(40, 200, 90, 255);
const PAPEL_HOVER: usize = 1;
/// O vermelho do meio do caminho azul (40) → laranja (240): a transição está NO AR.
const MEIO_R: std::ops::RangeInclusive<u8> = 120..=200;

/// O primeiro estágio — no buraco entre a cena 9 (`20..=21`) e a 10 (`30..`).
pub(crate) const FIRST: u8 = 22;
const SHOW: u8 = FIRST + 1;
pub(crate) const LAST: u8 = SHOW + 1;

fn so_um(vec: &VecState) -> Option<VecPathId> {
    match vec.pen.selected_paths() {
        [only] => Some(*only),
        _ => None,
    }
}

fn cor(scene: &ph2d_vec_scene::VecScene, id: VecPathId) -> Option<Rgba8> {
    match scene.path(id)?.fill {
        Some(Paint::Solid(c)) => Some(c),
        _ => None,
    }
}

/// Um passo da cena 35; devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let botao = so_um(vec)?;
    match stage {
        // A Duration no máximo pelo trilho do painel, depois de o Show do Default da 25 assentar.
        FIRST if cor(scene, botao) == Some(AZUL) => {
            let id = ids::VECTOR_STATE_DURATION;
            if let Some(ph2d_editor_core::InteractiveState::Slider { value, .. }) =
                hero.store.get_mut(id)
            {
                *value = 1.0;
            }
            hero.apply_event(WidgetEvent::ValueChanged(id));
            Some(SHOW)
        }
        SHOW if ph2d_panel_vector::state::ui_states_state()
            .is_some_and(|s| s.duration_s >= ph2d_ui_state::MAX_DURATION_S as f32 - 1e-3) =>
        {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(PAPEL_HOVER)));
            Some(LAST)
        }
        // A meio do voo, o artista escolhe verde no Fill (o seletor aberto na amostra e tocado).
        LAST if cor(scene, botao).is_some_and(|c| MEIO_R.contains(&c.r)) => {
            let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
            hero.store
                .open_picker_on(ph2d_tool_vector::ids::VECTOR_FILL_SWATCH);
            hero.store.set_blender_value(
                picker,
                ph2d_tokens::ColorValue::from_rgba8(VERDE.r, VERDE.g, VERDE.b, VERDE.a),
            );
            hero.store.note_colour_pick(picker);
            Some(9)
        }
        _ => None,
    }
}
