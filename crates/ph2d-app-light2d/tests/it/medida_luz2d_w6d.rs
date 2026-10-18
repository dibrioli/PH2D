//! **A rodada da W6d — o F5, o disco em zoom** (`docs/Luz2D/01_plano.md` F5): o que o dono vê HOJE
//! (a lei do miolo é a difusão, W5h) em ZOOMS CONTÍNUOS, contra o gabarito da difusão. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w6d -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o disco e a elipse POLÍGONOS da cena 5 × [`ZOOMS`] × [`FASES`] × sete anéis, produto × controlo
//!    (a lei antiga, `as_formas_redondas::config(.., true)`), intercalados;
//! B. o disco de IMAGEM (o sprite do atlas, pela captura do produto) nos mesmos zooms;
//! F. fotos (a luz recebida como o editor a pinta, píxel de ecrã a píxel de ecrã) do disco de imagem e
//!    do polígono em [`ZOOMS_FOTO`], e o gabarito — `target/fotos_w6d/*.ppm`.

use std::io::Write;

use ph2d_light2d_ecs::CenaDeLuz;
use vello::kurbo::{Affine, Point};

use crate::as_formas_redondas::{
    ANEIS, Leitura, Redondo, anel, cena_redonda, config, disco_de_imagem, ler_anel, passo_tx,
    vista_em,
};
use crate::comum::{Fluencia, fluencia_lida, gpu};
use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5d::solver;

/// Contínuos (geométricos) de 0,6 a 4: várias amostras dentro de cada oitava.
const N_ZOOMS: usize = 17;
const ZOOM_MIN: f64 = 0.6;
const ZOOM_MAX: f64 = 4.0;
const FASES: [u32; 3] = [0, 3, 5];
const MAIS_ANEIS: [(&str, f64, f64); 3] = [
    ("dentro 0,5", 0.5, 0.0),
    ("dentro 0,8", 0.8, 0.0),
    ("dentro 0,97", 0.97, 0.0),
];
const ZOOMS_FOTO: [f64; 4] = [1.0, 1.6, 2.5, 4.0];
const VARIANTES: [&str; 2] = ["produto", "controlo"];

fn zooms() -> Vec<f64> {
    (0..N_ZOOMS)
        .map(|k| ZOOM_MIN * (ZOOM_MAX / ZOOM_MIN).powf(k as f64 / (N_ZOOMS - 1) as f64))
        .collect()
}

fn aneis_todos() -> Vec<(&'static str, f64, f64)> {
    ANEIS.iter().chain(MAIS_ANEIS.iter()).copied().collect()
}

fn linha(fl: &Fluencia, r: &Redondo, gab_cena: &CenaDeLuz, tx: f64, pior: &mut Leitura) -> String {
    let aneis: Vec<_> = aneis_todos()
        .iter()
        .map(|(nome, f, fora)| (*nome, anel(gab_cena, r, *f, fora * tx)))
        .collect();
    let max = aneis
        .iter()
        .flat_map(|(_, (_, v))| v.iter().copied())
        .fold(0.0, f64::max);
    let mut s = String::new();
    for (nome, (pts, gab)) in &aneis {
        let eixo = |p: [f64; 2]| r.ao_eixo(p) < 2.0 * tx;
        let l = ler_anel(fl, pts, gab, max, passo_tx(pts, tx), eixo);
        pior.somar(l);
        s += &format!(
            " · {nome}: {}e({:.3}) ×{} ↔{:.1} TV {:+.2}",
            l.errados, l.pior, l.cruzes, l.desvio, l.tv
        );
    }
    s
}

#[test]
#[ignore = "a rodada da W6d: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w6d() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W6d · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) { "DEBUG" } else { "otimizado (smoke)" },
        loadavg()
    );
    let t0 = std::time::Instant::now();
    let zs = zooms();
    // [variante] — o pior por zoom (A polígonos · B imagem).
    let mut por_zoom_a = vec![[Leitura::default(); 2]; zs.len()];
    let mut por_zoom_b = vec![[Leitura::default(); 2]; zs.len()];
    for (zi, &zoom) in zs.iter().enumerate() {
        for ri in 0..2 {
            for &k in &FASES {
                let a = vista_em(zoom, cena_redonda([0.0; 2]).1[ri].centro);
                let cfg0 = config(&gpu, a, false);
                let tx = cfg0.grade.texel_m / f64::from(cfg0.fina);
                let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
                let (c, redondos) = cena_redonda(d);
                let r = redondos[ri];
                for j0 in 0..VARIANTES.len() {
                    let v = (j0 + k as usize + zi) % VARIANTES.len();
                    let mut s = solver(&gpu, a, config(&gpu, a, v == 1));
                    s.quadro(&gpu, &c);
                    let fl = fluencia_lida(&gpu, &s);
                    let l = linha(&fl, &r, &c, tx, &mut por_zoom_a[zi][v]);
                    eprintln!(
                        "A · zoom {zoom:.3} · tx {:.2} mm · {k}/8 · {} · {}{l}",
                        tx * 1e3,
                        r.nome,
                        VARIANTES[v]
                    );
                }
            }
        }
        let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
        for j0 in 0..VARIANTES.len() {
            let v = (j0 + zi) % VARIANTES.len();
            let cfg = config(&gpu, a, v == 1);
            let tx = cfg.grade.texel_m / f64::from(cfg.fina);
            let (fl, r, gab_cena) = disco_de_imagem(&gpu, a, cfg);
            let l = linha(&fl, &r, &gab_cena, tx, &mut por_zoom_b[zi][v]);
            eprintln!("B · imagem · zoom {zoom:.3} · {}{l}", VARIANTES[v]);
        }
    }
    for (rotulo, tabela) in [("A polígonos", &por_zoom_a), ("B imagem", &por_zoom_b)] {
        for (zi, &zoom) in zs.iter().enumerate() {
            let mut s = format!("RESUMO · {rotulo} · zoom {zoom:.3}");
            for (v, nome) in VARIANTES.iter().enumerate() {
                let l = tabela[zi][v];
                s += &format!(
                    " · {nome}: {}e({:.3}) ×{} ↔{:.1} TV {:+.2}",
                    l.errados, l.pior, l.cruzes, l.desvio, l.tv
                );
            }
            eprintln!("{s}");
        }
    }
    eprintln!(
        "# {:.0} s · loadavg depois {}",
        t0.elapsed().as_secs_f64(),
        loadavg()
    );
}

/// A cor de um píxel: a luz `l` (fluência) sobre o albedo `cor`, tonemap `x/(1+x)` e gama 2,2.
fn pinta(l: f64, cor: [f64; 3]) -> [u8; 3] {
    cor.map(|c| {
        let x = c * l / std::f64::consts::TAU;
        ((x / (1.0 + x)).powf(1.0 / 2.2).clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

fn grava(nome: &str, lado: u32, px: &[u8]) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fotos_w6d");
    std::fs::create_dir_all(&dir).expect("pasta");
    let p = dir.join(format!("{nome}.ppm"));
    let mut f = std::fs::File::create(&p).expect("foto");
    write!(f, "P6\n{lado} {lado}\n255\n").expect("cabeçalho");
    f.write_all(px).expect("píxeis");
    eprintln!("foto: {}", p.display());
}

/// A foto do disco `r` na vista `a`, píxel de ecrã a píxel de ecrã (o lado em píxeis = 1,3 × o
/// diâmetro no ecrã, no máximo 900), com a luz `luz(p)`.
fn foto(nome: &str, a: Affine, r: &Redondo, cor: [f64; 3], luz: impl Fn([f64; 2]) -> f64) {
    let inv = a.inverse();
    let ppm = (a * Point::new(1.0, 0.0) - a * Point::new(0.0, 0.0)).hypot();
    let lado = ((2.6 * r.semi[0] * ppm).round() as u32).min(900);
    let c = a * Point::new(r.centro[0], r.centro[1]);
    let mut px = Vec::with_capacity((lado * lado * 3) as usize);
    for j in 0..lado {
        for i in 0..lado {
            let q = Point::new(
                c.x - f64::from(lado) * 0.5 + f64::from(i) + 0.5,
                c.y - f64::from(lado) * 0.5 + f64::from(j) + 0.5,
            );
            let p = inv * q;
            let (x, y) = ((p.x - r.centro[0]) / r.semi[0], (p.y - r.centro[1]) / r.semi[1]);
            let dentro = x * x + y * y <= 1.0;
            px.extend(pinta(
                luz([p.x, p.y]),
                if dentro { cor } else { [0.85; 3] },
            ));
        }
    }
    grava(nome, lado, &px);
}

#[test]
#[ignore = "as fotos da W6d: corre-se à mão (cabeçalho, nome medida_luz2d_w6d_fotos)"]
fn medida_luz2d_w6d_fotos() {
    let Some(gpu) = gpu() else { return };
    let corpo = &ph2d_app_light2d::smoke::CORPOS_DO_VOLUME[0];
    let cor = corpo.cor.map(f64::from);
    let (c_pol, redondos) = cena_redonda([0.0; 2]);
    let r_pol = redondos[0];
    for &zoom in &ZOOMS_FOTO {
        let a = vista_em(zoom, r_pol.centro);
        let (fl, r, _) = disco_de_imagem(&gpu, a, config(&gpu, a, false));
        foto(&format!("imagem_zoom{zoom}"), a, &r, cor, |p| fl.amostra(p));
        let mut s = solver(&gpu, a, config(&gpu, a, false));
        s.quadro(&gpu, &c_pol);
        let fl = fluencia_lida(&gpu, &s);
        foto(&format!("poligono_zoom{zoom}"), a, &r_pol, cor, |p| fl.amostra(p));
    }
    // O gabarito não depende da vista: uma foto, na escala do zoom 1.
    let a = vista_em(1.0, r_pol.centro);
    foto("gabarito", a, &r_pol, cor, |p| ph2d_light2d_ref::fluencia(&c_pol, p)[1]);
}

/// O censo dos texels junto à borda do disco `r` (a fluência no CENTRO do texel, sem bilinear) contra
/// o gabarito: por faixa da distância à borda (texels; negativo = dentro), os texels a mais de
/// [`ERRADO_BORDA`] do máximo ESCUROS e CLAROS, longe das passagens luz/sombra do gabarito (onde ele
/// muda mais de 0,2 do máximo num texel ao longo da borda), e o pior escuro com o seu `d`.
fn censo_da_borda(fl: &Fluencia, r: &Redondo, gab: &CenaDeLuz, tx: f64) -> String {
    const FAIXAS: [f64; 5] = [-1.5, -0.5, 0.0, 0.5, 1.5];
    let g = fl.grade;
    let lei = |p: [f64; 2]| ph2d_light2d_ref::fluencia(gab, p)[1];
    let mut casos = Vec::new();
    for j in 0..g.altura {
        for i in 0..g.largura {
            let p = g.centro(i, j);
            let (dx, dy) = (p[0] - r.centro[0], p[1] - r.centro[1]);
            let rho = dx.hypot(dy);
            let d = (rho - r.semi[0]) / tx;
            if d < FAIXAS[0] || d >= FAIXAS[4] || rho < 1e-9 {
                continue;
            }
            let t = [-dy / rho * tx, dx / rho * tx];
            let v = f64::from(fl.v[(j * g.largura + i) as usize][1]);
            let (g0, ga, gb) = (lei(p), lei([p[0] + t[0], p[1] + t[1]]), lei([p[0] - t[0], p[1] - t[1]]));
            casos.push((d, v, g0, (ga - gb).abs()));
        }
    }
    let max = casos.iter().map(|c| c.2).fold(0.0, f64::max);
    let mut s = String::new();
    let mut pior = (0.0_f64, 0.0_f64);
    for f in 0..4 {
        let (mut escuros, mut claros, mut n, mut passagem) = (0, 0, 0, 0);
        for &(d, v, g0, dg) in &casos {
            if d < FAIXAS[f] || d >= FAIXAS[f + 1] {
                continue;
            }
            if dg > 0.2 * max {
                passagem += 1;
                continue;
            }
            n += 1;
            let e = (v - g0) / max;
            escuros += usize::from(e < -ERRADO_BORDA);
            claros += usize::from(e > ERRADO_BORDA);
            if e < pior.0 {
                pior = (e, d);
            }
        }
        s += &format!(
            " · [{:+.1},{:+.1}) {escuros}↓ {claros}↑ /{n} (+{passagem} na passagem)",
            FAIXAS[f],
            FAIXAS[f + 1]
        );
    }
    s + &format!(" · pior escuro {:+.3} em d {:+.2}", pior.0, pior.1)
}

const ERRADO_BORDA: f64 = 0.10;

/// Diagnóstico (não cura): o produto e as peças da borda desligadas, uma a uma.
const DIAG: [&str; 5] = ["produto", "sem borda exata", "sem pé", "sem face", "sem miolo"];

fn config_diag(gpu: &ph2d_gpu::GpuContext, a: Affine, v: usize) -> ph2d_light2d_gpu::Config {
    let c = config(gpu, a, false);
    match v {
        1 => ph2d_light2d_gpu::Config { borda_exata: false, ..c },
        2 => ph2d_light2d_gpu::Config { pe_na_borda: false, ..c },
        3 => ph2d_light2d_gpu::Config { vis_na_face: false, ..c },
        4 => ph2d_light2d_gpu::Config { miolo: None, ..c },
        _ => c,
    }
}

#[test]
#[ignore = "o censo da borda da W6d: corre-se à mão (cabeçalho, nome medida_luz2d_w6d_borda)"]
fn medida_luz2d_w6d_borda() {
    let Some(gpu) = gpu() else { return };
    eprintln!("# censo da borda · loadavg antes {}", loadavg());
    for zoom in [0.8, 1.6, 3.2] {
        for k in 0..8u32 {
            let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
            let cfg0 = config(&gpu, a, false);
            let tx = cfg0.grade.texel_m / f64::from(cfg0.fina);
            let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
            let (c, redondos) = cena_redonda(d);
            let r = redondos[0];
            for j0 in 0..DIAG.len() {
                let v = (j0 + k as usize) % DIAG.len();
                let mut s = solver(&gpu, a, config_diag(&gpu, a, v));
                s.quadro(&gpu, &c);
                let fl = fluencia_lida(&gpu, &s);
                eprintln!(
                    "BORDA · polígono · zoom {zoom} · {k}/8 · {}{}",
                    DIAG[v],
                    censo_da_borda(&fl, &r, &c, tx)
                );
            }
        }
        let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
        for v in 0..DIAG.len() {
            let cfg = config_diag(&gpu, a, v);
            let tx = cfg.grade.texel_m / f64::from(cfg.fina);
            let (fl, r, gab) = disco_de_imagem(&gpu, a, cfg);
            eprintln!(
                "BORDA · imagem · zoom {zoom} · {}{}",
                DIAG[v],
                censo_da_borda(&fl, &r, &gab, tx)
            );
        }
    }
    eprintln!("# loadavg depois {}", loadavg());
}
