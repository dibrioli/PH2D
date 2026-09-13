//! ⭐⭐ **Os controlos da régua — cada forma aqui já custou um censo errado, ou é a razão de a régua
//! anterior ver 44 % de um painel.**
//!
//! Um leitor de fonte que só se prova sobre a árvore do dia prova-se sobre as formas que a árvore
//! do dia por acaso tem. Estes textos são as formas, escritas de propósito, e cada teste falha
//! ALTO se uma delas voltar a enganar o leitor.

use ph2d_label_census::{is_language, language_literals_in};

fn texts(src: &str) -> Vec<String> {
    language_literals_in("x.rs", src)
        .into_iter()
        .map(|l| l.text)
        .collect()
}

/// ⭐⭐⭐ **As três formas que a régua anterior NÃO via** — e que são a maioria do texto de um painel.
#[test]
fn a_constructor_a_table_and_a_format_are_all_seen() {
    let src = r#"
        fn f(i: usize) {
            let b = Button::new(ID, "Apply");
            let labels = ["Paint", "Erase"];
            let seg = [(ID_R, "Reset"), (ID_K, "Keep")];
            let cb = Checkbox::new(ID_C, format!("Layer {} Color", i + 1));
            let opt = Some("Convert to Curve");
            let v: String = "Wet".into();
        }
    "#;
    assert_eq!(
        texts(src),
        [
            "Apply",
            "Paint",
            "Erase",
            "Reset",
            "Keep",
            "Layer {} Color",
            "Convert to Curve",
            "Wet"
        ]
    );
}

/// ⛔ `find('"')` é Rust legítimo — o censo de 10/09 leu aquela aspa como uma string a abrir e o
/// resto do ficheiro ao contrário (`438` onde havia `418`).
#[test]
fn a_char_literal_with_a_quote_does_not_open_a_string() {
    let src = r#"fn f(s: &str) { let k = s.find('"'); let e = '\\'; let m = '\u{00b7}'; paint(s, "Size"); }"#;
    assert_eq!(texts(src), ["Size"]);
}

/// ⚠️ Um tempo de vida abre com `'` e não fecha.
#[test]
fn a_lifetime_is_not_a_char_literal() {
    let src = r#"fn f<'a>(s: &'a str) -> &'a str { label(s, "Opacity") }"#;
    assert_eq!(texts(src), ["Opacity"]);
}

#[test]
fn a_raw_string_is_read_whole_and_a_raw_identifier_is_not_a_string() {
    let src = "fn f() { let r#type = 1; label(r#\"Say \"Hi\" now\"#); }";
    assert_eq!(texts(src), ["Say \"Hi\" now"]);
}

#[test]
fn comments_and_docs_never_count() {
    let src = r#"
        /// "Hidden Doc"
        //! "Inner Doc"
        fn f() { /* "Block Words" /* "Nested Words" */ */ label("Shown"); // "Trailing Words"
        }
    "#;
    assert_eq!(texts(src), ["Shown"]);
}

/// Nada disto chega a um ecrã — e cada exclusão tem o vizinho que TEM de ficar.
#[test]
fn what_never_paints_is_left_out() {
    let src = r#"
        #[cfg(feature = "Some Feature")]
        #[deprecated(note = "Use Something Else")]
        fn f(x: &str, o: Option<u8>) {
            panic!("Boom it broke");
            let _ = o.expect("Must be set");
            // ⚠️ Uma chave de um prefixo que NÃO existe: o censo de chaves varre o repo inteiro, e a
            //    1.ª redacção deste controlo (`panel.painter_layers.size`) foi lida como consumo real
            //    pelo gate do painel Painter — o censo a acusar a fixtura dele próprio.
            let _ = tr("exemplo.de.chave");
            let _ = hash_node_id("painter_size");
            let _ = x.starts_with("Prefix Words");
            let n = match x { "Scene Root" => 1, "Alpha Row" | "Beta Row" => 2, _ => 3 };
            if x == "Other Name" {}
            if "Left Name" != x {}
            eprintln!("Logged Words {n}");
            label("Kept Label");
        }
    "#;
    assert_eq!(texts(src), ["Kept Label"]);
}

/// ⚠️ **O `|` que precede NÃO decide sozinho**: um fecho que DEVOLVE um rótulo é produto.
#[test]
fn a_closure_returning_a_label_still_counts() {
    let src = r#"
        fn f(o: Option<&str>) {
            let a = o.map_or_else(|| "Fallback Label", |x| x);
            let b = o.map(|_| "Mapped Label");
            let c = o.unwrap_or("Plain Default");
        }
    "#;
    assert_eq!(
        texts(src),
        ["Fallback Label", "Mapped Label", "Plain Default"]
    );
}

#[test]
fn an_inline_test_item_is_not_product() {
    let src = r#"
        #[cfg(test)]
        mod tests { fn t() { label("Inside Test"); } }
        #[cfg(all(test, feature = "x"))]
        fn helper() { label("Helper Test"); }
        #[cfg(any(test, feature = "x"))]
        fn shipped() { label("Shipped With Feature"); }
        fn f() { label("Outside"); }
    "#;
    assert_eq!(texts(src), ["Shipped With Feature", "Outside"]);
}

#[test]
fn byte_strings_are_not_text() {
    let src = r#"fn f() { let b = b"Header Bytes"; let c = br"Raw Bytes"; label("Text"); }"#;
    assert_eq!(texts(src), ["Text"]);
}

#[test]
fn lines_count_from_one_and_survive_comments() {
    let src = "// \"no\"\n/* \"a\nb\" */\nfn f() {\n    label(\"Here\");\n}\n";
    let got = language_literals_in("x.rs", src);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].line, 5);
}

/// O critério de língua, com os dois lados de cada fronteira.
#[test]
fn the_language_test_has_both_sides_of_every_border() {
    // palavras
    assert!(is_language("Scene Root"));
    assert!(is_language("Size"));
    assert!(is_language("no canvas"));
    assert!(is_language("Brush over a defect · release to heal."));
    // símbolos sem palavra
    assert!(!is_language("X"));
    assert!(!is_language("%"));
    assert!(!is_language("{v:.2}%"));
    // marcadores de format!: a frase conta, a unidade não
    assert!(is_language("{entities} entities"));
    assert!(!is_language("{} px"));
    assert!(!is_language("{lid}:{ch}"));
    // chaves e identificadores
    assert!(!is_language("chrome.fill.title"));
    assert!(!is_language("area.menu.{slot}"));
    assert!(!is_language("painter_layers"));
    assert!(!is_language("SCREAMING_CASE"));
    assert!(!is_language("kebab-id"));
    assert!(!is_language("wgsl"));
    // caminhos
    assert!(!is_language("docs/design/icons/bone.svg"));
    assert!(!is_language("Cargo.toml"));
}

/// ⛔⛔ **Um escape do fonte NÃO é um caminho.** A 1.ª redacção deitava fora todo texto com `\`, e
/// com ele todo rótulo que tivesse um `\u{…}`, um `\n` ou um `\"` — mudo, até a Hierarquia perder a
/// frase do contador de entidades.
#[test]
fn a_source_escape_is_not_a_path_and_is_not_a_placeholder() {
    assert!(is_language(
        r"{entities} entities \u{00b7} {components} components"
    ));
    assert!(is_language(r"Line one\nLine two"));
    assert!(is_language(r#"Say \"Hi\""#));
    // o `\u{2026}` sozinho continua a não ser palavra
    assert!(!is_language(r"\u{2026}"));
    let src =
        "fn f(a: u32, b: u32) { let t = format!(\"{a} entities \\u{00b7} {b} components\"); }";
    assert_eq!(texts(src), ["{a} entities \\u{00b7} {b} components"]);
}
