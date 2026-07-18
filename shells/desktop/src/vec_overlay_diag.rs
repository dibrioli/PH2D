//! **Diagnóstico do overlay vetorial** — `PH2D_VEC_OVERLAY_DIAG=1`.
//!
//! Existe por um artefato que eu **não** consegui explicar por dedução: uma linha reta longa saindo
//! de uma forma envolvida por envelope (Enio, 2026-07-18). Três hipóteses foram MEDIDAS e refutadas
//! — a jacobiana do MLS é exata até o pino (não é cancelamento catastrófico), o kurbo não desenha
//! nada numa haste de comprimento zero, e um segmento reto/degenerado atravessa o `warp_path` sem
//! produzir ponto disparado.
//!
//! Quando a dedução falha três vezes, o passo certo não é a quarta hipótese: é **instrumentar o
//! produto** ([[feedback_harness_reproduces_mechanism_not_context]]). Este módulo imprime, do frame
//! REAL, quem tem geometria fora do lugar — e nomeia o dono do artefato em vez de o adivinhar.
//!
//! O sinal que ele procura é o que a foto mostra: um ponto de controle **muito** mais longe do que a
//! própria forma mede. Uma alça sã vive perto da curva; uma disparada é a assinatura de uma tangente
//! inválida a chegar ao fitter.

use ph2d_ecs::{Entity, SimWorld, VecEnvelope};
use ph2d_vec_scene::VecScene;

/// Quantas vezes a diagonal da própria forma um ponto de controle tem de estar para ser gritado.
/// Alto de propósito: o alvo é o disparado, não o levemente folgado.
const SUSPECT_K: f64 = 3.0;

/// Imprime o estado do overlay uma vez por segundo enquanto a env estiver ligada.
pub(crate) fn dump(scene: &VecScene, sim: &SimWorld, selection: Option<u64>, frame: u64) {
    if !frame.is_multiple_of(60) || std::env::var_os("PH2D_VEC_OVERLAY_DIAG").is_none() {
        return;
    }
    for path in scene.paths() {
        // A caixa das ÂNCORAS é o tamanho da forma; a das ALÇAS pode passar dela um pouco. Muito
        // além é o defeito.
        let (mut alo, mut ahi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        let (mut hlo, mut hhi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        let mut n = 0_usize;
        let mut nonfinite = 0_usize;
        let grow = |p: [f64; 2], lo: &mut [f64; 2], hi: &mut [f64; 2]| -> bool {
            if !p[0].is_finite() || !p[1].is_finite() {
                return false;
            }
            *lo = [lo[0].min(p[0]), lo[1].min(p[1])];
            *hi = [hi[0].max(p[0]), hi[1].max(p[1])];
            true
        };
        for v in path.verts_all() {
            n += 1;
            for (p, is_anchor) in [
                (v.anchor, true),
                (v.in_handle, false),
                (v.out_handle, false),
            ] {
                let ok = if is_anchor {
                    grow(p, &mut alo, &mut ahi)
                } else {
                    grow(p, &mut hlo, &mut hhi)
                };
                if !ok {
                    nonfinite += 1;
                }
            }
        }
        if n == 0 || !alo[0].is_finite() {
            continue;
        }
        let diag = (ahi[0] - alo[0]).hypot(ahi[1] - alo[1]).max(1e-9);
        let reach = (hhi[0] - hlo[0]).hypot(hhi[1] - hlo[1]) / diag;
        let flag = if nonfinite > 0 {
            " ⚠ NÃO-FINITO"
        } else if reach > SUSPECT_K {
            " ⚠ ALÇA DISPARADA"
        } else {
            ""
        };
        eprintln!(
            "[vec-diag] path {:?} verts={n} ancoras=({:.1},{:.1})..({:.1},{:.1}) \
             alcance_das_alcas={reach:.2}x{flag}",
            path.id, alo[0], alo[1], ahi[0], ahi[1]
        );
    }
    let Some(bits) = selection else {
        eprintln!("[vec-diag] selecao: NENHUMA (o overlay do envelope nao desenha nada)");
        return;
    };
    match sim.world().get::<VecEnvelope>(Entity::from_bits(bits)) {
        Some(env) => eprintln!(
            "[vec-diag] envelope {:?} gesto={:?} pinos={} cantos={:?}",
            bits,
            env.kind,
            env.pins.len(),
            env.corners
        ),
        None => eprintln!("[vec-diag] selecao {bits:?} nao e um envelope"),
    }
}
