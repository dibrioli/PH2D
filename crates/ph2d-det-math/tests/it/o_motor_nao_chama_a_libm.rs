//! Bug #58: as crates que decidem o byte do traço não chamam os transcendentais de `std`, que vão à libm do sistema.
//! A prova ao vivo é `scripts/prova-libm/prova.sh`; este censo é a catraca barata.

use std::fs;
use std::path::{Path, PathBuf};

const CRATES: [&str; 16] = [
    "ph2d-wet-paint",
    "ph2d-paint-tinta",
    "ph2d-pigment",
    "ph2d-painter-brush",
    "ph2d-tool-painter",
    "ph2d-painter-effects",
    "ph2d-color",
    "ph2d-pen-law",
    "ph2d-symmetry",
    "ph2d-arclen",
    "ph2d-affine",
    "ph2d-sfx-recipe",
    "ph2d-sfx-dsp",
    "ph2d-sfx-lab",
    "ph2d-sfx-explore",
    // B43.2: o premul `_in_linear`, a câmara, a lente e a escolha pelo clique.
    "ph2d-render",
];
const FUNCOES: [&str; 23] = [
    "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sin_cos", "sinh", "cosh", "tanh", "exp",
    "exp2", "exp_m1", "ln", "ln_1p", "log", "log2", "log10", "powf", "powi", "cbrt", "hypot",
];

fn e_de_teste(p: &Path) -> bool {
    p.components().any(|c| {
        let c = c.as_os_str().to_string_lossy();
        c == "tests" || c.starts_with("tests_") || c.ends_with("_tests")
    }) || p.file_stem().is_some_and(|s| {
        let s = s.to_string_lossy();
        s == "test"
            || s == "tests"
            || s.ends_with("_test")
            || s.ends_with("_tests")
            || s.contains("sonda")
            || s.ends_with("_probe")
            || s.starts_with("diag")
    })
}

fn colhe(d: &Path, r: &mut Vec<PathBuf>) {
    for e in fs::read_dir(d).expect("a pasta lê-se").flatten() {
        let p = e.path();
        if p.is_dir() {
            colhe(&p, r);
        } else if p.extension().is_some_and(|x| x == "rs") && !e_de_teste(&p) {
            r.push(p);
        }
    }
}

/// As chamadas proibidas de um fonte: o que vem antes de um `mod` de teste em linha, sem comentários.
fn proibidas(fonte: &str) -> Vec<String> {
    let linhas: Vec<&str> = fonte.lines().collect();
    let mut r = Vec::new();
    for (i, l) in linhas.iter().enumerate() {
        let seguinte = linhas.get(i + 1).map_or("", |s| s.trim());
        if l.trim() == "#[cfg(test)]" && seguinte.contains("mod ") && seguinte.ends_with('{') {
            break;
        }
        let codigo = l.split("//").next().unwrap_or("");
        for f in FUNCOES {
            let casa = codigo.contains(&format!(".{f}("))
                || [format!("f32::{f}"), format!("f64::{f}")].iter().any(|a| {
                    codigo.match_indices(a.as_str()).any(|(p, _)| {
                        !codigo[p + a.len()..]
                            .starts_with(|c: char| c == '_' || c.is_alphanumeric())
                    })
                });
            if casa {
                r.push(format!("{}: {}", i + 1, codigo.trim()));
            }
        }
    }
    r
}

#[test]
fn o_motor_nao_chama_a_libm_do_sistema() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut fontes = Vec::new();
    for c in CRATES {
        colhe(&raiz.join(c).join("src"), &mut fontes);
    }
    assert!(
        fontes.len() > 300,
        "o censo varreu só {} ficheiros",
        fontes.len()
    );
    let mut falhas = Vec::new();
    for p in &fontes {
        let fonte = fs::read_to_string(p).expect("o fonte lê-se");
        for x in proibidas(&fonte) {
            falhas.push(format!(
                "{}:{x}",
                p.strip_prefix(&raiz).unwrap_or(p).display()
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "use ph2d_det_math::DetMath (det_sin, det_powf…):\n{}",
        falhas.join("\n")
    );
}

#[test]
fn o_censo_ve_as_tres_formas_e_poupa_o_resto() {
    assert_eq!(proibidas("let a = x.sin();").len(), 1);
    assert_eq!(proibidas("it.map(f64::cbrt);").len(), 1);
    assert_eq!(proibidas("let a = f32::atan2(y, x);").len(), 1);
    assert!(proibidas("let a = x.det_sin(); // x.sin()").is_empty());
    assert_eq!(proibidas("let a = x.powi(3);").len(), 1);
    assert!(proibidas("let a = x.det_powi(3);").is_empty());
    assert!(proibidas("let a = f32::cosine(x); y.sqrt();").is_empty());
    assert!(proibidas("#[cfg(test)]\nmod t {\n x.sin();\n}").is_empty());
    assert_eq!(proibidas("#[cfg(test)]\nmod t;\nx.sin();").len(), 1);
}

/// B43.1: o `det_powi` dá o bit do `powi` CORRIDO (o `__powisf2`), o que o motor pinta hoje; o dobrado em compilação
/// (#65088: `pow` em `f64` arredondado) dá outro, e é por isso que o motor não chama o `powi`.
#[test]
fn det_powi_e_o_powi_corrido_ao_bit_e_o_dobrado_nao() {
    use ph2d_det_math::DetMath;
    use std::hint::black_box;
    let mut dobrado_difere = 0;
    for i in 0..20_000_u32 {
        let x = 0.37_f32 + i as f32 * 1.7e-4;
        for n in [-3, -1, 0, 1, 2, 3, 5, 7, 8, 13] {
            let corrido = black_box(x).powi(black_box(n));
            assert_eq!(x.det_powi(n).to_bits(), corrido.to_bits(), "f32 {x} ^ {n}");
            let xd = f64::from(x) * 1.3;
            assert_eq!(
                xd.det_powi(n).to_bits(),
                black_box(xd).powi(black_box(n)).to_bits()
            );
            dobrado_difere +=
                usize::from((f64::from(x).powi(n) as f32).to_bits() != corrido.to_bits());
        }
    }
    assert!(
        dobrado_difere > 1000,
        "o dobrado difere em {dobrado_difere}"
    );
}
