//! ⭐ **A árvore das tags chega à ponte ANTES do tique** (plano 30, W6, *a tag mais perto*).
//!
//! ⚠️ Gate de TEXTO pela razão do `os_pedidos_de_vida_chegam_a_ponte`: a fase pede a `App` inteira.
//! Entregue DEPOIS do tique (a 1.ª redacção, na fase dos sinais), o tique ao vivo do 1.º quadro depois
//! de editar as tags não achava ninguém e o replay desse tique, com a árvore já entregue, achava — um
//! scrub devolvia OUTRA corrida (a auditoria do fecho da W6).

const FASE: &str = include_str!("../../src/render_loop/fase_physics_step.rs");

#[test]
fn a_arvore_das_tags_vai_a_ponte_antes_do_tique() {
    let arvore = FASE
        .find("physics.set_tag_tree(tags)")
        .expect("a fase do passo entrega a árvore das tags à ponte");
    let tique = FASE
        .find("bridge::dispatch::dispatch(")
        .expect("a fase do passo anda o relógio da física");
    assert!(arvore < tique, "a árvore chega DEPOIS do tique");
}
