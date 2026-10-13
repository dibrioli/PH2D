//! **A rodada da W5p — o que pesa no compilador do driver** (a abertura a frio, W5o §8). À mão, com o
//! cache do driver DESLIGADO (cada compilação é a frio; o controlo «produto de novo» confirma-o):
//!
//! ```text
//! PH2D_GPU=1 __GL_SHADER_DISK_CACHE=0 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d \
//!     --profile smoke --test it medida_luz2d_w5p_compilador -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A pipeline da grade fina (e o anel) do PRODUTO por nível — as constantes de
//! [`ph2d_light2d_gpu::constantes_da_fina`] —, variantes do TEXTO do módulo e das constantes no mesmo
//! processo, rodadas intercaladas com a ordem rodada; vale o MÍNIMO (a mediana ao lado).

use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::{Config, constantes_comuns_da_c3, constantes_da_fina, fonte_do_rc};
use std::time::Instant;

use crate::medida_luz2d_w5e::{config_do_produto, fabrica};

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 3;

/// `de` → `para` exatamente `n` vezes (um `replace` que não casa é um no-op silencioso).
fn trocar(fonte: &str, de: &str, para: &str, n: usize) -> String {
    assert_eq!(fonte.matches(de).count(), n, "«{de}» no módulo");
    fonte.replace(de, para)
}

/// Um limite que o compilador não conhece: a mesma conta (nenhuma luz tem `n = u32::MAX`), sem
/// desenrolar.
const OPACO: &str = "lampadas.n == 0xffffffffu";

fn lacos_opacos(fonte: &str, pe: bool) -> String {
    let s = trocar(
        fonte,
        "let n = RAIOS_DIRETOS;",
        &format!("let n = select(RAIOS_DIRETOS, 1u, {OPACO});"),
        2,
    );
    let s = trocar(
        &s,
        "b < BISSECOES;",
        &format!("b < select(BISSECOES, 1u, {OPACO});"),
        2,
    );
    if pe {
        trocar(
            &s,
            "i < PASSOS_DO_PE;",
            &format!("i < select(PASSOS_DO_PE, 1u, {OPACO});"),
            1,
        )
    } else {
        s
    }
}

/// Uma variante: (nome, o texto do módulo, constantes por cima das do produto).
type Variante = (
    &'static str,
    fn(&str) -> String,
    &'static [(&'static str, f64)],
);

/// A marcha trocada por UMA leitura do campo (mantém a dependência dos dados): quanto do tempo são as
/// cópias embutidas da marcha.
fn marcha_leve(fonte: &str) -> String {
    let s = trocar(fonte, "fn marchar_ate(", "fn marchar_ate_velho(", 1);
    s + "\nfn marchar_ate(P: vec2<f32>, A: vec2<f32>, B: vec2<f32>, ate_o_fim: bool) -> Traco {\n    \
         return Traco(vec3<Real>(Real(0)), Real(livre(mix(A, B, 0.5))));\n}\n"
}

/// Rodadas 1–2 (`PH2D_W5P_RODADA=2`): a hipótese da W5o (os laços desenrolados) e os cortes de
/// diagnóstico — só o TEMPO de compilar; a imagem destas não vale.
const DIAGNOSTICO: [Variante; 8] = [
    ("produto", |s| s.to_owned(), &[]),
    ("produto de novo (controlo do frio)", |s| s.to_owned(), &[]),
    ("raios e bissecções opacos", |s| lacos_opacos(s, true), &[]),
    (
        "1 raio, 1 bissecção",
        |s| s.to_owned(),
        &[("RAIOS_DIRETOS", 1.0), ("BISSECOES", 1.0)],
    ),
    (
        "a luz uma vez (sem a transmitida)",
        |s| {
            let s = trocar(
                s,
                "t = luz_fina(alvo, Onde(ponto_fino(alvo), false, ponto_fino(alvo), 0.0));",
                "t = l;",
                1,
            );
            trocar(&s, "t = luz_exata(ponto_fino(id));", "t = l;", 1)
        },
        &[],
    ),
    (
        "sem a outra face",
        |s| {
            trocar(
                s,
                "return mix(l, luz_exata(onde.p2), onde.f2);",
                "return mix(l, l.yzx, onde.f2);",
                1,
            )
        },
        &[],
    ),
    (
        "sem a visibilidade na face",
        |s| s.to_owned(),
        &[("VIS_NA_FACE", 0.0)],
    ),
    ("a marcha leve", marcha_leve, &[]),
];

/// O bloco da luz e da transmitida da `c3_fina`, como está no módulo.
const LUZ_DUAS_VEZES: &str = "    var l = luz_fina(alvo, onde);
    var t = vec3<f32>(0.0);
    if onde.remapeado && tau > 0.0 {
        c0_tau = tau;
        t = luz_fina(alvo, Onde(ponto_fino(alvo), false, ponto_fino(alvo), 0.0));
        c0_tau = 1.0;
    }
";

/// O mesmo num laço de 1–2 voltas: `luz_fina` com UM lugar de chamada. `limite` é a expressão do
/// número de voltas (`passes` = 1 ou 2).
fn luz_num_laco(fonte: &str, limite: &str) -> String {
    let novo = format!(
        "    var l = vec3<f32>(0.0);
    var t = vec3<f32>(0.0);
    let passes = select(1u, 2u, onde.remapeado && tau > 0.0);
    for (var i = 0u; i < {limite}; i++) {{
        var o = onde;
        if i > 0u {{
            c0_tau = tau;
            o = Onde(ponto_fino(alvo), false, ponto_fino(alvo), 0.0);
        }}
        let x = luz_fina(alvo, o);
        if i > 0u {{
            t = x;
        }} else {{
            l = x;
        }}
    }}
    c0_tau = 1.0;
"
    );
    trocar(fonte, LUZ_DUAS_VEZES, &novo, 1)
}

fn atras_opaco(fonte: &str) -> String {
    trocar(
        fonte,
        "for (var i = 0u; i < 2u; i++) {",
        "for (var i = 0u; i < (2u | (lampadas.n & 0x80000000u)); i++) {",
        2,
    )
}

fn c0_opaca(fonte: &str) -> String {
    let s = trocar(
        fonte,
        "for (var i = -2; i <= 2; i++) {",
        "for (var i = -2; i <= 2 + i32(lampadas.n & 0x40000000u); i++) {",
        1,
    );
    trocar(
        &s,
        "for (var dy = -1; dy <= 2; dy++) {",
        "for (var dy = -1; dy <= 2 + i32(lampadas.n & 0x40000000u); dy++) {",
        1,
    )
}

/// O commit de ANTES da W5p (o topo da W5o).
pub(crate) const ANTES: &str = "2565d66bbf";

/// Os ficheiros do módulo das cascatas pela ordem do `rc!` (`ph2d-light2d-gpu/src/shaders.rs`) com
/// a aritmética f32 e o armazenamento f16 (o `Config::padrao`); [`modulo_antes`] confere a lista
/// contra o módulo de hoje.
const FICHEIROS_DO_RC: [&str; 11] = [
    "real_f32",
    "saida_f16",
    "queda",
    "c2_rc",
    "c3_direta",
    "c3_anel",
    "c3_preenche",
    "c4_rebote",
    "c3_borda",
    "c3_pe",
    "c2_hrc",
];

/// O texto do módulo das cascatas no commit [`ANTES`] (f32, f16).
pub(crate) fn modulo_antes() -> &'static str {
    static M: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../ph2d-light2d-gpu/src/wgsl");
        let hoje: String = FICHEIROS_DO_RC
            .iter()
            .map(|f| std::fs::read_to_string(format!("{dir}/{f}.wgsl")).expect("wgsl"))
            .collect();
        assert_eq!(
            hoje,
            fonte_do_rc(
                ph2d_light2d_gpu::Precisao::F32,
                ph2d_light2d_gpu::Precisao::F16
            ),
            "a lista FICHEIROS_DO_RC não é a do rc!"
        );
        let antes: String = FICHEIROS_DO_RC
            .iter()
            .map(|f| {
                let o = std::process::Command::new("git")
                    .args(["-C", dir, "show"])
                    .arg(format!("{ANTES}:crates/ph2d-light2d-gpu/src/wgsl/{f}.wgsl"))
                    .output()
                    .expect("git show");
                assert!(o.status.success(), "git show {f}");
                String::from_utf8(o.stdout).expect("utf-8")
            })
            .collect();
        Box::leak(antes.into_boxed_str())
    })
}

/// Rodada 5 (`PH2D_W5P_RODADA=5`): o que ainda pesa na W5p, com os distantes (o caminho crítico). Só o
/// TEMPO de compilar; a imagem destas não vale.
const RESTO: [Variante; 6] = [
    (
        "W5p com os distantes",
        |s| s.to_owned(),
        &[("DISTANTES", 1.0)],
    ),
    (
        "sem a visibilidade na face",
        |s| s.to_owned(),
        &[("DISTANTES", 1.0), ("VIS_NA_FACE", 0.0)],
    ),
    (
        "sem a c0 ao longo da face",
        |s| s.to_owned(),
        &[("DISTANTES", 1.0), ("C0_AO_LONGO_DA_FACE", 0.0)],
    ),
    ("a marcha leve", marcha_leve, &[("DISTANTES", 1.0)]),
    (
        "a de trás num laço opaco",
        atras_opaco,
        &[("DISTANTES", 1.0)],
    ),
    (
        "sem o pé na borda",
        |s| s.to_owned(),
        &[("DISTANTES", 1.0), ("PE_NA_BORDA", 0.0)],
    ),
];

/// Rodada 3 (a técnica: um sítio de chamada num laço); rodada 4 (`PH2D_W5P_RODADA=4`): a W5p contra
/// o [`ANTES`], com e sem os distantes. Só o TEMPO de compilar.
const ANTES_E_DEPOIS: [Variante; 5] = [
    ("antes (W5o)", |_| modulo_antes().to_owned(), &[]),
    (
        "antes (W5o) com os distantes",
        |_| modulo_antes().to_owned(),
        &[("DISTANTES", 1.0)],
    ),
    ("W5p", |s| s.to_owned(), &[]),
    ("W5p de novo (controlo do frio)", |s| s.to_owned(), &[]),
    (
        "W5p com os distantes",
        |s| s.to_owned(),
        &[("DISTANTES", 1.0)],
    ),
];

/// Rodadas 1–2 (a hipótese da W5o e os cortes de diagnóstico) no handoff W5p; rodada 3: a técnica
/// (um lugar de chamada num laço que o compilador não desenrola). Só o TEMPO de compilar.
const TECNICA: [Variante; 9] = [
    ("produto", |s| s.to_owned(), &[]),
    ("produto de novo (controlo do frio)", |s| s.to_owned(), &[]),
    (
        "produto com os distantes",
        |s| s.to_owned(),
        &[("DISTANTES", 1.0)],
    ),
    (
        "a luz uma vez (corte, o alvo)",
        |s| {
            trocar(
                s,
                "t = luz_fina(alvo, Onde(ponto_fino(alvo), false, ponto_fino(alvo), 0.0));",
                "t = l;",
                1,
            )
        },
        &[],
    ),
    (
        "a luz num laço (limite opaco)",
        |s| luz_num_laco(s, "(passes | (lampadas.n & 0x80000000u))"),
        &[],
    ),
    (
        "a luz num laço (limite 2 conhecido)",
        |s| luz_num_laco(s, "passes"),
        &[],
    ),
    ("a de trás num laço opaco", atras_opaco, &[]),
    ("a c0 da face com laços opacos", c0_opaca, &[]),
    (
        "os três",
        |s| {
            c0_opaca(&atras_opaco(&luz_num_laco(
                s,
                "(passes | (lampadas.n & 0x80000000u))",
            )))
        },
        &[],
    ),
];

fn compilar(
    gpu: &ph2d_gpu::GpuContext,
    modulo: &wgpu::ShaderModule,
    entrada: &str,
    constantes: &[(&str, f64)],
) -> f64 {
    let t0 = Instant::now();
    let _p = gpu
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("medida w5p"),
            layout: None,
            module: modulo,
            entry_point: Some(entrada),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: constantes,
                zero_initialize_workgroup_memory: true,
            },
            cache: None,
        });
    t0.elapsed().as_secs_f64()
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

/// ⭐ W5p — os segundos de compilar a grade fina e o anel do produto, por variante.
#[test]
#[ignore = "rodada de medição W5p: o que pesa no compilador do driver"]
fn medida_luz2d_w5p_compilador() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5p compilador · __GL_SHADER_DISK_CACHE={:?} · load {}",
        std::env::var("__GL_SHADER_DISK_CACHE").ok(),
        loadavg().trim()
    );
    let a = fabrica(FHD);
    let niveis: Vec<(NivelDaLuz, Config)> = [NivelDaLuz::Baixa, NivelDaLuz::Alta]
        .into_iter()
        .filter_map(|n| config_do_produto(a, FHD, degrau_do(n)).map(|c| (n, c)))
        .collect();
    // (nível, variante, entrada) → segundos por rodada.
    // As rodadas 2–3 transformam o texto de ANTES (foram medidas sobre ele).
    let (variantes, sobre_o_antes): (&[Variante], bool) =
        match std::env::var("PH2D_W5P_RODADA").as_deref() {
            Ok("2") => (&DIAGNOSTICO, true),
            Ok("3") => (&TECNICA, true),
            Ok("5") => (&RESTO, false),
            _ => (&ANTES_E_DEPOIS, false),
        };
    let mut t: std::collections::BTreeMap<(usize, usize, usize), Vec<f64>> = Default::default();
    for rodada in 0..RODADAS {
        for (ni, (_, cfg)) in niveis.iter().enumerate() {
            let fonte = if sobre_o_antes {
                modulo_antes()
            } else {
                fonte_do_rc(cfg.aritmetica, cfg.armazenamento)
            };
            let (entrada, proprias) = constantes_da_fina(cfg);
            let anel = format!("{entrada}_anel");
            for k in 0..variantes.len() {
                let vi = (k + rodada) % variantes.len();
                let (_, texto, por_cima) = variantes[vi];
                let modulo = gpu
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("medida w5p"),
                        source: wgpu::ShaderSource::Wgsl(texto(fonte).into()),
                    });
                let mut c: Vec<(&str, f64)> = proprias
                    .iter()
                    .copied()
                    .chain(constantes_comuns_da_c3(cfg))
                    .chain([("DISTANTES", 0.0)])
                    .collect();
                for (nome, v) in &mut c {
                    if let Some((_, x)) = por_cima.iter().find(|(n, _)| n == nome) {
                        *v = *x;
                    }
                }
                // O piso: uma entrada pequena do MESMO módulo (a borda exata).
                let piso = (vi == 0).then_some("c3_borda");
                for (ei, e) in [Some(entrada), Some(anel.as_str()), piso]
                    .into_iter()
                    .enumerate()
                {
                    let Some(e) = e else { continue };
                    t.entry((ni, vi, ei))
                        .or_default()
                        .push(compilar(&gpu, &modulo, e, &c));
                }
            }
        }
        eprintln!("  rodada {rodada} · load {}", loadavg().trim());
    }
    let minimo = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let mediana = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(f64::total_cmp);
        s[s.len() / 2]
    };
    for (ni, (n, cfg)) in niveis.iter().enumerate() {
        let (entrada, _) = constantes_da_fina(cfg);
        let piso = &t[&(ni, 0, 2)];
        eprintln!(
            "{n:?} ({entrada}) — segundos, mínimo (mediana) · fina · anel · o piso (c3_borda) {:.2} ({:.2})",
            minimo(piso),
            mediana(piso)
        );
        for (vi, (nome, ..)) in variantes.iter().enumerate() {
            let f = &t[&(ni, vi, 0)];
            let an = &t[&(ni, vi, 1)];
            eprintln!(
                "  {nome:<36} {:>6.2} ({:>6.2}) · {:>6.2} ({:>6.2})",
                minimo(f),
                mediana(f),
                minimo(an),
                mediana(an)
            );
        }
    }
}

/// A vista do produto e o entorno dela (o `Par` da W5o com a câmera `a`).
struct Vista {
    vista: ph2d_light2d_gpu::Solver,
    entorno: Option<ph2d_light2d_gpu::Solver>,
}

impl Vista {
    fn nova(
        gpu: &ph2d_gpu::GpuContext,
        a: kurbo::Affine,
        nivel: NivelDaLuz,
        modulo: Option<&'static str>,
    ) -> Self {
        use ph2d_light2d_gpu::Solver;
        let dg = degrau_do(nivel);
        let com = |c: Config| Config {
            modulo_rc: modulo,
            ..c
        };
        let cfg = com(config_do_produto(a, FHD, dg).expect("a vista"));
        assert_eq!(
            (cfg.aritmetica, cfg.armazenamento),
            (
                ph2d_light2d_gpu::Precisao::F32,
                ph2d_light2d_gpu::Precisao::F16
            ),
            "o modulo_antes é o f32/f16"
        );
        let mut v = Solver::novo(gpu, cfg).expect("solver da vista");
        let (i, f) = ph2d_app_light2d::regiao_da_janela(v.grade_da_fluencia(), a, FHD);
        v.regiao_fina(i, f);
        let mut e = crate::medida_luz2d_w5e::config_do_entorno(a, FHD, dg)
            .map(|c| Solver::novo(gpu, com(c)).expect("solver do entorno"));
        if let Some(e) = e.as_mut() {
            let g = v.config().grade;
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
        v.somar_entorno(gpu, e.as_ref());
        Self {
            vista: v,
            entorno: e,
        }
    }

    fn quadro(&mut self, gpu: &ph2d_gpu::GpuContext, cena: &ph2d_light2d_ecs::CenaDeLuz) {
        if let Some(e) = self.entorno.as_mut() {
            let sem_ceu = ph2d_light2d_ecs::CenaDeLuz {
                ceu: [0.0; 3],
                ..cena.clone()
            };
            e.quadro(gpu, &sem_ceu);
        }
        self.vista.quadro(gpu, cena);
    }
}

/// ⭐ W5p — a reescrita contra o [`ANTES`] no MESMO processo: a fluência da vista ao BIT (80 quadros,
/// o miolo assentado) e o relógio da placa da C3 (preenche + visibilidade + fina, e o rebote), o
/// mínimo de 7 rodadas intercaladas. Cenas 1–6 do smoke na vista de fábrica e a 3 com a câmera em
/// x = 2 (a luz de fora pelo entorno e os distantes), Low e High.
#[test]
#[ignore = "rodada de medição W5p (placa): a reescrita ao bit e o custo"]
fn medida_luz2d_w5p_placa() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!("W5p placa · load {}", loadavg().trim());
    let mut casos = Vec::new();
    for n in 1..=6 {
        casos.push((n, [0.0_f32, 0.0], 0.0_f32));
    }
    casos.push((3, [2.0, 0.0], 2.0));
    let mut todos_ao_bit = true;
    // O controlo da régua: o ANTES com uma mutação mínima na abertura TEM de mudar bytes (o módulo
    // trocado chega mesmo à placa).
    let mutado: &'static str = Box::leak(
        trocar(
            modulo_antes(),
            "soma += 0.5 * (t0 + t1) * largura;",
            "soma += 0.49 * (t0 + t1) * largura;",
            2,
        )
        .into_boxed_str(),
    );
    let mut controlo_ok = false;
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        for &(n, centro, altura) in &casos {
            let mut w = bevy_ecs::world::World::new();
            assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
            let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
            let camera = if altura > 0.0 {
                ph2d_render::Camera2d::new(centro, altura)
            } else {
                ph2d_render::Camera2d::default()
            };
            let a = camera.world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
            let mut par = [
                Vista::nova(&gpu, a, nivel, Some(modulo_antes())),
                Vista::nova(&gpu, a, nivel, None),
            ];
            for _ in 0..80 {
                for v in &mut par {
                    v.quadro(&gpu, &cena);
                }
            }
            let bytes = par
                .each_ref()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8));
            let diferentes = bytes[0]
                .iter()
                .zip(&bytes[1])
                .filter(|(x, y)| x != y)
                .count();
            let f = par
                .each_ref()
                .map(|v| crate::comum::fluencia_lida(&gpu, &v.vista));
            let (mut maximo, mut pico) = (0.0_f32, 0.0_f32);
            for (x, y) in f[0].v.iter().zip(&f[1].v) {
                for c in 0..3 {
                    maximo = maximo.max((x[c] - y[c]).abs());
                    pico = pico.max(x[c]);
                }
            }
            todos_ao_bit &= diferentes == 0;
            if !controlo_ok && n == 5 {
                let mut m = Vista::nova(&gpu, a, nivel, Some(mutado));
                for _ in 0..80 {
                    m.quadro(&gpu, &cena);
                }
                let bm = crate::comum::ler(&gpu, m.vista.fluencia(), 8);
                let d = bm.iter().zip(&bytes[0]).filter(|(x, y)| x != y).count();
                eprintln!("  controlo (o antes mutado × o antes): {d} bytes diferentes");
                assert!(d > 0, "a régua não vê o módulo trocado");
                controlo_ok = true;
            }
            // O relógio: rodadas intercaladas, a ordem rodada, 3 quadros por bloco.
            let mut minimo = [[f64::INFINITY; 2]; 2];
            for rodada in 0..7 {
                for j in 0..2 {
                    let i = (j + rodada) % 2;
                    for _ in 0..3 {
                        par[i].quadro(&gpu, &cena);
                    }
                    let t = par[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    minimo[i][0] = minimo[i][0].min(t.c3_preenche + t.c3_visibilidade + t.c3_fina);
                    minimo[i][1] = minimo[i][1].min(t.c3_rebote);
                }
            }
            eprintln!(
                "{nivel:?} · cena {n} {centro:?} · bytes diferentes {diferentes} de {} (máx {maximo:.6} numa luz de {pico:.1}) · C3 {:.3} → {:.3} ms · rebote {:.3} → {:.3} ms",
                bytes[0].len(),
                minimo[0][0],
                minimo[1][0],
                minimo[0][1],
                minimo[1][1]
            );
        }
    }
    assert!(controlo_ok);
    eprintln!(
        "W5p placa · ao bit em todos: {todos_ao_bit} · load {}",
        loadavg().trim()
    );
}
