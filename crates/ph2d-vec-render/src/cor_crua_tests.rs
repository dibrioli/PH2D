//! **Censo de COR CRUA nos overlays desta crate** (HR-15, F30 §E) — um `Color::from_rgba8(<número>…`
//! num ficheiro de produto é uma tinta que não segue o tema.
//!
//! Catraca EXACTA por ficheiro: os já tokenizados ficam em zero; a dívida que resta está contada e
//! só desce — e quem a baixar reescreve o número (descer sem reescrever reprova, para a lista não
//! virar licença).

use std::fs;
use std::path::Path;

/// A dívida medida em 25/10 (F30 §E). Fora daqui, zero.
const DIVIDA: &[(&str, usize)] = &[
    ("connector.rs", 5),
    ("cut_line.rs", 2),
    ("gradient.rs", 2),
    ("guides.rs", 2),
    ("marquee.rs", 4),
    ("symmetry_overlay.rs", 1),
];

fn cruas(src: &str) -> usize {
    let agulha = "from_rgba8(";
    src.match_indices(agulha)
        .filter(|(i, _)| {
            src[i + agulha.len()..]
                .trim_start()
                .starts_with(|c: char| c.is_ascii_digit())
        })
        .count()
}

#[test]
fn o_overlay_vetorial_nao_tem_cor_crua() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let (mut vistos, mut chamadas) = (0usize, 0usize);
    let mut achado: Vec<(String, usize)> = Vec::new();
    for e in fs::read_dir(&dir).expect("src/").flatten() {
        let nome = e.file_name().to_string_lossy().into_owned();
        if !nome.ends_with(".rs") || nome.ends_with("_tests.rs") {
            continue;
        }
        let src = fs::read_to_string(e.path()).expect("ler");
        vistos += 1;
        chamadas += src.matches("from_rgba8(").count();
        let n = cruas(&src);
        if n > 0 {
            achado.push((nome, n));
        }
    }
    // Controlo positivo: a varredura vê os ficheiros e as chamadas (o `token_ink` usa uma).
    assert!(vistos > 30, "a varredura achou só {vistos} ficheiros");
    assert!(
        chamadas > 0,
        "nenhum `from_rgba8(` — a agulha deixou de casar"
    );
    assert_eq!(
        cruas("Color::from_rgba8( 1, 2, 3, 4)"),
        1,
        "o detector não vê uma cor crua"
    );
    achado.sort();
    let esperado: Vec<(String, usize)> = DIVIDA.iter().map(|(f, n)| ((*f).into(), *n)).collect();
    assert_eq!(
        achado, esperado,
        "cor crua nos overlays: um ficheiro novo (ou o fio/hover/nós) usa RGBA fixo — use \
         `crate::token_ink(ColorToken::…, theme)`; ou a dívida desceu e a tabela `DIVIDA` tem de \
         descer com ela"
    );
}
