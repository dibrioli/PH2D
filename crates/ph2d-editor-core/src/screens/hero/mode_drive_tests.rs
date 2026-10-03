//! Os gates do quadro do modo, com uma família FALSA (a ferramenta `fake_paint`) — o lado
//! independente: nenhum gate lê a resposta da função que mede. O Painter de verdade tem os seus em
//! `ph2d_app_painter::paint_mode`.

use super::*;
use crate::floating_panel::{FloatingPanel, ToolId};
use crate::tool::Tool;
use ph2d_a11y::NodeId;

struct Fake(&'static str, bool);

impl Tool for Fake {
    fn id(&self) -> ToolId {
        ToolId::new(self.0)
    }
    fn label(&self) -> &str {
        self.0
    }
    fn icon_slug(&self) -> &str {
        self.0
    }
    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), self.0)
    }
    fn is_default(&self) -> bool {
        self.1
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

const PAINT: &str = "fake_paint";
const IMG: u64 = 10;
const IMG2: u64 = 11;
const EMPTY: u64 = 20;

fn in_hand(t: &ToolRegistry, id: &str) -> bool {
    t.active().is_some_and(|a| a.id() == ToolId::new(id))
}

const FAMILY: ModeFamily = ModeFamily {
    modes: &[(ObjectKind::Image, ObjectMode::Paint)],
    holds: |_, t| in_hand(t, PAINT),
    enter: |_, t| t.set_active(&ToolId::new(PAINT)),
    leave: |_, t| {
        t.activate_default();
    },
};

fn kind_of(bits: u64) -> ObjectKind {
    if bits == EMPTY {
        ObjectKind::Empty
    } else {
        ObjectKind::Image
    }
}

struct Cena {
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
}

fn cena() -> Cena {
    crate::test_support::ensure_panel_registry();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Fake("move", true)));
    tools.register(Box::new(Fake(PAINT, false)));
    tools.activate_default();
    Cena {
        tools,
        hero: HeroScreen::new(NodeId(1)),
        toasts: ToastQueue::new(),
    }
}

impl Cena {
    fn quadro(&mut self, req: Option<ModeRequest>) {
        drive(
            &[FAMILY],
            &kind_of,
            &|_| "Obj".to_string(),
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
    }
    fn paint_in_hand(&self) -> bool {
        in_hand(&self.tools, PAINT)
    }
}

/// ⭐⭐ GATE — `Tab` ida e volta, duas vezes: abre o módulo SOBRE o activo e larga-o ao voltar.
#[test]
fn tab_goes_there_and_back() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    for volta in 0..2 {
        c.quadro(Some(ModeRequest::Toggle));
        assert_eq!(
            c.hero.gizmo.mode.locked_entity(),
            Some(IMG),
            "volta {volta}"
        );
        assert!(
            c.paint_in_hand(),
            "volta {volta}: o modo não abriu o módulo"
        );
        c.quadro(None);
        assert_eq!(
            c.hero.gizmo.mode.current(),
            ObjectMode::Paint,
            "não se segurou"
        );
        c.quadro(Some(ModeRequest::Toggle));
        assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
        assert!(
            !c.paint_in_hand(),
            "volta {volta}: Object não largou o módulo"
        );
    }
}

/// ⭐⭐ GATE — com duas seleccionadas, o modo entra sobre o ÚLTIMO e a selecção colapsa nele.
#[test]
fn entering_takes_only_the_active() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    c.hero.gizmo.add_to_selection(IMG2);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert_eq!(c.hero.gizmo.mode.locked_entity(), Some(IMG2));
    assert_eq!(c.hero.gizmo.selection, Some(IMG2));
    assert!(c.hero.gizmo.extra_selection.is_empty());
}

/// ⭐⭐ GATE — a selecção trocada por OUTRA porta devolve o modo a Object e larga o módulo.
#[test]
fn another_door_changing_the_selection_returns_to_object() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(Some(IMG2));
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.active(), None);
    assert!(!c.paint_in_hand(), "o módulo saltou para o outro objecto");
}

/// O módulo largado por outra porta termina o modo, sem lhe mexer.
#[test]
fn the_module_dropped_elsewhere_ends_the_mode() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    c.quadro(Some(ModeRequest::Toggle));
    c.tools.activate_default();
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
}

/// ⭐ GATE — um tipo só-Object recusa o `Tab` e o DIZ; nada abre.
#[test]
fn an_object_only_type_refuses_and_says_so() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(EMPTY));
    let before = c.toasts.len();
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.active(), None);
    assert!(!c.paint_in_hand());
    assert_eq!(c.toasts.len(), before + 1, "a recusa ficou muda");
}

/// ⭐ GATE — a aba que pede o modo: abre sobre a imagem; sem ela, Object com a ferramenta de omissão.
#[test]
fn a_layout_mode_opens_only_on_a_compatible_active() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    c.quadro(Some(ModeRequest::Open(ObjectMode::Paint)));
    assert!(c.paint_in_hand());
    c.hero.gizmo.replace_selection(Some(EMPTY));
    c.quadro(None);
    c.quadro(Some(ModeRequest::Open(ObjectMode::Paint)));
    assert_eq!(c.hero.gizmo.mode.active(), None);
    assert!(in_hand(&c.tools, "move"));
}

/// ⭐ GATE — o seletor segue o tipo do activo; sem activo não há seletor.
#[test]
fn the_selector_follows_the_active_type() {
    let mut c = cena();
    c.quadro(None);
    assert!(c.hero.store.area_menus().is_empty());
    c.hero.gizmo.replace_selection(Some(IMG));
    c.quadro(None);
    assert_eq!(
        c.hero.store.area_menus()[0].faces,
        ["Object Mode", "Paint Mode"]
    );
    c.hero.gizmo.replace_selection(Some(EMPTY));
    c.quadro(None);
    assert_eq!(c.hero.store.area_menus()[0].faces, ["Object Mode"]);
}

/// ⭐⭐ GATE — o cadeado nas portas: num modo, outra entidade é recusada COM aviso; a própria e o
/// limpar passam; em Object nada é recusado.
#[test]
fn the_lock_door_refuses_and_says_why() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    assert!(
        !refused(&c.hero, Some(IMG2), true, &mut c.toasts),
        "Object não tranca"
    );
    c.quadro(Some(ModeRequest::Toggle));
    let before = c.toasts.len();
    assert!(refused(&c.hero, Some(IMG2), false, &mut c.toasts));
    assert_eq!(c.toasts.len(), before + 1, "recusa muda");
    assert!(!refused(&c.hero, Some(IMG), false, &mut c.toasts));
    assert!(!refused(&c.hero, None, false, &mut c.toasts));
    assert!(
        refused(&c.hero, None, true, &mut c.toasts),
        "o laço acrescenta"
    );
}

/// ⭐ GATE — o botão direito no canvas: em Object pede o menu Add; num modo, não o toma.
#[test]
fn the_right_click_adds_only_in_object_mode() {
    use crate::action_bus::{EditorAction, HierRequest};
    let asks_add = |h: &mut HeroScreen| {
        h.bus
            .drain()
            .any(|a| a == EditorAction::Hierarchy(HierRequest::AddRoot))
    };
    let mut c = cena();
    assert!(right_click_on_canvas(&mut c.hero));
    assert!(asks_add(&mut c.hero));
    c.hero.gizmo.replace_selection(Some(IMG));
    c.quadro(Some(ModeRequest::Toggle));
    let _ = c.hero.bus.drain().count();
    assert!(!right_click_on_canvas(&mut c.hero));
    assert!(!asks_add(&mut c.hero));
}
