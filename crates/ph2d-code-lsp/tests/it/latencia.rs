//! T4 — latency probe (`#[ignore]`): through OUR client and OUR definitions, `didChange →
//! publishDiagnostics`, completion and hover on `bob.luau` and synthetic modules of 200/1 000/3 000
//! lines, old × new solver INTERLEAVED in one process (docs/DevOps/MEDIR_VELOCIDADE.md): two servers
//! side by side, rounds of short blocks with the starting solver rotated, summary = min/p50/p99/p99.9.
//!
//! Run: `bash scripts/cargo-test-narrow.sh ph2d-code-lsp --profile smoke -- --ignored latencia`
//! `PH2D_LSP_PROBE_DEFINITIONS=<file>` swaps the definitions (default `runtime/luau/ph2d.d.luau`).

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ph2d_code_lsp::{LspClient, Position, Range, Solver, byte_to_position};

use crate::support::{bob_luau, ph2d_definitions, start, uri};

const ROUNDS: usize = 10;
const BLOCK: usize = 25;
const WARMUP: usize = 3;
const PROBE_LINE: &str = "  local _probe = 1\n";

/// A realistic module of about `lines` lines over PH2D's API (properties, helpers, per-frame steps).
fn synthetic(lines: usize) -> String {
    let head = "-- synthetic probe module (crates/ph2d-code-lsp/tests/it/latencia.rs)\n\
        ph2d.property(\"speed\", 2.0, { min = 0, max = 10 })\n\
        ph2d.property(\"direction\", ph2d.vec2(0, 1), { min = -1, max = 1 })\n\
        ph2d.property(\"tint\", ph2d.color(1, 1, 1))\n\n\
        local function clamp(x: number, lo: number, hi: number): number\n\
        \x20 if x < lo then return lo elseif x > hi then return hi end\n\
        \x20 return x\n\
        end\n\n";
    // 10 head lines + 15 per block (14 of body + 1 call in `update`) + 7 tail lines.
    let blocks = (lines.saturating_sub(17) / 15).max(1);
    let mut s = String::from(head);
    for k in 0..blocks {
        let _ = write!(
            s,
            "local function step_{k}(self, dt: number)\n\
             \x20 local x = ph2d.get(self.id, \"x\") or 0\n\
             \x20 local y = ph2d.get(self.id, \"y\") or 0\n\
             \x20 local v = ph2d.vec2(x + dt * {k}, y)\n\
             \x20 local t = {{ a = v.x, b = v.y, n = {k} }}\n\
             \x20 for i = 1, 3 do\n\
             \x20   t.a = clamp(t.a + i * dt, -100, 100)\n\
             \x20 end\n\
             \x20 ph2d.set(self.id, \"x\", t.a)\n\
             \x20 if t.a > 50 then\n\
             \x20   ph2d.emit(\"far_{k}\")\n\
             \x20 end\n\
             end\n\n"
        );
    }
    s.push_str("function init(self)\n  self.t = 0\nend\n\nfunction update(self, dt)\n  self.t = self.t + dt\n");
    for k in 0..blocks {
        let _ = writeln!(s, "  step_{k}(self, dt)");
    }
    s.push_str("end\n");
    s
}

struct Doc {
    name: String,
    lines: usize,
    text: String,
    edit_at: Position,
    completion_at: Position,
    hover_at: Position,
}

impl Doc {
    fn new(name: &str, text: String) -> Self {
        let first_get = text.find("ph2d.get").expect("doc uses ph2d.get");
        let update = text.find("function update(self, dt)\n").expect("doc has update");
        let edit_line = byte_to_position(&text, update).line + 1;
        Self {
            name: format!("{name}.luau"),
            lines: text.lines().count(),
            completion_at: byte_to_position(&text, first_get + "ph2d.".len()),
            hover_at: byte_to_position(&text, first_get + "ph2d.g".len()),
            edit_at: Position::new(edit_line, 0),
            text,
        }
    }
}

#[derive(Default)]
struct Samples {
    open_ms: f64,
    open_diags: usize,
    diag: Vec<f64>,
    completion: Vec<f64>,
    hover: Vec<f64>,
    edit_completion: Vec<f64>,
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

/// Inserts or removes the probe line (incremental sync), so the text alternates between two states.
fn edit(c: &mut LspClient, u: &str, d: &Doc, inserting: bool) {
    let end = if inserting { d.edit_at } else { Position::new(d.edit_at.line + 1, 0) };
    let text = if inserting { PROBE_LINE } else { "" };
    c.did_change(u, Range { start: d.edit_at, end }, text).expect("didChange");
}

fn complete(c: &mut LspClient, u: &str, d: &Doc) -> f64 {
    let t = Instant::now();
    let items = c.completion(u, d.completion_at).expect("completion");
    assert!(!items.is_empty(), "{}: empty completion after `ph2d.`", d.name);
    ms(t.elapsed())
}

/// Two edits: (1) edit → diagnostics, then a completion and a hover on the checked text;
/// (2) edit → completion IMMEDIATELY (the keystroke → popup path), diagnostics drained after.
fn iterate(c: &mut LspClient, d: &Doc, out: Option<&mut Samples>) {
    let u = uri(&d.name);
    c.pump();
    let t0 = Instant::now();
    edit(c, &u, d, true);
    let published = c.wait_diagnostics(&u, Duration::from_secs(120)).expect("diagnostics");
    let diag_ms = ms(published.received.saturating_duration_since(t0));
    let completion_ms = complete(c, &u, d);
    let t2 = Instant::now();
    let hover = c.hover(&u, d.hover_at).expect("hover");
    let hover_ms = ms(t2.elapsed());
    assert!(hover.is_some(), "{}: no hover on ph2d.get", d.name);

    c.pump();
    let t3 = Instant::now();
    edit(c, &u, d, false);
    let items = c.completion(&u, d.completion_at).expect("completion");
    let edit_completion_ms = ms(t3.elapsed());
    assert!(!items.is_empty(), "{}: empty completion right after an edit", d.name);
    c.wait_diagnostics(&u, Duration::from_secs(120)).expect("diagnostics");

    if let Some(s) = out {
        s.diag.push(diag_ms);
        s.completion.push(completion_ms);
        s.hover.push(hover_ms);
        s.edit_completion.push(edit_completion_ms);
    }
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    let rank = ((q * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

fn summary(v: &[f64]) -> String {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    format!(
        "{:.1} / {:.1} / {:.1} / {:.1}",
        s[0],
        quantile(&s, 0.5),
        quantile(&s, 0.99),
        quantile(&s, 0.999)
    )
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg").map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" ")).unwrap_or_else(|_| "?".into())
}

fn rss_mb(pid: u32) -> String {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()
        .and_then(|s| s.lines().find(|l| l.starts_with("VmRSS")).map(str::to_owned))
        .and_then(|l| l.split_whitespace().nth(1).and_then(|kb| kb.parse::<f64>().ok()))
        .map_or("?".into(), |kb| format!("{:.0}", kb / 1024.0))
}

#[test]
#[ignore = "latency probe (T4) — run explicitly; it takes minutes"]
fn latencia_old_vs_new_solver_interleaved() {
    let defs = std::env::var_os("PH2D_LSP_PROBE_DEFINITIONS").map_or_else(ph2d_definitions, PathBuf::from);
    let load_start = loadavg();
    let docs = [
        Doc::new("bob", bob_luau()),
        Doc::new("m200", synthetic(200)),
        Doc::new("m1000", synthetic(1000)),
        Doc::new("m3000", synthetic(3000)),
    ];
    let solvers = [Solver::Old, Solver::New];
    let mut clients: Vec<LspClient> = solvers.iter().map(|&s| start(&defs, s)).collect();
    let mut samples: Vec<Vec<Samples>> = solvers.iter().map(|_| docs.iter().map(|_| Samples::default()).collect()).collect();

    // Cold first check (an event of its own), then warm-up outside the ruler.
    for (si, c) in clients.iter_mut().enumerate() {
        for (di, d) in docs.iter().enumerate() {
            let u = uri(&d.name);
            c.pump();
            let t0 = Instant::now();
            c.did_open(&u, &d.text).expect("didOpen");
            let p = c.wait_diagnostics(&u, Duration::from_secs(120)).expect("open diagnostics");
            samples[si][di].open_ms = ms(p.received.saturating_duration_since(t0));
            samples[si][di].open_diags = p.diagnostics.len();
            for _ in 0..WARMUP {
                iterate(c, d, None);
            }
        }
    }

    for round in 0..ROUNDS {
        for k in 0..solvers.len() {
            let si = (round + k) % solvers.len();
            for j in 0..docs.len() {
                let di = (round + j) % docs.len();
                for _ in 0..BLOCK {
                    iterate(&mut clients[si], &docs[di], Some(&mut samples[si][di]));
                }
            }
        }
    }

    let profile = if cfg!(debug_assertions) { "debug (client side)" } else { "optimised (client side)" };
    let mut table = format!(
        "luau-lsp {} · definitions {} · client {profile} · {ROUNDS} rounds × {BLOCK} per block (n = {} per cell) · loadavg start {load_start} · end {}\n\n\
         | doc (lines) | solver | 1st check ms | didChange→diagnostics ms (min / p50 / p99 / p99.9) | didChange→completion ms | completion (checked) ms | hover ms | diagnostics | RSS MB |\n\
         |---|---|---|---|---|---|---|---|---|\n",
        ph2d_code_lsp::pinned_version(),
        defs.display(),
        ROUNDS * BLOCK,
        loadavg(),
    );
    for (di, d) in docs.iter().enumerate() {
        for (si, s) in solvers.iter().enumerate() {
            let x = &samples[si][di];
            let _ = writeln!(
                table,
                "| {} ({}) | {:?} | {:.1} | {} | {} | {} | {} | {} | {} |",
                d.name,
                d.lines,
                s,
                x.open_ms,
                summary(&x.diag),
                summary(&x.edit_completion),
                summary(&x.completion),
                summary(&x.hover),
                x.open_diags,
                rss_mb(clients[si].pid()),
            );
        }
    }
    println!("\n{table}");
    for c in clients {
        c.shutdown().expect("shutdown");
    }
}
