//! **A rodada da W5g — o F5, o miolo das formas** (`docs/Luz2D/01_plano.md` F5; método
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): as variantes no MESMO processo, intercaladas. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5g -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o disco e a elipse POLÍGONOS da cena 5 contra o gabarito, em ZOOMS CONTÍNUOS e três fases da
//!    grade, nos anéis do portão e mais três (0,5 · 0,8 · 0,97 — a passagem luz/sombra que sai dos
//!    pontos tangentes), por variante ([`VARIANTES`]);
//! B. o disco de IMAGEM (o sprite do atlas) nos mesmos zooms.

use ph2d_light2d_gpu::Config;
use vello::kurbo::Affine;

use crate::as_formas_redondas::{
    ANEIS, Leitura, anel, cena_redonda, disco_de_imagem, ler_anel, passo_tx, vista_em,
};
use crate::comum::gpu;
use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5d::solver;

// Um zoom por OITAVA: dentro dela a grade é a mesma no mundo (medido: 0,8 = 1; 1,3 = 1,7 = 2,1).
const ZOOMS: [f64; 3] = [1.0, 1.5, 2.8];
const FASES: [u32; 3] = [0, 3, 5];
const MAIS_ANEIS: [(&str, f64, f64); 3] = [
    ("dentro 0,5", 0.5, 0.0),
    ("dentro 0,8", 0.8, 0.0),
    ("dentro 0,97", 0.97, 0.0),
];
const N_ANEIS: usize = 7;

/// `(nome, Config::pe_na_borda, Config::vis_na_face)`.
const VARIANTES: [(&str, bool, bool); 4] = [
    ("W5f", false, false),
    ("pé", true, false),
    ("face", false, true),
    ("pé+face", true, true),
];

fn config(gpu: &ph2d_gpu::GpuContext, a: Affine, v: usize) -> Config {
    Config {
        pe_na_borda: VARIANTES[v].1,
        vis_na_face: VARIANTES[v].2,
        ..crate::medida_luz2d_w5d::config(gpu, a, crate::medida_luz2d_w5d::VARIANTES[2])
    }
}

#[test]
#[ignore = "a rodada da W5g: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w5g() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W5g · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    let aneis_todos: Vec<(&str, f64, f64)> =
        ANEIS.iter().chain(MAIS_ANEIS.iter()).copied().collect();
    assert_eq!(aneis_todos.len(), N_ANEIS);
    // [variante][anel] — o pior sobre zooms, fases e corpos.
    let mut pior = [[Leitura::default(); N_ANEIS]; VARIANTES.len()];
    for &zoom in &ZOOMS {
        for ri in 0..2 {
            for &k in &FASES {
                let a = vista_em(zoom, cena_redonda([0.0; 2]).1[ri].centro);
                let cfg0 = config(&gpu, a, 0);
                let tx = cfg0.grade.texel_m / f64::from(cfg0.fina);
                let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
                let (c, redondos) = cena_redonda(d);
                let r = redondos[ri];
                let aneis: Vec<_> = aneis_todos
                    .iter()
                    .map(|(nome, f, fora)| (*nome, anel(&c, &r, *f, fora * tx)))
                    .collect();
                let max = aneis
                    .iter()
                    .flat_map(|(_, (_, v))| v.iter().copied())
                    .fold(0.0, f64::max);
                // Intercaladas: a ordem das variantes roda com a fase.
                for j0 in 0..VARIANTES.len() {
                    let v = (j0 + k as usize) % VARIANTES.len();
                    let mut s = solver(&gpu, a, config(&gpu, a, v));
                    s.quadro(&gpu, &c);
                    let fl = crate::comum::fluencia_lida(&gpu, &s);
                    let mut linha = String::new();
                    for (j, (nome, (pts, gab))) in aneis.iter().enumerate() {
                        let eixo = |p: [f64; 2]| r.ao_eixo(p) < 2.0 * tx;
                        let l = ler_anel(&fl, pts, gab, max, passo_tx(pts, tx), eixo);
                        pior[v][j].somar(l);
                        linha += &format!(
                            " · {nome}: {}e ×{} ↔{:.1} TV {:+.2}",
                            l.errados, l.cruzes, l.desvio, l.tv
                        );
                    }
                    eprintln!(
                        "A · zoom {zoom} · {k}/8 · {} · {}{linha}",
                        r.nome, VARIANTES[v].0
                    );
                }
            }
        }
    }
    for (v, (nome, ..)) in VARIANTES.iter().enumerate() {
        for (j, (anel_nome, ..)) in aneis_todos.iter().enumerate() {
            let l = pior[v][j];
            eprintln!(
                "A · PIOR · {nome} · {anel_nome}: {} errados ({:.3}) · ×{} · ↔{:.1} tx · TV {:+.2}",
                l.errados, l.pior, l.cruzes, l.desvio, l.tv
            );
        }
    }
    for &zoom in &ZOOMS {
        let a = vista_em(zoom, cena_redonda([0.0; 2]).1[0].centro);
        for (v, variante) in VARIANTES.iter().enumerate() {
            let cfg = config(&gpu, a, v);
            let tx = cfg.grade.texel_m / f64::from(cfg.fina);
            let (fl, r, gab_cena) = disco_de_imagem(&gpu, a, cfg);
            let mut linha = String::new();
            let mut aneis = Vec::new();
            for (nome, f, fora) in &aneis_todos {
                aneis.push((*nome, anel(&gab_cena, &r, *f, fora * tx)));
            }
            let max = aneis
                .iter()
                .flat_map(|(_, (_, v))| v.iter().copied())
                .fold(0.0, f64::max);
            for (nome, (pts, gab)) in &aneis {
                let eixo = |p: [f64; 2]| r.ao_eixo(p) < 2.0 * tx;
                let l = ler_anel(&fl, pts, gab, max, passo_tx(pts, tx), eixo);
                linha += &format!(
                    " · {nome}: {}e ×{} ↔{:.1} TV {:+.2}",
                    l.errados, l.cruzes, l.desvio, l.tv
                );
            }
            eprintln!("B · imagem · zoom {zoom} · {}{linha}", variante.0);
        }
    }
    custo(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// C. o custo da C3 (preenche + visibilidade + fina, ms, o MÍNIMO de [`RODADAS`] intercaladas) nas
/// cenas 1–6, vista de fábrica, sem e com o pé + a face.
fn custo(gpu: &ph2d_gpu::GpuContext) {
    use crate::medida_luz2d_w5d::{RODADAS, vista};
    let a = vista(1.0);
    let cenas: Vec<_> = (1..=6)
        .map(|n| {
            let mut w = bevy_ecs::world::World::new();
            assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
            ph2d_light2d_ecs::coleta::cena_de_luz(&w)
        })
        .collect();
    let vs = [0, VARIANTES.len() - 1];
    let mut solvers = Vec::new();
    for ci in 0..cenas.len() {
        for &v in &vs {
            solvers.push((ci, solver(gpu, a, config(gpu, a, v))));
        }
    }
    let mut minimo = vec![f64::INFINITY; solvers.len()];
    for rodada in 0..RODADAS {
        for j in 0..solvers.len() {
            let i = (j + rodada) % solvers.len();
            let (ci, s) = &mut solvers[i];
            for _ in 0..3 {
                s.quadro(gpu, &cenas[*ci]);
            }
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            minimo[i] = minimo[i].min(t.c3_preenche + t.c3_visibilidade + t.c3_fina);
        }
    }
    for ci in 0..cenas.len() {
        eprintln!(
            "C · cena {} · {} {:.3} ms · {} {:.3} ms",
            ci + 1,
            VARIANTES[vs[0]].0,
            minimo[2 * ci],
            VARIANTES[vs[1]].0,
            minimo[2 * ci + 1]
        );
    }
}
