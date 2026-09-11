//! **O `outside_frame` não tem chamador de PRODUTO** — arch-gate.
//!
//! ⚠️ **Porque este gate existe:** até à W2/L2 a propriedade era garantida pelo
//! compilador, com um `#[cfg(test)]` na função. A extracção da família `physics`
//! partiu isso: os dois chamadores dela são gates que ficaram na **shell** (eles
//! precisam do `render_loop::inspector_joint_wheel`), e `cfg(test)` não atravessa
//! a fronteira de uma crate — ele fala dos testes de QUEM DECLARA, não dos de quem
//! depende. Tirar o `cfg` era a única forma de os dois gates continuarem a correr.
//!
//! O que o `cfg` defendia era isto: *um helper sem chamador de produto não é código
//! morto silencioso — é uma **segunda resposta** à mesma pergunta, esperando que
//! alguém a chame* (a lição do `warp_axis` e do `serial_side`). Essa propriedade
//! passa a ser afirmada aqui, por varredura.
//!
//! ⛔ **Este gate falha ABERTO se o nome mudar**, e por isso tem controlo positivo:
//! ele exige encontrar a DEFINIÇÃO antes de afirmar seja o que for sobre os usos.

use std::path::{Path, PathBuf};

fn raiz() -> PathBuf {
    // …/crates/ph2d-app-physics -> …/
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("raiz do repo")
        .to_path_buf()
}

fn varre(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(it) = std::fs::read_dir(dir) else {
        return;
    };
    for e in it.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            varre(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn outside_frame_has_no_production_caller() {
    let raiz = raiz();
    let mut ficheiros = Vec::new();
    varre(&raiz.join("crates"), &mut ficheiros);
    varre(&raiz.join("shells"), &mut ficheiros);

    let mut definicoes = 0usize;
    let mut culpados = Vec::new();

    for f in &ficheiros {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        if src.contains("pub fn outside_frame(") {
            definicoes += 1;
        }
        // Um ficheiro cujo nome diz `_tests` ou que vive em `tests/` é um gate —
        // é exactamente quem PODE chamar.
        let nome = f.to_string_lossy();
        let e_teste = nome.contains("_tests.rs") || nome.contains("/tests/");
        if e_teste {
            continue;
        }
        for (i, linha) in src.lines().enumerate() {
            if linha.contains("outside_frame(") && !linha.contains("pub fn outside_frame(") {
                culpados.push(format!("{}:{}", f.display(), i + 1));
            }
        }
    }

    // ⭐ Controlo positivo: sem a definição, as asserções abaixo seriam vácuo.
    assert_eq!(
        definicoes, 1,
        "esperava EXACTAMENTE uma definição de `outside_frame` (achei {definicoes}) — \
         se ela foi renomeada ou duplicada, este gate deixou de afirmar o que diz"
    );
    assert!(
        culpados.is_empty(),
        "`outside_frame` ganhou chamador de PRODUTO em {culpados:?}. Ela existe para \
         gates: um helper chamado pelo produto e por um gate é uma SEGUNDA RESPOSTA \
         à mesma pergunta, e as duas divergem no dia em que uma delas mudar."
    );
}
