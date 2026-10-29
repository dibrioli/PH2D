//! O gate da moldura da câmera (plano GEF 17 A2) sobre o nível 2 montado e o passe da câmera dos componentes.

use ph2d_app_game::moldura::*;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::{GameFormat, GamePreset};

const DT: f64 = 1.0 / 60.0;

fn nivel_2() -> SimWorld {
    let mut sim = SimWorld::new();
    let _ = crate::comp::montar(&mut sim, 2);
    sim
}

fn formato(p: GamePreset) -> GameFormat {
    GameFormat {
        preset: p,
        ..Default::default()
    }
}

/// O mapa mundo→pixel de uma câmara que cobre a janela (a mesma conta do `Camera2d`).
fn px(c: ([f32; 2], f32), janela: [f32; 2], p: [f32; 2]) -> [f32; 2] {
    let s = c.1 / janela[1];
    [
        janela[0] * 0.5 + (p[0] - c.0[0]) / s,
        janela[1] * 0.5 - (p[1] - c.0[1]) / s,
    ]
}

/// ⭐⭐ **A moldura é o rectângulo que o jogo mostra**, em dois formatos: a da câmera activa tem o centro da vista
/// do quadro, a proporção do formato, e com o «Look through» os cantos dela caem nos cantos da caixa do jogo.
#[test]
fn a_moldura_e_o_rectangulo_que_o_jogo_mostra() {
    let (janela, canvas) = ([1600.0, 900.0], [300.0, 60.0, 960.0, 700.0]);
    for p in [GamePreset::Wide16x9, GamePreset::GameBoy] {
        let f = formato(p);
        let mut sim = nivel_2();
        let (vista, _) =
            ph2d_app_components::camera_2d::update(&mut sim, f.aspect(), 1, DT, [0.0, 0.0]);
        let vista = vista.expect("o nível 2 tem câmera");
        let ms = molduras(sim.world_mut(), Some(vista.center), f.aspect(), None);
        let m = ms.iter().find(|m| m.ativa).expect("a activa");
        assert_eq!(m.centro, vista.center, "{p:?}");
        assert!((m.meia[0] / m.meia[1] - f.aspect()).abs() < 1e-4, "{p:?}");
        assert!((m.meia[1] * 2.0 - vista.height_world).abs() < 1e-4, "{p:?}");

        let caixa_px = ph2d_app_game::encaixe::caixa_do_jogo(canvas, f);
        let cam =
            ph2d_app_game::encaixe::na_janela(vista.center, vista.height_world, janela, caixa_px);
        let (lo, hi) = caixa(m);
        let a = px(cam, janela, [lo[0], hi[1]]);
        let b = px(cam, janela, [hi[0], lo[1]]);
        for (v, e) in [
            (a, [caixa_px[0], caixa_px[1]]),
            (b, [caixa_px[0] + caixa_px[2], caixa_px[1] + caixa_px[3]]),
        ] {
            assert!(
                (v[0] - e[0]).abs() < 0.05 && (v[1] - e[1]).abs() < 0.05,
                "{p:?}: canto {v:?}, a caixa do jogo diz {e:?}"
            );
        }
    }
}
