//! ⭐ Dono 07/11: pausado em tela cheia, Play e Stop não respondiam. Os ramos do editor em `pointer_button` tomavam o
//! clique antes dos botões. A porta do palco tem de vir depois da do jogo e antes do primeiro ramo do editor.
//! Prova ao vivo: cliques injetados em `on_mouse_input`, handoff de 07/11.

#[test]
fn a_porta_do_palco_vem_antes_dos_ramos_do_editor() {
    let fonte = include_str!("../../src/input_dispatch.rs");
    let clique = &fonte[fonte
        .find("pub(crate) fn pointer_button")
        .expect("o clique")..];
    let jogo = clique.find("self.rato_do_jogo(").expect("a porta do jogo");
    let palco = clique.find("if self.no_palco()").expect("a porta do palco");
    let editor = clique
        .find("self.ramo_alca_da_ancora(")
        .expect("o 1.º ramo do editor");
    assert!(jogo < palco && palco < editor);
    let mover = &fonte[fonte
        .find("pub(crate) fn pointer_moved")
        .expect("o movimento")..];
    assert!(mover.find("if self.no_palco()") < mover.find("self.ramo_pixel_mover()"));
}
