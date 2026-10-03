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

/// A família da imagem: Image ▸ Paint põe a ferramenta `fake_paint` em mãos.
struct ImageFamily;

impl ModeFamily for ImageFamily {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Image, ObjectMode::Paint)]
    }
    fn holds(&mut self, _: ObjectMode, _: u64, t: &mut ToolRegistry) -> bool {
        in_hand(t, PAINT)
    }
    fn enter(&mut self, _: ObjectMode, _: u64, t: &mut ToolRegistry) -> bool {
        t.set_active(&ToolId::new(PAINT))
    }
    fn leave(&mut self, _: ObjectMode, _: u64, t: &mut ToolRegistry) {
        t.activate_default();
    }
}

/// Uma escultura FALSA: Sculpt3D ▸ Sculpt · Paint, com o documento dela (a peça em mãos e o modo),
/// e um objecto que nasce a pedir o Sculpt.
#[derive(Default)]
struct SculptFamily {
    held: Option<(u64, ObjectMode)>,
    born: Option<u64>,
    followed: Vec<Option<ActiveMode>>,
}

impl ModeFamily for SculptFamily {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[
            (ObjectKind::Sculpt3D, ObjectMode::Sculpt),
            (ObjectKind::Sculpt3D, ObjectMode::Paint),
        ]
    }
    fn holds(&mut self, m: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held == Some((e, m))
    }
    fn enter(&mut self, m: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held = Some((e, m));
        true
    }
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        self.held = None;
    }
    fn follow(&mut self, current: Option<ActiveMode>, _: &mut ToolRegistry) {
        self.followed.push(current);
    }
    fn wants(&mut self, _: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        self.born.take().map(|e| (e, ObjectMode::Sculpt))
    }
}

const PIECE: u64 = 30;
const PIECE2: u64 = 31;

fn kind_of(bits: u64) -> ObjectKind {
    match bits {
        EMPTY => ObjectKind::Empty,
        PIECE | PIECE2 => ObjectKind::Sculpt3D,
        _ => ObjectKind::Image,
    }
}

struct Cena {
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
    sculpt: SculptFamily,
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
        sculpt: SculptFamily::default(),
    }
}

impl Cena {
    fn quadro(&mut self, req: Option<ModeRequest>) {
        drive(
            &mut [&mut ImageFamily, &mut self.sculpt],
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

/// ⭐⭐ GATE — `Paint` declarado por DOIS tipos: o quadro procura a família por (tipo, modo). Paint
/// sobre a peça abre a ESCULTURA e não a ferramenta da imagem; sobre a imagem, o contrário.
#[test]
fn paint_declared_by_two_types_opens_the_family_of_the_type() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(PIECE));
    c.quadro(None);
    assert_eq!(
        c.hero.store.area_menus()[0].faces,
        ["Object Mode", "Sculpt Mode", "Paint Mode"],
        "o seletor da peça não segue a ordem que a família declara"
    );
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert_eq!(c.sculpt.held, Some((PIECE, ObjectMode::Paint)));
    assert!(
        !c.paint_in_hand(),
        "Paint da peça abriu a ferramenta da imagem"
    );
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Object)));
    c.hero.gizmo.replace_selection(Some(IMG));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert!(c.paint_in_hand());
    assert_eq!(c.sculpt.held, None, "Paint da imagem mexeu na escultura");
}

/// ⭐⭐ GATE (spec/06 §4 F3) — duas peças do mesmo tipo, o modo numa: a outra fica intocada, e a
/// troca entre os modos da família fica na MESMA peça.
#[test]
fn two_pieces_the_mode_on_one_leaves_the_other_untouched() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(PIECE2));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Sculpt)));
    assert_eq!(c.sculpt.held, Some((PIECE2, ObjectMode::Sculpt)));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Paint)));
    assert_eq!(c.sculpt.held, Some((PIECE2, ObjectMode::Paint)));
    assert!(refused(&c.hero, Some(PIECE), false, &mut c.toasts));
    c.quadro(None);
    assert_eq!(c.hero.gizmo.mode.locked_entity(), Some(PIECE2));
    assert_eq!(c.sculpt.held, Some((PIECE2, ObjectMode::Paint)));
}

/// ⭐⭐ GATE — um objecto que NASCE num modo (a peça do menu Add, escolha do dono 03/10) fica
/// seleccionado e entra nele; o pedido corre UMA vez.
#[test]
fn a_born_object_is_selected_and_enters_its_mode_once() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(IMG));
    c.sculpt.born = Some(PIECE);
    c.quadro(None);
    assert_eq!(c.hero.gizmo.selection, Some(PIECE));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Sculpt);
    c.quadro(Some(ModeRequest::Toggle));
    c.quadro(None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Object,
        "o pedido do nascimento repetiu-se"
    );
}

/// ⭐ GATE — cada família vê o modo que FICOU depois do pedido, em todo quadro.
#[test]
fn every_family_follows_the_mode_that_stayed() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(PIECE));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Sculpt)));
    c.quadro(Some(ModeRequest::Toggle));
    let seen: Vec<_> = c
        .sculpt
        .followed
        .iter()
        .map(|a| a.map(|a| a.mode))
        .collect();
    assert_eq!(seen, [Some(ObjectMode::Sculpt), None]);
}

/// ⭐ GATE — o gizmo de transformação só existe em Object: entrar num modo esconde-o, sair devolve-o.
#[test]
fn the_object_gizmo_shows_only_in_object_mode() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(PIECE));
    assert!(
        object_gizmo_shows(&c.hero),
        "controlo: em Object o gizmo aparece"
    );
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Sculpt)));
    assert!(
        !object_gizmo_shows(&c.hero),
        "o gizmo ficou por cima do barro"
    );
    c.quadro(Some(ModeRequest::Toggle));
    assert!(object_gizmo_shows(&c.hero));
}
