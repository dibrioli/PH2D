//! ⛔ **Toda colagem de cena em produção passa por `ph2d_vector::scene::colar_cena`.** O `append` do vello
//! apaga o pedido de re-emitir o transform do texto (ver a porta). Uma colagem crua é reprovada aqui,
//! por forma: `.append(a, b)` com dois argumentos, ou `::append(a, b, c)` com três. O `append` de `Vec`,
//! `VecDeque` e `BTreeMap` tem um argumento (`Vec::append(a, b)`, dois), logo nenhum deles casa.

use std::path::{Path, PathBuf};

/// Medidos em 05/11. Sobem com o projeto; só descem por poda real, medida.
const PISO_FICHEIROS: usize = 7_000;
const PISO_CHAMADAS_DA_PORTA: usize = 29;

const PORTA: &str = "crates/ph2d-vector/src/scene.rs";

/// O fonte sem comentários e sem o miolo de strings e chars, com as quebras de linha preservadas.
fn so_codigo(fonte: &str) -> String {
    let b = fonte.as_bytes();
    let mut out = String::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut nivel = 1;
            i += 2;
            while i < b.len() && nivel > 0 {
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    nivel += 1;
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    nivel -= 1;
                    i += 2;
                } else {
                    if b[i] == b'\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
        } else if c == b'r' && matches!(b.get(i + 1), Some(b'"' | b'#')) && !antes_e_ident(b, i) {
            let mut j = i + 1;
            while b.get(j) == Some(&b'#') {
                j += 1;
            }
            if b.get(j) != Some(&b'"') {
                out.push('r');
                i += 1;
                continue;
            }
            let fecho = format!("\"{}", "#".repeat(j - i - 1));
            let resto = &fonte[j + 1..];
            let n = resto.find(&fecho).map_or(resto.len(), |n| n + fecho.len());
            out.extend(resto[..n].chars().filter(|&ch| ch == '\n'));
            out.push_str("\"\"");
            i = j + 1 + n;
        } else if c == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\n' {
                    out.push('\n');
                }
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
            out.push_str("\"\"");
        } else if c == b'\'' {
            // Char tem fecho perto; tempo de vida não.
            let escapado = b.get(i + 1) == Some(&b'\\');
            let fim = (i + 2..b.len().min(i + 12)).find(|&j| b[j] == b'\'');
            match fim {
                Some(j)
                    if escapado || fonte.get(i + 1..j).is_some_and(|s| s.chars().count() == 1) =>
                {
                    out.push_str("' '");
                    i = j + 1;
                }
                _ => {
                    out.push('\'');
                    i += 1;
                }
            }
        } else {
            let tam = fonte[i..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&fonte[i..i + tam]);
            i += tam;
        }
    }
    out
}

fn antes_e_ident(b: &[u8], i: usize) -> bool {
    i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')
}

/// Quantos argumentos tem a chamada cujo parêntese abre em `abre`.
fn argumentos(codigo: &str, abre: usize) -> usize {
    let (mut nivel, mut virgulas, mut algo_depois) = (0_i32, 0, false);
    for &c in &codigo.as_bytes()[abre..] {
        match c {
            b'(' | b'[' | b'{' => {
                nivel += 1;
                if nivel > 1 {
                    algo_depois = true;
                }
            }
            b')' | b']' | b'}' => {
                nivel -= 1;
                if nivel == 0 {
                    break;
                }
                algo_depois = true;
            }
            b',' if nivel == 1 => {
                virgulas += 1;
                algo_depois = false;
            }
            c if !c.is_ascii_whitespace() => algo_depois = true,
            _ => {}
        }
    }
    virgulas + usize::from(algo_depois)
}

/// As linhas das colagens cruas num fonte.
fn cruas(fonte: &str) -> Vec<usize> {
    let codigo = so_codigo(fonte);
    let mut linhas = Vec::new();
    for (i, _) in codigo.match_indices("append(") {
        let (antes, abre) = (&codigo[..i], i + "append".len());
        let n = argumentos(&codigo, abre);
        let antes = antes.trim_end();
        if (antes.ends_with('.') && n == 2) || (antes.ends_with("::") && n == 3) {
            linhas.push(codigo[..i].matches('\n').count() + 1);
        }
    }
    linhas
}

fn fontes(pasta: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(pasta).expect("pasta legível").flatten() {
        let p = e.path();
        let nome = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !matches!(
                nome.as_str(),
                "target" | "vendor" | "tests" | "benches" | "examples"
            ) {
                fontes(&p, out);
            }
        } else if nome.ends_with(".rs")
            && nome != "tests.rs"
            && !nome.ends_with("_tests.rs")
            && !nome.ends_with("_test.rs")
        {
            out.push(p);
        }
    }
}

#[test]
fn nenhuma_colagem_de_cena_fora_da_porta() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut ficheiros = Vec::new();
    fontes(&raiz.join("crates"), &mut ficheiros);
    fontes(&raiz.join("shells"), &mut ficheiros);
    let porta = raiz.join(PORTA);
    let (mut fora, mut na_porta, mut pela_porta) = (Vec::new(), 0, 0);
    for f in &ficheiros {
        let Ok(fonte) = std::fs::read_to_string(f) else {
            continue;
        };
        let linhas = cruas(&fonte);
        if *f == porta {
            na_porta = linhas.len();
        } else {
            fora.extend(linhas.iter().map(|l| format!("{}:{l}", f.display())));
            let codigo = so_codigo(&fonte);
            pela_porta += [".append_transformed(", ".append_scene(", "colar_cena("]
                .iter()
                .map(|p| codigo.matches(p).count())
                .sum::<usize>();
        }
    }
    assert!(
        fora.is_empty(),
        "colagem crua, use a porta `colar_cena`: {fora:#?}"
    );
    assert_eq!(na_porta, 1, "a porta cola uma vez, em `colar_cena`");
    let n = ficheiros.len();
    assert!(
        n >= PISO_FICHEIROS && pela_porta >= PISO_CHAMADAS_DA_PORTA,
        "o censo perdeu população: {n} ficheiros, {pela_porta} chamadas da porta"
    );
}

/// A régua vê a forma crua em todas as grafias, e só ela.
#[test]
fn o_censo_ve_a_colagem_crua_e_mais_nada() {
    let casos = [
        ("s.inner_mut().append(p.inner(), None);", 1),
        (
            "s.inner_mut()\n    .append(\n        p.inner(),\n        Some(f(a, b)),\n    );",
            1,
        ),
        ("composta.append(cena, None);", 1),
        ("vello::Scene::append(&mut a, &b, None);", 1),
        ("out.append(&mut outro);", 0),
        ("Vec::append(&mut a, &mut b);", 0),
        ("// s.append(p, None);", 0),
        ("/* s.append(p, None); */", 0),
        ("let t = \"s.append(p, None);\";", 0),
        ("let t = r#\"s.append(p, None);\"#;", 0),
        ("s.append_transformed(&p, t);", 0),
    ];
    for (fonte, n) in casos {
        assert_eq!(cruas(fonte).len(), n, "{fonte}");
    }
}
