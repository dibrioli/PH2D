//! **B42.4 — o ritmo da sim tem UMA fonte** (`ph2d_wet_paint::sim::SIM_HZ`). Censo: fora dela, nenhuma
//! fonte do motor nem da ferramenta escreve o 40 Hz como literal de divisão (`1.0 / 40.0`, `… / 40.0` de
//! tempo). Uma cópia que fica para trás quando o Hz muda é câmera lenta silenciosa (IN-111).

use std::path::Path;

/// Os padrões que escrevem o passo da sim à mão.
const COPIAS: [&str; 2] = ["1.0 / 40.0", "as f32 / 40.0"];

fn varre(dir: &Path, achados: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).expect("ler a pasta") {
        let p = e.expect("entrada").path();
        if p.is_dir() {
            varre(&p, achados);
        } else if de_produto(&p) {
            let s = std::fs::read_to_string(&p).expect("ler a fonte");
            for (i, l) in s.lines().enumerate() {
                let codigo = l.split("//").next().unwrap_or("");
                if COPIAS.iter().any(|c| codigo.contains(c)) {
                    achados.push(format!("{}:{}", p.display(), i + 1));
                }
            }
        }
    }
}

/// Fonte de produto: `.rs` que não é teste, sonda nem diagnóstico (esses fixam o seu `dt` de propósito).
fn de_produto(p: &Path) -> bool {
    let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
    nome.ends_with(".rs")
        && nome != "tests.rs"
        && !nome.ends_with("_tests.rs")
        && !nome.starts_with("diag_")
        && !nome.starts_with("measure_")
        && !nome.starts_with("sonda_")
        && !nome.ends_with("_probe.rs")
}

#[test]
fn um_so_sim_hz() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut achados = Vec::new();
    for c in ["ph2d-wet-paint/src", "ph2d-tool-painter/src"] {
        varre(&raiz.join(c), &mut achados);
    }
    assert!(
        achados.is_empty(),
        "o 40 Hz escrito à mão (use ph2d_wet_paint::sim::SIM_HZ): {achados:#?}"
    );
}

/// Controlo: o censo acha a forma que procura.
#[test]
fn o_censo_do_sim_hz_ve_a_copia() {
    let codigo = "pub(super) const STEP_S: f64 = 1.0 / 40.0;";
    assert!(COPIAS.iter().any(|c| codigo.contains(c)));
}

/// **B42.4 — o tecto do Spread (48 px) tem UMA fonte** (`BRUSH_SPREAD_MAX_PX`): o molhado, o seco, o setter
/// e a caixa do preview dissolvem à mesma distância (IN-112). Censo nas fontes de produto da ferramenta.
#[test]
fn um_so_tecto_do_spread() {
    let mut achados = Vec::new();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    varre_spread(&dir, &mut achados);
    assert!(achados.is_empty(), "o Spread com 48 à mão: {achados:#?}");
}

fn varre_spread(dir: &Path, achados: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).expect("ler a pasta") {
        let p = e.expect("entrada").path();
        if p.is_dir() {
            varre_spread(&p, achados);
        } else if de_produto(&p) {
            let s = std::fs::read_to_string(&p).expect("ler a fonte");
            for (i, l) in s.lines().enumerate() {
                let codigo = l.split("//").next().unwrap_or("");
                if codigo.contains("edge_spread") && codigo.contains("48.0") {
                    achados.push(format!("{}:{}", p.display(), i + 1));
                }
            }
        }
    }
}
