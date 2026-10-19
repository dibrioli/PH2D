//! W23 B3 (a) — a letra do mapa muda-se pela RAIZ e escala o mapa todo (o Miro, doc 04 §1.1).

use super::*;
use crate::{Dir, Handle};

/// ⭐ Na raiz, a letra nova escala TODOS os nós (letra e tamanho — o que o moldador real mede para
/// eles), o layout abre na mesma proporção, e é UM passo de desfazer; num nó que não é raiz não
/// muda nada.
#[test]
fn the_roots_font_scales_the_whole_map_in_one_step() {
    let (mut w, root) = W::new();
    let a = w.child("Goals");
    let a1 = w.child("Ship the beta");
    w.select(root);
    let b = w.child("Risks");
    w.select(root);
    let before: Vec<Element> = [root, a, a1, b]
        .iter()
        .map(|id| w.el(*id).clone())
        .collect();
    let gap = |w: &W| w.el(a).x - (w.el(root).x + w.el(root).w);
    let gap0 = gap(&w);
    w.ed.set_font_size(&mut w.doc, &mut w.h, 28.0);
    for (id, was) in [root, a, a1, b].iter().zip(&before) {
        let el = w.el(*id).clone();
        assert_eq!(el.style().font_size, 2.0 * was.style().font_size, "{id:?}");
        let fit = crate::mindmap::refit(&mut w.ts, &el);
        assert!(
            (fit.w - el.w).abs() < 0.5 && (fit.h - el.h).abs() < 0.5,
            "{id:?}: {}×{} contra o medido {}×{}",
            el.w,
            el.h,
            fit.w,
            fit.h
        );
    }
    assert!(
        (gap(&w) - 2.0 * gap0).abs() < 0.5,
        "o vão {} contra {}",
        gap(&w),
        2.0 * gap0
    );
    let c = w.el(root).center();
    assert!(
        (c[0] - before[0].center()[0]).abs() < 1e-6,
        "a raiz fica no sítio"
    );
    assert!(w.h.undo(&mut w.doc));
    for (id, was) in [root, a, a1, b].iter().zip(&before) {
        let el = w.el(*id);
        assert_eq!(el.style().font_size, was.style().font_size);
        assert!(
            (el.x - was.x).abs() < 1e-6 && (el.y - was.y).abs() < 1e-6,
            "{id:?}"
        );
    }
    w.select(a);
    let was = w.el(a).clone();
    w.ed.set_font_size(&mut w.doc, &mut w.h, 40.0);
    assert_eq!(
        w.el(a).style().font_size,
        was.style().font_size,
        "só pela raiz"
    );
}

/// ⭐ W23 B3 (b) — o Miro ao PAIRAR: sobre um nó (sem nada seleccionado) aparecem o «+» do filho e,
/// se ele tem filhos à vista, o «−» na borda de onde saem os ramos; clicar no «−» recolhe o ramo e
/// selecciona o nó. Uma folha não tem «−».
#[test]
fn hovering_a_node_shows_its_plus_and_a_minus_that_collapses() {
    let (mut w, root) = W::new();
    let a = w.child("Goals");
    let a1 = w.child("Ship the beta");
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.select(&w.doc, []);
    let over = w.el(a).center();
    w.ed.hover(&w.doc, at(over, NONE));
    let o = w.ed.overlay(&mut w.doc, &mut w.ts);
    assert_eq!(o.mind_plus.len(), 1, "o «+» do nó pairado");
    let edge = [w.el(a).x + w.el(a).w, w.el(a).center()[1]];
    assert_eq!(o.mind_minus, Some((edge, [1.0, 0.0])), "o «−» na borda de fora");
    let off = crate::mind_minus_offset(w.ed.metrics());
    w.click([edge[0] + off, edge[1]], NONE);
    assert!(w.el(a).mind().unwrap().collapsed, "o «−» recolheu");
    assert!(w.el(a1).is_hidden());
    assert_eq!(w.sel(), BTreeSet::from([a]));
    // Uma folha não tem «−»; a raiz também não (recolhe-se ramo a ramo).
    w.ed.select(&w.doc, []);
    for id in [a1, root] {
        let c = w.el(id).center();
        w.ed.hover(&w.doc, at(c, NONE));
        let o = w.ed.overlay(&mut w.doc, &mut w.ts);
        assert_eq!(o.mind_minus, None, "{id:?}");
    }
}

/// ⭐ Smoke do dono (19/10): com o nó SELECCIONADO a moldura pinta a pega de redimensionar no meio da
/// borda — o «−» ficava em cima dela. O «−» (onde se pinta e onde se agarra) fica a duas pegas de
/// toda pega da moldura e do «+»; clicar nele recolhe, clicar na pega ainda redimensiona.
#[test]
fn the_minus_of_a_selected_node_stays_clear_of_the_frame_handles() {
    let (mut w, _) = W::new();
    let a = w.child("Ideas");
    let _ = w.child("Sketch");
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.select(a);
    let m = *w.ed.metrics();
    let o = w.ed.overlay(&mut w.doc, &mut w.ts);
    let frame = o.frame.expect("a moldura do nó seleccionado");
    let (edge, d) = o.mind_minus.expect("o «−» do nó seleccionado");
    let off = crate::mind_minus_offset(&m);
    let minus = [edge[0] + d[0] * off, edge[1] + d[1] * off];
    let far = |p: [f64; 2], what: &str| {
        let gap = (p[0] - minus[0]).hypot(p[1] - minus[1]);
        assert!(gap >= 2.0 * m.handle, "{what}: {gap} px do «−»");
    };
    for h in std::iter::once(Handle::Rotate).chain(Dir::ALL.into_iter().map(Handle::Resize)) {
        far(frame.handle(h, 1.0, &m), &format!("{h:?}"));
    }
    for (p, d) in &o.mind_plus {
        far([p[0] + d[0] * m.dot, p[1] + d[1] * m.dot], "o «+»");
    }
    let e = frame.handle(Handle::Resize(Dir::E), 1.0, &m);
    assert_eq!(
        w.ed.handle_at(&w.doc, at(e, NONE)),
        Some(Handle::Resize(Dir::E)),
        "a pega continua a ser da moldura"
    );
    w.click(minus, NONE);
    assert!(w.el(a).mind().unwrap().collapsed, "o «−» recolheu");
}
