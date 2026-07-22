//! The contact marks: where they sit, and what their size means.

use super::*;
use ph2d_ecs::Entity;

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

fn contact(point: [f32; 2], impulse: f32) -> BodyContact {
    BodyContact {
        a: Entity::from_bits(1),
        b: Entity::from_bits(2),
        point,
        impulse,
    }
}

/// Half the width of a mark's horizontal arm, in screen px — the size the load maps
/// to. Read off the path so the test measures what is DRAWN, not what the constant
/// says.
fn arm_px(path: &BezPath) -> f64 {
    let pts: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    (pts[1].x - pts[0].x) / 2.0
}

#[test]
fn a_heavier_load_draws_a_bigger_mark() {
    // The size IS the reading. A stack shows a gradient because the bottom joint
    // carries everything above it, and if every mark were the same size the overlay
    // would be answering "are these touching" while pretending to answer "how hard".
    let light = &contact_marks(true, &[contact([0.0, 0.0], 0.005)], &camera(), window())[0];
    let heavy = &contact_marks(true, &[contact([0.0, 0.0], 0.045)], &camera(), window())[0];
    assert!(
        arm_px(heavy) > arm_px(light) * 1.5,
        "a 9x load should draw a visibly bigger mark ({} vs {})",
        arm_px(heavy),
        arm_px(light)
    );
    // And it saturates rather than growing without bound — past the ruler, "very
    // loaded" is the useful reading and a mark the size of the screen is not.
    let huge = &contact_marks(true, &[contact([0.0, 0.0], 500.0)], &camera(), window())[0];
    assert!(
        (arm_px(huge) - arm_px(heavy)).abs() < 1.0,
        "the mark must saturate at the ruler, got {} vs {}",
        arm_px(huge),
        arm_px(heavy)
    );
}

#[test]
fn the_mark_sits_on_the_contact_point_in_screen_space() {
    // A camera 10 world units tall over a 1000 px window is 100 px per unit, and the
    // centre of the world is the centre of the screen. A contact 2 units to the right
    // is therefore 200 px right of centre — the arithmetic a mark drawn at the body's
    // centre, or in world units, would get wrong.
    let marks = contact_marks(true, &[contact([2.0, 0.0], 0.01)], &camera(), window());
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    let cx = (pts[0].x + pts[1].x) / 2.0;
    let cy = (pts[2].y + pts[3].y) / 2.0;
    assert!(
        (cx - 700.0).abs() < 1.0 && (cy - 500.0).abs() < 1.0,
        "the mark should be centred at (700, 500) px, got ({cx}, {cy})"
    );
}

#[test]
fn the_toggle_switches_the_marks_off() {
    assert!(
        contact_marks(false, &[contact([0.0, 0.0], 0.01)], &camera(), window()).is_empty(),
        "a painter or vector user must never see physics chrome over their artwork"
    );
    assert!(
        contact_marks(true, &[], &camera(), window()).is_empty(),
        "a scene where nothing touches draws nothing"
    );
}

#[test]
fn the_waterline_is_drawn_across_the_pool_and_obeys_the_toggle() {
    // A metade visível do empuxo. A geometria vem PRONTA da física, então o que este
    // gate cobre é a projeção: a linha tem de sair na altura certa da tela, atravessar a
    // poça, e sumir com o toggle.
    //
    // Câmera de 10 unidades de mundo sobre 1000 px = 100 px por unidade, centro do mundo
    // no centro da tela. Uma superfície em y = 0 cai em 500 px; de x = −3,5 a +3,5 vira
    // 150..850 px, mais a sobra de 6 px em cada ponta.
    let marks = waterline_marks(true, &[([-3.5, 0.0], [3.5, 0.0])], &camera(), window());
    assert_eq!(marks.len(), 1);
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    assert!(
        (pts[0].y - 500.0).abs() < 1.0 && (pts[1].y - 500.0).abs() < 1.0,
        "a superfície em y = 0 tem de cair no meio da tela, e caiu em {pts:?}"
    );
    assert!(
        (pts[0].x - 144.0).abs() < 1.0 && (pts[1].x - 856.0).abs() < 1.0,
        "a linha tem de atravessar a poça inteira mais a sobra de 6 px, e ficou em {pts:?}"
    );

    assert!(
        waterline_marks(false, &[([-1.0, 0.0], [1.0, 0.0])], &camera(), window()).is_empty(),
        "a linha d'água obedece ao mesmo toggle que o resto do chrome de física"
    );
    assert!(
        waterline_marks(true, &[], &camera(), window()).is_empty(),
        "uma cena sem poça não desenha nada"
    );
}

#[test]
fn a_tilted_pool_draws_a_level_waterline() {
    // ⚠️ O gate que exige que a linha venha da FÍSICA e não do topo da caixa. Água é
    // horizontal mesmo numa poça torta, e é isso que a `surface_level` (perpendicular à
    // gravidade) devolve — um overlay que desenhasse a aresta de cima do collider daria
    // uma linha INCLINADA, e as duas só discordam aqui.
    //
    // A entrada é o que a física de fato produz para uma poça rotacionada: dois pontos
    // na MESMA altura, mas afastados do centro.
    let marks = waterline_marks(true, &[([-2.0, 1.4], [2.6, 1.4])], &camera(), window());
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    assert!(
        (pts[0].y - pts[1].y).abs() < 1e-6,
        "os dois extremos têm de sair na MESMA altura de tela ({pts:?})"
    );
}
