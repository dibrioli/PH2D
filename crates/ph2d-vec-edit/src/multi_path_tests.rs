//! **Gates da seleção de nós que ATRAVESSA formas** — irmão de `selection.rs`, o sujeito.
//!
//! O plano 25 §6 nomeava isto como ausência **POR CONSTRUÇÃO**: o `selected_verts` era uma lista
//! de índices planos dentro de um `selected` único, então dois índices de formas diferentes eram
//! indistinguíveis. O que o dono no par (`(VecPathId, usize)`) destrava não é uma feature a mais —
//! é a **remoção de três casos especiais** que existiam só para conter a ausência (a soma que
//! trocava de alvo, o marquee que elegia um caminho, o overlay que só acendia o primário).
//!
//! ⚠️ **O gate que carrega a wave não é o da CONTAGEM, é o do ESPAÇO.** Somar nós de duas formas é
//! visível — se falhar, o artista vê metade da seleção apagar-se. Já mover as duas com um único
//! `delta_to_local` **compila, roda e deforma em silêncio**, com a contagem certa o tempo todo: a
//! forma escalada anda a distância errada e a seleção se desmonta sob o dedo.
//!
//! Ao lado de cada gate novo vive o **CONTROLE**: com uma forma só, tudo é exatamente o que sempre
//! foi. Sem ele, a wave poderia ter trocado o comportamento de todo gesto de nó do editor e ainda
//! assim ficar verde nos gates de multi-forma.

use crate::PenTool;
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState, VecXforms, Xform, rectangle, xform_of};

/// A em `x ∈ [0,1]`, B em `x ∈ [2,3]` — separadas, para que uma caixa que apanhe as duas tenha de
/// atravessar o vão (o gesto real de *"estes cantos e aqueles"*).
fn two_squares() -> (VecScene, PenTool, VecPathId, VecPathId) {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    (scene, PenTool::default(), a, b)
}

/// Onde a âncora `i` de `id` está no MUNDO — o oráculo é montado à mão a partir do `Xform`
/// publicado, e **não** pelas portas de conversão do próprio `PenTool`: um oráculo que chama a
/// função sob teste concorda com ela mesmo quando as duas estão erradas.
fn world_anchor(scene: &VecScene, xf: &VecXforms, id: VecPathId, i: usize) -> [f64; 2] {
    let a = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| p.vert(i))
        .expect("a âncora existe")
        .anchor;
    xform_of(xf, id).apply(a)
}

// ── O ALCANCE ───────────────────────────────────────────────────────────────

/// **A caixa apanha os nós de TODAS as formas que cobre.** Medido antes da troca: **4 de 8**.
#[test]
fn the_box_takes_the_nodes_of_every_shape_it_covers() {
    let (scene, mut pen, a, b) = two_squares();
    pen.box_select(&scene, [-0.5, -0.5], [3.5, 1.5]);
    assert_eq!(pen.selected_verts().len(), 8, "as duas formas inteiras");
    assert!(pen.verts_in(a).count() == 4 && pen.verts_in(b).count() == 4);
    // E a seleção de OBJETO acompanha — quem tem nó escolhido está selecionado.
    assert_eq!(pen.selected_paths(), [a, b]);
}

/// **Somar a segunda forma NÃO expulsa a primeira.** Medido antes: o total ficava em 4 (a soma
/// trocava de alvo), porque *"somar só vale dentro do MESMO caminho"*.
#[test]
fn summing_a_second_shape_keeps_the_first() {
    let (scene, mut pen, a, b) = two_squares();
    pen.box_select_with(&scene, [-0.5, -0.5], [1.5, 1.5], false);
    assert_eq!(pen.selected_verts().len(), 4, "a fixture pegou A inteira");
    pen.box_select_with(&scene, [1.5, -0.5], [3.5, 1.5], true);
    assert_eq!(pen.selected_verts().len(), 8, "B somou, A ficou");
    assert_eq!(pen.verts_in(a).count(), 4, "os nos de A sobreviveram");
    assert_eq!(pen.verts_in(b).count(), 4);
}

/// **A caixa não alcança uma forma ESCONDIDA.** ⚠️ A exigência nasceu com esta wave: enquanto o
/// marquee elegia UM caminho, a falta do `is_pickable` quase nunca era observável; apanhando todas
/// as formas cobertas, uma invisível entraria na seleção em silêncio — e o Delete seguinte
/// apagaria nós que ninguém vê.
#[test]
fn the_box_does_not_reach_a_hidden_or_locked_shape() {
    let (scene, mut pen, a, b) = two_squares();
    pen.set_view(VecViewState {
        hidden: vec![b],
        ..Default::default()
    });
    pen.box_select(&scene, [-0.5, -0.5], [3.5, 1.5]);
    assert_eq!(pen.verts_in(a).count(), 4, "a visível responde");
    assert_eq!(pen.verts_in(b).count(), 0, "a escondida NAO entra");

    let (scene, mut pen, a, b) = two_squares();
    pen.set_view(VecViewState {
        locked: vec![b],
        ..Default::default()
    });
    pen.box_select(&scene, [-0.5, -0.5], [3.5, 1.5]);
    assert_eq!((pen.verts_in(a).count(), pen.verts_in(b).count()), (4, 0));
}

// ── O ESPAÇO ────────────────────────────────────────────────────────────────

/// **Cada forma anda no SEU espaço local, e o resultado é o mesmo deslocamento de MUNDO.**
///
/// ⚠️ Este é o gate que a wave existe para ter. A conversão mundo→local é POR FORMA (ADR-0111), e
/// um único `delta_to_local` para todas *compila, roda e tem a contagem certa* — só que a forma
/// escalada 2× anda o DOBRO, e a seleção se desmonta sob o dedo. A mutação (converter uma vez, com
/// o `inv` da forma agarrada) não é vista por nenhum gate de contagem.
#[test]
fn each_shape_moves_in_its_own_frame() {
    let (mut scene, mut pen, a, b) = two_squares();
    // B vive escalada 2× e transladada — A fica na identidade, e é o CONTROLE interno.
    let mut xf = VecXforms::new();
    xf.insert(b, Xform([2.0, 0.0, 0.0, 2.0, 10.0, 0.0]));
    pen.set_xforms(xf.clone());
    pen.selected_verts = vec![(a, 0), (b, 0)];

    let wa = world_anchor(&scene, &xf, a, 0);
    let wb = world_anchor(&scene, &xf, b, 0);
    assert!(pen.nudge(&mut scene, 1.0, 0.0));
    let wa2 = world_anchor(&scene, &xf, a, 0);
    let wb2 = world_anchor(&scene, &xf, b, 0);

    assert_eq!(
        wa2[0] - wa[0],
        1.0,
        "a forma na identidade andou 1 no mundo"
    );
    assert_eq!(
        wb2[0] - wb[0],
        1.0,
        "a forma ESCALADA tambem andou 1 no mundo -- andou {} (2.0 = o delta foi \
         convertido no frame da OUTRA forma)",
        wb2[0] - wb[0]
    );
    assert_eq!((wa2[1] - wa[1], wb2[1] - wb[1]), (0.0, 0.0));
}

/// O mesmo pelo ARRASTO, que é o gesto que o artista de facto faz — o `nudge` é o teclado.
#[test]
fn dragging_a_grouped_anchor_carries_the_other_shape_in_its_own_frame() {
    let (mut scene, mut pen, a, b) = two_squares();
    let mut xf = VecXforms::new();
    xf.insert(b, Xform([2.0, 0.0, 0.0, 2.0, 10.0, 0.0]));
    pen.set_xforms(xf.clone());
    // Os dois nós escolhidos; depois agarra o de A (que está em mundo `[0,0]`).
    pen.selected_verts = vec![(a, 0), (b, 0)];
    let wb = world_anchor(&scene, &xf, b, 0);
    pen.on_press(&mut scene, [0.0, 0.0], 1.0, false, &mut |p| p);
    pen.on_drag(&mut scene, [1.0, 0.0], &mut |p| p);
    pen.on_release();

    assert_eq!(
        world_anchor(&scene, &xf, a, 0)[0],
        1.0,
        "o agarrado seguiu o dedo"
    );
    assert_eq!(
        world_anchor(&scene, &xf, b, 0)[0] - wb[0],
        1.0,
        "o companheiro de OUTRA forma andou o mesmo tanto de mundo"
    );
}

/// **O Average encontra-se no MUNDO.** Mediar coordenadas locais de frames distintos não significa
/// nada: a mutação (média em local) deixa os dois nós **onde estavam**, porque em local os dois
/// valem `[0,0]` — e a média de dois zeros é zero.
#[test]
fn the_average_meets_in_the_world_not_in_a_local_frame() {
    // ⚠️ **As duas formas têm a MESMA geometria local**, e só a POSE as separa — é essa
    // coincidência que torna a fixture capaz de distinguir as duas leis: em local os dois nós
    // valem `[0,0]`, e a média de dois zeros é zero, logo a lei errada **não move nada**.
    //
    // A 1ª versão reusava o `two_squares` (B em `x ∈ [2,3]`), onde os locais DIFEREM — ali a
    // média local também move alguma coisa, e o gate mediria a lei certa contra uma resposta
    // meramente diferente em vez de contra um no-op. A fixture não continha o fenômeno.
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let mut pen = PenTool::default();
    let mut xf = VecXforms::new();
    xf.insert(b, Xform([2.0, 0.0, 0.0, 2.0, 10.0, 0.0]));
    pen.set_xforms(xf.clone());
    pen.select_many(&[a, b]);
    pen.selected_verts = vec![(a, 0), (b, 0)];
    let la = scene.path(a).expect("a").vert(0).expect("v").anchor;
    let lb = scene.path(b).expect("b").vert(0).expect("v").anchor;
    assert_eq!(
        la, lb,
        "a fixture perdeu a premissa: os locais têm de coincidir"
    );
    assert_eq!(world_anchor(&scene, &xf, a, 0), [0.0, 0.0]);
    assert_eq!(world_anchor(&scene, &xf, b, 0), [10.0, 0.0]);

    assert!(pen.average_selected_verts(&mut scene));
    assert_eq!(
        world_anchor(&scene, &xf, a, 0),
        [5.0, 0.0],
        "encontraram-se no meio"
    );
    assert_eq!(world_anchor(&scene, &xf, b, 0), [5.0, 0.0]);
}

// ── O QUE SE FAZ COM ELA ────────────────────────────────────────────────────

/// **Apagar atravessa formas, e a que MORRE não leva as outras.** Antes, o caminho único que
/// esvaziava zerava a seleção inteira e voltava — não havia outras para sobreviver.
#[test]
fn deleting_across_shapes_leaves_the_survivors_alone() {
    let (mut scene, mut pen, a, b) = two_squares();
    // ⚠️ A premissa é DECLARADA: nenhum gesto do produto deixa nós escolhidos sem a forma deles na
    // seleção de objeto, e uma fixture que atribui só os nós alcança um estado que o produto não
    // produz — o gate mediria a limpeza da seleção a partir de um lugar onde não há o que limpar.
    pen.select_many(&[a, b]);
    // Um nó de A (a forma sobrevive: 4 − 1 = 3) e TRÊS de B (sobra 1 ⇒ contorno degenerado).
    pen.selected_verts = vec![(a, 0), (b, 0), (b, 1), (b, 2)];
    assert!(pen.delete_selected_vertex(&mut scene));

    let survivor = scene.path(a).expect("A sobreviveu");
    assert_eq!(survivor.total_verts(), 3, "A perdeu UM no'");
    assert!(scene.path(b).is_none(), "B esvaziou e saiu da cena");
    assert_eq!(pen.selected_paths(), [a], "e saiu tambem da selecao");
    assert_eq!(pen.selected(), Some(a), "o primario caiu no sobrevivente");
}

/// **Retipar alcança as duas formas** — e só os nós escolhidos.
#[test]
fn retyping_reaches_every_shape_the_selection_touches() {
    let (mut scene, mut pen, a, b) = two_squares();
    pen.selected_verts = vec![(a, 0), (b, 2)];
    assert!(pen.set_selected_vertex_kind(&mut scene, ph2d_vec_scene::VertexKind::Smooth));
    let k = |id, i| scene.path(id).expect("p").vert(i).expect("v").kind;
    assert_eq!(k(a, 0), ph2d_vec_scene::VertexKind::Smooth);
    assert_eq!(k(b, 2), ph2d_vec_scene::VertexKind::Smooth);
    assert_eq!(
        k(a, 1),
        ph2d_vec_scene::VertexKind::Corner,
        "so' os escolhidos"
    );
    assert_eq!(k(b, 0), ph2d_vec_scene::VertexKind::Corner);
}

/// **`Ctrl+A` cobre as formas selecionadas**, não só a primária — e com UMA é o que sempre foi.
#[test]
fn select_all_covers_every_selected_shape() {
    let (scene, mut pen, a, b) = two_squares();
    pen.select(Some(a));
    assert!(pen.select_all_verts(&scene));
    assert_eq!(pen.selected_verts().len(), 4, "CONTROLE: uma forma, 4 nos");

    pen.select_many(&[a, b]);
    assert!(pen.select_all_verts(&scene));
    assert_eq!(pen.selected_verts().len(), 8, "duas formas, 8 nos");
}

// ── O CONTROLE ──────────────────────────────────────────────────────────────

/// **Com UMA forma, tudo é exatamente o que sempre foi.**
///
/// ⚠️ Sem este gate a wave poderia ter trocado o comportamento de todo gesto de nó do editor — o
/// caso comum, o único que o artista faz o dia inteiro — e ainda assim ficar verde em cada gate de
/// multi-forma acima.
#[test]
fn a_single_shape_behaves_exactly_as_it_always_did() {
    let (mut scene, mut pen, a, _b) = two_squares();

    // Caixa sobre A só: 4 nós, dela.
    pen.box_select(&scene, [-0.5, -0.5], [1.5, 1.5]);
    assert_eq!(pen.selected_verts().len(), 4);
    assert!(pen.selected_verts().iter().all(|&(p, _)| p == a));
    assert_eq!(pen.selected(), Some(a));
    assert_eq!(pen.selected_paths(), [a]);

    // O Tab percorre e SUBSTITUI (não soma ao andar).
    assert!(pen.step_vert_selection(&scene, true));
    assert_eq!(pen.selected_verts().len(), 1);

    // O nudge move só o escolhido, e o resto da forma fica.
    pen.selected_verts = vec![(a, 0)];
    let before: Vec<_> = scene
        .path(a)
        .expect("a")
        .verts_all()
        .map(|v| v.anchor)
        .collect();
    assert!(pen.nudge(&mut scene, 1.0, 0.0));
    let after: Vec<_> = scene
        .path(a)
        .expect("a")
        .verts_all()
        .map(|v| v.anchor)
        .collect();
    assert_eq!(after[0], [before[0][0] + 1.0, before[0][1]]);
    assert_eq!(&after[1..], &before[1..], "os nao-escolhidos ficam");
}
