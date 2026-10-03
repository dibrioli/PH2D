//! Os gates de Sculpt ▸ Sculpt · Paint com a escultura DE VERDADE — o lado da família; o quadro tem
//! os seus com famílias falsas (`ph2d_editor_core::screens::hero::mode_drive`).
//!
//! As leis ([`holds`], [`follow`]) são puras e correm sempre; o que pede uma cena pede uma placa
//! (`#[ignore]` + `gpu_or_skip!`):
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib sculpt_mode -- --ignored
//! ```

use super::*;
use crate::objects::ObjectId;
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::tool::Tool;

macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// ⭐⭐ GATE — o «tem em mãos»: o barro E a peça viva; o Sculpt sem o Painter, o Paint com ele.
/// ⚠️ O CONTROLO é a linha toda verdadeira para o modo certo: sem ela o gate passaria com `false`
/// sempre.
#[test]
fn holds_needs_the_clay_the_piece_and_the_right_hand() {
    let h = |clay, piece, painter| Hands {
        clay,
        piece,
        painter,
    };
    assert!(holds(ObjectMode::Sculpt, h(true, true, false)), "controlo");
    assert!(holds(ObjectMode::Paint, h(true, true, true)), "controlo");
    assert!(
        !holds(ObjectMode::Sculpt, h(false, true, false)),
        "o `D` tirou o barro"
    );
    assert!(
        !holds(ObjectMode::Sculpt, h(true, false, false)),
        "a peça foi apagada"
    );
    assert!(
        !holds(ObjectMode::Sculpt, h(true, true, true)),
        "Sculpt com o Painter em mãos"
    );
    assert!(
        !holds(ObjectMode::Paint, h(true, true, false)),
        "Paint sem o Painter"
    );
    assert!(
        !holds(ObjectMode::Object, h(true, true, false)),
        "Object não é desta família"
    );
}

/// ⭐⭐ GATE — o barro SEGUE o modo: um modo desta família prende a peça dele; sem ele o barro sai,
/// e uma peça nascida à espera da entidade não é largada antes de entrar.
#[test]
fn the_clay_follows_the_mode() {
    let p = ObjectId(7);
    assert_eq!(
        follow(Some((ObjectMode::Sculpt, Some(p))), false),
        Follow::Hold(p)
    );
    assert_eq!(
        follow(Some((ObjectMode::Paint, Some(p))), true),
        Follow::Hold(p)
    );
    assert_eq!(follow(None, false), Follow::Release);
    assert_eq!(
        follow(Some((ObjectMode::Paint, None)), false),
        Follow::Release,
        "o Paint de uma IMAGEM deixou o barro na tela"
    );
    assert_eq!(
        follow(None, true),
        Follow::Wait,
        "a peça nascida perdeu o barro antes de entrar"
    );
}

/// A ferramenta de omissão.
struct Move;

impl Tool for Move {
    fn id(&self) -> ToolId {
        ToolId::new("move")
    }
    fn label(&self) -> &str {
        "move"
    }
    fn icon_slug(&self) -> &str {
        "move"
    }
    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), "move")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

const A: u64 = 100;
const B: u64 = 101;

/// Duas esferas lado a lado, e a família com o mapa peça↔entidade dela.
fn duas_pecas(device: &wgpu::Device) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, ph2d_mesh::shapes::sculpt_sphere(1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.push_object(
        ph2d_mesh::shapes::sculpt_sphere(1.0),
        ph2d_mesh::Pose::new([3.0, 0.0, 0.0], 1.0),
    );
    s.frame_all(900.0 / 700.0);
    s
}

fn familia(scene: &mut Sculpt3dScene) -> Family<'_> {
    let pieces = [(scene.objects[0].id.0, A), (scene.objects[1].id.0, B)]
        .into_iter()
        .collect();
    Family {
        scene: Some(scene),
        pieces,
    }
}

fn centro_da_peca(scene: &Sculpt3dScene, i: usize) -> (f32, f32) {
    scene
        .project_window(scene.objects[i].pose.translation)
        .expect("a peça está à vista")
}

/// ⭐⭐⭐ GATE (spec/06 §4 F3) — duas peças, Sculpt numa: a mira sobre a OUTRA não a toma, e a peça
/// activa fica a presa. ⚠️ O CONTROLO é a mesma mira em Object: ela TOMA a outra peça — senão o
/// gate passaria com um ponto que não acerta em nada.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn two_pieces_sculpt_on_one_never_aims_at_the_other() {
    let gpu = gpu_or_skip!();
    let mut scene = duas_pecas(&gpu.device);
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.activate_default();
    let em_b = centro_da_peca(&scene, 1);

    assert!(
        scene.aim(em_b.0, em_b.1),
        "controlo: o ponto não acerta na peça B"
    );
    assert_eq!(scene.active, 1, "controlo: em Object a mira toma a peça B");

    let mut f = familia(&mut scene);
    assert!(f.enter(ObjectMode::Sculpt, A, &mut tools));
    assert!(f.holds(ObjectMode::Sculpt, A, &mut tools));
    drop(f);
    assert_eq!(
        scene.active, 0,
        "entrar não abriu SOBRE a peça desta entidade"
    );
    assert!(!scene.aim(em_b.0, em_b.1), "a mira saltou para a peça B");
    assert_eq!(scene.active, 0, "a peça activa deixou a presa");

    let em_a = centro_da_peca(&scene, 0);
    assert!(scene.aim(em_a.0, em_a.1), "a peça presa deixou de se mirar");

    let mut f = familia(&mut scene);
    f.leave(ObjectMode::Sculpt, A, &mut tools);
    drop(f);
    assert!(!scene.clay_on_screen(), "Object deixou o barro na tela");
    assert_eq!(scene.preso, None);
}

/// ⭐⭐ GATE — Paint na peça põe o Painter em mãos sobre ela, e sair larga-o; Sculpt larga-o também.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn paint_on_the_piece_holds_the_painter_and_sculpt_lets_it_go() {
    let gpu = gpu_or_skip!();
    let mut scene = duas_pecas(&gpu.device);
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(PainterTool::default()));
    tools.activate_default();
    let mut f = familia(&mut scene);
    assert!(f.enter(ObjectMode::Paint, B, &mut tools));
    assert!(painter_in_hand(&tools));
    assert!(f.holds(ObjectMode::Paint, B, &mut tools));
    assert!(!f.holds(ObjectMode::Sculpt, B, &mut tools));
    f.leave(ObjectMode::Paint, B, &mut tools);
    // ⚠️ O Painter em mãos por OUTRA porta (sem modo): entrar em Sculpt larga-o — o `leave` acima
    // não o cobre (prova de mutação, 03/10).
    assert!(tools.set_active(&ToolId::new(PAINTER)));
    assert!(f.enter(ObjectMode::Sculpt, B, &mut tools));
    assert!(
        !painter_in_hand(&tools),
        "Sculpt ficou com o Painter em mãos"
    );
    drop(f);
    assert_eq!(scene.active, 1);
}

/// ⭐⭐ GATE — a peça NASCIDA pede Sculpt UMA vez, sobre a entidade dela; o barro espera por ela,
/// e sem modo nenhum o barro sai.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn a_born_piece_asks_for_sculpt_once_and_otherwise_the_clay_leaves() {
    let gpu = gpu_or_skip!();
    let mut scene = duas_pecas(&gpu.device);
    let mut tools = ToolRegistry::new();
    let mut f = familia(&mut scene);
    f.follow(None, &mut tools);
    assert!(
        f.scene
            .as_deref()
            .is_some_and(Sculpt3dScene::clay_on_screen),
        "o follow tirou o barro da peça nascida antes de ela entrar"
    );
    assert!(
        f.wants(&mut tools).is_some(),
        "o follow largou a peça nascida antes de ela entrar"
    );
    drop(f);
    let mut f = familia(&mut scene);
    assert_eq!(
        f.wants(&mut tools),
        None,
        "o pedido do nascimento repetiu-se"
    );
    f.follow(None, &mut tools);
    drop(f);
    assert!(!scene.clay_on_screen(), "sem modo o barro ficou na tela");
}

/// ⭐ GATE — a peça nascida pede o modo pela ENTIDADE da peça activa (a última acrescentada).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_born_piece_is_the_active_one() {
    let gpu = gpu_or_skip!();
    let mut scene = duas_pecas(&gpu.device);
    scene.active = 1;
    let mut tools = ToolRegistry::new();
    let mut f = familia(&mut scene);
    assert_eq!(f.wants(&mut tools), Some((B, ObjectMode::Sculpt)));
}
