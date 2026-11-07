//! **Os gates da faixa de quadros** (report do dono 25/10): o tamanho da miniatura e o retorno
//! visual do arrasto. Filhos de [`super`] (a `Mesa`), à parte pelo tecto de linhas.

use super::*;

/// ⭐ Report do dono (25/10): a miniatura da faixa tem de ser um pouco MAIOR que a célula da folha
/// pintada logo acima — na janela de fábrica do teste e na folha do herói (8 × 4).
#[test]
fn a_miniatura_da_faixa_e_maior_que_a_celula_da_folha() {
    let mut m = Mesa::new();
    m.paint();
    let d = m.doc();
    let thumb = d.layout.frames[0].0;
    let cell = d.cell_rect(0);
    assert!(
        thumb.w > cell.w && thumb.h > cell.h,
        "miniatura {}×{} ≤ célula {}×{}",
        thumb.w,
        thumb.h,
        cell.w,
        cell.h
    );
    assert!((thumb.w - thumb.h).abs() < 1e-3, "a miniatura é quadrada");
}

/// O arrasto de `from` pousado em `x`: devolve (o intervalo do marcador PINTADO, o que a largada
/// comitou), e deixa a aba sem arrasto nem edições.
fn arrasto_ate(m: &mut Mesa, from: usize, x: f32) -> (Option<usize>, Vec<AnimFieldEdit>) {
    let l = m.doc().layout.clone();
    let y = centre(l.frames[from].0).1;
    m.pointer(PointerKind::Down, centre(l.frames[from].0));
    m.drag_to((x, y));
    m.paint();
    let marker = m.doc().painted.marker;
    m.pointer(PointerKind::Up, (x, y));
    let pending = std::mem::take(&mut m.doc().pending);
    m.doc().frame_selected = None;
    m.paint();
    (marker, pending)
}

/// ⭐⭐ Report do dono (25/10): arrastar um quadro não mostrava nada. O marcador pintado durante o
/// arrasto é EXACTAMENTE o sítio onde a largada põe o quadro — antes do primeiro, entre cada par,
/// sobre cada metade de cada miniatura e depois do último.
#[test]
fn o_marcador_pintado_e_onde_a_largada_poe_o_quadro() {
    let mut m = Mesa::new();
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, vec![5, 2, 7, 1]));
    m.frame();
    let authored = m.doc().authored_frames();
    let l = m.doc().layout.clone();
    let n = l.frames.len();
    let mut xs = vec![l.strip.x + 1.0]; // LITERAL-PX-OK: um pixel dentro da faixa
    for f in &l.frames {
        xs.push(f.0.x + f.0.w * 0.25); // LITERAL-PX-OK: a metade esquerda
        xs.push(f.0.x + f.0.w * 0.75); // LITERAL-PX-OK: a metade direita
    }
    let last = l.frames[n - 1].0;
    xs.push(last.x + last.w + ph2d_tokens::control_gap_px() * 3.0); // LITERAL-PX-OK: depois do último
    for from in [0, 2] {
        let mut seen = std::collections::BTreeSet::new();
        for &x in &xs {
            let (marker, pending) = arrasto_ate(&mut m, from, x);
            let gap = marker.unwrap_or_else(|| panic!("de {from} a x={x}: nenhum marcador"));
            seen.insert(gap);
            let to = doc::gap_to_index(from, gap);
            let expected = if to == from {
                vec![]
            } else {
                vec![AnimFieldEdit::Frames(0, doc::reorder(&authored, from, to))]
            };
            assert_eq!(pending, expected, "de {from} a x={x}: marcador {gap}");
        }
        assert_eq!(seen.first(), Some(&0), "antes do primeiro");
        assert_eq!(seen.last(), Some(&n), "depois do último");
        assert_eq!(seen.len(), n + 1, "todos os intervalos");
    }
}

/// O quadro arrastado segue a mão como fantasma enquanto o botão está premido, e some ao soltar.
#[test]
fn o_fantasma_segue_a_mao_e_some_ao_largar() {
    let mut m = Mesa::new();
    let l = m.doc().layout.clone();
    let from = centre(l.frames[0].0);
    let to = (centre(l.frames[3].0).0, l.strip.y - ph2d_tokens::ROW_H_PX);
    m.pointer(PointerKind::Down, from);
    m.drag_to(to);
    m.paint();
    assert_eq!(
        m.doc().painted.ghost,
        Some(l.frames[0].0),
        "o fantasma é o quadro pegado"
    );
    m.pointer(PointerKind::Up, to);
    m.paint();
    assert_eq!(
        m.doc().painted,
        doc::DragPaint::default(),
        "nada depois de soltar"
    );
}
