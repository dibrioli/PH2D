//! ⭐ **A faixa do fecho na cena 24** (`docs/Vector Module/BUGS_vector.md` #42, foto do dono de 15/10):
//! o rectângulo traço-ponto-ponto pintava uma FAIXA branca da quina inferior esquerda até à borda
//! direita da caixa, e os traços da aresta de baixo saíam escuros — conforme o pan e o zoom.
//!
//! A régua é a placa de verdade (`VelloPass`), a cena pelos MESMOS estágios do smoke e o desenho
//! pela MESMA porta da shell (`ph2d_vec_render::dispatch`), em 144 vistas (24 zooms × 6 pans com
//! fracção de píxel). O CONTROLO é o mesmo traço com o padrão SEM a folga do fecho
//! ([`ph2d_vec_scene::dash_fit::fit`]) — tem de pintar a faixa, senão a régua não a vê.

use super::{FIRST, SELECT, step};
use ph2d_vec_scene::{VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, VectorScene};

/// A cena 24 até ao estilo — as duas formas, pelos MESMOS estágios do smoke e pela mesma porta
/// que as torna vivas.
fn cena24() -> (VecScene, VecXforms) {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut vec = crate::state::VecState::default();
    let mut scene = VecScene::new();
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut s = FIRST;
    while s != SELECT {
        s = step(s, &mut vec, &mut scene, &mut hero).expect("o estágio avança");
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut vec.entities);
        crate::shape_live::make_committed_shape_live(
            &mut sim,
            &mut scene,
            &vec.entities,
            &mut vec.shape,
            false,
        );
    }
    let xf = ph2d_vec_entities::transform::build(&sim, &vec.entities);
    (scene, xf)
}

/// A câmara do editor (`Camera2d::world_to_screen_affine`): `k` px por metro, `c` ao centro, Y para
/// cima.
fn camera(size: (u32, u32), k: f64, c: [f64; 2]) -> Affine {
    Affine::translate((f64::from(size.0) * 0.5, f64::from(size.1) * 0.5))
        * Affine::scale_non_uniform(k, -k)
        * Affine::translate((-c[0], -c[1]))
}

/// A cena inteira pela porta da shell.
fn encode(scene: &VecScene, xf: &VecXforms, cam: Affine) -> VectorScene {
    let mut t = VectorScene::new();
    ph2d_vec_render::dispatch(
        scene,
        &VecViewState::default(),
        xf,
        &ph2d_vec_render::LiveGeometry::new(),
        &ph2d_vec_render::FxImages::new(),
        &ph2d_vec_render::WidgetSkins::new(),
        &ph2d_vec_render::PatternTiles::new(),
        &ph2d_vec_render::BrushArts::new(),
        &ph2d_vec_render::DilatedPaints::new(),
        cam,
        &mut t,
    );
    t
}

/// O rectângulo da cena no mundo (m) e a meia largura do traço.
const R: [f64; 4] = [-2.1, -0.8, -0.5, 0.8];
const MEIA_W: f64 = 0.025;

/// Píxeis brancos LONGE da banda do traço (mais de `½w + 1,5 px` do contorno do rectângulo), só do
/// lado dele (`x < 0` — a linha da cena fica à direita).
fn vazamentos(px: &[u8], size: (u32, u32), cam: Affine) -> usize {
    let inv = cam.inverse();
    let k = cam.determinant().abs().sqrt();
    let mut n = 0;
    for y in 0..size.1 {
        for x in 0..size.0 {
            let i = ((y * size.0 + x) * 4) as usize;
            if px[i] < 128 || px[i + 1] < 128 {
                continue;
            }
            let q = inv * ph2d_vector::Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
            if q.x > 0.0 {
                continue;
            }
            let dx = (R[0] - q.x).max(q.x - R[2]);
            let dy = (R[1] - q.y).max(q.y - R[3]);
            // Dentro: até à aresta mais próxima. Fora: Chebyshev — a junta Miter de um canto recto
            // é um QUADRADO, e a distância euclidiana acusaria os cantos dele.
            let d = if dx <= 0.0 && dy <= 0.0 {
                -dx.max(dy)
            } else {
                dx.max(dy)
            };
            if d * k > MEIA_W * k + 1.5 {
                n += 1;
            }
        }
    }
    n
}

/// As 144 vistas: 24 zooms (`100..3 700` px/m) × 6 pans, com fracção de píxel, à volta da aresta
/// de baixo e da quina onde o contorno começa.
fn vistas() -> Vec<(f64, [f64; 2])> {
    let mut v = Vec::new();
    for zi in 0..24 {
        let k = 100.0 * 1.17_f64.powi(zi);
        for (fx, sub) in [
            (0.0, 0.0),
            (0.0, 0.37),
            (0.1, 0.5),
            (0.5, 0.21),
            (1.0, 0.73),
            (0.25, 0.9),
        ] {
            v.push((
                k,
                [R[0] + fx * (R[2] - R[0]) + sub / k, R[1] + 13.0 * sub / k],
            ));
        }
    }
    v
}

/// ⭐ **Nenhuma vista pinta a faixa** — e, sem a folga do fecho, a maioria pinta.
///
/// ⚠️ É `#[ignore]` porque precisa de adapter; corre à mão com `PH2D_GPU=1` (o gate de CPU que
/// segura a lei em toda corrida é o `seam_tests` da `ph2d-vec-render`).
#[test]
#[ignore = "needs a GPU adapter; run with --ignored"]
fn a_cena_24_nao_pinta_a_faixa_do_fecho_em_vista_nenhuma() {
    use ph2d_vector::{Brush, Cap, Color, Join, Stroke};
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("este gate precisa da placa");
    let (scene, xf) = cena24();
    let size = (512, 512);
    let mut pass =
        ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, size).unwrap();
    let p = &scene.paths()[0];
    let s = p.stroke.as_ref().expect("traço");
    let cooked = p.cooked();
    let bp = ph2d_vec_render::build_bezpath(&cooked);
    let (total, _) = ph2d_vec_scene::dash_fit::longest_contour(&cooked).expect("contorno");
    let sem_folga =
        ph2d_vec_scene::dash_fit::fit(s.dash_lengths().expect("tracejado"), total, true);
    // A caneta do `kurbo_stroke` (ponta redonda, junta Miter ao limite da cena), com o padrão de ANTES.
    let controlo = Stroke::new(s.width)
        .with_caps(Cap::Round)
        .with_join(Join::Miter)
        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT)
        .with_dashes(0.0, sem_folga.as_slice().iter().copied());
    let (mut agora, mut antes, mut px_antes) = (0, 0, 0);
    let v = vistas();
    for &(k, c) in &v {
        let cam = camera(size, k, c);
        let px = pass
            .render_and_readback(&gpu, encode(&scene, &xf, cam).inner(), size)
            .unwrap();
        agora += usize::from(vazamentos(&px, size, cam) > 0);
        let mut t = VectorScene::new();
        let to = ph2d_vec_render::path_to_screen(&xf, p.id, cam);
        t.inner_mut()
            .stroke(&controlo, to, &Brush::Solid(Color::WHITE), None, &bp);
        let n = vazamentos(
            &pass.render_and_readback(&gpu, t.inner(), size).unwrap(),
            size,
            cam,
        );
        antes += usize::from(n > 0);
        px_antes += n;
    }
    eprintln!(
        "faixa: agora {agora}/{} vistas · sem a folga {antes}/{} ({px_antes} px)",
        v.len(),
        v.len()
    );
    assert!(
        antes > 0,
        "CONTROLO: sem a folga nenhuma vista pintou a faixa — a régua não a vê"
    );
    assert_eq!(agora, 0, "a faixa do fecho aparece em {agora} vistas");
}
