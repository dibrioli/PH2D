//! Gates de **ONDE A SELEÇÃO DE NÓS ESTÁ** (plano 25 §9, W6 — o X/Y numérico do nó).
//!
//! O que se prova aqui é a metade de LEITURA da feature — a de escrita é o `nudge`, que já é a
//! porta das setas do teclado e que o campo numérico apenas REUSA. As duas metades falham
//! independentemente: uma mediana lida em LOCAL sobre um `nudge` correto põe o número errado na
//! tela do artista, e a suíte do `nudge` fica verde.
//!
//! ⚠️ **A fixture tem `Transform` NÃO-IDENTIDADE de propósito.** Numa forma na identidade, *local*
//! e *mundo* são o mesmo número, então a leitura crua do vértice — o defeito — é indistinguível da
//! leitura correta. A pose desta fixture escala E translada, então os dois números discordam em
//! todo vértice.

use crate::PenTool;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms, Xform};

/// A pose que separa o certo do errado: escala 2× em x, 3× em y, mais uma translação.
/// Sob ela um vértice local `[1, 1]` mora no mundo em `[12, 25]`.
const POSE: Xform = Xform([2.0, 0.0, 0.0, 3.0, 10.0, 22.0]);

/// Um quadrado unitário selecionado, vestindo a [`POSE`].
fn posed_square() -> (VecScene, PenTool, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let mut pen = PenTool::default();
    pen.select(Some(id));
    let mut xf = VecXforms::new();
    xf.insert(id, POSE);
    pen.set_xforms(xf);
    (scene, pen, id)
}

/// A distância de MUNDO entre os dois nós escolhidos — o oráculo do gate anti-colapso.
fn selected_span_x(scene: &VecScene, pen: &PenTool) -> f64 {
    let sel = pen.selected_verts();
    assert_eq!(sel.len(), 2, "este oráculo mede um PAR");
    let world = |&(pid, i): &(VecPathId, usize)| -> [f64; 2] {
        let p = scene
            .paths()
            .iter()
            .find(|p| p.id == pid)
            .expect("o path selecionado existe");
        POSE.apply(p.verts[i].anchor)
    };
    (world(&sel[0])[0] - world(&sel[1])[0]).abs()
}

/// **A mediana é lida em MUNDO, não na coordenada guardada.**
///
/// Mutação que tem de sangrar: devolver `v.anchor` cru (o defeito) — a quina de cima-direita mede
/// `[1, 1]` no documento e `[12, 25]` na tela, e é a segunda que a régua do artista marca.
#[test]
fn the_selected_anchor_is_reported_in_world() {
    let (scene, mut pen, _) = posed_square();
    pen.box_select(&scene, [11.0, 24.0], [13.0, 26.0]);
    assert_eq!(
        pen.selected_verts().len(),
        1,
        "a caixa tinha de pegar UM nó"
    );

    let got = pen
        .selected_anchor_world(&scene)
        .expect("um nó selecionado tem posição");
    assert!(
        (got[0] - 12.0).abs() < 1e-9 && (got[1] - 25.0).abs() < 1e-9,
        "leu {got:?}, esperado [12, 25] — a coordenada guardada e' [1, 1], e mostra-la seria o \
         numero que discorda da regua sob o proprio no'"
    );
}

/// **Com vários nós escolhidos o número é a MEDIANA, e ela é uma afirmação sobre o conjunto.**
///
/// ⚠️ É a lição que o irmão [`PenTool::selected_vertex_kind`] pagou: ele devolvia o tipo do vértice
/// PRIMÁRIO, o que faz o painel afirmar sobre três nós uma verdade de um só. Uma mediana não tem
/// esse defeito, e é ela que torna o campo utilizável com N > 1 sem colapsar a forma.
#[test]
fn several_nodes_report_their_median() {
    let (scene, mut pen, _) = posed_square();
    // Os dois de baixo: locais [-1,-1] e [1,-1] ⇒ mundo [8,19] e [12,19].
    pen.box_select(&scene, [7.0, 18.0], [13.0, 20.0]);
    assert_eq!(pen.selected_verts().len(), 2, "a caixa tinha de pegar DOIS");

    let got = pen
        .selected_anchor_world(&scene)
        .expect("dois nós têm mediana");
    assert!(
        (got[0] - 10.0).abs() < 1e-9 && (got[1] - 19.0).abs() < 1e-9,
        "leu {got:?}, esperado [10, 19] — o ponto MEDIO dos dois"
    );
}

/// **Sem nó escolhido não há número**, e é isso que faz as duas fileiras sumirem do painel.
///
/// Um `Some([0, 0])` aqui seria pior que a ausência: o painel pintaria duas caixas afirmando que a
/// seleção está na origem.
#[test]
fn no_selection_has_no_position() {
    let (scene, pen, _) = posed_square();
    assert!(
        pen.selected_anchor_world(&scene).is_none(),
        "sem no' escolhido o painel nao tem numero a mostrar"
    );
}

/// **Digitar uma coordenada leva o conjunto para lá — e ele anda JUNTO, sem colapsar.**
///
/// Este é o gate anti-Inkscape: o que o dreno aplica é um DESLOCAMENTO (da mediana até o alvo),
/// então os dois nós de baixo, a 4 unidades de mundo um do outro, continuam a 4 depois do gesto. A
/// mutação que sangra é o modelo do Illustrator generalizado a N — escrever o alvo em CADA nó —,
/// que os junta no mesmo X e destrói a forma.
#[test]
fn typing_a_coordinate_moves_the_set_without_collapsing_it() {
    let (mut scene, mut pen, _) = posed_square();
    pen.box_select(&scene, [7.0, 18.0], [13.0, 20.0]);

    let before = pen.selected_anchor_world(&scene).expect("mediana");
    let span_before = selected_span_x(&scene, &pen);
    assert!(
        (span_before - 4.0).abs() < 1e-9,
        "premissa da fixture: os dois nós distam 4 em mundo (medido {span_before})"
    );

    // O gesto que o dreno faz: alvo menos a mediana, pela porta das setas.
    let target_x = 30.0;
    assert!(pen.nudge(&mut scene, target_x - before[0], 0.0));

    let after = pen.selected_anchor_world(&scene).expect("mediana");
    assert!(
        (after[0] - target_x).abs() < 1e-9,
        "a mediana tinha de pousar em {target_x}, pousou em {}",
        after[0]
    );
    assert!(
        (after[1] - before[1]).abs() < 1e-9,
        "digitar X nao pode mexer no Y (era {}, virou {})",
        before[1],
        after[1]
    );

    let span_after = selected_span_x(&scene, &pen);
    assert!(
        (span_after - 4.0).abs() < 1e-9,
        "os nós COLAPSARAM: distavam 4, agora distam {span_after} — isso e' um ALINHAR disfarcado \
         de coordenada, o modelo que o Inkscape ship e que esta wave recusa"
    );
}
