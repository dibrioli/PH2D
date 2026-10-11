//! ⭐ **A cena 18** (`PH2D_OBJECT_MODE_SMOKE=18`, F12 A — 11/10): O TRAÇO DAS FORMAS, as das fotos
//! do dono. No layout *Vetor*, com traço grosso: a seta dobrada e o coração, cada um numa caixa larga
//! e numa alta e estreita (o bico que saltava para o chanfro), o quadrado com *Curve* `0,010` e o pentágono com
//! *Curve* `0,070` (o «X» e as abas), a lua e o balão (as pontas que voltam 180°), e o rectângulo de
//! raio minúsculo (o canto que a placa chanfrava). Correr com e
//! sem `PH2D_MITER_CLIP=0` é o antes × depois do bico cortado. Doc 19.

use crate::state::VecState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_vec_scene::ShapeKind;

/// O traço grosso (px), pelo campo *Width* com as formas seleccionadas.
const TRACO_PX: f64 = 16.0;

/// Cada forma: o tipo, os campos que mudam do omissão, e os cantos opostos da caixa (m).
type Forma = (ShapeKind, &'static [(usize, f64)], [f64; 2], [f64; 2]);

const FORMAS: &[Forma] = &[
    (ShapeKind::ArrowBent, &[], [-3.0, 0.2], [-1.9, 1.2]),
    (ShapeKind::ArrowBent, &[], [-2.9, -1.8], [-2.3, -0.1]),
    (ShapeKind::Heart, &[(0, 0.10)], [-1.6, 0.2], [-0.5, 1.2]),
    (ShapeKind::Heart, &[(0, 0.30)], [-1.4, -1.8], [-0.8, -0.1]),
    (ShapeKind::Polygon, &[(0, 4.0), (3, 0.010)], [-0.2, 0.2], [0.8, 1.2]),
    (ShapeKind::Polygon, &[(0, 5.0), (3, 0.070)], [-0.2, -1.4], [0.8, -0.4]),
    (ShapeKind::Moon, &[], [1.1, 0.2], [2.0, 1.2]),
    (ShapeKind::SpeechRect, &[], [1.0, -1.4], [2.4, -0.4]),
    // O rectângulo de raio minúsculo (a foto do dono de 11/10: o canto saía em chanfro).
    (ShapeKind::RoundRect, &[(0, 0.001)], [2.4, 0.2], [3.2, 1.2]),
];

/// Um passo da cena 18; devolve o próximo estágio, se mudou. `100` = as formas · `101` = o traço
/// grosso pelo *Width* · `102` = um quadro para o barramento restilar a selecção · `103` = nada
/// seleccionado (a foto sem caixas) · `9` = feito.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<u8> {
    match stage {
        100 if vec.entities.is_empty() => {
            let c = ph2d_vec_edit::ShapeConstraint::default();
            for (k, sets, a, b) in FORMAS {
                let mut v = k.defaults();
                for (i, x) in *sets {
                    v[*i] = *x;
                }
                vec.shape.on_press(scene, *k, v, *a, 0.01, c);
                vec.shape.on_drag(scene, *b, c);
                vec.shape.on_release(scene);
            }
            Some(101)
        }
        101 if vec.entities.len() == FORMAS.len() => {
            let ids: Vec<_> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&ids);
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
            Some(102)
        }
        102 => Some(103),
        103 => {
            vec.pen.select_many(&[]);
            Some(9)
        }
        _ => None,
    }
}
