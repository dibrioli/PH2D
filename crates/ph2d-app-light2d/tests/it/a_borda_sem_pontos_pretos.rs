//! ⭐⭐ **A borda dos corpos não tem pontos PRETOS** (F5, W6d): rente à borda do lado aceso, um texel lia só o
//! ambiente — a fresta «fora» da reta por texel (a busca do ponto de fronteira caía nela) e o texel que a
//! borda exata põe fora e o iso-½ dentro (a marcha nascia dentro do corpo). Handoff W6d §1, §5.
//!
//! A régua: cada texel fino a menos de ½ texel da borda do disco, do lado ACESO (a menos de 86° da
//! lâmpada), o valor no centro contra o gabarito, longe das passagens luz/sombra; o controlo é a borda de
//! antes (`borda_continua` e `borda_fora` desligadas) na mesma corrida.

use ph2d_light2d_ecs::CenaDeLuz;

use crate::as_formas_redondas::{Redondo, cena_redonda, config, disco_de_imagem, vista_em};
use crate::comum::{Fluencia, fluencia_lida, gpu};
use crate::medida_luz2d_w5d::solver;

/// Um texel rente está PRETO abaixo disto (relativo ao máximo do gabarito): o produto medido, o pior
/// −0,176 em 24 configurações + 3 da imagem; o controlo −0,96.
const PRETO: f64 = -0.25;

/// O pior texel rente do lado aceso: `(pior relativo, quantos abaixo de PRETO)`.
fn pior_rente(fl: &Fluencia, r: &Redondo, gab: &CenaDeLuz, tx: f64) -> (f64, usize) {
    let g = fl.grade;
    let lampada = ph2d_app_light2d::smoke::LAMPADA_DO_VOLUME.map(f64::from);
    let lei = |p: [f64; 2]| ph2d_light2d_ref::fluencia(gab, p)[1];
    let mut casos = Vec::new();
    for j in 0..g.altura {
        for i in 0..g.largura {
            let p = g.centro(i, j);
            let n = [p[0] - r.centro[0], p[1] - r.centro[1]];
            let rho = n[0].hypot(n[1]);
            let d = (rho - r.semi[0]) / tx;
            if !(-0.5..0.5).contains(&d) || rho < 1e-9 {
                continue;
            }
            let l = [lampada[0] - p[0], lampada[1] - p[1]];
            let cos = (n[0] * l[0] + n[1] * l[1]) / (rho * l[0].hypot(l[1]));
            if cos < 86f64.to_radians().cos() {
                continue;
            }
            let t = [-n[1] / rho * tx, n[0] / rho * tx];
            let dg = (lei([p[0] + t[0], p[1] + t[1]]) - lei([p[0] - t[0], p[1] - t[1]])).abs();
            casos.push((f64::from(fl.v[(j * g.largura + i) as usize][1]), lei(p), dg));
        }
    }
    let max = casos.iter().map(|c| c.1).fold(0.0, f64::max);
    casos
        .iter()
        .filter(|c| c.2 <= 0.2 * max)
        .map(|c| (c.0 - c.1) / max)
        .fold((0.0_f64, 0), |(p, n), e| {
            (p.min(e), n + usize::from(e < PRETO))
        })
}

/// ⭐⭐ **GATE — sem pontos pretos rentes à borda**: o disco polígono (zoom 3,2, a fase que o produto de
/// antes tinha pior) e o disco de imagem (zoom 3,2); o controlo (a borda de antes) tem de os ter.
#[test]
fn a_borda_nao_tem_pontos_pretos() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let antes = |c: ph2d_light2d_gpu::Config| ph2d_light2d_gpu::Config {
        borda_continua: false,
        borda_fora: false,
        borda_vertice: None,
        ..c.sem_os_cantos()
    };
    let a = vista_em(3.2, cena_redonda([0.0; 2]).1[0].centro);
    let cfg = config(&gpu, a, false);
    let tx = cfg.grade.texel_m / f64::from(cfg.fina);
    let (c, redondos) = cena_redonda([5.0 * tx / 8.0, 15.0 * tx / 8.0]);
    let poligono = |cfg| {
        let mut s = solver(&gpu, a, cfg);
        s.quadro(&gpu, &c);
        pior_rente(&fluencia_lida(&gpu, &s), &redondos[0], &c, tx)
    };
    let imagem = |cfg| {
        let (fl, r, gab) = disco_de_imagem(&gpu, a, cfg);
        pior_rente(&fl, &r, &gab, tx)
    };
    let (p, p0) = (poligono(cfg), poligono(antes(cfg)));
    let (i, i0) = (imagem(cfg), imagem(antes(cfg)));
    eprintln!(
        "pretos rentes · polígono {:+.3} ({}) · imagem {:+.3} ({}) — antes {:+.3} ({}) · {:+.3} ({})",
        p.0, p.1, i.0, i.1, p0.0, p0.1, i0.0, i0.1
    );
    assert!(
        p0.1 > 0 && i0.1 > 0,
        "o controlo (a borda de antes) não tem pretos — a régua não os vê"
    );
    assert!(
        p.1 == 0 && i.1 == 0,
        "pontos pretos rentes à borda: polígono {p:?} · imagem {i:?}"
    );
}
