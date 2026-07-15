//! Os gates da POLÍTICA do Blend (a math já está gateada no motor, `ph2d-vec-blend`).
//!
//! O que se prova aqui é o que o artista vê acontecer com o DOCUMENTO:
//!
//! 1. as **fontes sobrevivem** (é um Blend, não uma booleana — ela é que consome os operandos);
//! 2. os passos nascem **entre** elas, no z delas;
//! 3. cada chamada é **UM** passo de undo;
//! 4. sem duas formas fechadas, não faz nada (e não corrompe a cena).
//!
//! (Este é o modelo DESTRUTIVO legado — só os smokes `PH2D_BUILD_SMOKE=7/8/9` o chamam. O painel
//! usa o Blend Object VIVO; os gates dele estão em `blend_live_tests.rs`.)

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
    steps: u32,
) -> ph2d_vec_edit::History {
    let mut history = ph2d_vec_edit::History::default();
    apply(scene, &mut history, pen, xf, session, steps, true);
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
    run(&mut scene, &mut pen, &xf, &mut session, 3);

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

// (O "Rotate Match" e o "Reverse Match" foram REMOVIDOS 2026-07-14: os dois eram bugs de design (o
// Reverse invertia o winding e colapsava a forma; o Rotate rodava a correspondência às cegas). A
// correspondência é 100% automática; o ajuste, no modelo vivo, é editar as formas-fonte. O gate do
// winding oposto vive no motor: `ph2d_vec_blend::tests::opposite_winding_does_not_collapse_the_middle`.)

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
        3,
        true,
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
    run(&mut scene, &mut pen, &xf, &mut session, 3);
    assert!(session.is_none() && scene.paths().len() == 3);

    pen.select_many(&[a, _b, c]); // três
    run(&mut scene, &mut pen, &xf, &mut session, 3);
    assert!(session.is_none() && scene.paths().len() == 3);
}

/// **A SEQUÊNCIA de z é o resultado, não um efeito colateral de quem nasceu primeiro.**
///
/// O smoke do Enio: *"a primeira forma criada, que deveria ficar por cima do círculo original,
/// ficou por baixo"*. A pilha que o blend pede vai da fonte de trás, pelos passos, até a da
/// frente — e o checkbox inverte a coisa inteira.
#[test]
fn the_blend_asks_for_the_whole_sequence_in_z() {
    let (mut scene, xf, mut pen, a, b) = two_shapes();
    let mut session = None;
    run(&mut scene, &mut pen, &xf, &mut session, 3);

    let s = session.as_ref().expect("a sessão");
    let up = s.stack();
    assert_eq!(
        up.first(),
        Some(&a),
        "com Stack Up, a fonte de trás é o FUNDO"
    );
    assert_eq!(up.last(), Some(&b), "e a da frente é o TOPO");
    assert_eq!(
        up[1..up.len() - 1],
        s.produced[..],
        "os passos vão no meio, em ordem"
    );

    // O checkbox desligado inverte a pilha INTEIRA (fontes inclusas).
    let mut down = s.clone();
    down.stack_up = false;
    let d = down.stack();
    let mut expect: Vec<VecPathId> = up.clone();
    expect.reverse();
    assert_eq!(
        d, expect,
        "sem Stack Up, cada passo nasce ABAIXO do anterior"
    );
}

/// **O 2º Blend NÃO é recusado — e o slider de Steps volta a valer.**
///
/// O gesto é o primeiro que qualquer artista faz: blenda, acha que ficou pouco, **arrasta Steps**,
/// clica **Blend** de novo. Isso não funcionava, e a causa é uma frase que deixou de ser verdade:
/// *"o Run exige exatamente duas formas fechadas selecionadas"*. **Depois de um Blend, o que está
/// selecionado são os PASSOS** (o `select_many` no fim do `apply`) — então o Run seguinte não
/// achava as duas fontes, recusava, e imprimia *"selecione exatamente DUAS regioes fechadas"* num
/// terminal que o artista não está lendo. Na tela: **nada acontecia**.
///
/// Enquanto a seleção for a que a sessão produziu, o artista está iterando no MESMO blend.
#[test]
fn the_second_run_re_runs_the_open_blend_instead_of_refusing_it() {
    let (mut scene, xf, mut pen, a, b) = two_shapes();
    let mut session = None;

    run(&mut scene, &mut pen, &xf, &mut session, 3);
    assert_eq!(scene.paths().len(), 5, "2 fontes + 3 passos");
    // É o `apply` que deixa os PASSOS selecionados — o gate não arruma a cena para si.
    assert_eq!(
        pen.selected_paths().len(),
        3,
        "depois do Blend, o selecionado são os passos (é isto que quebrava o 2º Run)"
    );

    // O artista arrasta Steps para 6 e clica Blend. Sem tocar na seleção.
    run(&mut scene, &mut pen, &xf, &mut session, 6);

    let s = session.as_ref().expect("a sessão continua aberta");
    assert_eq!(s.produced.len(), 6, "o Steps novo tem de valer");
    assert_eq!((s.a, s.b), (a, b), "as fontes são as MESMAS");
    assert_eq!(
        scene.paths().len(),
        8,
        "2 fontes + 6 passos — os 3 passos velhos foram TROCADOS, não empilhados"
    );
    for id in [a, b] {
        assert!(
            scene.paths().iter().any(|p| p.id == id),
            "a fonte {id} sumiu — o Blend não consome os operandos"
        );
    }
}

/// **Com `Steps = 2`, o 2º Blend comia os PRÓPRIOS PASSOS — em silêncio.**
///
/// Este é o caso perverso, e ele é pior que a recusa: com dois passos, o que ficava selecionado
/// **era** *"exatamente duas formas fechadas"*. O Run seguinte então achava um par válido — **os
/// dois passos** — e blendava ELES, jogando as fontes fora do cálculo. Nada avisava; a arte só
/// derretia para o meio a cada clique.
///
/// É o gate que prova que a pergunta certa não é *"há duas fechadas?"* e sim *"de quem são as
/// fontes?"*.
#[test]
fn with_two_steps_the_second_run_does_not_blend_its_own_steps() {
    let (mut scene, xf, mut pen, a, b) = two_shapes();
    let mut session = None;

    run(&mut scene, &mut pen, &xf, &mut session, 2);
    let first = session.as_ref().expect("sessão").produced.clone();
    assert_eq!(first.len(), 2);
    assert_eq!(
        pen.selected_paths().len(),
        2,
        "a seleção é DUAS formas fechadas — e elas são os passos, não as fontes"
    );

    run(&mut scene, &mut pen, &xf, &mut session, 2);

    let s = session.as_ref().expect("sessão");
    assert_eq!(
        (s.a, s.b),
        (a, b),
        "as fontes viraram os PASSOS do blend anterior — o 2º Run blendou o próprio resultado"
    );
    assert_eq!(
        scene.paths().len(),
        4,
        "2 fontes + 2 passos (não 2 + 2 + 2)"
    );
}
