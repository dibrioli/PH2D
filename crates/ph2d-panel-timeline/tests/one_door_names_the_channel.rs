//! **A row e o card nomeiam o canal pela MESMA porta** (FASE C.3 do plano 12).
//!
//! O defeito que este arquivo existe para impedir já aconteceu, e durou o tempo de vida
//! inteiro do card: o doc-comment do `ExprModal::title` prometia `"Ball · Position Y"`
//! enquanto o código montava `prop_label + "  #" + entity % 10000` — porque nada publicava
//! o nome do objeto e cada consumidor formatava o rótulo por conta própria. Duas cópias de
//! *"como este app chama um canal"* divergem no dia em que uma delas ganha o nome.
//!
//! ⚠️ **Por que um arch-gate e não só um teste de comportamento:** o rótulo é TEXTO
//! pintado, e o `MockPanelHost` devolve retângulos, não strings — um gate de unidade prova
//! que a porta responde certo, e não que os dois chamadores passam por ela. Este lê a
//! FONTE, com controle positivo nas duas pontas (um marcador que some por renomeação de
//! arquivo deixaria o gate verde sobre nada).

use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} não pôde ser lido: {e}", p.display()))
}

/// **Os dois chamadores passam por `track_label`.**
///
/// **Mutação que deve sangrar:** o card voltar a montar a string dele
/// (`format!("{}  #{}", prop_label(...), entity % 10_000)`).
#[test]
fn the_row_and_the_card_both_go_through_the_label_door() {
    let tracks = read("src/tracks.rs");
    let card = read("src/expr_modal_paint.rs");

    // Controle positivo: os dois arquivos ainda são os que eu penso que são.
    assert!(
        tracks.contains("fn track_label("),
        "a porta mudou de casa — este gate está lendo o arquivo errado"
    );
    assert!(
        card.contains("fn paint_title_band("),
        "o card mudou de casa — este gate está lendo o arquivo errado"
    );

    assert!(
        tracks.contains("track_label(snap.object_name("),
        "a ROW do dope-sheet tem de rotular pela porta única"
    );
    assert!(
        card.contains("track_label(snap.object_name("),
        "o TÍTULO do card tem de rotular pela porta única — foi exactamente esta linha \
         que passou a existência inteira do card dizendo `#nnnn` sob um doc que prometia \
         o nome do objeto"
    );
}

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
