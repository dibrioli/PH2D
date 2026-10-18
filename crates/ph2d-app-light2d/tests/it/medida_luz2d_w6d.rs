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
        // `antes` = a borda de antes da W6d (a reta por texel, a marcha de fora pelo iso-½).
        for (quando, borda) in [("depois", true), ("antes", false)] {
            let cfg = ph2d_light2d_gpu::Config {
                borda_continua: borda,
                borda_fora: borda,
                ..config(&gpu, a, false)
            };
            let (fl, r, _) = disco_de_imagem(&gpu, a, cfg);
            foto(&format!("imagem_zoom{zoom}_{quando}"), a, &r, cor, |p| fl.amostra(p));
            let mut s = solver(&gpu, a, cfg);
            s.quadro(&gpu, &c_pol);
            let fl = fluencia_lida(&gpu, &s);
            foto(&format!("poligono_zoom{zoom}_{quando}"), a, &r_pol, cor, |p| fl.amostra(p));
        }
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
            casos.push((d, v, g0, (ga - gb).abs(), p));
        }
    }
    let max = casos.iter().map(|c| c.2).fold(0.0, f64::max);
    let lampada = ph2d_app_light2d::smoke::LAMPADA_DO_VOLUME.map(f64::from);
    let mut s = String::new();
    let mut pior = (0.0_f64, 0.0_f64);
    let (mut fam_a, mut fam_b, mut pior_a) = (0, 0, 0.0_f64);
    for f in 0..4 {
        let (mut escuros, mut claros, mut n, mut passagem) = (0, 0, 0, 0);
        for &(d, v, g0, dg, p) in &casos {
            if d < FAIXAS[f] || d >= FAIXAS[f + 1] {
                continue;
            }
            if dg > 0.2 * max {
                passagem += 1;
                continue;
            }
            n += 1;
            let e = (v - g0) / max;
            if e < -ERRADO_BORDA && std::env::var_os("PH2D_W6D_LISTA").is_some() {
                eprintln!(
                    "    PRETO d {d:+.3} · ângulo à lâmpada {:.1}° · placa {v:.5} · gabarito {g0:.5} · texel ({:.3}, {:.3})",
                    angulo(r.centro, p, lampada),
                    (p[0] - g.origem_m[0]) / g.texel_m,
                    (p[1] - g.origem_m[1]) / g.texel_m
                );
            }
            if e < -ERRADO_BORDA && f < 3 && FAIXAS[f] >= -0.5 {
                if angulo(r.centro, p, lampada) < 86.0 {
                    fam_a += 1;
                    pior_a = pior_a.min(e);
                } else {
                    fam_b += 1;
                }
            }
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
    s + &format!(
        " · pior escuro {:+.3} em d {:+.2} · FAMÍLIAS (a) {fam_a} (pior {pior_a:+.3}) · (b) {fam_b}",
        pior.0, pior.1
    )
}

const ERRADO_BORDA: f64 = 0.10;

/// As variantes da decisão (handoff W6d §6): o produto, o campo + a marcha de fora pela mesma borda, o
/// iso-½; o campo sozinho (§4) só na lista.
const DIAG: [&str; 4] = [
    "produto",
    "campo + fora",
    "sem borda exata",
    "campo",
];
/// As da decisão; as outras só na lista (`PH2D_W6D_LISTA`).
const N_DECIDE: usize = 3;

fn config_diag(gpu: &ph2d_gpu::GpuContext, a: Affine, v: usize) -> ph2d_light2d_gpu::Config {
    let c = config(gpu, a, false);
    match v {
        1 => ph2d_light2d_gpu::Config { borda_continua: true, borda_fora: true, ..c },
        2 => ph2d_light2d_gpu::Config { borda_exata: false, ..c },
        3 => ph2d_light2d_gpu::Config { borda_continua: true, ..c },
        _ => c,
    }
}

/// A distância de `p` à borda do polígono `v` (metros) e se está dentro.
fn ao_poligono(v: &[[f32; 2]], p: [f64; 2]) -> (f64, bool) {
    let mut d = f64::INFINITY;
    let mut dentro = false;
    let mut j = v.len() - 1;
    for i in 0..v.len() {
        let (a, b) = (v[j].map(f64::from), v[i].map(f64::from));
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            dentro = !dentro;
        }
        let ab = [b[0] - a[0], b[1] - a[1]];
        let t = (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18))
            .clamp(0.0, 1.0);
        d = d.min((p[0] - a[0] - t * ab[0]).hypot(p[1] - a[1] - t * ab[1]));
        j = i;
    }
    (d, dentro)
}

/// Os texels FORA de todo corpo, a ≥ 1 texel fino de toda borda, em que `x` e `y` diferem nos bits.
fn diferentes_fora(x: &Fluencia, y: &Fluencia, c: &CenaDeLuz, tx: f64) -> (usize, usize) {
    let g = x.grade;
    let (mut dif, mut n) = (0, 0);
    for j in 0..g.altura {
        for i in 0..g.largura {
            let k = (j * g.largura + i) as usize;
            let p = g.centro(i, j);
            let fora = c.oclusores.iter().all(|o| {
                let (d, dentro) = ao_poligono(&o.vertices, p);
                !dentro && d >= tx
            });
            if !fora {
                continue;
            }
            n += 1;
            dif += usize::from(x.v[k].map(f32::to_bits) != y.v[k].map(f32::to_bits));
        }
    }
    (dif, n)
}

#[test]
#[ignore = "o censo da borda da W6d: corre-se à mão (cabeçalho, nome medida_luz2d_w6d_borda)"]
fn medida_luz2d_w6d_borda() {
    let Some(gpu) = gpu() else { return };
    eprintln!("# censo da borda · loadavg antes {}", loadavg());
    // `PH2D_W6D_LISTA=1`: só o zoom 1,6, a lista dos pretos, sem o custo.
    let lista = std::env::var_os("PH2D_W6D_LISTA").is_some();
    let zooms: &[f64] = if lista { &[3.2] } else { &[0.8, 1.6, 3.2] };
    let nv = if lista { DIAG.len() } else { N_DECIDE };
    for &zoom in zooms {
        for k in 0..8u32 {
            let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
            let cfg0 = config(&gpu, a, false);
            let tx = cfg0.grade.texel_m / f64::from(cfg0.fina);
            let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
            let (c, redondos) = cena_redonda(d);
            let r = redondos[0];
            let mut lidas: Vec<Option<Fluencia>> = (0..nv).map(|_| None).collect();
            for j0 in 0..nv {
                let v = (j0 + k as usize) % nv;
                let mut s = solver(&gpu, a, config_diag(&gpu, a, v));
                s.quadro(&gpu, &c);
                let fl = fluencia_lida(&gpu, &s);
                let mut pior = Leitura::default();
                let aneis = linha(&fl, &r, &c, tx, &mut pior);
                eprintln!(
                    "BORDA · polígono · zoom {zoom} · {k}/8 · {}{}",
                    DIAG[v],
                    censo_da_borda(&fl, &r, &c, tx)
                );
                eprintln!(
                    "ANEIS · polígono · zoom {zoom} · {k}/8 · {} · PIOR {}e({:.3}) ×{} ↔{:.1} TV {:+.2}{aneis}",
                    DIAG[v], pior.errados, pior.pior, pior.cruzes, pior.desvio, pior.tv
                );
                lidas[v] = Some(fl);
            }
            let (p0, p1) = (lidas[0].as_ref().unwrap(), lidas[1].as_ref().unwrap());
            let (dif, n) = diferentes_fora(p0, p1, &c, tx);
            eprintln!("BITS · zoom {zoom} · {k}/8 · contínua × produto fora dos corpos: {dif} / {n} diferentes");
        }
        let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
        for v in 0..nv {
            let cfg = config_diag(&gpu, a, v);
            let tx = cfg.grade.texel_m / f64::from(cfg.fina);
            let (fl, r, gab) = disco_de_imagem(&gpu, a, cfg);
            let mut pior = Leitura::default();
            let aneis = linha(&fl, &r, &gab, tx, &mut pior);
            eprintln!(
                "BORDA · imagem · zoom {zoom} · {}{}",
                DIAG[v],
                censo_da_borda(&fl, &r, &gab, tx)
            );
            eprintln!(
                "ANEIS · imagem · zoom {zoom} · {} · PIOR {}e({:.3}) ×{} ↔{:.1} TV {:+.2}{aneis}",
                DIAG[v], pior.errados, pior.pior, pior.cruzes, pior.desvio, pior.tv
            );
        }
    }
    if !lista {
        custo(&gpu);
    }
    eprintln!("# loadavg depois {}", loadavg());
}

/// O custo da C3 (preenche — com a borda — + visibilidade + fina, ms, o MÍNIMO de 7 intercaladas),
/// cenas 5 e 6, vista de fábrica: o produto × a contínua.
fn custo(gpu: &ph2d_gpu::GpuContext) {
    use crate::medida_luz2d_w5d::{RODADAS, vista};
    let a = vista(1.0);
    let mut solvers = Vec::new();
    for n in [5u32, 6] {
        let mut w = bevy_ecs::world::World::new();
        assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        for v in 0..2 {
            solvers.push((n, v, cena.clone(), solver(gpu, a, config_diag(gpu, a, v))));
        }

    }
    let mut minimo = vec![f64::INFINITY; solvers.len()];
    let mut partes_min = vec![[0.0; 3]; solvers.len()];
    for rodada in 0..RODADAS {
        for j in 0..solvers.len() {
            let i = (j + rodada) % solvers.len();
            let (_, _, cena, s) = &mut solvers[i];
            for _ in 0..3 {
                s.quadro(gpu, cena);
            }
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            let partes = [t.c3_preenche, t.c3_visibilidade, t.c3_fina];
            if partes.iter().sum::<f64>() < minimo[i] {
                minimo[i] = partes.iter().sum();
                partes_min[i] = partes;
            }
        }
    }
    for (i, (n, v, ..)) in solvers.iter().enumerate() {
        let [a, b, c] = partes_min[i];
        let nome = DIAG[*v];
        eprintln!(
            "CUSTO · cena {n} · {nome} · C3 {:.3} ms (preenche+borda {a:.3} · visibilidade {b:.3} · fina {c:.3}) · loadavg {}",
            minimo[i],
            loadavg()
        );
    }
}

/// O ângulo (graus) entre a normal de fora do disco em `p` e a direção à lâmpada (0 = de frente).
fn angulo(centro: [f64; 2], p: [f64; 2], lampada: [f64; 2]) -> f64 {
    let n = [p[0] - centro[0], p[1] - centro[1]];
    let l = [lampada[0] - p[0], lampada[1] - p[1]];
    ((n[0] * l[0] + n[1] * l[1]) / (n[0].hypot(n[1]) * l[0].hypot(l[1])))
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}
