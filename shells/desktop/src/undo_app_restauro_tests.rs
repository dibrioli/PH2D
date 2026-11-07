//! O restauro assenta sem virar passo (smoke do dono, 06/11: Ctrl+Z várias vezes e a moeda ficou sob a plataforma).
//! O valor da fotografia só existe com janela; sem ela `capture_project` é `None`, e `None` só pode ter vindo daqui.

use super::tests::{empty_state, headless_app};

/// *Mutação: tirar o ramo do `assenta_o_restauro` ⇒ o baseline fica, e o Ctrl+Z seguinte desfaz a própria pose.*
#[test]
fn o_quadro_depois_do_restauro_refaz_a_base_sem_criar_passo() {
    let mut app = headless_app();
    app.undo_baseline = Some(empty_state());
    app.assenta_o_restauro = 2;
    let passos = app.undo.depth();
    app.post_frame_undo();
    assert!(app.undo_baseline.is_none());
    assert_eq!(app.assenta_o_restauro, 1);
    assert_eq!(app.undo.depth(), passos);
}

/// O restauro arma os quadros, e o dreno dos canais corre logo depois do passo da física (que aplica a curva no jogo).
/// *Mutação: tirar qualquer uma das duas linhas.*
#[test]
fn o_restauro_arma_e_o_jogo_drena_os_canais_depois_da_fisica() {
    let undo = include_str!("undo_app.rs");
    let arma = undo.find("self.arma_o_restauro();").unwrap();
    let restored = undo.find("if let Some((state, mark)) = restored").unwrap();
    assert!(arma > restored && arma - restored < 120);
    let quadro = include_str!("render_loop/fase_frame_open.rs");
    assert!(
        quadro.contains("self.fase_physics_step(player_input);\n        self.drena_os_canais();")
    );
}
