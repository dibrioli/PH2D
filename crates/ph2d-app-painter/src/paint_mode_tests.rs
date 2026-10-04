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

impl Cena {
    fn painter(&mut self) -> &mut PainterTool {
        self.tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .expect("o Painter em mãos")
    }
    /// O que a shell faz mais tarde no quadro em que o Painter chega à mão: dá-lhe a imagem.
    fn dar_imagem(&mut self) {
        use ph2d_editor_core::tool::RasterEditTool;
        (self.painter() as &mut dyn RasterEditTool).set_source(vec![255u8; 4 * 4 * 4], 4, 4);
    }
    /// `(alvo é máscara, a camada de cor, a máscara dela)`.
    fn alvo(
        &mut self,
    ) -> (
        bool,
        Option<ph2d_tool_painter::LayerId>,
        Option<ph2d_tool_painter::LayerId>,
    ) {
        let layers = self.painter().layers();
        let active = layers.active().expect("camada activa");
        let cor = layers.owner_of_mask(active).unwrap_or(active);
        (
            layers.is_mask(active),
            Some(cor),
            layers.get(cor).and_then(|l| l.mask),
        )
    }
}

/// ⭐⭐ GATE (Image ▸ Mask, escolha do dono 04/10) — Mask pinta a MÁSCARA da camada (criada se
/// faltar), Paint volta à camada, e a troca NÃO larga o Painter: a máscara criada continua na mesma
/// pilha (largá-lo desmontaria a tela). Object larga-o. (Mutação: `leave` voltar a largar ⇒ RED.)
#[test]
fn mask_paints_the_layer_mask_and_paint_the_layer_without_dropping_the_painter() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Mask)));
    assert!(c.painter_na_mao(), "o Mask não abriu o Painter");
    c.dar_imagem();
    c.quadro(None);
    let (na_mascara, cor, mascara) = c.alvo();
    assert!(
        na_mascara && mascara.is_some(),
        "o Mask não pôs a máscara como alvo"
    );
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Mask);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert_eq!(
        c.alvo(),
        (false, cor, mascara),
        "o Paint não voltou à camada, ou o Painter caiu"
    );
    // Largado e retomado no mesmo quadro, o Painter desmonta a tela (0×0) ou deixa um bake pendente.
    assert_eq!(
        c.painter().canvas_size(),
        (4, 4),
        "a troca desmontou a tela"
    );
    assert!(
        !c.painter().take_deferred_bake(),
        "a troca largou o Painter"
    );
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Mask)));
    assert_eq!(
        c.alvo(),
        (true, cor, mascara),
        "o 2.º Mask criou outra máscara"
    );
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
    assert!(!c.painter_na_mao(), "Object não largou o Painter");
}

/// ⭐⭐ GATE — o artista escolhe a linha *Mask* no painel de camadas em Paint: o modo passa a Mask; a
/// camada de cor devolve-o a Paint. CONTROLO: sem o clique, Paint segura-se.
#[test]
fn choosing_the_mask_row_moves_the_mode() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    c.dar_imagem();
    c.quadro(None);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Paint, "controlo");
    let mascara = c.painter().add_mask_to_active().expect("máscara");
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Mask);
    assert!(c.painter_na_mao());
    let cor = c.painter().layers().owner_of_mask(mascara).expect("dona");
    c.painter().select_layer(cor);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Paint);
}

/// ⛔ GATE — entrar em Mask ANTES de a imagem chegar ao Painter (o quadro real) não o devolve a Paint:
/// o alvo põe-se quando há camadas.
#[test]
fn mask_waits_for_the_image_before_aiming() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Mask)));
    c.quadro(None);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Mask);
    c.dar_imagem();
    c.quadro(None);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Mask);
    assert!(c.alvo().0, "a máscara não é o alvo");
}

/// ⭐ GATE — o Painter sobre a TELA da escultura não é largado por esta família quando o modo cai.
/// CONTROLO: o mesmo quadro sobre a imagem larga-o (`tab_opens_the_painter_and_gives_the_canvas_back`).
#[test]
fn the_image_family_never_drops_the_sculpt_painter() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    c.quadro(None);
    assert!(c.painter().bind_screen_canvas(64, 64));
    c.quadro(None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Object,
        "o modo da imagem caiu"
    );
    assert!(
        c.painter_na_mao(),
        "a família da imagem largou o Painter da escultura"
    );
}

/// ⭐ GATE — Mask sem máscara possível (o tecto de camadas) volta a Paint e DIZ porquê: a recusa
/// chega ao ecrã como aviso, uma vez. CONTROLO: o Mask normal não avisa (o 1.º gate desta família).
#[test]
fn a_mask_that_cannot_be_added_falls_back_to_paint_and_says_why() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    c.dar_imagem();
    while c.painter().add_raster_layer("cheia").is_some() {}
    let n = c.painter().layers().all_ids().count();
    assert_eq!(
        n,
        ph2d_tool_painter::layers::HARD_CAP_LAYERS,
        "o tecto mudou"
    );
    c.quadro(None);
    let avisos = |c: &Cena| {
        c.toasts
            .iter()
            .filter(|t| t.severity == ph2d_editor_core::toast::ToastSeverity::Warning)
            .count()
    };
    assert_eq!(avisos(&c), 0, "controlo: Paint não avisa");
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Mask)));
    assert_eq!(avisos(&c), 1, "a recusa não chegou ao ecrã");
    c.quadro(None);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Paint);
    assert_eq!(avisos(&c), 1, "avisou mais de uma vez");
    assert!(!c.alvo().0);
}
