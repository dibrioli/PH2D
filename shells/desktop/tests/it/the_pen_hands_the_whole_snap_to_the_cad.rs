//! **Arch-gate: a caneta recebe o ENCAIXE inteiro, não só o ponto** (W4.4, `docs/BlenderConstraints/04_W4_cad.md`
//! §3). É o encaixe que diz QUAL vértice a guia acesa alinhou (`SnapAxis::dono`), e sem ele nenhuma regra de CAD nasce
//! e o arrasto não chama o solver. O comportamento vive na crate (`ph2d-vec-edit`, `cad_tests.rs`: o mesmo snap que a
//! shell corre); o que só se afirma pelo FONTE é que a shell chama as portas com encaixe.

fn dispatch() -> String {
    crate::input_text::dispatch()
}

/// Mutação que tem de sangrar: voltar a `on_press(`/`on_drag(` com `.apply(p)` na closure.
#[test]
fn the_pen_press_and_drag_take_the_snap_result() {
    let src = dispatch();
    for (porta, onde) in [
        (
            "let click = self.vec.pen.on_press_encaixe(",
            "o press da caneta",
        ),
        (".on_drag_encaixe(&mut gfx.vec_scene", "o arrasto da caneta"),
    ] {
        assert!(
            src.contains(porta),
            "{onde} não passa o encaixe — falta `{porta}`"
        );
    }
    assert!(
        !src.contains("Some(&mut grid)).apply(p)"),
        "a closure do press devolve o ponto em vez do encaixe"
    );
}
