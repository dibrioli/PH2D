//! B44.7 (GT-25, missão 4K N10): o censo ÚNICO das sondas. Cada família de mentira de sonda tem o seu
//! portão (S-01 raio, S-02 Gauss-Seidel, S-04 rota, S-05 sim, S-08 `--release`); este junta-os e acrescenta
//! os dois que faltavam: toda sonda (`#[ignore]` num ficheiro `measure_*`/`sonda_*`/`medida_*`/`mede_*`/`diag_*`)
//! DECLARA o que é (`#[ignore = "…"]`, B44.6) e, se lê o relógio, imprime o PERFIL (`perfil_da_sonda()`, S-10:
//! um número sem perfil ao lado vale zero). Controlo: o varredor acusa um pedaço mau feito à mão.

use std::path::Path;

/// `(ficheiro, portão)` de cada família já curada: apagar um portão cala a família inteira.
const PORTOES: &[(&str, &str)] = &[
    (
        "tests/it/as_sondas_passam_o_raio.rs",
        "fn as_sondas_passam_o_raio_ao_set_brush_size_px",
    ),
    (
        "src/tool/paint/measure_wetpaint_passes.rs",
        "fn as_sondas_dos_passes_medem_a_rota_do_produto",
    ),
    (
        "src/tool/paint/wetpaint_preset/sonda_oleo_furos.rs",
        "fn a_sonda_dos_furos_mede_a_rota_do_produto_por_omissao",
    ),
    (
        "src/tool/paint/measure_wetpaint_stamp.rs",
        "fn as_sondas_do_carimbo_com_a_sim_armam_a_agua_viva_e_contam_os_passos",
    ),
    (
        "tests/it/as_sondas_dizem_o_perfil.rs",
        "fn as_sondas_dizem_o_perfil_e_correm_em_smoke",
    ),
];

const PREFIXOS: [&str; 5] = ["measure_", "sonda_", "medida_", "mede_", "diag_"];

/// O corpo `{…}` que começa no primeiro `{` de `s` (chavetas contadas; as dos `format!` vêm aos pares).
fn corpo(s: &str) -> &str {
    let ini = s.find('{').unwrap_or(0);
    let mut fundo = 0i32;
    for (k, c) in s[ini..].char_indices() {
        match c {
            '{' => fundo += 1,
            '}' => {
                fundo -= 1;
                if fundo == 0 {
                    return &s[ini..=ini + k];
                }
            }
            _ => {}
        }
    }
    &s[ini..]
}

/// As sondas de um ficheiro: `(linha do #[ignore], nome, declarada, relógio sem perfil)`.
fn sondas(src: &str) -> Vec<(usize, String, bool, bool)> {
    let linhas: Vec<&str> = src.lines().collect();
    let mut fora = Vec::new();
    let mut i = 0;
    while i < linhas.len() {
        let t = linhas[i].trim_start();
        if !t.starts_with("#[ignore") {
            i += 1;
            continue;
        }
        let declarada = t.contains('=');
        let Some(j) = (i + 1..linhas.len()).find(|&j| {
            let l = linhas[j].trim_start();
            l.starts_with("fn ") || l.starts_with("pub fn ") || l.starts_with("async fn ")
        }) else {
            break;
        };
        let nome = linhas[j]
            .split("fn ")
            .nth(1)
            .and_then(|r| r.split(['(', '<']).next())
            .unwrap_or("?")
            .trim()
            .to_owned();
        let resto = linhas[j..].join("\n");
        let c = corpo(&resto);
        let relogio = c.contains("Instant::now") || c.contains(".elapsed()");
        fora.push((
            i + 1,
            nome,
            declarada,
            relogio && !c.contains("perfil_da_sonda"),
        ));
        i = j + 1;
    }
    fora
}

fn varre(dir: &Path, raiz: &Path, n: &mut usize, maus: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            varre(&p, raiz, n, maus);
            continue;
        }
        let nome = p.file_name().unwrap().to_string_lossy().into_owned();
        if !nome.ends_with(".rs") || !PREFIXOS.iter().any(|x| nome.starts_with(x)) {
            continue;
        }
        let rel = p
            .strip_prefix(raiz)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        for (linha, f, declarada, sem_perfil) in sondas(&std::fs::read_to_string(&p).unwrap()) {
            *n += 1;
            if !declarada {
                maus.push(format!(
                    "{rel}:{linha} {f}: `#[ignore]` sem dizer o que mede"
                ));
            }
            if sem_perfil {
                maus.push(format!(
                    "{rel}:{linha} {f}: lê o relógio e não chama perfil_da_sonda()"
                ));
            }
        }
    }
}

#[test]
fn as_sondas_dizem_o_que_medem() {
    // Controlo: o varredor acusa o pedaço mau e deixa passar o bom.
    let mau = "#[test]\n#[ignore]\nfn m() {\n    let t = Instant::now();\n    if x { y(format!(\"{}\", t.elapsed())) }\n}\n";
    let bom = "#[test]\n#[ignore = \"sonda\"]\nfn b() {\n    let _ = perfil_da_sonda();\n    let t = Instant::now();\n}\n";
    assert_eq!(sondas(mau), vec![(2, "m".to_owned(), false, true)]);
    assert_eq!(sondas(bom), vec![(2, "b".to_owned(), true, false)]);

    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut maus = Vec::new();
    for (f, portao) in PORTOES {
        let s = std::fs::read_to_string(raiz.join(f)).unwrap_or_default();
        if !s.contains(portao) {
            maus.push(format!("{f}: o portão `{portao}` sumiu"));
        }
    }
    let mut n = 0;
    for d in ["src", "tests", "examples"] {
        varre(&raiz.join(d), raiz, &mut n, &mut maus);
    }
    // 423 sondas no F6l: abaixo de 300 o varredor deixou de as ver (renomeação, outra pasta).
    assert!(
        n >= 300,
        "o censo viu só {n} sondas: deixou de ler o que diz ler"
    );
    assert!(
        maus.is_empty(),
        "{} sondas que não dizem o que medem:\n{}",
        maus.len(),
        maus.join("\n")
    );
}
