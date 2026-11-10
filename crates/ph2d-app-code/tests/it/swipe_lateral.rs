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

fn roda(m: &mut Mesa, (x, y): (f32, f32), dy: f32, shift: bool) {
    m.hero.store.set_shift_held(shift);
    let _ = wheel(&mut m.hero, x, y, dy);
    m.hero.store.set_shift_held(false);
}

fn pan(m: &mut Mesa, (x, y): (f32, f32), d: [f32; 2]) -> bool {
    scroll(&mut m.hero, x, y, BoardScroll::Pan(d))
}

#[test]
fn sobre_o_texto_o_deslizar_de_lado_e_o_shift_roda() {
    let (mut g, id, p) = largo();
    let (mut r, ..) = largo();
    assert!(pan(&mut g, p, [-120.0, 0.0]));
    roda(&mut r, p, -120.0, true);
    let s = g.docs().scroll_of(id).expect("aberto");
    assert!(s.0 > 0.0 && s.1 == 0.0, "rolou de lado: {s:?}");
    assert_eq!(Some(s), r.docs().scroll_of(id), "de lado = Shift+roda");
    let _ = pan(&mut g, p, [0.0, -120.0]);
    roda(&mut r, p, -120.0, false);
    let v = g.docs().scroll_of(id).expect("aberto");
    assert_eq!(v.0, s.0, "o vertical puro não rola de lado");
    assert_eq!(Some(v), r.docs().scroll_of(id), "vertical = a roda");
    let _ = pan(&mut g, p, [-60.0, -48.0]);
    roda(&mut r, p, -60.0, true);
    roda(&mut r, p, -48.0, false);
    let diag = (g.docs().scroll_of(id), r.docs().scroll_of(id));
    assert_eq!(diag.0, diag.1, "diagonal = cada eixo uma vez");
    let fora = (AREA.x + AREA.w + 50.0, AREA.y + AREA.h + 50.0); // LITERAL-PX-OK: fora da área
    let _ = pan(&mut g, fora, [-120.0, 0.0]);
    assert_eq!(g.docs().scroll_of(id), diag.0, "fora da aba, nada");
}
