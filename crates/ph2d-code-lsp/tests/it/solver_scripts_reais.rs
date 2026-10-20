//! I15 — o verificador de tipos antigo × o novo sobre os scripts REAIS (`#[ignore]`, medição): os
//! oito do corpus do I0c, os três que as cenas do produto trazem e o espelho estrito, todos abertos
//! no MESMO servidor como o app abre o projecto. Dois servidores lado a lado, rodadas intercaladas com
//! o solver inicial rodado (docs/DevOps/MEDIR_VELOCIDADE.md). Por solver: (1) o que cada um ACUSA em
//! cada script, normal e `--!strict`, e a diferença; (2) se o servidor MORRE em algum pedido do editor;
//! (3) o custo de cada pedido do editor depois de uma edição (diagnósticos, lista, balão, assinatura,
//! cores, dicas, símbolos, referências). Critério de morte (03_onda2 I15): se o novo estourar, fica o
//! antigo. Tabela: `docs/CodeEditor/ferramentas/medicoes_onda2/I15_solver.md`.
//!
//! Corre: `bash scripts/ph2d-run.sh cargo test -p ph2d-code-lsp --profile smoke --test it solver_scripts_reais -- --ignored --nocapture`

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

use ph2d_code_lsp::{Diagnostic, LspClient, LspError, Position, Range, Solver};
use serde_json::{Value, json};

use crate::support::{fixture, ph2d_definitions, repo_root, start, uri};

const ROUNDS: usize = 20;
const BLOCK: usize = 5;
const WARMUP: usize = 3;
const WAIT: Duration = Duration::from_secs(120);
/// A linha que cada iteração acrescenta no fim e tira: uma chamada da API (lista, balão, assinatura)
/// e um local (referências) em TODO script, tenha ele `ph2d.` ou não.
const PROBE: &str = "local _probe = ph2d.get(0, \"x\")\n";

const CORPUS: [&str; 8] = [
    "crates/ph2d-code-syntax/tests/it/fixtures/bob.luau",
    "crates/ph2d-app-components/src/script_boot.luau",
    "runtime/luau/examples/ph2d_script_attach.luau",
    "docs/scripting/examples/spike/lsp/case_2_coroutine.luau",
    "tests/spike/c7_typical_script.luau",
    "tests/spike/coro_timing.luau",
    "tests/spike/gc_stress.luau",
    "tests/spike/hello.luau",
];

const OPS: [&str; 8] = [
    "edição→diagnósticos",
    "lista (`ph2d.`)",
    "balão",
    "assinatura",
    "cores",
    "dicas de tipo",
    "símbolos",
    "referências",
];

/// O texto de `const <name>: &str = r#"…"#;` em `file` (a fonte única, nunca uma cópia).
fn raw_const(file: &str, name: &str) -> String {
    let src = std::fs::read_to_string(repo_root().join(file)).expect(file);
    let open = format!("pub const {name}: &str = r#\"");
    assert_eq!(src.matches(&open).count(), 1, "{name} em {file}");
    let start = src.find(&open).unwrap() + open.len();
    let len = src[start..].find("\"#;").expect("fim da string");
    src[start..start + len].to_owned()
}

struct Script {
    name: String,
    text: String,
    /// A 1.ª linha DEPOIS do texto (onde a sonda entra).
    end: u32,
}

fn scripts() -> Vec<Script> {
    let mut v: Vec<(String, String)> = CORPUS
        .iter()
        .map(|p| {
            let name = p.rsplit('/').next().unwrap().to_owned();
            (name, std::fs::read_to_string(repo_root().join(p)).expect(p))
        })
        .collect();
    let comp = "crates/ph2d-app-components/src";
    v.push((
        "code_smoke_bob.luau".into(),
        raw_const(&format!("{comp}/code_smoke.rs"), "BOB_LUAU"),
    ));
    v.push((
        "code_smoke_onda.luau".into(),
        raw_const(&format!("{comp}/code_smoke.rs"), "ONDA_LUAU"),
    ));
    v.push((
        "script_smoke_bob.luau".into(),
        raw_const(&format!("{comp}/script_smoke.rs"), "BOB_LUAU"),
    ));
    v.push((
        "bob_estrito.luau".into(),
        std::fs::read_to_string(fixture("bob_estrito.luau")).expect("espelho estrito"),
    ));
    v.into_iter()
        .map(|(name, mut text)| {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            let end = u32::try_from(text.lines().count()).unwrap();
            Script { name, text, end }
        })
        .collect()
}

fn strict(text: &str) -> String {
    if text.starts_with("--!strict") {
        text.to_owned()
    } else {
        format!("--!strict\n{text}")
    }
}

type Seen = Vec<(u32, String, String)>;

fn seen(d: &[Diagnostic]) -> Seen {
    let mut v: Seen = d
        .iter()
        .map(|x| {
            (
                x.range.start.line + 1,
                x.code.clone().unwrap_or_default(),
                x.message.lines().next().unwrap_or("").to_owned(),
            )
        })
        .collect();
    v.sort();
    v
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn pos(line: u32, ch: u32) -> Value {
    json!({ "line": line, "character": ch })
}

/// Um servidor com TODOS os scripts abertos (normal e estrito), como o app espelha o projecto.
struct Lado {
    solver: Solver,
    c: LspClient,
    abertura_ms: f64,
    mortes: BTreeMap<&'static str, usize>,
    /// `[script][op]` em ms.
    t: Vec<Vec<Vec<f64>>>,
    normal: Vec<Seen>,
    estrito: Vec<Seen>,
}

impl Lado {
    fn novo(solver: Solver, ss: &[Script]) -> Self {
        let c = start(&ph2d_definitions(), solver);
        let mut l = Self {
            solver,
            c,
            abertura_ms: 0.0,
            mortes: BTreeMap::new(),
            t: ss.iter().map(|_| vec![Vec::new(); OPS.len()]).collect(),
            normal: Vec::new(),
            estrito: Vec::new(),
        };
        let t0 = Instant::now();
        let (n, e) = l.abre(ss);
        l.abertura_ms = ms(t0.elapsed());
        l.normal = n;
        l.estrito = e;
        l
    }

    fn abre(&mut self, ss: &[Script]) -> (Vec<Seen>, Vec<Seen>) {
        let mut out = (Vec::new(), Vec::new());
        for s in ss {
            for (u, t, dst) in [
                (uri(&s.name), s.text.clone(), &mut out.0),
                (
                    uri(&format!("strict_{}", s.name)),
                    strict(&s.text),
                    &mut out.1,
                ),
            ] {
                self.c.pump();
                self.c.did_open(&u, &t).expect("didOpen");
                let p = self
                    .c
                    .wait_diagnostics(&u, WAIT)
                    .expect("diagnósticos ao abrir");
                dst.push(seen(&p.diagnostics));
            }
        }
        out
    }

    /// O servidor morreu: conta-se e recomeça (o `restart` reabre cada documento no texto dele).
    /// O texto do script volta ao original (a sonda pode ter ficado lá dentro).
    fn morreu(&mut self, op: &'static str, s: &Script) {
        *self.mortes.entry(op).or_default() += 1;
        self.c.restart().expect("restart");
        self.c
            .did_change_full(&uri(&s.name), &s.text)
            .expect("texto original");
    }

    fn pedido(&mut self, method: &str, params: Value) -> Result<f64, LspError> {
        let t = Instant::now();
        self.c.request(method, params)?;
        Ok(ms(t.elapsed()))
    }

    /// Uma iteração sobre `ss[i]`: a sonda entra no fim, cada pedido do editor, a sonda sai.
    fn itera(&mut self, ss: &[Script], i: usize, mede: bool) {
        let s = &ss[i];
        let u = uri(&s.name);
        let at = |ch| Position::new(s.end, ch);
        self.c.pump();
        let t0 = Instant::now();
        if self
            .c
            .did_change(
                &u,
                Range {
                    start: at(0),
                    end: at(0),
                },
                PROBE,
            )
            .is_err()
        {
            return self.morreu(OPS[0], s);
        }
        let diag = match self.c.wait_diagnostics(&u, WAIT) {
            Ok(p) => ms(p.received.saturating_duration_since(t0)),
            Err(_) => return self.morreu(OPS[0], s),
        };
        let doc = json!({ "uri": u });
        let get = PROBE.find("get").unwrap() as u32;
        let paren = PROBE.find('(').unwrap() as u32 + 1;
        let pedidos: [(&str, Value); 7] = [
            (
                "textDocument/completion",
                json!({ "textDocument": doc, "position": pos(s.end, get) }),
            ),
            (
                "textDocument/hover",
                json!({ "textDocument": doc, "position": pos(s.end, get + 1) }),
            ),
            (
                "textDocument/signatureHelp",
                json!({ "textDocument": doc, "position": pos(s.end, paren) }),
            ),
            (
                "textDocument/semanticTokens/full",
                json!({ "textDocument": doc }),
            ),
            (
                "textDocument/inlayHint",
                json!({ "textDocument": doc,
                        "range": { "start": pos(0, 0), "end": pos(s.end + 1, 0) } }),
            ),
            (
                "textDocument/documentSymbol",
                json!({ "textDocument": doc }),
            ),
            (
                "textDocument/references",
                json!({ "textDocument": doc, "position": pos(s.end, 7),
                        "context": { "includeDeclaration": true } }),
            ),
        ];
        let mut row = vec![diag];
        for (k, (m, p)) in pedidos.into_iter().enumerate() {
            match self.pedido(m, p) {
                Ok(t) => row.push(t),
                Err(LspError::ServerDied { .. }) => return self.morreu(OPS[k + 1], s),
                Err(e) => panic!("{:?} {m} em {}: {e}", self.solver, s.name),
            }
        }
        self.c.pump();
        let fim = Position::new(s.end + 1, 0);
        if self
            .c
            .did_change(
                &u,
                Range {
                    start: at(0),
                    end: fim,
                },
                "",
            )
            .is_err()
            || self.c.wait_diagnostics(&u, WAIT).is_err()
        {
            return self.morreu(OPS[0], s);
        }
        if mede {
            for (k, t) in row.into_iter().enumerate() {
                self.t[i][k].push(t);
            }
        }
    }
}

fn quantil(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let r = ((q * s.len() as f64).ceil() as usize).clamp(1, s.len());
    s[r - 1]
}

fn resumo(v: &[f64]) -> String {
    if v.is_empty() {
        return "—".into();
    }
    format!(
        "{:.2} / {:.2} / {:.2}",
        quantil(v, 0.0),
        quantil(v, 0.5),
        quantil(v, 0.99)
    )
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

fn rss_mb(pid: u32) -> String {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok())
        })
        .map_or("?".into(), |kb| format!("{:.0}", kb / 1024.0))
}

#[test]
#[ignore = "medição (I15): corre-se à mão, comando no topo do ficheiro"]
fn solver_scripts_reais() {
    let load0 = loadavg();
    let ss = scripts();
    let mut lados = [Lado::novo(Solver::Old, &ss), Lado::novo(Solver::New, &ss)];
    for l in &mut lados {
        for i in 0..ss.len() {
            for _ in 0..WARMUP {
                l.itera(&ss, i, false);
            }
        }
    }
    let t_rodada = Instant::now();
    for r in 0..ROUNDS {
        for k in 0..lados.len() {
            let li = (r + k) % lados.len();
            for j in 0..ss.len() {
                let i = (r + j) % ss.len();
                for _ in 0..BLOCK {
                    lados[li].itera(&ss, i, true);
                }
            }
        }
    }
    let parede = t_rodada.elapsed().as_secs_f64();

    let perfil = if cfg!(debug_assertions) {
        "debug"
    } else {
        "otimizado"
    };
    let mut out = format!(
        "luau-lsp {} · cliente {perfil} · {ROUNDS} rodadas × {BLOCK} (n = {} por célula) · loadavg início {load0} · fim {} · rodada {parede:.1} s\n",
        ph2d_code_lsp::pinned_version(),
        ROUNDS * BLOCK,
        loadavg()
    );
    let _ = writeln!(
        out,
        "\nAbrir os {} scripts (normal + estrito, {} documentos): antigo {:.0} ms · novo {:.0} ms · RSS antigo {} MB · novo {} MB",
        ss.len(),
        2 * ss.len(),
        lados[0].abertura_ms,
        lados[1].abertura_ms,
        rss_mb(lados[0].c.pid()),
        rss_mb(lados[1].c.pid())
    );
    let _ = writeln!(
        out,
        "\nMortes do servidor (pedido → vezes): antigo {:?} · novo {:?}",
        lados[0].mortes, lados[1].mortes
    );

    let _ = writeln!(
        out,
        "\n## O que cada um acusa (linha, código, 1.ª linha da mensagem)\n"
    );
    let mut iguais = 0;
    for (i, s) in ss.iter().enumerate() {
        for (modo, a, b) in [
            ("normal", &lados[0].normal[i], &lados[1].normal[i]),
            ("estrito", &lados[0].estrito[i], &lados[1].estrito[i]),
        ] {
            if a == b {
                iguais += 1;
                continue;
            }
            let _ = writeln!(out, "### {} ({} linhas) · {modo}", s.name, s.end);
            for x in a.iter().filter(|x| !b.contains(x)) {
                let _ = writeln!(out, "- só o ANTIGO: {x:?}");
            }
            for x in b.iter().filter(|x| !a.contains(x)) {
                let _ = writeln!(out, "- só o NOVO: {x:?}");
            }
        }
    }
    let _ = writeln!(
        out,
        "\n{iguais} de {} (script × modo) iguais nos dois.",
        2 * ss.len()
    );
    let conta = |v: &[Seen]| v.iter().map(Vec::len).sum::<usize>();
    let _ = writeln!(
        out,
        "Total acusado: normal antigo {} · novo {} · estrito antigo {} · novo {}",
        conta(&lados[0].normal),
        conta(&lados[1].normal),
        conta(&lados[0].estrito),
        conta(&lados[1].estrito)
    );

    let _ = writeln!(out, "\n## Custo por pedido, ms (min / p50 / p99)\n");
    let _ = writeln!(out, "| script (linhas) | solver | {} |", OPS.join(" | "));
    let _ = writeln!(out, "|---|---|{}", "---|".repeat(OPS.len()));
    for (i, s) in ss.iter().enumerate() {
        for l in &lados {
            let cel: Vec<String> = l.t[i].iter().map(|v| resumo(v)).collect();
            let _ = writeln!(
                out,
                "| {} ({}) | {:?} | {} |",
                s.name,
                s.end,
                l.solver,
                cel.join(" | ")
            );
        }
    }
    let _ = writeln!(
        out,
        "\n## O pior p50 de cada pedido sobre os scripts (ms)\n"
    );
    let _ = writeln!(out, "| solver | {} |", OPS.join(" | "));
    let _ = writeln!(out, "|---|{}", "---|".repeat(OPS.len()));
    for l in &lados {
        let cel: Vec<String> = (0..OPS.len())
            .map(|k| {
                let p = (0..ss.len())
                    .map(|i| quantil(&l.t[i][k], 0.5))
                    .fold(0.0, f64::max);
                format!("{p:.2}")
            })
            .collect();
        let _ = writeln!(out, "| {:?} | {} |", l.solver, cel.join(" | "));
    }
    println!("\n{out}");
    for l in lados {
        l.c.shutdown().expect("shutdown");
    }
}
