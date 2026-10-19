//! ⭐ **A cena 37** (`PH2D_OBJECT_MODE_SMOKE=37`, F22b — dono 19/10: *«passe a animar a
//! opacidade»*; `docs/UI_New_and_Simple/vetor_nova_ui/33_plano_F22_o_gesto_vence_a_maquina.md` §10).
//! Começa pela 25 inteira (o botão azul feito componente, Default e Hover gravados, o Show do Default
//! assente), regrava o Hover LARANJA a 30 %, mostra o Default (azul, 100 %) e depois o Hover ⇒ o botão
//! desvanece até 30 % e o Opacity do painel diz 30.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_vector::ids;
use ph2d_vec_scene::{Paint, Rgba8, VecPathId};

const AZUL: Rgba8 = Rgba8::new(40, 120, 240, 255);
const LARANJA: Rgba8 = Rgba8::new(240, 140, 40, 255);
/// A opacidade do Hover regravado.
pub(crate) const HOVER_OPACIDADE: f32 = 0.3;
const DEFAULT: usize = 0;
const PAPEL_HOVER: usize = 1;

/// O primeiro estágio — no buraco entre a 35 (`22..=24`) e a cena 10 (`30..`).
pub(crate) const FIRST: u8 = 25;
const DEFAULT_DE_VOLTA: u8 = FIRST + 1;
pub(crate) const LAST: u8 = DEFAULT_DE_VOLTA + 1;
const _: () = assert!(crate::vector_mode_gesto::LAST < FIRST);
const _: () = assert!(LAST < 30);

fn so_um(vec: &VecState) -> Option<VecPathId> {
    match vec.pen.selected_paths() {
        [only] => Some(*only),
        _ => None,
    }
}

/// A forma assente nesta cor e nesta opacidade (o fim de uma transição, nunca o meio).
fn assente(scene: &ph2d_vec_scene::VecScene, id: VecPathId, c: Rgba8, a: f32) -> bool {
    scene
        .path(id)
        .is_some_and(|p| p.fill == Some(Paint::Solid(c)) && (p.opacity.get() - a).abs() < 1e-6)
}

/// Um passo da cena 37; devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let botao = so_um(vec)?;
    match stage {
        // O Hover regravado laranja a 30 %, depois de o Show do Default da 25 assentar.
        FIRST if assente(scene, botao, AZUL, 1.0) => {
            scene.path_mut(botao)?.fill = Some(Paint::Solid(LARANJA));
            crate::appearance::set_opacity(scene, &[botao], HOVER_OPACIDADE);
            hero.apply_event(WidgetEvent::Click(ids::vector_state_record_id(PAPEL_HOVER)));
            Some(DEFAULT_DE_VOLTA)
        }
        DEFAULT_DE_VOLTA => {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(DEFAULT)));
            Some(LAST)
        }
        // Só depois de o Default ASSENTAR (azul a 100 %): o Show do Hover parte daí.
        LAST if assente(scene, botao, AZUL, 1.0) => {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(PAPEL_HOVER)));
            Some(9)
        }
        _ => None,
    }
}
