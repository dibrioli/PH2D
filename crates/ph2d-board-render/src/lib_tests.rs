use super::{dot_grid, grid_step_world};
use ph2d_board_model::Camera;
use ph2d_tokens::Spacing;
use ph2d_vector::{PathEl, Shape};

const AREA: [f64; 4] = [0.0, 0.0, 1600.0, 900.0];

fn dots(path: &ph2d_vector::BezPath) -> usize {
    path.elements()
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_)))
        .count()
}

#[test]
fn the_screen_spacing_stays_in_one_octave_at_every_zoom() {
    let min = f64::from(Spacing::Xl.px());
    for zoom in [1e-4, 0.01, 0.3, 1.0, 1.7, 40.0, 1e4] {
        let cam = Camera {
            center_x: 0.0,
            center_y: 0.0,
            zoom,
        };
        let px = grid_step_world(&cam) * zoom;
        assert!(px >= min && px < 2.0 * min, "zoom {zoom}: {px} px");
    }
}

#[test]
fn the_dot_count_is_bounded_by_the_area_not_by_the_zoom() {
    let min = f64::from(Spacing::Xl.px());
    let bound = ((AREA[2] / min + 2.0) * (AREA[3] / min + 2.0)) as usize;
    for zoom in [1e-4, 0.05, 1.0, 7.0, 1e4] {
        let cam = Camera {
            center_x: 12_345.0,
            center_y: -678.0,
            zoom,
        };
        let n = dots(&dot_grid(&cam, AREA));
        assert!(
            n > 0 && n <= bound,
            "zoom {zoom}: {n} pontos (tecto {bound})"
        );
    }
}

#[test]
fn dots_sit_on_world_multiples_of_the_step() {
    let cam = Camera {
        center_x: 3.0,
        center_y: 5.0,
        zoom: 1.0,
    };
    let step = grid_step_world(&cam);
    let path = dot_grid(&cam, AREA);
    let first = path.bounding_box();
    let w = cam.to_world(AREA, [first.x0 + 1.0, first.y0 + 1.0]);
    assert!((w[0] / step - (w[0] / step).round()).abs() < 1e-9);
    assert!((w[1] / step - (w[1] / step).round()).abs() < 1e-9);
}
