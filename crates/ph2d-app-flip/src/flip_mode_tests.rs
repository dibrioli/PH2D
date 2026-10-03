//! Os gates de Flip ▸ Draw · Edit — o lado da família, com o documento e a ferramenta Flip DE
//! VERDADE; o quadro tem os seus com famílias falsas (`ph2d_editor_core::screens::hero::mode_drive`).

use super::*;
use crate::autokey::{FlipEdit, target_drawing};
use ph2d_core::{Playhead, Vec2};
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::tool::Tool;
use ph2d_flip::{FlipStroke, Hold, KeyKind};

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

/// Dois desenhos com uma chave e um traço cada, o mapa desenho↔entidade, e as duas ferramentas
/// (a de omissão na mão).
fn dois_desenhos() -> (FlipDoc, FlipState, ToolRegistry) {
    let mut doc = FlipDoc::new();
    let mut state = FlipState::default();
    for (name, bits) in [("A", A), ("B", B)] {
        let oid = doc.push_object(name);
        let o = doc.object_mut(oid).expect("recém-criado");
        o.fps = 12.0;
        let l = o.add_layer("L");
        let did = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .expect("a chave");
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 1.0));
        o.drawing_mut(did).expect("o desenho").strokes.push(s);
        state.entities.insert(oid, bits);
    }
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(FlipTool::default()));
    tools.activate_default();
    (doc, state, tools)
}

fn id_of(doc: &FlipDoc, name: &str) -> FlipObjectId {
    doc.objects()
        .iter()
        .find(|o| o.name == name)
        .expect("o desenho")
        .id
}

/// O quadro `f` (12 fps).
fn at(f: i32) -> Playhead {
    let mut p = Playhead::new(1.0 / 12.0);
    p.pause();
    p.seek(f64::from(f) / 12.0);
    p
}

fn set_tool(tools: &mut ToolRegistry, m: FlipMode) {
    tools
        .tool_by_id_mut(&ToolId::new(FLIP))
        .and_then(|t| t.as_any_mut().downcast_mut::<FlipTool>())
        .expect("a ferramenta Flip")
        .set_mode(m);
}

/// ⭐⭐ GATE — o «tem em mãos»: o desenho do modo é o alvo E a ferramenta na mão é do modo. ⚠️ Os
/// CONTROLOS são as duas linhas verdadeiras: sem elas o gate passaria com `false` sempre.
#[test]
fn holds_needs_the_target_and_a_tool_of_the_mode() {
    assert!(
        holds(ObjectMode::Draw, true, Some(FlipMode::Fill)),
        "controlo"
    );
    assert!(
        holds(ObjectMode::Edit, true, Some(FlipMode::Reshape)),
        "controlo"
    );
    assert!(
        !holds(ObjectMode::Draw, false, Some(FlipMode::Draw)),
        "outro desenho é o alvo"
    );
    assert!(
        !holds(ObjectMode::Draw, true, None),
        "a ferramenta saiu da mão"
    );
    assert!(
        !holds(ObjectMode::Draw, true, Some(FlipMode::Edit)),
        "uma ferramenta do Edit no Draw"
    );
    assert!(
        !holds(ObjectMode::Object, true, Some(FlipMode::Draw)),
        "Object não é desta família"
    );
}

/// ⭐⭐ GATE — a porta antiga: a ferramenta na mão sem o modo dela pede-o; a que já o tem, não; e a
/// que trocou de grupo pede o outro.
#[test]
fn a_tool_in_hand_without_its_mode_asks_for_it() {
    assert_eq!(adopt(Some(FlipMode::Erase), None), Some(ObjectMode::Draw));
    assert_eq!(adopt(Some(FlipMode::Edit), None), Some(ObjectMode::Edit));
    assert_eq!(
        adopt(Some(FlipMode::Draw), Some(ObjectMode::Draw)),
        None,
        "já está no modo — pedir de novo re-entraria a cada quadro"
    );
    assert_eq!(
        adopt(Some(FlipMode::Reshape), Some(ObjectMode::Draw)),
        Some(ObjectMode::Edit)
    );
    assert_eq!(adopt(None, None), None, "sem a ferramenta, nada a pedir");
}

/// ⭐⭐ GATE — a ferramenta sai só quando um modo do Flip ACABOU; a que chegou agora espera o pedido
/// dela.
#[test]
fn the_tool_leaves_only_when_a_mode_ended() {
    assert!(releases(None, Some(ObjectMode::Draw)));
    assert!(
        !releases(None, None),
        "a ferramenta que uma cena acabou de pôr na mão seria arrancada antes de pedir o modo"
    );
    assert!(!releases(Some(ObjectMode::Edit), Some(ObjectMode::Draw)));
}

/// ⭐⭐⭐ GATE (spec/06 §4 F3) — **dois desenhos, Draw num: o gesto cai nele, e o outro fica
/// intocado.** ⚠️ O CONTROLO é o Object: ali nenhum gesto cai — o defeito de antes (o 1.º objecto,
/// SEMPRE) daria o A sem modo nenhum.
#[test]
fn two_drawings_draw_on_one_and_the_other_is_untouched() {
    let (mut doc, mut state, mut tools) = dois_desenhos();
    let (a, b) = (id_of(&doc, "A"), id_of(&doc, "B"));
    assert_eq!(
        target_drawing(
            &mut doc,
            &at(5),
            state.target,
            &mut state.strip,
            FlipEdit::Draw
        ),
        None,
        "controlo: em Object nenhum gesto cai"
    );

    let mut f = Family::new(&mut state, &doc);
    assert!(
        f.enter(ObjectMode::Draw, B, &mut tools),
        "entra em Draw no B"
    );
    assert!(f.holds(ObjectMode::Draw, B, &mut tools));
    assert!(
        !f.holds(ObjectMode::Draw, A, &mut tools),
        "o A não está em mãos"
    );
    assert_eq!(state.target.object, Some(b));

    let antes = doc.object(a).cloned();
    let (oid, _, _) = target_drawing(
        &mut doc,
        &at(5),
        state.target,
        &mut state.strip,
        FlipEdit::Draw,
    )
    .expect("o gesto cai num desenho");
    assert_eq!(oid, b, "o traço caiu no desenho em edição");
    assert_eq!(doc.object(a).cloned(), antes, "o A ficou intocado");
}

/// ⭐⭐ GATE — entrar põe a ferramenta do modo na mão; trocar de modo lembra a última ferramenta de
/// cada um; sair tira-a e larga o alvo.
#[test]
fn entering_a_mode_hands_its_tool_and_remembers_the_last_one() {
    let (doc, mut state, mut tools) = dois_desenhos();
    let mut f = Family::new(&mut state, &doc);
    assert!(f.enter(ObjectMode::Draw, A, &mut tools));
    assert_eq!(
        tool_in_hand(&mut tools),
        Some(FlipMode::Draw),
        "a 1.ª do Draw"
    );
    set_tool(&mut tools, FlipMode::Fill);
    f.follow(
        Some(ActiveMode {
            entity: A,
            mode: ObjectMode::Draw,
        }),
        &mut tools,
    );

    f.leave(ObjectMode::Draw, A, &mut tools);
    assert_eq!(tool_in_hand(&mut tools), None, "Object: a ferramenta sai");
    assert!(f.enter(ObjectMode::Edit, A, &mut tools));
    assert_eq!(
        tool_in_hand(&mut tools),
        Some(FlipMode::Edit),
        "a 1.ª do Edit"
    );

    f.leave(ObjectMode::Edit, A, &mut tools);
    assert!(f.enter(ObjectMode::Draw, A, &mut tools));
    assert_eq!(
        tool_in_hand(&mut tools),
        Some(FlipMode::Fill),
        "o Draw volta com a ferramenta que o artista tinha"
    );

    f.leave(ObjectMode::Draw, A, &mut tools);
    assert_eq!(state.target.object, None, "Object: nenhum alvo");
}

/// ⭐ GATE — trocar de desenho larga a camada e as chaves marcadas do anterior (o multiframe agiria
/// à distância).
#[test]
fn another_drawing_drops_the_layer_and_the_marked_keys() {
    let (doc, mut state, mut tools) = dois_desenhos();
    let a = id_of(&doc, "A");
    state.target = ph2d_flip::FlipTarget::on(a, Some(doc.object(a).expect("A").layers()[0].id));
    state.strip.selection = vec![0, 4];
    let mut f = Family::new(&mut state, &doc);
    assert!(f.enter(ObjectMode::Draw, A, &mut tools));
    assert!(
        state.target.layer.is_some(),
        "controlo: o mesmo desenho guarda a camada"
    );
    let mut f = Family::new(&mut state, &doc);
    assert!(f.enter(ObjectMode::Draw, B, &mut tools));
    assert_eq!(state.target.layer, None);
    assert!(state.strip.selection.is_empty());
}

/// ⭐⭐ GATE — a ferramenta segue o modo: o modo do Flip que acaba (por qualquer porta: a rede
/// `still_holds`, o desenho apagado) leva a ferramenta; a que uma cena acabou de pôr na mão fica
/// à espera do pedido dela.
#[test]
fn the_tool_follows_the_mode() {
    let (doc, mut state, mut tools) = dois_desenhos();
    tools.set_active(&ToolId::new(FLIP));
    let mut f = Family::new(&mut state, &doc);
    f.follow(None, &mut tools);
    assert_eq!(
        tool_in_hand(&mut tools),
        Some(FlipMode::Draw),
        "a porta antiga: ainda sem modo, a ferramenta fica"
    );
    assert_eq!(f.wants(&mut tools), Some((A, ObjectMode::Draw)), "e pede-o");

    assert!(f.enter(ObjectMode::Draw, B, &mut tools));
    let em_b = Some(ActiveMode {
        entity: B,
        mode: ObjectMode::Draw,
    });
    f.follow(em_b, &mut tools);
    assert_eq!(f.wants(&mut tools), None, "no modo, nada a pedir");
    f.follow(None, &mut tools);
    assert_eq!(
        tool_in_hand(&mut tools),
        None,
        "o modo acabou: a ferramenta sai"
    );
    assert_eq!(state.target.object, None);
}

/// ⭐⭐ GATE — o desenho que NASCE pelo menu Add pede o Draw (escolha do dono), uma vez, e espera a
/// entidade dele.
#[test]
fn a_born_drawing_asks_for_draw_once_it_has_an_entity() {
    let (doc, mut state, mut tools) = dois_desenhos();
    let b = id_of(&doc, "B");
    let bits = state.entities.remove(&b).expect("B");
    state.born = Some(b);
    let mut f = Family::new(&mut state, &doc);
    assert_eq!(f.wants(&mut tools), None, "sem entidade ainda: espera");
    f.state.entities.insert(b, bits);
    assert_eq!(f.wants(&mut tools), Some((B, ObjectMode::Draw)));
    assert_eq!(f.wants(&mut tools), None, "uma vez só");
}
