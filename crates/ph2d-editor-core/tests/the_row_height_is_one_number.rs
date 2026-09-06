//! ⭐⭐⭐ **A ALTURA DE UMA LINHA é UM número — e quem quiser outro diz porquê.**
//!
//! O dono pediu painéis mais compactos e o `chrome.row-h` desceu de **28 para 24 px**
//! (2026-09-06). A mudança foi um número no `tokens.json`… e deixou **quatro cópias para trás**:
//! duas no picker de cor (`TOGGLE_H`, `HEX_ROW_H`), uma no rig de impasto (cujo doc já dizia
//! *«sized to the row height»*) e uma no editor de áudio — todas `28.0` escrito à mão. Elas
//! passaram por cima do pedido sem que um único teste ficasse vermelho: *o app teria linhas de
//! duas alturas, e o defeito só se vê a olho.*
//!
//! ⇒ toda constante cujo NOME diz «altura de linha» ou **deriva do token**, ou está declarada
//! aqui com o que ela é. As duas famílias existem e são legítimas:
//!
//! - **a linha de PAINEL** (`ROW_H_PX`): o formulário, o transporte, a linha do picker;
//! - **uma lista DENSA ou a geometria de um canvas**, que tem a sua própria régua (22 px numa
//!   lista de âncoras, a linha de socket do grafo que escala com o zoom, o alvo de 44 px de uma
//!   barra de progresso).
//!
//! ⚠️ **A régua separa PALAVRAS, não subcadeias** — a primeira versão deste censo, escrita a
//! `grep`, acusou `DUR_ARROW_HALF_W` e `NARROW_HALF` três vezes, porque `ROW_H` vive dentro de
//! `ARROW_H`. *Um censo que parseia o fonte tem de saber a forma do que lê.*

use std::fs;
use std::path::{Path, PathBuf};

/// `(caminho relativo à raiz do repo, nome da constante, porquê NÃO é o token)`.
const OWN_RULER: &[(&str, &str, &str)] = &[
    (
        "crates/ph2d-editor-core/src/progress.rs",
        "ROW_H",
        "a barra de progresso e' um ALVO DE TOQUE (44 px), nao uma linha de formulario",
    ),
    (
        "crates/ph2d-editor-core/src/grid_snap/inspect.rs",
        "ROW_H",
        "lista densa do inspector da grade",
    ),
    (
        "crates/ph2d-editor-core/src/widget/blender_color_picker/paint.rs",
        "SLIDER_ROW_H",
        "a linha de um slider de canal do picker e' mais baixa que a de formulario, de proposito",
    ),
    (
        "crates/ph2d-panel-motion-graph/src/geom.rs",
        "ROW_H",
        "a linha de SOCKET de um cartao do grafo: geometria de canvas, escala com o zoom",
    ),
    (
        "crates/ph2d-panel-motion-graph/src/geom.rs",
        "MENU_ROW_H",
        "a linha do menu de adicionar no canvas do grafo: idem",
    ),
    (
        "crates/ph2d-panel-audio-editor/src/paint_variation.rs",
        "VAR_ROW_H",
        "lista densa de variacoes",
    ),
    (
        "crates/ph2d-panel-inspector/src/sections/anchors.rs",
        "ROW_H",
        "lista densa de ancoras",
    ),
    (
        "crates/ph2d-panel-inspector/src/sections/anim_rows.rs",
        "ROW_H",
        "lista densa de animacoes",
    ),
    (
        "shells/desktop/src/field3d_view_menu.rs",
        "ROW_H_PX",
        "a linha de um menu FLUTUANTE sobre a vista 3D (26 px), nao uma linha de painel",
    ),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// O nome fala de altura de linha? ⚠️ **Por PALAVRAS**: `VAR_ROW_H` sim, `DUR_ARROW_HALF_W` não.
fn names_a_row_height(name: &str) -> bool {
    let parts: Vec<&str> = name.split('_').collect();
    parts.windows(2).any(|w| w == ["ROW", "H"])
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            walk(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// As raízes de UI: as crates de widget/painel e a shell.
fn ui_sources() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    walk(&root.join("crates/ph2d-editor-core/src"), &mut out);
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut out);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut out);
    out.sort();
    out
}

/// `(caminho relativo, nome, é literal?)` para toda constante de altura de linha.
fn row_height_constants() -> Vec<(String, String, bool)> {
    let root = repo_root();
    let mut out = Vec::new();
    for p in ui_sources() {
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        if rel.contains("/tests/") || rel.ends_with("_tests.rs") || rel.ends_with("tests.rs") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let mut in_tests = false;
        for line in src.lines() {
            if line.trim_start().starts_with("#[cfg(test)]") {
                in_tests = true;
            }
            if in_tests {
                continue;
            }
            let t = line.trim_start();
            let Some(rest) = t.strip_prefix("const ").or_else(|| {
                t.strip_prefix("pub const ")
                    .or_else(|| t.strip_prefix("pub(crate) const "))
                    .or_else(|| t.strip_prefix("pub(super) const "))
            }) else {
                continue;
            };
            let Some((name, value)) = rest.split_once(": f32 = ") else {
                continue;
            };
            if !names_a_row_height(name) {
                continue;
            }
            let literal = value.trim_start().starts_with(|c: char| c.is_ascii_digit());
            out.push((rel.clone(), name.to_string(), literal));
        }
    }
    out
}

/// ⭐⭐⭐ **CENSO: uma altura de linha é o token, ou está declarada com a régua dela.**
///
/// **Mutação que deve sangrar:** pôr `pub const HEX_ROW_H: f32 = 28.0;` de volta no
/// `blender_color_picker/paint.rs` — é exactamente o estado em que a mudança do dono deixou o app.
#[test]
fn every_row_height_is_the_token_or_declares_its_own_ruler() {
    let found = row_height_constants();
    assert!(
        found.len() >= 10,
        "so' {} constantes de altura de linha lidas — o parser deixou de reconhecer a forma \
         (`const NOME: f32 = …`)",
        found.len()
    );
    let stray: Vec<String> = found
        .iter()
        .filter(|(file, name, literal)| {
            *literal && !OWN_RULER.iter().any(|(f, n, _)| f == file && n == name)
        })
        .map(|(file, name, _)| format!("{file}: {name}"))
        .collect();
    assert!(
        stray.is_empty(),
        "estas constantes dizem no NOME que sao altura de linha e carregam um LITERAL — se forem \
         a linha de painel, elas divergem no dia em que o dono pedir outra densidade (foi o que \
         aconteceu em 2026-09-06, quatro vezes): {stray:#?}\n\
         cura: `= ph2d_tokens::ROW_H_PX`, ou uma entrada em OWN_RULER com a regua propria."
    );
}

/// ⛔ **A metade que impede a lista de virar licença:** uma declaração que já não descreve nada
/// sai (a constante deixou de existir, mudou de nome, ou passou a derivar do token).
#[test]
fn the_own_ruler_list_has_no_stale_entries() {
    let found = row_height_constants();
    let stale: Vec<String> = OWN_RULER
        .iter()
        .filter(|(f, n, _)| {
            !found
                .iter()
                .any(|(file, name, literal)| file == f && name == n && *literal)
        })
        .map(|(f, n, _)| format!("{f}: {n}"))
        .collect();
    assert!(
        stale.is_empty(),
        "estas entradas ja' nao descrevem uma constante literal de altura de linha: {stale:#?}"
    );
}
