//! **A régua do custo dos cantos nas 7 câmeras** (handoff W6g §55 b / §56.1 — o alvo «C3 ≤ +5 % sobre a lei
//! W6d» em TODA câmera das cenas 5 e 6). À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_cantos_custo -- --ignored --nocapture --test-threads=1
//! ```
//!
//! O molde do `custo` da `medida_luz2d_w6d` (FHD, `DEGRAU_PADRAO`, sem rebote, a cena com os distantes que
//! tiver): por câmera, as [`VARIANTES`] num processo, [`RODADAS`] intercaladas em ordem rodada, o MÍNIMO da C3
//! (preenche + visibilidade + fina) e as parcelas da rodada mínima. A lei W6d entra duas vezes — o 2.º
//! solver é o controlo do ruído. As câmeras correm uma de cada vez (7 solvers vivos, não 49).

use std::collections::BTreeSet;

use ph2d_app_light2d::smoke;
use ph2d_light2d_gpu::{Canto, Config, VERTICE_PADRAO};
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::as_formas_redondas::config;
use crate::comum::gpu;
use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5d::{FHD, solver, vista};

const RODADAS: usize = 9;

#[derive(Clone, Copy)]
enum Camera {
    Zoom(f64),
    /// O centro em metros e a altura do mundo à vista (a de fábrica: 10 m).
    Centro([f32; 2], f32),
}

/// As 7 do §55 b: (cena, câmera).
const CAMERAS: [(u32, Camera); 7] = [
    (5, Camera::Centro([-6.0, -1.0], 2.5)),
    (5, Camera::Centro([6.0, -1.0], 2.5)),
    (5, Camera::Zoom(1.6)),
    (5, Camera::Zoom(1.0)),
    (6, Camera::Centro([0.0, -3.0], 1.5)),
    (6, Camera::Zoom(4.0)),
    (6, Camera::Zoom(1.0)),
];

pub(crate) const VARIANTES: [&str; 7] = [
    "lei W6d",
    "cunha",
    "cunha + V2",
    "cunha + canto",
    "padrão",
    "padrão sem a luz do canto",
    "lei W6d (controlo)",
];

fn afim(c: Camera) -> Affine {
    match c {
        Camera::Zoom(z) => vista(z),
        Camera::Centro(centro, h) => Camera2d::new(centro, h)
            .world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1)),
    }
}

fn nome(c: Camera) -> String {
    match c {
        Camera::Zoom(z) => format!("zoom {z}"),
        Camera::Centro([x, y], h) => format!("[{x}, {y}] h {h}"),
    }
}

pub(crate) fn variante(c: Config, v: usize) -> Config {
    let lei = Config {
        borda_vertice: None,
        ..c.sem_os_cantos()
    };
    let cunha = Config {
        borda_vertice: Some(VERTICE_PADRAO),
        ..c.sem_os_cantos()
    };
    match v {
        0 | 6 => lei,
        1 => cunha,
        2 => Config {
            borda_cantos: true,
            ..cunha
        },
        3 => Config {
            canto: Canto::LIGADO,
            ..cunha
        },
        4 => Config {
            borda_cantos: true,
            canto: Canto::LIGADO,
            ..cunha
        },
        _ => Config {
            borda_cantos: true,
            canto: Canto {
                luz: false,
                ..Canto::LIGADO
            },
            ..cunha
        },
    }
}

/// Os blocos 8×8 da captura com alguma cunha na `V2` (θ₁ < 50), o censo do `custo` da W6d.
fn blocos_com_cunha(gpu: &ph2d_gpu::GpuContext, s: &ph2d_light2d_gpu::Solver) -> Option<usize> {
    let (com, _) = mapa_das_cunhas(gpu, s)?;
    Some(com.iter().filter(|&&c| c).count())
}

/// Por bloco 8×8 da captura, se tem alguma cunha, e a largura em blocos.
fn mapa_das_cunhas(
    gpu: &ph2d_gpu::GpuContext,
    s: &ph2d_light2d_gpu::Solver,
) -> Option<(Vec<bool>, u32)> {
    let (b, [lx, ly]) = s.cantos_lidos(gpu)?;
    let (bx, by) = (lx.div_ceil(8), ly.div_ceil(8));
    let mut com = vec![false; (bx * by) as usize];
    for j in 0..ly {
        for i in 0..lx {
            let k = (j * lx + i) as usize * 8;
            if half::f16::from_le_bytes([b[k], b[k + 1]]).to_f32() < 50.0 {
                com[(j / 8 * bx + i / 8) as usize] = true;
            }
        }
    }
    Some((com, bx))
}

/// **Os grupos da fina PERTO de uma cunha** (handoff W6h §7.1 iii): por câmera, os grupos 8×8 da fina cujo
/// retângulo na captura, dilatado por `R` texels da FINA (as consultas do `num_corpo` à volta do texel; o limite
/// sem as sementes sem porta do `onde_calcular` é ~16,4 — W6h §7),
/// toca um bloco com cunha — a fração que a fina enxuta deixaria à pipeline de hoje. À mão:
///
/// ```text
/// PH2D_GPU=1 PH2D_CANTOS_RAIOS=1,2,4 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke \
///     --test it medida_luz2d_cantos_perto -- --ignored --nocapture
/// ```
#[test]
#[ignore = "o censo dos grupos perto das cunhas: corre-se à mão (cabeçalho)"]
fn medida_luz2d_cantos_perto() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    let raios: Vec<f32> = std::env::var("PH2D_CANTOS_RAIOS")
        .unwrap_or_else(|_| "1,2,4".into())
        .split(',')
        .map(|r| r.trim().parse().expect("PH2D_CANTOS_RAIOS"))
        .collect();
    for (n, cam) in CAMERAS {
        let mut w = bevy_ecs::world::World::new();
        assert!(smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        let a = afim(cam);
        let cfg = variante(config(&gpu, a, false), 4);
        let mut s = solver(&gpu, a, cfg);
        for _ in 0..3 {
            s.quadro(&gpu, &cena);
        }
        let (com, bx) = mapa_das_cunhas(&gpu, &s).expect("os cantos do padrão");
        let by = com.len() as u32 / bx;
        let (i, f) = ph2d_app_light2d::regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        // Texels da captura por texel da fina.
        let esc = cfg.captura as f32 / cfg.fina as f32;
        let (gx, gy) = ((f[0] - i[0]).div_ceil(8), (f[1] - i[1]).div_ceil(8));
        for &r in &raios {
            let rc = r * esc;
            let mut perto = 0;
            for y in 0..gy {
                for x in 0..gx {
                    let lo = [i[0] + 8 * x, i[1] + 8 * y].map(|v| (v as f32 + 0.5) * esc - rc);
                    let hi =
                        [i[0] + 8 * x + 7, i[1] + 8 * y + 7].map(|v| (v as f32 + 0.5) * esc + rc);
                    let bloco = |v: f32, m: u32| ((v.max(0.0) / 8.0) as u32).min(m - 1);
                    let toca = (bloco(lo[1], by)..=bloco(hi[1], by)).any(|j| {
                        (bloco(lo[0], bx)..=bloco(hi[0], bx)).any(|k| com[(j * bx + k) as usize])
                    });
                    perto += u32::from(toca);
                }
            }
            eprintln!(
                "PERTO · cena {n} · {} · R {r} · grupos {} · perto {perto} ({:.1} %) · captura/fina {esc}",
                nome(cam),
                gx * gy,
                100.0 * f64::from(perto) / f64::from(gx * gy)
            );
        }
        // A marcação do W6h §10: o grupo dilatado por 17 OU o retângulo dele até cada semente do
        // preenchimento que a janela 5×5 e as 3×3 células vizinhas do `onde_calcular` podem ler.
        let rc = 17.0 * esc;
        let toca = |lo: [f32; 2], hi: [f32; 2]| {
            let bloco = |v: f32, m: u32| ((v.max(0.0) / 8.0) as u32).min(m - 1);
            (bloco(lo[1] - rc, by)..=bloco(hi[1] + rc, by)).any(|j| {
                (bloco(lo[0] - rc, bx)..=bloco(hi[0] + rc, bx)).any(|k| com[(j * bx + k) as usize])
            })
        };
        let cel = s.config().preenchimento.expect("o preenchimento");
        let (celulas, saltos) = s.config().grade_do_preenchimento().expect("a grade");
        let nome_final = ["luz2d/tex/preenche_a", "luz2d/tex/preenche_b"][saltos.len() % 2];
        let (mut ts, _) = s.recursos_da_lista();
        let tp = ts
            .find(|(n, _)| *n == nome_final)
            .expect("o preenchimento")
            .1;
        let sementes = crate::comum::ler(&gpu, tp, 4);
        let semente = |cx: u32, cy: u32| {
            let o = 4 * (cy * celulas[0] + cx) as usize;
            let v = u32::from_le_bytes(sementes[o..o + 4].try_into().expect("u32"));
            (v != u32::MAX).then(|| [v & 0xffff, v >> 16].map(|t| (t as f32 + 0.5) * esc))
        };
        let (mut so_grupo, mut marcados) = (0u32, 0u32);
        for y in 0..gy {
            for x in 0..gx {
                let t0 = [i[0] + 8 * x, i[1] + 8 * y];
                let lo = t0.map(|v| (v as f32 + 0.5) * esc);
                let hi = t0.map(|v| (v as f32 + 7.5) * esc);
                if toca(lo, hi) {
                    so_grupo += 1;
                    marcados += 1;
                    continue;
                }
                let c_lo = t0.map(|v| (v.saturating_sub(2) / cel).saturating_sub(1));
                let c_hi = [0, 1].map(|k| ((t0[k] + 9) / cel + 1).min(celulas[k] - 1));
                let pela_semente = (c_lo[1]..=c_hi[1]).any(|cy| {
                    (c_lo[0]..=c_hi[0]).any(|cx| {
                        semente(cx, cy).is_some_and(|p| {
                            toca(
                                [lo[0].min(p[0]), lo[1].min(p[1])],
                                [hi[0].max(p[0]), hi[1].max(p[1])],
                            )
                        })
                    })
                });
                marcados += u32::from(pela_semente);
            }
        }
        eprintln!(
            "MARCA · cena {n} · {} · grupos {} · R 17 {so_grupo} ({:.1} %) · + sementes {marcados} ({:.1} %) · célula {cel}",
            nome(cam),
            gx * gy,
            100.0 * f64::from(so_grupo) / f64::from(gx * gy),
            100.0 * f64::from(marcados) / f64::from(gx * gy)
        );
    }
}

/// `PH2D_CANTOS_CUSTO_MODULOS=<pasta>`: cada `<pasta>/<nome>.wgsl` é mais uma variante — o PADRÃO com o
/// módulo das cascatas trocado (`Config::modulo_rc`), no mesmo processo (as formas de diagnóstico e as
/// candidatas da W6h §5.1, escritas sobre o `.wgsl` que o `medida_luz2d_regua_registos` grava).
fn modulos_extra() -> Vec<(String, &'static str)> {
    let Ok(pasta) = std::env::var("PH2D_CANTOS_CUSTO_MODULOS") else {
        return Vec::new();
    };
    let mut fs: Vec<_> = std::fs::read_dir(&pasta)
        .expect("PH2D_CANTOS_CUSTO_MODULOS")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|f| f.extension().is_some_and(|x| x == "wgsl"))
        .collect();
    fs.sort();
    fs.iter()
        .map(|f| {
            let nome = f.file_stem().expect("nome").to_string_lossy().into_owned();
            let texto = std::fs::read_to_string(f).expect("o módulo");
            (nome, &*Box::leak(texto.into_boxed_str()))
        })
        .collect()
}

#[test]
#[ignore = "a régua do custo dos cantos: corre-se à mão (cabeçalho)"]
fn medida_luz2d_cantos_custo() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Régua do custo dos cantos · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "otimizado"
        },
        loadavg()
    );
    let extras = modulos_extra();
    let nomes: Vec<&str> = VARIANTES
        .iter()
        .copied()
        .chain(extras.iter().map(|(n, _)| n.as_str()))
        .collect();
    for (n, cam) in CAMERAS {
        let mut w = bevy_ecs::world::World::new();
        assert!(smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        let a = afim(cam);
        let base = config(&gpu, a, false);
        let mut solvers: Vec<_> = (0..nomes.len())
            .map(|v| match extras.get(v.wrapping_sub(VARIANTES.len())) {
                Some(&(_, m)) => solver(
                    &gpu,
                    a,
                    Config {
                        modulo_rc: Some(m),
                        ..variante(base, 4)
                    },
                ),
                None => solver(&gpu, a, variante(base, v)),
            })
            .collect();
        let mut minimo = vec![f64::INFINITY; solvers.len()];
        let mut partes_min = vec![[0.0; 4]; solvers.len()];
        for rodada in 0..RODADAS {
            for j in 0..solvers.len() {
                let i = (j + rodada) % solvers.len();
                for _ in 0..3 {
                    solvers[i].quadro(&gpu, &cena);
                }
                let t = solvers[i].tempos(&gpu).expect("TIMESTAMP_QUERY");
                let c3 = t.c3_preenche + t.c3_visibilidade + t.c3_fina;
                if c3 < minimo[i] {
                    minimo[i] = c3;
                    partes_min[i] = [t.c3_preenche, t.c3_borda, t.c3_visibilidade, t.c3_fina];
                }
            }
        }
        let distantes = solvers[0].quadro(&gpu, &cena).distantes;
        let blocos = blocos_com_cunha(&gpu, &solvers[4]);
        let recusados = solvers[4].cantos_recusados(&gpu);
        eprintln!(
            "CAMERA · cena {n} · {} · distantes {distantes} · blocos com cunha {blocos:?} · recusados {recusados:?} · loadavg {}",
            nome(cam),
            loadavg()
        );
        for (v, nome_v) in nomes.iter().enumerate() {
            let [p, b, vis, f] = partes_min[v];
            eprintln!(
                "CUSTO · cena {n} · {} · {nome_v} · C3 {:.3} ms ({:+.1} % sobre a lei W6d) · preenche {p:.3} [borda {b:.3}] · visibilidade {vis:.3} · fina {f:.3}",
                nome(cam),
                minimo[v],
                100.0 * (minimo[v] / minimo[0] - 1.0)
            );
        }
    }
    eprintln!("# loadavg depois {}", loadavg());
}

/// Algum bloco 8×8 da captura com entrada na tabela dos cantos (superconjunto das cunhas) no retângulo do
/// segmento `p0`–`ps` dilatado por 17 texels da fina (o raio do `num_corpo`, W6h §8 i).
const PERTO_DE_CUNHA: &str = "
fn perto_de_cunha(p0: vec2<f32>, ps: vec2<f32>, h: f32) -> bool {
    let n = vec2<i32>(vec2<f32>(cas.tamanho) * ESCALA_DA_CAPTURA);
    let r = 17.0 * h * ESCALA_DA_CAPTURA;
    let lo = clamp(vec2<i32>(floor(min(p0, ps) * ESCALA_DA_CAPTURA - r)), vec2<i32>(0), n - 1) / 8;
    let hi = clamp(vec2<i32>(floor(max(p0, ps) * ESCALA_DA_CAPTURA + r)), vec2<i32>(0), n - 1) / 8;
    for (var y = lo.y; y <= hi.y; y++) {
        for (var x = lo.x; x <= hi.x; x++) {
            if cantos_tabela[y * ((n.x + 7) / 8) + x] != 0u {
                return true;
            }
        }
    }
    return false;
}
";

/// O módulo do padrão com a sonda `7` (W6h §9.1 b): o texel que tomou uma semente do JFA SEM PORTA no
/// `onde_calcular` (a janela 5×5 sem texel de fora: nenhum filtro de distância) grava `-7` e a maior distância
/// às sementes, em texels da fina; os outros da fina `-3`. A mistura do translúcido sai com a sonda. `fundo`: o
/// `FUNDO_SEM_LUZ` ligado à força (o que o miolo do padrão liga; com ele a sonda não chega à fluência).
fn modulo_sem_porta(cfg: &Config, fundo: bool) -> &'static str {
    let mut f = ph2d_light2d_gpu::fonte_do_rc(
        cfg.aritmetica,
        cfg.armazenamento,
        cfg.hrc_niveis_em_textura(),
    )
    .to_owned();
    let troca = |f: &mut String, velho: &str, novo: &str| {
        assert_eq!(f.matches(velho).count(), 1, "{velho}");
        *f = f.replace(velho, novo);
    };
    troca(
        &mut f,
        "var<private> canto_adiado: bool = false;",
        "var<private> canto_adiado: bool = false;\nvar<private> sem_porta: f32 = -1.0;\nvar<private> sem_porta_cunha: bool = false;",
    );
    troca(
        &mut f,
        "                cand[n_cand] = na_fronteira(P0, ps);",
        "                if na_janela == 0u {\n                    sem_porta = max(sem_porta, distance(ps, P0) / h);\n                    sem_porta_cunha = sem_porta_cunha || perto_de_cunha(P0, ps, h);\n                }\n                cand[n_cand] = na_fronteira(P0, ps);",
    );
    troca(
        &mut f,
        "        case 6u: { return vec3<f32>((onde.p2 - P0) / h, onde.f2); }",
        "        case 6u: { return vec3<f32>((onde.p2 - P0) / h, onde.f2); }\n        case 7u: { return vec3<f32>(select(-3.0, -7.0, sem_porta >= 0.0), max(sem_porta, 0.0), select(0.0, 1.0, sem_porta_cunha)); }",
    );
    f.push_str(PERTO_DE_CUNHA);
    troca(
        &mut f,
        "    if !MIOLO && translucido {",
        "    if !MIOLO && translucido && SONDA == 0u {",
    );
    if fundo {
        troca(
            &mut f,
            "    if !FUNDO_SEM_LUZ {\n        return false;\n    }\n    if lid == 0u {",
            "    if lid == 0u {",
        );
        troca(
            &mut f,
            "    if !FUNDO_SEM_LUZ || tau > 0.0 ||",
            "    if tau > 0.0 ||",
        );
    }
    Box::leak(f.into_boxed_str())
}

/// **O censo das sementes sem porta** (handoff W6h §9.1): por cena 1–8 (vista de fábrica) e pelas 7 câmeras,
/// o padrão sem miolo (a sonda só chega à fluência sem ele) com o [`modulo_sem_porta`], a saltar o fundo como o
/// miolo do padrão faz e sem saltar:
/// os texels da fina, os que tomaram semente sem porta, os grupos 8×8 deles e a maior distância. À mão:
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
///     medida_luz2d_sementes_sem_porta -- --ignored --nocapture
/// ```
#[test]
#[ignore = "o censo das sementes sem porta: corre-se à mão (cabeçalho)"]
fn medida_luz2d_sementes_sem_porta() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    let fabrica = (1..=8u32).map(|n| (n, Camera::Zoom(1.0)));
    for (n, cam) in fabrica.chain(CAMERAS) {
        let mut w = bevy_ecs::world::World::new();
        assert!(smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        let a = afim(cam);
        let padrao = variante(config(&gpu, a, false), 4);
        for (rotulo, fundo) in [("salta o fundo", true), ("sem saltar", false)] {
            let cfg = Config {
                miolo: None,
                sonda: 7,
                modulo_rc: Some(modulo_sem_porta(&padrao, fundo)),
                ..padrao
            };
            let mut s = solver(&gpu, a, cfg);
            for _ in 0..3 {
                s.quadro(&gpu, &cena);
            }
            let t = s.fluencia();
            let f32_ = t.format() == wgpu::TextureFormat::Rgba32Float;
            let bpp = if f32_ { 16 } else { 8 };
            let b = crate::comum::ler(&gpu, t, bpp);
            let canal = |k: usize, c: usize| {
                let o = k * bpp as usize + c * bpp as usize / 4;
                if f32_ {
                    f32::from_le_bytes(b[o..o + 4].try_into().expect("f32"))
                } else {
                    half::f16::from_le_bytes([b[o], b[o + 1]]).to_f32()
                }
            };
            let (largura, altura) = (t.width() as usize, t.height() as usize);
            let (mut fina, mut sem, mut longe, mut max, mut cunha) = (0u64, 0u64, 0u64, 0f32, 0u64);
            let (mut grupos, mut grupos_cunha) = (BTreeSet::new(), BTreeSet::new());
            for k in 0..largura * altura {
                let r = canal(k, 0);
                if r == -3.0 {
                    fina += 1;
                } else if r == -7.0 {
                    fina += 1;
                    sem += 1;
                    let d = canal(k, 1);
                    max = max.max(d);
                    longe += u64::from(d > 17.0);
                    grupos.insert((k % largura / 8, k / largura / 8));
                    if canal(k, 2) == 1.0 {
                        cunha += 1;
                        grupos_cunha.insert((k % largura / 8, k / largura / 8));
                    }
                }
            }
            eprintln!(
                "SEM PORTA · cena {n} · {} · {rotulo} · fina {fina} · sem porta {sem} ({:.3} %) · grupos ~{} · > 17 texels {longe} · max {max:.1} · perto de cunha {cunha} em ~{} grupos",
                nome(cam),
                100.0 * sem as f64 / fina.max(1) as f64,
                grupos.len(),
                grupos_cunha.len()
            );
        }
    }
}
