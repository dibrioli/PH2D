//! ⭐ **A instância acompanha a peça** (smoke do dono 11/11, doc 17 item 10): o comprimento de um
//! container DERIVA do conteúdo (regra do dono 2026-07-21), então uma instância que mostrava a peça
//! INTEIRA continua a mostrá-la quando a peça cresce ou encolhe. Sem isto a moldura laranja ficava
//! no comprimento velho e cortava a fita que se arrastou para lá do fim.
//!
//! A lei, num sítio só ([`crate::intent_apply::edit`] mede antes e chama [`acompanha`] depois):
//! - só a instância cuja fatia era a peça toda (`src_in ≈ 0`, `src_out ≈ antigo`) — a recortada fica;
//! - `src_out = novo`; em `Once` o span segue à MESMA velocidade, em Loop/PingPong só a fatia muda
//!   (o span fica e as voltas re-derivam);
//! - um container com `length_override` não muda de comprimento, não entra;
//! - em todas as pilhas (a da cena e a de cada container), até ao ponto fixo: um container que
//!   cresce dentro de outro faz crescer esse outro.

use crate::StripLoop;
use crate::doc::TimelineDoc;

const EPS: f64 = 1e-6;

/// O comprimento de cada container ANTES da edição — `None` = tem `length_override` (não acompanha).
pub(crate) fn comprimentos(doc: &TimelineDoc) -> Vec<Option<f64>> {
    (0..doc.containers().len())
        .map(|c| {
            doc.containers()[c]
                .length_override
                .is_none()
                .then(|| doc.container_length_seconds(c))
        })
        .collect()
}

/// Leva cada instância inteira de um container que mudou de comprimento ao comprimento novo.
pub(crate) fn acompanha(doc: &mut TimelineDoc, antes: &[Option<f64>]) {
    // Cada volta só pode mudar um container a mais (o hospedeiro); `n + 1` voltas chegam ao ponto fixo.
    for _ in 0..=antes.len() {
        let mudou: Vec<(u16, f64, f64)> = antes
            .iter()
            .enumerate()
            .filter_map(|(c, a)| {
                let old = (*a)?;
                let new = doc.container_length_seconds(c);
                let livre = doc.containers().get(c)?.length_override.is_none();
                let ix = u16::try_from(c).ok()?;
                (livre && (new - old).abs() > EPS).then_some((ix, old, new))
            })
            .collect();
        let mut algum = false;
        let n = doc.containers().len();
        let mut segue = |lanes: &mut Vec<crate::ClipLane>| {
            for s in lanes.iter_mut().flat_map(|l| l.strips.iter_mut()) {
                let Some(&(_, old, new)) = mudou
                    .iter()
                    .find(|m| s.source.container_index() == Some(m.0))
                else {
                    continue;
                };
                if s.src_in.abs() > EPS || (s.src_out - old).abs() > EPS {
                    continue; // recortada (ou já no novo): fica
                }
                s.src_out = new;
                if s.loop_mode == StripLoop::Once && s.speed > 0.0 {
                    s.t_end = s.t_start + new / s.speed;
                }
                algum = true;
            }
        };
        segue(doc.stack_mut());
        for c in 0..n {
            if let Some(lanes) = doc.container_stack_mut(c) {
                segue(lanes);
            }
        }
        if !algum {
            return;
        }
    }
}
