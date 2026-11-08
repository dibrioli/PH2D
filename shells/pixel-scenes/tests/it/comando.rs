//! **P12 R: o Run Command** — as linhas do Pixel na paleta `Ctrl+K` do app (`P12_plano.md` §9.2,
//! oráculo O10): busca difusa, a tecla viva à direita, correr = a tecla, indisponível = inerte.

use ph2d_app_pixel::comando;
use ph2d_editor_core::widget::command_palette::{PaletteModel, top_match};
use ph2d_pixel_keys::{Chord, Context, PixelAction as A};
use winit::keyboard::KeyCode;

use crate::comum::Palco;

fn modelo(p: &mut Palco) -> PaletteModel {
    PaletteModel {
        groups: comando::group(&mut p.app).into_iter().collect(),
        ..PaletteModel::default()
    }
}

#[test]
fn run_command_flp_ranks_flip_horizontal_first() {
    let mut p = Palco::limpo();
    assert_eq!(ph2d_pixel_keys::search("flp")[0].0, A::FlipH);
    let m = modelo(&mut p);
    assert_eq!(
        top_match(&m, "flp"),
        Some(comando::id(A::FlipH)),
        "o Enter da paleta"
    );
}

#[test]
fn run_command_row_shows_the_users_rebound_key() {
    let mut p = Palco::limpo();
    assert_eq!(
        comando::label(&p.app, A::FlipH),
        "Flip Horizontal \u{b7} Shift+H"
    );
    let mut c = Chord::new(KeyCode::F9);
    (c.ctrl, c.alt) = (true, true);
    let feito = p.app.keys.assign(A::FlipH, c, Context::Normal);
    assert_eq!(feito, Ok(()), "{c} livre");
    let m = modelo(&mut p);
    let row = m.groups[0].subs.iter().flat_map(|s| &s.items);
    let row = row
        .into_iter()
        .find(|it| it.id == comando::id(A::FlipH))
        .expect("linha");
    assert_eq!(row.label, "Flip Horizontal \u{b7} Shift+H, Ctrl+Alt+F9");
}

fn doc(p: &Palco) -> &ph2d_app_pixel::Doc {
    p.app.doc(p.key).expect("doc")
}

#[test]
fn run_command_new_layer_is_one_undo_step_named_like_the_menu() {
    let mut p = Palco::limpo();
    let (camadas, passos) = (doc(&p).sprite.layers().len(), doc(&p).history.len());
    assert!(comando::run(&mut p.app, comando::id(A::NewLayer)).is_some());
    p.quadro();
    assert_eq!(doc(&p).sprite.layers().len(), camadas + 1);
    assert_eq!(doc(&p).history.len(), passos + 1, "UM passo");
    let (ultimo, _) = doc(&p).history.entries().nth(passos).expect("o passo");
    assert_eq!(
        ph2d_i18n::tr(ultimo.key()),
        A::NewLayer.name(),
        "o nome do menu"
    );
}

#[test]
fn run_command_unavailable_row_does_nothing() {
    let mut p = Palco::limpo();
    assert!(
        !comando::available(&p.app, A::DeleteSlices),
        "sem slices seleccionadas"
    );
    let passos = p.app.doc(p.key).expect("doc").history.len();
    assert!(comando::run(&mut p.app, comando::id(A::DeleteSlices)).is_none());
    p.quadro();
    assert_eq!(p.app.doc(p.key).expect("doc").history.len(), passos);
    let m = modelo(&mut p);
    let off = m.groups[0].subs.last().expect("a secção cinzenta");
    assert!(
        off.items
            .iter()
            .any(|it| it.id == comando::id(A::DeleteSlices))
    );
}
