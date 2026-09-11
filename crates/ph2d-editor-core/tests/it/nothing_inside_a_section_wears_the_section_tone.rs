//! ⛔⛔ **O `Bg1` é o tom de um CARTÃO DE SECÇÃO — nada pintado DENTRO de um o pode usar.**
//!
//! Report do dono, 2026-09-06, **duas vezes**: *«o card de Painter: Jitter não se vê mais»* e, no
//! dia seguinte à primeira cura, *«não apareceu»*.
//!
//! # Porque a primeira cura não bastou
//!
//! O `Bg1` sempre foi o fundo de um cartão. Quando a wave 12 pôs um cartão de **SECÇÃO** por trás
//! do corpo de cada painel — também `Bg1` —, tudo o que já pintava `Bg1` lá dentro **desapareceu
//! dentro dele**: *dois tons iguais encostados são uma superfície só*.
//!
//! ⚠️ A primeira cura mexeu no `card.rs::card_frame`, e a `jitter_card.rs` **não passa por ele** —
//! ela pinta o próprio `Bg1`. *Uma lei escrita em dois sítios ainda não é uma lei; só uma PORTA é*,
//! e este app tinha **doze** cópias dela. Curar a que se conhece e declarar o assunto fechado é
//! como um report volta com «não apareceu».
//!
//! # A lei
//!
//! *Quem pinta uma superfície DENTRO de um corpo com cartões pede o tom à porta*
//! ([`ph2d_editor_core::widget::section_cards::CardDepth`]) — nunca ao token da secção.
//!
//! ⛔ **O que fica de fora é CANVAS, e está nomeado abaixo:** uma célula de tira de filme e a
//! camada de áudio sobre o desenho não vivem dentro de secção nenhuma, e ali o `Bg1` continua a
//! ser o que sempre foi.

use std::fs;
use std::path::{Path, PathBuf};

/// `(caminho relativo, porquê o `Bg1` continua certo ali)`.
const NOT_INSIDE_A_SECTION: &[(&str, &str)] = &[
    (
        "crates/ph2d-panel-flip-frames/src/paint_cells.rs",
        "a celula de uma tira de filme e' CANVAS: ela nao vive dentro de um corpo com cartoes",
    ),
    (
        "shells/desktop/src/render_loop/audio_overlay.rs",
        "a camada de audio pinta-se SOBRE o desenho, fora de todo painel",
    ),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                walk(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// As crates de PAINEL e a shell — quem pinta dentro de um corpo.
fn panel_sources() -> Vec<(String, String)> {
    let root = repo_root();
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut files);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut files);
    files
        .into_iter()
        // ⚠️ **Fora as duas formas de teste, e a segunda é a que morde:** uma pasta `tests/` e um
        // ficheiro `*_tests.rs` DENTRO do `src` — este repo usa as duas, e a 1.ª redacção só
        // conhecia a pasta. Ela acusou o `canvas_clear_tests.rs`, que nomeia o `Bg1` para o
        // *testar*. *Um censo que varre uma árvore tem de saber as formas em que ela guarda
        // testes.*
        .filter(|p| {
            !p.components().any(|c| c.as_os_str() == "tests")
                && !p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with("_tests.rs"))
        })
        .filter_map(|p| {
            let rel = p
                .strip_prefix(&root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            fs::read_to_string(&p).ok().map(|s| (rel, s))
        })
        .collect()
}

/// ⭐ **Ninguém pinta o tom da secção dentro dela.**
///
/// *Mutação que deve sangrar:* devolver `ColorToken::Bg1` a qualquer um dos dez sítios curados.
#[test]
fn no_panel_surface_wears_the_section_card_tone() {
    let exempt: Vec<&str> = NOT_INSIDE_A_SECTION.iter().map(|(p, _)| *p).collect();
    let mut offenders = Vec::new();
    for (rel, src) in panel_sources() {
        if exempt.iter().any(|e| rel.ends_with(e)) {
            continue;
        }
        for (n, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if line.contains("ColorToken::Bg1") {
                offenders.push(format!("{rel}:{}: {}", n + 1, t));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{} superficie(s) pintam o tom do CARTAO DE SECCAO dentro dele — elas desaparecem, que foi \
         o report do Painter/Jitter (duas vezes). O tom vem da porta \
         `section_cards::CardDepth::Subsection.token()`:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// ⚠️ **E a metade JUSTA: uma isenção que já não descreve nada tem de cair.**
///
/// *Uma catraca sem censo de obsolescência não desce: ela vira licença.*
#[test]
fn every_exemption_still_names_a_file_that_paints_it() {
    let root = repo_root();
    let mut stale = Vec::new();
    for (rel, why) in NOT_INSIDE_A_SECTION {
        let p = root.join(rel);
        match fs::read_to_string(&p) {
            Ok(s) if s.contains("ColorToken::Bg1") => {}
            Ok(_) => stale.push(format!(
                "{rel} ja' nao pinta `Bg1` — a isencao («{why}») expirou"
            )),
            Err(_) => stale.push(format!("{rel} nao existe — a isencao aponta para o vazio")),
        }
    }
    assert!(
        stale.is_empty(),
        "isencoes obsoletas:\n  {}",
        stale.join("\n  ")
    );
}
