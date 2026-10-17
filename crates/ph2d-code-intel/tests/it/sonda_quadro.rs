//! ⏱ SONDA (I1, `docs/CodeEditor/03_onda2.md`): o que do reparse fora da tecla ainda corre na
//! THREAD DO QUADRO — deslocar a árvore (`edit_deferred`), tomar e enviar o trabalho (`job` +
//! `submit`), e trazer a árvore nova (`drain` + `adopt`, que larga a velha) — a 51 e a 7 500 linhas,
//! intercalados em blocos rodados no mesmo processo. Resumo = min / p50 / p99 / p99.9 (ms).
//!
//! `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-code-intel --test it sonda_quadro -- --ignored --nocapture`

use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_code_intel::GrammarWorker;
use ph2d_code_syntax::Syntax;
use std::time::Instant;

const BOB: &str = include_str!("../../../ph2d-code-syntax/tests/it/fixtures/bob.luau");

struct Lado {
    linhas: usize,
    buf: Buffer,
    syn: Syntax,
    worker: GrammarWorker<u8>,
    k: usize,
    edit: Vec<f64>,
    job: Vec<f64>,
    pump: Vec<f64>,
    /// Largar as árvores velhas NA thread do quadro (o antes da cura), quadro sim quadro não.
    largar_aqui: Vec<f64>,
    /// Entregá-las ao worker (a cura).
    largar_no_worker: Vec<f64>,
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn resumo(v: &[f64]) -> String {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let q = |p: f64| s[((s.len() - 1) as f64 * p).round() as usize];
    format!(
        "{:.3} / {:.3} / {:.3} / {:.3}",
        s[0],
        q(0.5),
        q(0.99),
        q(0.999)
    )
}

impl Lado {
    fn novo(linhas: usize) -> Self {
        let texto: String = BOB
            .lines()
            .cycle()
            .take(linhas)
            .map(|l| format!("{l}\n"))
            .collect();
        let mut buf = Buffer::new(&texto);
        let alvo = texto.find("self.t = self.t + dt").expect("alvo") + texto.len() / 2;
        let alvo = texto[alvo.min(texto.len() - 1)..]
            .find("self.t + dt")
            .map_or(0, |i| i + alvo.min(texto.len() - 1) + "self.t + dt".len());
        buf.set_selection(Selection::single(Range::cursor(alvo)));
        let syn = Syntax::new(buf.text());
        Self {
            linhas,
            buf,
            syn,
            worker: GrammarWorker::spawn(),
            k: 0,
            edit: Vec::new(),
            job: Vec::new(),
            pump: Vec::new(),
            largar_aqui: Vec::new(),
            largar_no_worker: Vec::new(),
        }
    }

    /// Uma tecla (letra; a cada 5, cinco Backspaces fora da régua) e, a cada 3, um quadro.
    fn tecla(&mut self) {
        let edits = self.buf.insert("x");
        let t = Instant::now();
        self.syn.edit_deferred(&edits);
        self.edit.push(ms(t));
        let t = Instant::now();
        if let Some(j) = self.syn.job(self.buf.text()) {
            self.worker.submit(0, j);
        }
        self.job.push(ms(t));
        self.k += 1;
        if self.k % 3 == 0 {
            // O quadro: o worker já acabou (16 ms depois, no app) — espera-se FORA da régua.
            self.worker.wait_all();
            let t = Instant::now();
            let mut retired = Vec::new();
            for (_, r) in self.worker.drain() {
                self.syn.adopt_into(r, &mut retired);
            }
            if let Some(j) = self.syn.job(self.buf.text()) {
                self.worker.submit(0, j);
            }
            self.pump.push(ms(t));
            let t = Instant::now();
            if (self.k / 3) % 2 == 0 {
                drop(retired);
                self.largar_aqui.push(ms(t));
            } else {
                self.worker.retire(retired);
                self.largar_no_worker.push(ms(t));
            }
        }
        if self.k % 5 == 0 {
            for _ in 0..5 {
                let e = self.buf.delete_backward();
                self.syn.edit_deferred(&e);
            }
        }
    }
}

#[test]
#[ignore = "medição: corre-se à mão (comando no topo do ficheiro)"]
fn sonda_quadro() {
    let mut lados = [Lado::novo(51), Lado::novo(7_500)];
    for l in &mut lados {
        for _ in 0..60 {
            l.tecla();
        }
        (l.edit.clear(), l.job.clear(), l.pump.clear());
        (l.largar_aqui.clear(), l.largar_no_worker.clear());
    }
    for r in 0..40 {
        for i in 0..2 {
            let l = &mut lados[(r + i) % 2];
            for _ in 0..30 {
                l.tecla();
            }
        }
    }
    let perfil = if cfg!(debug_assertions) {
        "debug"
    } else {
        "otimizado"
    };
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\nperfil {perfil} · loadavg {load}");
    println!(
        "| linhas | edit_deferred (n) | job + submit | drain + adopt + job (por quadro, n) | largar a velha AQUI | entregá-la ao worker |"
    );
    println!("|---|---|---|---|---|---|");
    for l in &lados {
        println!(
            "| {} | {} ({}) | {} | {} ({}) | {} | {} |",
            l.linhas,
            resumo(&l.edit),
            l.edit.len(),
            resumo(&l.job),
            resumo(&l.pump),
            l.pump.len(),
            resumo(&l.largar_aqui),
            resumo(&l.largar_no_worker)
        );
    }
}
