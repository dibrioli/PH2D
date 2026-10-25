//! ⭐ **A cena 39** (`PH2D_OBJECT_MODE_SMOKE=39`, F30 §E — report do dono 25/10): O FIO DO CAMINHO.
//! A Caneta a meio de um caminho de três pontos com traço de largura 0 e sem preenchimento: o que se
//! vê é o fio de 1 px no acento (e o segmento até ao cursor), por baixo dos nós.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_scene::{Rgba8, StrokeSpec};

const PONTOS: [[f64; 2]; 3] = [[-1.5, -0.6], [0.0, 0.8], [1.5, -0.6]];

/// O primeiro estágio — no buraco entre a cena 38 (`42`) e a 12 (`50..`).
pub(crate) const FIRST: u8 = 43;
pub(crate) const LAST: u8 = FIRST + 1;
const _: () = assert!(LAST < 50);

/// Um passo da cena 39; devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    match stage {
        FIRST => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_PEN));
            Some(LAST)
        }
        LAST if vec.draw_config.mode == DrawMode::Pen => {
            for (i, p) in PONTOS.into_iter().enumerate() {
                vec.pen.on_press(scene, p, 0.01, false, &mut |q| q);
                vec.pen.on_release();
                if i == 0 {
                    let id = vec.pen.active_path()?;
                    let path = scene.path_mut(id)?;
                    path.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 0), 0.0));
                    path.fill = None;
                }
            }
            Some(9)
        }
        _ => None,
    }
}
