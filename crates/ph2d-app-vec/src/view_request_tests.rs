use super::*;
use ph2d_ecs::{PresentWorld, Transform, VecPathRef};
use ph2d_vec_scene::rectangle;

/// Duas formas longe uma da outra; a 1.ª selecionada.
fn mundo() -> (SimWorld, VecScene, Vec<u64>, u64) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let b = scene.push_path(rectangle([30.0, 20.0], [32.0, 22.0]));
    let ea = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(a)))
        .id();
    sim.world_mut().spawn((Transform::IDENTITY, VecPathRef(b)));
    (sim, scene, vec![a, b], ea.to_bits())
}

const JANELA: WindowSize = WindowSize::new(1600, 1000);
const AREA: Area = Area {
    x: 300.0,
    y: 50.0,
    w: 1000.0,
    h: 900.0,
};

/// Corre o pedido e o voo até assentar (a mola de brincar chega num passo), como o quadro faz.
fn pede(
    kind: ViewFocusKind,
    camera: &mut Camera2d,
    flight: &mut ViewFlight,
    drawn: bool,
) -> Option<&'static str> {
    let (sim, scene, paths, sel) = mundo();
    let mut present = PresentWorld::new();
    let xf = VecXforms::new();
    let key = resolve(
        kind,
        Sources {
            sim: &sim,
            present: present.world_mut(),
            scene: &scene,
            xf: &xf,
            drawn_paths: if drawn { paths } else { Vec::new() },
            others: Vec::new(),
            other_box: &mut |_| None,
            selection: vec![sel],
            area: AREA,
            window: JANELA,
            margin: 44.0,
        },
        camera,
        flight,
        Some(0.0),
    );
    while let Some(v) = flight.tick(camera.view(), |g| g) {
        camera.set_view(v);
    }
    key
}

fn caixa(min: [f32; 2], max: [f32; 2]) -> WorldBbox {
    WorldBbox { min, max }
}

/// ⭐ **O pedido chega à câmera** — *Enquadrar tudo* encaixa as DUAS formas desenhadas na área (a lei
/// pura decide onde; aqui prova-se que o pedido a pergunta e que o voo assenta lá); *Enquadrar
/// seleção* só a selecionada; o centrar (`F`) não mexe no zoom; a vista de antes volta exacta.
#[test]
fn each_request_lands_the_camera_where_the_law_says() {
    let partida = Camera2d::new([5.0, -3.0], 7.0);
    let mut cam = partida;
    let mut voo = ViewFlight::default();
    assert_eq!(
        pede(ViewFocusKind::All, &mut cam, &mut voo, true),
        Some("shell.view.view_all")
    );
    let tudo = camera_frame::frame(caixa([0.0, 0.0], [32.0, 22.0]), AREA, JANELA, 44.0).unwrap();
    assert_eq!(cam.view(), tudo);

    assert_eq!(
        pede(ViewFocusKind::Previous, &mut cam, &mut voo, true),
        None,
        "silencioso"
    );
    assert_eq!(cam.view(), partida.view(), "a vista de antes, exacta");

    assert_eq!(
        pede(ViewFocusKind::FrameSelected, &mut cam, &mut voo, true),
        Some("shell.view.view_selected")
    );
    let sel = camera_frame::frame(caixa([0.0, 0.0], [2.0, 1.0]), AREA, JANELA, 44.0).unwrap();
    assert_eq!(cam.view(), sel);

    let antes = Camera2d::new([9.0, 9.0], 3.0);
    let mut cam = antes;
    assert_eq!(
        pede(ViewFocusKind::Selected, &mut cam, &mut voo, true),
        Some("shell.view.view_selected")
    );
    assert_eq!(
        (cam.center, cam.height_world),
        ([1.0, 0.5], 3.0),
        "o F centra e não mexe no zoom"
    );
}

/// A cena vazia volta à vista de omissão (o oráculo Excalidraw: a 100 %).
#[test]
fn an_empty_scene_returns_to_the_default_view() {
    let mut cam = Camera2d::new([5.0, -3.0], 7.0);
    let mut voo = ViewFlight::default();
    assert_eq!(
        pede(ViewFocusKind::All, &mut cam, &mut voo, false),
        Some("shell.view.view_all_empty_scene")
    );
    assert_eq!(cam.view(), Camera2d::default().view());
}
