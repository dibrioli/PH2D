//! ⭐ **A cena 17** (`PH2D_OBJECT_MODE_SMOKE=17`, F11 — 11/10): A CAIXA POR FORA DO TRAÇO. No layout
//! *Vetor*, um rectângulo arredondado de traço GROSSO; o botão *Forma*; e, pelo MESMO press e arrasto
//! da shell, o ⊙ do canto de cima à esquerda levado na DIAGONAL para dentro — os quatro cantos
//! arredondam e cada ⊙ fica no centro do arco dele (o *corner widget* do Illustrator). A foto final
//! mostra a caixa de transformação POR FORA do traço, os quatro ⊙, o ⇕ do *Corner* na borda e a ④ à
//! folga nova.

use crate::state::VecState;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_vec_scene::ShapeKind;

/// O rectângulo: os cantos oposto (m) — largo, no meio da vista de omissão.
const RECT: ([f64; 2], [f64; 2]) = ([-2.0, -1.2], [2.0, 1.2]);

/// O traço GROSSO (px), escrito no campo *Width* do painel com a forma seleccionada: a caixa fica
/// 10 px por fora da geometria.
const TRACO_PX: f64 = 20.0;

/// Quanto a mão leva o ⊙ para dentro, em cada eixo (m): o raio dos quatro cantos.
const RAIO: f64 = 0.45;

/// Um passo da cena 17; devolve o próximo estágio, se mudou. `90` = o rectângulo · `91` = o traço
/// grosso pelo *Width* · `92` = o modo Forma pelo botão · `93` = o ⊙ na diagonal: os cantos
/// arredondam · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    match stage {
        90 if vec.entities.is_empty() => {
            let k = ShapeKind::RoundRect;
            let mut v = k.defaults();
            v[0] = 0.0;
            let c = ph2d_vec_edit::ShapeConstraint::default();
            vec.shape.on_press(scene, k, v, RECT.0, 0.01, c);
            vec.shape.on_drag(scene, RECT.1, c);
            vec.shape.on_release(scene);
            Some(91)
        }
        // A largura pelo MESMO evento do campo *Width* (restila a forma seleccionada).
        91 if vec.entities.len() == 1 => {
            let id = *vec.entities.keys().next()?;
            vec.pen.select_many(&[id]);
            // O que o campo guarda quando se escreve nele (o painel lê-o antes da ficha da tool).
            let track = ph2d_tool_vector::params::px_to_slider(TRACO_PX);
            hero.store
                .set_slider_value(ph2d_tool_vector::ids::VECTOR_WIDTH, track);
            hero.store
                .set_number_value(ph2d_tool_vector::ids::VECTOR_WIDTH_NUM, TRACO_PX);
            hero.bus
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    ph2d_tool_vector::ids::VECTOR_WIDTH,
                    f64::from(track),
                )));
            Some(92)
        }
        92 => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
            Some(93)
        }
        // O ⊙ de cima à esquerda, na diagonal para dentro (para a direita e para baixo).
        93 if vec.draw_config.mode == ph2d_tool_vector::DrawMode::Shape => {
            let walk = |a: [f64; 2]| -> Vec<[f64; 2]> {
                (1..=6)
                    .map(|k| {
                        let t = RAIO * f64::from(k) / 6.0;
                        [a[0] + t, a[1] - t]
                    })
                    .collect()
            };
            crate::vector_mode_estrela::pull(vec, scene, sim, 0, walk).then_some(9)
        }
        _ => None,
    }
}
