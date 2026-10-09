//! **O QuickShape na shell** (F7): as duas portas que ela chama — o QUADRO (quem segura o relógio)
//! e o `Esc`. A lei mora em `ph2d_vec_edit::quickshape`, o gatilho em `ph2d_vec_edit::quickshape_hold`.

use crate::state::VecState;
use ph2d_vec_edit::quickshape_hold::HoldTick;
use ph2d_vec_scene::VecScene;

/// A cada quadro com o lápis aberto: `now_ns` é o relógio dos carimbos do ponteiro, `shift` o
/// `regular` (o 2.º dedo do Procreate, que o toque já entrega como `Shift`). `true` = a forma
/// NASCEU agora (a shell toca o clique: foi a mão que segurou).
pub fn frame(vec: &mut VecState, scene: &mut VecScene, now_ns: u128, shift: bool) -> bool {
    vec.pencil.is_active()
        && vec
            .pencil
            .hold_tick(scene, now_ns, shift, vec.draw_config.pencil_quickshape)
            == HoldTick::Recognised
}

/// O `Esc` com o lápis aberto: com a forma, devolve o traço livre; sem ela, desiste do traço (o
/// que a dica do Lápis sempre prometeu e ninguém fazia). `true` = consumido.
pub fn escape(vec: &mut VecState, scene: &mut VecScene) -> bool {
    if !vec.pencil.is_active() {
        return false;
    }
    if !vec.pencil.quick_dismiss(scene) {
        vec.pencil.cancel(scene);
    }
    true
}

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
