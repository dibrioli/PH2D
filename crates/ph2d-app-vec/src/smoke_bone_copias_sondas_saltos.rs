//! Medida (report do dono, smoke 2 da `=6` a `170,170`/`170,140`): *«a parte externa da junta se
//! deforma aos saltos sem permanecer arredondada»*. Na TAMPA de fora de cada junta (o lado convexo,
//! a menos de `0,6` da junta): o desenho contra a imagem EXACTA do contorno em repouso, a mudança de
//! quadro a quadro de um e de outro numa varredura fina da dobra, e o raio de curvatura mínimo.

use super::a13::{dist_pl, fechados_de};
use super::refino::{Barra, cena};
use super::*;
use ph2d_vec_skin::curva::{REFINO_DO_PRODUTO, Refino};

/// O raio da janela da tampa, em unidades da cena (a barra tem `0,75` de largura).
const R: f64 = 0.6;

/// As juntas da barra `k` (as pontas partilhadas por dois ossos), com a BISSECTRIZ dos dois ossos
/// (para o lado dos membros): a tampa de fora fica do outro lado.
fn juntas(sim: &SimWorld, k: usize) -> Vec<([f64; 2], [f64; 2])> {
    let segs = ph2d_skeleton_live::skin_live::bone_segments(sim);
    let mut out = Vec::new();
    for (_, a, b) in &segs {
        let lado = if k == 0 { b[0] < 0.0 } else { b[0] > 0.0 };
        if !lado {
            continue;
        }
        if let Some((_, _, c)) = segs
            .iter()
            .find(|(_, a2, _)| (a2[0] - b[0]).hypot(a2[1] - b[1]) < 1e-6)
        {
            let un = |v: [f64; 2]| {
                let l = v[0].hypot(v[1]).max(1e-12);
                [v[0] / l, v[1] / l]
            };
            let (u1, u2) = (
                un([a[0] - b[0], a[1] - b[1]]),
                un([c[0] - b[0], c[1] - b[1]]),
            );
            out.push((*b, un([u1[0] + u2[0], u1[1] + u2[1]])));
        }
    }
    out.sort_by(|x, y| x.0[0].total_cmp(&y.0[0]));
    out
}

/// A tampa de fora da junta `(j, m)`: os pontos a menos de [`R`] dela, do lado oposto aos membros.
fn na_tampa((j, m): ([f64; 2], [f64; 2]), p: [f64; 2]) -> bool {
    let d = [p[0] - j[0], p[1] - j[1]];
    d[0].hypot(d[1]) <= R && d[0] * m[0] + d[1] * m[1] < 0.0
}

/// Hausdorff de `a` e `b` restrita à tampa (cada ponto da tampa de um contra o OUTRO inteiro).
fn haus_na_tampa(a: &[Vec<[f64; 2]>], b: &[Vec<[f64; 2]>], jm: ([f64; 2], [f64; 2])) -> f64 {
    // Só os troços perto da junta contam como alvo (o resto fica longe de toda a tampa).
    let perto = |y: &[Vec<[f64; 2]>]| -> Vec<Vec<[f64; 2]>> {
        let mut out = Vec::new();
        for l in y {
            let mut run: Vec<[f64; 2]> = Vec::new();
            for p in l {
                if (p[0] - jm.0[0]).hypot(p[1] - jm.0[1]) <= R + 0.4 {
                    run.push(*p);
                } else if !run.is_empty() {
                    run.push(*p);
                    out.push(std::mem::take(&mut run));
                }
            }
            if !run.is_empty() {
                out.push(run);
            }
        }
        out
    };
    let lado = |x: &[Vec<[f64; 2]>], y: &[Vec<[f64; 2]>]| {
        let y = perto(y);
        x.iter()
            .flatten()
            .filter(|p| na_tampa(jm, **p))
            .map(|p| dist_pl(&y, *p))
            .fold(0.0, f64::max)
    };
    lado(a, b).max(lado(b, a))
}

/// O menor raio de curvatura na tampa: cada corrida contígua da polilinha dentro dela, reamostrada
/// a passos de `h` de arco, e a viragem medida sobre `5` passos.
fn raio_min(pl: &[Vec<[f64; 2]>], jm: ([f64; 2], [f64; 2])) -> f64 {
    const H: f64 = 0.004;
    let mut melhor = f64::MAX;
    for l in pl {
        let mut corrida: Vec<[f64; 2]> = Vec::new();
        for p in l.iter().chain(std::iter::once(&[f64::NAN; 2])) {
            if !p[0].is_nan() && na_tampa(jm, *p) {
                corrida.push(*p);
                continue;
            }
            if corrida.len() > 2 {
                // Reamostragem por arco.
                let mut r = vec![corrida[0]];
                let mut falta = H;
                for w in corrida.windows(2) {
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
                for i in 5..r.len().saturating_sub(5) {
                    let (p0, p1, p2) = (r[i - 5], r[i], r[i + 5]);
                    let (u, v) = (
                        [p1[0] - p0[0], p1[1] - p0[1]],
                        [p2[0] - p1[0], p2[1] - p1[1]],
                    );
                    let ang = (u[0] * v[1] - u[1] * v[0])
                        .atan2(u[0] * v[0] + u[1] * v[1])
                        .abs();
                    if ang > 1e-9 {
                        melhor = melhor.min(5.0 * H / ang);
                    }
                }
            }
            corrida.clear();
        }
    }
    melhor
}

/// O desenho fechado de `b` com `refino`, em polilinha.
fn desenho(b: &Barra, refino: Option<Refino>) -> (Vec<Vec<[f64; 2]>>, usize) {
    let d = b.assa(refino);
    (polilinhas(&fechados_de(&d), 64), d.verts_all().count())
}

/// Uma linha de SVG (y para baixo) de uma polilinha.
fn svg_pl(pl: &[Vec<[f64; 2]>], cor: &str, w: f64) -> String {
    let mut s = String::new();
    for l in pl {
        let pts: Vec<String> = l
            .iter()
            .map(|p| format!("{:.5},{:.5}", p[0], -p[1]))
            .collect();
        let _ = write!(
            s,
            "<polyline points='{}' fill='none' stroke='{cor}' stroke-width='{w}' stroke-linejoin='round'/>",
            pts.join(" ")
        );
    }
    s
}

/// A janela `±0,8` em volta da junta `j`: a verdade (azul fino), o desenho com o refino (vermelho)
/// e sem ele (verde).
fn foto(
    j: [f64; 2],
    verdade: &[Vec<[f64; 2]>],
    k2: &[Vec<[f64; 2]>],
    off: &[Vec<[f64; 2]>],
    titulo: &str,
) -> String {
    let (x0, y0, l) = (j[0] - 0.8, -j[1] - 0.8, 1.6);
    format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{x0} {y0} {l} {l}' width='600' height='600'>\
         <rect x='{x0}' y='{y0}' width='{l}' height='{l}' fill='white'/>{}{}{}\
         <circle cx='{}' cy='{}' r='0.01' fill='black'/>\
         <text x='{}' y='{}' font-size='0.06' fill='black'>{titulo}</text></svg>",
        svg_pl(off, "#2a9d2a", 0.012),
        svg_pl(k2, "#d00000", 0.008),
        svg_pl(verdade, "#1e66ff", 0.004),
        j[0],
        -j[1],
        x0 + 0.03,
        y0 + 0.08
    )
}

/// As leis comparadas.
const LEIS: [(&str, Option<Refino>); 2] = [("produto", REFINO_DO_PRODUTO), ("off", None)];

fn graus(base: f32, passo: f32, i: u16) -> f32 {
    base + passo * f32::from(i)
}

/// Uma varredura de `poses` numa barra: por junta e lei, o pior salto do desenho ACIMA do da
/// verdade entre quadros vizinhos (e a razão), a pose, o desvio máximo à verdade na tampa e os nós.
/// Com `cada`, imprime cada passo da junta `cada - 1`.
fn varre(rotulo: &str, poses: &[(f32, f32)], barra: usize, cada: Option<usize>) {
    type Pior = (f64, f64, (f32, f32), f64, (usize, usize));
    let mut pior: Vec<Vec<Pior>> = vec![vec![(0.0, 0.0, (0.0, 0.0), 0.0, (usize::MAX, 0)); 2]; 2];
    type Contornos = Vec<Vec<[f64; 2]>>;
    let mut antes: Option<(Contornos, [Contornos; 2])> = None;
    for &(g1, g2) in poses {
        let (sim, _, st, ids) = cena(g1, g2);
        let b = Barra::de(&sim, &st, ids[barra]);
        let js = juntas(&sim, barra);
        let verdade = b.imagem(512);
        let ds: Vec<(Vec<Vec<[f64; 2]>>, usize)> =
            LEIS.iter().map(|(_, l)| desenho(&b, *l)).collect();
        for (ji, jm) in js.iter().enumerate().take(2) {
            let mut linha = format!("    {g1:.1}/{g2:.1} j{}", ji + 1);
            for (li, (d, nos)) in ds.iter().enumerate() {
                let p = &mut pior[ji][li];
                let desvio = haus_na_tampa(d, &verdade, *jm);
                p.3 = p.3.max(desvio);
                p.4 = (p.4.0.min(*nos), p.4.1.max(*nos));
                if let Some((v0, d0)) = &antes {
                    let dv = haus_na_tampa(&verdade, v0, *jm);
                    let dd = haus_na_tampa(d, &d0[li], *jm);
                    linha += &format!(
                        " · {}: salto {dd:.4} (verdade {dv:.4}) desvio {desvio:.4} nós {nos}",
                        LEIS[li].0
                    );
                    if dd - dv > p.0 {
                        *p = (dd - dv, dd / dv.max(1e-9), (g1, g2), p.3, p.4);
                    }
                }
            }
            if cada == Some(ji + 1) {
                println!("{linha}");
            }
        }
        antes = Some((verdade, [ds[0].0.clone(), ds[1].0.clone()]));
    }
    for (ji, por_lei) in pior.iter().enumerate() {
        for (li, (exc, razao, pose, desvio, nos)) in por_lei.iter().enumerate() {
            println!(
                "{rotulo} · barra {barra} junta {} · {:>3}: pior salto {exc:.4} acima da verdade \
                 (razão {razao:.1}×) em {}/{} · desvio máx à verdade na tampa {desvio:.4} · nós {}–{}",
                ji + 1,
                LEIS[li].0,
                pose.0,
                pose.1,
                nos.0,
                nos.1
            );
        }
    }
}

/// ⭐ **SONDA — os SALTOS da tampa da junta.** `SONDA_VARREDURA` escolhe: `g1_140`, `g1_170`
/// (`g1 = 150…175`, passo `0,1`) ou `g2_170` (`g2 = 150…175` com `g1 = 170`); `SONDA_BARRA`, a
/// barra. Corra em `--release`.
#[test]
#[ignore = "sonda: imprime; corra em --release"]
fn diag_os_saltos_da_tampa_da_junta() {
    let qual = std::env::var("SONDA_VARREDURA").unwrap_or_else(|_| "g1_170".into());
    let barras: Vec<usize> = std::env::var("SONDA_BARRA")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(vec![0, 1], |b| vec![b]);
    let poses: Vec<(f32, f32)> = match qual.as_str() {
        "g1_140" => (0..=250).map(|i| (graus(150.0, 0.1, i), 140.0)).collect(),
        "g2_170" => (0..=250).map(|i| (170.0, graus(150.0, 0.1, i))).collect(),
        _ => (0..=250).map(|i| (graus(150.0, 0.1, i), 170.0)).collect(),
    };
    for b in barras {
        varre(&qual, &poses, b, None);
    }
}

/// ⭐ **SONDA — o salto DE PERTO**: `g2` de `SONDA_DE` a `SONDA_ATE` (passo `0,1`) com `g1 = 170`,
/// barra `0`, cada passo da junta `SONDA_JUNTA` (omissão `1`).
#[test]
#[ignore = "sonda: imprime; corra em --release"]
fn diag_o_salto_de_perto() {
    let g = |n: &str, o: f32| {
        std::env::var(n)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(o)
    };
    let (de, ate) = (g("SONDA_DE", 158.0), g("SONDA_ATE", 162.0));
    let junta = std::env::var("SONDA_JUNTA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let mut poses = Vec::new();
    let mut i = 0u16;
    while graus(de, 0.1, i) <= ate + 1e-3 {
        poses.push((170.0, graus(de, 0.1, i)));
        i += 1;
    }
    varre("de perto", &poses, 0, Some(junta));
}

/// ⭐ **SONDA — a REDONDEZA e as fotos** nas poses do dono e no controlo `110/110`, e a folha de
/// contacto (`g1 = 150…175` de `0,5` em `0,5` com `g2 = 170`, barra `0`). Em `SONDA_SAIDA`
/// (omissão `target/prova/saltos`).
#[test]
#[ignore = "sonda: imprime e escreve SVG; corra em --release"]
fn diag_a_redondeza_e_as_fotos() {
    let saida = std::env::var("SONDA_SAIDA").unwrap_or_else(|_| "target/prova/saltos".into());
    std::fs::create_dir_all(&saida).expect("pasta");
    for (g1, g2) in [
        (170f32, 170f32),
        (170.0, 140.0),
        (170.0, 160.2),
        (110.0, 110.0),
    ] {
        let (sim, _, st, ids) = cena(g1, g2);
        for (barra, id) in ids.iter().enumerate() {
            let b = Barra::de(&sim, &st, *id);
            let verdade = b.imagem(512);
            let (k2, _) = desenho(&b, REFINO_DO_PRODUTO);
            let (off, _) = desenho(&b, None);
            for (ji, jm) in juntas(&sim, barra).iter().enumerate().take(2) {
                println!(
                    "{g1}/{g2} barra {barra} junta {} · raio mínimo na tampa: verdade {:.4} · k2 {:.4} · \
                     off {:.4} · desvio à verdade k2 {:.4} off {:.4}",
                    ji + 1,
                    raio_min(&verdade, *jm),
                    raio_min(&k2, *jm),
                    raio_min(&off, *jm),
                    haus_na_tampa(&k2, &verdade, *jm),
                    haus_na_tampa(&off, &verdade, *jm)
                );
                if barra == 0 {
                    let f = format!("{saida}/tampa_{g1}_{g2}_junta{}.svg", ji + 1);
                    std::fs::write(
                        &f,
                        foto(
                            jm.0,
                            &verdade,
                            &k2,
                            &off,
                            &format!("{g1}/{g2} junta {}", ji + 1),
                        ),
                    )
                    .expect("svg");
                }
            }
        }
    }
    for (rotulo, var) in [("g1", true), ("g2", false)] {
        for i in 0..=50u16 {
            let (g1, g2) = if var {
                (graus(150.0, 0.5, i), 170.0)
            } else {
                (170.0, graus(150.0, 0.5, i))
            };
            let (sim, _, st, ids) = cena(g1, g2);
            let b = Barra::de(&sim, &st, ids[0]);
            let verdade = b.imagem(512);
            let (k2, _) = desenho(&b, REFINO_DO_PRODUTO);
            let (off, _) = desenho(&b, None);
            for (ji, jm) in juntas(&sim, 0).iter().enumerate().take(2) {
                let f = format!("{saida}/folha_{rotulo}_j{}_{i:02}.svg", ji + 1);
                std::fs::write(
                    &f,
                    foto(
                        jm.0,
                        &verdade,
                        &k2,
                        &off,
                        &format!("{g1:.1}/{g2:.1} j{}", ji + 1),
                    ),
                )
                .expect("svg");
            }
        }
    }
    println!("fotos em {saida}");
}

/// As leis comparadas na CONTINUIDADE: sem refino, o de hoje e os candidatos que não dependem
/// da pose.
fn candidatos() -> Vec<(String, Option<Refino>)> {
    let mut v = vec![("sem".to_string(), None)];
    for passo in [0.1, 0.05, 0.025] {
        v.push((format!("peso {passo}"), Some(Refino { passo })));
    }
    v
}

/// ⭐ **A régua da CONTINUIDADE** — as três varreduras de `0,1°` (a cena presa UMA vez e só a pose
/// a mudar), as duas barras e as duas juntas: para cada lei, cada passo dá `(Δ desenho, Δ verdade)`
/// na tampa (Hausdorff entre quadros vizinhos). Devolve, por lei, todos os pares.
pub(crate) fn continuidade(leis: &[Option<Refino>]) -> Vec<Vec<(f64, f64, (f32, f32))>> {
    let mut pares = vec![Vec::new(); leis.len()];
    let varreduras: [Vec<(f32, f32)>; 3] = [
        (0..=250u16)
            .map(|i| (graus(150.0, 0.1, i), 140.0))
            .collect(),
        (0..=250u16)
            .map(|i| (graus(150.0, 0.1, i), 170.0))
            .collect(),
        (0..=250u16)
            .map(|i| (170.0, graus(150.0, 0.1, i)))
            .collect(),
    ];
    for poses in &varreduras {
        let (g1, g2) = poses[0];
        let (mut sim, _, st, ids, raizes) = super::refino::cena_com_raizes(g1, g2);
        let mut agora = (g1, g2);
        type Quadro = (Vec<Vec<[f64; 2]>>, Vec<Vec<Vec<[f64; 2]>>>);
        let mut antes: Vec<Option<Quadro>> = vec![None, None];
        for &(a, b) in poses {
            for r in &raizes {
                crate::smoke_bone_par::dobra_duas(&mut sim, *r, a - agora.0, b - agora.1);
            }
            agora = (a, b);
            for barra in 0..2 {
                let br = Barra::de(&sim, &st, ids[barra]);
                let verdade = br.imagem(128);
                let ds: Vec<Vec<Vec<[f64; 2]>>> = leis.iter().map(|l| desenho(&br, *l).0).collect();
                if let Some((v0, d0)) = &antes[barra] {
                    for jm in juntas(&sim, barra).iter().take(2) {
                        let dv = haus_na_tampa(&verdade, v0, *jm);
                        for (li, d) in ds.iter().enumerate() {
                            pares[li].push((haus_na_tampa(d, &d0[li], *jm), dv, (a, b)));
                        }
                    }
                }
                antes[barra] = Some((verdade, ds));
            }
        }
    }
    pares
}

/// ⭐ **SONDA — a continuidade por lei**: o pior salto acima da verdade, a pior razão (com o salto
/// acima de `5e-4`) e o maior `Δ desenho`. Corra em `--release`.
#[test]
#[ignore = "sonda: imprime; corra em --release"]
fn diag_a_continuidade_por_lei() {
    let ls = candidatos();
    let leis: Vec<Option<Refino>> = ls.iter().map(|x| x.1).collect();
    let pares = continuidade(&leis);
    for ((nome, _), p) in ls.iter().zip(&pares) {
        let exc = p
            .iter()
            .max_by(|x, y| (x.0 - x.1).total_cmp(&(y.0 - y.1)))
            .expect("passos");
        let razao = p
            .iter()
            .filter(|x| x.0 > 5e-4)
            .map(|x| (x.0 / x.1.max(1e-12), x.2))
            .max_by(|x, y| x.0.total_cmp(&y.0));
        let dmax = p.iter().map(|x| x.0).fold(0.0, f64::max);
        let vmax = p.iter().map(|x| x.1).fold(0.0, f64::max);
        println!(
            "{nome:>10}: pior salto {:.4} acima da verdade em {:?} · pior razão {:?} · Δ desenho máx {dmax:.4} \
             (verdade máx {vmax:.4}) · {} passos",
            exc.0 - exc.1,
            exc.2,
            razao.map(|r| (format!("{:.1}", r.0), r.1)),
            p.len()
        );
    }
}
