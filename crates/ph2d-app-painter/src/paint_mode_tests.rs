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
    // Dono, 06/10: *«melhor deixar o gizmo invisível no paint mode»* (o pincel ganha-lhe o clique).
    // CONTROLO: o Edit da imagem mostra-o (`image_edit_mode_tests`).
    assert!(
        !ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows(&c.hero),
        "o gizmo apareceu no Paint (as alças não se agarram com o pincel na mão)"
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

/// Duas camadas na imagem `a`, guardada no `doc_cache` porque `b` foi ligada depois. Devolve a
/// camada de baixo (a que se vai pedir) e a de cima (a activa ao guardar).
fn a_guardada_com_duas_camadas(c: &mut Cena, a: u64, b: u64) -> (u64, u64) {
    let px = || vec![255u8; 4 * 4 * 4];
    let p = painter(&mut c.tools).expect("painter");
    p.bind_document(a, px(), 4, 4);
    let cima = p.add_raster_layer("Tinta").expect("camada").0;
    p.bind_document(b, px(), 4, 4);
    let baixo = p
        .layer_outline_of(a)
        .expect("a ficou guardada")
        .iter()
        .map(|i| i.key)
        .find(|k| *k != cima)
        .expect("a camada de baixo");
    (baixo, cima)
}

fn activa(c: &mut Cena, e: u64) -> Option<u64> {
    painter(&mut c.tools)?
        .layer_outline_of(e)?
        .iter()
        .find(|i| i.active)
        .map(|i| i.key)
}

/// ⭐⭐ RED-FIRST (onda 4, item 10b) — a camada pedida a uma imagem cujo Paint CAIU antes de o
/// documento ligar não pode ser aberta quando a imagem se liga depois por outro caminho.
#[test]
fn a_layer_asked_before_the_paint_left_is_not_opened_later() {
    let mut c = cena();
    let (a, b) = (c.imagem(), c.imagem());
    let (baixo, cima) = a_guardada_com_duas_camadas(&mut c, a, b);
    c.quadro(Some(ModeRequest::OpenContent {
        entity: a,
        mode: ObjectMode::Paint,
        key: baixo,
    }));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Paint);
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Object,
        "o Paint não saiu"
    );
    let px = vec![255u8; 4 * 4 * 4];
    painter(&mut c.tools)
        .expect("painter")
        .bind_document(a, px, 4, 4);
    assert_eq!(
        activa(&mut c, a),
        Some(cima),
        "a camada pedida ANTES da saída do Paint foi aberta numa ligação posterior"
    );
}

/// ⭐⭐ RED-FIRST (onda 4, item 10b) — ligar OUTRA imagem larga o pedido: voltar à primeira mais
/// tarde não abre a camada antiga.
#[test]
fn binding_another_image_drops_the_layer_asked_of_the_first() {
    let mut c = cena();
    let (a, b, d) = (c.imagem(), c.imagem(), c.imagem());
    let (baixo, cima) = a_guardada_com_duas_camadas(&mut c, a, b);
    c.quadro(Some(ModeRequest::OpenContent {
        entity: a,
        mode: ObjectMode::Paint,
        key: baixo,
    }));
    let px = || vec![255u8; 4 * 4 * 4];
    let p = painter(&mut c.tools).expect("painter");
    p.bind_document(d, px(), 4, 4);
    p.bind_document(a, px(), 4, 4);
    assert_eq!(
        activa(&mut c, a),
        Some(cima),
        "o pedido de `a` sobreviveu à ligação de outra imagem"
    );
}

/// CONTROLO — o caminho feliz: a ligação de `a` no mesmo fio cumpre o pedido.
#[test]
fn the_layer_asked_is_opened_when_its_image_binds() {
    let mut c = cena();
    let (a, b) = (c.imagem(), c.imagem());
    let (baixo, _) = a_guardada_com_duas_camadas(&mut c, a, b);
    c.quadro(Some(ModeRequest::OpenContent {
        entity: a,
        mode: ObjectMode::Paint,
        key: baixo,
    }));
    let px = vec![255u8; 4 * 4 * 4];
    painter(&mut c.tools)
        .expect("painter")
        .bind_document(a, px, 4, 4);
    assert_eq!(activa(&mut c, a), Some(baixo));
}
