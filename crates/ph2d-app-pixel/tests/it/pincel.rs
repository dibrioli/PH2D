//! **O Ctrl+B no app** (P7b): a selecção vira o pincel de padrão e cai; escolher uma cor pinta o
//! pincel inteiro nela (dono 02/11, §6.1 do handoff P7a); sem selecção, arrastar define o pincel e o
//! botão direito também o corta; a borracha apaga com a forma.

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_pixel_io::brush_file::BrushParams;
use winit::keyboard::KeyCode;

use crate::edit::{at, canvas};
use crate::selecao::Palco;

const R: [u8; 4] = [255, 0, 0, 255];
const B: [u8; 4] = [0, 0, 255, 255];
const G: [u8; 4] = [0, 160, 0, 255];
const T: [u8; 4] = [0; 4];

fn ponto(p: &mut Palco, c: (i32, i32)) {
    p.arrasta(c, c, Mods::default());
}

/// Um palco com `R B` em (60,50)–(61,50) e o lápis na mão.
fn palco() -> Palco {
    let mut p = Palco::limpo();
    for x in 58..64 {
        assert_eq!(p.rgba(x, 50), T, "a área de trabalho começa vazia");
    }
    p.ferramenta(Tool::Pencil);
    p.pede(PixelRequest::Fg(R));
    ponto(&mut p, (60, 50));
    p.pede(PixelRequest::Fg(B));
    ponto(&mut p, (61, 50));
    p
}

fn seleciona(p: &mut Palco, a: (i32, i32), b: (i32, i32)) {
    p.ferramenta(Tool::RectMarquee);
    p.arrasta(a, b, Mods::default());
    p.ferramenta(Tool::Pencil);
}

#[test]
fn ctrl_b_turns_the_selection_into_the_brush_and_drops_the_selection() {
    let mut p = palco();
    seleciona(&mut p, (60, 50), (61, 50));
    assert!(p.mascara().is_some());
    p.tecla(KeyCode::KeyB, true, false);
    assert!(p.mascara().is_none(), "o Ctrl+B larga a selecção");
    // Source (a omissão): o padrão fica preso a x = 60, então 70 = R e 71 = B.
    ponto(&mut p, (71, 55));
    assert_eq!((p.rgba(70, 55), p.rgba(71, 55)), (R, B));
    // O `B` sozinho continua o lápis (e não larga o pincel).
    p.tecla(KeyCode::KeyB, false, false);
    ponto(&mut p, (71, 57));
    assert_eq!((p.rgba(70, 57), p.rgba(71, 57)), (R, B));
}

#[test]
fn choosing_a_colour_paints_the_whole_brush_in_it_even_the_same_colour() {
    let mut p = palco();
    seleciona(&mut p, (60, 50), (61, 50));
    p.tecla(KeyCode::KeyB, true, false);
    p.pede(PixelRequest::Fg(G));
    ponto(&mut p, (71, 55));
    assert_eq!(
        (p.rgba(70, 55), p.rgba(71, 55)),
        (G, G),
        "INTEIRO, sem o (0,0) do Aseprite"
    );
    // O Trial muda até com a MESMA cor (rb2b_frente_paint_cores, «verde = a frente do Ctrl+B»).
    seleciona(&mut p, (60, 50), (61, 50));
    p.tecla(KeyCode::KeyB, true, false);
    p.pede(PixelRequest::Fg(p.app.fg));
    ponto(&mut p, (71, 57));
    assert_eq!(p.rgba(70, 57), p.app.fg);
    // A troca X também é escolher a frente.
    seleciona(&mut p, (60, 50), (61, 50));
    p.tecla(KeyCode::KeyB, true, false);
    p.tecla(KeyCode::KeyX, false, false);
    ponto(&mut p, (71, 59));
    assert_eq!(p.rgba(70, 59), p.rgba(71, 59));
    assert_eq!(p.rgba(70, 59), p.app.fg);
}

#[test]
fn ctrl_b_without_a_selection_drags_the_brush_bounds_and_right_drag_cuts() {
    let mut p = palco();
    assert!(p.mascara().is_none());
    let n = p.passos();
    p.tecla(KeyCode::KeyB, true, false);
    assert_eq!(p.passos(), n, "entrar no recorte não é passo");
    p.arrasta((60, 50), (61, 50), Mods::default());
    assert_eq!(p.passos(), n, "o recorte esquerdo não muda o sprite");
    assert_eq!((p.rgba(60, 50), p.rgba(61, 50)), (R, B));
    ponto(&mut p, (71, 55));
    assert_eq!((p.rgba(70, 55), p.rgba(71, 55)), (R, B));
    // Direito: corta (UM passo) e o pincel é o que lá estava.
    p.tecla(KeyCode::KeyB, true, false);
    let c = canvas(&p.sim, &p.cam, p.bits);
    let a = at(&p.app, &p.sim, &p.cam, p.bits, 70, 55);
    let b = at(&p.app, &p.sim, &p.cam, p.bits, 71, 55);
    let r = input::press(&mut p.app, &c, a, Button::Secondary, Mods::default());
    assert_eq!(r, Press::Done);
    let c = canvas(&p.sim, &p.cam, p.bits);
    assert!(input::moved(&mut p.app, &c, b, Mods::default()));
    p.release();
    assert_eq!(p.passos(), n + 2, "o corte é UM passo");
    assert_eq!((p.rgba(70, 55), p.rgba(71, 55)), (T, T), "cortado");
    ponto(&mut p, (71, 57));
    assert_eq!((p.rgba(70, 57), p.rgba(71, 57)), (R, B));
    // Esc larga o recorte pendente: o arrasto seguinte volta a ser do lápis.
    p.tecla(KeyCode::KeyB, true, false);
    p.tecla(KeyCode::Escape, false, false);
    let antes = p.passos();
    ponto(&mut p, (71, 59));
    assert_eq!(p.passos(), antes + 1, "depois do Esc o clique pinta");
}

#[test]
fn the_eraser_erases_with_the_brush_shape() {
    let mut p = palco();
    seleciona(&mut p, (60, 50), (61, 50));
    p.tecla(KeyCode::KeyB, true, false);
    ponto(&mut p, (71, 55));
    p.ferramenta(Tool::Eraser);
    ponto(&mut p, (71, 55));
    assert_eq!((p.rgba(70, 55), p.rgba(71, 55)), (T, T));
}

fn com_alt(p: &mut Palco, k: KeyCode) {
    let m = Mods {
        alt: true,
        ..Mods::default()
    };
    let a = ph2d_app_pixel::input::key_with(&mut p.app, p.sim.world(), Some(p.bits), k, m);
    assert!(a.is_some(), "{k:?} é da família");
    p.quadro();
}

#[test]
fn each_ctrl_b_is_a_new_slot_and_alt_n_recalls_it_with_the_images_colours() {
    let mut p = palco();
    p.pede(PixelRequest::Fg(G));
    ponto(&mut p, (62, 50));
    seleciona(&mut p, (60, 50), (61, 50));
    p.tecla(KeyCode::KeyB, true, false);
    seleciona(&mut p, (61, 50), (62, 50));
    p.tecla(KeyCode::KeyB, true, false);
    let l = p.app.espacos().to_vec();
    assert_eq!(l.len(), 2, "cada Ctrl+B, um espaço");
    assert!(
        l.iter()
            .all(|s| !s.locked && s.params == BrushParams::NEW_BRUSH)
    );
    // Escolher uma cor pinta o pincel inteiro nela; Alt+1 devolve o 1.º com as cores da imagem.
    p.pede(PixelRequest::Fg(R));
    p.opcoes(|o| o.size = 7);
    com_alt(&mut p, KeyCode::Digit1);
    ponto(&mut p, (71, 55));
    assert_eq!(
        (p.rgba(70, 55), p.rgba(71, 55)),
        (R, B),
        "o 1.º, com as cores dele"
    );
    assert_eq!(p.app.options.size, 7, "o tamanho não é dele: fica o actual");
    com_alt(&mut p, KeyCode::Digit2);
    ponto(&mut p, (71, 57));
    assert_eq!(
        (p.rgba(70, 57), p.rgba(71, 57)),
        (G, B),
        "o 2.º (B G, Source preso a 61)"
    );
    com_alt(&mut p, KeyCode::Digit9);
}

#[test]
fn the_slots_live_in_the_users_folder_and_survive_a_new_session() {
    let dir = std::env::temp_dir().join(format!("ph2d_pinceis_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut p = palco();
    p.app.pinceis = Some(dir.clone());
    seleciona(&mut p, (60, 50), (61, 50));
    p.tecla(KeyCode::KeyB, true, false);
    let texto = std::fs::read_to_string(dir.join(ph2d_app_pixel::pinceis::FILE)).expect("gravou");
    assert!(texto.contains("row ff0000ff 0000ffff\n"), "{texto}");
    let mut outra = ph2d_app_pixel::PixelApp::default();
    outra.pinceis = Some(dir.clone());
    assert_eq!(
        outra.espacos(),
        p.app.espacos(),
        "a sessão nova lê a mesma lista"
    );
    std::fs::write(dir.join(ph2d_app_pixel::pinceis::FILE), "lixo").expect("w");
    let mut estragada = ph2d_app_pixel::PixelApp::default();
    estragada.pinceis = Some(dir.clone());
    assert!(estragada.espacos().is_empty());
    assert!(
        estragada.take_avisos().iter().any(|(erro, _)| *erro),
        "e avisa"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
