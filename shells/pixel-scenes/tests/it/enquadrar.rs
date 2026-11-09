//! ⭐ **Frame All / Frame Selected conhecem o Pixel Sprite e a Luz** (P14, cena 26): a cena MONTADA
//! pelo roteador, desenhada como o quadro a desenha, e o pedido resolvido pela lei do botão VIEW
//! (`ph2d_app_vec::view_request::resolve`) com as fontes que a shell compõe
//! (`shells/desktop/src/render_loop/fase_view_frame.rs`, conferida por texto abaixo).
//!
//! ⛔ Antes: «View · All (empty scene · reset)» numa cena só de Pixel Sprites e uma Luz, e o *Frame
//! Selected* sem mexer a vista — `frame_boxes` só via imagens, formas e o Flip.

use ph2d_app_light2d::gizmo as lz;
use ph2d_app_pixel::PixelApp;
use ph2d_app_pixel::draw as px;
use ph2d_app_vec::view_request::{Sources, resolve};
use ph2d_ecs::{Entity, Name, PresentWorld, SimWorld, Transform};
use ph2d_editor_core::ViewFocusKind;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_render::camera_frame::ViewFlight;
use ph2d_tokens::Theme;
use ph2d_vec_scene::{VecScene, VecXforms};
use ph2d_vector::VectorScene;

use crate::comum::spawn_root;

const JANELA: WindowSize = WindowSize::new(1600, 1000);
const AREA: ph2d_editor_core::zones::Rect =
    ph2d_editor_core::zones::Rect::new(300.0, 50.0, 1000.0, 900.0);
const PPM: f32 = 100.0;

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).expect(nome).0
}

/// A cena 26 montada e desenhada uma vez (o quadro que precede o pedido).
fn montada() -> (PixelApp, SimWorld) {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 26, spawn_root
    ));
    let ids: Vec<u64> = ph2d_app_pixel::mode::sprites(sim.world())
        .into_iter()
        .map(|(b, _)| b)
        .collect();
    let mut cena = VectorScene::new();
    let cam = Camera2d::default();
    let tema = Theme::default();
    px::draw(
        &mut app,
        sim.world(),
        &cam,
        JANELA,
        PPM,
        None,
        &ids,
        tema,
        &mut cena,
    );
    (app, sim)
}

/// O pedido `kind` com `selection`, pelas fontes da shell; devolve a chave do aviso e a câmera assente.
fn pede(
    sim: &SimWorld,
    kind: ViewFocusKind,
    selection: Vec<u64>,
) -> (Option<&'static str>, Camera2d) {
    let w = sim.world();
    let mut others = Vec::new();
    others.extend(
        px::caixas_desenhadas(w, PPM)
            .into_iter()
            .chain(lz::caixas_desenhadas(w, PPM)),
    );
    let mut present = PresentWorld::new();
    let (scene, xf) = (VecScene::new(), VecXforms::new());
    let mut cam = Camera2d::default();
    let mut voo = ViewFlight::default();
    let key = resolve(
        kind,
        Sources {
            sim,
            present: present.world_mut(),
            scene: &scene,
            xf: &xf,
            drawn_paths: Vec::new(),
            others,
            other_box: &mut |e| px::caixa_mundo(w, e, PPM).or_else(|| lz::caixa_mundo(w, e, PPM)),
            selection,
            area: AREA,
            window: JANELA,
            margin: 44.0,
        },
        &mut cam,
        &mut voo,
        Some(0.0),
    );
    while let Some(v) = voo.tick(cam.view(), |g| g) {
        cam.set_view(v);
    }
    (key, cam)
}

fn posicao(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("transform");
    [t.translation.x, t.translation.y]
}

/// ⭐ *Frame All* na cena 26 enquadra o herói, o chão, a parede e a lâmpada; o herói (16 px) ocupa
/// uma fracção legível da altura da vista (medido 18,2 % nesta área de 1000 × 900 com a cena compacta
/// de 09/11 — era 11,2 % com o chão de 1 m; na janela do app ~1/4), nunca os 16 px de 1000 da vista
/// por omissão.
#[test]
fn o_frame_all_enquadra_o_pixel_sprite() {
    let (_app, mut sim) = montada();
    let heroi = por_nome(&mut sim, "Hero");
    let caixa = px::caixa_mundo(sim.world(), heroi, PPM).expect("o herói foi desenhado");
    let (key, cam) = pede(&sim, ViewFocusKind::All, Vec::new());
    assert_eq!(
        key,
        Some("shell.view.view_all"),
        "Frame All viu a cena vazia"
    );
    let fracao = (caixa.max[1] - caixa.min[1]) / cam.height_world;
    eprintln!(
        "[enquadrar] Frame All: centro {:?}, altura {:.3} m; herói {:.3} m = {:.1} % da altura",
        cam.center,
        cam.height_world,
        caixa.max[1] - caixa.min[1],
        100.0 * fracao
    );
    assert!(
        fracao > 0.15,
        "o herói ocupa só {:.1} % da altura da vista",
        100.0 * fracao
    );
    // A vista inteira (a janela, não só a área): cada peça e a lâmpada cabem nela.
    let meia = [
        cam.height_world * 0.5 * JANELA.width as f32 / JANELA.height as f32,
        cam.height_world * 0.5,
    ];
    let cabe = |p: [f32; 2]| (0..2).all(|i| (p[i] - cam.center[i]).abs() <= meia[i]);
    for nome in ["Hero", "Floor", "Wall"] {
        let e = por_nome(&mut sim, nome);
        let b = px::caixa_mundo(sim.world(), e, PPM).expect(nome);
        assert!(cabe(b.min) && cabe(b.max), "«{nome}» {b:?} fora da vista");
    }
    let luz = por_nome(&mut sim, "Light");
    assert!(cabe(posicao(&sim, luz)), "a lâmpada fora da vista");
}

/// ⭐ *Frame Selected* com só a Luz seleccionada voa para ela (o anel do objecto vazio ou a lâmpada).
#[test]
fn o_frame_selected_enquadra_a_luz() {
    let (_app, mut sim) = montada();
    let luz = por_nome(&mut sim, "Light");
    let (key, cam) = pede(&sim, ViewFocusKind::FrameSelected, vec![luz.to_bits()]);
    assert_eq!(
        key,
        Some("shell.view.view_selected"),
        "Frame Selected não achou a Luz"
    );
    let p = posicao(&sim, luz);
    let d = (cam.center[0] - p[0]).hypot(cam.center[1] - p[1]);
    assert!(
        d < 1e-3,
        "a vista não foi para a Luz: centro {:?}, Luz {p:?}",
        cam.center
    );
    assert!(
        cam.height_world < Camera2d::default().height_world,
        "a vista aproximou"
    );
}

/// A shell compõe as MESMAS fontes que estes gates usam (sem espaços: o `rustfmt` parte linhas).
#[test]
fn a_shell_pergunta_a_caixa_ao_pixel_e_a_luz() {
    let fonte: String = include_str!("../../../desktop/src/render_loop/fase_view_frame.rs")
        .split_whitespace()
        .collect();
    for elo in [
        "px::caixas_desenhadas(w,ppm)",
        "lz::caixas_desenhadas(w,ppm)",
        "px::caixa_mundo(w,e,ppm)",
        "lz::caixa_mundo(w,e,ppm)",
    ] {
        assert!(
            fonte.contains(elo),
            "a shell não compõe `{elo}` no pedido de enquadrar"
        );
    }
}
