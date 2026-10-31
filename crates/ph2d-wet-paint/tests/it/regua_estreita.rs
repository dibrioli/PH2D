//! **A régua de tolerância do caminho B** (rodada 10, `docs/Painter/49_realshader/MEDIDA_31_10_RODADA10.md`).
//!
//! Duas `Engine` idênticas correm a MESMA cena em passo-trancado no mesmo processo: a lei de hoje
//! (`estreito::ESTREITO = 0`) e a variante estreita (override em execução). Depois de CADA passo de água
//! compara-se a cor final 8-bit (`render_pigment_only`, todas as camadas): o pior quadro de `max` e de
//! `p99,9` de `|a − b|` por canal. Critério invisível: `max ≤ 1` e `p99,9 ≤ 0,5`.
//!
//! ```text
//! PH2D_ESTREITO=<bits> bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-wet-paint --test it \
//!   regua_estreita -- --ignored --nocapture
//! ```
//! `PH2D_ESTREITO` (bits de `ph2d_wet_paint::estreito`) — sem ele corre o lote das variantes.

use std::sync::atomic::Ordering;

use ph2d_wet_paint::estreito::{self, ESTREITO};
use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::render::{RenderLayer, render_pigment_only};

const QUADROS_MIN: usize = 300;

struct Par {
    a: Engine,
    b: Engine,
    modo: usize,
    quadros: usize,
    pior_max: u8,
    pior_max_q: usize,
    pior_p999: f64,
    pior_p999_q: usize,
    desalinhou: usize,
    buf_a: Vec<u8>,
    buf_b: Vec<u8>,
}

fn render(e: &Engine, out: &mut [u8]) {
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    render_pigment_only(&layers, out);
}

impl Par {
    fn new(w: usize, h: usize, modo: usize, monta: impl Fn(&mut Engine)) -> Self {
        let mut a = Engine::new(w, h);
        let mut b = Engine::new(w, h);
        monta(&mut a);
        monta(&mut b);
        Par {
            a,
            b,
            modo,
            quadros: 0,
            pior_max: 0,
            pior_max_q: 0,
            pior_p999: 0.0,
            pior_p999_q: 0,
            desalinhou: 0,
            buf_a: vec![0; w * h * 4],
            buf_b: vec![0; w * h * 4],
        }
    }

    /// A mesma acção nas duas, com o override ligado só na `b`.
    fn ambos<R: PartialEq + std::fmt::Debug>(&mut self, f: impl Fn(&mut Engine) -> R) -> R {
        ESTREITO.store(0, Ordering::Relaxed);
        let ra = f(&mut self.a);
        ESTREITO.store(self.modo, Ordering::Relaxed);
        let rb = f(&mut self.b);
        ESTREITO.store(0, Ordering::Relaxed);
        if ra != rb && self.desalinhou == 0 {
            // A água parou numa e não na outra: diferença de MODELO (visível por definição).
            self.desalinhou = self.quadros.max(1);
        }
        ra
    }

    fn passo(&mut self) -> bool {
        let ran = self.ambos(|e| e.step_simulation());
        if ran {
            self.quadros += 1;
            self.compara();
        }
        ran
    }

    fn compara(&mut self) {
        render(&self.a, &mut self.buf_a);
        render(&self.b, &mut self.buf_b);
        let mut hist = [0u64; 256];
        for (pa, pb) in self
            .buf_a
            .as_chunks::<4>()
            .0
            .iter()
            .zip(self.buf_b.as_chunks::<4>().0)
        {
            for c in 0..3 {
                hist[pa[c].abs_diff(pb[c]) as usize] += 1;
            }
        }
        let n: u64 = hist.iter().sum();
        let max = hist.iter().rposition(|&c| c > 0).unwrap_or(0) as u8;
        // p99,9 com interpolação entre níveis inteiros (a diferença é inteira; 0,5 = metade do intervalo).
        let alvo = 0.999 * n as f64;
        let mut acum = 0.0;
        let mut p = 0.0;
        for (v, &c) in hist.iter().enumerate() {
            let antes = acum;
            acum += c as f64;
            if acum >= alvo {
                p = if v == 0 || c == 0 {
                    v as f64
                } else {
                    (v - 1) as f64 + (alvo - antes) / c as f64
                };
                break;
            }
        }
        if max > self.pior_max {
            self.pior_max = max;
            self.pior_max_q = self.quadros;
        }
        if p > self.pior_p999 {
            self.pior_p999 = p;
            self.pior_p999_q = self.quadros;
        }
    }

    /// Um traço recto, um `pointer_frame` por relógio, a água corre quando a engine pede.
    fn traco(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, ppf: f64) {
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt();
        self.ambos(|e| e.pointer_down(x0, y0, None));
        let mut t = 0.0;
        let (mut x, mut y) = (x0, y0);
        while t < len {
            t = (t + ppf).min(len);
            x = x0 + dx * t / len;
            y = y0 + dy * t / len;
            self.ambos(|e| e.pointer_frame(x, y));
            if self.ambos(|e| e.sim_should_run()) {
                self.passo();
            }
        }
        self.ambos(|e| e.pointer_up());
        while self.ambos(|e| e.release_frame(x, y)) {}
    }

    fn agua(&mut self, n: usize) {
        for _ in 0..n {
            if !self.passo() {
                break;
            }
        }
    }

    fn veredito(&self, cena: &str) -> bool {
        let ok = self.pior_max <= 1 && self.pior_p999 <= 0.5 && self.desalinhou == 0;
        if self.desalinhou != 0 {
            println!(
                "  ⚠ {cena}: as duas engines desalinharam no quadro {}",
                self.desalinhou
            );
        }
        println!(
            "RÉGUA modo={:#04x} cena={cena}: {} quadros · pior max {} (quadro {}) · pior p99,9 {:.3} (quadro {}) · {}",
            self.modo,
            self.quadros,
            self.pior_max,
            self.pior_max_q,
            self.pior_p999,
            self.pior_p999_q,
            if ok { "INVISÍVEL" } else { "VISÍVEL" }
        );
        if self.quadros < QUADROS_MIN {
            println!("  ⚠ {cena}: só {} quadros (a água secou)", self.quadros);
        }
        ok
    }
}

/// O wash 4096×2560 da rodada 3 (`examples/nucleos_wash.rs`), depois água até `QUADROS_MIN`.
fn cena_wash(modo: usize) -> bool {
    let (w, h) = (4096usize, 2560usize);
    let mut p = Par::new(w, h, modo, |e| {
        e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
        e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
        e.sim.pickup_assentado = 0.0;
        e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
        e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
        e.sliders.water = 1.0;
        e.sliders.size = 1.0;
        e.wet_canvas_now();
    });
    let mut y = -32.0;
    while y < h as f64 + 32.0 {
        let (x0, x1) = (-64.0, w as f64 + 64.0);
        p.ambos(|e| e.pointer_down(x0, y, None));
        let mut x = x0;
        while x < x1 {
            x = (x + 16.0).min(x1);
            p.ambos(|e| e.dispatch_pressure_dab(x, y, 6.0, 60.0));
        }
        p.ambos(|e| e.pointer_up());
        while p.ambos(|e| e.release_frame(x1, y)) {}
        y += 12.0;
    }
    p.agua(QUADROS_MIN);
    p.veredito("wash-4K")
}

/// Tinta grossa (pouca água, pigmento cheio, pincel grande) em traços cruzados com água entre eles.
fn cena_impasto(modo: usize) -> bool {
    let (w, h) = (1024usize, 768usize);
    let mut p = Par::new(w, h, modo, |e| {
        e.sliders.water = 0.25;
        e.sliders.pressure = 1.0;
        e.sliders.size = 1.0;
        e.sliders.pigmento = 1.0;
        e.color = [180.0, 60.0, 30.0];
    });
    let mut volta = 0;
    while p.quadros < QUADROS_MIN && volta < 6 {
        let o = 17.0 * volta as f64;
        p.ambos(|e| e.color = [180.0, 60.0 + 20.0 * volta as f64, 30.0]);
        for k in 0..6 {
            let y = 120.0 + 100.0 * k as f64 + o;
            p.traco(60.0, y, 960.0, y + 40.0, 6.0);
            p.agua(15);
        }
        p.ambos(|e| e.color = [30.0, 90.0, 200.0 - 20.0 * volta as f64]);
        for k in 0..6 {
            let x = 150.0 + 140.0 * k as f64 + o;
            p.traco(x, 60.0, x + 30.0, 700.0, 6.0);
            p.agua(15);
        }
        volta += 1;
    }
    let falta = QUADROS_MIN.saturating_sub(p.quadros) + 20;
    p.agua(falta);
    p.veredito("impasto-cruzado")
}

/// Aguarela em três camadas, a de cima em velatura (K–M glaze), com água entre as passagens.
fn cena_aguarela(modo: usize) -> bool {
    let (w, h) = (1024usize, 768usize);
    let mut p = Par::new(w, h, modo, |e| {
        e.sliders.water = 1.0;
        e.sliders.size = 0.9;
        e.color = [40.0, 120.0, 200.0];
    });
    for k in 0..5 {
        let y = 150.0 + 110.0 * k as f64;
        p.traco(80.0, y, 940.0, y + 20.0, 5.0);
    }
    p.agua(100);
    p.ambos(|e| {
        e.add_layer();
        e.select_layer(1);
        e.color = [220.0, 170.0, 40.0];
        e.km_glaze = true;
    });
    for k in 0..4 {
        let x = 180.0 + 200.0 * k as f64;
        p.traco(x, 80.0, x + 60.0, 690.0, 5.0);
    }
    p.ambos(|e| e.tool = Tool::Wet);
    p.traco(100.0, 400.0, 900.0, 420.0, 8.0);
    p.ambos(|e| e.tool = Tool::Paint);
    p.agua(80);
    p.ambos(|e| {
        e.add_layer();
        e.select_layer(2);
        e.color = [200.0, 40.0, 90.0];
    });
    p.traco(120.0, 120.0, 900.0, 650.0, 5.0);
    let falta = QUADROS_MIN.saturating_sub(p.quadros) + 20;
    p.agua(falta);
    p.veredito("aguarela-camadas-glaze")
}

#[test]
#[ignore = "régua do caminho B: minutos em 4K — corre por nome com --ignored"]
fn regua_estreita() {
    let modos: Vec<usize> = match std::env::var("PH2D_ESTREITO") {
        Ok(s) => s
            .split(',')
            .map(|t| t.trim().parse().expect("bits"))
            .collect(),
        Err(_) => vec![
            estreito::VEL_ULP,
            estreito::VEL_F16,
            estreito::VEL_BF16,
            estreito::RGB_U16,
            estreito::MASSA_F16,
            estreito::MASSA_U16,
            estreito::VEL_F16 | estreito::RGB_U16 | estreito::MASSA_U16,
        ],
    };
    type Cena = (&'static str, fn(usize) -> bool);
    let cenas: &[Cena] = &[
        ("impasto", cena_impasto),
        ("aguarela", cena_aguarela),
        ("wash", cena_wash),
    ];
    let so = std::env::var("PH2D_ESTREITO_CENA").ok();
    for &m in &modos {
        for (nome, f) in cenas {
            if so.as_deref().is_some_and(|s| !s.contains(nome)) {
                continue;
            }
            f(m);
        }
    }
}
