//! Gates das três operações de nó da W4 ([`super`]) — Join · Average · Reverse.

use crate::PenTool;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VertexKind};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x - 0.3, y - 0.7],
        out_handle: [x + 0.5, y + 0.2],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn open(scene: &mut VecScene, pts: &[[f64; 2]]) -> VecPathId {
    scene.push_path(VecPath {
        verts: pts.iter().map(|p| v(p[0], p[1])).collect(),
        closed: false,
        ..VecPath::default()
    })
}

// ── AVERAGE ─────────────────────────────────────────────────────────────────

#[test]
fn average_collapses_the_selected_nodes_onto_their_centroid() {
    let mut scene = VecScene::default();
    let id = open(&mut scene, &[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.selected_verts = vec![(id, 0), (id, 1)];

    assert!(pen.average_selected_verts(&mut scene));

    let p = &scene.paths()[0];
    assert_eq!(p.verts[0].anchor, [5.0, 0.0]);
    assert_eq!(p.verts[1].anchor, [5.0, 0.0]);
    assert_eq!(p.verts[2].anchor, [10.0, 10.0], "o não-selecionado fica");
}

/// O vértice viaja INTEIRO: a tangente de cada nó sobrevive à média. Se só a âncora andasse, os
/// handles ficariam para trás e a curva daria um nó no ponto que o artista acabou de alinhar.
#[test]
fn average_translates_the_whole_vertex_so_the_tangent_survives() {
    let mut scene = VecScene::default();
    let id = open(&mut scene, &[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
    let before = scene.paths()[0].verts[0];
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.selected_verts = vec![(id, 0), (id, 1)];

    pen.average_selected_verts(&mut scene);

    let after = scene.paths()[0].verts[0];
    let d = [
        after.anchor[0] - before.anchor[0],
        after.anchor[1] - before.anchor[1],
    ];
    assert_eq!(
        after.in_handle,
        [before.in_handle[0] + d[0], before.in_handle[1] + d[1]]
    );
    assert_eq!(
        after.out_handle,
        [before.out_handle[0] + d[0], before.out_handle[1] + d[1]]
    );
}

#[test]
fn average_is_refused_when_there_is_nothing_to_average() {
    let mut scene = VecScene::default();
    let id = open(&mut scene, &[[0.0, 0.0], [10.0, 0.0]]);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    assert!(!pen.average_selected_verts(&mut scene), "0 selecionados");
    pen.selected_verts = vec![(id, 1)];
    assert!(!pen.average_selected_verts(&mut scene), "1 selecionado");
    // Já coincidentes: nada se move ⇒ nenhum passo de undo espúrio.
    let same = scene.push_path(VecPath {
        verts: vec![v(3.0, 3.0), v(3.0, 3.0)],
        closed: false,
        ..VecPath::default()
    });
    pen.select(Some(same));
    pen.selected_verts = vec![(same, 0), (same, 1)];
    assert!(!pen.average_selected_verts(&mut scene), "já no mesmo ponto");
}

/// O par canônico do Illustrator: **Average + Join** solda duas pontas EXATAMENTE. É o gate que
/// justifica a tolerância de solda ser apertada — quem quer coincidência usa o Average.
#[test]
fn average_then_join_welds_two_ends_into_a_single_vertex() {
    let mut scene = VecScene::default();
    let a = open(&mut scene, &[[-5.0, 0.0], [0.0, 0.0]]);
    let b = open(&mut scene, &[[0.4, 0.0], [5.0, 0.0]]);
    let mut pen = PenTool::new();

    // Sem o Average as pontas estão a 0,4 → o Join deixa o segmento entre elas.
    pen.select_many(&[a, b]);
    assert!(pen.join_selection(&mut scene));
    assert_eq!(scene.paths()[0].verts.len(), 4, "as duas pontas ficam");

    // Com o Average elas coincidem → a solda funde num vértice só.
    let mut scene = VecScene::default();
    let a = open(&mut scene, &[[-5.0, 0.0], [0.0, 0.0]]);
    let b = open(&mut scene, &[[0.4, 0.0], [5.0, 0.0]]);
    let mut pen = PenTool::new();
    // ⚠️ Esta metade escrevia as duas âncoras À MÃO, sob a nota *"o Average é por-caminho (a
    // seleção de nó pertence a UM)"*: a fixture CONTORNAVA exatamente a ausência que a seleção
    // com dono removeu. Agora o gesto é o real — escolher a ponta de cada forma e mediar —, que
    // é o par canônico do Illustrator que este teste sempre alegou exercitar.
    pen.select_many(&[a, b]);
    pen.selected_verts = vec![(a, 1), (b, 0)];
    assert!(
        pen.average_selected_verts(&mut scene),
        "o Average atravessa as duas formas"
    );
    assert!(pen.join_selection(&mut scene));
    assert_eq!(scene.paths()[0].verts.len(), 3, "a costura é UM vértice");
}

// ── JOIN ────────────────────────────────────────────────────────────────────

/// ⚠️ **Join NÃO fecha um caminho só.** O `Close Path` da seção PATH já é essa metade — e a
/// auditoria desta wave achou-o já lá. Uma segunda porta para "fechar" divergiria dele no
/// primeiro refino; o que o Join deu ao `Close Path` foi a SOLDA, não um botão paralelo.
#[test]
fn join_with_fewer_than_two_paths_is_refused_because_closing_has_its_own_button() {
    let mut scene = VecScene::default();
    let id = open(&mut scene, &[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]]);
    let mut pen = PenTool::new();
    pen.select(Some(id));

    assert!(!pen.join_selection(&mut scene), "um caminho só: não é Join");
    assert!(!scene.paths()[0].closed, "e nada foi fechado por acidente");
}

/// Três pedaços numa cadeia — cada solda re-lê o SOBREVIVENTE. Guardar o 1º id como alvo fixo
/// tentaria soldar no 3º um objeto que já não existe.
///
/// ⚠️ **A ORDEM da seleção é o que torna este gate afiado.** Quem sobrevive a uma solda é o de
/// mais BAIXO na pilha de z, então selecionar de baixo para cima faria o sobrevivente coincidir
/// com `ids[0]` em todo passo — re-ler e não re-ler dariam o mesmo resultado, e a fixture não
/// conteria o fenômeno. Aqui a seleção vai do TOPO para o fundo, e cada solda muda o alvo.
#[test]
fn join_folds_three_selected_paths_into_one_chain() {
    let mut scene = VecScene::default();
    let low = open(&mut scene, &[[4.0, 0.0], [5.0, 0.0]]);
    let mid = open(&mut scene, &[[2.0, 0.0], [3.0, 0.0]]);
    let high = open(&mut scene, &[[0.0, 0.0], [1.0, 0.0]]);
    let mut pen = PenTool::new();
    pen.select_many(&[high, mid, low]);

    assert!(pen.join_selection(&mut scene));

    assert_eq!(scene.paths().len(), 1, "três viraram um");
    let xs: Vec<f64> = scene.paths()[0].verts.iter().map(|v| v.anchor[0]).collect();
    // ⚠️ O oráculo é a cadeia ser MONOTÔNICA e completa, não a direção dela: qual das duas pontas
    // vira o começo cai da orientação que a solda escolheu, e o artista nunca pediu uma delas —
    // é literalmente por isso que o botão Reverse existe nesta mesma seção. Cravar `[0..5]` seria
    // pinar um acidente e ficaria vermelho sobre produto correto.
    let monotone = xs.windows(2).all(|w| w[0] < w[1]) || xs.windows(2).all(|w| w[0] > w[1]);
    assert!(monotone, "a cadeia não pode dar nó: {xs:?}");
    let mut sorted = xs.clone();
    sorted.sort_by(f64::total_cmp);
    assert_eq!(sorted, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0], "nada se perdeu");
    assert_eq!(pen.selected(), Some(scene.paths()[0].id), "segue o vivo");
}

/// ⚠️ **Defesa em camadas, e o registro é honesto:** no ramo de 2+ o `select` final também limpa
/// a seleção de nó, então a limpeza explícita do `join_selection` é a camada que responde quando
/// NENHUM par solda (o `select` não corre) — e é esse o caso que este gate dirige.
#[test]
fn a_join_that_welds_nothing_still_drops_the_vertex_selection() {
    let mut scene = VecScene::default();
    let a = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0)],
        closed: true,
        ..VecPath::default()
    });
    let b = scene.push_path(VecPath {
        verts: vec![v(4.0, 0.0), v(5.0, 0.0), v(5.0, 1.0)],
        closed: true,
        ..VecPath::default()
    });
    let mut pen = PenTool::new();
    pen.select_many(&[a, b]);
    pen.selected_verts = vec![(a, 0), (b, 2)];

    assert!(!pen.join_selection(&mut scene), "dois fechados: nada solda");
    assert!(
        pen.selected_verts().is_empty(),
        "a seleção de nó não sobrevive a um gesto de topologia"
    );
}

#[test]
fn join_with_nothing_selected_is_refused() {
    let mut scene = VecScene::default();
    open(&mut scene, &[[0.0, 0.0], [1.0, 0.0]]);
    let mut pen = PenTool::new();
    assert!(!pen.join_selection(&mut scene));
}

// ── REVERSE ─────────────────────────────────────────────────────────────────

#[test]
fn reverse_turns_every_selected_path_and_drops_the_stale_vertex_selection() {
    let mut scene = VecScene::default();
    let a = open(&mut scene, &[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]);
    let b = open(&mut scene, &[[9.0, 0.0], [8.0, 0.0]]);
    let mut pen = PenTool::new();
    pen.select_many(&[a, b]);
    pen.selected_verts = vec![(a, 0)];

    assert!(pen.reverse_selected_paths(&mut scene));

    assert_eq!(scene.paths()[0].verts[0].anchor, [2.0, 0.0]);
    assert_eq!(scene.paths()[1].verts[0].anchor, [8.0, 0.0]);
    assert!(
        pen.selected_verts().is_empty(),
        "os índices descreviam a ordem antiga"
    );
}

#[test]
fn reverse_with_nothing_selected_is_refused() {
    let mut scene = VecScene::default();
    open(&mut scene, &[[0.0, 0.0], [1.0, 0.0]]);
    let mut pen = PenTool::new();
    assert!(!pen.reverse_selected_paths(&mut scene));
}

/// A guarda do divisor: índices de vértice ESTALE (nenhum resolve) davam `0/0 = NaN`, e o NaN
/// viajava para toda âncora selecionada. É a única metade load-bearing da guarda do Average.
#[test]
fn average_with_only_stale_indices_never_writes_a_nan() {
    let mut scene = VecScene::default();
    let id = open(&mut scene, &[[1.0, 2.0], [3.0, 4.0]]);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.selected_verts = vec![(id, 50), (id, 51)];

    assert!(!pen.average_selected_verts(&mut scene));

    for v in &scene.paths()[0].verts {
        assert!(v.anchor[0].is_finite() && v.anchor[1].is_finite(), "NaN");
    }
    assert_eq!(scene.paths()[0].verts[0].anchor, [1.0, 2.0], "intocado");
}
