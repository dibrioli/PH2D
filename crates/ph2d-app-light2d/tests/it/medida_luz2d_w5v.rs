//! ⭐ W5v — o custo da CENA DO DONO (cena 5 do smoke) com a configuração do APP
//! ([`ph2d_app_light2d::config_da_vista`]: a aritmética do dispositivo, as chaves de ambiente), um
//! caso por processo — o relógio da placa por passe que o `PH2D_LIGHT2D_CUSTO=1` lê no app inteiro. As
//! réguas antigas (cena 1 quase vazia, aritmética F32) davam fina 0,24 onde o app dá 3,2 (handoff W5u §7).
//!
//! Um caso: `PH2D_W5V_CENA=1..8` · `PH2D_W5V_NIVEL=low|medium|high` · `PH2D_W5V_VAR=<variante>` (as da
//! fina, [`variante`]) · `PH2D_W5V_ALVO=<l>x<a>` (a janela do canvas; omissão 1920×1080) ·
//! `PH2D_W5V_FASES=1` (as etapas da holográfica). O roteiro intercala os processos.

use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::{config_da_vista, config_do_entorno, grade_do_quadro, regiao_da_janela};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Refino, Solver, TemposMs};
use vello::kurbo;

use crate::medida_luz2d_w5e::fabrica;
use crate::medida_luz2d_w5o::{mediana, minimo};

const AQUECE: usize = 12;
const QUADROS: usize = 24;

/// As variantes da FINA (atribuição do relógio, não o bit): cada uma desliga ou barateia uma peça.
fn variante(nome: &str, c: Config) -> Config {
    let bordas = |f: &dyn Fn(&mut ph2d_light2d_gpu::Bordas)| match c.refino {
        Refino::Bordas(mut b) => {
            f(&mut b);
            Refino::Bordas(b)
        }
        r @ Refino::Tudo => r,
    };
    match nome {
        "base" => c,
        "tudo" => Config {
            refino: Refino::Tudo,
            ..c
        },
        "raios2" => Config {
            refino: bordas(&|b| b.raios = 2),
            ..c
        },
        "bissec2" => Config {
            refino: bordas(&|b| b.bissecoes = 2),
            ..c
        },
        "limiar4" => Config {
            refino: bordas(&|b| b.limiar *= 4.0),
            ..c
        },
        "sem_curvatura" => Config {
            refino: bordas(&|b| b.curvatura = false),
            ..c
        },
        "sem_joelho" => Config {
            refino: bordas(&|b| b.joelho = false),
            ..c
        },
        "vis_grossa" => Config {
            refino: bordas(&|b| b.visibilidade_no_passo_fino = false),
            ..c
        },
        "sem_face" => Config {
            vis_na_face: false,
            face_pelo_ponto: false,
            ..c
        },
        "sem_pe" => Config {
            pe_na_borda: false,
            ..c
        },
        "sem_borda_exata" => Config {
            borda_exata: false,
            pe_na_borda: false,
            ..c
        },
        "sem_media_eixo" => Config {
            media_no_eixo: false,
            ..c
        },
        "sem_ao_longo" => Config {
            c0_ao_longo_da_face: 0.0,
            ..c
        },
        "sem_distantes" => Config {
            distantes: false,
            ..c
        },
        "passo2" => Config {
            passo_direto: c.passo_direto * 2.0,
            ..c
        },
        "sem_anel" => Config {
            anel_fora_da_regiao: false,
            ..c
        },
        "sem_rebote" => Config { rebote: false, ..c },
        "fundo_calculado" => Config {
            miolo_salta_o_fundo: false,
            ..c
        },
        v => panic!("PH2D_W5V_VAR = {v}: não há esta variante"),
    }
}

fn nivel_do_ambiente() -> NivelDaLuz {
    match std::env::var("PH2D_W5V_NIVEL").as_deref() {
        Ok("low") => NivelDaLuz::Baixa,
        Ok("medium") => NivelDaLuz::Media,
        Ok("high") | Err(_) => NivelDaLuz::Alta,
        Ok(n) => panic!("PH2D_W5V_NIVEL = low | medium | high, não {n}"),
    }
}

fn alvo_do_ambiente() -> (u32, u32) {
    std::env::var("PH2D_W5V_ALVO").map_or((1920, 1080), |v| {
        let (l, a) = v.split_once('x').expect("PH2D_W5V_ALVO = <l>x<a>");
        (l.parse().expect("largura"), a.parse().expect("altura"))
    })
}

/// A vista e o entorno com a configuração do APP (a do `LuzDoQuadro`), trocada por `com`.
pub(crate) struct App {
    pub(crate) vista: Solver,
    pub(crate) entorno: Option<Solver>,
}

impl App {
    pub(crate) fn novo(
        gpu: &ph2d_gpu::GpuContext,
        a: kurbo::Affine,
        alvo: (u32, u32),
        nivel: NivelDaLuz,
        com: &dyn Fn(Config) -> Config,
    ) -> Self {
        let dg = degrau_do(nivel);
        let g = grade_do_quadro(a, alvo, dg).expect("a grade");
        let mut vista = Solver::novo(gpu, com(config_da_vista(&gpu.device, g, a, dg)))
            .expect("solver da vista");
        let (i, f) = regiao_da_janela(vista.grade_da_fluencia(), a, alvo);
        vista.regiao_fina(i, f);
        let mut entorno = config_do_entorno(&gpu.device, a, alvo, dg)
            .map(|c| Solver::novo(gpu, com(c)).expect("solver do entorno"));
        if let Some(e) = entorno.as_mut() {
            let g = vista.config().grade;
            e.mascarar_miolo(
                gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ]),
            );
        }
        vista.somar_entorno(gpu, entorno.as_ref());
        Self { vista, entorno }
    }

    pub(crate) fn quadro(&mut self, gpu: &ph2d_gpu::GpuContext, cena: &CenaDeLuz) {
        if let Some(e) = self.entorno.as_mut() {
            let sem_ceu = CenaDeLuz {
                ceu: [0.0; 3],
                ..cena.clone()
            };
            e.quadro(gpu, &sem_ceu);
        }
        self.vista.quadro(gpu, cena);
    }
}

/// A cena `n` do smoke como o app a coleta.
pub(crate) fn cena_do_smoke(n: u32) -> CenaDeLuz {
    let mut w = bevy_ecs::world::World::new();
    assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
    ph2d_light2d_ecs::coleta::cena_de_luz(&w)
}

/// ⭐ W5v — um caso por processo, a configuração do APP: o relógio por passe da vista e do entorno.
#[test]
#[ignore = "rodada de medição W5v (placa): o custo da cena do dono com a configuração do app"]
fn medida_luz2d_w5v_caso() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let n: u32 = std::env::var("PH2D_W5V_CENA").map_or(5, |v| v.parse().expect("PH2D_W5V_CENA"));
    let nivel = nivel_do_ambiente();
    let var = std::env::var("PH2D_W5V_VAR").unwrap_or_else(|_| "base".into());
    let alvo = alvo_do_ambiente();
    let cena = cena_do_smoke(n);
    // `PH2D_W5V_CAMERA=<x>,<y>,<altura em m>` (omissão: a de fábrica).
    let a = std::env::var("PH2D_W5V_CAMERA").map_or_else(
        |_| fabrica(alvo),
        |v| {
            let n: Vec<f32> = v
                .split(',')
                .map(|x| x.parse().expect("PH2D_W5V_CAMERA"))
                .collect();
            ph2d_render::Camera2d::new([n[0], n[1]], n[2])
                .world_to_screen_affine(ph2d_host::WindowSize::new(alvo.0, alvo.1))
        },
    );
    // `PH2D_W5V_MUTA=<de>⇒<para>[⇐…]`: o texto do módulo das cascatas mutado (experimentos).
    let muta = |c: Config| match std::env::var("PH2D_W5V_MUTA") {
        Ok(m) => {
            let mut fonte =
                ph2d_light2d_gpu::fonte_do_rc(c.aritmetica, c.armazenamento).to_string();
            for par in m.split('⇐') {
                let (de, para) = par.split_once('⇒').expect("<de>⇒<para>");
                assert_eq!(fonte.matches(de).count(), 1, "a agulha: {de}");
                fonte = fonte.replacen(de, para, 1);
            }
            Config {
                modulo_rc: Some(Box::leak(fonte.into_boxed_str())),
                ..c
            }
        }
        Err(_) => c,
    };
    let mut app = App::novo(&gpu, a, alvo, nivel, &|c| muta(variante(&var, c)));
    let (vista, entorno) = (&mut app.vista, &mut app.entorno);
    let fluencia = vista.grade_da_fluencia();
    let (i, f) = regiao_da_janela(fluencia, a, alvo);
    let rotulo = format!("cena {n} · {nivel:?} · {var} · {}×{}", alvo.0, alvo.1);
    let c = vista.config();
    eprintln!(
        "O {rotulo} · grade {}×{} · fina ×{} captura ×{} · fluência {}×{} região {}×{} · {:?} · {} emissores {} oclusores · load {}",
        c.grade.largura,
        c.grade.altura,
        c.fina,
        c.captura,
        fluencia.largura,
        fluencia.altura,
        f[0] - i[0],
        f[1] - i[1],
        c.aritmetica,
        cena.emissores.len(),
        cena.oclusores.len(),
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let sem_ceu = CenaDeLuz {
        ceu: [0.0; 3],
        ..cena.clone()
    };
    let quadro = |vista: &mut Solver, entorno: &mut Option<Solver>| {
        if let Some(e) = entorno.as_mut() {
            e.quadro(&gpu, &sem_ceu);
        }
        vista.quadro(&gpu, &cena);
    };
    for q in 0..AQUECE {
        let t0 = std::time::Instant::now();
        quadro(vista, entorno);
        let t = vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
        eprintln!(
            "A {rotulo} · quadro {q} · parede {:.1} ms · fina {:.3} · miolo {:.3} · total {:.3}",
            t0.elapsed().as_secs_f64() * 1e3,
            t.c3_fina,
            t.c3_miolo,
            t.total()
        );
    }
    let mut series: Vec<(TemposMs, f64)> = Vec::new();
    for _ in 0..QUADROS {
        quadro(vista, entorno);
        let t = vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
        let e = entorno
            .as_ref()
            .map_or(0.0, |e| e.tempos(&gpu).expect("TIMESTAMP_QUERY").total());
        series.push((t, e));
    }
    type Coluna = (&'static str, fn(&(TemposMs, f64)) -> f64);
    const PASSES: [Coluna; 13] = [
        ("custo", |(t, e)| t.total() + e),
        ("c0", |(t, _)| t.c0),
        ("c1", |(t, _)| t.c1),
        ("c2", |(t, _)| t.c2_total()),
        ("vis", |(t, _)| t.c3_visibilidade),
        ("fina", |(t, _)| t.c3_fina),
        ("c0_fina", |(t, _)| t.c0_fina),
        ("preenche", |(t, _)| t.c3_preenche),
        ("másc", |(t, _)| t.c3_mascaras),
        ("rebote", |(t, _)| t.c0_rebote + t.c3_rebote),
        ("miolo", |(t, _)| t.c3_miolo),
        ("vista", |(t, _)| t.total()),
        ("entorno", |(_, e)| *e),
    ];
    let q: Vec<String> = PASSES
        .iter()
        .map(|(nome, ler)| {
            let v: Vec<f64> = series.iter().map(ler).collect();
            format!("{nome} {:.3}/{:.3}", minimo(&v), mediana(&v))
        })
        .collect();
    eprintln!("Q {rotulo} · mín/med: {}", q.join(" · "));
    let serie = |ler: fn(&(TemposMs, f64)) -> f64| {
        series
            .iter()
            .map(|x| format!("{:.2}", ler(x)))
            .collect::<Vec<_>>()
            .join(" ")
    };
    eprintln!("Z {rotulo} · fina: {}", serie(PASSES[5].1));
    eprintln!("Z {rotulo} · vis: {}", serie(PASSES[4].1));
    eprintln!("Z {rotulo} · c2: {}", serie(PASSES[3].1));
    if std::env::var("PH2D_W5V_FASES").as_deref() == Ok("1") && vista.medir_fases_da_hrc(&gpu) {
        let mut por: Vec<(&'static str, u32, Vec<f64>)> = Vec::new();
        for _ in 0..QUADROS / 2 {
            quadro(vista, entorno);
            for (etapa, nivel, ms, _) in vista.fases_da_hrc(&gpu).expect("fases") {
                match por.iter_mut().find(|o| o.0 == etapa && o.1 == nivel) {
                    Some(o) => o.2.push(ms),
                    None => por.push((etapa, nivel, vec![ms])),
                }
            }
        }
        let mut etapas: Vec<(&'static str, f64)> = Vec::new();
        for (etapa, _, ms) in &por {
            match etapas.iter_mut().find(|o| o.0 == *etapa) {
                Some(o) => o.1 += minimo(ms),
                None => etapas.push((etapa, minimo(ms))),
            }
        }
        let s: Vec<String> = etapas
            .iter()
            .map(|(e, ms)| format!("{e} {ms:.3}"))
            .collect();
        eprintln!(
            "F {rotulo} · etapas (mín, soma dos níveis): {}",
            s.join(" · ")
        );
    }
    let bytes = crate::comum::ler(&gpu, vista.fluencia(), 8);
    let digital = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    });
    eprintln!(
        "H {rotulo} · a fluência (FNV-1a de {} bytes): {digital:016x}",
        bytes.len()
    );
    eprintln!(
        "W5v fim · {rotulo} · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}

/// O texto do módulo das cascatas (que leva o `c3_direta.wgsl`) com cada `de` trocado por `para` (uma vez).
fn mutado(cfg: &Config, trocas: &[(&str, &str)]) -> &'static str {
    let mut fonte = ph2d_light2d_gpu::fonte_do_rc(cfg.aritmetica, cfg.armazenamento).to_string();
    for (de, para) in trocas {
        assert_eq!(fonte.matches(de).count(), 1, "a agulha do controlo: {de}");
        fonte = fonte.replacen(de, para, 1);
    }
    Box::leak(fonte.into_boxed_str())
}

/// ⭐ W5v — a fina que SALTA o fundo dos opacos ([`Config::miolo_salta_o_fundo`]) contra a que o calcula,
/// no MESMO processo e com a configuração do APP: a fluência da vista ao BIT (80 quadros: o miolo e o
/// rebote assentados) nos casos da W5q (cenas 1–7, câmeras de perto, a cena que muda a meio, a caixa a
/// andar) e na cena 8, Low · Medium · High. Dois controlos que TÊM de mudar bytes: (1) o fundo sem
/// exigir as vizinhas dentro (o anel fica sem luz); (2) o salto forçado com a suavidade 0,5 (onde a luz
/// do fundo entra na imagem — a lei que o `FUNDO_SEM_LUZ` guarda).
#[test]
#[ignore = "rodada de medição W5v (placa): o fundo dos opacos ao bit"]
fn medida_luz2d_w5v_fundo_ao_bit() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5v fundo · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let mut casos = crate::medida_luz2d_w5q::casos();
    let c8 = cena_do_smoke(8);
    casos.push(("cena 8".into(), c8.clone(), c8, fabrica((1920, 1080))));
    let vizinhas =
        "        if !num_corpo(ponto_fino(vec2<u32>(vec2<i32>(alvo) + VIZINHAS_DO_FUNDO[k]))) {";
    let sem_vizinhas = "        if false && !num_corpo(ponto_fino(vec2<u32>(vec2<i32>(alvo) + VIZINHAS_DO_FUNDO[k]))) {";
    let forcado = [
        ("    if !FUNDO_SEM_LUZ || tau > 0.0", "    if tau > 0.0"),
        ("    if !FUNDO_SEM_LUZ {", "    if false {"),
    ];
    let suave = |c: Config| Config {
        miolo: c.miolo.map(|m| ph2d_light2d_gpu::Miolo {
            suavidade: 0.5,
            ..m
        }),
        ..c
    };
    let (mut todos_ao_bit, mut ctl) = (true, [0usize; 2]);
    // `PH2D_W5V_CASO=<pedaço do rótulo>` · `PH2D_W5V_NIVEIS=low,high`: só esses (isolar um caso).
    let filtro = std::env::var("PH2D_W5V_CASO").unwrap_or_default();
    let niveis = std::env::var("PH2D_W5V_NIVEIS").unwrap_or_else(|_| "low,medium,high".into());
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Media, NivelDaLuz::Alta] {
        let nome = match nivel {
            NivelDaLuz::Baixa => "low",
            NivelDaLuz::Media => "medium",
            NivelDaLuz::Alta => "high",
        };
        if !niveis.split(',').any(|n| n == nome) {
            continue;
        }
        for (rotulo, cena, cena2, a) in casos.iter().filter(|c| c.0.contains(filtro.as_str())) {
            let ler = |com: &dyn Fn(Config) -> Config| {
                let mut v = App::novo(&gpu, *a, (1920, 1080), nivel, com);
                let c = v.vista.config();
                eprintln!(
                    "  · {rotulo} {nome}: grade {}×{} fina ×{} salta {}",
                    c.grade.largura, c.grade.altura, c.fina, c.miolo_salta_o_fundo
                );
                for q in 0..80 {
                    v.quadro(&gpu, if q < 40 { cena } else { cena2 });
                }
                let b = crate::comum::ler(&gpu, v.vista.fluencia(), 8);
                drop(v);
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                b
            };
            let dif = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(a, b)| a != b).count();
            let antes = ler(&|c| Config {
                miolo_salta_o_fundo: false,
                ..c
            });
            let novo = ler(&|c| c);
            let d = dif(&antes, &novo);
            // O corte SOZINHO: a difusão exata sem o salto (o grupo nunca salta) contra o produto novo.
            let sem_salto = ler(&|c| Config {
                modulo_rc: Some(mutado(&c, &[("    if !FUNDO_SEM_LUZ {", "    if true {")])),
                ..c
            });
            let d_corte = dif(&sem_salto, &novo);
            if d > 0 {
                let largura = App::novo(&gpu, *a, (1920, 1080), nivel, &|c| c)
                    .vista
                    .grade_da_fluencia()
                    .largura as usize;
                let meia = |b: &[u8], i: usize| half::f16::from_le_bytes([b[i], b[i + 1]]).to_f32();
                for (i, _) in antes
                    .iter()
                    .zip(&novo)
                    .enumerate()
                    .filter(|(_, (x, y))| x != y)
                    .take(4)
                {
                    let (t, canal) = (i / 8, (i % 8) / 2);
                    let base = t * 8;
                    let px = |b: &[u8]| [0, 2, 4, 6].map(|o| meia(b, base + o));
                    eprintln!(
                        "  ≠ texel ({}, {}) canal {canal} · antes {:?} · novo {:?}",
                        t % largura,
                        t / largura,
                        px(&antes),
                        px(&novo)
                    );
                }
            }
            // `PH2D_W5V_REPETE=1`: o «antes» de novo — a placa dá o mesmo bit duas vezes?
            if std::env::var("PH2D_W5V_REPETE").as_deref() == Ok("1") {
                let outra = ler(&|c| Config {
                    miolo_salta_o_fundo: false,
                    ..c
                });
                eprintln!(
                    "{nivel:?} · {rotulo} · antes contra antes: {} bytes",
                    dif(&antes, &outra)
                );
            }
            todos_ao_bit &= d_corte == 0;
            let c1 = dif(
                &ler(&|c| Config {
                    modulo_rc: Some(mutado(&c, &[(vizinhas, sem_vizinhas)])),
                    ..c
                }),
                &novo,
            );
            let antes_suave = ler(&|c| Config {
                miolo_salta_o_fundo: false,
                ..suave(c)
            });
            let c2 = dif(
                &ler(&|c| {
                    let c = suave(c);
                    Config {
                        modulo_rc: Some(mutado(&c, &forcado)),
                        ..c
                    }
                }),
                &antes_suave,
            );
            ctl[0] += usize::from(c1 > 0);
            ctl[1] += usize::from(c2 > 0);
            eprintln!(
                "{nivel:?} · {rotulo} · o corte: {d_corte} bytes · o produto contra hoje: {d} de {} · controlo sem vizinhas {c1} · controlo suavidade 0,5 {c2}",
                novo.len()
            );
        }
    }
    eprintln!(
        "W5v fundo · o corte ao bit em todos: {todos_ao_bit} · os controlos mudaram {ctl:?} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    assert!(
        ctl[0] > 0 && ctl[1] > 0,
        "um controlo não mudou nenhum caso: a régua não vê o fundo"
    );
}
