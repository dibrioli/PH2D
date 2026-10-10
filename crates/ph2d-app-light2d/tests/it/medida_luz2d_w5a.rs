//! **A rodada da W5a — a captura FINA dos opacos e o preenchimento** (a fila do dono F1 · F2 · F3,
//! `docs/Luz2D/01_plano.md` §7.1; método `docs/DevOps/MEDIR_VELOCIDADE.md`): um lote, as variantes no
//! MESMO processo (configurações do solver, nunca `const` recompilada), intercaladas com a ordem
//! rodada, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5a -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o custo de placa (0 · 1 · 4 · 6 lâmpadas) a 1920×1080 por nível e variante — e o teto de
//!    lâmpadas diretas que ele dá;
//! B. a escada da sombra (degrau e MÉDIA, F1) por nível e variante;
//! D. a faixa escura na face acesa da parede (F3) por nível e variante;
//! E. o oclusor fino de 5 cm (F2) por nível e variante;
//! F. o Vello da camada do vetor à resolução de cada captura.

use std::time::Instant;

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    Degrau, LuzDoQuadro, captura_do_quadro, fina_do_quadro, grade_do_quadro, qualidade::degrau_do,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_ecs::VecPathRef;
use ph2d_light2d_ecs::{BloqueiaLuz, CenaDeLuz};
use ph2d_light2d_gpu::{Bordas, Config, Refino, Solver, TemposMs};
use ph2d_vec_scene::{VecScene, VecXforms, rectangle};

use crate::comum::{Fluencia, fluencia_lida, gpu};
use crate::medida_luz2d_w4b::{afim, cena_com, loadavg, solver};

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 9;
const NIVEIS: [(&str, NivelDaLuz); 3] = [
    ("Low", NivelDaLuz::Baixa),
    ("Medium", NivelDaLuz::Media),
    ("High", NivelDaLuz::Alta),
];

/// Uma variante: o lado do texel da captura (`None` = a das cascatas), o passo da luz direta (em
/// texels FINOS) e o junto à borda, o preenchimento e onde ele corre, e a curvatura.
struct Variante {
    nome: &'static str,
    captura_px: Option<f64>,
    passo: f32,
    borda: f32,
    preenchimento: Option<u32>,
    grade_inteira: bool,
    curvatura: bool,
}

const fn v(nome: &'static str, captura_px: Option<f64>, passo: f32, borda: f32) -> Variante {
    Variante {
        nome,
        captura_px,
        passo,
        borda,
        preenchimento: Some(2),
        grade_inteira: false,
        curvatura: true,
    }
}

/// 1.ª volta (10/10): a captura (grossa · 1 px · ½ px) × o preenchimento (célula 1 · 2) — o F3 é do
/// preenchimento (célula 2 = célula 1 a ¼ do custo), o F2 da captura e da curvatura (sem ela, o fundo
/// da lasca no Low 0,257), e o degrau do F1 do PASSO da marcha (a sonda `medida_causa_do_desvio`).
/// 2.ª volta: o passo fino só junto à borda cura o degrau a +7 % por lâmpada (o passo fino em tudo,
/// +75 %); a captura de ½ px leva a média a −0,79 px (1 px: −1,16); o preenchimento na grade inteira
/// custa +0,22 ms. (Nas voltas 1–2 o passo contava-se em texels da CAPTURA: «½ px · passo 2» é o
/// passo 1 de hoje, em texels finos.) 3.ª volta (esta): a combinação escolhida, a célula do
/// preenchimento e o teto de lâmpadas (a semente do campo lia a captura fina num fio por célula —
/// 0,71 ms; a MARCA paralela, 0,20).
const VARIANTES: [Variante; 4] = [
    Variante {
        nome: "W4b (grossa, sem preencher)",
        preenchimento: None,
        ..v("", None, 1.0, 1.0)
    },
    Variante {
        nome: "padrão W5a (½ px · passo 1 · borda ½ · preenche 4 na grade inteira)",
        grade_inteira: true,
        preenchimento: Some(4),
        ..v("", Some(0.5), 1.0, 0.5)
    },
    Variante {
        nome: "… preenche 8 na grade inteira",
        grade_inteira: true,
        preenchimento: Some(8),
        ..v("", Some(0.5), 1.0, 0.5)
    },
    Variante {
        nome: "… preenche 4 só na região",
        preenchimento: Some(4),
        ..v("", Some(0.5), 1.0, 0.5)
    },
];

fn degrau(nivel: NivelDaLuz, v: &Variante) -> Degrau {
    let d = degrau_do(nivel);
    Degrau {
        captura_px: v.captura_px.unwrap_or(d.c0_px),
        ..d
    }
}

/// A configuração do PRODUTO (a do `LuzDoQuadro::gravar`) para o nível e a variante.
fn config(gpu: &GpuContext, nivel: NivelDaLuz, v: &Variante) -> Config {
    let d = degrau(nivel, v);
    let g = grade_do_quadro(afim(), FHD, d).expect("grade");
    Config {
        fina: fina_do_quadro(afim(), d),
        captura: captura_do_quadro(afim(), d),
        preenchimento: v.preenchimento,
        preenche_a_grade_inteira: v.grade_inteira,
        passo_direto: v.passo,
        passo_na_borda: v.borda,
        refino: match ph2d_light2d_gpu::REFINO_PADRAO {
            Refino::Bordas(b) => Refino::Bordas(Bordas {
                curvatura: v.curvatura,
                ..b
            }),
            Refino::Tudo => Refino::Tudo,
        },
        ..Config::padrao_para(&gpu.device, g)
    }
}

fn fluencia(gpu: &GpuContext, cfg: Config, cena: &CenaDeLuz) -> Fluencia {
    let mut s = solver(gpu, cfg);
    s.quadro(gpu, cena);
    fluencia_lida(gpu, &s)
}

#[test]
#[ignore = "a rodada da W5a: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w5a() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W5a · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    custo(&gpu);
    escada(&gpu);
    faixa(&gpu);
    oclusor_fino(&gpu);
    vello(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. O custo de placa por lâmpada.
fn custo(gpu: &GpuContext) {
    let lampadas = [0_usize, 1, 4, 6, 7, 8];
    let mut cenas: Vec<CenaDeLuz> = lampadas.iter().map(|&n| cena_com(n)).collect();
    // As 6 lâmpadas na posição CARA: as da `cena_com(8)` sem as duas primeiras (as duas últimas
    // ficam junto à parede — a penumbra delas cobre mais da vista; a W4b mediu nelas 0,686 ms).
    let mut junto = cena_com(8);
    let lampadas_da_cena: Vec<usize> = junto
        .emissores
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, ph2d_light2d_ecs::Emissor::Disco { .. }))
        .map(|(i, _)| i)
        .collect();
    for i in lampadas_da_cena.iter().take(2).rev() {
        junto.emissores.remove(*i);
    }
    cenas.push(junto);
    let mut variantes: Vec<(String, Solver)> = Vec::new();
    for (nome, nivel) in NIVEIS {
        for v in &VARIANTES {
            let cfg = config(gpu, nivel, v);
            variantes.push((
                format!("{nome} · {} (captura ×{})", v.nome, cfg.captura),
                solver(gpu, cfg),
            ));
        }
    }
    let n = variantes.len() * cenas.len();
    let mut minimo: Vec<Option<TemposMs>> = vec![None; n];
    for rodada in 0..RODADAS {
        for k in 0..n {
            let i = (k + rodada * 7) % n;
            let (v, c) = (i / cenas.len(), i % cenas.len());
            let s = &mut variantes[v].1;
            s.quadro(gpu, &cenas[c]);
            s.quadro(gpu, &cenas[c]);
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            if minimo[i].as_ref().is_none_or(|m| t.total() < m.total()) {
                minimo[i] = Some(t);
            }
        }
    }
    eprintln!("\n## A. Custo de placa (ms, MÍNIMO de {RODADAS} rodadas intercaladas), 1920×1080");
    eprintln!(
        "| variante | 0 lâmpadas | 1 | 4 | 6 | 7 | 8 | 6 junto à parede | ms/lâmpada (6 · junto) | C0 · C0 fina · C1 · cascatas · preenche · C3 fina (0) | C3 vis · fina (6) | MB alocados | teto (5,56 ms, junto) |"
    );
    eprintln!("|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for (v, (nome, s)) in variantes.iter().enumerate() {
        let t = |c: usize| minimo[v * cenas.len() + c].clone().expect("medido");
        let tot: Vec<f64> = (0..cenas.len()).map(|c| t(c).total()).collect();
        let por = (tot[3] - tot[0]) / 6.0;
        let por_junto = (tot[6] - tot[0]) / 6.0;
        let (t0, t6) = (t(0), t(3));
        eprintln!(
            "| {nome} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {por:.3} · {por_junto:.3} | {:.2} · {:.2} · {:.2} · {:.2} · {:.2} · {:.2} | {:.2} · {:.2} | {:.1} | {:.1} |",
            tot[0],
            tot[1],
            tot[2],
            tot[3],
            tot[4],
            tot[5],
            tot[6],
            t0.c0,
            t0.c0_fina,
            t0.c1,
            t0.c2_total(),
            t0.c3_preenche,
            t0.c3_fina,
            t6.c3_visibilidade,
            t6.c3_fina,
            s.pass_list().bytes_alocados() as f64 / 1e6,
            (ph2d_app_light2d::qualidade::ORCAMENTO_MS - tot[0]) / por_junto.max(1e-6)
        );
    }
}

/// B. A escada (degrau rms dos 5 contornos e a MÉDIA do de 50 %, F1).
fn escada(gpu: &GpuContext) {
    eprintln!("\n## B. Escada (cena 4, lâmpada pequena): degrau rms · média (px)");
    eprintln!("| variante | degrau 2 · 10 · 50 · 90 · 98 % | média 2 · 10 · 50 · 90 · 98 % |");
    eprintln!("|---|---|---|");
    let (com, sem) = crate::a_escada_da_sombra::cena4_com_e_sem();
    let (cc, cs) = (cena_de_luz(&com), cena_de_luz(&sem));
    for (nome, nivel) in NIVEIS {
        for v in &VARIANTES {
            let cfg = config(gpu, nivel, v);
            let e = crate::a_escada_da_sombra::escada_de(
                &fluencia(gpu, cfg, &cc),
                &fluencia(gpu, cfg, &cs),
            );
            eprintln!(
                "| {nome} · {} | {} | {} |",
                v.nome,
                crate::a_escada_da_sombra::por_contorno(&e, |x| x.degrau_px),
                crate::a_escada_da_sombra::por_contorno(&e, |x| x.media_px)
            );
        }
    }
}

/// D. A faixa da face acesa (F3).
fn faixa(gpu: &GpuContext) {
    eprintln!("\n## D. Face acesa da parede (cena 1): a pior razão média placa ÷ gabarito");
    eprintln!("| variante | pior · a px da face |");
    eprintln!("|---|---|");
    let cena = cena_de_luz(&crate::a_face_acesa_da_parede::cena_1());
    let regua = crate::a_face_acesa_da_parede::Regua::nova(&cena);
    for (nome, nivel) in NIVEIS {
        for v in &VARIANTES {
            let (pior, onde) = regua.pior(&fluencia(gpu, config(gpu, nivel, v), &cena));
            eprintln!("| {nome} · {} | {pior:.3} · {onde:.2} |", v.nome);
        }
    }
}

/// E. O oclusor fino de 5 cm (F2): o pior erro relativo da massa e o pior do fundo.
fn oclusor_fino(gpu: &GpuContext) {
    use crate::o_oclusor_fino::{CASOS, LAMPADAS, mundo, sombra};
    eprintln!("\n## E. Oclusor fino de 5 cm (as duas orientações, ponto e lâmpada pequena)");
    eprintln!("| variante | pior massa (rel.) | pior fundo |");
    eprintln!("|---|---|---|");
    let [a, b, ..] = afim().as_coeffs();
    let ppm = a.hypot(b);
    for (nome, nivel) in NIVEIS {
        for v in &VARIANTES {
            let cfg = config(gpu, nivel, v);
            let (mut massa, mut fundo) = (0.0_f64, 0.0_f64);
            for (_, raio) in LAMPADAS {
                let sem: World = mundo(raio, None);
                let (gs, fs) = (cena_de_luz(&sem), fluencia(gpu, cfg, &cena_de_luz(&sem)));
                for caso in &CASOS {
                    let gc = cena_de_luz(&mundo(raio, Some(caso.lados)));
                    let fc = fluencia(gpu, cfg, &gc);
                    let gab = |p: [f64; 2]| {
                        ph2d_light2d_ref::fluencia(&gc, p)[1]
                            / ph2d_light2d_ref::fluencia(&gs, p)[1].max(1e-12)
                    };
                    let placa = |p: [f64; 2]| fc.amostra(p) / fs.amostra(p).max(1e-12);
                    for x in caso.xs {
                        let (mp, fp) = sombra(placa, x, caso.ys, ppm);
                        let (mg, fg) = sombra(gab, x, caso.ys, ppm);
                        massa = massa.max((mp - mg).abs() / mg);
                        fundo = fundo.max((fp - fg).abs());
                    }
                }
            }
            eprintln!("| {nome} · {} | {massa:.3} | {fundo:.3} |", v.nome);
        }
    }
}

/// F à parte (o relógio de parede pede `load` baixo): à mão, `… medida_luz2d_w5a_vello -- --ignored`.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn medida_luz2d_w5a_vello() {
    let Some(gpu) = gpu() else { return };
    eprintln!("# loadavg antes {}", loadavg());
    vello(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// F. O Vello da camada do vetor (20 caminhos) à resolução de cada captura, no Low.
fn vello(gpu: &GpuContext) {
    eprintln!("\n## F. Camada do vetor (Low, 20 caminhos, 1920×1080): `desenhar_vetores` ms (mín)");
    eprintln!("| captura | px do Vello por texel grosso | ms |");
    eprintln!("|---|---|---|");
    let mut doc = VecScene::new();
    let mut xforms = VecXforms::new();
    let mut w = World::new();
    w.spawn(ph2d_light2d_ecs::Luz::default());
    for k in 0..20 {
        let x = -8.0 + 0.8 * f64::from(k);
        let id = doc.push_path(rectangle([x, -1.0], [x + 0.4, 1.0]));
        xforms.insert(id, ph2d_vec_scene::Xform::IDENTITY);
        w.spawn((VecPathRef(id), BloqueiaLuz { opacidade: 1.0 }));
    }
    let um_px = v("1 px", Some(1.0), 1.0, 1.0);
    for v in [&VARIANTES[0], &um_px, &VARIANTES[1], &VARIANTES[0], &um_px, &VARIANTES[1]] {
        let d = degrau(NivelDaLuz::Baixa, v);
        let mut l = LuzDoQuadro::com_degrau(d);
        let mut ms = Vec::new();
        for rodada in 0..(3 * RODADAS + 2) {
            assert!(l.preparar(&w, afim(), FHD));
            let t0 = Instant::now();
            l.desenhar_vetores(gpu, &doc, &xforms);
            gpu.device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            if rodada >= 2 {
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        eprintln!(
            "| {} | {} | {:.2} |",
            v.nome,
            captura_do_quadro(afim(), d),
            ms.iter().copied().fold(f64::INFINITY, f64::min)
        );
    }
}
