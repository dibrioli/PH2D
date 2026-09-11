//! ⛔⛔ **AS QUATRO PORTAS DE «ABRIR A RECEITA» SÃO A MESMA LEI** (2026-09-07).
//!
//! O verbo tem quatro acessos — o cartão da biblioteca, o botão do painel vetorial, o item do menu
//! da Hierarquia e, desde hoje, o botão colado à linha de proveniência do Inspector. ⚠️ **Cada um
//! nasceu copiado do vizinho**, e o modo de falha é sempre o mesmo: um deles a chamar outra coisa,
//! perfeitamente ligado e perfeitamente errado, com os gates das pontas verdes.
//!
//! ⇒ este censo exige que o braço do Inspector chame a **porta** (`instance_open::open_prefab`), e
//! não uma segunda resolução escrita à mão. *Quatro respostas a «de que prefab isto é cópia?»
//! divergem no dia em que uma delas ganhar um filtro.*
//!
//! ⛔ Ele é textual porque o braço vive dentro do laço de quadro da `render_loop`, cuja função tem
//! ~35 argumentos e um `AppGfx` com uma surface de janela real.

use std::path::Path;

/// O corpo do ficheiro sem comentários — senão o censo lê o que o código DIZ sobre si.
fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O botão do cartão abre pela PORTA, e a selecção segue.**
///
/// **Mutação que deve sangrar:** o braço a deixar de chamar o `open_prefab`, ou a não escrever a
/// selecção (o botão passa a dizer que abriu e nada aparece — a 1.ª espécie de controlo morto).
#[test]
fn the_inspector_card_opens_the_prefab_through_the_shared_door() {
    let body = code_of("render_loop/mod.rs");
    let at = body
        .find("EditorAction::InspectorOpenPrefab")
        .expect("o braço do botão do cartão desapareceu do dreno");
    // ⚠️ **A janela acaba no braço SEGUINTE, e não num número de caracteres** — a 1.ª redacção
    // cortava em `520` e o `replace_selection` ficava 30 bytes lá fora, com o gate a reprovar
    // sobre código certo. *Uma janela contada em bytes mede a indentação.*
    let rest = &body[at..];
    let end = rest[1..]
        .find("EditorAction::")
        .map_or(rest.len(), |i| i + 1);
    let arm = &rest[..end];
    assert!(
        arm.contains("instance_open::open_prefab("),
        "o braço do cartão resolve a receita por outro caminho — quatro respostas à mesma pergunta \
         divergem:\n{arm}"
    );
    assert!(
        arm.contains("replace_selection"),
        "o braço não move a selecção — abrir é SELECCIONAR, e sem isso o botão fala e não mostra \
         nada:\n{arm}"
    );
}
