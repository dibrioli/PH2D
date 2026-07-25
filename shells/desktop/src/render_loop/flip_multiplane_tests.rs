//! Gates da paralaxe multiplano (2.5D, ADR-0114 §Decisão 3) — o `parallax_model`
//! e o wiring do `composite_layers`. Irmão do `flip_pass_tests.rs` (o cap de LOC
//! do HR-18 os separa); declarado pelo pai via `#[path]`, então `super` é
//! `render_loop::flip_pass`.
//!
//! **Por que NÃO há gate de GPU aqui.** A paralaxe vive 100% no transform de
//! vértice: `parallax_model` desloca a translação do `model`, `fold_model` a dobra
//! no `world_to_clip`, e o rasterizador só desenha triângulos nas posições de clip
//! que a MATRIZ produziu. Um readback de GPU testaria o rasterizador (inalterado
//! por esta feature), não a paralaxe. O oráculo fiel é projetar o vértice pela
//! MESMA matriz que a GPU consome — determinístico, sem wgpu. É o que o gate
//! `the_far_layer_lags_the_near_one_under_pan` faz.

use super::{camera_raw, fold_model, parallax_model};
use ph2d_flip_render::CameraRaw;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;

const WIN: WindowSize = WindowSize::new(800, 600);
const HEIGHT_WORLD: f32 = 10.0;

/// A posição NDC-x da origem do objeto (local `(0,0)`) projetada por `cam`. É a
/// coluna de translação da `world_to_clip` col-major (`m[col][row]`): local
/// `(0,0,0,1)` seleciona a coluna 3. Divide pelo `w` (1 no ortográfico, mas
/// honesto). É o pixel onde a GPU poria o vértice da origem.
fn origin_ndc_x(cam: &CameraRaw) -> f32 {
    let m = cam.world_to_clip;
    m[3][0] / m[3][3]
}

/// A câmera do passe para uma camada em `depth`, panhada para `center`.
fn layer_cam(center: [f32; 2], model: &Xform, depth: f32) -> CameraRaw {
    let base = camera_raw(&Camera2d::new(center, HEIGHT_WORLD), WIN);
    fold_model(&base, &parallax_model(model, center, depth))
}

/// 🔴 **`depth == 1.0` devolve o `model` INTACTO** — o caminho comum (toda camada
/// flat) é byte-idêntico ao pré-multiplano. Fixture com um `model` NÃO-trivial
/// (escala + rotação + translação) para provar passthrough real, não só o caso
/// identidade.
///
/// Mutação que sangra: qualquer fórmula que não seja no-op em `depth 1` (p.ex.
/// trocar `1.0 - depth` por `depth`) muda a translação e o gate falha.
#[test]
fn depth_of_one_returns_the_model_untouched() {
    let model = Xform([2.0, 0.5, -0.3, 1.5, 7.0, -3.0]);
    let out = parallax_model(&model, [9.0, 9.0], 1.0);
    assert_eq!(
        out.0, model.0,
        "depth 1 = flat = model intacto, byte a byte"
    );
}

/// 🔴 **`depth == 0.0` PRENDE a camada à câmera** — a translação do model vira o
/// `cam_center` EXATO, independente da origem autorada. É o fundo distante que
/// não se move na tela ao panhar (o mais longe do multiplano).
///
/// Mutação que sangra: âncora errada (dropar o termo da origem) muda o resultado
/// para uma origem `(7,-3)` ≠ 0 sob `cam=(5,2)`.
#[test]
fn a_layer_at_depth_zero_pins_to_the_camera() {
    let model = Xform([1.0, 0.0, 0.0, 1.0, 7.0, -3.0]); // origem (7,-3)
    let out = parallax_model(&model, [5.0, 2.0], 0.0);
    assert!((out.0[4] - 5.0).abs() < 1e-9, "e vira cam.x");
    assert!((out.0[5] - 2.0).abs() < 1e-9, "f vira cam.y");
}

/// 🔴 **A camada FUNDA acompanha MENOS o pan que a da FRENTE** — o coração do
/// multiplano. Panhando de `(0,0)` para `(4,0)`, a origem de uma camada `depth
/// 0.2` desliza na tela exatamente `0.2×` o que a `depth 1.0` desliza. Projetado
/// pela matriz REAL do passe (o que a GPU consome).
///
/// Mutação que sangra: `parallax_model` ignorar `depth` (devolver `*model`
/// sempre) ⇒ a funda desliza o MESMO que a da frente ⇒ razão `1.0` ≠ `0.2`.
#[test]
fn the_far_layer_lags_the_near_one_under_pan() {
    let m = Xform::IDENTITY; // objeto Flip comum (arte em coords de mundo)
    let (pan_a, pan_b) = ([0.0, 0.0], [4.0, 0.0]);
    let delta = |depth: f32| {
        origin_ndc_x(&layer_cam(pan_b, &m, depth)) - origin_ndc_x(&layer_cam(pan_a, &m, depth))
    };
    let near = delta(1.0);
    let far = delta(0.2);
    assert!(near.abs() > 1e-4, "a camada flat DE FATO desliza ao panhar");
    assert!(
        far.abs() < near.abs(),
        "a funda desliza MENOS que a da frente"
    );
    assert!(
        (far / near - 0.2).abs() < 1e-4,
        "o deslocamento é depth×pan: far/near = {} (esperado 0.2)",
        far / near
    );
}

/// 🔴 **Enquadrado de frente, todos os planos coincidem** — a âncora é a origem do
/// objeto: quando a câmera está SOBRE ela (`cam == origem`), toda `depth` devolve o
/// `model` intocado (sem paralaxe, nada a separar). É o invariante que faz a
/// paralaxe nascer só ao panhar.
///
/// Mutação que sangra: ancorar em `0` em vez da origem (`e·depth + cam·(1−depth)`
/// vira `cam·(1−depth)`) ⇒ com `cam == origem == (7,-3)` e depth 0.5 o resultado
/// vira `(3.5,-1.5)` ≠ `(7,-3)`.
#[test]
fn all_planes_coincide_when_the_camera_is_over_the_origin() {
    let model = Xform([1.0, 0.0, 0.0, 1.0, 7.0, -3.0]); // origem (7,-3)
    for depth in [0.0, 0.5, 1.0] {
        let out = parallax_model(&model, [7.0, -3.0], depth);
        assert!(
            (out.0[4] - 7.0).abs() < 1e-9 && (out.0[5] - (-3.0)).abs() < 1e-9,
            "cam sobre a origem: depth {depth} nao separa nada"
        );
    }
}

/// 🔴 **Arch-gate: o `composite_layers` costura o `cam_center` E o `depth` de CADA
/// camada.** A projeção acima prova a MATEMÁTICA, mas não que o passe passa o pan
/// certo (`camera.center`) e a profundidade CERTA (`l.depth`) — esse wiring exige
/// GPU (`GameRt`/wgpu) e nenhum unit test o alcança. Lê o fonte e afirma a
/// PROPRIEDADE (a chamada que costura os três), não distância em bytes.
///
/// Mutação que sangra: trocar `l.depth` por `1.0` (paralaxe morta) ou `cam_center`
/// por `[0.0, 0.0]` (pan ignorado) some do fonte.
#[test]
fn composite_layers_threads_the_camera_center_and_layer_depth() {
    let src = include_str!("flip_pass.rs");
    assert!(
        src.contains("parallax_model(&l.model, cam_center, l.depth)"),
        "o loop de camadas deve costurar model+cam_center+depth pela porta única"
    );
    assert!(
        src.contains("camera.center,"),
        "o callsite deve passar o pan REAL da câmera (camera.center) ao composite"
    );
}
