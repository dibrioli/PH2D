//! O tocador NUNCA lê o relógio de parede (plano §2): o tempo entra sempre pelo chamador.
//! Uma macro que consultasse `Instant` tocaria diferente em cada máquina.

use std::path::Path;

#[test]
fn the_macro_core_never_reads_the_wall_clock() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut lidos = 0;
    for entry in std::fs::read_dir(&src).expect("src/") {
        let path = entry.expect("entrada").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("lê");
        lidos += 1;
        for proibido in ["Instant", "SystemTime", "std::time", "thread::sleep"] {
            assert!(
                !text.contains(proibido),
                "{} usa `{proibido}`",
                path.display()
            );
        }
    }
    // Controlo: a varredura viu os ficheiros do crate (uma pasta errada passaria vazia).
    assert!(lidos >= 6, "só {lidos} ficheiros varridos");
}
