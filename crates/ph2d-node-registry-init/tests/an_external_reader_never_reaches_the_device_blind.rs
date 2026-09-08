//! ⛔⛔ **UM NÓ QUE LÊ O CANAL DE EXTERNOS NÃO CHEGA AO DISPOSITIVO ÀS CEGAS**
//! (ciclo 3, W5b — [doc 106](../../../docs/Motion%20Nodes/106_ciclo_3_transformes_e_deformadores.md)).
//!
//! ## A lei, e o número que a torna uma lei
//!
//! O `ctx.external(&key)` é a porta por onde a shell entrega ao grafo o que o **documento**
//! sabe e o stream não carrega: a polilinha de uma forma desenhada, a pose de um objecto, o
//! cursor, uma tabela de ficheiro. ⚠️ **O sequenciador de GPU não tem canal nenhum para ela** —
//! `grep -c external crates/ph2d-gpu-cook/src/*.rs` devolve **zero**, e um `ColumnBinding` só
//! sabe endereçar uma coluna de uma PORTA de entrada.
//!
//! ⇒ um nó que leia um externo e registe um kernel **não fica lento: fica ERRADO**, e erra da
//! pior maneira possível — o dispositivo cozinha o braço de omissão (a cúbica autorada, a
//! origem do mundo, a tabela vazia) e devolve uma imagem **plausível**. Nada estoura, nada
//! avisa, e o artista vê a curva errada.
//!
//! ## O que este censo pergunta, e o que ele NÃO pergunta
//!
//! Ele pergunta: *este nó lê um externo, tem kernel, e alguém escreveu uma cláusula
//! `applicable`?* ⚠️ **`applicable: Some(..)` é um PROXY e o censo diz isso de si mesmo:** ele
//! prova que a recusa foi **pensada**, nunca que ela é a certa. A prova de que é a certa é o
//! gate de paridade daquele nó — aqui mede-se a AUSÊNCIA de pensamento, que é o modo de falha
//! que passa em silêncio.
//!
//! ⭐ **O precedente é o `motion.look_at`, e ele já estava certo antes deste censo existir:**
//! os modos `Object` e `Cursor` resolvem o alvo pela tabela de externos, e o kernel dele recusa
//! o dispositivo fora do modo `Point`, com o custo nomeado no comentário. *Este ficheiro não
//! inventa a lei — ele impede que o próximo nó a redescubra por report.*
//!
//! ⚠️ **Comentários são retirados antes da busca.** Um censo textual que não separa prosa de
//! código mente nos dois sentidos: acusa um nó cujo doc menciona a porta e absolve um cujo doc
//! a explica sem a chamar.

use std::path::{Path, PathBuf};

/// A pasta `crates/`, a partir deste crate.
fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("o crate vive dentro de crates/")
        .to_path_buf()
}

/// Todos os crates `ph2d-node-*`.
fn node_crates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(crates_dir())
        .expect("crates/ existe")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.starts_with("ph2d-node-"))
        })
        .collect();
    out.sort();
    out
}

/// Os ficheiros de PRODUTO de um crate de nó — tudo em `src/` menos os irmãos de teste (a
/// exclusão que o censo do relógio já paga: uma fixtura pode chamar o que o produto não chama).
fn product_sources(crate_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![crate_dir.join("src")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && !p
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.ends_with("_tests.rs"))
            {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// O ficheiro sem as linhas de comentário — ver o cabeçalho.
fn code_only(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `(lê externo, regista kernel, declara applicable)` — só sobre o produto, só sobre código.
fn survey(crate_dir: &Path) -> (bool, bool, bool) {
    let (mut ext, mut kernel, mut applicable) = (false, false, false);
    for f in product_sources(crate_dir) {
        let Ok(raw) = std::fs::read_to_string(&f) else {
            continue;
        };
        let src = code_only(&raw);
        ext |= src.contains("ctx.external(")
            || src.contains(".external(&")
            || src.contains(".external(ph2d_nodegraph");
        kernel |= src.contains("register_gpu_kernel");
        applicable |= src.contains("applicable: Some(");
    }
    (ext, kernel, applicable)
}

#[test]
fn an_external_reader_that_reaches_the_device_declares_a_refusal() {
    let (mut leem, mut com_kernel, mut acusados) = (0usize, 0usize, Vec::new());
    for c in node_crates() {
        let (ext, kernel, applicable) = survey(&c);
        if !ext {
            continue;
        }
        leem += 1;
        if !kernel {
            continue;
        }
        com_kernel += 1;
        if !applicable {
            acusados.push(c.file_name().unwrap().to_string_lossy().to_string());
        }
    }

    // ⚠️ **DOIS controles, porque há duas maneiras de este censo ficar cego** — e a segunda é a
    // que interessa: se nenhum nó tivesse kernel E externo, a asserção seria vacuamente
    // verdadeira e o ficheiro passaria a ser decoração.
    assert!(
        leem >= 8,
        "só {leem} nós lêem o canal de externos — a varredura não achou os ficheiros \
         (medido em 2026-09-08: 11)"
    );
    assert!(
        com_kernel >= 1,
        "nenhum nó lê externo E tem kernel — a célula que este censo mede está VAZIA, \
         então ele não está a medir nada (o `motion.look_at` estava nela em 2026-09-08)"
    );

    assert!(
        acusados.is_empty(),
        "estes nós lêem o canal de externos e registam um kernel de GPU sem declarar \
         `applicable`: {acusados:?}\n\
         O sequenciador não tem binding para um externo — no dispositivo o nó cozinharia o \
         braço de omissão e devolveria uma imagem PLAUSÍVEL e errada. A saída é uma cláusula \
         `applicable` que recuse enquanto o externo estiver em jogo (o `motion.look_at` é o \
         precedente), ou um canal de externos no substrato."
    );
}
