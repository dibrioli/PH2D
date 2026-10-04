//! Os gates de Image ▸ Paint com o Painter DE VERDADE, pelo mesmo quadro que a shell corre
//! (`ph2d_editor_core::screens::hero::mode_drive::drive` sobre a [`Family`]).

use super::*;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::mode_drive::drive;
use ph2d_editor_core::toast::ToastQueue;

/// A ferramenta de omissão (o `move` do arranque).
struct Move;

impl ph2d_editor_core::Tool for Move {
    fn id(&self) -> ToolId {
        ToolId::new("move")
    }
    fn label(&self) -> &str {
        "move"
    }
    fn icon_slug(&self) -> &str {
        "move"
    }
    fn build_panel(&self) -> ph2d_editor_core::FloatingPanel {
        ph2d_editor_core::FloatingPanel::new(self.id(), "move")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct Cena {
    sim: SimWorld,
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
}

fn cena() -> Cena {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(PainterTool::default()));
    tools.activate_default();
    Cena {
        sim: SimWorld::default(),
        tools,
        hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
        toasts: ToastQueue::new(),
    }
}

impl Cena {
    fn imagem(&mut self) -> u64 {
        let sprite = ph2d_render::Sprite::atlas(0, [4.0, 2.0], [1.0; 4]);
        self.sim
            .world_mut()
            .spawn((Transform::IDENTITY, sprite))
            .id()
            .to_bits()
    }
    fn vazio(&mut self) -> u64 {
        self.sim
            .world_mut()
            .spawn((Transform::IDENTITY,))
            .id()
            .to_bits()
    }
    /// ⚠️ O lado INDEPENDENTE do tipo: escrito aqui, pelo marcador, e não lido do `kind_of`.
    fn quadro(&mut self, req: Option<ModeRequest>) {
        let world = self.sim.world();
        let kind_of = |b: u64| {
            if world
                .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(b))
                .is_some()
            {
                ObjectKind::Image
            } else {
                ObjectKind::Empty
            }
        };
        let name_of = |_| String::new();
        drive(
            &mut [&mut Family],
            &kind_of,
            &name_of,
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
    }
    fn painter_na_mao(&self) -> bool {
        self.tools
            .active()
            .is_some_and(|t| t.id() == ToolId::new(PAINTER))
    }
}

/// ⭐⭐ GATE (F3 da Imagem) — duas imagens; Paint numa abre o Painter SOBRE ELA e deixa a OUTRA
/// intocada: fora da selecção, sem modo, com a sprite igual ao bit.
#[test]
fn painting_one_image_leaves_the_other_untouched() {
    let mut c = cena();
    let (a, b) = (c.imagem(), c.imagem());
    let antes = *c
        .sim
        .world()
        .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(a))
        .expect("imagem");
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert!(c.painter_na_mao(), "o modo não abriu o Painter");
    assert!(holds_an_image(&mut c.tools));
    assert_eq!(
        c.hero.gizmo.mode.locked_entity(),
        Some(b),
        "o activo é o ÚLTIMO"
    );
    assert_eq!(
        (c.hero.gizmo.selection, c.hero.gizmo.extra_selection.len()),
        (Some(b), 0)
    );
    let depois = c
        .sim
        .world()
        .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(a));
    assert_eq!(depois, Some(&antes));
}

/// ⭐⭐ GATE — `Tab` ida e volta com o Painter: abre, segura-se, e larga-o a favor da de omissão.
#[test]
fn tab_opens_the_painter_and_gives_the_canvas_back() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert!(c.painter_na_mao());
    c.quadro(None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Paint,
        "não se segurou"
    );
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
    assert!(
        c.tools
            .active()
            .is_some_and(|t| t.id() == ToolId::new("move"))
    );
}

/// ⭐ GATE — Paint é da IMAGEM: um vazio recusa, e o Painter não abre.
#[test]
fn an_empty_object_has_no_paint_mode() {
    let mut c = cena();
    let v = c.vazio();
    c.hero.gizmo.replace_selection(Some(v));
    c.quadro(Some(ModeRequest::Toggle));
    assert!(!c.painter_na_mao());
    assert_eq!(c.hero.gizmo.mode.active(), None);
}

/// ⭐⭐ GATE — o Painter sobre a TELA da peça 3D (o Paint do Sculpt, F3) NÃO é o modo da imagem: o
/// quadro não o adopta nem o larga como se fosse.
#[test]
fn the_painter_on_the_sculpt_screen_is_not_the_image_mode() {
    let mut c = cena();
    assert!(c.tools.set_active(&ToolId::new(PAINTER)));
    assert!(
        holds_an_image(&mut c.tools),
        "controlo: sem tela, é o Painter da imagem"
    );
    let painter = c
        .tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>());
    assert!(painter.expect("é o Painter").bind_screen_canvas(64, 64));
    assert!(!holds_an_image(&mut c.tools));
}
