//! Os gates da POLÍTICA do Blend (a math já está gateada no motor, `ph2d-vec-blend`).
//!
//! O que se prova aqui é o que o artista vê acontecer com o DOCUMENTO:
//!
//! 1. as **fontes sobrevivem** (é um Blend, não uma booleana — ela é que consome os operandos);
//! 2. os passos nascem **entre** elas, no z delas;
//! 3. o **escape re-roda** (Rotate/Reverse trocam os passos por outros, sem o artista desfazer);
//! 4. cada ação é **UM** passo de undo;
//! 5. sem duas formas fechadas, o botão **não faz nada** (e não corrompe a cena).

use super::*;
use ph2d_vec_scene::{ShapeKind, cook};

/// Uma forma do catálogo, na pose identidade (o `xforms` vazio ⇒ mundo = local).
fn shape(scene: &mut VecScene, kind: ShapeKind, c: [f64; 2], r: f64, params: &[f64]) -> VecPathId {
    scene.push_path(cook(
        kind,
        [c[0] - r, c[1] - r],
        [c[0] + r, c[1] + r],
        params,
    ))
}

/// Cena: um quadrado e um círculo, separados. Devolve (cena, xforms, pen, a, b).
fn two_shapes() -> (
    VecScene,
    VecXforms,
    ph2d_vec_edit::PenTool,
    VecPathId,
    VecPathId,
) {
    let mut scene = VecScene::new();
    let a = shape(&mut scene, ShapeKind::Rectangle, [0.0, 0.0], 1.0, &[]);
    let b = shape(&mut scene, ShapeKind::Ellipse, [6.0, 0.0], 1.0, &[]);
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[a, b]);
    (scene, VecXforms::new(), pen, a, b)
}

fn run(
    scene: &mut VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    xf: &VecXforms,
    session: &mut Option<BlendSession>,
    action: BlendAction,
    steps: u32,
) -> ph2d_vec_edit::History {
    let mut history = ph2d_vec_edit::History::default();
    apply(scene, &mut history, pen, xf, session, action, steps);
    history
}

/// **As fontes sobrevivem, e os passos nascem ENTRE elas.**
///
/// É a diferença entre o Blend e a booleana: a booleana **consome** os operandos; o blend deixa
/// as duas formas onde estavam e põe os intermediários no meio do caminho — inclusive no **z**.
#[test]
fn the_sources_survive_and_the_steps_are_born_between_them() {
    let (mut scene, xf, mut pen, a, b) = two_shapes();
    let mut session = None;
    run(&mut scene, &mut pen, &xf, &mut session, BlendAction::Run, 3);

    let ids: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    assert!(
        ids.contains(&a) && ids.contains(&b),
        "as fontes foram comidas"
    );
    assert_eq!(ids.len(), 5, "2 fontes + 3 passos");

    let (za, zb) = (
        ids.iter().position(|i| *i == a).unwrap(),
        ids.iter().position(|i| *i == b).unwrap(),
    );
    let produced = session
        .as_ref()
        .expect("a sessão ficou aberta")
        .produced
        .clone();
    for id in &produced {
        let z = ids.iter().position(|i| i == id).unwrap();
        assert!(
            z > za && z < zb,
            "o passo caiu fora do intervalo de z das fontes (z={z}, fontes em {za} e {zb})"
        );
    }
}

/// **O escape RE-RODA** — e é isso que o torna usável.
///
/// Um blend destrutivo de tiro único obrigaria o artista a: blend → ver o erro → Ctrl+Z → mexer
/// num número → blend de novo, às cegas. Aqui, *Rotate Match* troca os passos por outros **na
/// hora**: mesmos ids de fonte, mesma contagem, geometria diferente.
#[test]
fn rotate_match_replaces_the_steps_in_place() {
    let (mut scene, xf, mut pen, ..) = two_shapes();
    let mut session = None;
    run(&mut scene, &mut pen, &xf, &mut session, BlendAction::Run, 2);
    let before: Vec<[f64; 2]> = session
        .as_ref()
        .unwrap()
        .produced
        .iter()
        .filter_map(|id| scene.paths().iter().find(|p| p.id == *id))
        .map(|p| p.verts[0].anchor)
        .collect();
    let count_before = scene.paths().len();

    run(
        &mut scene,
        &mut pen,
        &xf,
        &mut session,
        BlendAction::Rotate,
        2,
    );

    let s = session.as_ref().expect("a sessão continua aberta");
    assert_eq!(s.opts.offset, 1, "o Rotate tem de mexer no `offset`");
    assert_eq!(
        scene.paths().len(),
        count_before,
        "o re-rodar tem de TROCAR os passos, não empilhar outros por cima"
    );
    let after: Vec<[f64; 2]> = s
        .produced
        .iter()
        .filter_map(|id| scene.paths().iter().find(|p| p.id == *id))
        .map(|p| p.verts[0].anchor)
        .collect();
    assert_ne!(
        before, after,
        "o Rotate não mudou a geometria — o escape é decorativo"
    );
}

/// **Reverse Match** também re-roda, e inverte o sentido de percurso de B.
#[test]
fn reverse_match_flips_the_winding_and_re_runs() {
    let (mut scene, xf, mut pen, ..) = two_shapes();
    let mut session = None;
    run(&mut scene, &mut pen, &xf, &mut session, BlendAction::Run, 2);
    let count = scene.paths().len();

    run(
        &mut scene,
        &mut pen,
        &xf,
        &mut session,
        BlendAction::Reverse,
        2,
    );
    let s = session.as_ref().unwrap();
    assert!(s.opts.reverse, "o Reverse tem de inverter o sentido");
    assert_eq!(scene.paths().len(), count, "trocou, não empilhou");
}

/// **Uma ação = UM passo de undo** (inclusive o re-rodar).
///
/// Um Ctrl+Z depois do Blend devolve a cena **como ela estava**: as duas formas, e mais nada. Se
/// a ação tivesse empurrado dois passos, o artista precisaria de dois Ctrl+Z para desfazer um
/// clique — que é exatamente a queixa que abriu esta linha.
#[test]
fn every_action_is_exactly_one_undo_step() {
    let (mut scene, xf, mut pen, ..) = two_shapes();
    let mut session = None;
    let mut history = ph2d_vec_edit::History::default();

    apply(
        &mut scene,
        &mut history,
        &mut pen,
        &xf,
        &mut session,
        BlendAction::Run,
        3,
    );
    assert_eq!(scene.paths().len(), 5);

    let back = history.undo(&scene).expect("UM passo de undo");
    assert_eq!(back.paths().len(), 2, "um Ctrl+Z devolve a cena de antes");
    assert!(
        !history.can_undo(),
        "o Blend empurrou MAIS de um passo — um clique exigiria dois Ctrl+Z"
    );
}

/// Sem exatamente **duas** formas fechadas, o botão não faz nada — e não corrompe a cena.
///
/// (Três formas não têm um "entre" definido. Adivinhar qual par o artista quis seria pior que
/// recusar: ele veria dois passos nascerem no lugar errado e não saberia por quê.)
#[test]
fn without_exactly_two_closed_shapes_the_button_does_nothing() {
    let (mut scene, xf, mut pen, a, _b) = two_shapes();
    let c = shape(&mut scene, ShapeKind::Ellipse, [12.0, 0.0], 1.0, &[]);
    let mut session = None;

    pen.select_many(&[a]); // uma só
    run(&mut scene, &mut pen, &xf, &mut session, BlendAction::Run, 3);
    assert!(session.is_none() && scene.paths().len() == 3);

    pen.select_many(&[a, _b, c]); // três
    run(&mut scene, &mut pen, &xf, &mut session, BlendAction::Run, 3);
    assert!(session.is_none() && scene.paths().len() == 3);
}

/// **Rotate sem um Blend aberto não faz nada** — não há o que rodar, e inventar uma sessão a
/// partir da seleção faria o botão significar duas coisas.
#[test]
fn the_escape_without_an_open_blend_is_a_no_op() {
    let (mut scene, xf, mut pen, ..) = two_shapes();
    let mut session = None;
    run(
        &mut scene,
        &mut pen,
        &xf,
        &mut session,
        BlendAction::Rotate,
        3,
    );
    assert!(session.is_none());
    assert_eq!(scene.paths().len(), 2, "a cena não foi tocada");
}

/// O botão do painel → a ação. O gate anti-item-morto do seam: um id que o painel PINTA e que
/// ninguém traduz é um botão morto (e a suíte inteira fica verde).
#[test]
fn every_painted_blend_button_maps_to_an_action() {
    for (id, want) in [
        (ph2d_editor::ids::VECTOR_BLEND_RUN, BlendAction::Run),
        (ph2d_editor::ids::VECTOR_BLEND_ROTATE, BlendAction::Rotate),
        (ph2d_editor::ids::VECTOR_BLEND_REVERSE, BlendAction::Reverse),
    ] {
        assert_eq!(action_for_id(id), Some(want));
    }
    assert_eq!(action_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION), None);
}
