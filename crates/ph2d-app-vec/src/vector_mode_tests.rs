//! Os gates de Vector ▸ Edit — o lado da família, com a ferramenta do vetor e objectos DE VERDADE; o
//! quadro tem os seus com famílias falsas (`ph2d_editor_core::screens::hero::mode_drive`, o
//! multi-objecto em `a_mode_that_joins_takes_the_selected_of_the_same_kind`).

use super::*;
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::tool::Tool;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_entities::entities::{object, sync};

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

/// A cena dos gates: dois objectos, cada um com UMA forma, e as duas ferramentas (a de omissão na
/// mão). Devolve `(sim, cena, vec, ferramentas, [objecto A, objecto B], [forma a, forma b])`.
struct Cena {
    sim: SimWorld,
    scene: ph2d_vec_scene::VecScene,
    vec: VecState,
    tools: ToolRegistry,
    obj: [u64; 2],
    path: [VecPathId; 2],
}

fn dois_objectos() -> Cena {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut vec = VecState::default();
    let a = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(ph2d_vec_scene::rectangle([2.0, 0.0], [3.0, 1.0]));
    sync(&mut sim, &mut scene, &mut vec.entities);
    object::adopt_loose(&mut sim, &vec.entities, None, &[], "Vector");
    let obj_of = |sim: &SimWorld, p| {
        let e = Entity::from_bits(vec.entities[&p]);
        object_of(sim, e).expect("a forma ganhou objecto").to_bits()
    };
    let obj = [obj_of(&sim, a), obj_of(&sim, b)];
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(VectorTool::default()));
    tools.activate_default();
    Cena {
        sim,
        scene,
        vec,
        tools,
        obj,
        path: [a, b],
    }
}

fn edit(entity: u64) -> Option<ActiveMode> {
    Some(ActiveMode {
        entity,
        mode: ObjectMode::Edit,
    })
}

fn bits(c: &Cena, p: VecPathId) -> u64 {
    c.vec.entities[&p]
}

/// ⭐ GATE — as três leis, puras.
#[test]
fn the_three_laws() {
    assert!(holds(ObjectMode::Edit, true, true));
    assert!(!holds(ObjectMode::Edit, false, true), "outro objecto");
    assert!(!holds(ObjectMode::Edit, true, false), "sem a ferramenta");
    assert!(!holds(ObjectMode::Object, true, true));
    assert!(adopt(true, false));
    assert!(!adopt(true, true), "já seguia");
    assert!(!adopt(false, false), "sem a ferramenta");
    assert!(releases(false, true));
    assert!(!releases(true, true) && !releases(false, false));
}

/// ⭐⭐ GATE (spec/06 F3) — **dois objectos, Edit num, o outro intocado**: entrar põe a ferramenta
/// na mão (QUALQUER `DrawMode`: o painel inteiro) e prende a vista às formas DELE; a forma do outro
/// fica fora; sair larga a ferramenta e solta a vista.
#[test]
fn two_objects_edit_in_one_and_the_other_is_untouched() {
    let mut c = dois_objectos();
    let [oa, ob] = c.obj;
    let [a, b] = c.path;
    let (ba, bb) = (bits(&c, a), bits(&c, b));
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert!(fam.enter(ObjectMode::Edit, oa, &mut c.tools));
    assert!(in_hand(&mut c.tools));
    tool_mut(&mut c.tools)
        .expect("na mão")
        .set_mode(DrawMode::Pen);
    assert!(
        fam.holds(ObjectMode::Edit, oa, &mut c.tools),
        "a caneta é do Edit"
    );
    assert!(!fam.holds(ObjectMode::Edit, ob, &mut c.tools));
    let parts = fam.parts(oa).expect("o Edit tem partes");
    assert!(parts.contains(&ba) && !parts.contains(&bb), "{parts:?}");
    assert_eq!(fam.parts(ob), None);
    assert!(
        !fam.enter(ObjectMode::Edit, ba, &mut c.tools),
        "uma forma não é o objecto"
    );
    assert_eq!(c.vec.edit.editing(&c.sim, &c.vec.entities), Some(vec![a]));
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    fam.leave(ObjectMode::Edit, oa, &mut c.tools);
    assert_eq!(
        c.vec.edit.editing(&c.sim, &c.vec.entities),
        None,
        "a vista ficou presa"
    );
    assert!(
        !in_hand(&mut c.tools),
        "a ferramenta ficou na mão em Object"
    );
}

/// ⭐⭐ GATE — **uma forma responde pelo objecto dela** (o seletor e o `Tab` sobre uma linha de
/// forma); o objecto responde por si.
#[test]
fn a_shape_answers_for_its_object() {
    let mut c = dois_objectos();
    let [oa, ob] = c.obj;
    let [a, b] = c.path;
    let (ba, bb) = (bits(&c, a), bits(&c, b));
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert_eq!(fam.owner_of(ba), Some(oa));
    assert_eq!(fam.owner_of(bb), Some(ob));
    assert_eq!(fam.owner_of(oa), None);
}

/// ⭐⭐ GATE — **o multi-objecto**: os objectos que o quadro junta entram no Edit; as formas de todos
/// e os outros objectos são as partes.
#[test]
fn the_joined_objects_enter_together() {
    let mut c = dois_objectos();
    let [oa, ob] = c.obj;
    let [a, b] = c.path;
    let (ba, bb) = (bits(&c, a), bits(&c, b));
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert!(fam.joins(ObjectMode::Edit));
    assert!(fam.enter_with(ObjectMode::Edit, ob, &[oa], &mut c.tools));
    let parts = fam.parts(ob).expect("partes");
    for p in [oa, ba, bb] {
        assert!(parts.contains(&p), "{p} fora das partes {parts:?}");
    }
    assert_eq!(c.vec.edit.objects, vec![ob, oa]);
    let mut editing = c.vec.edit.editing(&c.sim, &c.vec.entities).expect("Edit");
    editing.sort_unstable();
    assert_eq!(editing, vec![a, b]);
}

/// ⭐⭐ GATE — **a ferramenta segue o modo**: um Edit que ACABOU larga a ferramenta (o Object não a
/// tem na mão), e voltar ao Edit devolve a ferramenta que o artista tinha.
#[test]
fn the_tool_follows_the_mode() {
    let mut c = dois_objectos();
    let [oa, _] = c.obj;
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert!(fam.enter(ObjectMode::Edit, oa, &mut c.tools));
    tool_mut(&mut c.tools)
        .expect("na mão")
        .set_mode(DrawMode::Width);
    fam.follow(edit(oa), &mut c.tools);
    fam.follow(None, &mut c.tools);
    assert!(
        !in_hand(&mut c.tools),
        "a ferramenta ficou na mão em Object"
    );
    assert!(c.vec.edit.objects.is_empty());
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert!(fam.enter(ObjectMode::Edit, oa, &mut c.tools));
    assert_eq!(
        tool_mut(&mut c.tools).map(|t| t.mode()),
        Some(DrawMode::Width)
    );
}

/// ⭐⭐ GATE — **a porta antiga**: a ferramenta na mão sem o modo pede o Edit sobre o OBJECTO da
/// forma seleccionada; sem forma, espera (não arranca a ferramenta).
#[test]
fn the_vector_tool_arriving_without_the_mode_asks_for_the_object_of_the_selection() {
    let mut c = dois_objectos();
    let [_, ob] = c.obj;
    let [_, b] = c.path;
    c.tools.set_active(&ToolId::new(VECTOR));
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert_eq!(
        fam.wants(&mut c.tools),
        None,
        "sem forma seleccionada, pediu"
    );
    c.vec.pen.select(Some(b));
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert_eq!(fam.wants(&mut c.tools), Some((ob, ObjectMode::Edit)));
    c.tools.activate_default();
    assert_eq!(fam.wants(&mut c.tools), None, "sem a ferramenta, pediu");
}

/// ⭐⭐ GATE — **o que nasce no Edit é do Edit desde o 1.º quadro**: uma forma ainda SOLTA (em gesto,
/// ou à espera do assentamento) agarra-se e é parte — senão o cadeado derrubaria o modo no clique que
/// a seleccionasse (a lição do O_MODEL).
#[test]
fn a_shape_born_in_the_edit_belongs_to_it_before_it_settles() {
    let mut c = dois_objectos();
    let [oa, _] = c.obj;
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert!(fam.enter(ObjectMode::Edit, oa, &mut c.tools));
    let n = c
        .scene
        .push_path(ph2d_vec_scene::rectangle([5.0, 5.0], [6.0, 6.0]));
    sync(&mut c.sim, &mut c.scene, &mut c.vec.entities);
    let bn = bits(&c, n);
    let editing = c.vec.edit.editing(&c.sim, &c.vec.entities).expect("Edit");
    assert!(editing.contains(&n), "a forma nova não se agarra");
    let mut fam = Family::new(&mut c.vec, &mut c.sim);
    assert!(fam.parts(oa).expect("partes").contains(&bn));
    let into = c.vec.edit.object();
    object::adopt_loose(&mut c.sim, &c.vec.entities, into, &[], "Vector");
    assert_eq!(
        object_of(&c.sim, Entity::from_bits(bn)).map(Entity::to_bits),
        Some(oa)
    );
}

/// ⭐⭐ GATE (spec/06 F3: *«clicar numa forma selecciona o objecto inteiro»*) — **em Object o clique
/// sobe ao objecto vetorial**, sem repetidos e na ordem; o que não é do vetor fica; e em Edit as
/// formas escolhem-se uma a uma (a lista não se mexe).
#[test]
fn in_object_mode_a_click_on_a_shape_names_its_object() {
    let c = dois_objectos();
    let [oa, ob] = c.obj;
    let [a, b] = c.path;
    let (ba, bb) = (bits(&c, a), bits(&c, b));
    let sprite = 9_999_u64;
    let object = ph2d_vec_scene::VecViewState::default();
    let mut hits = vec![bb, sprite, ba, oa];
    lift_to_objects(&c.sim, &object, &mut hits);
    assert_eq!(hits, vec![ob, sprite, oa]);
    let edit = ph2d_vec_scene::VecViewState {
        editing: Some(vec![a]),
        ..Default::default()
    };
    let mut hits = vec![ba, bb];
    lift_to_objects(&c.sim, &edit, &mut hits);
    assert_eq!(hits, vec![ba, bb], "em Edit o clique subiu ao objecto");
}

/// ⭐⭐ GATE (spec/06 F3: *«todas as ferramentas dentro de um objeto»*) — **um envelope criado no
/// Edit fica no objecto, INTEIRO**: o contentor nasce na raiz com as formas, e a regra das soltas
/// leva-o para o objecto do Edit sem as separar (embrulhar cada forma partiria a gaiola).
#[test]
fn an_envelope_made_in_the_edit_stays_whole_in_the_object() {
    let mut c = dois_objectos();
    let [oa, _] = c.obj;
    let [a, _] = c.path;
    let c2 = c
        .scene
        .push_path(ph2d_vec_scene::rectangle([0.0, 2.0], [1.0, 3.0]));
    sync(&mut c.sim, &mut c.scene, &mut c.vec.entities);
    let into = Some(Entity::from_bits(oa));
    object::adopt_loose(&mut c.sim, &c.vec.entities, into, &[], "Vector");
    let container =
        crate::envelope_live::create(&mut c.sim, &mut c.scene, &c.vec.entities, &[a, c2])
            .map(Entity::from_bits)
            .expect("o envelope nasceu");
    object::adopt_loose(&mut c.sim, &c.vec.entities, into, &[], "Vector");
    let parent = |e: Entity| {
        c.sim
            .world()
            .get::<ph2d_ecs::ChildOf>(e)
            .map(|p| p.parent())
    };
    assert_eq!(
        parent(container),
        into,
        "o envelope saiu do objecto do Edit"
    );
    for p in [a, c2] {
        assert_eq!(parent(Entity::from_bits(bits(&c, p))), Some(container));
    }
}
