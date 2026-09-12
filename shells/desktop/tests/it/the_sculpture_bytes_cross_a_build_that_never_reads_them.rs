//! ⛔⛔ **O PASSA-ADIANTE TEM DE FICAR SEM `cfg`** — e esta é a metade que nenhum teste de
//! comportamento alcança.
//!
//! Os bytes de uma escultura gravada atravessam um binário construído **sem** a feature
//! `sculpt3d`, do load ao save, sem ninguém os ler. Sem isso, abrir um projeto com escultura
//! nesse binário e gravá-lo descarta a obra do artista **em silêncio**.
//!
//! # Por que ele nasceu, e o que ele substitui
//!
//! ⚠️ **A propriedade mudou de casa em 2026-09-11 (W2/L3-B).** Ela morava em
//! `Sculpt3dRequests::doc`, dentro da crate da família, e o gate dela era um teste de unidade
//! que dizia de si mesmo: *«este teste corre numa crate que não conhece o módulo 3D — é essa a
//! prova»*. Ao voltar a ser [`App::sculpt_doc`], esse enquadramento deixou de existir: agora o
//! campo vive na shell, e o que o protege é a **ausência de um atributo**.
//!
//! ⚠️⚠️ **E uma ausência não se mede correndo o produto:** o `cargo test` corre com a feature
//! LIGADA, então um `#[cfg(feature = "sculpt3d")]` acrescentado a este campo deixaria toda a
//! suíte verde — e o defeito só apareceria no binário de um utilizador que nunca esculpe, como
//! *«o meu modelo desapareceu ao gravar»*. ⇒ o gate lê o código do produto.
//!
//! ⭐ **O irmão de COMPORTAMENTO existe e é mais forte que o teste apagado**:
//! `project_sculpt_tests::a_session_that_cannot_build_the_sculpture_hands_its_bytes_back`
//! atravessa o load e o save de verdade. As duas metades são precisas — aquela afirma que os
//! bytes voltam, esta que o sítio onde eles esperam não depende da feature.

use crate::sculpt_source;

/// **Mutação que deve sangrar:** pôr `#[cfg(feature = "sculpt3d")]` em cima do `sculpt_doc`.
#[test]
fn the_pass_through_field_carries_no_feature_gate() {
    // ⚠️ A fonte CRUA, e não a `sculpt_source::source` — aquela descarta comentários, e um
    // `#[cfg(...)]` não é um comentário mas a linha `/// doc` acima dele é: sem a prosa, a
    // janela de três linhas mediria outro sítio do ficheiro.
    let src = std::fs::read_to_string(format!("{}/src/app_state.rs", env!("CARGO_MANIFEST_DIR")))
        .expect("o `app_state.rs` da shell existe");

    let linhas: Vec<&str> = src.lines().collect();
    let at = linhas
        .iter()
        .position(|l| {
            l.trim_start()
                .starts_with("pub(crate) sculpt_doc: Vec<u8>,")
        })
        .expect(
            "controlo positivo: o campo `App::sculpt_doc` sumiu — ou foi renomeado, e este gate \
             passaria a medir o vazio",
        );

    // O atributo mora imediatamente acima da declaração, depois da prosa dela.
    let anterior = linhas[..at]
        .iter()
        .rev()
        .find(|l| {
            let t = l.trim_start();
            !t.is_empty() && !t.starts_with("///") && !t.starts_with("//")
        })
        .copied()
        .unwrap_or("");

    assert!(
        !anterior.trim_start().starts_with("#[cfg"),
        "o `App::sculpt_doc` ganhou um `cfg` (`{}`) — num binário sem a feature `sculpt3d` o \
         campo deixa de existir, e o próximo save grava vazio por cima de uma escultura gravada, \
         sem erro e sem aviso",
        anterior.trim()
    );
}

/// ⛔ **E o campo do ARQUIVO também não pode ganhar um.**
///
/// ⚠️ Um `ProjectFile.sculpt` condicional daria **duas formas de ficheiro sob o mesmo número de
/// schema** — o postcard é posicional, então um binário sem a feature leria todos os campos
/// seguintes deslocados, em silêncio. É a razão pela qual o par `sculpt_doc`/`sculpt` existe.
#[test]
fn the_file_field_carries_no_feature_gate_either() {
    let src = sculpt_source::source("project.rs");
    let at = src
        .find("sculpt:")
        .expect("controlo positivo: o `ProjectFile.sculpt` sumiu");
    let janela = &src[at.saturating_sub(200)..at];
    assert!(
        !janela.contains("#[cfg(feature = \"sculpt3d\")]"),
        "o `ProjectFile.sculpt` ganhou um `cfg` — duas formas de arquivo sob um número de schema"
    );
}
