//! ⭐⭐ W5o — **o custo do miolo e do entorno** (`docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W5o_2026-10-13_CUSTO.md`):
//! o miolo é limitado pelo NÚMERO de despachos (medido: ~1,2 µs cada, 605 por quadro na W5n; o nível
//! de 5×3 levava 64 por ciclo), e o entorno resolvia uma grade fina que ninguém lê. Na grade da
//! fluência do Low a 1080p (2560×1536), um disco opaco, a região da janela menor que a grade.
//!
//! Medido (RTX 5060 Ti, `smoke`, a rodada `medida_luz2d_w5o`, mínimo de 7 intercaladas):
//!
//! | | despachos do miolo | `c3_miolo` vazia · cena 5 (Low) | entorno cena 5 High |
//! |---|---|---|---|
//! | W5n | 605 | 1,067 · 1,171 ms | 8,70 ms |
//! | W5o | 247 | 0,720 · 0,813 ms (×0,68 · ×0,69) | 2,82 ms |

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Grade, HOLOGRAFICA_PADRAO, MIOLO_PADRAO, Miolo, Solver};

use crate::comum::{disco, gpu, oclusor, regular};

const LARGURA: u32 = 2560;
const ALTURA: u32 = 1536;
const TEXEL_M: f64 = 0.004;

/// Os despachos do miolo por quadro com a base no grupo, uma relaxação por despacho — MEDIDO 247 (sem
/// ela 605); o teto é o medido + 5 %.
const TETO_DESPACHOS: usize = 260;
/// ⭐ W5q — com o bloco também (`Miolo::bloco` = 4 nos níveis pequenos) — MEDIDO 157 (sem ele 247); o
/// teto é o medido + 5 %.
const TETO_DESPACHOS_DO_BLOCO: usize = 165;
/// O `c3_miolo` com a base ÷ sem ela, no mesmo processo — MEDIDO ×0,68–×0,69; o teto deixa folga para
/// o ruído do relógio (o controlo, a razão 1 do custo antigo, reprova).
const TETO_RAZAO: f64 = 0.8;
/// A grade fina + a visibilidade do entorno (`so_a_c0`) ÷ as mesmas sem a flag — os passes vazios
/// medem ~0 contra ~0,2 ms (a vista vazia) a ~5 ms (a cena 5 High).
const TETO_ENTORNO: f64 = 0.1;

fn cena() -> CenaDeLuz {
    let c = [
        (f64::from(LARGURA) * 0.5 * TEXEL_M) as f32,
        (f64::from(ALTURA) * 0.5 * TEXEL_M) as f32,
    ];
    CenaDeLuz {
        emissores: vec![disco(
            [c[0] - 1.6, c[1] + 0.4],
            0.15,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores: vec![oclusor(regular(c, 0.9, 96, 0.1), 1.0)],
        luzes: 1,
        ..Default::default()
    }
}

/// `bloco` = o `Miolo::bloco` (1 = uma relaxação por despacho: a lei da base isolada).
fn solver(gpu: &ph2d_gpu::GpuContext, base: bool, so_a_c0: bool, bloco: u32) -> Solver {
    let g = Grade {
        origem_m: [0.0, 0.0],
        texel_m: TEXEL_M,
        largura: LARGURA,
        altura: ALTURA,
    };
    let cfg = Config {
        holografica: Some(HOLOGRAFICA_PADRAO),
        miolo: Some(Miolo {
            base_no_grupo: base,
            bloco,
            ..MIOLO_PADRAO
        }),
        so_a_c0,
        ..Config::padrao_para(&gpu.device, g)
    };
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.regiao_fina([LARGURA / 4, ALTURA / 4], [LARGURA * 3 / 4, ALTURA * 3 / 4]);
    s
}

fn tem_relogio(gpu: &ph2d_gpu::GpuContext) -> bool {
    gpu.device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
}

/// ⭐⭐ **GATE — o miolo grava poucos despachos**: com a base no grupo (`c3_miolo_base.wgsl`) ≤
/// [`TETO_DESPACHOS`]; o controlo (um despacho por relaxação, a W5n) passa dele. ⭐ W5q: e com o bloco ≤
/// [`TETO_DESPACHOS_DO_BLOCO`], o controlo (sem ele) acima.
#[test]
fn a_base_do_miolo_corta_os_despachos() {
    let Some(gpu) = gpu() else { return };
    if !tem_relogio(&gpu) {
        eprintln!("sem TIMESTAMP_QUERY — o instrumento das fases não corre; gate saltado");
        return;
    }
    let c = cena();
    let despachos = |base: bool, bloco: u32| {
        let mut s = solver(&gpu, base, false, bloco);
        assert!(s.medir_fases_do_miolo(&gpu));
        s.quadro(&gpu, &c);
        let f = s.fases_do_miolo(&gpu).expect("fases");
        f.iter().map(|(_, _, n)| n).sum::<usize>()
    };
    let (com, sem) = (despachos(true, 1), despachos(false, 1));
    let bloco = despachos(true, MIOLO_PADRAO.bloco);
    eprintln!("despachos do miolo: com a base {com} · sem ela {sem} · com a base e o bloco {bloco}");
    assert!(
        com > TETO_DESPACHOS_DO_BLOCO,
        "controlo: sem o bloco o miolo grava {com} despachos — o teto do bloco não o separa"
    );
    assert!(
        bloco <= TETO_DESPACHOS_DO_BLOCO,
        "com o bloco o miolo grava {bloco} despachos por quadro (teto {TETO_DESPACHOS_DO_BLOCO})"
    );
    assert!(
        sem > TETO_DESPACHOS,
        "controlo: sem a base o miolo grava {sem} despachos — o teto não o separa"
    );
    assert!(
        com <= TETO_DESPACHOS,
        "o miolo grava {com} despachos por quadro (teto {TETO_DESPACHOS})"
    );
}

/// ⭐⭐ **GATE — o custo do miolo desce com a base, e o entorno não paga a grade fina**: no mesmo
/// processo, intercalado, o MÍNIMO de 7 rodadas (`docs/DevOps/MEDIR_VELOCIDADE.md`); a régua é a razão,
/// não o número. Controlos: o custo antigo contra si mesmo (razão 1) e a grade fina sem a flag.
#[test]
fn o_custo_do_miolo_desce_com_a_base() {
    let Some(gpu) = gpu() else { return };
    if !tem_relogio(&gpu) {
        eprintln!("sem TIMESTAMP_QUERY — gate saltado");
        return;
    }
    let c = cena();
    // [com a base, sem ela, sem ela (o controlo), entorno com a flag, entorno sem ela].
    let mut s = [
        solver(&gpu, true, false, 1),
        solver(&gpu, false, false, 1),
        solver(&gpu, false, false, 1),
        solver(&gpu, true, true, 1),
        solver(&gpu, true, false, 1),
    ];
    let medir = |t: &ph2d_light2d_gpu::TemposMs, k: usize| {
        if k >= 3 {
            t.c3_fina + t.c3_visibilidade + t.c3_miolo
        } else {
            t.c3_miolo
        }
    };
    let mut minimo = [f64::INFINITY; 5];
    for x in &mut s {
        for _ in 0..3 {
            x.quadro(&gpu, &c);
        }
    }
    for rodada in 0..7 {
        for o in 0..s.len() {
            let k = (o + rodada) % s.len();
            for _ in 0..3 {
                s[k].quadro(&gpu, &c);
            }
            let t = s[k].tempos(&gpu).expect("TIMESTAMP_QUERY");
            minimo[k] = minimo[k].min(medir(&t, k));
        }
    }
    let razao = minimo[0] / minimo[1];
    let controlo = minimo[2] / minimo[1];
    let entorno = minimo[3] / minimo[4];
    eprintln!(
        "c3_miolo: com a base {:.3} · sem ela {:.3} · {:.3} ms (razão {razao:.3}, controlo {controlo:.3}); \
         entorno fina+vis+miolo: {:.3} contra {:.3} ms (razão {entorno:.3})",
        minimo[0], minimo[1], minimo[2], minimo[3], minimo[4]
    );
    assert!(
        controlo > TETO_RAZAO,
        "controlo: o custo antigo contra si mesmo dá {controlo:.3} — a régua não separa"
    );
    assert!(
        razao <= TETO_RAZAO,
        "o miolo com a base custa {razao:.3} do de sem ela (teto {TETO_RAZAO})"
    );
    assert!(
        entorno <= TETO_ENTORNO,
        "o entorno com `so_a_c0` ainda paga {entorno:.3} da grade fina e do miolo (teto {TETO_ENTORNO})"
    );
}
