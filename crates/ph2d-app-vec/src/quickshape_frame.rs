//! **O QuickShape na shell** (F7): as duas portas que ela chama — o QUADRO (quem segura o relógio)
//! e o `Esc`. A lei mora em `ph2d_vec_edit::quickshape`, o gatilho em `ph2d_vec_edit::quickshape_hold`.

use crate::state::VecState;
use ph2d_vec_edit::pencil::ShapeEditState;
use ph2d_vec_edit::quickshape_hold::HoldTick;
use ph2d_vec_scene::VecScene;

/// A cada quadro com o lápis aberto: `now_ns` é o relógio dos carimbos do ponteiro, `shift` o
/// `regular` (o 2.º dedo do Procreate, que o toque já entrega como `Shift`). `true` = a forma
/// NASCEU agora (a shell toca o clique: foi a mão que segurou).
pub fn frame(vec: &mut VecState, scene: &mut VecScene, now_ns: u128, shift: bool) -> bool {
    shape_edit(vec, scene, shift);
    vec.pencil.is_active()
        && vec
            .pencil
            .hold_tick(scene, now_ns, shift, vec.draw_config.pencil_quickshape)
            == HoldTick::Recognised
}

/// ⭐ **O Edit Shape** (F7e) a cada quadro: o botão da ④ abre e fecha os nós; o `Shift` refaz o
/// arrasto ao vivo; a forma só vale com o lápis na mão, o caminho dela selecionado e a geometria
/// que ela escreveu (o afim é o que a caneta recebeu neste quadro, DEPOIS do assentamento); e a ④
/// e a dica ⑤ são TOLD.
fn shape_edit(vec: &mut VecState, scene: &mut VecScene, shift: bool) {
    if ph2d_panel_vector::take_shape_edit_toggle() {
        vec.pencil.toggle_shape_edit();
    }
    vec.pencil.shape_edit_regular(scene, shift);
    if let Some(id) = vec.pencil.shape_edit_path() {
        let in_hand = vec.draw_config.mode == ph2d_tool_vector::DrawMode::Pencil;
        let xf = vec.pen.xf(id);
        vec.pencil
            .shape_edit_tick(scene, xf, vec.pen.selected(), in_hand);
    }
    let state = vec.pencil.shape_edit_state();
    ph2d_panel_vector::set_current_shape_edit(
        state != ShapeEditState::None,
        state == ShapeEditState::Open,
    );
}

/// O `Esc` com o lápis aberto: com a forma, devolve o traço livre; sem ela, desiste do traço (o
/// que a dica do Lápis sempre prometeu e ninguém fazia). Com os nós do Edit Shape à vista e sem
/// arrasto, fecha-os. `true` = consumido.
pub fn escape(vec: &mut VecState, scene: &mut VecScene) -> bool {
    if !vec.pencil.is_active() {
        return vec.pencil.shape_edit_escape();
    }
    if !vec.pencil.quick_dismiss(scene) {
        vec.pencil.cancel(scene);
    }
    true
}

#[cfg(test)]
#[path = "quickshape_frame_edit_tests.rs"]
mod edit_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn the_hold_is_the_same_number_as_the_touch_long_press() {
        // Duas casas para o MESMO meio segundo (a crate do lápis não vê a fundação): este gate é a
        // costura que as impede de divergir.
        assert_eq!(
            ph2d_vec_edit::quickshape_hold::HOLD_NS,
            ph2d_editor_core::interaction::touch::LONG_PRESS_NS
        );
    }

    #[test]
    fn the_hold_slop_is_the_same_number_as_the_touch_slop() {
        assert_eq!(
            ph2d_vec_edit::quickshape_hold::HOLD_SLOP_PX,
            f64::from(ph2d_editor_core::interaction::touch::SLOP_PT)
        );
    }

    #[test]
    fn the_shift_reaches_the_hold_as_regular() {
        let mut vec = crate::state::VecState::default();
        let mut scene = ph2d_vec_scene::VecScene::default();
        let d = |i: u128| ph2d_vec_edit::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: i * 4_000_000,
        };
        let corners = [
            [0.0, 0.0],
            [400.0, 0.0],
            [400.0, 250.0],
            [0.0, 250.0],
            [0.0, 0.0],
        ];
        let mut n = 0u128;
        vec.pencil.on_press(&mut scene, corners[0], 1.0, d(0));
        for w in corners.windows(2) {
            for k in 1..=50 {
                let u = f64::from(k) / 50.0;
                n += 1;
                let p = [
                    w[0][0] + (w[1][0] - w[0][0]) * u,
                    w[0][1] + (w[1][1] - w[0][1]) * u,
                ];
                vec.pencil.on_drag(&mut scene, p, d(n));
            }
        }
        let held = n * 4_000_000 + ph2d_vec_edit::quickshape_hold::HOLD_NS;
        assert!(super::frame(&mut vec, &mut scene, held, true));
        let id = vec.pencil.active_path().expect("o traço vivo");
        let v: Vec<[f64; 2]> = scene
            .path(id)
            .expect("o caminho")
            .verts
            .iter()
            .map(|v| v.anchor)
            .collect();
        let side = |i: usize| (v[i][0] - v[(i + 1) % 4][0]).hypot(v[i][1] - v[(i + 1) % 4][1]);
        assert!(
            (side(0) - side(1)).abs() < 1e-6,
            "o Shift faz o QUADRADO: {v:?}"
        );
    }

    #[test]
    fn esc_gives_back_the_free_stroke_then_cancels_it() {
        let mut vec = crate::state::VecState::default();
        let mut scene = ph2d_vec_scene::VecScene::default();
        let d = |i: u128| ph2d_vec_edit::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: i * 4_000_000,
        };
        let pts: Vec<[f64; 2]> = (0..=100)
            .map(|i| {
                let a = std::f64::consts::TAU * f64::from(i) / 100.0;
                [200.0 * a.cos(), 200.0 * a.sin()]
            })
            .collect();
        vec.pencil.on_press(&mut scene, pts[0], 1.0, d(0));
        for (i, &p) in pts.iter().enumerate().skip(1) {
            vec.pencil.on_drag(&mut scene, p, d(i as u128));
        }
        let held = 100 * 4_000_000 + ph2d_vec_edit::quickshape_hold::HOLD_NS;
        vec.draw_config.pencil_quickshape = false;
        assert!(
            !super::frame(&mut vec, &mut scene, held, false),
            "o interruptor desligado manda"
        );
        vec.draw_config.pencil_quickshape = true;
        assert!(
            super::frame(&mut vec, &mut scene, held + 1, false),
            "segurou ⇒ círculo"
        );
        assert!(
            super::escape(&mut vec, &mut scene) && vec.pencil.is_active() && !vec.pencil.is_quick()
        );
        assert!(
            super::escape(&mut vec, &mut scene) && !vec.pencil.is_active(),
            "o 2.º Esc desiste"
        );
        assert!(
            !super::escape(&mut vec, &mut scene),
            "sem traço o Esc é de outro"
        );
    }
}
