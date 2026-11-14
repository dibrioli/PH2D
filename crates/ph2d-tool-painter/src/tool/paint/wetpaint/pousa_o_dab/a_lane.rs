//! **A lane do dab e o peso dele** — o começo de [`super::pousa_o_dab`] (filho dele, LOC cap, B52.R11).

use super::super::*;
use super::LoteDoDab;
use ph2d_det_math::DetMath as _;

/// A lane que o dab continua, o centro ANTERIOR dela e o caminho desde esse centro.
pub(super) struct LaneDoDab {
    pub(super) li: usize,
    /// O centro anterior (antes deste dab o mudar) — a fonte do deslocamento das ferramentas directas; uma
    /// lane que nasce não tem, e a borracha não tem lanes.
    pub(super) lane_prev: Option<[f32; 2]>,
    pub(super) caminho: f32,
}

/// A lane do dab ([`super::super::rota_da_lane::casa_a_lane`]); a borracha não tem lanes.
pub(super) fn casa(l: &LoteDoDab, sess: &mut WetSession, didx: usize, d: &Dab) -> LaneDoDab {
    let &LoteDoDab {
        ref brush,
        erasing,
        wet_tool,
        ref copia,
        virtual_,
        gota,
        ..
    } = l;
    let [x, y] = d.center;
    let (li, lane_born) = if erasing {
        (0, false)
    } else if let Some(&li) = sess.fecho.get(didx) {
        // Um dab virtual das pontas rectas: a lane é a que ele continua ([`super::super::tampa`]).
        let p = sess.lanes[li].pos;
        let corda = (x - p[0]).det_hypot(y - p[1]);
        let caminho = super::super::tampa::passo_virtual(&sess.lanes[li], brush);
        super::super::rota_da_lane::anda_a_lane(sess, li, corda, caminho, wet_tool);
        (li, false)
    } else {
        super::super::rota_da_lane::casa_a_lane(sess, d, brush, copia[didx], wet_tool, gota)
    };
    let lane_prev: Option<[f32; 2]> = (!erasing && !lane_born).then(|| sess.lanes[li].pos);
    // O caminho desde o dab anterior da lane: o de um dab virtual é o passo dela (a abertura anda para trás).
    let caminho = if lane_prev.is_none() {
        0.0
    } else if virtual_ {
        super::super::tampa::passo_virtual(&sess.lanes[li], brush)
    } else {
        (d.arc_len - sess.lanes[li].arco).max(0.0)
    };
    LaneDoDab {
        li,
        lane_prev,
        caminho,
    }
}

/// O peso do pigmento do dab e os passos do pente do relevo, ambos por COMPRIMENTO de caminho.
pub(super) fn pesos(l: &LoteDoDab, sess: &WetSession, lane: &LaneDoDab, d: &Dab) -> (f32, f64) {
    let &LoteDoDab {
        ref brush,
        ratio,
        passo_do_pigmento,
        capsulas,
        ..
    } = l;
    let (lane_prev, caminho) = (lane.lane_prev, lane.caminho);
    // Sem Pigment Spacing, a cápsula pesa cada sub-dab pelo sub-passo NATURAL (antes do arredondamento a células):
    // a tinta por área não depende de onde o diâmetro cai na grelha ([`super::super::lote::sub_passo_natural`]).
    let peso_da_capsula = if capsulas && passo_do_pigmento <= 0.0 && lane_prev.is_some() {
        let w = caminho / super::super::lote::sub_passo_do_peso(&sess.engine.tuning, brush, ratio);
        if (w - 1.0).abs() < 1e-3 { 1.0 } else { w }
    } else {
        1.0
    };
    let peso = lane_prev
        .filter(|_| passo_do_pigmento > 0.0)
        .map_or(peso_da_capsula, |_| {
            let w = caminho / brush.passo_ao_longo_px(passo_do_pigmento, d.dir);
            // O passo do PRÓPRIO Spacing de afinação é o dab inteiro, ao byte (a corda sai dos centros em f32).
            if (w - 1.0).abs() < 1e-3 {
                1.0
            } else if capsulas {
                w
            } else {
                w.min(1.0)
            }
        });
    // O pente do relevo também é por comprimento: as células de caminho desde o dab anterior da lane (a
    // borracha, sem lanes, pelo sub-passo da cápsula).
    let passos_do_pente = lane_prev.map_or(1.0, |_| {
        let n = f64::from(caminho) / f64::from(ratio);
        if (n - 1.0).abs() < 1e-3 { 1.0 } else { n }
    });
    (peso, passos_do_pente)
}
