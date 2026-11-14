//! State fingerprint after a deterministic scripted session — the refactor
//! guard: any hot-loop rewrite (drying locals, flow hoists) must leave this
//! EXACT value untouched, because the engine is deterministic end to end and
//! "the JS is the spec" leaves no room for an ulp of drift.
//!
//! The pinned value was produced by the FIRST straight port (pre any perf
//! work) and cross-checked against it after each rewrite via git stash.

use crate::util;

use ph2d_wet_paint::painter::{Engine, Tool};
use util::drive_stroke;

/// FNV-1a over the f32 bit patterns of the fields that matter. The byte
/// ORDER is the original planar one (all R, then all G, then all B) so the
/// pin survives the interleaved-color layout change — the layout is not the
/// simulation.
fn fingerprint(e: &Engine) -> u64 {
    let g = e.active_grid();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    fn eat(h: &mut u64, arr: &[f32]) {
        for v in arr {
            for b in v.to_bits().to_le_bytes() {
                *h ^= b as u64;
                *h = h.wrapping_mul(0x1000_0000_01b3);
            }
        }
    }
    fn eat_channel(h: &mut u64, arr: &[[f32; 3]], ch: usize) {
        for v in arr {
            for b in v[ch].to_bits().to_le_bytes() {
                *h ^= b as u64;
                *h = h.wrapping_mul(0x1000_0000_01b3);
            }
        }
    }
    eat(&mut h, &g.film);
    eat(&mut h, &g.susp);
    eat(&mut h, &g.sett);
    eat_channel(&mut h, &g.susp_rgb, 0);
    eat_channel(&mut h, &g.susp_rgb, 1);
    eat_channel(&mut h, &g.susp_rgb, 2);
    eat_channel(&mut h, &g.sett_rgb, 0);
    eat_channel(&mut h, &g.sett_rgb, 1);
    eat_channel(&mut h, &g.sett_rgb, 2);
    eat(&mut h, &g.vel_x);
    eat(&mut h, &g.vel_y);
    h
}

fn scripted_session(wet_lift: Option<f64>) -> Engine {
    scripted_session_with(wet_lift, true)
}

fn scripted_session_with(wet_lift: Option<f64>, gather: bool) -> Engine {
    scripted_session_por_tile(wet_lift, gather, false)
}

/// `por_tile` = `Sim::secagem_por_tile` (B13.4), fixado na sessão.
fn scripted_session_por_tile(wet_lift: Option<f64>, gather: bool, por_tile: bool) -> Engine {
    // A session that exercises deposit + trail + drying + flow + advect +
    // projection + a wet pass + drip under tilt.
    let mut e = Engine::new(300, 200);
    e.sim.order_invariant = gather;
    e.sim.secagem_por_tile = por_tile;
    if let Some(v) = wet_lift {
        e.set_knob(ph2d_wet_paint::tuning::Knob::WetLift, v);
    }
    drive_stroke(&mut e, 40.0, 60.0, 260.0, 90.0, 4.0, 30);
    e.tool = Tool::Wet;
    drive_stroke(&mut e, 40.0, 120.0, 260.0, 120.0, 5.0, 10);
    e.tool = Tool::Paint;
    e.sim.gravity_override = Some([0.0, 1.0]);
    drive_stroke(&mut e, 60.0, 40.0, 240.0, 40.0, 3.0, 80);
    e
}

#[test]
fn scripted_session_fingerprint_is_stable() {
    let fp = fingerprint(&scripted_session(None));
    // Pinned (see history below). If a rewrite moves this number, the
    // rewrite changed the simulation — find out why before touching the pin.
    assert_eq!(fp, PINNED, "session fingerprint drifted: {fp:#018x}");
}

/// **A rota Gauss-Seidel reproduz o pino DELA, ao byte** (doc 28 §5.45).
///
/// ⚠️ **Este é o gate que torna a troca de modelo AUDITÁVEL em vez de um pino
/// que se moveu.** O `order_invariant = false` devolve o
/// [`ph2d_wet_paint::solver::advect`] serial, e a sessão inteira volta a
/// bater o valor que estava pinado antes da wave — logo **nada além do
/// advect mudou**: nem a secagem, nem o fluxo, nem a projeção, nem o
/// depósito, nem o `lift_settled`. Sem ele, um pino novo é indistinguível de
/// uma regressão silenciosa em qualquer outro passe.
#[test]
fn the_gauss_seidel_route_still_reproduces_its_own_pin() {
    let fp = fingerprint(&scripted_session_with(None, false));
    assert_eq!(
        fp, PINNED_GAUSS_SEIDEL,
        "a rota serial mudou — o gather nao era a unica diferenca: {fp:#018x}"
    );
}

/// Doc 23 §5 gate: `wetLift = 0` IS the pre-doc-23 model, to the byte — the
/// same scripted session reproduces the pin that stood before the active
/// lift landed. This is also the proof that extracting `lift_settled` out
/// of the drying pass was pure code motion (the passive re-wet runs in this
/// session and hashes identically).
///
/// ⚠️ Roda na rota **serial**: o pino que ele afirma é anterior ao gather, e
/// compará-lo com o produto de hoje mediria as DUAS mudanças de uma vez.
#[test]
fn wet_lift_zero_is_the_old_model_to_the_byte() {
    let fp = fingerprint(&scripted_session_with(Some(0.0), false));
    assert_eq!(
        fp, PINNED_PRE_DOC23,
        "wetLift=0 must reproduce the pre-doc-23 session: {fp:#018x}"
    );
}

// Pin history — every move names the SEMANTIC change that justified it:
// 0x1d26_6795_d687_a4c4  first straight port (survived the drying-locals,
//                        flow row-slice and interleaved-color rewrites —
//                        those were layout, not simulation).
// 0x6097_a692_a23d_bd5f  the port-verify parity FIXES (both move the Rust
//                        TOWARD the JS): advect's `film[i] += q00+q10+q01+q11`
//                        sums the q's before adding to the cell, and the
//                        paper lattices are Float32Array (each rng draw
//                        rounds to f32 before the noise sampler reads it).
// 0x99d8_891b_57a7_2abe  doc 23 P1: the Wet tool ACTIVELY lifts settled
//                        pigment through `lift_settled` (wetLift 0.25) —
//                        the session's Wet stroke now dissolves what the
//                        first stroke dried. wetLift=0 still reproduces
//                        the previous pin (gate above): the drift is the
//                        feature, not an accident.
// 0x8dc7_134c_39c9_f84c  doc 28 §5.45: o `advect` virou GATHER (Jacobi). É
//                        uma troca de MODELO, não uma reescrita — e o que a
//                        justifica não é velocidade: o Gauss-Seidel QUEBRA a
//                        simetria da cena (medido em
//                        `tests/solver_symmetry.rs`: 1189 unidades de massa
//                        de viés esquerda→direita numa folha espelhada, contra
//                        0,000000 do gather), porque ele lê o canto que a
//                        célula anterior já drenou. O pino ANTIGO segue
//                        executável na rota `order_invariant = false` (gate
//                        acima), e é isso que prova que o advect foi a ÚNICA
//                        coisa que mudou.
// 0x5b699a43f65b6c34     doc 28 §5.45, a outra metade: a SECAGEM também. O
//                        fator de borda lia a vizinhança 3×3 de `susp` que o
//                        próprio passe reescreve — vizinhos JÁ secos à
//                        esquerda, ainda molhados à direita —, e agora ele é
//                        materializado num pré-passe. O gate da rota serial
//                        continua verde COM a extração do `dry_cell`, e é
//                        isso que prova que a fatoração do kernel foi pure
//                        code motion.
// 0xc6dc_cb4a_3205_0bd3  03/11: a umidade sobe em uma célula e a orla não seca o miolo
//                        (docs/DevOps/MEDICAO_P1_P4_LARGURA_2026-11-03.md, §D). Os três pinos movem-se juntos; o da cura
//                        62e7fc8505 nunca foi gravado.
//                        Estes três são a omissão do produto (`Sim::secagem_por_tile = false`).
const PINNED: u64 = 0xc6dc_cb4a_3205_0bd3;

/// O pino da rota serial — o `PINNED` que valia antes da §5.45.
const PINNED_GAUSS_SEIDEL: u64 = 0x217c_aed8_783d_a46c;
const PINNED_PRE_DOC23: u64 = 0x32a4_8f31_eab5_9161;

/// 14/11 (B13.4, desligado até o dono aprovar): a cadência da secagem é do TILE 64×64 — o tile calmo seca no ritmo
/// calmo mesmo quando outro corre (`docs/Painter/missao_4k/medicoes/N10_agua_wet.md`). `(gather, GS, pré-doc23)`.
const PINNED_POR_TILE: [u64; 3] = [
    0x6fab_49bb_a2a3_c433,
    0x2cce_ea8a_dd8a_352f,
    0x1f86_0b43_c379_c36d,
];

#[test]
fn the_per_tile_drying_cadence_reproduces_its_three_pins() {
    let got = [
        fingerprint(&scripted_session_por_tile(None, true, true)),
        fingerprint(&scripted_session_por_tile(None, false, true)),
        fingerprint(&scripted_session_por_tile(Some(0.0), false, true)),
    ];
    let s = got.iter().map(|h| format!("{h:#018x}")).collect::<Vec<_>>();
    assert_eq!(
        got, PINNED_POR_TILE,
        "a secagem por tile mudou de bits: {s:?}"
    );
}
