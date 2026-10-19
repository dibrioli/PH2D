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
use crate::o_campo_da_borda::ao_poligono;

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
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
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
            let (x, y) = (
                (p.x - r.centro[0]) / r.semi[0],
                (p.y - r.centro[1]) / r.semi[1],
            );
            let dentro = x * x + y * y <= 1.0;
            px.extend(pinta(luz([p.x, p.y]), if dentro { cor } else { [0.85; 3] }));
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
            foto(&format!("imagem_zoom{zoom}_{quando}"), a, &r, cor, |p| {
                fl.amostra(p)
            });
            let mut s = solver(&gpu, a, cfg);
            s.quadro(&gpu, &c_pol);
            let fl = fluencia_lida(&gpu, &s);
            foto(
                &format!("poligono_zoom{zoom}_{quando}"),
                a,
                &r_pol,
                cor,
                |p| fl.amostra(p),
            );
        }
    }
    // O gabarito não depende da vista: uma foto, na escala do zoom 1.
    let a = vista_em(1.0, r_pol.centro);
    foto("gabarito", a, &r_pol, cor, |p| {
        ph2d_light2d_ref::fluencia(&c_pol, p)[1]
    });
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
            let (g0, ga, gb) = (
                lei(p),
                lei([p[0] + t[0], p[1] + t[1]]),
                lei([p[0] - t[0], p[1] - t[1]]),
            );
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

/// As variantes da decisão: antes da W6d (a reta por texel, sem o fora) e o produto; o iso-½ só na
/// lista. (As rodadas 9–12 mediram também a união e o empréstimo pela normal própria: handoff W6d §16–§23.)
const DIAG: [&str; 6] = [
    "antes da W6d",
    "lei W6d",
    "cunha",
    "cunha + 10 + 9",
    "cunha larga",
    "sem borda exata",
];
/// As da decisão; as outras só na lista (`PH2D_W6D_LISTA`).
const N_DECIDE: usize = 4;

fn config_diag(gpu: &ph2d_gpu::GpuContext, a: Affine, v: usize) -> ph2d_light2d_gpu::Config {
    let c = config(gpu, a, false);
    match v {
        0 => ph2d_light2d_gpu::Config {
            borda_continua: false,
            borda_fora: false,
            ..c
        },
        1 => ph2d_light2d_gpu::Config {
            borda_vertice: None,
            ..c
        },
        2 => ph2d_light2d_gpu::Config {
            borda_vertice: Some(ph2d_light2d_gpu::VERTICE_PADRAO),
            ..c
        },
        // ⭐ W6f: o campo que guarda o vértice e a face no canto (handoff W6f §8).
        3 => ph2d_light2d_gpu::Config {
            borda_vertice: Some(ph2d_light2d_gpu::VERTICE_PADRAO),
            borda_cantos: true,
            canto: ph2d_light2d_gpu::Canto::LIGADO,
            ..c
        },
        4 => ph2d_light2d_gpu::Config {
            borda_vertice: Some(ph2d_light2d_gpu::Vertice {
                pre: 0.02,
                area: 0.05,
            }),
            ..c
        },
        5 => ph2d_light2d_gpu::Config {
            borda_exata: false,
            ..c
        },
        _ => c,
    }
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
    // `PH2D_W6D_CUSTO=<n>`: só o custo, `n` vezes no mesmo processo.
    if let Some(n) = std::env::var("PH2D_W6D_CUSTO")
        .ok()
        .and_then(|n| n.parse::<u32>().ok())
    {
        for _ in 0..n {
            custo(&gpu);
        }
        return;
    }
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
            let antes = lidas[0].as_ref().unwrap();
            let mut linha = String::new();
            for v in 1..nv {
                let (dif, n) = diferentes_fora(antes, lidas[v].as_ref().unwrap(), &c, tx);
                linha += &format!(" · {} {dif}/{n}", DIAG[v]);
            }
            eprintln!("BITS · zoom {zoom} · {k}/8 · fora dos corpos, diferentes de antes{linha}");
        }
        let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
        for (v, nome) in DIAG.iter().enumerate().take(nv) {
            let cfg = config_diag(&gpu, a, v);
            let tx = cfg.grade.texel_m / f64::from(cfg.fina);
            let (fl, r, gab) = disco_de_imagem(&gpu, a, cfg);
            let mut pior = Leitura::default();
            let aneis = linha(&fl, &r, &gab, tx, &mut pior);
            eprintln!(
                "BORDA · imagem · zoom {zoom} · {nome}{}",
                censo_da_borda(&fl, &r, &gab, tx)
            );
            eprintln!(
                "ANEIS · imagem · zoom {zoom} · {nome} · PIOR {}e({:.3}) ×{} ↔{:.1} TV {:+.2}{aneis}",
                pior.errados, pior.pior, pior.cruzes, pior.desvio, pior.tv
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
        for v in 0..N_DECIDE {
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

/// Uma variante da sonda do canto: o nome e a troca na configuração.
type Variante = (
    String,
    Box<dyn Fn(ph2d_light2d_gpu::Config) -> ph2d_light2d_gpu::Config>,
);
/// O gabarito dos cantos e o máximo do interior, por lei (antiga, difusão).
type GabaritosDosCantos = [(Vec<([f64; 2], f64)>, f64); 2];

/// ⭐ W6e — as variantes do canto: antes da W6d, a lei da W6d e a cunha com `(pre, area)`.
const CANTO_CUNHAS: [(f32, f32); 9] = [
    (0.02, 0.01),
    (0.02, 0.02),
    (0.02, 0.05),
    (0.05, 0.01),
    (0.05, 0.02),
    (0.05, 0.05),
    (0.10, 0.01),
    (0.10, 0.02),
    (0.10, 0.05),
];

/// ⭐ W6e (o mecanismo: handoff W6d §14) — os QUATRO cantos da caixa do `o_interior_do_opaco` (os casos do
/// portão B: zoom 1 e 2,5 × fase 0, 3, 4, 6/8) contra o gabarito dos cantos, nas variantes antes da W6d ·
/// a lei da W6d · a cunha × [`CANTO_CUNHAS`]; a lei antiga do miolo (`miolo: None`) e o produto (a
/// difusão) nas três primeiras. O texel do aberto (zoom 1, 3/8, cima-direita) sai à parte. À mão
/// (cabeçalho, nome `medida_luz2d_w6d_canto`).
#[test]
#[ignore = "a sonda do canto: corre-se à mão"]
fn medida_luz2d_w6d_canto() {
    use crate::o_interior_do_opaco::{EXTERIOR, cena, gabarito_dos_cantos, gabarito_por};
    use ph2d_app_light2d::{
        DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela,
    };
    use ph2d_light2d_gpu::{Config, Vertice};
    let Some(gpu) = gpu() else {
        return;
    };
    eprintln!("# canto · loadavg antes {}", loadavg());
    let fhd = (1920u32, 1080u32);
    let fabrica = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(fhd.0, fhd.1));
    let dg = DEGRAU_PADRAO;
    let mut variantes: Vec<Variante> = vec![
        (
            "antes da W6d".into(),
            Box::new(|c| Config {
                borda_continua: false,
                borda_fora: false,
                ..c
            }),
        ),
        (
            "lei W6d".into(),
            Box::new(|c| Config {
                borda_vertice: None,
                ..c
            }),
        ),
    ];
    for (pre, area) in CANTO_CUNHAS {
        variantes.push((
            format!("cunha {pre}/{area}"),
            Box::new(move |c| Config {
                borda_vertice: Some(Vertice { pre, area }),
                ..c
            }),
        ));
    }
    // [variante][lei] — (errados, pior) somados nos casos.
    let mut resumo = vec![[(0usize, 0.0_f64); 2]; variantes.len()];
    for zoom in [1.0, 2.5] {
        let a = fabrica.then_scale_about(zoom, Point::new(960.0, 540.0));
        let grade = grade_do_quadro(a, fhd, dg).expect("grade");
        let base = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg
                .holografica
                .then_some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO),
            rebote: false,
            ..Config::padrao_para(&gpu.device, grade)
        };
        let tx = grade.texel_m / f64::from(base.fina);
        for k in [0u32, 3, 4, 6] {
            let d = [
                (f64::from(k) * tx / 8.0) as f32,
                (f64::from(k) * tx * 3.0 / 8.0) as f32,
            ];
            let c = cena(d);
            let antiga = ph2d_light2d_ref::Gabarito::com_miolo(&c, EXTERIOR);
            let mut gabs: Option<GabaritosDosCantos> = None;
            for (vi, (nome, f)) in variantes.iter().enumerate() {
                for lei in 0..2 {
                    if lei == 1 && ![0, 1, 6].contains(&vi) {
                        continue;
                    }
                    let cfg = f(if lei == 0 {
                        Config {
                            miolo: None,
                            ..base
                        }
                    } else {
                        base
                    });
                    let mut s = ph2d_light2d_gpu::Solver::novo(&gpu, cfg).expect("solver");
                    let (i, fi) = regiao_da_janela(s.grade_da_fluencia(), a, fhd);
                    s.regiao_fina(i, fi);
                    s.quadro(&gpu, &c);
                    let fl = fluencia_lida(&gpu, &s);
                    let g = fl.grade;
                    let gabs = gabs.get_or_insert_with(|| {
                        let por = |l: &dyn Fn([f64; 2]) -> f64| {
                            let max = gabarito_por(g, d, l)
                                .0
                                .iter()
                                .map(|x| x.1)
                                .fold(0.0, f64::max);
                            (gabarito_dos_cantos(g, d, l), max)
                        };
                        [
                            por(&|p| antiga.fluencia(p)[1]),
                            por(&|p| ph2d_light2d_ref::fluencia(&c, p)[1]),
                        ]
                    });
                    let (gab, max) = &gabs[lei];
                    let mut linha = String::new();
                    for &(p, gv) in gab {
                        let v = fl.amostra(p);
                        let e = (v - gv) / max;
                        resumo[vi][lei].0 += usize::from(e.abs() > 0.10);
                        if e.abs() > resumo[vi][lei].1.abs() {
                            resumo[vi][lei].1 = e;
                        }
                        linha += &format!(" ({:.4}; {:.4}) {v:.3}/{gv:.3}", p[0], p[1]);
                    }
                    eprintln!(
                        "CANTO · zoom {zoom} · {k}/8 · {} · {nome} ·{linha}",
                        ["lei antiga", "difusão"][lei]
                    );
                    if zoom == 1.0 && k == 3 && lei == 0 {
                        let v = fl.amostra([1.8516, 0.9141]);
                        eprintln!("ABERTO · o texel (1,8516; 0,9141) · {nome}: {v:.3}");
                    }
                }
            }
        }
    }
    for (vi, (nome, _)) in variantes.iter().enumerate() {
        let [(na, pa), (nd, pd)] = resumo[vi];
        eprintln!(
            "RESUMO CANTO · {nome} · lei antiga {na} errados (pior {pa:+.3}) · difusão {nd} (pior {pd:+.3})"
        );
    }
    eprintln!("# loadavg depois {}", loadavg());
}
