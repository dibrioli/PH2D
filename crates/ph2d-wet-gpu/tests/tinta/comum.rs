//! As cenas e as réguas da E11, partilhadas pelos portões de `tests/tinta.rs`.

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::passo::PassoPlaca;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::grid::tinta::pigmento::optica_celula;
use ph2d_wet_paint::grid::tinta::{CelulaTinta, em, pilha};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::{LeiDaTinta, Sim, sim_step};
use ph2d_wet_paint::tuning::Knob;

/// A cor sobre branco, níveis 0–255, nas células com tinta à vista ([`com_tinta`]). O p99.9 é o do degrau da lei velha
/// (`tests/passo.rs`), e vale duro: o pior medido é 0,39 (a pasta, nas duas placas).
pub(crate) const COR_P999: f64 = 0.5;
/// O máximo e a fracção acima do [`COR_P999`] são da FRANJA: a máscara `active` pergunta `film > 0`, e um resíduo de film que
/// desaparece muda o escoamento de poucas células. A própria CPU move-se assim (só zerar abaixo de 1e-38: 13 níveis em 3 células da
/// pasta; abaixo de 1e-20: 112 em 119). Pior medido: 21,6 níveis, 8,3e-4 (a pasta). Tabela: `docs/Painter/49_realshader/PLACA_E11_tinta.md`.
pub(crate) const COR_MAX: f64 = 32.0;
pub(crate) const FRACAO_NA_FRANJA: f64 = 1e-3;
/// Quanto a soma do filme e do aberto (massa inteira da grade) pode afastar-se da CPU, relativo: ~13× o pior medido (7,5e-7).
pub(crate) const MASSA_REL: f64 = 1e-5;

pub(crate) struct Cena {
    pub nome: &'static str,
    pub medium: f64,
    /// `None` = a Dilution de fábrica.
    pub dilution: Option<f64>,
    pub body: f64,
    /// A abertura de partida de toda a tinta (`1` = fresca): abaixo de 1 o re-molhar e o reabrir pelo volume mordem.
    pub abertura: f32,
    /// A fracção que desce mais que a água, com a gravidade da cena (`0` = sem `sedimenta`).
    pub sedimenta: f64,
}

pub(crate) const CENAS: [Cena; 6] = [
    Cena {
        nome: "água",
        medium: 0.0,
        dilution: None,
        body: 0.0,
        abertura: 1.0,
        sedimenta: 0.0,
    },
    Cena {
        nome: "óleo diluído",
        medium: 1.0,
        dilution: Some(0.8),
        body: 0.0,
        abertura: 1.0,
        sedimenta: 0.0,
    },
    Cena {
        // φ = 0,8: Bingham a morder (acima de φ_c) e um resto de líquido livre; abaixo de `Dilution` 0,1 não há líquido
        // nenhum e o passo não tem o que andar (a cena seria muda).
        nome: "pasta alta",
        medium: 1.0,
        dilution: Some(0.2),
        body: 0.5,
        abertura: 1.0,
        sedimenta: 0.0,
    },
    Cena {
        nome: "óleo meio curado",
        medium: 1.0,
        dilution: Some(0.8),
        body: 0.0,
        abertura: 0.5,
        sedimenta: 0.0,
    },
    Cena {
        // Quase fechada e com o re-molhar no máximo: o que levanta é a abertura que decide.
        nome: "óleo quase fechado",
        medium: 1.0,
        dilution: Some(0.8),
        body: 0.0,
        abertura: 0.02,
        sedimenta: 0.0,
    },
    Cena {
        nome: "água a sedimentar",
        medium: 0.0,
        dilution: None,
        body: 0.0,
        abertura: 1.0,
        sedimenta: 0.3,
    },
];

/// O que a CPU e a placa podem discordar por campo, no p99.9 das células com tinta: ~5× o pior das 6 cenas nas duas placas
/// (tabela em `docs/Painter/49_realshader/PLACA_E11_tinta.md`). A cor não vê uma lei que ainda não mudou a tinta; o campo vê.
pub(crate) const FILM_P999: f64 = 2.5e-4;
pub(crate) const MASSA_P999: f64 = 0.5;
pub(crate) const SOMA_P999: f64 = 3.5e-4;
pub(crate) const ABERTURA_P999: f64 = 3e-5;
/// A pior célula: duas curas de `2,78e-4` cada.
pub(crate) const ABERTURA_MAX: f64 = 5.6e-4;
/// Abaixo deste volume a abertura não governa nada que se veja.
pub(crate) const ABERTO_QUE_CONTA: f32 = 1e-6;
pub(crate) const MEIO_P999: f64 = 3e-6;
pub(crate) const ESPELHO_P999: f64 = 0.03;

pub(crate) fn contexto() -> Option<GpuContext> {
    match GpuContext::new(GpuContext::default_instance(), None) {
        Ok(g) => {
            let i = g.adapter.get_info();
            eprintln!("placa: {} · {:?} · {}", i.name, i.backend, i.driver_info);
            Some(g)
        }
        Err(e) => {
            eprintln!("⛔ sem placa ({e}) — nada medido");
            None
        }
    }
}

pub(crate) fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64) {
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 6.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

pub(crate) fn pinta(c: &Cena, w: usize, h: usize) -> Engine {
    let mut e = Engine::new(w, h);
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.pickup_assentado = 0.0;
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.set_knob(Knob::Medium, c.medium);
    e.set_knob(Knob::Body, c.body);
    if let Some(d) = c.dilution {
        e.set_knob(Knob::Dilution, d);
    }
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let (wf, hf) = (w as f64, h as f64);
    // O primeiro traço entra pela borda da folha: o dreno do anel tem de levar a soma com a massa.
    traco(&mut e, 2.0, 0.9 * wf, 0.3 * hf);
    e.color = [0.8, 0.2, 0.1];
    traco(&mut e, 0.2 * wf, 0.8 * wf, 0.36 * hf);
    e.color = [0.1, 0.3, 0.9];
    traco(&mut e, 0.15 * wf, 0.7 * wf, 0.62 * hf);
    if c.sedimenta > 0.0 {
        e.sim.sedimenta = c.sedimenta;
        e.sim.gravity_override = Some([0.0, 0.08]);
    }
    if c.abertura < 0.1 {
        // Muito assente debaixo da água: sem isto o que levanta fica abaixo da régua e a abertura não decide nada à vista.
        e.set_knob(Knob::Rewet, 8.0);
        let idx = e.active_layer;
        let g = &mut e.layers[idx].grid;
        for i in 0..g.cells {
            if g.film[i] > 0.0 {
                g.sett[i] = 2000.0;
                g.tinta[i].aberto = ph2d_wet_paint::grid::tinta::soma_da_massa(
                    [200.0, 60.0, 30.0],
                    0.2,
                    1.0,
                    2000.0,
                );
            }
        }
    }
    if c.abertura < 1.0 {
        let idx = e.active_layer;
        for t in &mut e.layers[idx].grid.tinta {
            t.o = c.abertura;
        }
    }
    e
}

pub(crate) fn da_placa(gpu: &GpuContext, e: &Engine, passos: usize) -> (Sim, Grid) {
    let idx = e.active_layer;
    let (mut s, mut g) = (e.sim.clone(), e.layers[idx].grid.clone());
    let mut p = PassoPlaca::new(gpu, &g, &s, &e.tuning);
    for _ in 0..passos {
        p.passo(gpu, &mut s, &mut g, &e.tuning);
    }
    p.desce(gpu, &mut g);
    (s, g)
}

/// O ciclo do WORKER (`offthread/placa.rs`): `cada` passos, `desce_suja`, `sobe_sujo`; a grade final é só o que a descida suja
/// trouxe, nunca uma descida inteira.
pub(crate) fn do_ciclo(gpu: &GpuContext, e: &Engine, passos: usize, cada: usize) -> (Sim, Grid) {
    let idx = e.active_layer;
    let (mut s, mut g) = (e.sim.clone(), e.layers[idx].grid.clone());
    let mut p = PassoPlaca::new(gpu, &g, &s, &e.tuning);
    for n in 1..=passos {
        p.passo(gpu, &mut s, &mut g, &e.tuning);
        if n % cada == 0 && n < passos {
            p.desce_suja(gpu, &mut g);
            let subiu = p.sobe_sujo(gpu, &g);
            assert!(subiu.is_none(), "a CPU não mudou nada e subiu {subiu:?}");
        }
    }
    p.desce_suja(gpu, &mut g);
    (s, g)
}

pub(crate) fn da_cpu(e: &Engine, passos: usize) -> (Sim, Grid) {
    let idx = e.active_layer;
    let (mut s, mut g) = (e.sim.clone(), e.layers[idx].grid.clone());
    for _ in 0..passos {
        sim_step(&mut s, &mut g, &e.tuning);
    }
    (s, g)
}

/// A cor sobre branco da lei nova, sRGB 0–255: filme e aberto pela óptica da célula sobre o fundo curado.
pub(crate) fn cor(g: &Grid, i: usize) -> [f64; 3] {
    let c = em(g, i);
    let molhado = g.film[i] > 0.0;
    let o = optica_celula(&c.filme, &c.aberto, None, c.meio(), molhado);
    pilha(&o, &c)
        .0
        .0
        .map(|r| f64::from(ph2d_paint_tinta::oklab::linear_para_srgb(r)) * 255.0)
}

pub(crate) fn d_cor(a: &Grid, b: &Grid, i: usize) -> f64 {
    let (x, y) = (cor(a, i), cor(b, i));
    (x[0] - y[0])
        .abs()
        .max((x[1] - y[1]).abs())
        .max((x[2] - y[2]).abs())
}

/// As células do interior COM TINTA À VISTA em algum dos dois lados (a cor sobre branco a mais de 1/2 nível): o denominador das
/// réguas. A folha é quase toda branca, e a massa em si não serve: `v ~ 1e-39` é massa e ninguém a vê (40 000 células com massa,
/// 32 000 à vista, 64 000 na folha).
pub(crate) fn com_tinta(a: &Grid, b: &Grid) -> Vec<usize> {
    let tem = |g: &Grid, i: usize| cor(g, i).iter().any(|&v| v < 254.5);
    let mut v = Vec::new();
    for y in 1..=a.h {
        for x in 1..=a.w {
            let i = x + y * a.s;
            if tem(a, i) || tem(b, i) {
                v.push(i);
            }
        }
    }
    v
}

/// `(máx, p99.9, células acima do` [`COR_P999`]`)` da cor nas células `pop`.
pub(crate) fn desvio_cor(a: &Grid, b: &Grid, pop: &[usize]) -> (f64, f64, usize) {
    let (mx, p) = desvio_em(pop, |i| d_cor(a, b, i));
    let n = pop.iter().filter(|&&i| d_cor(a, b, i) > COR_P999).count();
    (mx, p, n)
}

/// `(máx, p99.9)` de `|f|` nas células `pop`.
pub(crate) fn desvio_em(pop: &[usize], f: impl Fn(usize) -> f64) -> (f64, f64) {
    assert!(!pop.is_empty(), "população vazia: a régua seria muda");
    let mut d: Vec<f64> = pop.iter().map(|&i| f(i).abs()).collect();
    d.sort_by(f64::total_cmp);
    let n = d.len();
    (d[n - 1], d[((n as f64 * 0.999) as usize).min(n - 1)])
}

/// `(máx, p99.9)` de `|a − b|` no interior da folha.
pub(crate) fn desvio(g: &Grid, f: impl Fn(usize) -> f64) -> (f64, f64) {
    let mut d = Vec::with_capacity(g.w * g.h);
    for y in 1..=g.h {
        for x in 1..=g.w {
            d.push(f(x + y * g.s).abs());
        }
    }
    d.sort_by(f64::total_cmp);
    let n = d.len();
    (d[n - 1], d[(n as f64 * 0.999) as usize])
}

pub(crate) fn campos(g: &Grid, i: usize) -> [f32; 16] {
    let c: CelulaTinta = em(g, i);
    let m = g.oleo.get(i).map_or(0.0, |o| o.meio);
    [
        g.film[i],
        g.susp[i],
        g.sett[i],
        c.filme.v,
        c.filme.lab[0],
        c.filme.lab[1],
        c.filme.lab[2],
        c.filme.t,
        c.filme.m,
        c.aberto.v,
        c.aberto.lab[0],
        c.aberto.lab[1],
        c.aberto.lab[2],
        c.aberto.t,
        c.aberto.m,
        m,
    ]
}

pub(crate) fn total(g: &Grid, f: impl Fn(&CelulaTinta) -> f32) -> f64 {
    (0..g.cells).map(|i| f64::from(f(&em(g, i)))).sum()
}

/// Tudo o que a placa devolve de uma célula, ao bit: os campos da lei, a abertura, as máscaras e os espelhos sRGB. O `corpo` entra
/// embora a placa não o escreva (nem a CPU o move no passo: só o pouso): a régua pega quem passar a mexer nele de um lado só.
/// `r_g`, `r_g0`, `altura_fechada` e a aparência não entram: só a CPU os escreve, e a grade de partida é a mesma.
pub(crate) fn bits(g: &Grid, i: usize) -> [u32; 26] {
    let c = em(g, i);
    let mut b = [0u32; 26];
    for (o, v) in b.iter_mut().zip(campos(g, i)) {
        *o = v.to_bits();
    }
    b[16] = c.o.to_bits();
    b[17] = g.oleo.get(i).map_or(0.0, |o| o.corpo).to_bits();
    b[18] = u32::from(g.active[i]);
    b[19] = u32::from(g.wet[i]);
    for k in 0..3 {
        b[20 + k] = g.susp_rgb[i][k].to_bits();
        b[23 + k] = g.sett_rgb[i][k].to_bits();
    }
    b
}

fn dif_bit_em(
    a: &Grid,
    b: &Grid,
    fina: impl Fn(usize) -> bool,
    fl: impl Fn(usize) -> bool,
) -> usize {
    let finas = (0..a.cells)
        .filter(|&i| fina(i) && bits(a, i) != bits(b, i))
        .count();
    let igual = |x: f32, y: f32| x.to_bits() == y.to_bits();
    let fluxo = (0..a.flow.cells)
        .filter(|&i| {
            fl(i)
                && !(igual(a.vel_x[i], b.vel_x[i])
                    && igual(a.vel_y[i], b.vel_y[i])
                    && igual(a.flow_x[i], b.flow_x[i])
                    && igual(a.flow_y[i], b.flow_y[i]))
        })
        .count();
    finas + fluxo
}

/// Quantas células (finas + de fluxo) diferem ao bit entre `a` e `b`.
pub(crate) fn dif_bit(a: &Grid, b: &Grid) -> usize {
    dif_bit_em(a, b, |_| true, |_| true)
}

/// [`dif_bit`] sem as duas colunas de cada parede. `bordas` despacha a folha inteira e escreve nelas `wet = 1` e `vel = ±max_v`, e a
/// descida suja não as traz fora da caixa: é condição de parede DERIVADA, reescrita a cada passo na placa e na CPU, sem água naquelas
/// linhas para a ler. Medido 05/11: todo o resíduo das 6 cenas está nestas colunas.
pub(crate) fn dif_bit_sem_paredes(a: &Grid, b: &Grid) -> usize {
    let dentro = |i: usize, s: usize, w: usize| (2..w).contains(&(i % s));
    dif_bit_em(
        a,
        b,
        |i| dentro(i, a.s, a.w),
        |i| dentro(i, a.flow.s, a.flow.w),
    )
}
