//! ⛔⛔⛔ **CENSO — a linha de comando que uma sonda escreve no doc dela tem de nomear a
//! CRATE onde ela vive.**
//!
//! Achado por um report do dono (2026-09-20): **36** sondas desta crate, em **23**
//! ficheiros, mandavam correr o pacote da SHELL. A família saiu de lá na W2 de 11/09 e os
//! endereços ficaram — e o modo de falha é o pior que há: um filtro que não casa nada
//! corre **zero** testes e o cargo imprime **`ok`**.
//!
//! ⚠️ *Um endereço obsoleto não é «difícil de achar»: ele mede outro programa e diz que
//! passou.* Eu próprio segui um deles nesta wave, esperei `2 m 48 s` por uma build e li
//! `running 0 tests` como se fosse um resultado.
//!
//! ⭐ O censo é **derivado**: ele varre os ficheiros e não uma lista. Uma sonda nova com o
//! endereço errado reprova, e uma crate que mude de nome reprova em todas de uma vez.

use std::path::Path;

/// O que uma linha de sonda desta crate tem de dizer.
const PORTA: &str = "-p ph2d-app-sculpt3d";

/// A agulha, montada em runtime — ⛔ **escrita como literal ela apanharia ESTE ficheiro**,
/// e um censo que se lê a si mesmo encontra sempre o que procura.
fn agulha() -> String {
    format!("{} test -p ", "cargo")
}

#[test]
fn toda_sonda_desta_crate_aponta_a_crate_desta_crate() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let agulha = agulha();
    let mut acusados: Vec<String> = Vec::new();
    let mut vistos = 0usize;
    visita(&raiz, &mut |p| {
        let Ok(texto) = std::fs::read_to_string(p) else {
            return;
        };
        for (n, linha) in texto.lines().enumerate() {
            if !linha.contains(&agulha) {
                continue;
            }
            vistos += 1;
            if !linha.contains(PORTA) {
                let rel = p.strip_prefix(&raiz).unwrap_or(p).display();
                acusados.push(format!("{rel}:{}: {}", n + 1, linha.trim()));
            }
        }
    });
    // ⛔ **O piso de população:** sem ele uma varredura partida devolve zero acusados e
    // lê-se como aprovada — a lei que esta casa paga em todo censo por directório.
    assert!(
        vistos >= 30,
        "o censo so' viu {vistos} linhas de invocacao — ele deixou de varrer o que devia"
    );
    assert!(
        acusados.is_empty(),
        "sondas desta crate a apontar para outra ({} de {vistos}):\n  {}\n\n\
         O filtro nao casa nada, corre ZERO testes e o cargo imprime `ok`.",
        acusados.len(),
        acusados.join("\n  ")
    );
}

/// ⭐ **O CONTROLO POSITIVO: a régua acusa mesmo o que promete acusar.** Sem ele, uma
/// varredura que lesse zero linhas ficaria verde pelo mesmo motivo que o defeito que este
/// censo existe para apanhar.
#[test]
fn o_censo_acha_a_porta_errada_quando_ela_existe() {
    let linha = format!(
        "///   {}ph2d-host-desktop --release --bins foo -- --ignored",
        agulha()
    );
    assert!(linha.contains(&agulha()), "a agulha tem de casar");
    assert!(
        !linha.contains(PORTA),
        "uma linha com a crate ERRADA tem de ser acusada"
    );
}

fn visita(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            visita(&p, f);
        } else if p.extension().is_some_and(|x| x == "rs") {
            f(&p);
        }
    }
}
