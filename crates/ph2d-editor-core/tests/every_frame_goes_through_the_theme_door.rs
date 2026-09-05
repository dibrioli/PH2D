//! ⭐⭐⭐ **Toda MOLDURA de controlo passa pela porta do TEMA — e a dívida que ainda não passa tem
//! nome, e só encolhe.**
//!
//! Decisão do Enio (2026-09-04): o modelo é o Godot 4.6 «Modern» — *«minimalista, plana, concisa,
//! coerente e simples»*. A pele plana é uma propriedade do TEMA, e a porta que a impõe é
//! `paint::stroke_frame` / `paint::frame_radius` (⇒ `ph2d_tokens::visuals::{frame, radius}`): no
//! clássico o pintor traça o que sempre traçou; num tema moderno traça o que a tabela diz — nada
//! em repouso.
//!
//! ⛔ **Por que um censo pelo FONTE, e não só o gate de produto ao lado:** um pintor que trace a
//! moldura com `stroke_rounded_rect` directo continua a compilar e a passar a suíte inteira — e
//! desenha um contorno num tema que prometeu não ter nenhum. É a mesma família do
//! `every_form_row_reserves_the_animation_column`: *uma promessa minha de «faço os restantes a
//! seguir» não sobrevive a uma janela de contexto; isto sobrevive.*
//!
//! # A régua
//!
//! Todo `.rs` de produto em `src/widget/` e `src/screens/hero/` que chame `stroke_rounded_rect(`
//! tem de **conhecer a porta** — chamar `stroke_frame(` ou consultar `visuals::` — ou estar numa
//! das duas listas abaixo, cada uma com a metade de obsolescência que impede a lista de virar
//! licença (`CLAUDE.md` §5.0).

use std::fs;
use std::path::{Path, PathBuf};

/// ⏳ **Dívida MEDIDA, e só encolhe.** Cada ficheiro aqui traça molduras sem perguntar ao tema —
/// num tema moderno ele ainda desenha o contorno que a pele plana apagou nos vizinhos.
///
/// ⚠️ **Não acrescente entradas.** Um pintor novo nasce com a porta; é uma chamada.
const NOT_YET: &[&str] = &[
    "screens/hero/asset_drag_ghost.rs",
    "screens/hero/chrome/input_map.rs",
    "screens/hero/topbar/mod.rs",
    "widget/avatar.rs",
    "widget/blender_color_picker/harmony.rs",
    "widget/blender_color_picker/hex_field.rs",
    "widget/blender_color_picker/paint.rs",
    "widget/blender_color_picker/value_slider.rs",
    "widget/color_picker.rs",
    "widget/combobox.rs",
    "widget/context_menu.rs",
    "widget/key_value_list.rs",
    "widget/radial_menu.rs",
    "widget/radio_group.rs",
    "widget/rect2_editor.rs",
    "widget/showcase/body.rs",
    "widget/slider.rs",
    "widget/slider_with_chip/number_chip.rs",
    "widget/status_bar.rs",
    "widget/tree_view.rs",
    "screens/hero/chrome/fill_modal.rs",
    "screens/hero/chrome/onion_modal.rs",
    "screens/hero/topbar/cluster_painter.rs",
];

/// ⛔ **Isentos POR MECANISMO, cada um com o motivo** — e o motivo é sobre o que o ficheiro É,
/// não sobre uma contagem que envelhece.
const EXEMPT: &[(&str, &str)] = &[
    (
        "paint.rs",
        "e' a PORTA: `stroke_frame` chama `stroke_rounded_rect` por definicao.",
    ),
    (
        "widget/toggle_classic.rs",
        "pintor SO' do classico (recuperado do git para o `PH2D_UI_NEW=0`); nunca corre num tema moderno.",
    ),
    (
        "widget/skin.rs",
        "a PELE de um DOCUMENTO (vetor autorado), nao cromo do editor — o raio e a moldura sao do artista.",
    ),
    (
        "screens/hero/selection.rs",
        "o marquee de seleccao sobre o CANVAS: um contorno e' o proprio significado, nao decoracao.",
    ),
    (
        "screens/hero/canvas.rs",
        "a moldura de «largue aqui» sobre o CANVAS durante um arrasto: o contorno E' a mensagem.",
    ),
    (
        "screens/hero/slot_tabs.rs",
        "o unico traco e' o contorno da aba DESTINO enquanto se arrasta outra: e' a mensagem, nao moldura de repouso.",
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

/// `(caminho relativo a src/, traça molduras?, conhece a porta?)` para todo ficheiro de produto.
fn census() -> Vec<(String, bool, bool)> {
    let root = src_root();
    let mut files = Vec::new();
    for top in ["widget", "screens/hero"] {
        walk(&root.join(top), &mut files);
    }
    files.push(root.join("paint.rs"));
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
        // ⚠️ Só o que PINTA: um `use` que nomeia a função não traça nada.
        let body: String = src
            .lines()
            .filter(|l| !l.trim_start().starts_with("use "))
            .collect::<Vec<_>>()
            .join("\n");
        let strokes = body.contains("stroke_rounded_rect(");
        let door = body.contains("stroke_frame(") || body.contains("visuals::");
        out.push((rel, strokes, door));
    }
    out.sort();
    out
}

/// ⭐⭐⭐ **CENSO: quem traça molduras conhece a porta — ou está na dívida declarada.**
///
/// **Mutação que deve sangrar:** trocar o `stroke_frame` de `widget/card.rs` de volta por
/// `stroke_rounded_rect` — o cartão volta a ter contorno num tema moderno, e este gate acusa-o.
#[test]
fn every_frame_goes_through_the_theme_door() {
    let stray: Vec<String> = census()
        .into_iter()
        .filter(|(rel, strokes, door)| {
            *strokes
                && !*door
                && !NOT_YET.contains(&rel.as_str())
                && !EXEMPT.iter().any(|(e, _)| e == rel)
        })
        .map(|(rel, ..)| rel)
        .collect();
    assert!(
        stray.is_empty(),
        "estes ficheiros tracam molduras com `stroke_rounded_rect` directo e nao perguntam ao \
         tema — num tema moderno eles desenham o contorno que a pele plana apagou: {stray:?}\n\
         cura: `crate::paint::stroke_frame(scene, rect, radius, theme, feel, w, colour)` (e o raio \
         por `crate::paint::frame_radius`)."
    );
}

/// ⛔ **A metade que impede as listas de virar licença.**
#[test]
fn the_debt_and_the_exemptions_have_no_stale_entries() {
    let all = census();
    let by_name = |rel: &str| all.iter().find(|(r, ..)| r == rel);
    // Uma entrada da dívida que já conhece a porta, ou que já não traça nada, sai.
    let done: Vec<&&str> = NOT_YET
        .iter()
        .filter(|rel| match by_name(rel) {
            Some((_, strokes, door)) => !*strokes || *door,
            None => true,
        })
        .collect();
    assert!(
        done.is_empty(),
        "estas entradas ja' nao descrevem nada — o ficheiro ja' passa pela porta, ja' nao traca, \
         ou nao existe: {done:?}\ncura: apague-as de NOT_YET. Uma catraca sem censo de \
         obsolescencia nao desce, vira licenca."
    );
    // E uma isenção tem de descrever um ficheiro que EXISTE e que TRAÇA — senão protege o nada.
    let ghosts: Vec<&&str> = EXEMPT
        .iter()
        .map(|(e, _)| e)
        .filter(|rel| !matches!(by_name(rel), Some((_, true, _))))
        .collect();
    assert!(
        ghosts.is_empty(),
        "isencoes sobre ficheiros que nao existem ou ja' nao tracam: {ghosts:?}"
    );
}
