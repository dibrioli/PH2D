//! **O pedido de quadro das abas** (`Documents::wants_frame`: o voo do editor de animações, o Kanban, o SFX Lab) não
//! tem leitor na shell, e hoje não precisa: o laço redesenha sempre (`ControlFlow::Poll`). Decisão de 08/11
//! (`docs/GameEngineFuncional/27_ferramentas_na_folha_visivel.md` §4): o contrato fica, e este portão prende o par —
//! quem tirar o laço contínuo tem de ler o pedido, senão uma animação de aba congela até o rato mexer.
//! Lê o fonte da shell, como a catraca `the_shell_only_shrinks` (veio da shell, 10/11: a shell só encolhe).

use std::fs;
use std::path::{Path, PathBuf};

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src")
}

/// *Mutação: trocar `ControlFlow::Poll` por `Wait` no `main.rs` ⇒ vermelho (ninguém lê o pedido).*
#[test]
fn sem_laco_continuo_alguem_le_o_pedido_de_quadro() {
    let src = shell_src();
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

/// B23-cont: o voo de uma máquina de Morph anda por quadro (`MorphMachine::is_flying`); sem leitor no
/// `pede_o_quadro`, o laço a pedido deixa-o a 4 q/s (o despertador) depois do prazo da tecla.
#[test]
fn o_voo_do_morph_pede_quadro() {
    assert!(
        corpo_do_leitor().contains(".is_flying()"),
        "o `pede_o_quadro` não lê o voo do Morph: a transição congela entre os despertadores"
    );
}

/// B23-cont: a prévia do editor de áudio a tocar (o cursor da onda) e os medidores andam por quadro.
#[test]
fn o_som_a_sair_pede_quadro() {
    assert!(
        corpo_do_leitor().contains(".audivel()"),
        "o `pede_o_quadro` não lê o som: o cursor da onda e os medidores andam a 4 q/s"
    );
}

/// B23-cont: a folha ao vivo conta `QUADROS_PARADOS` antes de ler a tinta e o check vive pelo relógio — a 4 q/s
/// a leitura chegava ~4 s depois do traço.
#[test]
fn a_folha_devida_pede_quadro() {
    assert!(
        corpo_do_leitor().contains("SheetCache::quer_quadro"),
        "o `pede_o_quadro` não lê a folha devida: a prévia da folha pintada demora segundos"
    );
}

fn corpo_do_leitor() -> String {
    let src = fs::read_to_string(shell_src().join("janela_do_jogo.rs")).unwrap();
    let corpo = src.split("fn pede_o_quadro").nth(1).expect("o leitor");
    corpo.split("\n    }\n").next().unwrap().to_owned()
}
