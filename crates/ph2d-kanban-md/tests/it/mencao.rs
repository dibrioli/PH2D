//! A menção vai ao markdown e volta com o nome INTEIRO: `@Ana` quando o texto puro a devolve, o
//! link `mention:` quando não (espaço, acento depois do nome, `]`, ponto final).

use ph2d_kanban_md::{parse, render};
use ph2d_richdoc::{Block, Inline, Mark, RichDoc};

/// Um parágrafo `antes @nome depois` com a menção marcada, como o editor a insere.
fn com_mencao(nome: &str) -> RichDoc {
    let mencao = format!("@{nome}");
    let mut inline = Inline::plain("Ask ");
    inline.push(&mencao, &[Mark::Mention]);
    inline.push(" today", &[]);
    RichDoc::from_blocks(vec![Block::new(ph2d_richdoc::BlockKind::Paragraph, inline)])
}

fn mencoes(d: &RichDoc) -> Vec<String> {
    let i = &d.blocks()[0].inline;
    i.marks()
        .iter()
        .filter(|s| s.mark == Mark::Mention)
        .map(|s| i.text()[s.start..s.end].to_owned())
        .collect()
}

#[test]
fn mencao_com_espaco_acentos_e_colchete_volta_com_o_nome_inteiro() {
    for nome in [
        "Ana",
        "Ana Silva",
        "João Conceição",
        "Zoë",
        "Ana ]x[ Silva",
        "R2-D2.",
        "Ana (dev) Silva",
        "Ana\\Silva",
        "Mary-Jane O'Neil",
        "李 小龍",
    ] {
        let doc = com_mencao(nome);
        let md = render(&doc);
        let back = parse(&md);
        assert_eq!(
            back.blocks()[0].inline.text(),
            doc.blocks()[0].inline.text(),
            "{nome}: o texto volta igual ({md:?})"
        );
        assert_eq!(mencoes(&back), vec![format!("@{nome}")], "{nome}: {md:?}");
        assert_eq!(render(&back), md, "{nome}: ponto fixo");
    }
}

#[test]
fn mencao_simples_fica_arroba_e_a_composta_e_um_link_legivel() {
    assert_eq!(render(&com_mencao("Ana")), "Ask @Ana today\n");
    assert_eq!(
        render(&com_mencao("Ana Silva")),
        "Ask [@Ana Silva](mention:Ana%20Silva) today\n"
    );
    assert_eq!(
        render(&com_mencao("João Conceição")),
        "Ask [@João Conceição](mention:Jo%C3%A3o%20Concei%C3%A7%C3%A3o) today\n"
    );
    assert_eq!(
        render(&com_mencao("Ana ]x[ Silva")),
        "Ask [@Ana \\]x\\[ Silva](mention:Ana%20%5Dx%5B%20Silva) today\n"
    );
}
