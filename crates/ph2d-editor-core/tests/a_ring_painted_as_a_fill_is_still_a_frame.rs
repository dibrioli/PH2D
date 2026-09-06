//! ⭐⭐⭐ **A MOLDURA QUE NÃO É UM TRAÇO — e que o censo do traço não via.**
//!
//! O `every_frame_goes_through_the_theme_door` lê quem chama `stroke_rounded_rect`. Mas há uma
//! segunda forma de desenhar uma moldura, e o app usa-a: **pintar um rectângulo na cor da borda e
//! pôr o conteúdo por cima, recuado**. A amostra de cor fazia-o, e por isso continuou com anel
//! depois de a pele plana ter apagado as molduras dos vizinhos (wave 5 do redesenho, 2026-09-05).
//!
//! ⚠️ **A régua não pode ser «um `fill` na cor da borda é suspeito»**, porque a maior parte deles
//! é legítima e não é moldura nenhuma: um **divisor** (uma linha fina entre secções), a **trilha**
//! de um slider, o separador do trilho de ferramentas. O fonte não diz qual é qual — a espessura
//! vive numa variável.
//!
//! ⇒ a régua é a **DECLARAÇÃO**: todo sítio que pinta um `fill` numa cor de borda está nesta
//! tabela, com o que ele É. Quem acrescentar um anel novo por preenchimento tem de o declarar como
//! `Ring`, e um `Ring` **tem de conhecer a porta** (`paint::fill_ring`). *A espécie morre na
//! próxima vez que alguém a escrever, que é o que um censo compra.*

use std::fs;
use std::path::{Path, PathBuf};

/// O que um `fill` numa cor de borda É.
#[derive(Copy, Clone, PartialEq, Debug)]
enum Kind {
    /// Uma MOLDURA: cobre o controlo e o conteúdo vem por cima, recuado. Passa pela porta.
    Ring,
    /// Uma linha entre duas coisas. Não é moldura de repouso — fica.
    Divider,
    /// A trilha/base de um controlo (o sublinhado de um slider, o poço de uma barra). Conteúdo.
    Track,
}
use Kind::{Divider, Ring, Track};

/// `(ficheiro, linha que contém o `fill`, o que é, porquê)`.
///
/// ⚠️ **A linha é o texto, não o número** — um número envelhece a cada edição do ficheiro.
const DECLARED: &[(&str, &str, Kind, &str)] = &[
    (
        "widget/color_swatch.rs",
        "fill_rounded_rect(scene, rect, radius, colour);",
        Ring,
        "o anel da amostra: rect na cor da borda com a cor do artista por cima, recuada",
    ),
    (
        "widget/showcase/body.rs",
        "scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));",
        Divider,
        "o separador entre duas seccoes da galeria",
    ),
    (
        "widget/status_bar.rs",
        "scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));",
        Divider,
        "o divisor de 1 px entre segmentos da barra",
    ),
    (
        "widget/slider.rs",
        "fill_rounded_rect(scene, track, r, resolve(ColorToken::Border, theme));",
        Track,
        "a TRILHA de um slider desactivado — a superficie do controlo, nao uma moldura",
    ),
    (
        "widget/tool_rail/paint.rs",
        "scene.fill_rect(rect_to_vello(slot_rect), resolve(ColorToken::Border, theme));",
        Divider,
        "o separador entre grupos do trilho (`ToolRailEntry::Divider`)",
    ),
    (
        "widget/property_box/paint.rs",
        "fill_rounded_rect(scene, base, 0.0, resolve(ColorToken::Border, theme));",
        Track,
        "a BASE do slider sublinhado — a linha que o preenchimento de acento cobre",
    ),
    (
        "widget/card.rs",
        "fill_rounded_rect(scene, div_rect, 0.5, resolve(ColorToken::Border, theme));",
        Divider,
        "os divisores entre o corpo, o cabecalho e o rodape do cartao",
    ),
    (
        "widget/radio_group.rs",
        "scene.fill_rect(rect_to_vello(rect), resolve(ColorToken::Border, theme));",
        Track,
        "a bandeja do grupo — a superficie sobre a qual os segmentos assentam",
    ),
];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("dir legivel") {
        let p = entry.expect("entrada legivel").path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Todo `(ficheiro, linha)` de produto que pinta um `fill` numa cor de BORDA.
fn fills_in_a_border_colour() -> Vec<(String, String)> {
    let root = src_root();
    let mut files = Vec::new();
    for top in ["widget", "screens/hero"] {
        walk(&root.join(top), &mut files);
    }
    let mut out = Vec::new();
    for p in files {
        let rel = p
            .strip_prefix(&root)
            .expect("dentro de src/")
            .to_string_lossy()
            .replace('\\', "/");
        if rel.contains("/tests") || rel.ends_with("_tests.rs") || rel.ends_with("tests.rs") {
            continue;
        }
        let src = fs::read_to_string(&p).expect("ficheiro legivel");
        let mut in_tests = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                in_tests = true;
            }
            if in_tests {
                continue;
            }
            let t = line.trim();
            let paints = t.contains("fill_rounded_rect(") || t.contains("fill_rect(");
            let border = t.contains("ColorToken::Border")
                || t.contains("ColorToken::BorderStrong")
                || t.contains("ColorToken::BorderEmph");
            // ⚠️ A amostra pinta a cor JÁ RESOLVIDA (ela vem da porta), então o token não aparece
            //    na linha — é a variável `colour`. Ela entra pelo nome dela.
            let resolved_ring = t.contains("fill_rounded_rect(scene, rect, radius, colour)");
            if paints && (border || resolved_ring) {
                out.push((rel.clone(), t.to_string()));
            }
        }
    }
    out.sort();
    out
}

/// ⭐⭐⭐ **CENSO: todo `fill` numa cor de borda está declarado, e todo ANEL conhece a porta.**
///
/// **Mutação que deve sangrar:** trocar o `paint::fill_ring` do `color_swatch.rs` de volta pelo
/// `fill_rounded_rect(rect, border_token)` directo — o anel deixa de conhecer a porta e volta a
/// pintar-se num tema que prometeu não ter molduras.
#[test]
fn every_fill_in_a_border_colour_is_declared_and_every_ring_knows_the_door() {
    let found = fills_in_a_border_colour();
    let mut undeclared = Vec::new();
    for (file, line) in &found {
        if !DECLARED
            .iter()
            .any(|(f, l, ..)| f == file && line.contains(l))
        {
            undeclared.push(format!("{file}: {line}"));
        }
    }
    assert!(
        undeclared.is_empty(),
        "estes sitios pintam um `fill` numa cor de BORDA e nao dizem o que sao — se for uma \
         MOLDURA, ela nao passa pela porta do tema e sobrevive a pele plana (a amostra de cor \
         fez exactamente isso): {undeclared:#?}\n\
         cura: declare-o em DECLARED como Divider/Track (com o motivo), ou torne-o um Ring que \
         chama `crate::paint::fill_ring`."
    );
    // ⭐ E um `Ring` tem de conhecer a porta — declarar não chega.
    let root = src_root();
    for (file, _, kind, _) in DECLARED {
        if *kind != Ring {
            continue;
        }
        let src = fs::read_to_string(root.join(file)).expect("ficheiro do anel legivel");
        assert!(
            src.contains("fill_ring("),
            "{file} esta' declarado como ANEL e nao chama `paint::fill_ring` — ele pinta a \
             moldura a' revelia do tema"
        );
    }
}

/// ⛔ **A metade que impede a tabela de virar licença:** uma entrada que já não descreve nada sai.
#[test]
fn the_declarations_have_no_stale_entries() {
    let found = fills_in_a_border_colour();
    let stale: Vec<&str> = DECLARED
        .iter()
        .filter(|(f, l, ..)| !found.iter().any(|(ff, ll)| ff == f && ll.contains(l)))
        .map(|(f, ..)| *f)
        .collect();
    assert!(
        stale.is_empty(),
        "estas declaracoes ja' nao descrevem nenhuma linha do produto: {stale:?}"
    );
}
