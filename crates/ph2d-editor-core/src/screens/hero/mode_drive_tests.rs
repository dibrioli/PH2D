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

/// Um Model FALSO: Model3D ▸ Edit, que edita as PARTES da peça (as formas dela).
#[derive(Default)]
struct ModelFamily {
    held: Option<u64>,
}

const MODEL: u64 = 40;
const MODEL2: u64 = 41;
const SHAPE: u64 = 42;
const SHAPE2: u64 = 43;

impl ModeFamily for ModelFamily {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Model3D, ObjectMode::Edit)]
    }
    fn holds(&mut self, _: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held == Some(e)
    }
    fn enter(&mut self, _: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held = Some(e);
        true
    }
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        self.held = None;
    }
    fn parts(&mut self, e: u64) -> Option<Vec<u64>> {
        (e == MODEL).then(|| vec![SHAPE, SHAPE2])
    }
    fn owner_of(&mut self, bits: u64) -> Option<u64> {
        [SHAPE, SHAPE2].contains(&bits).then_some(MODEL)
    }
}

fn model_kind(bits: u64) -> ObjectKind {
    match bits {
        MODEL | MODEL2 => ObjectKind::Model3D,
        _ => ObjectKind::Empty,
    }
}

/// ⭐⭐ GATE (spec/06 F3, o Edit do Model) — **num modo de PARTES a selecção anda dentro da peça**:
/// uma forma em Object oferece o Edit do dono; em Edit, as formas (uma, duas, nenhuma) seguram o
/// modo e o seletor continua a dizer Edit; outra peça é recusada; e o `Tab` de volta devolve a
/// selecção à peça inteira, para o `Tab` seguinte voltar ao Edit.
#[test]
fn a_mode_of_parts_lets_the_selection_move_inside_the_piece() {
    let mut c = cena();
    let mut model = ModelFamily::default();
    let quadro = |c: &mut Cena, model: &mut ModelFamily, req| {
        drive(
            &mut [model],
            &model_kind,
            &|_| "Obj".to_string(),
            &mut c.tools,
            &mut c.hero,
            &mut c.toasts,
            req,
        );
    };
    c.hero.gizmo.replace_selection(Some(SHAPE));
    quadro(&mut c, &mut model, None);
    assert!(
        c.hero.gizmo.mode.available().contains(&ObjectMode::Edit),
        "uma forma em Object não oferece o Edit da peça dela"
    );
    quadro(&mut c, &mut model, Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.locked_entity(), Some(MODEL));
    c.hero.gizmo.replace_selection(Some(SHAPE));
    c.hero.gizmo.extra_selection = vec![SHAPE2];
    quadro(&mut c, &mut model, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "duas formas derrubaram o Edit"
    );
    assert!(
        c.hero.gizmo.mode.available().contains(&ObjectMode::Edit),
        "o seletor perdeu o Edit com uma forma seleccionada"
    );
    assert!(
        refused(&c.hero, Some(MODEL2), false, &mut c.toasts),
        "outra peça"
    );
    assert!(
        !refused(&c.hero, Some(SHAPE2), true, &mut c.toasts),
        "acrescentar uma forma"
    );
    c.hero.gizmo.replace_selection(None);
    quadro(&mut c, &mut model, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "desseleccionar saiu do Edit"
    );
    assert!(
        c.hero.gizmo.mode.available().contains(&ObjectMode::Edit),
        "sem nada seleccionado o seletor perdeu o Edit — o activo publicado tem de ser a peça"
    );
    c.hero.gizmo.replace_selection(Some(SHAPE));
    quadro(&mut c, &mut model, Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
    assert_eq!(
        c.hero.gizmo.selection,
        Some(MODEL),
        "o Tab não devolveu a peça inteira"
    );
    quadro(&mut c, &mut model, Some(ModeRequest::Toggle));
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "o Tab não voltou ao Edit"
    );
    c.hero.gizmo.replace_selection(Some(MODEL2));
    quadro(&mut c, &mut model, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Object,
        "outra peça por outra porta e o Edit ficou de pé"
    );
}

/// Um vetor FALSO: Vector ▸ Edit leva junto as formas seleccionadas (o multi-objecto do Blender).
#[derive(Default)]
struct JoinFamily {
    held: Vec<u64>,
}

const VEC_A: u64 = 50;
const VEC_B: u64 = 51;
const VEC_C: u64 = 52;
/// Uma FORMA dentro do objecto vetorial — uma parte de outro tipo (spec/06 F3, o contentor).
const VEC_SHAPE: u64 = 53;

impl ModeFamily for JoinFamily {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Vector, ObjectMode::Edit)]
    }
    fn holds(&mut self, _: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held.first() == Some(&e)
    }
    fn enter(&mut self, m: ObjectMode, e: u64, t: &mut ToolRegistry) -> bool {
        self.enter_with(m, e, &[], t)
    }
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        self.held.clear();
    }
    fn joins(&self, _: ObjectMode) -> bool {
        true
    }
    fn parts_take_the_object_gizmo(&self, _: ObjectMode) -> bool {
        true
    }
    fn enter_with(&mut self, _: ObjectMode, e: u64, joined: &[u64], _: &mut ToolRegistry) -> bool {
        self.held = [e].into_iter().chain(joined.iter().copied()).collect();
        true
    }
    fn parts(&mut self, e: u64) -> Option<Vec<u64>> {
        (self.held.first() == Some(&e)).then(|| {
            let mut p = self.held.clone();
            p.push(VEC_SHAPE);
            p
        })
    }
}

fn vec_kind(bits: u64) -> ObjectKind {
    match bits {
        VEC_A | VEC_B | VEC_C => ObjectKind::Vector,
        VEC_SHAPE => ObjectKind::Empty,
        _ => ObjectKind::Image,
    }
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector, o multi-objecto) — **um modo que JUNTA leva os do mesmo tipo**:
/// com duas formas e uma imagem seleccionadas, o `Tab` entra no Edit do activo COM a outra forma
/// (a imagem sai da selecção); a selecção anda entre as duas, uma terceira forma é recusada; e o
/// `Tab` de volta devolve as duas à selecção (e NÃO as formas de dentro, que também são partes),
/// para o seguinte voltar a juntá-las.
#[test]
fn a_mode_that_joins_takes_the_selected_of_the_same_kind() {
    let mut c = cena();
    let mut fam = JoinFamily::default();
    let quadro = |c: &mut Cena, fam: &mut JoinFamily, req| {
        drive(
            &mut [fam],
            &vec_kind,
            &|_| "Obj".to_string(),
            &mut c.tools,
            &mut c.hero,
            &mut c.toasts,
            req,
        );
    };
    let selected = |c: &Cena| {
        let mut s: Vec<u64> = c.hero.gizmo.selection.iter().copied().collect();
        s.extend(&c.hero.gizmo.extra_selection);
        s.sort_unstable();
        s
    };
    c.hero.gizmo.replace_selection(Some(VEC_A));
    c.hero.gizmo.add_to_selection(IMG);
    c.hero.gizmo.add_to_selection(VEC_B);
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(
        c.hero.gizmo.mode.locked_entity(),
        Some(VEC_B),
        "o activo é o último"
    );
    assert_eq!(
        fam.held,
        vec![VEC_B, VEC_A],
        "a outra forma não entrou no Edit"
    );
    assert_eq!(
        selected(&c),
        vec![VEC_A, VEC_B],
        "a imagem ficou, ou uma forma saiu"
    );
    quadro(&mut c, &mut fam, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "as duas derrubaram o Edit"
    );
    assert!(
        refused(&c.hero, Some(VEC_C), false, &mut c.toasts),
        "uma 3.ª forma"
    );
    assert!(
        !refused(&c.hero, Some(VEC_A), false, &mut c.toasts),
        "a forma que entrou"
    );
    c.hero.gizmo.replace_selection(None);
    quadro(&mut c, &mut fam, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "desseleccionar saiu do Edit"
    );
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
    assert_eq!(
        selected(&c),
        vec![VEC_A, VEC_B],
        "o Tab não devolveu as duas"
    );
    assert_eq!(
        object_mode::active_of(c.hero.gizmo.selection, &c.hero.gizmo.extra_selection),
        Some(VEC_B),
        "o activo mudou ao sair"
    );
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(
        fam.held,
        vec![VEC_B, VEC_A],
        "o Tab seguinte não as juntou de novo"
    );
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector; report do dono 04/10: *«o gizmo não aparece e não consigo a
/// multiseleção»*) — **num modo de PARTES que o declara, a parte seleccionada tem o gizmo** (o
/// Select do Edit do vetor transforma as formas por ele), o objecto trancado não; e **o laço fica só
/// com as partes** em vez de recusar. Num modo de objecto inteiro (Sculpt) o gizmo some e o laço é
/// recusado, como antes.
#[test]
fn a_parts_mode_gives_the_part_its_gizmo_and_the_lasso_its_parts() {
    let mut c = cena();
    let mut fam = JoinFamily::default();
    let quadro = |c: &mut Cena, fam: &mut JoinFamily, req| {
        drive(
            &mut [fam],
            &vec_kind,
            &|_| "Obj".to_string(),
            &mut c.tools,
            &mut c.hero,
            &mut c.toasts,
            req,
        );
    };
    c.hero.gizmo.replace_selection(Some(VEC_A));
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Edit);
    assert!(
        !object_gizmo_shows(&c.hero),
        "o objecto trancado ganhou o gizmo em Edit"
    );
    c.hero.gizmo.replace_selection(Some(VEC_SHAPE));
    quadro(&mut c, &mut fam, None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Edit);
    assert!(
        object_gizmo_shows(&c.hero),
        "a forma seleccionada em Edit não tem gizmo"
    );
    let mut bits = vec![VEC_SHAPE, IMG, VEC_C];
    assert!(
        lasso_admits(&c.hero, &mut bits, &mut c.toasts),
        "o laço foi recusado em Edit"
    );
    assert_eq!(bits, vec![VEC_SHAPE], "o laço levou o que não é parte");

    let mut s = cena();
    s.hero.gizmo.replace_selection(Some(PIECE));
    s.quadro(Some(ModeRequest::Enter(ObjectMode::Sculpt)));
    let mut bits = vec![PIECE, PIECE2];
    assert!(
        !lasso_admits(&s.hero, &mut bits, &mut s.toasts),
        "o laço passou num modo inteiro"
    );
    assert!(!object_gizmo_shows(&s.hero));
}
