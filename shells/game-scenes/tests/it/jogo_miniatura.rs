//! Os gates da miniatura do que a câmera vê (A6, `docs/GameEngineFuncional/18_miniatura_da_camera_plano.md`):
//! QUAL câmera, a vista dela e ONDE no canvas. O desenho é o do canvas (a vista troca-se, `vista_da_camera.rs`).

use ph2d_app_game::miniatura::{caixa, escolhida, vista};
use ph2d_app_game::moldura::molduras;
use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;

const DT: f64 = 1.0 / 60.0;
const ASPECTO: f32 = 16.0 / 9.0;

fn nivel_2() -> SimWorld {
    let mut sim = SimWorld::new();
    let _ = crate::comp::montar(&mut sim, 2);
    sim
}

/// A câmera do nível com a moldura dela, seleccionada (`selecao`) ou não.
fn com_moldura(sim: &mut SimWorld, selecao: bool) -> Vec<ph2d_app_game::moldura::Moldura> {
    let (v, _) = ph2d_app_components::camera_2d::update(sim, ASPECTO, 1, DT, [0.0, 0.0]);
    let v = v.expect("o nível 2 tem câmera");
    let cam = ph2d_ecs::active_camera_of(sim.world_mut()).expect("a activa");
    let sel = selecao.then_some(cam.to_bits());
    molduras(sim.world_mut(), Some(v.center), ASPECTO, sel)
}

/// ⭐⭐ **A miniatura filma a MOLDURA**: os cantos da moldura da câmera seleccionada caem nos cantos do alvo da
/// miniatura (o tamanho da caixa) — o que o canto mostra é o rectângulo que a moldura desenha, nem mais nem menos.
///
/// **Mutações que devem sangrar:** a altura sem o `2 ×` · o centro da activa em vez do da moldura.
#[test]
fn a_miniatura_filma_o_rectangulo_da_moldura() {
    let mut sim = nivel_2();
    let ms = com_moldura(&mut sim, true);
    let m = escolhida(&ms).expect("a seleccionada");
    let c = vista(sim.world(), m);
    let [_, _, w, h] = caixa([300.0, 60.0, 1280.0, 900.0], ASPECTO, f32::INFINITY).expect("cabe");
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let janela = WindowSize {
        width: w as u32,
        height: h as u32,
    };
    let (lo, hi) = ph2d_app_game::moldura::caixa(m);
    let a = c.world_to_screen([lo[0], hi[1]], janela);
    let b = c.world_to_screen([hi[0], lo[1]], janela);
    let folga = 0.6; // a caixa arredonda-se ao píxel
    for (v, e) in [(a, (0.0, 0.0)), (b, (w, h))] {
        assert!(
            (v.0 - e.0).abs() < folga && (v.1 - e.1).abs() < folga,
            "canto {v:?}, o alvo diz {e:?}"
        );
    }
}

/// ⭐ Sem câmera seleccionada não há miniatura (a A6 mostra a SELECCIONADA, não a activa).
#[test]
fn sem_selecao_nao_ha_miniatura() {
    let mut sim = nivel_2();
    assert!(escolhida(&com_moldura(&mut sim, false)).is_none());
    assert!(escolhida(&com_moldura(&mut sim, true)).is_some());
}

/// ⭐ A máscara de camadas é a da CÂMERA: uma camada que ela não filma também falta na miniatura.
#[test]
fn a_mascara_de_camadas_e_a_da_camera() {
    let mut sim = nivel_2();
    let cam = ph2d_ecs::active_camera_of(sim.world_mut()).expect("a activa");
    sim.world_mut()
        .get_mut::<ph2d_ecs::GameCamera>(cam)
        .expect("a câmera")
        .cull_mask = 0b101;
    let ms = com_moldura(&mut sim, true);
    let m = escolhida(&ms).expect("a seleccionada");
    assert_eq!(vista(sim.world(), m).cull_mask, 0b101);
}

/// ⭐ O canto: em baixo à direita, por cima da faixa do HUD de baixo, no aspecto do jogo; um canvas pequeno demais
/// (ou uma faixa que come a altura toda) não a tem.
///
/// **Mutações que devem sangrar:** ignorar o `chao` · ignorar a régua no `topo`.
#[test]
fn a_caixa_fica_no_canto_de_baixo_por_cima_do_hud() {
    let canvas = [300.0, 60.0, 1280.0, 900.0];
    let fundo_do_canvas = canvas[1] + canvas[3];
    let [x, y, w, h] = caixa(canvas, ASPECTO, f32::INFINITY).expect("cabe");
    assert!((w / h - ASPECTO).abs() < 0.02, "aspecto {}", w / h);
    assert!(x + w < canvas[0] + canvas[2], "dentro à direita");
    assert!(x + w > canvas[0] + canvas[2] - 30.0, "encostada à direita");
    assert!(
        y + h < fundo_do_canvas && y + h > fundo_do_canvas - 30.0,
        "encostada em baixo"
    );
    let chao = fundo_do_canvas - 50.0;
    let [_, y2, _, h2] = caixa(canvas, ASPECTO, chao).expect("cabe");
    assert!(y2 + h2 <= chao, "por cima do HUD: {} > {chao}", y2 + h2);
    let regua = canvas[1] + ph2d_editor_core::ruler::RULER_PX;
    assert!(
        caixa(canvas, ASPECTO, regua + h).is_none(),
        "a régua e o HUD comem a altura"
    );
    assert!(caixa([0.0, 0.0, 120.0, 60.0], ASPECTO, f32::INFINITY).is_none());
}

/// ⭐ Com o ponteiro por cima a miniatura dá lugar ao que está por baixo (a alça do canto da moldura); a borda conta.
#[test]
fn o_ponteiro_por_cima_destapa_a_moldura() {
    use ph2d_app_game::miniatura::sob_o_ponteiro;
    let c = [100.0, 50.0, 200.0, 100.0];
    assert!(sob_o_ponteiro(c, (150.0, 80.0)));
    assert!(sob_o_ponteiro(c, (300.0, 150.0)));
    assert!(!sob_o_ponteiro(c, (99.0, 80.0)));
    assert!(!sob_o_ponteiro(c, (150.0, 151.0)));
}
