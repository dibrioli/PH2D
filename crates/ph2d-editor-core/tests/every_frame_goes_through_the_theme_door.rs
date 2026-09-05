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
/// ⭐ **VAZIA desde 2026-09-05** (wave 3): os 22 ficheiros da wave 2 passaram pela porta — 20
/// convertidos, 2 isentos por mecanismo (abaixo). ⚠️ **Não acrescente entradas.** Um pintor novo
/// nasce com a porta; é uma chamada.
const NOT_YET: &[&str] = &[];

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
    (
        "widget/rect2_editor.rs",
        "as ALCAS de um gizmo sobre CONTEUDO (as pegas de mover/redimensionar um rect): o contorno e' o que as separa da imagem por baixo, como o marquee.",
    ),
    (
        "widget/showcase/body.rs",
        "o contorno de seccao do showcase e' a MARCA de realce que o utilizador escolheu (`Section outline`, cor de marcador): conteudo autorado, nao moldura de repouso.",
    ),
    // ── Painéis (wave 4) — o contorno que É a mensagem, ou conteúdo do documento ──
    (
        "ph2d-panel-flip-frames/src/paint_cells.rs",
        "o fantasma de onde a celula vai POUSAR durante um arrasto: a mensagem, nao moldura de repouso.",
    ),
    (
        "ph2d-panel-hierarchy/src/paint.rs",
        "o indicador de «largar DENTRO deste no'» durante um arrasto: a mensagem, nao moldura de repouso.",
    ),
    (
        "ph2d-panel-inspector/src/paint_frame.rs",
        "o contorno de seccao e' a MARCA de realce que o utilizador escolheu (cor de marcador): conteudo autorado — o gemeo do showcase.",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_overlays.rs",
        "o marquee de seleccao sobre o CANVAS do grafo: um contorno e' o proprio significado — o gemeo do `selection.rs`.",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_wire.rs",
        "o anel do crachá `pre` sobre um FIO do grafo: conteudo do documento, nao cromo.",
    ),
    (
        "ph2d-panel-motion-graph/src/paint_socket.rs",
        "o HALO de um socket-alvo durante um arrasto de fio (`Accent` compativel · `Danger` incompativel): a mensagem, nao moldura de repouso.",
    ),
    (
        "ph2d-panel-physics/src/paint/matrix.rs",
        "a diagonal da matriz de camadas e' contornada para o olho achar o eixo de simetria: a mensagem, nao moldura.",
    ),
    (
        "ph2d-panel-timeline/src/strip_paint.rs",
        "o contorno de uma STRIP na lane: duas strips adjacentes com a mesma tinta so' se separam por ele — conteudo do documento.",
    ),
];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// ⭐⭐ **Os PAINÉIS e a SHELL também traçam** (wave 4, 2026-09-05): medido, 59 ficheiros em
/// `crates/ph2d-panel-*/src` chamavam `stroke_rounded_rect` directo e **nenhum** conhecia a porta —
/// e é nos painéis que o artista vive. Cada raiz vem com o prefixo da crate, para que a chave de
/// uma isenção nunca colida com a de um ficheiro do `editor-core`.
fn panel_and_shell_roots() -> Vec<(String, PathBuf)> {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut out = Vec::new();
    for entry in fs::read_dir(&crates).expect("crates/ legivel").flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        if p.is_dir() && name.starts_with("ph2d-panel-") && p.join("src").is_dir() {
            out.push((format!("{name}/src"), p.join("src")));
        }
    }
    let shell = crates.join("../shells/desktop/src");
    if shell.is_dir() {
        out.push(("shells/desktop/src".to_string(), shell));
    }
    out.sort();
    out
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

/// `(traça molduras?, conhece a porta?)` de um ficheiro de produto.
fn classify(p: &Path) -> (bool, bool) {
    let src = fs::read_to_string(p).expect("ficheiro legivel");
    // ⚠️ Só o que PINTA: um `use` que nomeia a função não traça nada.
    let body: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n");
    let strokes = body.contains("stroke_rounded_rect(");
    let door = body.contains("stroke_frame(") || body.contains("visuals::");
    (strokes, door)
}

fn is_test_file(rel: &str) -> bool {
    rel.contains("/tests") || rel.ends_with("_tests.rs") || rel.ends_with("tests.rs")
}

/// `(chave, traça molduras?, conhece a porta?)` para todo ficheiro de produto — os do
/// `editor-core` com o caminho relativo a `src/` (`widget/…`, `screens/hero/…`, `paint.rs`), os
/// dos painéis e da shell com o prefixo da crate (`ph2d-panel-x/src/…`, `shells/desktop/src/…`).
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
        if is_test_file(&rel) {
            continue;
        }
        let (strokes, door) = classify(&p);
        out.push((rel, strokes, door));
    }
    for (prefix, src) in panel_and_shell_roots() {
        let mut files = Vec::new();
        walk(&src, &mut files);
        for p in files {
            let rel = p
                .strip_prefix(&src)
                .expect("dentro da raiz")
                .to_string_lossy()
                .replace('\\', "/");
            if is_test_file(&rel) {
                continue;
            }
            let (strokes, door) = classify(&p);
            out.push((format!("{prefix}/{rel}"), strokes, door));
        }
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
