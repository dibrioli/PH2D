//! W11 (o quadro ao vivo, ADR-0181) no editor: as chaves novas de z e de ordem saem pela porta do
//! documento (`BoardDoc::key_between`), que lhes põe a cauda da sessão.

use super::*;

/// ⛔ Census: nenhuma ferramenta calcula uma chave nova por conta própria — só pela porta do
/// documento. `FracKey::between(None, None)` (a 1.ª chave de uma lista nova, sem irmãos) é a excepção.
#[test]
fn no_tool_mints_a_key_outside_the_documents_door() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut seen = 0;
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if !name.ends_with(".rs") || name.ends_with("_tests.rs") || name == "tests.rs" {
            continue;
        }
        seen += 1;
        let src = std::fs::read_to_string(&p).unwrap();
        for (i, line) in src.lines().enumerate() {
            if line.contains("FracKey::between(") && !line.contains("FracKey::between(None, None)") {
                bad.push(format!("{name}:{}", i + 1));
            }
        }
    }
    assert!(seen > 20, "a varredura leu {seen} ficheiros");
    assert!(bad.is_empty(), "chaves fora da porta do documento: {bad:?}");
}

/// Duas pessoas a criar uma forma no topo ao mesmo tempo, cada uma na sua sessão: as chaves de z
/// diferem (`colab_oracle` c05 — no Excalidraw nascem as duas com `a0`).
#[test]
fn two_live_editors_create_on_top_without_tying() {
    let mut a = world();
    a.rect([0.0, 0.0, 10.0, 10.0]);
    let mut b = world();
    b.doc = a.doc.clone();
    a.doc.set_live(Some(7));
    b.doc.set_live(Some(8));
    let ra = a.rect([0.0, 0.0, 10.0, 10.0]);
    let rb = b.rect([0.0, 0.0, 10.0, 10.0]);
    assert_ne!(ra, rb);
    assert_ne!(a.el(ra).z, b.el(rb).z);
}
