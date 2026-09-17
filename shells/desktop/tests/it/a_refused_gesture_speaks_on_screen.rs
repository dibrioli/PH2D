//! ⛔⛔ **UMA RECUSA DE GESTO FALA NA TELA, NUNCA NO TERMINAL** (auditoria de 2026-09-06).
//!
//! # O achado
//!
//! O `Ctrl+G` / `Ctrl+Shift+G` do editor vetorial recusava por `eprintln!` — *«selecione >= 2
//! objetos distintos»* saía no terminal, que o artista não tem aberto. O **gémeo** do mesmo verbo,
//! o item *Group* do menu da Hierarquia, já respondia com toast: ⇒ **duas superfícies do mesmo
//! gesto, uma muda**, e a muda é a do atalho, que é a que se usa depois de aprender.
//!
//! ⚠️ **O sintoma é indistinguível de uma feature partida** — e foi exactamente assim que um smoke
//! desta linha mandou o dono agrupar com um objecto só e ficar a olhar para o nada.
//!
//! # A régua
//!
//! O corpo de `vec_group` não tem `eprintln!`. ⛔ **Não é um censo do ficheiro inteiro:** o
//! `input_dispatch.rs` tem `eprintln!` legítimos noutros sítios (diagnóstico de arranque), e um
//! censo largo seria uma catraca que outra linha teria de negociar.
//!
//! **Mutação que deve sangrar:** trocar qualquer um dos dois toasts de volta por `eprintln!`.

use std::path::Path;

/// O corpo de uma `fn` do ficheiro, do cabeçalho até à chaveta que o fecha.
///
/// ⚠️ **Descasca comentários ANTES de varrer** — e não é um detalhe de higiene: a 1.ª redacção
/// deste gate reprovou sobre a própria cura, porque o comentário que explica a cura **nomeia** o
/// `eprintln!` que ela tirou. *Um censo textual que lê comentários mede o que o código DIZ sobre
/// si, não o que ele FAZ.*
fn body_of(rel: &str, signature: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    // O `input_dispatch.rs` lê-se reconstituído (os ramos correm no sítio da chamada).
    let raw = if rel == "input_dispatch.rs" {
        crate::input_text::dispatch()
    } else {
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
    };
    let src: String = raw
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let start = src
        .find(signature)
        .unwrap_or_else(|| panic!("{rel} ja' nao tem `{signature}` — reancore este censo"));
    let rest = &src[start..];
    let mut depth = 0usize;
    let mut seen = false;
    for (i, c) in rest.char_indices() {
        match c {
            '{' => {
                depth += 1;
                seen = true;
            }
            '}' => {
                depth -= 1;
                if seen && depth == 0 {
                    return rest[..=i].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("{rel}: a chaveta de `{signature}` nao fecha");
}

/// ⭐⭐⭐ **As duas recusas do agrupar por teclado são VISÍVEIS.**
#[test]
fn the_keyboard_group_refuses_out_loud() {
    let body = body_of("input_dispatch.rs", "fn vec_group(&mut self, group: bool)");
    assert!(
        !body.contains("eprintln!"),
        "o Ctrl+G voltou a recusar so' no terminal — o artista ve' um gesto que nao faz nada e \
         conclui que a feature esta' partida"
    );
    assert_eq!(
        body.split_whitespace()
            .collect::<String>()
            .matches("toasts.push")
            .count(),
        2,
        "as duas recusas do agrupar/desagrupar tem de falar — uma muda le'-se como defeito"
    );
}
