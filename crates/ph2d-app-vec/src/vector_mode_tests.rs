//! Os gates de Vector ▸ Edit (spec/06 F3 ▸ Vector, 2.ª volta: cada forma é um objecto) — as leis
//! puras, a família com a ferramenta do vetor DE VERDADE, e o fluxo do dono pelo MESMO quadro que a
//! shell corre (`mode_drive::drive`).

use super::*;
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::screens::hero::mode_drive::drive;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::tool::Tool;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_entities::entities::sync;

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

struct Cena {
    sim: SimWorld,
    scene: ph2d_vec_scene::VecScene,
    vec: VecState,
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
}

fn cena() -> Cena {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(VectorTool::default()));
    tools.activate_default();
    Cena {
        sim: SimWorld::default(),
        scene: ph2d_vec_scene::VecScene::new(),
        vec: VecState::default(),
        tools,
        hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
        toasts: ToastQueue::new(),
    }
}

impl Cena {
    /// Uma forma nova (o que um traço deixa), já com entidade — devolve os bits dela.
    fn forma(&mut self, x: f64) -> u64 {
        let id = self
            .scene
            .push_path(ph2d_vec_scene::rectangle([x, 0.0], [x + 1.0, 1.0]));
        sync(&mut self.sim, &mut self.scene, &mut self.vec.entities);
        self.vec.entities[&id]
    }
    /// ⚠️ O lado INDEPENDENTE do tipo: escrito aqui, pelo marcador, e não lido do `kind_of`.
    fn quadro(&mut self, req: Option<ModeRequest>) {
        let sim = &self.sim;
        let kind_of = |b: u64| {
            if sim
                .world()
                .get::<ph2d_ecs::VecPathRef>(Entity::from_bits(b))
                .is_some()
            {
                ObjectKind::Vector
            } else {
                ObjectKind::Empty
            }
        };
        let name_of = |_| String::new();
        let mut fam = Family::new(&mut self.vec, sim);
        drive(
            &mut [&mut fam],
            &kind_of,
            &name_of,
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
    }
    fn na_mao(&mut self) -> bool {
        in_hand(&mut self.tools)
    }
    fn em_edit(&self) -> Option<u64> {
        self.hero
            .gizmo
            .mode
            .active()
            .filter(|a| a.mode == ObjectMode::Edit)
            .map(|a| a.entity)
    }
    fn avisos_de_entrada(&self) -> usize {
        let entered = ph2d_i18n::tr_with("object_mode.entered", &[("mode", &"")]);
        let marca = entered.trim();
        self.toasts
            .iter()
            .filter(|t| t.message.contains(marca))
            .count()
    }
}

/// ⭐ GATE — as leis, puras: «tem em mãos», a porta antiga, a ferramenta que segue o modo e o
/// NASCIMENTO (uma só forma nova, fora de gesto, depois do 1.º quadro).
#[test]
fn the_pure_laws() {
    assert!(holds(ObjectMode::Edit, true, true));
    assert!(!holds(ObjectMode::Edit, false, true), "outra forma");
    assert!(!holds(ObjectMode::Edit, true, false), "sem a ferramenta");
    assert!(!holds(ObjectMode::Object, true, true));
    assert!(adopt(true, false));
    assert!(!adopt(true, true), "já seguia");
    assert!(!adopt(false, false), "sem a ferramenta");
    assert!(releases(false, true));
    assert!(!releases(true, true) && !releases(false, false));
    assert_eq!(newborn(&[1, 2], Some(&[1]), &[]), Some(2));
    assert_eq!(newborn(&[1, 2], Some(&[1]), &[2]), None, "ainda em gesto");
    assert_eq!(
        newborn(&[1, 2, 3], Some(&[1]), &[]),
        None,
        "várias de uma vez"
    );
    assert_eq!(newborn(&[1, 2], None, &[]), None, "antes do 1.º quadro");
    assert_eq!(newborn(&[1], Some(&[1]), &[]), None, "nada de novo");
}

/// ⭐⭐ GATE (escolha do dono, 05/10) — **o fluxo**: *Add ▸ Vector Drawing* não cria NADA e põe a
/// ferramenta na mão; a 1.ª forma nasce objecto e entra em Edit; a 2.ª é OUTRO objecto e o Edit passa
/// a ela SEM largar a ferramenta (o `DrawMode` escolhido sobrevive) e sem repetir o aviso.
/// (Mutação: o `leave` voltar a largar a ferramenta ⇒ o `DrawMode` volta ao de fábrica ⇒ RED.)
#[test]
fn add_arms_the_tool_and_each_drawn_shape_becomes_the_object_in_edit() {
    let mut c = cena();
    c.quadro(None);
    let antes = c.sim.world().entities().len();
    assert!(crate::object_add::add(
        crate::object_add::VECTOR,
        &mut c.vec
    ));
    c.quadro(None);
    assert!(c.na_mao(), "o Add não pôs a ferramenta na mão");
    assert_eq!(
        c.sim.world().entities().len(),
        antes,
        "o Add criou uma entidade"
    );
    assert_eq!(c.em_edit(), None, "sem forma não há Edit");
    tool_mut(&mut c.tools)
        .expect("na mão")
        .set_mode(DrawMode::Pencil);
    let a = c.forma(0.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a), "a 1.ª forma não entrou em Edit");
    assert_eq!(c.hero.gizmo.selection, Some(a));
    assert_eq!(
        c.vec.edit.editing(&c.vec.entities).map(|v| v.len()),
        Some(1)
    );
    let b = c.forma(3.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b), "o Edit não passou à 2.ª forma");
    assert_eq!(c.hero.gizmo.selection, Some(b));
    assert!(c.na_mao(), "a passagem largou a ferramenta");
    assert_eq!(
        tool_mut(&mut c.tools).expect("na mão").mode(),
        DrawMode::Pencil,
        "a ferramenta foi largada e retomada (o modo de desenho perdeu-se)"
    );
    assert_eq!(c.avisos_de_entrada(), 1, "o aviso do Edit repetiu-se");
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), None);
    assert!(!c.na_mao(), "Object não largou a ferramenta");
}

/// ⭐⭐ GATE — **uma forma que nasce SEM a ferramenta na mão não pede o Edit** (colar, desfazer, a
/// Hierarquia em Object): o Edit é do gesto do vetor. CONTROLO: com a ferramenta, a mesma forma pede.
#[test]
fn a_shape_born_without_the_tool_does_not_ask_for_edit() {
    let mut c = cena();
    c.quadro(None);
    let a = c.forma(0.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), None, "pediu o Edit sem a ferramenta");
    assert!(!c.na_mao());
    c.vec.edit.arm();
    c.quadro(None);
    let b = c.forma(3.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b), "controlo: com a ferramenta pede");
    assert_ne!(a, b);
}

/// ⭐⭐ GATE (spec/06 F3) — **duas formas, Edit numa, a outra intocada**: só as formas do Edit se
/// agarram (a vista), a outra não é «tida em mãos», e o `Tab` sobre a outra entra nela.
#[test]
fn two_shapes_edit_in_one_and_the_other_is_untouched() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    let id_of = |c: &Cena, bits: u64| {
        c.vec
            .entities
            .iter()
            .find(|(_, x)| **x == bits)
            .map(|(id, _)| *id)
    };
    assert_eq!(
        c.vec.edit.editing(&c.vec.entities),
        Some(id_of(&c, a).into_iter().collect())
    );
    let mut fam = Family::new(&mut c.vec, &c.sim);
    assert!(fam.holds(ObjectMode::Edit, a, &mut c.tools));
    assert!(!fam.holds(ObjectMode::Edit, b, &mut c.tools));
    assert_eq!(fam.parts(a), None, "uma forma só: o cadeado é exacto");
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b));
}

/// ⭐ GATE — **multi-objecto**: `Tab` com duas formas seleccionadas = Edit das duas (a activa à
/// frente), e as duas agarram-se.
#[test]
fn tab_with_two_shapes_edits_both() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b), "o activo é o ÚLTIMO");
    assert_eq!(c.vec.edit.objects, vec![b, a]);
    assert_eq!(
        c.vec.edit.editing(&c.vec.entities).map(|v| v.len()),
        Some(2)
    );
}

/// ⭐ GATE — **a porta antiga**: a ferramenta que chega à mão SEM o modo (a aba de cima, as cenas
/// `PH2D_*_SMOKE`) pede o Edit sobre a forma que a caneta tem seleccionada; sem selecção, espera.
#[test]
fn the_vector_tool_arriving_without_the_mode_asks_for_the_selected_shape() {
    let mut c = cena();
    let a = c.forma(0.0);
    c.quadro(None);
    assert!(c.tools.set_active(&ToolId::new(VECTOR)));
    c.quadro(None);
    assert_eq!(c.em_edit(), None, "sem selecção, espera");
    let id = *c.vec.entities.keys().next().expect("a forma");
    c.vec.pen.select_many(&[id]);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a));
}
