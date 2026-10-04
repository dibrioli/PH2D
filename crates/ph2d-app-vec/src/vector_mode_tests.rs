//! Os gates de Vector ▸ Edit — o lado da família, com a ferramenta do vetor DE VERDADE; o quadro
//! tem os seus com famílias falsas (`ph2d_editor_core::screens::hero::mode_drive`, o multi-objecto
//! em `a_mode_that_joins_takes_the_selected_of_the_same_kind`).

use super::*;
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::tool::Tool;

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

const A: u64 = 100;
const B: u64 = 101;

/// Duas formas no mapa forma↔entidade e as duas ferramentas (a de omissão na mão).
fn duas_formas() -> (VecState, ToolRegistry, VecPathId, VecPathId) {
    let mut scene = ph2d_vec_scene::VecScene::new();
    let a = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(ph2d_vec_scene::rectangle([2.0, 0.0], [3.0, 1.0]));
    let mut vec = VecState::default();
    vec.entities.insert(a, A);
    vec.entities.insert(b, B);
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(VectorTool::default()));
    tools.activate_default();
    (vec, tools, a, b)
}

fn edit(entity: u64) -> Option<ActiveMode> {
    Some(ActiveMode {
        entity,
        mode: ObjectMode::Edit,
    })
}

/// ⭐ GATE — a partição do `DrawMode` (D3): o Edit são as `EDIT_TOOLS`, e só elas; cada variante tem
/// um modo. ⛔ Uma ferramenta nova que mexa na forma no lugar tem de entrar nas duas.
#[test]
fn the_edit_tools_are_exactly_the_draw_modes_of_edit() {
    for m in DrawMode::ALL {
        assert_eq!(
            DrawMode::EDIT_TOOLS.contains(m),
            m.object_mode() == ObjectMode::Edit,
            "{m:?}"
        );
    }
    assert!(DrawMode::EDIT_TOOLS.contains(&DrawMode::Node));
    for criar in [
        DrawMode::Pen,
        DrawMode::Pencil,
        DrawMode::Shape,
        DrawMode::Text,
    ] {
        assert_eq!(
            criar.object_mode(),
            ObjectMode::Object,
            "{criar:?} cria um objecto"
        );
    }
}

/// ⭐ GATE — as três leis, puras.
#[test]
fn the_three_laws() {
    let node = Some(DrawMode::Node);
    let pen = Some(DrawMode::Pen);
    assert!(holds(ObjectMode::Edit, true, node));
    assert!(!holds(ObjectMode::Edit, false, node), "outra forma");
    assert!(
        !holds(ObjectMode::Edit, true, pen),
        "a caneta não é do Edit"
    );
    assert!(!holds(ObjectMode::Edit, true, None), "sem a ferramenta");
    assert!(adopt(node, false));
    assert!(!adopt(node, true), "já seguia");
    assert!(!adopt(pen, false), "a caneta não pede o Edit");
    assert!(releases(false, true));
    assert!(!releases(true, true) && !releases(false, false));
}

/// ⭐⭐ GATE (spec/06 F3) — **duas formas, Edit numa, a outra intocada**: entrar põe o Node na mão
/// e prende a vista à forma; a outra fica fora; sair larga a ferramenta e solta a vista.
#[test]
fn two_shapes_edit_in_one_and_the_other_is_untouched() {
    let (mut vec, mut tools, a, b) = duas_formas();
    let mut fam = Family::new(&mut vec);
    assert!(fam.enter(ObjectMode::Edit, A, &mut tools));
    assert_eq!(tool_in_hand(&mut tools), Some(DrawMode::Node));
    assert!(fam.holds(ObjectMode::Edit, A, &mut tools));
    assert!(!fam.holds(ObjectMode::Edit, B, &mut tools));
    assert_eq!(fam.parts(A), Some(vec![A]));
    assert_eq!(fam.parts(B), None);
    assert_eq!(vec.edit.paths, vec![a]);
    assert!(!vec.edit.paths.contains(&b));
    let mut fam = Family::new(&mut vec);
    fam.leave(ObjectMode::Edit, A, &mut tools);
    assert!(vec.edit.paths.is_empty(), "a vista ficou presa");
    assert_eq!(
        tool_in_hand(&mut tools),
        None,
        "a ferramenta do Edit ficou na mão"
    );
}

/// ⭐⭐ GATE — **o multi-objecto**: as formas que o quadro junta entram no Edit e são as partes.
#[test]
fn the_joined_shapes_enter_together() {
    let (mut vec, mut tools, a, b) = duas_formas();
    let mut fam = Family::new(&mut vec);
    assert!(fam.joins(ObjectMode::Edit));
    assert!(fam.enter_with(ObjectMode::Edit, B, &[A], &mut tools));
    assert_eq!(fam.parts(B), Some(vec![B, A]));
    assert_eq!(vec.edit.paths, vec![b, a]);
}

/// ⭐⭐ GATE — **a ferramenta segue o modo**: um Edit que ACABOU larga a ferramenta do Edit; a de
/// criar que o artista pegou no trilho (e que acabou o modo) fica; e o `Tab` volta à última
/// ferramenta do Edit.
#[test]
fn the_tool_follows_the_mode() {
    let (mut vec, mut tools, _, _) = duas_formas();
    let mut fam = Family::new(&mut vec);
    assert!(fam.enter(ObjectMode::Edit, A, &mut tools));
    tool_mut(&mut tools)
        .expect("na mão")
        .set_mode(DrawMode::Width);
    fam.follow(edit(A), &mut tools);
    fam.follow(None, &mut tools);
    assert_eq!(
        tool_in_hand(&mut tools),
        None,
        "o Width ficou na mão em Object"
    );
    assert!(fam.enter(ObjectMode::Edit, A, &mut tools));
    assert_eq!(
        tool_in_hand(&mut tools),
        Some(DrawMode::Width),
        "o Edit não voltou à última ferramenta"
    );
    fam.follow(edit(A), &mut tools);
    tool_mut(&mut tools)
        .expect("na mão")
        .set_mode(DrawMode::Pen);
    fam.follow(None, &mut tools);
    assert_eq!(
        tool_in_hand(&mut tools),
        Some(DrawMode::Pen),
        "a caneta do trilho saiu"
    );
}

/// ⭐⭐ GATE — **a porta antiga**: o Node na mão sem o modo pede o Edit sobre a forma seleccionada;
/// sem forma, espera (não arranca a ferramenta); a caneta não pede nada.
#[test]
fn an_edit_tool_arriving_without_the_mode_asks_for_it() {
    let (mut vec, mut tools, _, b) = duas_formas();
    tools.set_active(&ToolId::new(VECTOR));
    tool_mut(&mut tools)
        .expect("na mão")
        .set_mode(DrawMode::Node);
    let mut fam = Family::new(&mut vec);
    assert_eq!(fam.wants(&mut tools), None, "sem forma seleccionada, pediu");
    vec.pen.select(Some(b));
    let mut fam = Family::new(&mut vec);
    assert_eq!(fam.wants(&mut tools), Some((B, ObjectMode::Edit)));
    tool_mut(&mut tools)
        .expect("na mão")
        .set_mode(DrawMode::Pen);
    assert_eq!(fam.wants(&mut tools), None, "a caneta pediu o Edit");
}

/// ⭐⭐ GATE — **a ferramenta que o menu Add pediu entra quando a ferramenta chega à mão**: antes
/// disso o pedido ESPERA (o `ActivateTool` só se aplica depois do dreno), e depois é gasto uma vez.
#[test]
fn the_add_menu_tool_enters_when_the_vector_tool_is_in_hand() {
    let (mut vec, mut tools, _, _) = duas_formas();
    assert!(crate::object_add::arm(crate::object_add::PENCIL, &mut vec));
    Family::new(&mut vec).follow(None, &mut tools);
    assert_eq!(vec.edit.armed, Some(DrawMode::Pencil), "o pedido perdeu-se antes da mão");
    assert!(tools.set_active(&ToolId::new(VECTOR)));
    Family::new(&mut vec).follow(None, &mut tools);
    assert_eq!(tool_in_hand(&mut tools), Some(DrawMode::Pencil));
    assert_eq!(vec.edit.armed, None, "o pedido ficou para o próximo quadro");
}
