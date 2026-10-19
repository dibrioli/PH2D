//! The pinned StyLua, RUN (CLAUDE.md §0.9): every gate here needs `bash scripts/stylua-fetch.sh`.
//! The fixtures are StyLua 2.5.2's own answers with `ph2d-format.toml`
//! (`docs/CodeEditor/ferramentas/medicoes_onda2/I0d_stylua.md`): `bob.formatado.luau` = `bob.luau`
//! formatted; `bob.sem_end.luau` = `bob.luau` without the `end` of `init`.

use ph2d_code_format::{Outcome, Program, Refusal, STYLUA, Worker, changes};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn fixture(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn bob() -> String {
    fixture("../ph2d-code-syntax/tests/it/fixtures/bob.luau")
}

fn pinned() -> Program {
    Program::pinned().unwrap_or_else(|e| {
        panic!("⛔ {e}\n   Este gate precisa do StyLua fixado: `bash scripts/stylua-fetch.sh`.")
    })
}

fn apply(old: &str, edits: &[ph2d_code_format::Edit]) -> String {
    let mut s = old.to_owned();
    for e in edits.iter().rev() {
        s.replace_range(e.range.clone(), &e.insert);
    }
    s
}

#[test]
fn o_stylua_resolvido_e_a_versao_fixada() {
    let Program::Stylua { bin, .. } = pinned() else {
        unreachable!()
    };
    let out = std::process::Command::new(&bin)
        .arg("--version")
        .output()
        .expect("stylua --version");
    let said = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        said.trim(),
        format!("stylua {}", STYLUA.version()),
        "{}",
        bin.display()
    );
}

#[test]
fn formata_como_o_programa_na_mao_e_e_idempotente() {
    let p = pinned();
    let (texto, esperado) = (bob(), fixture("tests/it/fixtures/bob.formatado.luau"));
    let novo = p.format(&texto).expect("o bob formata");
    assert_eq!(
        novo, esperado,
        "a saída da 2.5.2 com ph2d-format.toml mudou"
    );
    assert_eq!(
        p.format(&novo).as_deref(),
        Ok(esperado.as_str()),
        "formatar de novo não muda nada"
    );
    let e = changes(&texto, &novo);
    assert_eq!(apply(&texto, &e), novo);
    // Só o que o StyLua mudou: os 4 vãos dos comentários alinhados e o `if … then return end` partido.
    assert!(e.len() <= 8, "{e:#?}");
    assert!(
        e.iter().all(|x| x.range.len() + x.insert.len() < 60),
        "{e:#?}"
    );
    // Bordas medidas no I0d: vazio → vazio; sem `\n` final ganha um; CRLF → LF.
    assert_eq!(p.format("").as_deref(), Ok(""));
    assert_eq!(p.format("local x = 1").as_deref(), Ok("local x = 1\n"));
    assert_eq!(p.format("local x = 1\r\n").as_deref(), Ok("local x = 1\n"));
}

#[test]
fn as_recusas_sao_as_da_versao_fixada() {
    let p = pinned();
    match p.format(&fixture("tests/it/fixtures/bob.sem_end.luau")) {
        Err(Refusal::Syntax(m)) => assert!(m.contains("expected `end`"), "{m}"),
        r => panic!("sem o `end`: {r:?}"),
    }
    assert!(matches!(
        p.format("local x = = 1\n"),
        Err(Refusal::Syntax(_))
    ));
    // ⛔ 2.5.2 reescreve `{ read number }` como `{ readnumber }`; o `--verify` recusa.
    assert_eq!(
        p.format("type A = { read number }\n"),
        Err(Refusal::WouldChangeCode)
    );
    // O que nunca chega a correr.
    let falta = Program::stylua(PathBuf::from("/nao/existe/stylua")).expect("config");
    assert!(matches!(
        falta.format("local x = 1\n"),
        Err(Refusal::Failed(_))
    ));
}

/// A config do utilizador nunca vaza: com um `stylua.toml` hostil (tabs, aspas simples) e um
/// `.editorconfig` no cwd do processo-filho, a saída continua a nossa.
#[test]
fn a_config_de_fora_nunca_entra() {
    let Program::Stylua { bin, config } = pinned() else {
        unreachable!()
    };
    let d = std::env::temp_dir().join(format!("ph2d-stylua-hostil-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(
        d.join("stylua.toml"),
        "indent_type = \"Tabs\"\nquote_style = \"ForceSingle\"\n",
    )
    .unwrap();
    std::fs::write(
        d.join(".editorconfig"),
        "root = true\n[*]\nindent_style = tab\n",
    )
    .unwrap();
    let mut child = std::process::Command::new(&bin)
        .current_dir(&d)
        .arg("--config-path")
        .arg(&config)
        .args(["--verify", "--color", "never", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"if a then\nlocal s = 'x'\nend\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&d);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "if a then\n  local s = \"x\"\nend\n"
    );
    // O nosso ficheiro de regras nunca se chama `stylua.toml` (o StyLua lê-o sozinho, I0d §Achados 1).
    assert_ne!(
        config.file_name().and_then(|n| n.to_str()),
        Some("stylua.toml")
    );
}

#[test]
fn o_trabalhador_devolve_as_edicoes_do_texto_enviado() {
    let p = Arc::new(pinned());
    let mut w: Worker<u32> = Worker::default();
    let texto = bob();
    w.submit(&p, 1, 10, texto.clone());
    w.submit(&p, 2, 11, "local x = = 1\n".into());
    w.submit(&p, 3, 12, fixture("tests/it/fixtures/bob.formatado.luau"));
    w.wait(Duration::from_secs(30));
    let mut got = w.poll();
    assert_eq!(w.in_flight(), 0);
    got.sort_by_key(|d| d.key);
    assert_eq!(
        got.iter().map(|d| (d.key, d.ticket)).collect::<Vec<_>>(),
        [(1, 10), (2, 11), (3, 12)]
    );
    match &got[0].outcome {
        Outcome::Changes(e) => assert_eq!(
            apply(&texto, e),
            fixture("tests/it/fixtures/bob.formatado.luau")
        ),
        o => panic!("{o:?}"),
    }
    assert!(matches!(
        got[1].outcome,
        Outcome::Refused(Refusal::Syntax(_))
    ));
    assert_eq!(
        got[2].outcome,
        Outcome::Changes(Vec::new()),
        "já formatado = nenhuma edição"
    );
}

/// O texto da bancada (o `bob` repetido, como em `ph2d-app-code/tests/it/bancada.rs` e no `gera.py`
/// do I0d), com `n` linhas.
fn juntado(n: usize) -> String {
    let b: Vec<String> = bob().lines().skip(2).map(str::to_owned).collect();
    (0..n)
        .map(|i| {
            let l = &b[i % b.len()];
            if i >= b.len() && l.starts_with("ph2d.property(") {
                format!("-- {l}\n")
            } else {
                format!("{l}\n")
            }
        })
        .collect()
}

/// A medição do I13 (fora do fio da UI, mas é o atraso entre o Ctrl+S e o texto formatado): o
/// StyLua + as edições a 51 / 7 500 linhas, e o pior caso das edições (TODA linha re-recuada).
/// Só imprime — o relógio sob carga não é veredito; o registo vai para a medição do I13.
#[test]
fn medicao_do_atraso_da_formatacao() {
    let p = pinned();
    println!(
        "loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for n in [51, 7500] {
        let t = juntado(n);
        let mut fmt = Vec::new();
        let mut dif = Vec::new();
        for _ in 0..7 {
            let a = Instant::now();
            let novo = p.format(&t).expect("formata");
            let b = Instant::now();
            let e = changes(&t, &novo);
            fmt.push((b - a).as_secs_f64() * 1e3);
            dif.push(b.elapsed().as_secs_f64() * 1e3);
            assert_eq!(apply(&t, &e), novo);
        }
        let min = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
        println!(
            "{n} linhas: stylua --verify min {:.2} ms · edições min {:.2} ms",
            min(&fmt),
            min(&dif)
        );
    }
    // Pior caso das edições: todo o texto re-recuado (4 espaços a mais por linha).
    let t = juntado(7500);
    let torto: String = t.lines().map(|l| format!("    {l}\n")).collect();
    let a = Instant::now();
    let e = changes(&torto, &t);
    println!(
        "7500 linhas re-recuadas: edições {:.2} ms ({} edições)",
        a.elapsed().as_secs_f64() * 1e3,
        e.len()
    );
    assert_eq!(apply(&torto, &e), t);
    // Sem relógio: linha re-recuada = UMA edição (o caminho por pares; sem ele, 10 193 edições e
    // 1 849 ms — medido 18/10, medicoes_onda2/I13.md).
    assert_eq!(e.len(), 7500, "uma edição por linha re-recuada");
}
