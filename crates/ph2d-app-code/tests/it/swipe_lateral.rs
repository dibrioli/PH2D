//! O deslizar de lado do trackpad (InputMap onda 3 item 7): sobre o texto é o Shift+roda da mesma grandeza;
//! o vertical puro não mexe na horizontal; o diagonal aplica cada eixo uma vez. Pela porta do hero.

use crate::{AREA, Mesa, centro};
use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll, wheel};
use ph2d_editor_core::screens::hero::document_tabs;

fn largo() -> (Mesa, ph2d_script::ScriptId, (f32, f32)) {
    let texto = format!("{}\n", "x".repeat(400)).repeat(200);
    let (mut m, ids) = Mesa::new(&[("largo.luau", texto.as_str())]);
    m.pede(ids[0]);
    m.quadro();
    document_tabs::follow_surfaces(&mut m.hero);
    m.pinta(ids[0]);
    let (x, y) = centro(AREA);
    (m, ids[0], (x + AREA.w * 0.25, y))
}

/// A roda do rato com o Shift premido ou solto.
fn roda(m: &mut Mesa, (x, y): (f32, f32), dy: f32, shift: bool) {
    m.hero.store.set_shift_held(shift);
    let _ = wheel(&mut m.hero, x, y, dy);
    m.hero.store.set_shift_held(false);
}

#[test]
fn sobre_o_texto_o_deslizar_de_lado_e_o_shift_roda() {
    let (mut g, id, p) = largo();
    let (mut r, ..) = largo();
    assert!(scroll(
        &mut g.hero,
        p.0,
        p.1,
        BoardScroll::Pan([-120.0, 0.0])
    ));
    roda(&mut r, p, -120.0, true);
    let s = g.docs().scroll_of(id).expect("aberto");
    assert!(s.0 > 0.0 && s.1 == 0.0, "rolou de lado: {s:?}");
    assert_eq!(Some(s), r.docs().scroll_of(id), "de lado = Shift+roda");

    let _ = scroll(&mut g.hero, p.0, p.1, BoardScroll::Pan([0.0, -120.0]));
    roda(&mut r, p, -120.0, false);
    let v = g.docs().scroll_of(id).expect("aberto");
    assert_eq!(v.0, s.0, "o vertical puro não rola de lado");
    assert_eq!(Some(v), r.docs().scroll_of(id), "vertical = a roda");

    let _ = scroll(&mut g.hero, p.0, p.1, BoardScroll::Pan([-60.0, -48.0]));
    roda(&mut r, p, -60.0, true);
    roda(&mut r, p, -48.0, false);
    assert_eq!(
        g.docs().scroll_of(id),
        r.docs().scroll_of(id),
        "diagonal = cada eixo uma vez"
    );
}

#[test]
fn fora_da_aba_o_deslizar_de_lado_nao_faz_nada() {
    let (mut m, id, _) = largo();
    let antes = m.docs().scroll_of(id);
    let fora = (AREA.x + AREA.w + 50.0, AREA.y + AREA.h + 50.0); // LITERAL-PX-OK: fora da área
    let _ = scroll(&mut m.hero, fora.0, fora.1, BoardScroll::Pan([-120.0, 0.0]));
    assert_eq!(m.docs().scroll_of(id), antes);
}
