use super::*;
use crate::raster;

const HEAD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60">"#;

fn svg(body: &str) -> String {
    format!("{HEAD}{body}</svg>")
}

/// Os píxeis com alguma tinta, e os de tinta azul dominante.
fn ink(src: &str) -> (usize, usize) {
    let px = raster::rasterize(&raster::parse(src.as_bytes()).unwrap(), [200, 60]).unwrap();
    let any = px.chunks(4).filter(|p| p[3] > 0).count();
    let blue = px
        .chunks(4)
        .filter(|p| p[3] > 128 && p[2] > 200 && p[0] < 60)
        .count();
    (any, blue)
}

/// Sem `<text>` não há passagem: os MESMOS bytes, sem cópia (o caminho rápido).
#[test]
fn an_svg_without_text_pays_nothing() {
    let s = svg(r#"<rect width="9" height="9"/><g id="context"/>"#);
    assert!(outline_texts(s.as_bytes()).is_none());
    assert_eq!(text_gaps(s.as_bytes()), TextGaps::default());
}

/// A família ausente NOMEIA-SE (as genéricas e as presentes não); a da lista que vinha antes da usada
/// também.
#[test]
fn a_missing_family_is_named() {
    let g = |f: &str| text_gaps(svg(&format!(r#"<text font-family="{f}">a</text>"#)).as_bytes());
    assert_eq!(g("Nao Existe Font").missing_families, ["Nao Existe Font"]);
    assert_eq!(
        g("'Nao Existe', 'DejaVu Serif', serif").missing_families,
        ["Nao Existe"]
    );
    assert_eq!(
        g("DejaVu Sans, 'Depois'").missing_families,
        Vec::<String>::new()
    );
    assert_eq!(g("serif").missing_families, Vec::<String>::new());
    // Gémeo de métricas e nome alternativo (o Chromium): não faltam.
    assert_eq!(g("Arial").missing_families, Vec::<String>::new());
    assert_eq!(g("Helvetica").missing_families, Vec::<String>::new());
    assert_eq!(g("Arial").undrawn, 0);
}

/// O que não se desenha como escrito diz-se pelo nome, e conta.
#[test]
fn the_undrawn_constructs_are_reported() {
    let cases = [
        (
            r##"<path id="p" d="M0 9H99"/><text><textPath href="#p">a</textPath></text>"##,
            "textPath",
        ),
        (r#"<text rotate="30">ab</text>"#, "rotate"),
        (r#"<text textLength="90">ab</text>"#, "textLength"),
        (
            r#"<g writing-mode="tb"><text>ab</text></g>"#,
            "writing-mode",
        ),
        (
            r#"<text>a<tspan baseline-shift="super">b</tspan></text>"#,
            "baseline-shift",
        ),
        (
            r#"<text style="text-decoration: underline">ab</text>"#,
            "text-decoration",
        ),
    ];
    for (body, what) in cases {
        let g = text_gaps(svg(body).as_bytes());
        assert_eq!(g.unsupported, [what], "{body}");
        assert!(g.undrawn >= 1, "{body}");
    }
    // `rotate="0"` e `writing-mode` horizontal não são lacunas.
    let ok = svg(r#"<text rotate="0" writing-mode="horizontal-tb">ab</text>"#);
    assert_eq!(text_gaps(ok.as_bytes()), TextGaps::default());
}

/// Uma regra do `<style>` por TIPO (`text {…}`) continua a valer no `<g>` que substitui o texto: a
/// cor e o tamanho.
#[test]
fn a_type_selector_rule_still_applies() {
    let css = svg(
        r#"<style>text { fill: #0000ff; font-size: 40px; font-family: 'DejaVu Sans' }</style><text x="5" y="45">Hi</text>"#,
    );
    let px = raster::rasterize(&raster::parse(css.as_bytes()).unwrap(), [200, 60]).unwrap();
    let solid = px.chunks(4).filter(|p| p[3] > 128);
    let (blue, other) = solid.fold((0, 0), |(b, o), p| {
        if p[2] > 200 && p[0] < 60 {
            (b + 1, o)
        } else {
            (b, o + 1)
        }
    });
    assert!(blue > 100 && other == 0, "azul {blue}, outra cor {other}");
    let laid = lay_out_texts(&css);
    // «H» da DejaVu Sans a 40 px avança ~30 (o t01 do oráculo).
    assert!(
        (laid[0].at[1][0] - laid[0].at[0][0] - 30.0).abs() < 0.5,
        "{:?}",
        laid[0].at
    );
}

/// Um documento com prefixo (`<svg:text>`) troca-se pelo mesmo prefixo.
#[test]
fn a_prefixed_document_keeps_its_prefix() {
    let s = r#"<svg:svg xmlns:svg="http://www.w3.org/2000/svg" width="200" height="60"><svg:text x="5" y="40" font-size="30">Hi</svg:text></svg:svg>"#;
    assert!(ink(s).0 > 50);
}

/// Dentro de um `<clipPath>` (que não aceita `<g>`) o texto recorta: um rectângulo cheio só se vê
/// onde estão as letras.
#[test]
fn a_text_clip_path_clips() {
    let s = svg(
        r##"<clipPath id="c"><text x="5" y="45" font-size="40" font-family="DejaVu Sans">Hi</text></clipPath><rect width="200" height="60" fill="#0000ff" clip-path="url(#c)"/>"##,
    );
    let (any, blue) = ink(&s);
    assert!(blue > 100, "{blue}");
    assert!(any < 200 * 60 / 4, "o rectângulo não foi recortado: {any}");
}

/// Uma entidade no conteúdo e um SVG aninhado num `data:` também se desenham.
#[test]
fn entities_and_nested_svgs_draw() {
    let e = format!(
        r#"<!DOCTYPE svg [<!ENTITY w "Hi">]>{}"#,
        svg(r#"<text x="5" y="40" font-size="30">&w;</text>"#)
    );
    assert!(ink(&e).0 > 50);
    let inner = svg(r#"<text x="5" y="40" font-size="30">Hi</text>"#);
    let uri: String = inner
        .bytes()
        .map(|b| match b {
            b'<' => "%3C".to_owned(),
            b'>' => "%3E".to_owned(),
            b'"' => "%22".to_owned(),
            b'#' => "%23".to_owned(),
            b' ' => "%20".to_owned(),
            b => (b as char).to_string(),
        })
        .collect();
    let outer = svg(&format!(
        r#"<image width="200" height="60" href="data:image/svg+xml,{uri}"/>"#
    ));
    assert!(ink(&outer).0 > 50);
}

/// O `font` abreviado entra (o negrito alarga as letras).
#[test]
fn the_font_shorthand_is_read() {
    let w = |style: &str| {
        let s = svg(&format!(
            r#"<text style="font: {style} 30px 'DejaVu Sans'">Hm</text>"#
        ));
        let t = &lay_out_texts(&s)[0];
        t.at[1][0] - t.at[0][0]
    };
    assert!(w("bold") > w("normal") + 1.0);
}

/// O mesmo ficheiro desenha-se a cada nível de zoom: a passagem guarda-se e não se refaz.
#[test]
fn the_pass_is_kept_per_file() {
    let s = svg(r#"<text x="5" y="40">guardado por ficheiro</text>"#);
    let a = outline_texts(s.as_bytes()).unwrap();
    let b = outline_texts(s.clone().as_bytes()).unwrap();
    assert!(Arc::ptr_eq(&a, &b));
}

/// O caracter que a família pedida não tem cai na reserva do sistema (aqui, um ideograma que a DejaVu
/// não tem): desenha-se, não sai o `.notdef`.
#[test]
fn a_character_the_family_lacks_falls_back() {
    let s = svg(r#"<text x="5" y="40" font-family="DejaVu Sans" font-size="30">a漢é</text>"#);
    let t = &lay_out_texts(&s)[0];
    assert_eq!(t.at.len(), 3);
    assert_eq!(t.notdef, 0, "um glifo .notdef");
    assert!(
        t.at[1][0] > t.at[0][0] && t.at[2][0] > t.at[1][0] + 15.0,
        "{:?}",
        t.at
    );
}
