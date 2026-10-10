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
const RECT: ph2d_editor_core::zones::Rect =
    ph2d_editor_core::zones::Rect::new(300.0, 50.0, 1000.0, 900.0);

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
            area: RECT,
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

/// O pedido resolve-se UM quadro depois de feito — nem antes (o que nasceu nesse quadro ainda não foi
/// desenhado), nem perdido (um quadro sem pedido entrega o anterior).
#[test]
fn a_request_is_answered_one_frame_later() {
    let mut v = ViewState::default();
    assert_eq!(v.defer(Some(ViewFocusKind::All)), None);
    assert_eq!(v.defer(None), Some(ViewFocusKind::All));
    assert_eq!(v.defer(None), None);
    assert_eq!(v.defer(Some(ViewFocusKind::Previous)), None);
    assert_eq!(
        v.defer(Some(ViewFocusKind::FrameSelected)),
        Some(ViewFocusKind::Previous)
    );
}

/// ⭐ **As duas costuras da shell que nenhum teste de unidade alcança** (a mutação F8 provou-o): o
/// quadro pede pelo ADIAMENTO (`ViewState::defer`) e a roda passa pela ARBITRAGEM do voo. ⚠️ Lê a
/// shell pelo texto: quem mover estas linhas move a agulha junto (HOWTO §2).
#[test]
fn the_shell_asks_through_the_deferral_and_the_wheel_through_the_arbitration() {
    let shell = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src");
    let ler =
        |f: &str| std::fs::read_to_string(shell.join(f)).unwrap_or_else(|e| panic!("{f}: {e}"));
    let fase = ler("render_loop/fase_view_frame.rs");
    assert!(
        fase.contains("view_request::resolve("),
        "controlo: a fase mudou de casa"
    );
    assert!(
        fase.contains(".view.defer(kind)?"),
        "o pedido não espera um quadro"
    );
    let zoom = ler("canvas_zoom.rs");
    assert!(
        zoom.contains("pub(crate) fn publish("),
        "controlo: o publish mudou de casa"
    );
    assert!(
        zoom.contains("flight.arbitrate(zoom.target.is_some())"),
        "a roda não passa pela arbitragem do voo"
    );
    let relogio = ler("render_loop/fase_chrome_clock.rs");
    assert!(
        relogio.contains("crate::canvas_zoom::publish("),
        "o relógio não publica pela porta"
    );
}
