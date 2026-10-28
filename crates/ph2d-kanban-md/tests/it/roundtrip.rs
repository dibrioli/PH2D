//! Lei: `md → doc → md` é ponto fixo depois da 1.ª normalização, sobre o corpus `fixtures/`.

use ph2d_kanban_md::{normalize, parse};

pub const CORPUS: [(&str, &str); 6] = [
    ("01_blocos", include_str!("fixtures/01_blocos.md")),
    ("02_marcas", include_str!("fixtures/02_marcas.md")),
    ("03_maus", include_str!("fixtures/03_maus.md")),
    ("04_listas", include_str!("fixtures/04_listas.md")),
    ("05_estrangeiro", include_str!("fixtures/05_estrangeiro.md")),
    ("06_codigo", include_str!("fixtures/06_codigo.md")),
];

#[test]
fn roundtrip_md_doc_md_is_a_fixed_point_after_the_first_normalization() {
    for (name, md0) in CORPUS {
        let md1 = normalize(md0);
        let md2 = normalize(&md1);
        if std::env::var_os("PH2D_MD_DUMP").is_some() {
            eprintln!("===== {name}\n{md1}");
        }
        assert_eq!(md1, md2, "{name}: não é ponto fixo");
        assert_eq!(parse(&md1), parse(&md2), "{name}: o documento muda");
    }
}

#[test]
fn roundtrip_every_line_prefix_of_the_corpus_is_also_a_fixed_point() {
    // Cortes arbitrários fazem casos maus que ninguém escreveu à mão (listas a meio, cercas
    // por fechar, marcas abertas).
    for (name, md0) in CORPUS {
        let lines: Vec<&str> = md0.lines().collect();
        for n in 0..=lines.len() {
            let cut = lines[..n].join("\n");
            let md1 = normalize(&cut);
            assert_eq!(
                md1,
                normalize(&md1),
                "{name}: prefixo de {n} linhas:\n{cut}"
            );
        }
    }
}

#[test]
fn roundtrip_foreign_markdown_survives_as_literal_text() {
    let doc = parse(CORPUS[4].1);
    let text = doc.plain_text();
    for needle in [
        "| col a | col b |",
        "<div class=\"x\">",
        "<b>html</b>",
        "<span>inline</span>",
        "![imagem](https://i.png \"título\")",
        "[^1]",
        "$math$",
    ] {
        assert!(text.contains(needle), "perdeu {needle:?} em:\n{text}");
    }
    // A 1.ª volta normaliza (os espaços no começo das linhas do HTML caem); daí em diante igual.
    let once = normalize(CORPUS[4].1);
    let once_text = parse(&once).plain_text();
    assert!(once_text.contains("<b>html</b>") && once_text.contains("| 1     | 2     |"));
    assert_eq!(parse(&normalize(&once)).plain_text(), once_text);
}
