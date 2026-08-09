//! Gates do **emissor** (plano UI/UX W8b).

use super::*;

fn row(kind: &str, label: &str, key: &str) -> RowSpec {
    RowSpec {
        kind: kind.into(),
        label: label.into(),
        key: key.into(),
        rgba: None,
        icon: None,
        icon_slug: None,
    }
}

/// A mesma row, com a cor que só um tipo de cor carrega.
fn row_tinted(kind: &str, label: &str, key: &str, rgba: [u8; 4]) -> RowSpec {
    RowSpec {
        rgba: Some(rgba),
        ..row(kind, label, key)
    }
}

/// A mesma row, com o glifo DESENHADO que só um botão de ícone carrega.
fn row_iconed(kind: &str, label: &str, key: &str, d: &str) -> RowSpec {
    RowSpec {
        icon: Some(d.into()),
        ..row(kind, label, key)
    }
}

/// A mesma row, com o ícone ESCOLHIDO — a outra rota do glifo.
fn row_chosen(kind: &str, label: &str, key: &str, slug: &str) -> RowSpec {
    RowSpec {
        icon_slug: Some(slug.into()),
        ..row(kind, label, key)
    }
}

fn demo() -> PanelSpec {
    PanelSpec {
        id: "demo".into(),
        title: "Demo".into(),
        rows: vec![
            row("SectionHeader", "Aparência", "aparencia"),
            row("Slider", "Opacity", "opacity"),
            row("Toggle", "Visível", "visivel"),
        ],
    }
}

/// **A mesma entrada dá os MESMOS bytes** — e é isso que torna o gate de staleness possível.
///
/// ⚠️ Sem determinismo, um gerador não pode ser vigiado: o gate compararia o arquivo commitado com
/// uma saída que muda sozinha, e a única forma de o manter verde seria desligá-lo.
#[test]
fn the_same_spec_emits_the_same_bytes() {
    assert_eq!(emit(&demo()), emit(&demo()));
}

/// **A ordem das rows é a ordem do spec** — nunca uma ordenação própria.
///
/// ⚠️ O artista ordena as coisas na árvore, e essa ordem É a decisão dele. Um gerador que
/// ordenasse por nome ou por tipo estaria a re-decidir o desenho, e o painel sairia noutra ordem
/// que a moldura mostra.
#[test]
fn the_rows_come_out_in_the_order_they_went_in() {
    let src = emit(&demo());
    let at = |needle: &str| src.find(needle).expect("a row saiu do emitido");
    assert!(
        at("\"Aparência\"") < at("\"Opacity\"") && at("\"Opacity\"") < at("\"Visível\""),
        "o emissor reordenou as rows:\n{src}"
    );
}

/// **O tipo sai como CAMINHO do catálogo, e é o compilador que o confere.**
///
/// ⚠️ É esta linha que faz o gerado quebrar o build quando alguém renomeia um variante do
/// `WidgetKind` — em vez de compilar e pintar a coisa errada.
#[test]
fn the_kind_comes_out_as_a_catalogue_path() {
    let src = emit(&demo());
    assert!(
        src.contains("kind: WidgetKind::Slider,") && src.contains("label: \"Opacity\","),
        "a row nao saiu com o caminho do catalogo:\n{src}"
    );
}

/// **Um rótulo com aspas não produz um arquivo que não compila.**
///
/// ⚠️ O rótulo vem do `Name` da entidade, que o artista DIGITA. Um nome de camada com aspas — ou
/// com uma quebra de linha vinda de um copiar-e-colar — fecharia o literal no meio, e o modo de
/// falha seria um arquivo gerado que o `cargo` recusa com um erro que não fala do desenho.
#[test]
fn a_label_the_artist_typed_cannot_break_the_literal() {
    let s = PanelSpec {
        id: "x".into(),
        title: "x".into(),
        rows: vec![row("Button", "diz \"olá\"\ne quebra\\", "k")],
    };
    let src = emit(&s);
    assert!(
        src.contains(r#""diz \"olá\"\ne quebra\\""#),
        "o literal nao foi escapado:\n{src}"
    );
    // E o balanço de aspas não-escapadas é PAR — a prova de que nenhum literal ficou aberto.
    let mut open = 0usize;
    let mut prev_backslash = false;
    for c in src.chars() {
        match c {
            '"' if !prev_backslash => open += 1,
            _ => {}
        }
        prev_backslash = c == '\\' && !prev_backslash;
    }
    assert_eq!(open % 2, 0, "sobrou um literal aberto:\n{src}");
}

/// **Um painel sem rows emite uma tabela VAZIA, não um erro.**
///
/// ⚠️ Uma moldura recém-desenhada não tem filhos vestidos, e é justamente aí que o artista aperta
/// o botão pela primeira vez. Recusar seria a feature parecer quebrada no único momento em que ela
/// é descoberta — a mesma lei da *face vazia* que a seção de física e a de estados já seguem.
#[test]
fn an_empty_panel_emits_an_empty_table() {
    let src = emit(&PanelSpec {
        id: "vazio".into(),
        title: "Vazio".into(),
        rows: Vec::new(),
    });
    assert!(
        src.contains("pub const ROWS: &[RowConst] = &[\n];"),
        "a tabela vazia nao saiu bem formada:\n{src}"
    );
}

/// **A cor sai como um literal que o Rust aceita, e só onde ela existe.**
///
/// ⚠️ O `None` não é a ausência do campo: a struct tem sempre os cinco, e omitir um não compila —
/// que é metade da razão de ela ter substituído a tupla. E o `Some` sai com os quatro canais:
/// deixar o alfa de fora faria uma swatch transparente emitir como opaca, em silêncio.
#[test]
fn the_colour_comes_out_as_a_literal_only_where_it_exists() {
    let src = emit(&PanelSpec {
        id: "demo".into(),
        title: "Demo".into(),
        rows: vec![
            row("Slider", "Opacity", "opacity"),
            row_tinted("ColorSwatch", "Tint", "tint", [214, 92, 64, 128]),
        ],
    });
    assert!(
        src.contains("label: \"Opacity\",\n        key: \"opacity\",\n        rgba: None,"),
        "quem nao tem cor emitiu alguma coisa:\n{src}"
    );
    assert!(
        src.contains("rgba: Some([214, 92, 64, 128]),"),
        "a cor nao saiu com os quatro canais:\n{src}"
    );
}

/// **O glifo sai como um literal de string, e só onde ele existe.**
///
/// ⚠️ E ele passa pelo MESMO escapador do rótulo: um `d` de SVG não tem aspas hoje, mas o campo é
/// texto vindo de fora desta crate, e um emissor que concatenasse a string crua produziria um
/// arquivo que não compila no dia em que ela tivesse uma. *O escape é do CAMPO, não do conteúdo
/// que ele carrega hoje.*
#[test]
fn the_glyph_comes_out_as_a_string_only_where_it_exists() {
    let src = emit(&PanelSpec {
        id: "demo".into(),
        title: "Demo".into(),
        rows: vec![
            row("Slider", "Opacity", "opacity"),
            row_iconed("IconButton", "Play", "play", "M0,0 L2,1 \"Z\""),
        ],
    });
    assert!(
        src.contains("label: \"Opacity\",\n        key: \"opacity\",\n        rgba: None,\n        icon: None,"),
        "quem nao tem glifo emitiu alguma coisa:\n{src}"
    );
    assert!(
        src.contains(r#"icon: Some("M0,0 L2,1 \"Z\""),"#),
        "o glifo nao saiu escapado:\n{src}"
    );
}

/// **O ícone ESCOLHIDO sai pelo SLUG, e nunca ao lado do desenho.**
///
/// ⚠️ A segunda metade é a que importa: os dois campos são mutuamente exclusivos por construção
/// (quem os preenche pergunta a precedência à porta única do catálogo), e um emissor que
/// escrevesse os dois deixaria o painel compilado a escolher sozinho — uma segunda resposta a
/// *qual ícone?*, escondida dentro do gerado.
#[test]
fn the_chosen_icon_comes_out_as_a_slug_and_never_beside_a_drawing() {
    let src = emit(&PanelSpec {
        id: "demo".into(),
        title: "Demo".into(),
        rows: vec![
            row_chosen("IconButton", "Trash", "trash", "trash"),
            row_iconed("IconButton", "Play", "play", "M0,0 L2,1 Z"),
        ],
    });
    assert!(
        src.contains("icon: None,\n        icon_slug: Some(\"trash\"),"),
        "o slug nao saiu, ou saiu ao lado de um desenho:\n{src}"
    );
    assert!(
        src.contains("icon: Some(\"M0,0 L2,1 Z\"),\n        icon_slug: None,"),
        "o desenho saiu ao lado de um slug:\n{src}"
    );
}
