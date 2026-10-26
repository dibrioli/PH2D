//! **Arch-gate: a caneta recebe o ENCAIXE inteiro** (W4.4, `docs/BlenderConstraints/04_W4_cad.md` §3.4) — é ele que
//! diz QUAL vértice a guia alinhou; o comportamento é gate na crate (`ph2d-vec-edit` `cad_tests.rs`).

/// Mutação que tem de sangrar: voltar a `on_press(`/`on_drag(` com `.apply(p)` na closure.
#[test]
fn the_pen_press_and_drag_take_the_snap_result() {
    let src = crate::input_text::dispatch();
    assert!(
        src.contains("let click = self.vec.pen.on_press_encaixe("),
        "o press"
    );
    assert!(
        src.contains(".on_drag_encaixe(&mut gfx.vec_scene"),
        "o arrasto"
    );
    assert!(
        !src.contains("Some(&mut grid)).apply(p)"),
        "a closure devolve o ponto"
    );
}
