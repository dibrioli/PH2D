//! Medida (report do dono, `=6` a `170,170`: *«a parte externa da junta … sem permanecer
//! arredondada»*). Na aresta de FORA da junta 1 da barra 0 (em repouso a recta `y = −0,975`), por
//! ponto: os pesos, o centro da mistura, o ângulo médio, o raio à junta antes e depois — e o raio de
//! curvatura mínimo da aresta posta com a lei do produto, a desdobrada, a linear e pesos sintéticos.

use crate::smoke_bone_copias::build;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton::{MisturaDoAngulo, Skin};
use ph2d_vec_scene::VecScene;

type P = [f64; 2];

fn cena(
    g1: f32,
    g2: f32,
) -> (
    SimWorld,
    crate::state::VecState,
    Vec<ph2d_vec_scene::VecPathId>,
) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
    let pend = st.bone_smoke_pend.take().expect("pendentes");
    for (id, raiz) in &pend {
        ph2d_skeleton_live::skin_live::bind(&mut sim, &mut scene, &st.entities, &[*id], *raiz);
        crate::smoke_bone_par::dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
    }
    let ids = pend.iter().map(|p| p.0).collect();
    (sim, st, ids)
}

struct Lado {
    pele: Skin,
    prep: std::rc::Rc<ph2d_skeleton_live::skin_desenho::Preparado>,
    cor: Vec<ph2d_skeleton::Correccao>,
}

impl Lado {
    fn de(sim: &SimWorld, st: &crate::state::VecState, id: ph2d_vec_scene::VecPathId) -> Self {
        let e = st
            .entities
            .get(&id)
            .and_then(|b| Entity::try_from_bits(*b))
            .expect("entidade");
        let bind = sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e)
            .expect("bind")
            .clone();
        Self {
            pele: ph2d_skeleton_live::skin_live::skin_of(sim, e).expect("pele"),
            prep: ph2d_skeleton_live::skin_desenho::lida(e.to_bits(), &bind).expect("fonte"),
            cor: bind.correcoes_resolvidas(),
        }
    }
    fn pesos(&self, p: P) -> Vec<f64> {
        let campo = self.prep.guardado.campo.as_ref().expect("campo");
        let mut q = p;
        let linha = loop {
            if let Some(l) = campo.linha_com(q, self.prep.indice.as_ref()) {
                break l;
            }
            q[1] += if p[1] < -0.6 { 1e-4 } else { -1e-4 };
        };
        let mut w = self.pele.scratch();
        self.pele
            .weights_corrected(p, Some(&linha), &mut w, &self.cor);
        w
    }
}

fn circ(pele: &Skin, w: &[f64]) -> f64 {
    let (mut x, mut y) = (0.0, 0.0);
    for (b, &k) in pele.bones().iter().zip(w) {
        let t = b.angulo_da_pose();
        x += k * t.cos();
        y += k * t.sin();
    }
    y.atan2(x)
}

/// O menor raio de curvatura de uma polilinha (reamostrada a `H` de arco, viragem sobre `5` passos)
/// e o índice da amostra ORIGINAL mais próxima do mínimo.
fn raio_min(pl: &[P]) -> (f64, usize) {
    const H: f64 = 0.002;
    let mut r: Vec<(P, usize)> = vec![(pl[0], 0)];
    let mut falta = H;
    for (i, w) in pl.windows(2).enumerate() {
        let (mut a, b) = (w[0], w[1]);
        let mut seg = (b[0] - a[0]).hypot(b[1] - a[1]);
        while seg >= falta {
            let f = falta / seg;
            a = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
            r.push((a, i));
            seg -= falta;
            falta = H;
        }
        falta -= seg;
    }
    let mut melhor = (f64::MAX, 0);
    for i in 5..r.len().saturating_sub(5) {
        let (p0, p1, p2) = (r[i - 5].0, r[i].0, r[i + 5].0);
        let (u, v) = (
            [p1[0] - p0[0], p1[1] - p0[1]],
            [p2[0] - p1[0], p2[1] - p1[1]],
        );
        let ang = (u[0] * v[1] - u[1] * v[0])
            .atan2(u[0] * v[0] + u[1] * v[1])
            .abs();
        if ang > 1e-9 && 5.0 * H / ang < melhor.0 {
            melhor = (5.0 * H / ang, r[i].1);
        }
    }
    melhor
}

fn cruzamentos(pl: &[P]) -> usize {
    let x = |a: P, b: P, c: P, d: P| {
        let o = |p: P, q: P, r: P| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
        o(a, b, c) * o(a, b, d) < 0.0 && o(c, d, a) * o(c, d, b) < 0.0
    };
    let mut n = 0;
    for i in 0..pl.len() - 1 {
        for j in i + 2..pl.len() - 1 {
            if x(pl[i], pl[i + 1], pl[j], pl[j + 1]) {
                n += 1;
            }
        }
    }
    n
}

fn dist_min(p: P, pl: &[P]) -> f64 {
    pl.iter()
        .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(f64::MAX, f64::min)
}

fn suave(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Uma junta da barra 0: `(x da junta, x do início do osso PAI em repouso, y de FORA, y de DENTRO)`.
const JUNTAS: [(f64, f64, f64, f64); 2] = [
    (-3.225, -4.475, -0.975, -0.225),
    (-1.975, -3.225, -0.225, -0.975),
];

/// ⭐ **SONDA — o MECANISMO do bico da tampa de fora.** `cargo test --release -p ph2d-app-vec
/// --lib diag_o_mecanismo_da_tampa -- --ignored --nocapture`.
#[test]
#[ignore = "sonda: imprime; corra em --release"]
fn diag_o_mecanismo_da_tampa() {
    let n: usize = 1201;
    let mut casos: Vec<(f32, f32, usize)> = Vec::new();
    for g in [110.0_f32, 140.0, 160.0, 170.0, 175.0] {
        casos.push((g, 110.0, 0));
    }
    for g in [110.0_f32, 140.0, 160.0, 170.0, 175.0] {
        casos.push((110.0, g, 1));
    }
    casos.push((170.0, 170.0, 0));
    casos.push((170.0, 170.0, 1));
    for (g1, g2, ji) in casos {
        let (xj, xpai, yf, yd) = JUNTAS[ji];
        let j = [xj, -0.6];
        let xs: Vec<f64> = (0..n)
            .map(|i| xj - 0.6 + 1.2 * (i as f64) / ((n - 1) as f64))
            .collect();
        let (sim, st, ids) = cena(g1, g2);
        let l = Lado::de(&sim, &st, ids[0]);
        let pele = &l.pele;
        let ip = pele
            .bones()
            .iter()
            .position(|b| (b.rest_a[0] - xpai).abs() < 1e-3)
            .expect("pai");
        let ic = pele
            .bones()
            .iter()
            .position(|b| (b.rest_a[0] - xj).abs() < 1e-3)
            .expect("filho");
        let jp = pele.bones()[ip].pose.apply(j);
        let fora: Vec<P> = xs.iter().map(|&x| [x, yf]).collect();
        let dentro: Vec<P> = xs.iter().map(|&x| [x, yd]).collect();
        let ws: Vec<Vec<f64>> = fora.iter().map(|p| l.pesos(*p)).collect();
        let wd: Vec<Vec<f64>> = dentro.iter().map(|p| l.pesos(*p)).collect();
        let tp = pele.bones()[ip].angulo_da_pose();
        let g = (pele.bones()[ic].angulo_da_pose() - tp + std::f64::consts::PI)
            .rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        let faixa = |f: &dyn Fn(usize) -> f64, lo: f64, hi: f64| {
            let a = (0..n)
                .find(|&i| f(i) >= lo)
                .map_or(f64::NAN, |i| xs[i] - xj);
            let b = (0..n)
                .find(|&i| f(i) >= hi)
                .map_or(f64::NAN, |i| xs[i] - xj);
            (a, b)
        };
        let wc = |i: usize| ws[i][ic];
        let (a5, b95) = faixa(&wc, 0.05, 0.95);
        let w50 = faixa(&wc, 0.5, 0.5).0;
        let i50 = (0..n).find(|&i| wc(i) >= 0.5).unwrap_or(n / 2);
        let dw = (wc(i50 + 5) - wc(i50 - 5)) / (xs[i50 + 5] - xs[i50 - 5]);
        let th = |i: usize| {
            let d = circ(pele, &ws[i]) - tp;
            ((d + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI)
                / g
        };
        let (t10, t90) = faixa(&th, 0.10, 0.90);
        let outros = (0..n)
            .map(|i| 1.0 - ws[i][ip] - ws[i][ic])
            .fold(0.0, f64::max);
        let (mut cmax, mut dr) = (0.0_f64, 0.0_f64);
        for (p, w) in fora.iter().zip(&ws) {
            if let Some(c) = pele.centro_de_rotacao(w) {
                cmax = cmax.max((c[0] - j[0]).hypot(c[1] - j[1]));
            }
            let q = pele.blend(*p, w);
            dr = dr
                .max(((q[0] - jp[0]).hypot(q[1] - jp[1]) - (p[0] - j[0]).hypot(p[1] - j[1])).abs());
        }
        println!(
            "{g1}/{g2} junta {} (dobra {:.1}°): FORA — filho 5–95 % x−J [{a5:+.3},{b95:+.3}] (largura {:.3}; 50 % em {w50:+.3}; dw/dx {dw:.2}) · \
             θ̄ 10–90 % [{t10:+.3},{t90:+.3}] (largura {:.3}; previsão 2·tan(g/2)·dw/dx = {:.1} rad/u) · terceiro osso ≤ {outros:.4} · |c−J| ≤ {cmax:.1e} · \
             ||p'−J'|−|p−J|| ≤ {dr:.1e}",
            ji + 1,
            g.to_degrees(),
            b95 - a5,
            t90 - t10,
            2.0 * (g.abs() / 2.0).tan() * dw
        );
        // O núcleo: |x − J| ≤ 0,3.
        let nucleo: Vec<usize> = (0..n).filter(|&i| (xs[i] - xj).abs() <= 0.3).collect();
        let (k0, k1) = (nucleo[0], *nucleo.last().expect("núcleo") + 1);
        type Lei<'a> = (&'a str, Box<dyn Fn(P, &[f64]) -> P + 'a>);
        let leis: Vec<Lei> = vec![
            (
                "circulo (produto)",
                Box::new(|p, w| pele.blend_com(p, w, MisturaDoAngulo::Circulo)),
            ),
            (
                "desdobrado",
                Box::new(|p, w| pele.blend_com(p, w, MisturaDoAngulo::Desdobrado)),
            ),
            ("linear (LBS)", Box::new(|p, w| pele.blend_linear(p, w))),
        ];
        for (nome, f) in &leis {
            let pf: Vec<P> = fora.iter().zip(&ws).map(|(p, w)| f(*p, w)).collect();
            let pd: Vec<P> = dentro.iter().zip(&wd).map(|(p, w)| f(*p, w)).collect();
            let (rf, i) = raio_min(&pf[k0..k1]);
            let i = i + k0;
            let (rj, _) = raio_min(&pf);
            let esp = pf
                .iter()
                .map(|p| dist_min(*p, &pd))
                .fold(f64::MAX, f64::min);
            println!(
                "    {nome:>18}: raio mín FORA (núcleo ±0,3) {rf:.4} em x−J {:+.3} (filho {:.3}, |p'−J'| {:.3}) · janela ±0,6 {rj:.4} · \
                 DENTRO cruzamentos {} · espessura mín {esp:.3}",
                xs[i] - xj,
                wc(i),
                (pf[i][0] - jp[0]).hypot(pf[i][1] - jp[1]),
                cruzamentos(&pd)
            );
        }
        let mut linha = String::from("    sintético (meia-largura a):");
        for a in [0.05_f64, 0.1875, 0.375, 0.75] {
            let sint = |p: P| {
                let t = suave((p[0] - xj + a) / (2.0 * a));
                let mut w = vec![0.0; pele.len()];
                w[ip] = 1.0 - t;
                w[ic] = t;
                w
            };
            linha += &format!(" a={a}:");
            for (nome, lei) in [
                ("circ", MisturaDoAngulo::Circulo),
                ("desd", MisturaDoAngulo::Desdobrado),
            ] {
                let pf: Vec<P> = fora[k0..k1]
                    .iter()
                    .map(|p| pele.blend_com(*p, &sint(*p), lei))
                    .collect();
                let pd: Vec<P> = dentro
                    .iter()
                    .map(|p| pele.blend_com(*p, &sint(*p), lei))
                    .collect();
                let esp = pf
                    .iter()
                    .map(|p| dist_min(*p, &pd))
                    .fold(f64::MAX, f64::min);
                linha += &format!(
                    " {nome} {:.3}/esp {esp:.3}/cruz {}",
                    raio_min(&pf).0,
                    cruzamentos(&pd)
                );
            }
            linha += " ·";
        }
        println!("{linha}");
        if std::env::var("SONDA_PERFIL").is_ok() && ji == 1 && (g1, g2) == (170.0, 170.0) {
            for i in (k0..k1).step_by(10) {
                let q = pele.blend(fora[i], &ws[i]);
                println!(
                    "      x−J {:+.3} filho {:.3} θ̄ {:.1}° |p'−J'| {:.3} |p−J| {:.3} ang {:.1}°",
                    xs[i] - xj,
                    wc(i),
                    th(i) * g.to_degrees(),
                    (q[0] - jp[0]).hypot(q[1] - jp[1]),
                    (fora[i][0] - j[0]).hypot(fora[i][1] - j[1]),
                    (q[1] - jp[1]).atan2(q[0] - jp[0]).to_degrees()
                );
            }
        }
    }
}

/// ⭐ **SONDA — as CÓPIAS e a distância de cada contorno à junta.** Por contorno fechado da fonte
/// presa (as cópias do *Repeater* são contornos), por junta: a menor distância em repouso do
/// contorno à junta `d`, e o menor raio de curvatura do contorno POSTO (`point_corrected`) na zona
/// que em repouso está a menos de `d + 0,25` da junta.
#[test]
#[ignore = "sonda: imprime; corra em --release"]
fn diag_as_copias_e_a_distancia_a_junta() {
    for (g1, g2) in [
        (110.0_f32, 110.0_f32),
        (140.0, 140.0),
        (170.0, 170.0),
        (175.0, 175.0),
    ] {
        let (sim, st, ids) = cena(g1, g2);
        let l = Lado::de(&sim, &st, ids[0]);
        let mut f = l.prep.guardado.path.clone();
        f.subpaths.retain(|c| c.closed);
        let pls = crate::smoke_bone_copias::sondas::polilinhas(&f, 512);
        if (g1, g2) == (110.0, 110.0) {
            for (k, pl) in pls.iter().enumerate() {
                let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
                for p in pl {
                    lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                    hi = [hi[0].max(p[0]), hi[1].max(p[1])];
                }
                println!(
                    "  contorno {k}: {} pontos, caixa {lo:.3?}–{hi:.3?}",
                    pl.len()
                );
            }
        }
        for (ji, (xj, ..)) in JUNTAS.iter().enumerate() {
            let j = [*xj, -0.6];
            for (k, pl) in pls.iter().enumerate() {
                let d = pl
                    .iter()
                    .map(|p| (p[0] - j[0]).hypot(p[1] - j[1]))
                    .fold(f64::MAX, f64::min);
                // As corridas do contorno perto da junta.
                let mut melhor = f64::MAX;
                let mut corrida: Vec<P> = Vec::new();
                for p in pl.iter().chain(std::iter::once(&[f64::NAN; 2])) {
                    if !p[0].is_nan() && (p[0] - j[0]).hypot(p[1] - j[1]) <= d + 0.25 {
                        let w = l.pesos(*p);
                        corrida.push(l.pele.blend(*p, &w));
                        continue;
                    }
                    if corrida.len() > 12 {
                        melhor = melhor.min(raio_min(&corrida).0);
                    }
                    corrida.clear();
                }
                println!(
                    "{g1}/{g2} junta {} contorno {k}: d (repouso, à junta) {d:.3} · raio mín posto {melhor:.3} · razão {:.2}",
                    ji + 1,
                    melhor / d.max(1e-9)
                );
            }
        }
    }
}

/// ⭐ O RAIO DA TAMPA, robusto a quinas pequenas fora dela: o arco onde a curva faz os 80 % do meio
/// da sua viragem total, a dividir por essa viragem. Um arco de raio `R` dá `R`; um bico dá `≈ 0`.
fn raio_da_tampa(pl: &[P]) -> (f64, f64) {
    const H: f64 = 0.002;
    let mut r: Vec<P> = vec![pl[0]];
    let mut falta = H;
    for w in pl.windows(2) {
        let (mut a, b) = (w[0], w[1]);
        let mut seg = (b[0] - a[0]).hypot(b[1] - a[1]);
        while seg >= falta {
            let f = falta / seg;
            a = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
            r.push(a);
            seg -= falta;
            falta = H;
        }
        falta -= seg;
    }
    let mut acc = vec![0.0_f64];
    for i in 1..r.len() - 1 {
        let (u, v) = (
            [r[i][0] - r[i - 1][0], r[i][1] - r[i - 1][1]],
            [r[i + 1][0] - r[i][0], r[i + 1][1] - r[i][1]],
        );
        let a = (u[0] * v[1] - u[1] * v[0]).atan2(u[0] * v[0] + u[1] * v[1]);
        acc.push(acc[i - 1] + a);
    }
    let tot = *acc.last().expect("viragem");
    let s = |q: f64| {
        acc.iter()
            .position(|&x| x / tot >= q)
            .unwrap_or(acc.len() - 1) as f64
            * H
    };
    (
        (s(0.9) - s(0.1)) / (0.8 * tot.abs()).max(1e-12),
        tot.to_degrees(),
    )
}

/// ⭐ **SONDA — as LEIS sobre as duas arestas de fora.** Por junta: a aresta de FORA da barra
/// (contorno 0, `|x − J| ≤ 0,6`) e a aresta da CÓPIA que passa perto da junta (contorno 1), postas pela
/// mistura em círculo (produto), desdobrada, linear, e por pesos SINTÉTICOS de dois ossos (suave em
/// `x − J`, meia-largura `a`). Imprime o [`raio_da_tampa`] (e a viragem).
#[test]
#[ignore = "sonda: imprime; corra em --release"]
fn diag_as_leis_na_aresta_da_copia() {
    for g in [110.0_f32, 140.0, 160.0, 170.0, 175.0] {
        let (sim, st, ids) = cena(g, g);
        let l = Lado::de(&sim, &st, ids[0]);
        let pele = &l.pele;
        let mut f = l.prep.guardado.path.clone();
        f.subpaths.retain(|c| c.closed);
        let pls = crate::smoke_bone_copias::sondas::polilinhas(&f, 512);
        for (ji, (xj, xpai, yf, _)) in JUNTAS.iter().enumerate() {
            let j = [*xj, -0.6];
            let ip = pele
                .bones()
                .iter()
                .position(|b| (b.rest_a[0] - xpai).abs() < 1e-3)
                .expect("pai");
            let ic = pele
                .bones()
                .iter()
                .position(|b| (b.rest_a[0] - xj).abs() < 1e-3)
                .expect("filho");
            let pl = &pls[1];
            let d = pl
                .iter()
                .map(|p| (p[0] - j[0]).hypot(p[1] - j[1]))
                .fold(f64::MAX, f64::min);
            let mut copia: Vec<P> = pl
                .iter()
                .filter(|p| (p[0] - j[0]).abs() <= 0.6 && (p[1] - j[1]).abs() < 0.3)
                .copied()
                .collect();
            copia.sort_by(|a, b| a[0].total_cmp(&b[0]));
            let barra: Vec<P> = (0..=600)
                .map(|i| [xj - 0.6 + 0.002 * f64::from(i), *yf])
                .collect();
            for (rot, aresta) in [("barra (d 0,375)", &barra), ("cópia", &copia)] {
                let ws: Vec<Vec<f64>> = aresta.iter().map(|p| l.pesos(*p)).collect();
                let mut linha = format!("{g}/{g} junta {} {rot} (d {d:.3}):", ji + 1);
                for (nome, lei) in [
                    ("circ", MisturaDoAngulo::Circulo),
                    ("desd", MisturaDoAngulo::Desdobrado),
                ] {
                    let pf: Vec<P> = aresta
                        .iter()
                        .zip(&ws)
                        .map(|(p, w)| pele.blend_com(*p, w, lei))
                        .collect();
                    let (r, v) = raio_da_tampa(&pf);
                    linha += &format!(" {nome} {r:.3} ({v:.0}°)");
                }
                let pf: Vec<P> = aresta
                    .iter()
                    .zip(&ws)
                    .map(|(p, w)| pele.blend_linear(*p, w))
                    .collect();
                linha += &format!(" LBS {:.3}", raio_da_tampa(&pf).0);
                // O ângulo de MEIO-ÂNGULO (o do dual quaternion), em torno do MESMO centro.
                let meio = |p: P, w: &[f64]| -> P {
                    let Some(c) = pele.centro_de_rotacao(w) else {
                        return pele.blend_linear(p, w);
                    };
                    let (mut x, mut y) = (0.0, 0.0);
                    for (bb, &k) in pele.bones().iter().zip(w) {
                        let t = bb.angulo_da_pose() * 0.5;
                        x += k * t.cos();
                        y += k * t.sin();
                    }
                    let t = 2.0 * y.atan2(x);
                    let base = pele.blend_linear(c, w);
                    let d = [p[0] - c[0], p[1] - c[1]];
                    [
                        base[0] + t.cos() * d[0] - t.sin() * d[1],
                        base[1] + t.sin() * d[0] + t.cos() * d[1],
                    ]
                };
                let pf: Vec<P> = aresta.iter().zip(&ws).map(|(p, w)| meio(*p, w)).collect();
                let (rm, vm) = raio_da_tampa(&pf);
                let wc: Vec<f64> = ws
                    .iter()
                    .map(|w| w[ic] / (w[ic] + w[ip]).max(1e-12))
                    .collect();
                let em = |q: f64| aresta[wc.iter().position(|&v| v >= q).unwrap_or(0)][0] - xj;
                linha += &format!(
                    " MEIO {rm:.3} ({vm:.0}°) · filho 5–95 % [{:+.3},{:+.3}] 50 % {:+.3} · sintético:",
                    em(0.05),
                    em(0.95),
                    em(0.5)
                );
                for a in [0.1875_f64, 0.375, 0.75] {
                    let sint = |p: P| {
                        let t = suave((p[0] - xj + a) / (2.0 * a));
                        let mut w = vec![0.0; pele.len()];
                        w[ip] = 1.0 - t;
                        w[ic] = t;
                        w
                    };
                    linha += &format!(" a={a}");
                    for (nome, lei) in [
                        ("circ", MisturaDoAngulo::Circulo),
                        ("desd", MisturaDoAngulo::Desdobrado),
                    ] {
                        let pf: Vec<P> = aresta
                            .iter()
                            .map(|p| pele.blend_com(*p, &sint(*p), lei))
                            .collect();
                        linha += &format!(" {nome} {:.3}", raio_da_tampa(&pf).0);
                    }
                }
                println!("{linha}");
            }
            // O RISCO do lado de DENTRO da barra: laços (cruzamentos) e a espessura mínima.
            let yd = JUNTAS[ji].3;
            let dentro: Vec<P> = (0..=600)
                .map(|i| [xj - 0.6 + 0.002 * f64::from(i), yd])
                .collect();
            let wd: Vec<Vec<f64>> = dentro.iter().map(|p| l.pesos(*p)).collect();
            let wf: Vec<Vec<f64>> = barra.iter().map(|p| l.pesos(*p)).collect();
            let meio = |p: P, w: &[f64]| -> P {
                let Some(c) = pele.centro_de_rotacao(w) else {
                    return pele.blend_linear(p, w);
                };
                let (mut x, mut y) = (0.0, 0.0);
                for (bb, &k) in pele.bones().iter().zip(w) {
                    let t = bb.angulo_da_pose() * 0.5;
                    x += k * t.cos();
                    y += k * t.sin();
                }
                let t = 2.0 * y.atan2(x);
                let base = pele.blend_linear(c, w);
                let d = [p[0] - c[0], p[1] - c[1]];
                [
                    base[0] + t.cos() * d[0] - t.sin() * d[1],
                    base[1] + t.sin() * d[0] + t.cos() * d[1],
                ]
            };
            type Lei<'a> = (&'a str, Box<dyn Fn(P, &[f64]) -> P + 'a>);
            let leis: Vec<Lei> = vec![
                (
                    "circ",
                    Box::new(|p, w| pele.blend_com(p, w, MisturaDoAngulo::Circulo)),
                ),
                (
                    "desd",
                    Box::new(|p, w| pele.blend_com(p, w, MisturaDoAngulo::Desdobrado)),
                ),
                ("MEIO", Box::new(meio)),
                ("LBS", Box::new(|p, w| pele.blend_linear(p, w))),
            ];
            let mut linha = format!("{g}/{g} junta {} DENTRO da barra:", ji + 1);
            for (nome, f) in &leis {
                let pd: Vec<P> = dentro.iter().zip(&wd).map(|(p, w)| f(*p, w)).collect();
                let pf: Vec<P> = barra.iter().zip(&wf).map(|(p, w)| f(*p, w)).collect();
                let esp = pf
                    .iter()
                    .map(|p| dist_min(*p, &pd))
                    .fold(f64::MAX, f64::min);
                linha += &format!(" {nome} laços {} espessura {esp:.3} ·", cruzamentos(&pd));
            }
            println!("{linha}");
        }
    }
}
