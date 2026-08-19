//! **O CARIMBO POSTAL** — os gates do `preview` do card, cortados do irmão no teto de LOC do
//! shell (600). O corte é por RESPONSABILIDADE: o arquivo pai responde *o que o card LÊ*
//! (readout, hot, massa, o selo da GPU) e este *que figura ele DESENHA*.
//!
//! ⚠️ Os helpers (`flow_scene`, `frame`) vivem no pai e chegam aqui pelo `use super::*` —
//! duplicá-los seria a segunda cópia que diverge no dia em que a cena de teste mudar.

use super::*;

/// **The postage stamp is a bounded, STRIDED subsample** — bounded so the cost of the stamps is
/// a function of how many cards there are, never of how big the streams are (Nuke has to tell
/// you to switch its thumbnails off on a heavy script; a scatter of 96 dots has no such cliff);
/// strided so the SHAPE survives.
///
/// FALSIFIED by `take(96)`: the first 96 points of a 5 000-point spiral are the first eighth of
/// one turn, and the card would show a comma and call it a spiral.
#[test]
fn the_postage_stamp_is_a_bounded_strided_subsample() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("wire");
    // 40 x 40 = 1600 points: far more than the stamp may carry.
    g.set_param(grid, "rows", 40.0);
    g.set_param(grid, "cols", 40.0);
    motion.doc.graph = g;
    motion.sinks = vec![out];

    let s = frame(&mut motion, 0, 0.0);
    let stamp = s.nodes[0].preview.as_ref().expect("the grid has a stamp");
    assert!(
        stamp.len() <= PREVIEW_POINTS,
        "bounded: {} points",
        stamp.len()
    );

    // The stamp SPANS the grid: a `take(96)` of a 1600-point grid would cover the first few
    // rows only, and the stamp's vertical extent would collapse.
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for p in stamp {
        lo = lo.min(p[1]);
        hi = hi.max(p[1]);
    }
    let full: Vec<f32> = motion
        .pump
        .instances
        .iter()
        .map(|i| i.world_pos[1])
        .collect();
    let (fl, fh) = (
        full.iter().copied().fold(f32::MAX, f32::min),
        full.iter().copied().fold(f32::MIN, f32::max),
    );
    assert!(
        (hi - lo) > (fh - fl) * 0.8,
        "the stamp spans the shape ({lo}..{hi} of {fl}..{fh})"
    );
}

/// **O CARIMBO NÃO PODE DESENHAR UMA RETA QUE NÃO EXISTE.**
///
/// ⚠️ **Este gate é um report do Enio, escrito como número.** Os cards de uma cena de blocos
/// 21×21 mostravam um **traço diagonal** enquanto o canvas mostrava manchas — e o gate irmão
/// acima ficava verde, porque *abranger* a grade não é *ter a forma dela*: uma diagonal toca
/// todas as linhas e todas as colunas.
///
/// A causa é ressonância entre o passo e a largura da fileira: com 441 pontos e 21 colunas o
/// passo é 10, e `10·k mod 21` anda **−1 por linha**. Medido, **21 das 45 amostras** caíam
/// numa única diagonal, e só **5** das 21 diagonais eram tocadas.
///
/// ⚠️ **O CONTROLE NEGATIVO está dentro do gate**: ele reconstrói o passo fixo à mão sobre a
/// mesma grade e exige que ELE reprove. Sem isso, a barra poderia ser larga demais e o gate
/// passaria sobre o defeito que existe para apanhar.
#[test]
fn the_postage_stamp_of_a_lattice_is_not_a_diagonal_streak() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("wire");
    // 21 x 21 = 441: a grade da cena `=60`, que é onde o defeito apareceu.
    const COLS: usize = 21;
    g.set_param(grid, "rows", COLS as f32);
    g.set_param(grid, "cols", COLS as f32);
    motion.doc.graph = g;
    motion.sinks = vec![out];

    let s = frame(&mut motion, 0, 0.0);
    let stamp = s.nodes[0].preview.as_ref().expect("the grid has a stamp");

    // As posições do carimbo, de volta a (coluna, linha) — a grade é regular, então o índice
    // sai da própria coordenada e o gate não precisa de saber QUAIS índices foram escolhidos.
    let full: Vec<[f32; 2]> = motion.pump.instances.iter().map(|i| i.world_pos).collect();
    let cell = |p: &[f32; 2]| {
        let k = full
            .iter()
            .position(|q| (q[0] - p[0]).abs() < 1e-4 && (q[1] - p[1]).abs() < 1e-4)
            .expect("o ponto do carimbo veio da grade");
        (k % COLS, k / COLS)
    };
    let worst_diagonal = |pts: &[(usize, usize)]| {
        let mut best = 0usize;
        for d in 0..COLS {
            let up = pts.iter().filter(|(c, r)| (c + r) % COLS == d).count();
            let down = pts
                .iter()
                .filter(|(c, r)| (c + COLS - r % COLS) % COLS == d)
                .count();
            best = best.max(up).max(down);
        }
        best
    };

    let ours: Vec<(usize, usize)> = stamp.iter().map(cell).collect();
    let mine = worst_diagonal(&ours);
    assert!(
        mine * 4 < ours.len(),
        "o carimbo poe {mine} das {} amostras na MESMA diagonal -- isso desenha um traco",
        ours.len()
    );

    // ⚠️ CONTROLE NEGATIVO: o passo fixo de antes, sobre a mesma grade.
    let step = full.len().div_ceil(stamp.len().max(1)).max(1);
    let strided: Vec<(usize, usize)> = (0..full.len())
        .step_by(step)
        .map(|k| (k % COLS, k / COLS))
        .collect();
    let old = worst_diagonal(&strided);
    assert!(
        old * 4 >= strided.len(),
        "o controle negativo (passo fixo) tinha de REPROVAR nesta barra, e poe so' {old} de {} \
         numa diagonal -- a barra esta' larga demais para apanhar o defeito",
        strided.len()
    );
    eprintln!(
        "[carimbo] pior diagonal: nosso {mine}/{}, passo fixo {old}/{}",
        ours.len(),
        strided.len()
    );
}

/// A VALUE node has no positions, so it gets no stamp — its number IS its stamp. An empty box
/// under it would promise a picture that is not coming.
#[test]
fn a_value_node_has_no_postage_stamp() {
    let (mut motion, [_, lfo, _, _]) = flow_scene();
    let s = frame(&mut motion, 0, 0.0);
    let node = s.nodes.iter().find(|n| n.id == lfo.0).expect("in view");
    assert!(node.preview.is_none(), "a value node's stamp is its number");
    assert!(node.readout.is_some(), "…and it has one");
}
