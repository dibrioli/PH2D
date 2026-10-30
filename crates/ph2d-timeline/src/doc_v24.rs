//! ⭐ **A migração v24 → v25** — a caixa do cartão (`View::Toggle`) passa a `0` = desligada,
//! `1` = ligada (`ph2d_ecs::canal_componente::e_caixa`). Até à v24 o índice era a ordem da
//! declaração (`Toggle(ligado, desligado)`), então cada chave, `default` e `rest` de uma trilha de
//! caixa vira `1 − v`: o estado da caixa em cada chave fica o mesmo. A interpolação não muda.
//! Gate: `tests/it/caixa_ordem.rs` contra `tests/fixtures/timeline_doc_v24.bin`.

use super::*;

impl TimelineDoc {
    pub(super) fn desvira_caixas(&mut self) {
        let vira = |v: f32| 1.0 - v;
        let alvos: Vec<AnimTarget> = (self.bindings.iter())
            .filter(
                |b| matches!(b.prop, PropKind::Field(k) if ph2d_ecs::canal_componente::e_caixa(k)),
            )
            .map(|b| b.target)
            .collect();
        for c in &mut self.clips {
            for &alvo in &alvos {
                let Some(track) = c.clip.track_mut(alvo) else {
                    continue;
                };
                for id in track.ids().to_vec() {
                    if let Some(AnimValue::Float(v)) = track.key(id).map(|k| k.value) {
                        track.set_value(id, AnimValue::Float(vira(v)));
                    }
                }
                if let AnimValue::Float(d) = track.default_value() {
                    let t = std::mem::replace(track, Track::new(Vec::new()));
                    *track = t.with_default(AnimValue::Float(vira(d)));
                }
            }
        }
        for b in (self.bindings.iter_mut()).filter(|b| alvos.contains(&b.target)) {
            b.rest = b.rest.map(vira);
        }
    }
}
