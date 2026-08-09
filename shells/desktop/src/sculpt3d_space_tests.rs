//! **O FRUSTUM DE UMA PEÇA** — os gates da porta que monta a vista.
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`). ⚠️ **Nada aqui
//! pede GPU**, e é por isso que o [`super::stencil_of`] é uma função LIVRE: uma
//! [`Sculpt3dScene`] exige um `wgpu::Device`, e o que este cálculo precisa é uma
//! câmera, um viewport e uma pose.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::space::tests
//! ```

use super::*;
use ph2d_sculpt3d::{Alpha, AlphaImage, Brush};

const VIEWPORT: (u32, u32) = (1920, 1080);

/// A câmera enquadrando a esfera unitária, como a cena a enquadra.
fn camera() -> Camera3d {
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(ph2d_mesh::shapes::uv_sphere(16, 24, 1.0).bounds(), {
        let (w, h) = VIEWPORT;
        w as f32 / h as f32
    });
    cam
}

/// Bandas diagonais — estruturado nos DOIS eixos, senão metade de um desvio
/// concorda consigo mesma.
fn stamp() -> Alpha {
    let n = 32u32;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let v = u8::from((x + y) % 8 < 4) * 255;
            let i = ((y * n + x) * 4) as usize;
            rgba[i..i + 3].fill(v);
            rgba[i + 3] = 255;
        }
    }
    Alpha::Image(std::sync::Arc::new(
        AlphaImage::from_rgba(n, n, &rgba).expect("imagem válida"),
    ))
}

/// O que o carimbo desenha numa grade de pixels, para uma peça de pose dada.
///
/// ⚠️ **A grade é a MESMA em MUNDO para as duas poses** — é o que torna a
/// comparação sobre *o que o artista vê no mesmo pixel*, e não sobre números que
/// cada espaço local produz.
fn field_through(cam: &Camera3d, pose: Pose, world: &[[f32; 3]]) -> Vec<f32> {
    let brush = Brush {
        alpha: Some(stamp()),
        alpha_stencil: Some(stencil_of(cam, VIEWPORT, pose)),
        alpha_stencil_scale: 0.25,
        ..Brush::default()
    };
    let frame = brush.alpha_frame();
    world
        .iter()
        .map(|p| brush.alpha_weight(pose.point_to_local(*p), &frame))
        .collect()
}

/// Uma grade de pontos de MUNDO no plano do alvo — o que os pixels do meio da
/// tela atingem.
fn world_grid(cam: &Camera3d) -> Vec<[f32; 3]> {
    let (right, up) = cam.screen_basis();
    let target = cam.target;
    let span = cam.view_height_per_depth(VIEWPORT) * cam.distance;
    let n = 48usize;
    let mut out = Vec::with_capacity(n * n);
    for row in 0..n {
        for col in 0..n {
            let u = (col as f32 / n as f32 - 0.5) * span;
            let v = (row as f32 / n as f32 - 0.5) * span;
            out.push((target + right * u + up * v).into());
        }
    }
    out
}

/// **UMA PEÇA ESCALADA RECEBE O MESMO CARIMBO NA TELA** — a metade do frustum que
/// mora na fronteira, e a que uma mutação plausível quebra em silêncio.
///
/// ⚠️ **A razão do frustum é ADIMENSIONAL** (mundo por mundo), então ela NÃO é
/// dividida pela escala da peça — a pose entra uma vez só, no olho. Dividi-la
/// também seria a escala aplicada duas vezes, e o carimbo sairia do tamanho
/// errado exatamente nas peças que não estão na escala 1: o defeito nasceria
/// invisível numa cena de uma peça só, que é a cena de todo gate deste módulo.
///
/// ⚠️ **E o oráculo são PONTOS DE MUNDO comuns às duas poses**, levados a cada
/// espaço local pela porta do produto. Um gate que amostrasse cada espaço local
/// por conta própria compararia duas grades diferentes e passaria com qualquer
/// coisa.
#[test]
fn a_scaled_piece_wears_the_same_stamp_on_screen() {
    let cam = camera();
    let grid = world_grid(&cam);
    let plain = Pose::IDENTITY;
    let big = Pose::new([0.0, 0.0, 0.0], 3.0);

    let a = field_through(&cam, plain, &grid);
    let b = field_through(&cam, big, &grid);
    let d = a
        .iter()
        .zip(&b)
        .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()));
    assert!(
        d <= 1e-3,
        "a mesma tela desenhou carimbos diferentes em duas peças de escalas \
         diferentes ({d:.4} de desvio) — a régua da vista pegou a escala da peça"
    );
}

/// **E DUAS PERGUNTAS COM A MESMA POSE DÃO O MESMO ESTÊNCIL** — a âncora não
/// existe mais, e este gate é o que impede que ela volte por um parâmetro.
///
/// ⚠️ **Ele parece trivial e não é:** enquanto a porta recebia um PONTO, duas
/// chamadas com a mesma pose devolviam réguas diferentes conforme quem chamava —
/// o dab no acerto, o preview no centro da peça —, e era essa diferença que o
/// artista via como *"a máscara projetada não corresponde ao que está sendo
/// esculpido"*.
#[test]
fn the_view_of_a_piece_is_a_function_of_the_camera_and_the_pose_alone() {
    let cam = camera();
    let pose = Pose::new([0.3, -0.2, 0.1], 1.7);
    assert_eq!(
        stencil_of(&cam, VIEWPORT, pose),
        stencil_of(&cam, VIEWPORT, pose),
        "o estêncil de uma peça deixou de ser função da câmera e da pose"
    );
}
