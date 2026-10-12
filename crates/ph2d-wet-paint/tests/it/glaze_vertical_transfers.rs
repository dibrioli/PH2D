//! **Com o «Glaze layering» ligado, as transferências verticais guardam a lei que o ecrã mostra.**
//!
//! A caixa é UMA lei da sessão (`Sim::camadas_transparentes` → `Sim::gather_params` → `Params::ecra`): o
//! render compõe com ela e as quatro portas que mudam pigmento de camada — assentar na secagem
//! (`dry_cell`), o Dry canvas, o re-molhar do Wet e o do Smear (`lift_settled`) — resolvem a cor nova pela
//! inversa da MESMA lei. Se uma delas construísse o seu `Ecra` à parte (o tapar do produto, `esconde` 5,
//! enquanto o ecrã tapa 2), o traço mudaria de cor ao secar ou ao re-molhar e nenhum portão de texto o via
//! (`05 §4b` «Mistura II», mutação M7b).
//!
//! A régua é o ecrã SECO que o render mostra, célula a célula (`assenta::sobre_branco(p.ecra.seco(), …)`),
//! antes e depois de cada passe vertical. A cena: duas tintas diferentes sobrepostas (assentada por baixo,
//! em suspensão por cima), quatro pares de tintas em faixas, massas que variam por linha e são uniformes ao
//! longo de `x` — o Smear arrasta na horizontal e numa cena uniforme em `x` o arrasto lateral é nulo, então o
//! que muda o ecrã é só a transferência vertical.
//!
//! ⚠️ Assentar preserva o ecrã ao centésimo de nível (com e sem a caixa); RE-MOLHAR tintas diferentes NÃO o
//! preserva por desenho — a caixa das tintas (`assenta::na_caixa`) prende a inversa, e sem o «Glaze» ela
//! prende quase todas as células (um traço de Wet muda até 40 níveis, de Smear até 48). Por isso o re-molhar
//! mede-se pela lei dele: dab a dab, onde a caixa NÃO prendeu, o ecrã fica; e o traço inteiro tem tecto.

use ph2d_wet_paint::assenta::{Lei, sobre_branco};
use ph2d_wet_paint::colorops::{
    TINTA_ESCONDE, TINTA_MOLHADA_ESCURECE, VELATURA_DO_FILME, VELATURA_MEDIDA,
};
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::painter::{Engine, Tool};

use crate::util::drive_stroke;

const W: usize = 48;
const H: usize = 40;

const AZUL: [f32; 3] = [20.0, 50.0, 110.0];
const AMARELO: [f32; 3] = [245.0, 205.0, 25.0];
const VERMELHO: [f32; 3] = [220.0, 60.0, 60.0];
const VERDE: [f32; 3] = [40.0, 150.0, 70.0];
/// (assentada, em suspensão) por faixa de 10 linhas.
const PARES: [([f32; 3], [f32; 3]); 4] = [
    (AZUL, AMARELO),
    (VERMELHO, AZUL),
    (AMARELO, VERMELHO),
    (VERDE, VERMELHO),
];

// Os tectos, em níveis de 0–255, MEDIDOS nesta cena (`05 §4b` «Mistura II», «O portão das transferências»).
/// Assentar (secagem, Dry canvas), por célula: sem a caixa 1,4e-4 · com 1,3e-4 · M7b 51,3.
const TOL_ASSENTAR: f64 = 0.01;
/// Re-molhar, dab a dab, nas células que a caixa das tintas NÃO prendeu — o tecto de `assenta::tests`;
/// com o «Glaze» Wet 0,0095 · Smear 0,264; M7b no Smear 2,33. Com a caixa no ECRÃ (Bug #50, 12/10): Wet 1,50 ·
/// Smear 2,97 — um canal encostado à mais clara das duas tintas não se repõe quando o re-molhar afina o de baixo
/// (a caixa obriga o ecrã a mudar com a tinta levantada). O M7b continua a reprovar pelas livres e pelo traço.
const TOL_LIVRES: f64 = 3.0;
/// Quantas células-dab (17 dabs) a caixa das tintas tem de deixar livres: com o «Glaze» Wet 484 · Smear 476;
/// sem a caixa — e com M7b — 0 e 6 (a inversa do tapar 5 sai da caixa quase sempre).
const MIN_LIVRES: usize = 100;
/// O traço inteiro de re-molhar, o maior desvio de uma célula: com o «Glaze» Wet 4,13 · Smear 13,84; sem a
/// caixa 40,2 · 47,7; M7b 25,6 · 43,4. Com a caixa no ECRÃ (Bug #50): Wet 15,36 · Smear 14,71 (sem o «Glaze»
/// 20,67 · 19,90, sem tecto).
const TECTO_TRACO_WET: f64 = 16.0;
const TECTO_TRACO_SMEAR: f64 = 25.0;

/// O motor com a lei da sessão do produto (`authored_actions::ensure_wet_session`) e a cena das duas tintas.
fn cena(glaze: bool) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.glaze = VELATURA_MEDIDA;
    e.sim.glaze_filme = VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = Lei::Cobre;
    e.sim.molhada_escurece = TINTA_MOLHADA_ESCURECE;
    e.sim.esconde = TINTA_ESCONDE;
    e.sim.camadas_transparentes = glaze;
    let g = e.active_grid_mut();
    for y in 1..=H {
        let (sc, uc) = PARES[((y - 1) / 10) % PARES.len()];
        let k = ((y - 1) % 10) as f32;
        for x in 1..=W {
            let i = x + y * g.s;
            g.sett[i] = 60.0 + 90.0 * k;
            g.susp[i] = 900.0 - 80.0 * k;
            g.film[i] = 0.3;
            g.sett_rgb[i] = sc;
            g.susp_rgb[i] = uc;
        }
    }
    g.has_fluid = true;
    g.expand_bbox(1, 1, W as i32, H as i32);
    ph2d_wet_paint::solver::rebuild_active_region(g);
    e
}

/// O ecrã SECO de cada célula, pela porta do render.
fn ecra_seco(e: &Engine) -> Vec<[f64; 3]> {
    let lei = e.sim.gather_params(&e.tuning).ecra.seco();
    let g = e.active_grid();
    let mut v = Vec::with_capacity(W * H);
    for y in 1..=H {
        for x in 1..=W {
            let i = x + y * g.s;
            v.push(sobre_branco(
                lei,
                g.sett_rgb[i].map(f64::from),
                g.susp_rgb[i].map(f64::from),
                alpha_of_mass(g.sett[i] as f64),
                alpha_of_mass(g.susp[i] as f64),
            ));
        }
    }
    v
}

/// O desvio de uma célula entre dois ecrãs (o maior canal, em níveis).
fn desvio(p: [f64; 3], q: [f64; 3]) -> f64 {
    (0..3).map(|c| (p[c] - q[c]).abs()).fold(0.0, f64::max)
}

/// O que um passe fez ao ecrã: o maior desvio, e em quantas células o pigmento mudou de camada.
#[derive(Debug, Default, Clone, Copy)]
struct Medida {
    desvio: f64,
    movidas: usize,
}

/// Assentar: um passe que só DESCE pigmento — o ecrã seco de cada célula, antes e depois.
fn medir_assentar(glaze: bool, passe: impl Fn(&mut Engine)) -> Medida {
    let mut e = cena(glaze);
    let mut m = Medida::default();
    for _ in 0..4 {
        let (antes, sett) = (ecra_seco(&e), e.active_grid().sett.clone());
        passe(&mut e);
        let depois = ecra_seco(&e);
        let g = e.active_grid();
        let mut k = 0;
        for y in 1..=H {
            for x in 1..=W {
                let i = x + y * g.s;
                if (g.sett[i] - sett[i]).abs() > 1e-3 {
                    m.movidas += 1;
                    m.desvio = m.desvio.max(desvio(antes[k], depois[k]));
                }
                k += 1;
            }
        }
    }
    m
}

/// A secagem que o produto roda (Jacobi): a evaporação leva parte da água e o pigmento dela assenta.
fn secagem(e: &mut Engine) {
    let p = e.sim.gather_params(&e.tuning);
    let (rewet, bypass) = (e.sim.rewet_base, e.sim.ext_bypass);
    ph2d_wet_paint::drying::drying_pass_jacobi(e.active_grid_mut(), &p, 40.0, rewet, bypass);
}

/// Re-molhar, UM dab de cada vez (a caixa das tintas lê-se por dab): nas células em que o filme novo ficou
/// DENTRO da caixa das duas tintas (a inversa não foi presa), o ecrã seco não muda. `livres` conta-as;
/// `presas` as que a caixa prendeu (lá o ecrã muda por desenho — `assenta::na_caixa`).
fn medir_levantar(glaze: bool, tool: Tool) -> (Medida, usize, f64) {
    let mut e = cena(glaze);
    e.tool = tool;
    let (mut livres, mut presas, mut pior_presa) = (Medida::default(), 0usize, 0.0f64);
    let mut x = 8.0;
    while x <= 40.0 {
        let antes = ecra_seco(&e);
        let g = e.active_grid();
        let (sett, sc, uc) = (g.sett.clone(), g.sett_rgb.clone(), g.susp_rgb.clone());
        e.dispatch_pressure_dab(x, 20.0, 2.0, 1.0, 0.0, 6.0);
        let depois = ecra_seco(&e);
        let g = e.active_grid();
        let mut k = 0;
        for yy in 1..=H {
            for xx in 1..=W {
                let i = xx + yy * g.s;
                if sett[i] - g.sett[i] > 1e-3 {
                    let u2 = g.susp_rgb[i];
                    let dentro = (0..3).all(|c| {
                        let (lo, hi) = (uc[i][c].min(sc[i][c]), uc[i][c].max(sc[i][c]));
                        u2[c] > lo + 1e-3 && u2[c] < hi - 1e-3
                    });
                    let d = desvio(antes[k], depois[k]);
                    if dentro {
                        livres.movidas += 1;
                        livres.desvio = livres.desvio.max(d);
                    } else {
                        presas += 1;
                        pior_presa = pior_presa.max(d);
                    }
                }
                k += 1;
            }
        }
        x += 2.0;
    }
    (livres, presas, pior_presa)
}

/// Um traço inteiro de re-molhar pelo caminho do produto (`drive_stroke`): o maior desvio de uma célula.
fn medir_traco(glaze: bool, tool: Tool) -> f64 {
    let mut e = cena(glaze);
    let antes = ecra_seco(&e);
    e.tool = tool;
    drive_stroke(&mut e, 6.0, 20.0, 42.0, 20.0, 3.0, 0);
    let depois = ecra_seco(&e);
    antes
        .iter()
        .zip(&depois)
        .map(|(p, q)| desvio(*p, *q))
        .fold(0.0, f64::max)
}

/// **Assentar não muda o ecrã seco — com o «Glaze layering» como sem ele** (a secagem e o Dry canvas).
/// Mutação que sangra: a secagem (`DryConst`) ou o Dry canvas com o `Ecra` construído à parte com o tapar do
/// produto (M7b.1, M7b.2: Δ51,3).
#[test]
fn com_o_glaze_assentar_guarda_o_ecra_seco() {
    for glaze in [false, true] {
        for (porta, m) in [
            ("secagem", medir_assentar(glaze, secagem)),
            ("Dry canvas", medir_assentar(glaze, |e| e.dry_canvas_now())),
        ] {
            println!(
                "glaze={glaze} {porta}: Δ{:e} ({} células)",
                m.desvio, m.movidas
            );
            assert!(
                m.movidas >= W * H,
                "{porta}: a porta não foi exercida ({} células assentaram)",
                m.movidas
            );
            assert!(
                m.desvio <= TOL_ASSENTAR,
                "glaze={glaze} {porta}: o ecrã seco mudou Δ{:.3} ao assentar — a porta guarda outra lei que o ecrã",
                m.desvio
            );
        }
    }
}

/// **Com o «Glaze layering», re-molhar guarda o ecrã seco onde a caixa das tintas não prende**, e o traço
/// inteiro fica sob o tecto medido (o Wet e o Smear, as duas portas do `lift_settled` ativo). Mutação que
/// sangra: o Wet ou o Smear com o `Ecra` construído à parte com o tapar do produto (M7b.3: 0 células livres,
/// traço Δ25,6; M7b.4: 6 livres com Δ2,33, traço Δ43,4).
#[test]
fn com_o_glaze_re_molhar_guarda_o_ecra_seco_onde_a_caixa_das_tintas_nao_prende() {
    for (tool, tecto) in [
        (Tool::Wet, TECTO_TRACO_WET),
        (Tool::Smear, TECTO_TRACO_SMEAR),
    ] {
        let (sem, sem_presas, _) = medir_levantar(false, tool);
        let (livres, presas, pior_presa) = medir_levantar(true, tool);
        let (traco_sem, traco) = (medir_traco(false, tool), medir_traco(true, tool));
        println!(
            "{tool:?}: com o glaze {} livres Δ{:.4} · {presas} presas Δ{pior_presa:.2} · traço Δ{traco:.2} \
             | sem: {} livres · {sem_presas} presas · traço Δ{traco_sem:.2}",
            livres.movidas, livres.desvio, sem.movidas
        );
        assert!(
            livres.movidas >= MIN_LIVRES,
            "{tool:?}: só {} células com o filme dentro da caixa das tintas — a inversa do re-molhar não é a do \
             «Glaze» que o ecrã mostra",
            livres.movidas
        );
        assert!(
            livres.desvio <= TOL_LIVRES,
            "{tool:?}: onde a caixa não prendeu o ecrã seco mudou Δ{:.3} — a porta guarda outra lei que o ecrã",
            livres.desvio
        );
        assert!(
            traco <= tecto,
            "{tool:?}: um traço de re-molhar mudou o ecrã seco Δ{traco:.2} > {tecto}"
        );
    }
}
