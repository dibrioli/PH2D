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

/// ⭐ GATE — **o Add não entra no Edit de uma forma VELHA**: a caneta com uma selecção antiga não
/// pode puxar o Edit pela porta antiga quando a ferramenta chega à mão — o Add espera o 1.º traço.
/// CONTROLO: a mesma selecção com a ferramenta posta à mão (sem o Add) entra
/// (`the_vector_tool_arriving_without_the_mode_asks_for_the_selected_shape`).
/// (Mutação: o `follow` não limpar a caneta ao armar ⇒ RED.)
#[test]
fn add_never_enters_the_edit_of_a_stale_pen_selection() {
    let mut c = cena();
    c.forma(0.0);
    c.quadro(None);
    let velha = *c.vec.entities.keys().next().expect("a forma");
    c.vec.pen.select_many(&[velha]);
    c.vec.edit.arm();
    c.quadro(None);
    c.quadro(None);
    assert!(c.na_mao(), "o Add não pôs a ferramenta na mão");
    assert_eq!(c.em_edit(), None, "o Add entrou no Edit da forma velha");
}

/// ⭐⭐ GATE (report do dono, 05/10: *«em edit mode se clicar no canvas vazio (desselecionar) sai do
/// modo Edit. Não permita isso»*) — **limpar a selecção num Edit não sai dele**: ela volta à forma (e
/// às duas, num Edit de duas). CONTROLO: a forma APAGADA ainda faz o modo cair a Object.
/// (Mutação: apagar o passo 0b do `mode_drive::drive` ⇒ RED.)
#[test]
fn clearing_the_selection_in_edit_keeps_the_edit() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    c.hero.gizmo.replace_selection(None);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a), "o clique no vazio saiu do Edit");
    assert_eq!(
        c.hero.gizmo.selection,
        Some(a),
        "a selecção não voltou à forma"
    );
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(None);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b));
    assert_eq!(c.hero.gizmo.selected_len(), 2, "o Edit de duas perdeu uma");
    c.sim.world_mut().despawn(Entity::from_bits(b));
    c.hero.gizmo.replace_selection(None);
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        None,
        "controlo: a forma apagada não larga o modo"
    );
}

/// ⭐⭐ GATE (report do dono, 05/10: *«ao fazer um boolean o gizmo já não aparece»*) — **a forma que a
/// booleana deixa fica em Edit COM o gizmo**: as duas do Edit somem, nasce uma, o Edit passa a ela e
/// o gizmo de objecto mostra-se (a forma trancada é a própria selecção).
/// (Mutação: o `object_gizmo_shows` voltar a excluir o objecto trancado ⇒ RED.)
#[test]
fn the_shape_a_boolean_leaves_is_in_edit_with_its_gizmo() {
    use ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows;
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b));
    // O caminho do programa: a CANETA selecciona as duas no Edit (e a shell copia-a para a
    // selecção) — elas juntam-se ao Edit, e o cadeado não o larga.
    let ids: Vec<VecPathId> = c.vec.entities.keys().copied().collect();
    c.vec.pen.select_many(&ids);
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b), "a selecção da caneta largou o Edit");
    assert!(
        c.vec.edit.objects.contains(&a),
        "a forma da caneta não entrou no Edit"
    );
    assert!(object_gizmo_shows(&c.hero), "controlo: duas formas em Edit");
    // A booleana: as duas saem da cena (e do mapa), nasce o resultado.
    let ids: Vec<VecPathId> = c.vec.entities.keys().copied().collect();
    for id in ids {
        let e = c.vec.entities.remove(&id).expect("a forma");
        c.sim.world_mut().despawn(Entity::from_bits(e));
        c.scene.remove_path(id);
    }
    // ⚠️ No programa a forma nova só ganha entidade no quadro SEGUINTE (a sincronia): o Edit cai
    // num quadro e o resultado nasce no outro — com tudo num quadro só este gate passava e a foto não.
    c.quadro(None);
    assert!(
        c.na_mao(),
        "o Edit caiu com as formas consumidas e largou a ferramenta"
    );
    let u = c.forma(1.0);
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(u),
        "o Edit não passou à forma da booleana"
    );
    assert!(c.na_mao());
    assert!(
        object_gizmo_shows(&c.hero),
        "a forma da booleana ficou sem gizmo"
    );
}

/// ⭐ GATE — **uma selecção VELHA da caneta não entra no Edit seguinte**: as formas que a caneta
/// selecciona juntam-se ao Edit, mas só as que ela seleccionou DENTRO dele — o `Tab` noutra forma
/// abre o Edit dessa forma só.
/// (Mutação: o `enter_with` não podar a caneta ⇒ RED.)
#[test]
fn a_stale_pen_selection_never_joins_the_next_edit() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    let id_a = c
        .vec
        .entities
        .iter()
        .find(|(_, x)| **x == a)
        .map(|(id, _)| *id)
        .expect("a");
    c.vec.pen.select_many(&[id_a]);
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(Some(ModeRequest::Toggle));
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b));
    assert_eq!(
        c.vec.edit.objects,
        vec![b],
        "a selecção velha entrou no Edit"
    );
}
