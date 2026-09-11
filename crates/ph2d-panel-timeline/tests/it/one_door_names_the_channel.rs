//! **O canal tem UM nome, montado num lugar só** (FASE C.3 do plano 12).
//!
//! O rótulo de uma track (`"Ball · Position Y"`) sai da porta única
//! `tracks::track_label`, e o fallback dela — `#nnnn`, o id curto de quem ainda não tem
//! `Name` — é o que este gate vigia: ele tem de ser montado **uma** vez na crate.
//!
//! ⚠️ **O gate IRMÃO foi removido com o card de expressão** (2026-07-30). Ele afirmava
//! que a row e o card passavam pela MESMA porta, e existia porque o defeito já tinha
//! acontecido: o doc do `ExprModal::title` prometia `"Ball · Position Y"` enquanto o
//! código montava o rótulo por conta própria. Com um consumidor só, *"os dois passam pela
//! porta"* virou afirmação que não pode falhar — o que **pode** falhar é o próximo
//! consumidor montar a string dele, e é isso que a asserção abaixo pega.
//!
//! ⚠️ **Por que um arch-gate e não um teste de comportamento:** o rótulo é TEXTO pintado,
//! e o `MockPanelHost` devolve retângulos, não strings.

use std::path::Path;

/// **O id curto é formatado num lugar só.**
///
/// A metade que o gate acima não afirma: um chamador pode ir pela porta E ainda montar um
/// rótulo próprio ao lado. O `entity % 10_000` é a assinatura do fallback, e ele tem de
/// existir **uma** vez na crate inteira — dentro da porta.
///
/// **Mutação que deve sangrar:** qualquer segundo `entity % 10_000` no `src/`.
#[test]
fn the_short_id_is_formatted_in_exactly_one_place() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&src).expect("src/ existe") {
        let path = entry.expect("entrada legível").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("arquivo legível");
        for (i, line) in text.lines().enumerate() {
            if line.contains("entity % 10_000") {
                hits.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
    assert_eq!(
        hits.len(),
        1,
        "o id curto `#nnnn` é o FALLBACK da porta única e só pode ser montado lá; \
         achei em: {hits:?}"
    );
}
