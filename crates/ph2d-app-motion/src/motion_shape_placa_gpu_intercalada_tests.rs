//! ⭐ doc 121 §9.16 — **A SONDA INTERCALADA: todas as variantes e cenas num SÓ processo.**
//!
//! A régua de antes (`mede_sonda_das_estrelas.sh`) abria um processo por célula, esperava `20` s de calma
//! antes de cada um e compilava um binário por variante: a rodada do §9.15 (`360` células) levou `4`–`5` h,
//! das quais `2` h de espera com a máquina ociosa. Aqui as variantes são PASSES do mesmo processo
//! ([`ph2d_shape_gpu::VarianteDoPasse`]: as constantes `override` escolhem-se ao criar o pipeline), e
//! cada rodada passa por todas elas em BLOCOS de quadros, por ordem rodada — uma carga alheia cai em
//! todas por igual. A régua é a SOMA dos passes do relógio da placa (`pass_profiler::drain`); o resumo
//! é o MÍNIMO das rodadas (uma interferência só soma tempo) e a mediana ao lado.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --profile smoke -- --ignored --nocapture sonda_intercalada
//! ```
//! Ambiente: `PH2D_SONDA_VARIANTES=F,F-D` · `PH2D_SONDA_CENAS=esticadas_tr1,densas_tr0` ·
//! `PH2D_SONDA_BLOCO=20` (quadros por bloco) · `PH2D_SONDA_RODADAS=7` · `PH2D_SONDA_VELLO=0` (sem a
//! referência do Vello).

use super::*;
use ph2d_shape_gpu::VarianteDoPasse;

/// Os passes cuja soma é a régua (doc 121 §9.13: o relógio POR passe mente na fronteira).
const PASSES: [&str; 4] = [
    "render.contorno.conta",
    "render.contorno.escreve",
    "render.contorno.celulas",
    "render.formas",
];

/// As cenas: `(nome, modo, nível denso, tracejado)` — os arranjos da régua do doc 121.
const CENAS: [(&str, &str, &str, bool); 6] = [
    ("esticadas_tr0", "", "", false),
    ("esticadas_tr1", "", "", true),
    ("conformes_tr0", "conforme", "", false),
    ("conformes_tr1", "conforme", "", true),
    ("densas_tr0", "", "2", false),
    ("densas_tr1", "", "2", true),
];

/// As variantes: o `F` (o produto) e as duas portas de EXECUÇÃO que ficam (o prefixo por subgrupo e a medida
/// no início, doc 121 §9.15). Os pedaços `override` do §9.15 dobraram-se e os candidatos do §9.17 saíram com o
/// código deles (doc 121 §9.17); um candidato novo entra aqui como `override` ([`ph2d_shape_gpu::CONSTANTES_DO_CONTORNO`]).
fn variantes() -> Vec<(&'static str, VarianteDoPasse)> {
    // `PH2D_SONDA_AA=k`: `k` cópias do `F` a mais — o controlo A/A (o que a régua lê entre variantes iguais).
    const AA: [&str; 12] = [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
    ];
    let aa = (numero("PH2D_SONDA_AA", 0) as usize).min(AA.len());
    let mut v = vec![
        ("F", VarianteDoPasse::default()),
        (
            "F-D",
            VarianteDoPasse {
                sem_subgrupo: true,
                ..VarianteDoPasse::default()
            },
        ),
        (
            "F-c2",
            VarianteDoPasse {
                sem_medida_no_inicio: true,
                ..VarianteDoPasse::default()
            },
        ),
    ];
    v.extend(AA[..aa].iter().map(|n| (*n, VarianteDoPasse::default())));
    v
}

/// A lista do ambiente (`a,b,c`), ou tudo.
fn filtro(var: &str) -> Option<Vec<String>> {
    std::env::var(var)
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| v.split(',').map(str::to_owned).collect())
}

fn numero(var: &str, omissao: u32) -> u32 {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(omissao)
}

/// A mediana e o mínimo de uma amostra.
fn min_med(v: &[f64]) -> (f64, f64) {
    let mut o = v.to_vec();
    o.sort_by(f64::total_cmp);
    (o.first().copied().unwrap_or(f64::NAN), o[o.len() / 2])
}

#[test]
#[ignore = "sonda de relógio"]
fn sonda_intercalada() {
    let Some(gpu) = gpu() else { return };
    ph2d_gpu::pass_profiler::init_forced(&gpu.device, &gpu.queue);
    let bloco = numero("PH2D_SONDA_BLOCO", 20);
    let rodadas = numero("PH2D_SONDA_RODADAS", 7);
    let vello = !std::env::var("PH2D_SONDA_VELLO").is_ok_and(|v| v == "0");
    let so_variantes = filtro("PH2D_SONDA_VARIANTES");
    let so_cenas = filtro("PH2D_SONDA_CENAS");
    let todas: Vec<_> = variantes()
        .into_iter()
        .filter(|(n, _)| {
            so_variantes
                .as_ref()
                .is_none_or(|f| f.iter().any(|x| x == n))
        })
        .collect();
    assert!(!todas.is_empty(), "nenhuma variante casa o filtro");
    let placa_nome = gpu.adapter.get_info().name;
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "INTERCALADA placa «{placa_nome}» · {} variantes · bloco {bloco} · {rodadas} rodadas · load {}",
        todas.len(),
        carga().trim()
    );
    let t0 = std::time::Instant::now();
    let quadro = |p: &mut PlacaDeFormas,
                  geo: &mut GeometriasDaPlaca,
                  insts: &[VectorInstance],
                  store: &VecPathStore| {
        assert!(p.decide(true, insts, store, geo, camara()));
        let _ = p.desenha(&gpu, (LADO, LADO), geo, None);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        ph2d_gpu::pass_profiler::end_frame(&gpu.device, &gpu.queue);
    };
    for (cena, modo, denso, tracejado) in CENAS {
        if so_cenas
            .as_ref()
            .is_some_and(|f| !f.iter().any(|x| x == cena))
        {
            continue;
        }
        let (store, insts) = estrelas_da_sonda(modo, denso, tracejado);
        let mut placas: Vec<(PlacaDeFormas, GeometriasDaPlaca)> = todas
            .iter()
            .map(|(_, v)| {
                let mut p = PlacaDeFormas::default();
                p.com_variante(v.clone());
                (p, GeometriasDaPlaca::default())
            })
            .collect();
        // O aquecimento: a criação dos pipelines, a capacidade medida e o regime — fora da régua.
        let mut camadas = Vec::with_capacity(placas.len());
        for (p, geo) in &mut placas {
            for _ in 0..4 {
                quadro(p, geo, &insts, &store);
            }
            let camada = le_a_camada(&gpu, p);
            let tinta = camada
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|px| px[3] > 0)
                .count();
            assert!(
                tinta > 1000,
                "{cena}: uma variante nao desenhou ({tinta} px)"
            );
            camadas.push(camada);
        }
        // §9.17: a imagem de cada variante contra a do `F` (o produto), byte a byte — a `E1F` é o controlo.
        let i_f = todas.iter().position(|(n, _)| *n == "F");
        let imagem: Vec<String> = camadas
            .iter()
            .map(|c| {
                i_f.map_or_else(
                    || "?".to_owned(),
                    |f| match c.iter().zip(&camadas[f]).filter(|(x, y)| x != y).count() {
                        0 => "imagem = F".to_owned(),
                        d => format!("imagem DIFERE em {d} B"),
                    },
                )
            })
            .collect();
        // As arestas reservadas · escritas de cada variante (§9.17: a reserva faz crescer a memória).
        let arestas: Vec<_> = placas
            .iter()
            .map(|(p, _)| p.arestas_do_ultimo_quadro(&gpu))
            .collect();
        let _ = ph2d_gpu::pass_profiler::drain(&gpu.device);
        // Por variante, por rodada: a soma e os quatro passes, por quadro (ms).
        let n = placas.len();
        let mut amostras = vec![Vec::<[f64; 5]>::new(); n];
        for r in 0..rodadas as usize {
            for k in 0..n {
                let i = (k + r) % n;
                let (p, geo) = &mut placas[i];
                for _ in 0..bloco {
                    quadro(p, geo, &insts, &store);
                }
                let Some(t) = ph2d_gpu::pass_profiler::drain(&gpu.device) else {
                    panic!("sem o relogio da placa (TIMESTAMP_QUERY)");
                };
                let q = f64::from(t.frames.max(1));
                let ms = |rotulo: &str| {
                    t.per_label
                        .iter()
                        .filter(|(l, _, _)| *l == rotulo)
                        .map(|(_, ns, _)| *ns as f64 / q / 1.0e6)
                        .sum::<f64>()
                };
                let [c, e, cel, f] = PASSES.map(ms);
                amostras[i].push([c + e + cel + f, c, e, cel, f]);
            }
        }
        let vello_ms = vello.then(|| {
            let mut c = ph2d_vector::VectorScene::new();
            let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
            crate::motion_shape_gen::encode(
                &insts,
                &store,
                &mut |_, _| None,
                camara(),
                Some(janela),
                ph2d_render::ImageFilterMode::Smooth,
                &mut c,
            );
            let mut vp = ph2d_render::VelloPass::new(
                &gpu,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                (LADO, LADO),
            )
            .expect("vello");
            let _ = vp.render_and_readback(&gpu, c.inner(), (LADO, LADO));
            let t = std::time::Instant::now();
            for _ in 0..30 {
                let _ = vp.render_and_readback(&gpu, c.inner(), (LADO, LADO));
            }
            t.elapsed().as_secs_f64() * 1e3 / 30.0
        });
        let base = min_med(&amostras[0].iter().map(|a| a[0]).collect::<Vec<_>>()).0;
        for (i, (nome, _)) in todas.iter().enumerate() {
            let col = |j: usize| min_med(&amostras[i].iter().map(|a| a[j]).collect::<Vec<_>>());
            let (smin, smed) = col(0);
            eprintln!(
                "INTERCALADA {cena:<14} {nome:<6} soma min {smin:.3} med {smed:.3} ({:+.1} % do 1.º) · conta {:.3} · escreve {:.3} · celulas {:.3} · formas {:.3} · arestas {} · {} · {}{}",
                (smin / base - 1.0) * 100.0,
                col(1).0,
                col(2).0,
                col(3).0,
                col(4).0,
                arestas[i].0,
                arestas[i].1,
                imagem[i],
                if i == 0 {
                    vello_ms.map_or(String::new(), |v| format!(" · vello (parede) {v:.3}"))
                } else {
                    String::new()
                }
            );
        }
    }
    eprintln!(
        "INTERCALADA fim em {:.1} s · load {}",
        t0.elapsed().as_secs_f64(),
        carga().trim()
    );
    // §9.18 (F): o perfilador solta o dispositivo antes do `exit` (na RTX, segurado até lá, o processo morria).
    ph2d_gpu::pass_profiler::shutdown();
}
