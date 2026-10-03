//! ⭐⭐ **Quem tem um ponto da JANELA pergunta ao chrome pelas portas FÍSICAS** (2026-10-02, a escala
//! da interface inteira — `docs/UI_New_and_Simple/spec/05_a_escala_da_interface.md`).
//!
//! O chrome vive em píxeis LÓGICOS (janela / `s`); a shell e os módulos `ph2d-app-*` têm o ponteiro
//! e a câmera em FÍSICOS. ⛔ Um ponto físico no índice lógico acerta no VIZINHO a qualquer escala ≠
//! 100 % — e a 100 % (a fábrica, onde corre a suíte) os dois espaços são o mesmo, logo nenhum outro
//! gate o vê. ⇒ censo: fora do `ph2d-editor-core`, as perguntas cruas são proibidas; as portas são
//! `HeroScreen::{chrome_hit, chrome_panel_at, panel_rect_fisico, handle_pointer_fisico,
//! handle_wheel_fisico, escala}`.

use std::path::{Path, PathBuf};

const CRUAS: [&str; 6] = [
    "hit_index.hit(",
    ".store.panel_at(",
    ".store.panel_rect(",
    ".handle_pointer_with_text(",
    ".handle_pointer(",
    ".handle_wheel(",
];

fn fontes(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if nome != "tests" {
                fontes(&p, out);
            }
        } else if nome.ends_with(".rs") && !nome.ends_with("_tests.rs") {
            out.push(p);
        }
    }
}

#[test]
fn a_shell_pergunta_ao_chrome_pelas_portas_fisicas() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut ficheiros = Vec::new();
    fontes(&raiz.join("shells/desktop/src"), &mut ficheiros);
    for c in std::fs::read_dir(raiz.join("crates"))
        .expect("crates/")
        .flatten()
    {
        let nome = c.file_name().to_string_lossy().into_owned();
        if nome.starts_with("ph2d-app-") || nome == "ph2d-viewport3d" {
            fontes(&c.path().join("src"), &mut ficheiros);
        }
    }
    assert!(
        ficheiros.len() > 100,
        "o censo não achou a shell — régua partida"
    );
    let mut fora = Vec::new();
    for f in &ficheiros {
        let texto = std::fs::read_to_string(f).unwrap_or_default();
        // Um `.hit_index` partido em linhas conta como cru também: colapsa o espaço, cola o `.`.
        let junto = texto.split_whitespace().collect::<Vec<_>>().join(" ");
        let junto = junto.replace(" .", ".");
        for c in CRUAS {
            let n = junto.matches(c).count();
            if n > 0 {
                fora.push(format!(
                    "{} — {n}× `{c}`",
                    f.strip_prefix(&raiz).unwrap_or(f).display()
                ));
            }
        }
    }
    assert!(
        fora.is_empty(),
        "perguntas CRUAS ao chrome com um ponto da janela ({}):\n  {}",
        fora.len(),
        fora.join("\n  ")
    );
}

/// ⭐ **HiDPI: o factor do ECRÃ chega ao hero ANTES da pintura que constrói o índice de hit**, e há
/// um só escritor. Sem ele, num ecrã de factor `2,0` o chrome sai a metade do tamanho (o factor da
/// porta fica em `1,0`) e nenhum gate do `ph2d-editor-core` o vê: lá o factor é escrito à mão.
#[test]
fn o_factor_do_ecra_chega_ao_hero_antes_da_pintura() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut ficheiros = Vec::new();
    fontes(&raiz.join("shells/desktop/src"), &mut ficheiros);
    let escritores: Vec<_> = ficheiros
        .iter()
        .filter(|f| {
            std::fs::read_to_string(f)
                .unwrap_or_default()
                .contains(".escala_do_ecra =")
        })
        .collect();
    assert_eq!(
        escritores.len(),
        1,
        "um só escritor do factor do ecrã: {escritores:?}"
    );
    let fase = std::fs::read_to_string(escritores[0]).unwrap_or_default();
    let escreve = fase.find(".escala_do_ecra =").expect("escritor");
    let pinta = fase
        .find("paint_hero_screen_na_escala(")
        .expect("o escritor é a fase que pinta o hero");
    assert!(escreve < pinta, "o factor tem de entrar ANTES da pintura");
    assert!(
        fase.contains(".scale().get()"),
        "o factor vem do host (o `scale_factor` do winit)"
    );
}
