//! Os gates da fase do modo (spec/06 F2 + a F3 da Imagem), dirigidos pela MESMA `drive` que o
//! quadro corre, sobre o registo de ferramentas do arranque e duas imagens de verdade.

use super::drive;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::{HeroScreen, NodeId, ToolId, ToolRegistry};

struct Cena {
    sim: SimWorld,
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
    a: u64,
    b: u64,
    vazio: u64,
}

fn cena() -> Cena {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut sim = SimWorld::default();
    let mut imagem = || {
        sim.world_mut()
            .spawn((
                Transform::IDENTITY,
                ph2d_render::Sprite::atlas(0, [4.0, 2.0], [1.0; 4]),
            ))
            .id()
            .to_bits()
    };
    let a = imagem();
    let b = imagem();
    let vazio = ph2d_app_components::object_add::spawn_empty_root(&mut sim, "Empty");
    let mut tools = ToolRegistry::new();
    ph2d_tool_registry_init::register_all_tools(&mut tools);
    tools.activate_default();
    Cena {
        sim,
        tools,
        hero: HeroScreen::new(NodeId(1)),
        toasts: ToastQueue::new(),
        a,
        b,
        vazio,
    }
}

impl Cena {
    fn quadro(&mut self, req: Option<ModeRequest>) {
        drive(
            self.sim.world(),
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
    }

    fn seleciona(&mut self, bits: u64) {
        self.hero.gizmo.replace_selection(Some(bits));
    }

    fn painter_na_mao(&self) -> bool {
        self.tools
            .active()
            .is_some_and(|t| t.id() == ToolId::new("painter"))
    }
}

/// ⭐⭐ GATE — `Tab` numa imagem entra em Paint com o Painter SOBRE ELA, e o 2.º `Tab` volta a
/// Object e larga o Painter. Ida e volta duas vezes (o regresso lembra o modo).
#[test]
fn tab_paints_the_image_and_comes_back() {
    let mut c = cena();
    c.seleciona(c.a);
    for volta in 0..2 {
        c.quadro(Some(ModeRequest::Toggle));
        assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Paint, "volta {volta}");
        assert_eq!(c.hero.gizmo.mode.locked_entity(), Some(c.a));
        assert!(c.painter_na_mao(), "volta {volta}: o modo não abriu o Painter");
        c.quadro(None);
        assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Paint, "o modo não se segurou");
        c.quadro(Some(ModeRequest::Toggle));
        assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
        assert!(!c.painter_na_mao(), "volta {volta}: Object não largou o Painter");
    }
}

/// ⭐⭐ GATE (F3 da Imagem) — duas imagens; entrar em Paint numa deixa a OUTRA intocada: fora da
/// selecção, sem modo, com a sprite igual ao bit.
#[test]
fn painting_one_image_leaves_the_other_untouched() {
    let mut c = cena();
    let antes = *c
        .sim
        .world()
        .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(c.a))
        .expect("é imagem");
    c.seleciona(c.a);
    c.hero.gizmo.add_to_selection(c.b);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert_eq!(c.hero.gizmo.mode.locked_entity(), Some(c.b), "o activo é o ÚLTIMO");
    assert_eq!(c.hero.gizmo.selection, Some(c.b));
    assert!(c.hero.gizmo.extra_selection.is_empty(), "a outra ficou seleccionada");
    assert!(!c.hero.gizmo.is_selected(c.a));
    assert_eq!(
        c.sim
            .world()
            .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(c.a)),
        Some(&antes)
    );
}

/// ⭐⭐ GATE — a selecção trocada por OUTRA porta (criar, duplicar, desfazer…) devolve o modo a
/// Object e larga o Painter, em vez de ele saltar para o objecto novo.
#[test]
fn another_door_changing_the_selection_returns_to_object() {
    let mut c = cena();
    c.seleciona(c.a);
    c.quadro(Some(ModeRequest::Toggle));
    assert!(c.painter_na_mao());
    c.seleciona(c.b);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.active(), None);
    assert!(!c.painter_na_mao(), "o Painter saltou para a outra imagem");
}

/// O Painter largado por outra porta (outra ferramenta, `Esc`) termina o modo.
#[test]
fn the_painter_dropped_elsewhere_ends_the_mode() {
    let mut c = cena();
    c.seleciona(c.a);
    c.quadro(Some(ModeRequest::Toggle));
    c.tools.activate_default();
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
}

/// ⭐ GATE — um objecto só-Object recusa o `Tab` e o diz; o Painter não abre.
#[test]
fn an_object_only_type_refuses_tab_and_says_so() {
    let mut c = cena();
    c.seleciona(c.vazio);
    let avisos = c.toasts.len();
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.active(), None);
    assert!(!c.painter_na_mao());
    assert_eq!(c.toasts.len(), avisos + 1, "a recusa ficou muda");
}

/// ⭐ GATE — a aba Draw (o `Open(Paint)`): pinta a imagem activa; sem imagem fica em Object com a
/// ferramenta de omissão, e não cria nada.
#[test]
fn the_draw_tab_paints_an_image_and_otherwise_stays_in_object() {
    let mut c = cena();
    c.seleciona(c.a);
    c.quadro(Some(ModeRequest::Open(ObjectMode::Paint)));
    assert!(c.painter_na_mao());
    c.seleciona(c.vazio);
    c.quadro(None);
    let objectos = c.sim.world().entities().len();
    c.quadro(Some(ModeRequest::Open(ObjectMode::Paint)));
    assert_eq!(c.hero.gizmo.mode.active(), None);
    assert!(!c.painter_na_mao());
    assert_eq!(c.sim.world().entities().len(), objectos, "a aba criou um objecto");
}

/// ⭐ GATE — o seletor é publicado para o activo, com as faces do tipo; sem activo, não há.
#[test]
fn the_selector_follows_the_active_type() {
    let mut c = cena();
    c.quadro(None);
    assert!(c.hero.store.area_menus().is_empty(), "seletor sem objecto activo");
    c.seleciona(c.a);
    c.quadro(None);
    assert_eq!(c.hero.store.area_menus()[0].faces, ["Object Mode", "Paint Mode"]);
    c.seleciona(c.vazio);
    c.quadro(None);
    assert_eq!(c.hero.store.area_menus()[0].faces, ["Object Mode"]);
}

/// ⭐⭐ GATE de costura — o CLIQUE real: o chip abre o pulldown e a linha *Paint Mode* chega ao
/// barramento como o pedido que a fase drena; com ele, a imagem entra em Paint.
#[test]
fn a_real_click_on_the_selector_enters_the_mode() {
    let mut c = cena();
    c.seleciona(c.a);
    c.quadro(None);
    c.hero.apply_event(WidgetEvent::Click(
        ph2d_editor_core::ids::area_menu_button(0),
    ));
    assert_eq!(
        c.hero.store.context_menu().map(|m| m.kind),
        Some(ContextMenuKind::AreaCommands { slot: 0 }),
        "o chip não abriu o seletor"
    );
    c.hero.bus.clear();
    c.hero
        .apply_event(WidgetEvent::Click(ph2d_editor_core::ids::OBJECT_MODE_PAINT));
    let pedido = c.hero.bus.drain().find_map(|a| match a {
        EditorAction::ObjectMode(r) => Some(r),
        _ => None,
    });
    assert_eq!(pedido, Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert!(c.hero.store.context_menu().is_none(), "o pulldown ficou aberto");
    c.quadro(pedido);
    assert!(c.painter_na_mao());
}
