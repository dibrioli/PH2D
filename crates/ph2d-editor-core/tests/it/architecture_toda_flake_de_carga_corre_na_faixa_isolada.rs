//! ⛔ **Toda flake de carga corre na faixa isolada do nextest** — a lista do doc e o filtro do
//! `.config/nextest.toml` são a MESMA lista escrita em dois sítios, e este gate os ata.
//!
//! Medido em 2026-10-02: a família vivia no `CLAUDE.md §5.0` e promover um membro era escrever o
//! nome LÁ; a faixa (`threads-required = 'num-cpus'`, `priority = -100`) só foi preenchida uma vez,
//! em 10/09. Resultado: **oito** membros promovidos depois disso nunca entraram na faixa e
//! continuaram a reprovar portões sob fan-out — cada um, uma corrida de portão a mais.
//! A lista viva é `docs/DevOps/FLAKES_DE_CARGA.md`.
//!
//! Três metades:
//! 1. todo membro da tabela está no filtro da faixa;
//! 2. todo nome do filtro está na tabela ou na lista «na faixa por outra razão» (senão a faixa
//!    serializa testes que ninguém sabe porque estão lá);
//! 3. todo nome ainda é um `fn` na árvore — o censo de obsolescência (um nome morto no filtro não
//!    casa nada e lê-se como «está protegido»).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ler(rel: &str) -> String {
    let p = raiz().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("ler {}: {e}", p.display()))
}

/// Os nomes da tabela de membros (`| \`nome\` | …`) e os da lista «na faixa por outra razão»
/// (`- \`nome\` — …`).
fn nomes_do_doc(doc: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let ident = |s: &str| -> Option<String> {
        let s = s.strip_prefix('`')?;
        let fim = s.find('`')?;
        let n = &s[..fim];
        n.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            .then(|| n.to_string())
    };
    let mut membros = BTreeSet::new();
    let mut outros = BTreeSet::new();
    for l in doc.lines() {
        if let Some(n) = l.strip_prefix("| ").and_then(ident) {
            membros.insert(n);
        } else if let Some(n) = l.strip_prefix("- ").and_then(ident) {
            outros.insert(n);
        }
    }
    (membros, outros)
}

/// Os `test(nome)` do filtro da faixa isolada — a linha `filter =` que precede
/// `threads-required = 'num-cpus'`.
fn nomes_da_faixa(toml: &str) -> BTreeSet<String> {
    let linhas: Vec<&str> = toml.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.trim() == "threads-required = 'num-cpus'")
        .expect("a faixa isolada (`threads-required = 'num-cpus'`) sumiu do nextest.toml");
    let filtro = linhas[..i]
        .iter()
        .rev()
        .find(|l| l.trim_start().starts_with("filter ="))
        .expect("a faixa isolada não tem `filter =`");
    let mut out = BTreeSet::new();
    let mut resto = *filtro;
    while let Some(p) = resto.find("test(") {
        let t = &resto[p + 5..];
        let fim = t.find(')').unwrap_or(t.len());
        let n = &t[..fim];
        if !n.starts_with('/') {
            out.insert(n.to_string());
        }
        resto = &t[fim..];
    }
    out
}

fn fns_da_arvore(dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            fns_da_arvore(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let Ok(s) = std::fs::read_to_string(&p) else {
                continue;
            };
            let mut resto = s.as_str();
            while let Some(i) = resto.find("fn ") {
                let t = &resto[i + 3..];
                let fim = t
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(t.len());
                out.insert(t[..fim].to_string());
                resto = &t[fim..];
            }
        }
    }
}

#[test]
fn architecture_toda_flake_de_carga_corre_na_faixa_isolada() {
    let (membros, outros) = nomes_do_doc(&ler("docs/DevOps/FLAKES_DE_CARGA.md"));
    let faixa = nomes_da_faixa(&ler(".config/nextest.toml"));
    // Pisos de população: uma régua que lesse zero passaria por vácuo nas três metades.
    assert!(
        membros.len() >= 25,
        "a tabela do doc deu só {} membros — o formato mudou",
        membros.len()
    );
    assert!(
        faixa.len() >= 25,
        "o filtro da faixa deu só {} nomes — o formato mudou",
        faixa.len()
    );

    let fora: Vec<_> = membros.difference(&faixa).collect();
    assert!(
        fora.is_empty(),
        "membros da família que NÃO correm na faixa isolada (vão reprovar portões sob carga):\n  {fora:?}\n\
         Cura: acrescentar `+ test(<nome>)` ao filtro da faixa em .config/nextest.toml."
    );
    let sem_dono: Vec<_> = faixa
        .iter()
        .filter(|n| !membros.contains(*n) && !outros.contains(*n))
        .collect();
    assert!(
        sem_dono.is_empty(),
        "nomes na faixa isolada que o doc não explica:\n  {sem_dono:?}\n\
         Cura: uma linha na tabela de docs/DevOps/FLAKES_DE_CARGA.md (ou na lista «na faixa por \
         outra razão»), ou tirá-los do filtro."
    );

    let mut fns = BTreeSet::new();
    for topo in ["crates", "shells"] {
        fns_da_arvore(&raiz().join(topo), &mut fns);
    }
    assert!(
        fns.len() > 10_000,
        "o censo de `fn` achou só {} — a varredura partiu",
        fns.len()
    );
    let mortos: Vec<_> = membros
        .union(&faixa)
        .filter(|n| !fns.contains(*n))
        .collect();
    assert!(
        mortos.is_empty(),
        "nomes que já não são `fn` nenhum na árvore (um filtro morto lê-se como «protegido»):\n  {mortos:?}"
    );
}

/// Controlo positivo do leitor do doc: as duas formas de linha são reconhecidas e o resto não.
#[test]
fn o_leitor_do_doc_separa_membros_de_outros() {
    let doc = "| teste | onde |\n|---|---|\n| `um_membro` | x |\n- `outro_motivo` — y\n- texto\n";
    let (m, o) = nomes_do_doc(doc);
    assert_eq!(
        m.into_iter().collect::<Vec<_>>(),
        vec!["um_membro".to_string()]
    );
    assert_eq!(
        o.into_iter().collect::<Vec<_>>(),
        vec!["outro_motivo".to_string()]
    );
}

// ── B07.7 (14/11): todo portão de RAZÃO entre relógios corre na faixa; o PRAZO não ─────────────────
//
// Um teste que lê o relógio faz uma de duas coisas: afirma um PRAZO (`t.elapsed() < 5 s`, um tecto de
// segurança que a carga não vira, salvo ×10) ou uma RAZÃO entre dois relógios (`t4 / t2 < 1,5`) — e
// a razão é o que a fome de núcleo da suíte inverte. Serializar os prazos pararia a suíte por nada;
// deixar uma razão fora é a flake de carga à espera. O censo classifica pelo TEXTO do corpo:
// relógio = `let` cujo valor vem de `.elapsed()`, de outro relógio ou de um `fn` do mesmo ficheiro
// que lê `.elapsed()` e devolve `f64`/`Duration`/`u128`; razão = um `let` que divide dois relógios
// ou um `assert!` cuja condição junta dois. Só `#[test]` sem `#[ignore]`.

fn corpo(s: &str, de: usize) -> &str {
    let Some(j) = s[de..].find('{').map(|j| de + j) else {
        return "";
    };
    let mut d = 0i32;
    for (k, c) in s[j..].char_indices() {
        match c {
            '{' => d += 1,
            '}' => {
                d -= 1;
                if d == 0 {
                    return &s[j..=j + k];
                }
            }
            _ => {}
        }
    }
    &s[j..]
}

fn ident(t: &str) -> &str {
    let fim = t
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(t.len());
    &t[..fim]
}

/// Os identificadores minúsculos de uma expressão (os nomes de variável).
fn ids(e: &str) -> BTreeSet<&str> {
    e.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| {
            w.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
        .collect()
}

/// Os `fn` do ficheiro que leem o relógio e devolvem um tempo.
fn cronometros(s: &str) -> BTreeSet<&str> {
    let mut out = BTreeSet::new();
    let mut i = 0;
    while let Some(p) = s[i..].find("fn ") {
        let a = i + p + 3;
        let nome = ident(&s[a..]);
        let assinatura = s[a..].split('{').next().unwrap_or("");
        let devolve = assinatura.rsplit_once("->").map_or("", |(_, r)| r);
        if ["f64", "Duration", "u128"]
            .iter()
            .any(|t| devolve.contains(t))
            && corpo(s, a).contains("elapsed()")
        {
            out.insert(nome);
        }
        i = a;
    }
    out
}

/// `true` = razão entre relógios; `None` = o corpo nem lê o relógio.
fn e_razao(corpo: &str, crono: &BTreeSet<&str>) -> Option<bool> {
    let mut lets: Vec<(&str, &str)> = Vec::new();
    let mut i = 0;
    while let Some(p) = corpo[i..].find("let ") {
        let mut a = i + p + 4;
        i = a;
        if corpo[a..].starts_with("mut ") {
            a += 4;
        }
        if corpo[a..].starts_with('(') {
            a += 1;
        }
        let v = ident(&corpo[a..]);
        let resto = &corpo[a..];
        let (Some(eq), pv) = (resto.find('='), resto.find(';').unwrap_or(resto.len())) else {
            continue;
        };
        if v.is_empty() || eq > pv {
            continue;
        }
        lets.push((v, &resto[eq + 1..pv]));
    }
    let mut rel: BTreeSet<&str> = BTreeSet::new();
    for _ in 0..4 {
        for (v, e) in &lets {
            let ie = ids(e);
            if e.contains("elapsed()") || ie.iter().any(|w| rel.contains(w) || crono.contains(w)) {
                rel.insert(v);
            }
        }
    }
    if rel.is_empty() {
        return None;
    }
    let razoes: BTreeSet<&str> = lets
        .iter()
        .filter(|(_, e)| e.contains('/') && ids(e).iter().filter(|w| rel.contains(*w)).count() >= 2)
        .map(|(v, _)| *v)
        .collect();
    let mut i = 0;
    while let Some(p) = corpo[i..].find("assert!(") {
        let a = i + p + 8;
        i = a;
        let cond = corpo[a..].split(',').next().unwrap_or("");
        let ic = ids(cond);
        let n_rel = ic
            .iter()
            .filter(|w| rel.contains(*w) && !razoes.contains(*w))
            .count();
        if ic.iter().any(|w| razoes.contains(w)) || n_rel >= 2 {
            return Some(true);
        }
    }
    Some(false)
}

/// Os `#[test]` sem `#[ignore]` de um ficheiro, classificados: `(nome, razão?)`.
fn testes_com_relogio(s: &str) -> Vec<(String, bool)> {
    let crono = cronometros(s);
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = s[i..].find("#[test]") {
        let mut a = i + p + 7;
        i = a;
        let mut ignorado = false;
        loop {
            let t = s[a..].trim_start();
            a = s.len() - t.len();
            if t.starts_with("#[") {
                let fim = t.find(']').unwrap_or(t.len());
                ignorado |= t[..fim].contains("ignore");
                a += fim + 1;
            } else {
                break;
            }
        }
        let t = s[a..].trim_start_matches("pub ");
        let Some(t) = t.strip_prefix("fn ") else {
            continue;
        };
        let nome = ident(t);
        if ignorado {
            continue;
        }
        let mut c = crono.clone();
        c.remove(nome);
        if let Some(r) = e_razao(corpo(s, a), &c) {
            out.push((nome.to_string(), r));
        }
    }
    out
}

fn razoes_da_arvore(dir: &Path, out: &mut BTreeSet<String>, prazos: &mut usize) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            razoes_da_arvore(&p, out, prazos);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let Ok(s) = std::fs::read_to_string(&p) else {
                continue;
            };
            // O próprio censo traz os exemplos do controlo no texto.
            if !s.contains("elapsed()") || p.ends_with(file!().rsplit('/').next().unwrap_or("")) {
                continue;
            }
            for (n, r) in testes_com_relogio(&s) {
                if r {
                    out.insert(n);
                } else {
                    *prazos += 1;
                }
            }
        }
    }
}

#[test]
fn architecture_toda_razao_de_relogios_corre_na_faixa_isolada() {
    let faixa = nomes_da_faixa(&ler(".config/nextest.toml"));
    let (mut razoes, mut prazos) = (BTreeSet::new(), 0usize);
    for topo in ["crates", "shells"] {
        razoes_da_arvore(&raiz().join(topo), &mut razoes, &mut prazos);
    }
    // Pisos medidos 14/11: 15 razões, 40 com relógio que não o são (prazo ou só impressão).
    assert!(
        razoes.len() >= 12,
        "o censo achou só {} razões — a varredura partiu",
        razoes.len()
    );
    assert!(
        prazos >= 20,
        "o censo achou só {prazos} prazos — a varredura partiu"
    );
    let fora: Vec<_> = razoes.iter().filter(|n| !faixa.contains(*n)).collect();
    assert!(
        fora.is_empty(),
        "portões de RAZÃO entre relógios fora da faixa isolada (a suíte cheia inverte-os):\n  {fora:?}\n\
         Cura: `+ test(<nome>)` no filtro da faixa em .config/nextest.toml e a linha na tabela de \
         docs/DevOps/FLAKES_DE_CARGA.md (as duas escritas)."
    );
}

/// Controlo do classificador: a razão (por `let` e por `assert!` directo) acusa-se, o prazo não.
#[test]
fn o_censo_separa_o_prazo_da_razao() {
    let s = "fn ms(f: &dyn Fn()) -> f64 { let t = Instant::now(); f(); t.elapsed().as_secs_f64() }\n\
             #[test]\nfn razao_por_let() { let a = ms(&|| {}); let b = ms(&|| {}); let r = a / b; assert!(r < 1.5); }\n\
             #[test]\nfn razao_directa() { let t = Instant::now(); let a = t.elapsed(); let b = t.elapsed(); assert!(a < b * 2); }\n\
             #[test]\nfn prazo() { let t = Instant::now(); let d = t.elapsed(); assert!(d < Duration::from_secs(5)); }\n\
             #[test]\n#[ignore = \"sonda\"]\nfn sonda() { let a = ms(&|| {}); let b = ms(&|| {}); assert!(a / b < 2.0); }\n\
             #[test]\nfn sem_relogio() { let x = 1; assert!(x == 1); }\n";
    assert_eq!(
        testes_com_relogio(s),
        vec![
            ("razao_por_let".to_string(), true),
            ("razao_directa".to_string(), true),
            ("prazo".to_string(), false),
        ]
    );
}
