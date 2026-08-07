//! Os gates do **empacotamento** do AO de tela.
//!
//! ⚠️ O que se pode afirmar sem device é o que a CPU DECIDE: a régua que leva
//! mundo a pixels, os clamps da porta, e o raio semeado pela peça. O que o passe
//! DESENHA é oráculo de GPU e vive em `tests/gpu_render.rs` — separar as duas
//! coisas é o que impede um gate barato de fingir cobrir o caro.

use super::*;

fn ident() -> [[f32; 4]; 4] {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m
}

/// **A régua mundo→pixels é a da PROJEÇÃO, e o gate a computa por fora.**
///
/// Um objeto de uma unidade de altura a um metro do olho ocupa
/// `altura_do_alvo / (2·tan(fov/2))` pixels — a definição da perspectiva. Se este
/// número estiver errado o raio do AO fica certo em unidades de mundo e ERRADO na
/// tela, que é a forma de a oclusão parecer boa numa distância e sumir na
/// seguinte.
#[test]
fn a_regua_leva_uma_unidade_de_mundo_a_pixels_pela_perspectiva() {
    let fov = core::f32::consts::FRAC_PI_2; // 90 graus: tan(45) = 1 exato
    let raw = SsaoRaw::pack(SsaoParams::default(), ident(), (800, 600), fov);
    // Com tan(fov/2) == 1, a escala é exatamente metade da ALTURA.
    assert!(
        (raw.params[1] - 300.0).abs() < 1e-3,
        "escala {} deveria ser metade da altura em fov 90",
        raw.params[1]
    );

    // Campo mais estreito enxerga menos mundo por pixel, logo a escala SOBE.
    let narrow = SsaoRaw::pack(SsaoParams::default(), ident(), (800, 600), fov * 0.5);
    assert!(
        narrow.params[1] > raw.params[1],
        "um campo mais estreito tem de dar mais pixels por unidade ({} vs {})",
        narrow.params[1],
        raw.params[1]
    );
}

/// ⚠️ **A escala usa a ALTURA e não a largura**, e a fixture separa as duas: num
/// alvo não-quadrado, trocá-las muda o raio efetivo pela razão de aspecto — uma
/// oclusão que engorda quando a janela é alargada.
#[test]
fn a_escala_mede_a_altura_do_alvo_nao_a_largura() {
    let fov = core::f32::consts::FRAC_PI_2;
    let wide = SsaoRaw::pack(SsaoParams::default(), ident(), (1600, 600), fov);
    let tall = SsaoRaw::pack(SsaoParams::default(), ident(), (600, 1600), fov);
    assert!((wide.params[1] - 300.0).abs() < 1e-3, "{}", wide.params[1]);
    assert!((tall.params[1] - 800.0).abs() < 1e-3, "{}", tall.params[1]);
}

/// O device não tem opinião: raio zero, fatias zero e passos zero produziriam
/// divisão por zero DENTRO do laço, onde nada pode reclamar.
#[test]
fn a_porta_clampa_o_que_o_device_nao_sabe_recusar() {
    let bad = SsaoParams {
        radius: -3.0,
        slices: 0,
        steps: 0,
        power: 0.0,
    };
    let raw = SsaoRaw::pack(bad, ident(), (0, 0), 0.0);
    assert!(raw.params[0] > 0.0, "raio {}", raw.params[0]);
    assert!(raw.params[2] >= 1.0, "fatias {}", raw.params[2]);
    assert!(raw.params[3] >= 1.0, "passos {}", raw.params[3]);
    assert!(
        raw.screen[0] >= 1.0 && raw.screen[1] >= 1.0,
        "alvo degenerado"
    );
    assert!(raw.screen[2] > 0.0, "potencia {}", raw.screen[2]);

    // E o teto do outro lado: um valor absurdo não pode virar um laço de milhares
    // de passos por pixel.
    let huge = SsaoParams {
        radius: 1.0,
        slices: 9999,
        steps: 9999,
        power: 1000.0,
    };
    let raw = SsaoRaw::pack(huge, ident(), (64, 64), 1.0);
    assert!(raw.params[2] <= 16.0 && raw.params[3] <= 16.0, "sem teto");
}

/// **O raio nasce da PEÇA** — a lição que o bake já pagou. Uma miniatura e um
/// coloso pedem números diferentes, e o artista não deveria descobrir isso
/// reclamando de que o AO não faz nada.
#[test]
fn o_raio_e_uma_fracao_da_peca() {
    let small = SsaoParams::for_bounds(ph2d_mesh::Aabb {
        min: [-0.5, -0.5, -0.5],
        max: [0.5, 0.5, 0.5],
    });
    let big = SsaoParams::for_bounds(ph2d_mesh::Aabb {
        min: [-50.0, -0.5, -0.5],
        max: [50.0, 0.5, 0.5],
    });
    assert!(
        (small.radius - RADIUS_FRACTION).abs() < 1e-6,
        "peca de 1 unidade: {}",
        small.radius
    );
    assert!(
        (big.radius - 100.0 * RADIUS_FRACTION).abs() < 1e-4,
        "o maior lado manda: {}",
        big.radius
    );
}

/// Uma caixa degenerada não pode produzir raio zero (que o clamp da porta cobre,
/// mas aqui a resposta já sai sã) nem NaN.
#[test]
fn uma_caixa_degenerada_nao_produz_raio_invalido() {
    let p = SsaoParams::for_bounds(ph2d_mesh::Aabb {
        min: [0.0; 3],
        max: [0.0; 3],
    });
    assert!(p.radius.is_finite() && p.radius > 0.0, "{}", p.radius);
}

/// **A fração é a MESMA das duas fontes**, e é isso que impede uma costura
/// visível onde uma delas acaba: o assado e o de tela medem a mesma grandeza.
#[test]
fn as_duas_fontes_de_ao_medem_o_mesmo_alcance() {
    let b = ph2d_mesh::Aabb {
        min: [-1.0, -2.0, -1.0],
        max: [1.0, 2.0, 1.0],
    };
    let screen = SsaoParams::for_bounds(b);
    let baked = ph2d_sdf::AoParams::for_bounds(b);
    assert!(
        (screen.radius - baked.radius).abs() < 1e-5,
        "tela {} contra assado {}",
        screen.radius,
        baked.radius
    );
}

/// O uniform tem o tamanho que o layout do WGSL espera: uma `mat4x4` (64 B) e
/// dois `vec4` (16 B cada).
#[test]
fn o_uniform_tem_o_tamanho_que_o_wgsl_declara() {
    assert_eq!(SsaoRaw::SIZE, 96, "mat4x4 + 2 vec4");
}
