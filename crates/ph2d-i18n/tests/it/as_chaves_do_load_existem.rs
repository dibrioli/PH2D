//! Toda recusa do load é um toast: uma chave sem texto mostraria o nome da chave ao artista, em silêncio. Nenhum outro
//! censo cobre `shells/desktop`, que chama `toast_refused` com a chave em texto.

use std::path::Path;

fn chaves(fonte: &str) -> Vec<&str> {
    fonte
        .split('"')
        .filter(|s| s.starts_with("shell.project_load."))
        .collect()
}

#[test]
fn toda_chave_do_load_tem_texto() {
    let shell = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src");
    let mut vistas = 0;
    let mut sem_texto = Vec::new();
    for f in ["project_load.rs", "project_load_read.rs"] {
        let fonte = std::fs::read_to_string(shell.join(f)).expect("o fonte lê-se");
        for k in chaves(&fonte) {
            vistas += 1;
            if ph2d_i18n::tr(k) == k {
                sem_texto.push(k.to_owned());
            }
        }
    }
    assert!(vistas >= 8, "o censo só viu {vistas} chaves");
    assert!(sem_texto.is_empty(), "sem texto: {sem_texto:?}");
}

#[test]
fn a_recusa_das_receitas_diz_o_que_aconteceu() {
    let t = ph2d_i18n::tr_with(
        "shell.project_load.project_refused_its_recipes",
        &[("e", &"bad version")],
    );
    assert!(
        t.contains("sound recipes") && t.contains("bad version"),
        "{t}"
    );
}
