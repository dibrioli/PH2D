//! **O pedido de quadro das abas** (`Documents::wants_frame`: o voo do editor de animações, o Kanban, o SFX Lab) não
//! tem leitor na shell, e hoje não precisa: o laço redesenha sempre (`ControlFlow::Poll`). Decisão de 08/11
//! (`docs/GameEngineFuncional/27_ferramentas_na_folha_visivel.md` §4): o contrato fica, e este portão prende o par —
//! quem tirar o laço contínuo tem de ler o pedido, senão uma animação de aba congela até o rato mexer.

use std::fs;
use std::path::Path;

/// *Mutação: trocar `ControlFlow::Poll` por `Wait` no `main.rs` ⇒ vermelho (ninguém lê o pedido).*
#[test]
fn sem_laco_continuo_alguem_le_o_pedido_de_quadro() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let main = fs::read_to_string(src.join("main.rs")).unwrap();
    if main.contains("set_control_flow(ControlFlow::Poll)") {
        return;
    }
    let mut leitores = Vec::new();
    let mut pilha = vec![src];
    while let Some(d) = pilha.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && fs::read_to_string(&p).unwrap().contains(".wants_frame()")
            {
                leitores.push(p);
            }
        }
    }
    assert!(
        !leitores.is_empty(),
        "o laço deixou de ser contínuo e nenhum sítio da shell lê `Documents::wants_frame`"
    );
}
