//! **A ARTE DOS PADRÕES no arquivo de projeto** (plano 33, W8) — filho de `project_tests`
//! (declarado lá via `#[path]`), então `super::*` alcança as fixtures dele.
//!
//! ⚠️ **FILHO e não irmão, pela razão exacta do `sculpt` e do `tape`:** as fixtures desta suíte
//! (`headless_app`, `write_project_art`, `tmp_path`, `empty_state`) são as portas dele, e copiá-las
//! seria um segundo escritor de arquivo de projeto — que divergiria no próximo campo novo.
//!
//! O corte foi cobrado pelo teto de LOC do HR-18 e é **por responsabilidade**: aqui só o que a arte
//! dos padrões faz ao *load*.

use super::*;

/// ⭐⭐⭐ **UMA ARTE DE PADRÃO ILEGÍVEL RECUSA O ARQUIVO INTEIRO** — a mesma lei da animação e da
/// escultura, aplicada ao **único blob com versão própria que não a obedecia** (2026-08-30).
///
/// ⚠️ **A cadeia que a ausência desta lei produzia, ponta a ponta:** o blob degrada → o restore
/// devolvia um `eprintln!` e o load **seguia** → a fonte de cada estampa deixa de resolver → toda
/// forma com padrão pinta a cor de recurso → o toast diz *"Project loaded"* → e o Ctrl+S seguinte
/// reescreve o ficheiro **sem** os pixels, porque o coletor salta o `AssetId` que já não está no
/// `AssetDb`. *A arte não some por um defeito; some porque o app abriu, mentiu e salvou* — a frase
/// que o irmão da animação já tinha escrita, com o sujeito trocado.
///
/// ⭐ E ela fecha um buraco de FUTURO: o `PATTERN_ART_DOC_VERSION` vive **fora** da escada do
/// `PROJECT_SCHEMA` de propósito, e o preço era que no dia em que ele subisse um ficheiro anterior
/// abriria **sem a arte** em vez de ser recusado.
#[test]
fn unreadable_pattern_art_refuses_the_whole_file_and_leaves_the_session_alone() {
    let mut app = headless_app();
    app.playhead.play();
    app.playhead.advance_ticks(120);
    let before = app.playhead.time();
    app.undo.push_undo(empty_state());

    let path = tmp_path("load_bad_pattern_art");
    // Não é um `(u32, Vec<SavedPatternArt>)`.
    write_project_art(
        &path,
        PROJECT_SCHEMA,
        Vec::new(),
        Vec::new(),
        vec![0xff, 0xff, 0xff, 0xff],
    );
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(app.playhead.time(), before, "o relógio foi tocado");
    assert!(
        app.undo.can_undo(),
        "o histórico do trabalho aberto NÃO sobreviveu - o load mutou a sessão antes de recusar"
    );
}

/// ⭐⭐ **CONTROLO: o MESMO arquivo, com a arte bem-formada, ABRE.**
///
/// Sem esta linha a lei da recusa poderia ser *"recusa sempre"* e passaria — e um projecto com
/// estampas deixaria de abrir. ⚠️ O blob **vazio** é o caso normal (um projecto sem padrão nenhum
/// não paga byte nenhum) e é coberto pelos irmãos desta suíte, que o escrevem assim; este mede o
/// blob **cheio e legível**, que é o outro lado.
#[test]
fn a_well_formed_pattern_art_blob_still_opens_the_file() {
    let mut app = headless_app();
    app.undo.push_undo(empty_state());
    let arte = crate::project_texture_pattern::encode_for_test(4, 3);

    let path = tmp_path("load_good_pattern_art");
    write_project_art(&path, PROJECT_SCHEMA, Vec::new(), Vec::new(), arte);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    // Um load ACEITE limpa a fila de undo do documento anterior — é o que o distingue de uma
    // recusa, e é a mesma régua que o gate da animação usa ao contrário.
    assert!(
        !app.undo.can_undo(),
        "o arquivo foi RECUSADO - a lei da recusa ficou larga demais e come o caso bom"
    );
}
