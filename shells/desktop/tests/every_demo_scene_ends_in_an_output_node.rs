//! **Uma cena de demonstração que não constrói um `motion.output` desenha NADA.**
//!
//! O laço de render **não usa** a lista de sinks que o construtor da cena devolve: ele a
//! re-resolve a cada quadro a partir dos nós de saída do grafo
//! (`render_loop/motion_bridge.rs`: `motion.sinks = output_nodes(&motion.doc.graph)`, e
//! `output_nodes` colhe **todo nó cujo `type_name` é `motion.output`**). Uma cadeia terminada
//! no último nó de efeito cozinha certo, passa em todos os gates da cena — e a tela fica
//! **VAZIA**, que é indistinguível da feature quebrada.
//!
//! ⚠️ **Isto aconteceu** (2026-08-15): a cena `=48` do grupo H nasceu com as seis bandas a
//! terminar no `motion.collide`. Os seis gates dela mediam a geometria COZIDA e ficavam
//! verdes; o defeito só apareceu quando o Enio leu o código do documento.
//!
//! O gate varre a FAMÍLIA de arquivos, nunca uma lista escrita à mão: a cena que nascer
//! amanhã entra na varredura sozinha, que é o que uma enumeração não faz.

use std::fs;

/// Toda cena de conferência constrói pelo menos um nó de saída.
#[test]
fn every_demo_scene_ends_in_an_output_node() {
    let dir = fs::read_dir("src").expect("shells/desktop/src");

    let mut scanned = Vec::new();
    let mut silent = Vec::new();
    for entry in dir {
        let path = entry.expect("entrada de diretorio").path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        // As cenas: `motion_state_conferencia_demos*.rs`, menos os arquivos de gate
        // (um `*_tests.rs` não constrói cena nenhuma).
        // ⚠️ `_probes.rs` entrou na exclusão em 2026-08-25 pela MESMA razão que `_tests.rs`:
        // uma sonda de medição é irmã de um gate, não uma cena — ela não desenha, ela MEDE.
        // O nome dela partilha o prefixo porque ela pertence à cena que mede.
        if !name.starts_with("motion_state_conferencia_demos")
            || name.ends_with("_tests.rs")
            || name.ends_with("_probes.rs")
        {
            continue;
        }
        let src = fs::read_to_string(&path).expect("le a cena");
        scanned.push(name.to_string());
        if !src.contains("\"motion.output\"") {
            silent.push(name.to_string());
        }
    }

    // CONTROLE POSITIVO: sem ele um `read_dir` que deixasse de casar (um rename da
    // família, um corte de diretório) tornaria este gate uma varredura VAZIA, verde
    // para sempre.
    assert!(
        scanned.len() >= 10,
        "a varredura achou {} cenas de conferencia -- a familia mudou de nome e o gate esta' a \
         medir nada: {scanned:?}",
        scanned.len(),
    );

    assert!(
        silent.is_empty(),
        "estas cenas nao constroem um `motion.output`, entao o render colhe zero sinks e elas \
         desenham NADA na tela: {}",
        silent.join(" · "),
    );
}
