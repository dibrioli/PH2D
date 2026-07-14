//! Testes dos gestos do Edit Mode (`flip_edit_gesture`), módulo-irmão pelo cap de LOC.

use super::*;
use ph2d_flip::{Fill, FlipStroke, Point, Rgba};

fn line(pts: &[(f32, f32)]) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 4.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s
}

fn drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes = strokes;
    d
}

/// **A caixa pega o traço que ela ATRAVESSA, mesmo sem um vértice dentro.**
///
/// Uma reta longa pode cruzar a caixa inteira sem ter um único ponto nela — e o usuário
/// que desenhou a caixa em cima dela espera pegá-la. "Algum ponto dentro" é o teste
/// ingênuo, e ele erra exatamente no caso mais comum de um desenho de linhas.
///
/// Mutação que sangra: tire o teste de cruzamento de segmento e fique só com o
/// ponto-dentro.
#[test]
fn the_marquee_catches_a_stroke_it_crosses_without_containing_a_vertex() {
    // Uma reta de (-100, 5) a (100, 5): nenhum vértice dentro da caixa 0..10 × 0..10.
    let pts = [Vec2::new(-100.0, 5.0), Vec2::new(100.0, 5.0)];
    assert!(
        stroke_touches_rect(&pts, Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        "a caixa nao pegou a linha que a ATRAVESSA de lado a lado"
    );
    // E uma linha que passa longe não é pega.
    let far = [Vec2::new(-100.0, 50.0), Vec2::new(100.0, 50.0)];
    assert!(!stroke_touches_rect(
        &far,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0)
    ));
}

/// **Marquee simples SUBSTITUI; com Shift, SOMA.**
#[test]
fn the_marquee_replaces_but_with_shift_it_adds() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (5.0, 0.0)]),
        line(&[(100.0, 100.0), (105.0, 100.0)]),
    ]);
    d.strokes[1].selected = true; // já selecionado, LONGE da caixa

    // Sem Shift: a caixa pega o 1º e DESMARCA o 2º.
    assert!(apply_marquee(
        &mut d,
        Vec2::new(-1.0, -1.0),
        Vec2::new(10.0, 10.0),
        false
    ));
    assert_eq!(d.selected_indices(), vec![0]);

    // Com Shift: soma (o 1º continua).
    d.strokes[1].selected = false;
    assert!(apply_marquee(
        &mut d,
        Vec2::new(99.0, 99.0),
        Vec2::new(110.0, 110.0),
        true
    ));
    assert_eq!(d.selected_indices(), vec![0, 1], "o Shift nao SOMOU");
}

/// 🔴 **Mover translada os pontos E OS BURACOS.**
///
/// Um preenchimento carrega os furos dele em anéis próprios (o "O"). Mover só os pontos
/// deixaria os furos para trás e a forma se quebraria — e isso só apareceria no desenho do
/// usuário, meses depois. É a MESMA regra que o Sculpt já obedece.
///
/// Mutação que sangra: tire o laço dos `holes` do `translate_selection`.
#[test]
fn moving_the_selection_carries_the_holes_with_it() {
    let mut s = line(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
    s.closed = true;
    s.fill = Some(Fill {
        color: Rgba::new(0.5, 0.5, 0.5, 1.0),
        opacity: 1.0,
    });
    s.holes = vec![vec![
        Vec2::new(4.0, 4.0),
        Vec2::new(6.0, 4.0),
        Vec2::new(6.0, 6.0),
    ]];
    s.selected = true;
    let mut d = drawing(vec![s]);

    assert!(translate_selection(&mut d, Vec2::new(100.0, 50.0)));

    assert_eq!(d.strokes[0].positions()[0], Vec2::new(100.0, 50.0));
    assert_eq!(
        d.strokes[0].holes[0][0],
        Vec2::new(104.0, 54.0),
        "o BURACO ficou para tras — a forma se quebrou"
    );
}

/// **Um traço NÃO selecionado não anda.** É a fronteira do gesto.
#[test]
fn an_unselected_stroke_never_moves() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)]),
        line(&[(0.0, 10.0), (10.0, 10.0)]),
    ]);
    d.strokes[1].selected = true;
    let still = d.strokes[0].positions().to_vec();

    translate_selection(&mut d, Vec2::new(7.0, 7.0));

    assert_eq!(
        d.strokes[0].positions(),
        &still[..],
        "um traco NAO selecionado foi arrastado junto"
    );
    assert_eq!(d.strokes[1].positions()[0], Vec2::new(7.0, 17.0));
}

/// **Um arrasto trêmulo é um CLIQUE, não um marquee.** Sem o slop, o tremor da mão num
/// clique no vazio desenharia uma caixa de 2 px e o usuário veria a seleção piscar.
#[test]
fn a_shaky_click_is_not_a_marquee() {
    assert!(!passed_slop((100.0, 100.0), (101.0, 101.5)));
    assert!(passed_slop((100.0, 100.0), (100.0, 110.0)));
}

// ── W7.2: mover a arte COMPARTILHADA move a POSE, não a geometria ────────────────

/// Um doc com um objeto, uma camada, um traço no quadro 0 e a chave 5 **instanciada**
/// (mesma arte). Devolve `(doc, oid, lid, did)`.
fn doc_with_instanced_key() -> (
    ph2d_flip::FlipDoc,
    ph2d_flip::FlipObjectId,
    ph2d_flip::LayerId,
    ph2d_flip::DrawingId,
) {
    use ph2d_flip::{DupMode, Hold, KeyKind};
    let mut doc = ph2d_flip::FlipDoc::default();
    let oid = doc.push_object("Flip");
    let obj = doc.object_mut(oid).unwrap();
    let lid = obj.add_layer("L");
    let did = obj
        .insert_frame(lid, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut st = FlipStroke::new();
    st.push_default(Vec2::new(0.0, 0.0));
    st.push_default(Vec2::new(10.0, 0.0));
    st.selected = true; // o gesto de mover age sobre a seleção
    obj.drawing_mut(did).unwrap().strokes.push(st);
    assert!(obj.duplicate_frame(lid, 0, 5, DupMode::Instance));
    (doc, oid, lid, did)
}

/// 🔴 **A correção do smoke do Enio**: *"a instância não pode ser movida sozinha, sempre
/// fica exatamente sobre a outra"*.
///
/// Arrastar numa arte COMPARTILHADA move a **pose da chave** — a arte não é deformada
/// (ela é dos dois quadros) e o gêmeo fica onde estava. Sem esta regra, a instância seria
/// indistinguível de um hold: a mesma imagem, no mesmo lugar, por mais tempo.
///
/// Mutação que sangra: `move_drawing` chamar `translate_selection` também no caso
/// instanciado.
#[test]
fn dragging_shared_art_moves_the_key_pose_not_the_geometry() {
    let (mut doc, oid, lid, did) = doc_with_instanced_key();
    let before: Vec<Vec2> = doc.object(oid).unwrap().drawing(did).unwrap().strokes[0]
        .positions()
        .to_vec();

    // Arrasta a chave 5 (a instância).
    assert!(move_drawing(
        &mut doc,
        oid,
        lid,
        5,
        did,
        Vec2::new(100.0, 0.0)
    ));

    let obj = doc.object(oid).unwrap();
    assert_eq!(
        obj.frame_offset(lid, 5),
        Vec2::new(100.0, 0.0),
        "a instancia nao andou: a pose nao foi escrita"
    );
    assert_eq!(
        obj.frame_offset(lid, 0),
        Vec2::ZERO,
        "o quadro 0 andou junto — as duas chaves seguem uma sobre a outra"
    );
    assert_eq!(
        obj.drawing(did).unwrap().strokes[0].positions(),
        &before[..],
        "a GEOMETRIA compartilhada foi reescrita — o gemeo se moveu por baixo do usuario"
    );
}

/// **O irmão de presença**: arte EXCLUSIVA continua movendo a **geometria**, byte a byte
/// como antes. Sem este gate, o de cima ficaria verde num mundo em que TODO arrasto virou
/// pose — e aí ninguém mais moveria um traço só, dentro de um desenho.
#[test]
fn dragging_exclusive_art_still_moves_the_geometry() {
    use ph2d_flip::{Hold, KeyKind};
    let mut doc = ph2d_flip::FlipDoc::default();
    let oid = doc.push_object("Flip");
    let obj = doc.object_mut(oid).unwrap();
    let lid = obj.add_layer("L");
    let did = obj
        .insert_frame(lid, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut st = FlipStroke::new();
    st.push_default(Vec2::new(0.0, 0.0));
    st.push_default(Vec2::new(10.0, 0.0));
    st.selected = true;
    obj.drawing_mut(did).unwrap().strokes.push(st);

    assert!(move_drawing(
        &mut doc,
        oid,
        lid,
        0,
        did,
        Vec2::new(5.0, 2.0)
    ));

    let obj = doc.object(oid).unwrap();
    assert_eq!(
        obj.drawing(did).unwrap().strokes[0].positions()[0],
        Vec2::new(5.0, 2.0),
        "a geometria nao andou: o caminho comum regrediu"
    );
    assert_eq!(
        obj.frame_offset(lid, 0),
        Vec2::ZERO,
        "escreveu POSE numa arte exclusiva — o documento ganhou estado que ninguem le"
    );
}

/// **Um traço NÃO-selecionado da arte compartilhada anda junto** — e tem de andar: a pose
/// é do QUADRO, e o quadro inteiro se move. (É a diferença honesta entre mover a
/// instância e deformar a arte; quem quer divergir a arte quebra o vínculo.)
#[test]
fn moving_shared_art_carries_every_stroke_of_the_frame() {
    let (mut doc, oid, lid, did) = doc_with_instanced_key();
    // Um 2º traço, NÃO selecionado.
    let mut st = FlipStroke::new();
    st.push_default(Vec2::new(50.0, 50.0));
    st.push_default(Vec2::new(60.0, 50.0));
    doc.object_mut(oid)
        .unwrap()
        .drawing_mut(did)
        .unwrap()
        .strokes
        .push(st);

    assert!(move_drawing(
        &mut doc,
        oid,
        lid,
        5,
        did,
        Vec2::new(100.0, 0.0)
    ));

    // A pose move o quadro inteiro — nada de "metade do desenho ficou para trás".
    assert_eq!(
        doc.object(oid).unwrap().frame_offset(lid, 5),
        Vec2::new(100.0, 0.0)
    );
}
