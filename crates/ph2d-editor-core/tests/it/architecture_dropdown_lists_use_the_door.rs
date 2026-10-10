//! ⭐⭐⭐ **UMA LISTA DE DROPDOWN QUE ROLA PASSA PELA PORTA** (`panel::paint_dropdown_list`).
//!
//! ⛔ Medido em 2026-10-10: **catorze** cópias da mesma lei em cinco painéis — clamp, publicação,
//! rolagem gravada de volta, pintura, hits pela faixa visível, trilha da barra —, cada uma
//! «espelho exacto» de outra. Duas já tinham divergido: os pickers de ícone e de fonte do Vector
//! perguntavam pelo polegar sem dono e pintavam-no em repouso enquanto o dedo o arrastava.
//!
//! O censo lê o FONTE dos painéis (`crates/ph2d-panel-*/src`) e reprova uma linha de CÓDIGO (não
//! de comentário) que remonte a lei à mão: o id da barra partilhada, a geometria de uma linha, ou o
//! rect do popover. Os três pintores da lista aberta já são `pub(crate)` no `widget` — o
//! compilador barra-os; a agulha do que rola fica para o dia em que alguém o reabrir.

use std::fs;
use std::path::{Path, PathBuf};

/// As agulhas — cada uma é um pedaço da lei que só a porta pode escrever.
const AGULHAS: [&str; 6] = [
    "DROPDOWN_SCROLLBAR_ID",
    "option_rect_in_scrolled(",
    "option_rect_in(",
    "popover_rect_clamped(",
    "set_dropdown_popover(",
    "paint_dropdown_popover_scrolled(",
];

/// A chamada da porta — a população que prova que a varredura alcança os painéis.
const PORTA: &str = "paint_dropdown_list(";

/// ⛔ **Piso de população** (`CLAUDE.md` §5.0: uma varredura que deixe de alcançar as crates
/// devolve zero acusados e lê-se como aprovada). Medido 2026-10-10: **19** ficheiros de painel
/// chamam a porta — os catorze que rolavam por cópia, os dois que já usavam o núcleo (o filtro da
/// Hierarquia e as fontes da Timeline) e os três que nem rolavam (Color Equalization, a barra do
/// Flip Frames, a animação da Timeline). O Inspector fica no núcleo: o passe dele não tem o store
/// mutável.
const PISO_PORTA: usize = 19;

fn raiz_das_crates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn percorre(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            percorre(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn ficheiros_dos_paineis() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entradas) = fs::read_dir(raiz_das_crates()) else {
        return out;
    };
    for e in entradas.flatten() {
        let p = e.path();
        let painel = p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("ph2d-panel-"));
        if painel && p.join("src").is_dir() {
            percorre(&p.join("src"), &mut out);
        }
    }
    out.sort();
    out
}

/// O CÓDIGO de uma linha — sem o comentário de fim de linha, e vazio numa linha de comentário.
fn codigo(linha: &str) -> &str {
    let t = linha.trim_start();
    if t.starts_with("//") {
        return "";
    }
    t.find("//").map_or(t, |i| &t[..i])
}

/// A agulha que esta linha de código contém, se alguma.
fn acusa(linha: &str) -> Option<&'static str> {
    let c = codigo(linha);
    AGULHAS.iter().copied().find(|a| c.contains(a))
}

/// ⭐⭐⭐ **NENHUM PAINEL REMONTA A LISTA QUE ROLA À MÃO.**
///
/// **Mutação que deve sangrar:** devolver qualquer um dos catorze popovers ao corpo antigo (o
/// `paint_dropdown_popover_scrolled` + o laço do `option_rect_in_scrolled` + a barra).
#[test]
fn as_listas_que_rolam_passam_pela_porta() {
    let raiz = raiz_das_crates();
    let mut chamam_a_porta = 0;
    let mut acusados = Vec::new();
    for p in ficheiros_dos_paineis() {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        if src.lines().any(|l| codigo(l).contains(PORTA)) {
            chamam_a_porta += 1;
        }
        for (n, l) in src.lines().enumerate() {
            if let Some(a) = acusa(l) {
                let rel = p.strip_prefix(&raiz).unwrap_or(&p).display().to_string();
                acusados.push(format!("{rel}:{} — `{a}`", n + 1));
            }
        }
    }
    assert!(
        chamam_a_porta >= PISO_PORTA,
        "a varredura alcançou só {chamam_a_porta} ficheiros que chamam a porta (piso {PISO_PORTA}) \
         — ou ela deixou de ver os painéis, ou um painel saiu da porta"
    );
    assert!(
        acusados.is_empty(),
        "um painel remonta à mão a lista de dropdown que ROLA — use \
         `ph2d_editor_core::panel::paint_dropdown_list` (com um pintor de linha se a linha desenha \
         outra coisa que o rótulo):\n{}",
        acusados.join("\n")
    );
}

/// CONTROLO do detector: acusa o código, poupa o comentário e o núcleo da porta.
#[test]
fn o_detector_acusa_o_codigo_e_poupa_o_comentario() {
    assert_eq!(
        acusa("        hit_index.register(DROPDOWN_SCROLLBAR_ID, track);"),
        Some("DROPDOWN_SCROLLBAR_ID")
    );
    assert_eq!(
        acusa("let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);"),
        Some("option_rect_in_scrolled(")
    );
    assert_eq!(
        acusa("    paint_dropdown_popover_scrolled(&dd, chip, panel, s, v, sc, t, th);"),
        Some("paint_dropdown_popover_scrolled(")
    );
    assert_eq!(
        acusa("let panel = dd.popover_rect_clamped(chip, viewport);"),
        Some("popover_rect_clamped(")
    );
    assert_eq!(
        acusa("store.set_dropdown_popover(id, panel);"),
        Some("set_dropdown_popover(")
    );
    assert_eq!(
        acusa("/// a barra `DROPDOWN_SCROLLBAR_ID` é do despacho"),
        None
    );
    assert_eq!(acusa("let x = 1; // DROPDOWN_SCROLLBAR_ID"), None);
    assert_eq!(
        acusa("let p = paint_dropdown_popover_scrolled_with_hits(&dd, c, r, s, sc, t, th, h);"),
        None
    );
}
