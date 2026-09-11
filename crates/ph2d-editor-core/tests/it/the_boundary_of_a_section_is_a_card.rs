//! ⭐⭐⭐ **A fronteira de uma secção é a BORDA DE UM CARTÃO — e o risco azul não volta.**
//!
//! Enio, 2026-09-06, com o Blender ao lado: *«vamos eliminar os nossos divisores azuis»*, e a
//! razão que ele deu para preferir o modelo deles — *«uma secção está dentro de um card»*.
//!
//! ⚠️ **Porque isto precisa de um censo e não bastava apagar as chamadas:** o
//! [`paint_section_separator`] continua a existir e **tem de continuar**, porque é ele que o tema
//! CLÁSSICO desenha (`PH2D_UI_NEW=0`). A função viva é a tentação: o próximo painel que quiser uma
//! fronteira vai encontrá-la antes de encontrar a porta, exactamente como aconteceu com a moldura
//! (waves 2–5) e com o vão de uma linha (wave 8). *Uma porta só é a única enquanto ninguém puder
//! chamar a antiga.*
//!
//! ⇒ **um sítio só** pode nomear o risco: o braço clássico de
//! [`widget::section_cards::close_section`].

use std::fs;
use std::path::{Path, PathBuf};

/// O ÚNICO ficheiro autorizado a nomear o risco — o braço clássico da porta.
const THE_CLASSIC_ARM: &str = "widget/section_cards/mod.rs";

/// E o ficheiro onde o risco VIVE (a definição dele).
const WHERE_IT_LIVES: &str = "widget/showcase/mod.rs";

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

fn production_sources() -> Vec<(String, String)> {
    let root = repo_root();
    let mut files = Vec::new();
    walk(&root.join("crates"), &mut files);
    walk(&root.join("shells"), &mut files);
    files
        .into_iter()
        .filter(|p| !p.components().any(|c| c.as_os_str() == "tests"))
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

/// ⭐ **Ninguém pinta um risco entre secções — a fronteira é o cartão.**
///
/// *Mutação que deve sangrar:* pôr de volta um `paint_section_separator(...)` em qualquer painel.
#[test]
fn no_panel_paints_the_blue_rule_between_sections() {
    let mut offenders = Vec::new();
    for (rel, src) in production_sources() {
        if rel.ends_with(THE_CLASSIC_ARM) || rel.ends_with(WHERE_IT_LIVES) {
            continue;
        }
        for (n, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if line.contains("paint_section_separator") {
                offenders.push(format!("{rel}:{}: {}", n + 1, t));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{} sítio(s) voltaram a pintar o risco entre secções. A porta é \
         `widget::section_cards::close_section` — ela desenha o cartão no tema moderno e o risco \
         no clássico, e é isso que mantém o `PH2D_UI_NEW=0` intacto:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// ⚠️ **E o controlo POSITIVO: o braço clássico tem de continuar a existir.**
///
/// Sem ele este censo passaria a verde no dia em que alguém apagasse o risco por inteiro — e aí
/// `PH2D_UI_NEW=0` deixaria de ter fronteiras, que é um defeito de produto que nenhum outro gate
/// desta linha vê. *Um censo de ausência precisa de uma testemunha de presença.*
#[test]
fn the_classic_arm_still_draws_the_rule() {
    let mut arm = None;
    let mut home = None;
    for (rel, src) in production_sources() {
        if rel.ends_with(THE_CLASSIC_ARM) {
            arm = Some(src);
        } else if rel.ends_with(WHERE_IT_LIVES) {
            home = Some(src);
        }
    }
    let arm = arm.unwrap_or_else(|| panic!("{THE_CLASSIC_ARM} desapareceu — o censo ficou cego"));
    let home = home.unwrap_or_else(|| panic!("{WHERE_IT_LIVES} desapareceu — o risco não vive lá"));
    assert!(
        home.contains("pub fn paint_section_separator("),
        "o risco tem de continuar a existir: é o que o tema CLÁSSICO desenha"
    );
    // ⚠️⚠️ **A busca ignora COMENTÁRIOS, e não é zelo — é a cura de uma vacuidade que a mutação
    //    apanhou** (2026-09-06): a 1.ª redacção fazia `arm.contains(...)` sobre o ficheiro
    //    inteiro, e o doc do próprio módulo cita a chamada antiga em prosa. Apagar o braço
    //    clássico do CÓDIGO deixava este controlo verde, porque ele estava a ler uma frase.
    //    *Um censo de presença que lê comentários testemunha a documentação, não o produto.*
    let arm_calls_it = arm
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .any(|l| l.contains("paint_section_separator(scene, theme, x, w, y)"));
    assert!(
        arm_calls_it,
        "o braço clássico da porta deixou de chamar o risco — `PH2D_UI_NEW=0` ficaria sem \
         fronteira entre secções"
    );
}
