//! ⭐ A posse do teclado e do rato com a cena a correr (CodeEditor item 7) está perguntada NO SÍTIO: a lei tem os testes
//! dela (`ph2d_app_components::posse_do_jogo`); uma porta certa no sítio errado é o Espaço a pausar a cena outra vez.
//! Ponta a ponta: `docs/CodeEditor/ferramentas/conduz_posse/corre.sh` (a app numa Xvfb, teclas reais).

fn corpo(ficheiro: &str, assinatura: &str) -> String {
    let src = std::fs::read_to_string(ficheiro).expect(ficheiro);
    src.split_once(assinatura).expect(assinatura).1.to_owned()
}

fn em(c: &str, agulha: &str) -> usize {
    c.find(agulha).unwrap_or_else(|| panic!("`{agulha}` sumiu"))
}

/// Depois da escuta e da paleta (modais, ganham sempre), antes de todo atalho (o Espaço mora na cauda).
#[test]
fn a_tecla_pergunta_a_posse_antes_de_todo_atalho() {
    let c = corpo("src/input_dispatch/keyboard.rs", "pub(crate) fn key_input(");
    let porta = em(&c, "self.tecla_do_jogo(");
    assert!(em(&c, "self.command_palette_keys(") < porta);
    for depois in [
        "self.toggle_legacy_chrome()",
        "self.modal_owns_the_keyboard(",
        "self.board_owns_the_keyboard(",
        "ph2d_app_flip::peek::key_transition",
        "self.key_tail(",
    ] {
        assert!(
            porta < em(&c, depois),
            "`{depois}` antes da posse: a tecla faz duas coisas"
        );
    }
    assert!(
        !c.contains("self.input.apply_event"),
        "a tecla entra no jogo por UMA porta"
    );
}

/// Depois do blur (um campo com o foco não rouba o 1.º clique na cena), antes do primeiro ramo que devolve.
#[test]
fn o_clique_pergunta_a_posse_antes_do_primeiro_ramo_do_editor() {
    let c = corpo("src/input_dispatch.rs", "pub(crate) fn on_mouse_input(");
    let porta = em(&c, "self.rato_do_jogo(");
    assert!(em(&c, "self.ramo_aperto_solta_teclado(") < porta);
    assert!(
        porta < em(&c, "return;"),
        "um ramo do editor devolve antes de o jogo ter o clique"
    );
    assert!(em(&c, "self.clique_so_do_jogo(") < em(&c, "return;"));
}

/// Uma vez por quadro, antes do jogador (`fase_pointer_subjects`) e dos scripts (`fase_input_and_drops`).
#[test]
fn o_foco_anda_antes_dos_leitores_do_teclado() {
    let c = corpo("src/render_loop/mod.rs", "fn run_render_frame(");
    let foco = em(&c, "self.quadro_do_foco_do_jogo()");
    assert!(
        foco < em(&c, "self.fase_pointer_subjects()")
            && foco < em(&c, "self.fase_input_and_drops(")
    );
}

/// O Espaço e o F5 são UMA porta; o Shift+F5 é o botão Rebobinar.
#[test]
fn o_espaco_e_o_f5_sao_uma_porta() {
    let teclas =
        std::fs::read_to_string("src/input_dispatch/handlers_teclas_editor.rs").expect("teclas");
    let espaco = teclas
        .split_once("KeyCode::Space =>")
        .expect("o braço do Espaço")
        .1;
    assert!(espaco[..em(espaco, "KeyCode::")].contains("posse_do_jogo::alterna_o_relogio("));
    assert!(
        !teclas.contains("toggle_play()"),
        "uma segunda cópia do alternar"
    );
    let porta = std::fs::read_to_string("src/input_dispatch/posse_do_jogo.rs").expect("porta");
    assert!(
        porta.contains("(Some(Transporte::Alterna), Some(gfx)) =>")
            && porta.contains("TransportCmd::Reset")
    );
}
