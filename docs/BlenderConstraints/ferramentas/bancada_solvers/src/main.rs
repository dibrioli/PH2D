//! Bancada da W4: ezpz × fiksi sobre as MESMAS cenas, no MESMO processo, em blocos intercalados.
//! `cenas <dir>` grava as entradas (lidas também pelo oráculo FreeCAD); `mede <dir> <freecad.json> <saida.json>`.
//! Método: docs/DevOps/MEDIR_VELOCIDADE.md. Desenho e leitura: docs/BlenderConstraints/04_W4_cad.md.

use std::collections::HashMap;
use std::f64::consts::TAU;
use std::hint::black_box;
use std::time::Instant;

use serde::{Deserialize, Serialize};

const PASSOS: usize = 60;
const RAIO_ARRASTO: f64 = 40.0;
const RODADAS: usize = 7;
const PASSOS_COMPARADOS: [usize; 4] = [15, 30, 45, 60];
/// px: uma regra «cumpre» abaixo disto (a tela tem 1 px; 1e-4 é 4 ordens abaixo).
const TOL_PX: f64 = 1e-4;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "t", rename_all = "snake_case")]
enum Regra {
    H { a: usize, b: usize },
    V { a: usize, b: usize },
    Paralelo { a: usize, b: usize, c: usize, d: usize },
    Perpendicular { a: usize, b: usize, c: usize, d: usize },
    Distancia { a: usize, b: usize, d: f64 },
    Igual { a: usize, b: usize, c: usize, d: usize },
    SobreLinha { p: usize, a: usize, b: usize },
    Simetrico { p: usize, q: usize, a: usize, b: usize },
    Tangente { a: usize, b: usize, circulo: usize },
    Coincidente { a: usize, b: usize },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Circulo {
    centro: usize,
    raio: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Arrasto {
    ponto: usize,
    caminho: Vec<[f64; 2]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Cena {
    nome: String,
    pontos: Vec<[f64; 2]>,
    fixos: Vec<usize>,
    circulos: Vec<Circulo>,
    regras: Vec<Regra>,
    arrasto: Arrasto,
}

// ───────────────────────── cenas (entradas NOSSAS) ─────────────────────────

fn caminho(origem: [f64; 2]) -> Vec<[f64; 2]> {
    (1..=PASSOS)
        .map(|s| {
            let a = TAU * s as f64 / PASSOS as f64;
            [origem[0] + RAIO_ARRASTO * (a.cos() - 1.0), origem[1] + RAIO_ARRASTO * a.sin()]
        })
        .collect()
}

/// Perfil rectilíneo fechado com `dentes` dentes: H/V em todo segmento, larguras iguais, alturas cotadas a cada 3.º dente.
/// Sub-restringido (o caso normal enquanto se desenha). N = 4·dentes + 2.
fn escada(dentes: usize) -> Cena {
    let (w, b) = (20.0, 30.0);
    let mut p = Vec::new();
    for i in 0..dentes {
        let x0 = i as f64 * 2.0 * w;
        let h = 60.0 + 13.0 * ((i * 7) % 5) as f64;
        p.extend([[x0, 0.0], [x0, h], [x0 + w, h], [x0 + w, 0.0]]);
    }
    let xl = (dentes - 1) as f64 * 2.0 * w + w;
    p.extend([[xl, -b], [0.0, -b]]);
    let n = p.len();
    let mut regras = Vec::new();
    for k in 0..n {
        let (a, c) = (k, (k + 1) % n);
        if (p[a][1] - p[c][1]).abs() < 1e-9 {
            regras.push(Regra::H { a, b: c });
        } else {
            regras.push(Regra::V { a, b: c });
        }
    }
    for i in 1..dentes {
        regras.push(Regra::Igual { a: 1, b: 2, c: 4 * i + 1, d: 4 * i + 2 });
    }
    for i in (0..dentes).step_by(3) {
        let (a, c) = (4 * i, 4 * i + 1);
        regras.push(Regra::Distancia { a, b: c, d: dist(p[a], p[c]) });
    }
    let k = 4 * (dentes / 2) + 1;
    Cena {
        nome: format!("escada_{n}"),
        arrasto: Arrasto { ponto: k, caminho: caminho(p[k]) },
        pontos: p,
        fixos: vec![0],
        circulos: vec![],
        regras,
    }
}

/// Módulos de 10 pontos (rectângulo rodado, ponto sobre aresta, par simétrico, recta tangente a um círculo, coincidência),
/// encadeados por paralelismo e igualdade: arrastar um canto roda TODOS os módulos (o caso acoplado, o pior para decompor).
fn misto(modulos: usize) -> Cena {
    let th: f64 = 0.3;
    let (u, nv) = ([th.cos(), th.sin()], [-th.sin(), th.cos()]);
    let em = |o: [f64; 2], a: f64, b: f64| [o[0] + a * u[0] + b * nv[0], o[1] + a * u[1] + b * nv[1]];
    let h = 70.0;
    let mut p = Vec::new();
    let mut circulos = Vec::new();
    let mut regras = Vec::new();
    for j in 0..modulos {
        let o = [(j % 10) as f64 * 300.0, (j / 10) as f64 * 300.0];
        let w = 120.0 + 5.0 * (j % 7) as f64;
        let i0 = p.len();
        let p4 = em(o, 0.4 * w, 0.0);
        let p7 = em(o, 0.6 * w, -50.0);
        let p8 = em(o, -40.0, -60.0);
        p.extend([
            o,
            em(o, w, 0.0),
            em(o, w, h),
            em(o, 0.0, h),
            p4,
            em(o, w + 30.0, 20.0),
            em(o, w - 30.0, 20.0),
            p7,
            p8,
            p4,
        ]);
        let raio = dist_ponto_recta(p7, p8, p4);
        circulos.push(Circulo { centro: i0 + 7, raio });
        let q = |k: usize| i0 + k;
        regras.extend([
            Regra::Paralelo { a: q(0), b: q(1), c: q(3), d: q(2) },
            Regra::Perpendicular { a: q(0), b: q(1), c: q(1), d: q(2) },
            Regra::Paralelo { a: q(1), b: q(2), c: q(0), d: q(3) },
            Regra::Distancia { a: q(0), b: q(1), d: w },
            Regra::SobreLinha { p: q(4), a: q(0), b: q(1) },
            Regra::Simetrico { p: q(5), q: q(6), a: q(1), b: q(2) },
            Regra::Tangente { a: q(8), b: q(9), circulo: j },
            Regra::Coincidente { a: q(9), b: q(4) },
        ]);
        if j > 0 {
            let r = i0 - 10;
            regras.push(Regra::Paralelo { a: q(0), b: q(1), c: r, d: r + 1 });
            regras.push(Regra::Igual { a: q(1), b: q(2), c: r + 1, d: r + 2 });
        }
    }
    let k = 10 * (modulos / 2) + 2;
    let n = p.len();
    Cena {
        nome: format!("misto_{n}"),
        arrasto: Arrasto { ponto: k, caminho: caminho(p[k]) },
        pontos: p,
        fixos: vec![0],
        circulos,
        regras,
    }
}

fn cenas() -> Vec<Cena> {
    vec![escada(12), escada(50), escada(200), misto(5), misto(20), misto(80)]
}

// ───────────────────────── a régua de cumprimento (nossa, a mesma para todos) ─────────────────────────

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn cruz(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn ponto(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn dist_ponto_recta(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    cruz(sub(b, a), sub(p, a)).abs() / dist(a, b)
}

/// Violação de cada regra em px (ângulos: desvio do 1.º segmento ao longo do seu comprimento).
fn violacao(c: &Cena, x: &[[f64; 2]], r: &Regra) -> f64 {
    match *r {
        Regra::H { a, b } => (x[a][1] - x[b][1]).abs(),
        Regra::V { a, b } => (x[a][0] - x[b][0]).abs(),
        Regra::Paralelo { a, b, c: cc, d } => {
            let (u, v) = (sub(x[b], x[a]), sub(x[d], x[cc]));
            cruz(u, v).abs() / dist(x[cc], x[d])
        }
        Regra::Perpendicular { a, b, c: cc, d } => {
            let (u, v) = (sub(x[b], x[a]), sub(x[d], x[cc]));
            ponto(u, v).abs() / dist(x[cc], x[d])
        }
        Regra::Distancia { a, b, d } => (dist(x[a], x[b]) - d).abs(),
        Regra::Igual { a, b, c: cc, d } => (dist(x[a], x[b]) - dist(x[cc], x[d])).abs(),
        Regra::SobreLinha { p, a, b } => dist_ponto_recta(x[p], x[a], x[b]),
        Regra::Simetrico { p, q, a, b } => {
            let (d, l) = (sub(x[b], x[a]), dist(x[a], x[b]));
            let t = ponto(sub(x[p], x[a]), d) / (l * l);
            let pe = [x[a][0] + t * d[0], x[a][1] + t * d[1]];
            dist([2.0 * pe[0] - x[p][0], 2.0 * pe[1] - x[p][1]], x[q])
        }
        Regra::Tangente { a, b, circulo } => {
            let ci = &c.circulos[circulo];
            (dist_ponto_recta(x[ci.centro], x[a], x[b]) - ci.raio).abs()
        }
        Regra::Coincidente { a, b } => dist(x[a], x[b]),
    }
}

fn pior_violacao(c: &Cena, x: &[[f64; 2]], alvo: [f64; 2]) -> f64 {
    let fixos = c.fixos.iter().map(|&i| dist(x[i], c.pontos[i])).fold(0.0, f64::max);
    let arr = dist(x[c.arrasto.ponto], alvo);
    c.regras.iter().map(|r| violacao(c, x, r)).fold(fixos.max(arr), f64::max)
}

// ───────────────────────── as variantes ─────────────────────────

trait Variante {
    fn repoe(&mut self, c: &Cena);
    fn arrasta(&mut self, alvo: [f64; 2]);
    fn posicoes(&self) -> Vec<[f64; 2]>;
    fn falhas(&self) -> usize {
        0
    }
}

mod ez {
    use super::*;
    use kcl_ezpz::datatypes::inputs::{DatumCircle, DatumDistance, DatumLineSegment, DatumPoint};
    use kcl_ezpz::datatypes::AngleKind;
    use kcl_ezpz::{Config, Constraint as K, ConstraintRequest as R, IdGenerator};

    pub struct Ezpz {
        pts: Vec<DatumPoint>,
        reqs: Vec<R>,
        valores: Vec<f64>,
        arrastado: DatumPoint,
        falhas: usize,
    }

    fn seg(p: &[DatumPoint], a: usize, b: usize) -> DatumLineSegment {
        DatumLineSegment::new(p[a], p[b])
    }

    /// ezpz tem a tangência ORIENTADA: escolhe-se a ordem da recta que já cumpre na pose de partida.
    fn tangente_orientada(c: &Cena, a: usize, b: usize, ci: &Circulo) -> (usize, usize) {
        for (s, e) in [(a, b), (b, a)] {
            let mut ids = IdGenerator::default();
            let (ps, pe, pc) = (DatumPoint::new(&mut ids), DatumPoint::new(&mut ids), DatumPoint::new(&mut ids));
            let r = DatumDistance::new(ids.next_id());
            let circ = DatumCircle { center: pc, radius: r };
            let g = vec![
                (ps.id_x(), c.pontos[s][0]),
                (ps.id_y(), c.pontos[s][1]),
                (pe.id_x(), c.pontos[e][0]),
                (pe.id_y(), c.pontos[e][1]),
                (pc.id_x(), c.pontos[ci.centro][0]),
                (pc.id_y(), c.pontos[ci.centro][1]),
                (r.id, ci.raio),
            ];
            let mut q: Vec<R> = g.iter().map(|&(id, v)| R::highest_priority(K::Fixed(id, v))).collect();
            q.push(R::highest_priority(K::LineTangentToCircle(DatumLineSegment::new(ps, pe), circ)));
            if let Ok(o) = kcl_ezpz::solve(&q, g, Config::default())
                && o.is_satisfied()
            {
                return (s, e);
            }
        }
        panic!("tangente {a}-{b} não cumpre em nenhuma orientação na pose de partida");
    }

    impl Ezpz {
        pub fn novo(c: &Cena) -> Self {
            let mut ids = IdGenerator::default();
            let pts: Vec<DatumPoint> = c.pontos.iter().map(|_| DatumPoint::new(&mut ids)).collect();
            let circs: Vec<DatumCircle> = c
                .circulos
                .iter()
                .map(|ci| DatumCircle { center: pts[ci.centro], radius: DatumDistance::new(ids.next_id()) })
                .collect();
            let n = ids.next_id() as usize;
            let mut reqs = Vec::new();
            let hp = R::highest_priority;
            for &i in &c.fixos {
                reqs.push(hp(K::Fixed(pts[i].id_x(), c.pontos[i][0])));
                reqs.push(hp(K::Fixed(pts[i].id_y(), c.pontos[i][1])));
            }
            for (ci, dc) in c.circulos.iter().zip(&circs) {
                reqs.push(hp(K::Fixed(dc.radius.id, ci.raio)));
            }
            for r in &c.regras {
                let k = match *r {
                    Regra::H { a, b } => K::Horizontal(seg(&pts, a, b)),
                    Regra::V { a, b } => K::Vertical(seg(&pts, a, b)),
                    Regra::Paralelo { a, b, c: cc, d } => {
                        K::LinesAtAngle(seg(&pts, a, b), seg(&pts, cc, d), AngleKind::Parallel)
                    }
                    Regra::Perpendicular { a, b, c: cc, d } => {
                        K::LinesAtAngle(seg(&pts, a, b), seg(&pts, cc, d), AngleKind::Perpendicular)
                    }
                    Regra::Distancia { a, b, d } => K::Distance(pts[a], pts[b], d),
                    Regra::Igual { a, b, c: cc, d } => K::LinesEqualLength(seg(&pts, a, b), seg(&pts, cc, d)),
                    Regra::SobreLinha { p, a, b } => K::PointLineDistance(pts[p], seg(&pts, a, b), 0.0),
                    Regra::Simetrico { p, q, a, b } => K::Symmetric(seg(&pts, a, b), pts[p], pts[q]),
                    Regra::Tangente { a, b, circulo } => {
                        let (s, e) = tangente_orientada(c, a, b, &c.circulos[circulo]);
                        K::LineTangentToCircle(seg(&pts, s, e), circs[circulo])
                    }
                    Regra::Coincidente { a, b } => K::PointsCoincident(pts[a], pts[b]),
                };
                reqs.push(hp(k));
            }
            let arrastado = pts[c.arrasto.ponto];
            reqs.push(hp(K::Fixed(arrastado.id_x(), 0.0)));
            reqs.push(hp(K::Fixed(arrastado.id_y(), 0.0)));
            let mut s = Ezpz { pts, reqs, valores: vec![0.0; n], arrastado, falhas: 0 };
            for (ci, dc) in c.circulos.iter().zip(&circs) {
                s.valores[dc.radius.id as usize] = ci.raio;
            }
            s.repoe(c);
            s
        }

        /// O custo da análise de graus de liberdade (as cores do CAD) — só quando a ESTRUTURA muda.
        pub fn analise(&self) -> usize {
            let g: Vec<_> = self.valores.iter().enumerate().map(|(i, &v)| (i as u32, v)).collect();
            match kcl_ezpz::solve_analysis(&self.reqs_com(self.ponto_arrastado()), g, Config::default()) {
                Ok(o) => o.analysis.underconstrained().len(),
                Err(_) => usize::MAX,
            }
        }

        fn ponto_arrastado(&self) -> [f64; 2] {
            [self.valores[self.arrastado.id_x() as usize], self.valores[self.arrastado.id_y() as usize]]
        }

        fn reqs_com(&self, alvo: [f64; 2]) -> Vec<R> {
            let mut q = self.reqs.clone();
            let n = q.len();
            q[n - 2] = R::highest_priority(K::Fixed(self.arrastado.id_x(), alvo[0]));
            q[n - 1] = R::highest_priority(K::Fixed(self.arrastado.id_y(), alvo[1]));
            q
        }
    }

    impl Variante for Ezpz {
        fn repoe(&mut self, c: &Cena) {
            for (p, v) in self.pts.iter().zip(&c.pontos) {
                self.valores[p.id_x() as usize] = v[0];
                self.valores[p.id_y() as usize] = v[1];
            }
        }
        fn arrasta(&mut self, alvo: [f64; 2]) {
            let n = self.reqs.len();
            self.reqs[n - 2] = R::highest_priority(K::Fixed(self.arrastado.id_x(), alvo[0]));
            self.reqs[n - 1] = R::highest_priority(K::Fixed(self.arrastado.id_y(), alvo[1]));
            self.valores[self.arrastado.id_x() as usize] = alvo[0];
            self.valores[self.arrastado.id_y() as usize] = alvo[1];
            let g: Vec<_> = self.valores.iter().enumerate().map(|(i, &v)| (i as u32, v)).collect();
            match kcl_ezpz::solve(&self.reqs, g, Config::default()) {
                Ok(o) => self.valores.copy_from_slice(o.final_values()),
                Err(_) => self.falhas += 1,
            }
        }
        fn posicoes(&self) -> Vec<[f64; 2]> {
            self.pts.iter().map(|p| [self.valores[p.id_x() as usize], self.valores[p.id_y() as usize]]).collect()
        }
        fn falhas(&self) -> usize {
            self.falhas
        }
    }
}

mod fk {
    use super::*;
    use fiksi::{ElementHandle, SolvingOptions, System, constraints as k, elements as e};

    #[derive(Clone, Copy)]
    pub enum Modo {
        LmInteiro,
        LmPassagemUnica,
        LbfgsInteiro,
        LmSemPerturbar,
        /// até 10 chamadas por passo — o que um produto faria para tirar a fiksi da paragem cedo.
        Lm10x,
    }

    pub struct Fiksi {
        sys: System,
        pts: Vec<ElementHandle<e::Point>>,
        arrastado: ElementHandle<e::Point>,
        modo: Modo,
    }

    impl Fiksi {
        pub fn novo(c: &Cena, modo: Modo) -> Self {
            let mut s = System::new();
            let pts: Vec<_> = c.pontos.iter().map(|p| e::Point::create(&mut s, p[0], p[1])).collect();
            // H e V não existem na fiksi: paralelo/perpendicular a uma recta de referência fixa.
            let (r0, r1) = (e::Point::create(&mut s, 0.0, 0.0), e::Point::create(&mut s, 1.0, 0.0));
            r0.fix(&mut s);
            r1.fix(&mut s);
            let refh = e::Line::create(&mut s, r0, r1);
            let mut linhas: HashMap<(usize, usize), ElementHandle<e::Line>> = HashMap::new();
            let mut linha = |s: &mut System, a: usize, b: usize| {
                *linhas.entry((a, b)).or_insert_with(|| e::Line::create(s, pts[a], pts[b]))
            };
            let circs: Vec<_> = c
                .circulos
                .iter()
                .map(|ci| {
                    let r = e::Length::create(&mut s, ci.raio);
                    r.fix(&mut s);
                    e::Circle::create(&mut s, pts[ci.centro], r)
                })
                .collect();
            for &i in &c.fixos {
                pts[i].fix(&mut s);
            }
            for r in &c.regras {
                match *r {
                    Regra::H { a, b } => {
                        let l = linha(&mut s, a, b);
                        k::LineLineParallelism::create(&mut s, l, refh);
                    }
                    Regra::V { a, b } => {
                        let l = linha(&mut s, a, b);
                        k::LineLinePerpendicularity::create(&mut s, l, refh);
                    }
                    Regra::Paralelo { a, b, c: cc, d } => {
                        let (l1, l2) = (linha(&mut s, a, b), linha(&mut s, cc, d));
                        k::LineLineParallelism::create(&mut s, l1, l2);
                    }
                    Regra::Perpendicular { a, b, c: cc, d } => {
                        let (l1, l2) = (linha(&mut s, a, b), linha(&mut s, cc, d));
                        k::LineLinePerpendicularity::create(&mut s, l1, l2);
                    }
                    Regra::Distancia { a, b, d } => {
                        k::PointPointDistance::create(&mut s, pts[a], pts[b], d);
                    }
                    Regra::Igual { a, b, c: cc, d } => {
                        k::SegmentSegmentLengthEquality::create(&mut s, pts[a], pts[b], pts[cc], pts[d]);
                    }
                    Regra::SobreLinha { p, a, b } => {
                        let l = linha(&mut s, a, b);
                        k::PointLineIncidence::create(&mut s, pts[p], l);
                    }
                    // sem simétrico nativo: |PA| = |QA| e |PB| = |QB| (reflexão sobre AB) — composição medida.
                    Regra::Simetrico { p, q, a, b } => {
                        k::SegmentSegmentLengthEquality::create(&mut s, pts[p], pts[a], pts[q], pts[a]);
                        k::SegmentSegmentLengthEquality::create(&mut s, pts[p], pts[b], pts[q], pts[b]);
                    }
                    Regra::Tangente { a, b, circulo } => {
                        let l = linha(&mut s, a, b);
                        k::LineCircleTangency::create(&mut s, l, circs[circulo]);
                    }
                    Regra::Coincidente { a, b } => {
                        k::PointPointCoincidence::create(&mut s, pts[a], pts[b]);
                    }
                }
            }
            let arrastado = pts[c.arrasto.ponto];
            arrastado.fix(&mut s);
            Fiksi { sys: s, pts, arrastado, modo }
        }

        fn opcoes(&self) -> SolvingOptions {
            let (optimizer, decomposer) = match self.modo {
                Modo::LmInteiro => (fiksi::solve::Optimizer::LevenbergMarquardt, fiksi::Decomposer::None),
                Modo::LmPassagemUnica => (fiksi::solve::Optimizer::LevenbergMarquardt, fiksi::Decomposer::SinglePass),
                Modo::LbfgsInteiro => (fiksi::solve::Optimizer::LBfgs, fiksi::Decomposer::None),
                Modo::LmSemPerturbar | Modo::Lm10x => (fiksi::solve::Optimizer::LevenbergMarquardt, fiksi::Decomposer::None),
            };
            let perturb = !matches!(self.modo, Modo::LmSemPerturbar | Modo::Lm10x);
            SolvingOptions { optimizer, decomposer, perturb }
        }

        /// Sobre-restrição (a lista «apague uma destas») — experimental na fiksi; só quando a ESTRUTURA muda.
        pub fn analise(&mut self) -> usize {
            self.sys.analyze().overconstrained.len()
        }
    }

    impl Variante for Fiksi {
        fn repoe(&mut self, c: &Cena) {
            for (h, p) in self.pts.iter().zip(&c.pontos) {
                h.update_value(&mut self.sys, p[0], p[1]);
            }
        }
        fn arrasta(&mut self, alvo: [f64; 2]) {
            self.arrastado.update_value(&mut self.sys, alvo[0], alvo[1]);
            let n = if matches!(self.modo, Modo::Lm10x) { 10 } else { 1 };
            for _ in 0..n {
                let o = self.opcoes();
                self.sys.solve(o);
            }
        }
        fn posicoes(&self) -> Vec<[f64; 2]> {
            self.pts
                .iter()
                .map(|h| {
                    let p = h.get_value(&self.sys);
                    [p.x, p.y]
                })
                .collect()
        }
    }
}

// ───────────────────────── medição intercalada ─────────────────────────

const NOMES: [&str; 6] = ["ezpz", "fiksi_lm", "fiksi_lm_passagem_unica", "fiksi_lbfgs", "fiksi_lm_sem_perturbar", "fiksi_lm_10x"];

fn nova(i: usize, c: &Cena) -> Box<dyn Variante> {
    match i {
        0 => Box::new(ez::Ezpz::novo(c)),
        1 => Box::new(fk::Fiksi::novo(c, fk::Modo::LmInteiro)),
        2 => Box::new(fk::Fiksi::novo(c, fk::Modo::LmPassagemUnica)),
        3 => Box::new(fk::Fiksi::novo(c, fk::Modo::LbfgsInteiro)),
        4 => Box::new(fk::Fiksi::novo(c, fk::Modo::LmSemPerturbar)),
        _ => Box::new(fk::Fiksi::novo(c, fk::Modo::Lm10x)),
    }
}

#[derive(Serialize, Default)]
struct Bloco {
    pior_passo_ms: f64,
    media_passo_ms: f64,
    pior_violacao_px: f64,
    passos_fora_da_tolerancia: usize,
    falhas_declaradas: usize,
    maior_salto_px: f64,
    passos_feitos: usize,
    #[serde(skip)]
    amostras: Vec<Vec<[f64; 2]>>,
}

/// Variante cujo pior passo no aquecimento passa disto (100× o critério de 2 ms) mede-se numa rodada só.
const LENTA_MS: f64 = 200.0;
const LIMITE_BLOCO_S: f64 = 20.0;

/// Um bloco que passa de `limite_s` pára no passo em que está: a variante já está 100× fora do critério.
fn bloco(v: &mut dyn Variante, c: &Cena, limite_s: f64) -> Bloco {
    v.repoe(c);
    let f0 = v.falhas();
    let mut b = Bloco::default();
    let mut antes = c.pontos.clone();
    let mut soma = 0.0;
    for (s, &alvo) in c.arrasto.caminho.iter().enumerate() {
        if soma > limite_s * 1e3 {
            break;
        }
        b.passos_feitos = s + 1;
        let t = Instant::now();
        v.arrasta(black_box(alvo));
        let ms = t.elapsed().as_secs_f64() * 1e3;
        soma += ms;
        b.pior_passo_ms = b.pior_passo_ms.max(ms);
        let x = v.posicoes();
        let pv = pior_violacao(c, &x, alvo);
        b.pior_violacao_px = b.pior_violacao_px.max(pv);
        if !(pv < TOL_PX) {
            b.passos_fora_da_tolerancia += 1;
        }
        let salto = x
            .iter()
            .zip(&antes)
            .enumerate()
            .filter(|(i, _)| *i != c.arrasto.ponto)
            .map(|(_, (a, z))| dist(*a, *z))
            .fold(0.0, f64::max);
        b.maior_salto_px = b.maior_salto_px.max(salto);
        if PASSOS_COMPARADOS.contains(&(s + 1)) {
            b.amostras.push(x.clone());
        }
        antes = x;
    }
    b.media_passo_ms = soma / b.passos_feitos as f64;
    b.falhas_declaradas = v.falhas() - f0;
    b
}

fn minimo(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}
fn mediana(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    s[s.len() / 2]
}

fn min_de<F: FnMut() -> f64>(n: usize, mut f: F) -> f64 {
    (0..n).map(|_| f()).fold(f64::INFINITY, f64::min)
}

#[derive(Serialize)]
struct LinhaVariante {
    variante: &'static str,
    rodadas_medidas: usize,
    passos_feitos_min: usize,
    pior_passo_ms_min: f64,
    pior_passo_ms_mediana: f64,
    media_passo_ms_min: f64,
    media_passo_ms_mediana: f64,
    pior_violacao_px: f64,
    passos_fora_da_tolerancia_max: usize,
    falhas_declaradas_max: usize,
    maior_salto_px: f64,
    determinista: bool,
    estrutura_ms_min: f64,
    analise_ms_min: Option<f64>,
    analise_resultado: Option<usize>,
    desvio_do_freecad_px: Option<f64>,
}

#[derive(Serialize)]
struct LinhaCena {
    cena: String,
    pontos: usize,
    regras: usize,
    freecad_pior_passo_ms: Option<f64>,
    freecad_pior_violacao_px: Option<f64>,
    variantes: Vec<LinhaVariante>,
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim().to_string()
}

/// Uma cena por processo (um solver que pendura só perde a sua cena); `W4_SEM=a,b` tira variantes.
fn mede(dir: &str, freecad: &str, saida_dir: &str, so: &str) {
    let sem = std::env::var("W4_SEM").unwrap_or_default();
    let ativas: Vec<usize> = (0..NOMES.len()).filter(|&i| !sem.split(',').any(|x| x == NOMES[i])).collect();
    let fc: serde_json::Value = std::fs::read_to_string(freecad)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    let la0 = loadavg();
    let mut linhas = Vec::new();
    for nome in cenas().iter().map(|c| c.nome.clone()).filter(|n| n == so) {
        let c: Cena = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/{nome}.json")).unwrap()).unwrap();
        let mut vs: Vec<Box<dyn Variante>> = ativas.iter().map(|&i| nova(i, &c)).collect();
        let lenta: Vec<bool> = vs
            .iter_mut()
            .enumerate()
            .map(|(j, v)| {
                let b = bloco(v.as_mut(), &c, LIMITE_BLOCO_S); // aquecimento, fora da régua
                eprintln!("{nome} · {} · aquecimento: pior passo {:.3} ms, {} passos", NOMES[ativas[j]], b.pior_passo_ms, b.passos_feitos);
                b.pior_passo_ms > LENTA_MS
            })
            .collect();
        let nv = ativas.len();
        let mut rodadas: Vec<Vec<Bloco>> = (0..nv).map(|_| Vec::new()).collect();
        for r in 0..RODADAS {
            for k in 0..nv {
                let i = (k + r) % nv;
                if lenta[i] && r > 0 {
                    continue;
                }
                rodadas[i].push(bloco(vs[i].as_mut(), &c, LIMITE_BLOCO_S));
            }
        }
        eprintln!("{nome} · medido · loadavg {}", loadavg());
        let fc_cena = &fc["cenas"][&nome];
        let fc_amostras: Option<Vec<Vec<[f64; 2]>>> = serde_json::from_value(fc_cena["amostras"].clone()).ok();
        let mut variantes = Vec::new();
        for (j, rs) in rodadas.iter().enumerate() {
            let i = ativas[j];
            let pior: Vec<f64> = rs.iter().map(|b| b.pior_passo_ms).collect();
            let media: Vec<f64> = rs.iter().map(|b| b.media_passo_ms).collect();
            let determinista = rs.windows(2).all(|w| w[0].amostras == w[1].amostras);
            let reps = if lenta[j] { 1 } else { 3 };
            let estrutura_ms_min = min_de(reps, || {
                let t = Instant::now();
                let mut v = nova(i, &c);
                v.arrasta(c.pontos[c.arrasto.ponto]);
                black_box(v.posicoes());
                t.elapsed().as_secs_f64() * 1e3
            });
            let (analise_ms_min, analise_resultado) = match i {
                0 => {
                    let v = ez::Ezpz::novo(&c);
                    let mut res = 0;
                    let ms = min_de(reps, || {
                        let t = Instant::now();
                        res = black_box(v.analise());
                        t.elapsed().as_secs_f64() * 1e3
                    });
                    (Some(ms), Some(res))
                }
                1 => {
                    let mut res = 0;
                    let ms = min_de(reps, || {
                        let mut v = fk::Fiksi::novo(&c, fk::Modo::LmInteiro);
                        let t = Instant::now();
                        res = black_box(v.analise());
                        t.elapsed().as_secs_f64() * 1e3
                    });
                    (Some(ms), Some(res))
                }
                _ => (None, None),
            };
            let desvio_do_freecad_px = fc_amostras.as_ref().map(|fa| {
                fa.iter()
                    .zip(&rs[0].amostras)
                    .flat_map(|(a, b)| a.iter().zip(b).map(|(p, q)| dist(*p, *q)))
                    .fold(0.0, f64::max)
            });
            variantes.push(LinhaVariante {
                variante: NOMES[i],
                rodadas_medidas: rs.len(),
                passos_feitos_min: rs.iter().map(|b| b.passos_feitos).min().unwrap(),
                pior_passo_ms_min: minimo(&pior),
                pior_passo_ms_mediana: mediana(&pior),
                media_passo_ms_min: minimo(&media),
                media_passo_ms_mediana: mediana(&media),
                pior_violacao_px: rs.iter().map(|b| b.pior_violacao_px).fold(0.0, f64::max),
                passos_fora_da_tolerancia_max: rs.iter().map(|b| b.passos_fora_da_tolerancia).max().unwrap(),
                falhas_declaradas_max: rs.iter().map(|b| b.falhas_declaradas).max().unwrap(),
                maior_salto_px: rs.iter().map(|b| b.maior_salto_px).fold(0.0, f64::max),
                determinista,
                estrutura_ms_min,
                analise_ms_min,
                analise_resultado,
                desvio_do_freecad_px,
            });
        }
        linhas.push(LinhaCena {
            cena: nome.clone(),
            pontos: c.pontos.len(),
            regras: c.regras.len(),
            freecad_pior_passo_ms: fc_cena["pior_passo_ms"].as_f64(),
            freecad_pior_violacao_px: fc_cena["pior_violacao_px"].as_f64(),
            variantes,
        });
    }
    let la1 = loadavg();
    println!("loadavg início: {la0} · fim: {la1}");
    println!(
        "| cena | variante | pior passo ms (mín · med) | média ms (mín · med) | viol. px | fora tol | salto px | determ. | estrutura ms | análise ms | Δ FreeCAD px |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|");
    for l in &linhas {
        for v in &l.variantes {
            println!(
                "| {} ({} pts, {} regras) | {} ({}×, {} passos) | {:.3} · {:.3} | {:.3} · {:.3} | {:.1e} | {} | {:.2} | {} | {:.2} | {} | {} |",
                l.cena,
                l.pontos,
                l.regras,
                v.variante,
                v.rodadas_medidas,
                v.passos_feitos_min,
                v.pior_passo_ms_min,
                v.pior_passo_ms_mediana,
                v.media_passo_ms_min,
                v.media_passo_ms_mediana,
                v.pior_violacao_px,
                v.passos_fora_da_tolerancia_max,
                v.maior_salto_px,
                v.determinista,
                v.estrutura_ms_min,
                v.analise_ms_min.map(|x| format!("{x:.2} ({})", v.analise_resultado.unwrap())).unwrap_or("—".into()),
                v.desvio_do_freecad_px.map(|x| format!("{x:.3}")).unwrap_or("—".into()),
            );
        }
        if let Some(ms) = l.freecad_pior_passo_ms {
            println!(
                "| {} | FreeCAD 1.1.4 (referência) | {ms:.3} | — | {:.1e} | — | — | — | — | — | 0 |",
                l.cena,
                l.freecad_pior_violacao_px.unwrap_or(f64::NAN)
            );
        }
    }
    let out = serde_json::json!({
        "cabecalho": {
            "o_que": "W4: ezpz × fiksi, arrasto de 60 passos (círculo de 40 px), 7 rodadas intercaladas, resumo = mínimo (mediana ao lado)",
            "ezpz": "kcl-ezpz 0.2.18 (crates.io)",
            "fiksi": "endoli/fiksi @ ff4850546738fb82e95ff27c4db63e924d938473",
            "perfil": "release = cópia do smoke do workspace (codegen-units 16, sem LTO)",
            "tolerancia_px": TOL_PX,
            "loadavg_inicio": la0,
            "loadavg_fim": la1,
            "gerador": "docs/BlenderConstraints/ferramentas/bancada_solvers",
        },
        "cenas": linhas,
    });
    std::fs::write(format!("{saida_dir}/{so}.json"), serde_json::to_string_pretty(&out).unwrap()).unwrap();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("cenas") => {
            std::fs::create_dir_all(&a[2]).unwrap();
            for c in cenas() {
                let x: Vec<[f64; 2]> = c.pontos.clone();
                let v = c.regras.iter().map(|r| violacao(&c, &x, r)).fold(0.0, f64::max);
                assert!(v < 1e-9, "{}: a pose de partida viola uma regra ({v})", c.nome);
                std::fs::write(format!("{}/{}.json", a[2], c.nome), serde_json::to_string_pretty(&c).unwrap()).unwrap();
                println!("{}: {} pontos, {} regras", c.nome, c.pontos.len(), c.regras.len());
            }
        }
        Some("mede") => mede(&a[2], &a[3], &a[4], &a[5]),
        // diag <dir> <cena> <variante>: violação na pose de partida (sem arrasto e com o ponto arrastado onde está),
        // depois 1 passo, com as 5 regras piores — separa «o adaptador traduz mal» de «o solver não converge».
        Some("diag") => {
            let c: Cena = serde_json::from_str(&std::fs::read_to_string(format!("{}/{}.json", a[2], a[3])).unwrap()).unwrap();
            let i = NOMES.iter().position(|n| *n == a[4]).unwrap();
            let mut v = nova(i, &c);
            let parado = c.pontos[c.arrasto.ponto];
            for (rotulo, alvo) in [("sem mexer", parado), ("1 passo", c.arrasto.caminho[0]), ("2.º passo", c.arrasto.caminho[1])] {
                v.arrasta(alvo);
                let x = v.posicoes();
                let mut vs: Vec<(f64, String)> = c.regras.iter().map(|r| (violacao(&c, &x, r), format!("{r:?}"))).collect();
                vs.sort_by(|p, q| q.0.total_cmp(&p.0));
                println!("{rotulo}: pior {:.3e} px · arrastado a {:.3e} px do alvo", vs[0].0, dist(x[c.arrasto.ponto], alvo));
                for (e, r) in vs.iter().take(5) {
                    println!("   {e:.3e}  {r}");
                }
            }
        }
        _ => eprintln!("uso: cenas <dir> | mede <dir> <freecad.json> <saida_dir> <cena>"),
    }
}
