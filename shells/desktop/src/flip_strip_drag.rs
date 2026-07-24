//! O drain dos **pedidos do arrasto** da tira (o lado do shell).
//!
//! O painel (`ph2d-panel-flip-frames`) interpreta o gesto e enfileira um
//! [`FlipStripIntent`]; aqui ele vira documento. Módulo irmão do `flip_strip` por
//! responsabilidade — lá mora o drain do `PanelEvent` (clique de botão), aqui o do arrasto,
//! e o arquivo de lá está no teto de LOC.
//!
//! **Nenhuma operação nova:** os dois pedidos caem em `FlipObject::move_frame` e
//! `set_exposure`, exatamente as que os botões `◀`/`▶` e a caixa **Hold** chamam. Se um dia
//! a semântica de mover uma chave mudar, ela muda num lugar só.

use ph2d_flip::{FlipDoc, LayerId};

/// Aplica os pedidos que o arrasto da tira enfileirou neste frame.
///
/// Devolve `true` se o documento MUDOU — o chamador marca a edição, e é isso que faz o
/// gesto virar **um** passo na fila global de undo (o painel só enfileira no `End`, então
/// um arrasto inteiro chega aqui como um pedido só).
pub(crate) fn apply_strip_intents(
    flip: &mut FlipDoc,
    active_layer: Option<LayerId>,
    strip: &mut crate::flip_strip::FlipStrip,
) -> bool {
    #[cfg(not(feature = "panel-flip-frames"))]
    {
        let _ = (flip, active_layer, strip);
        false
    }
    #[cfg(feature = "panel-flip-frames")]
    {
        use ph2d_panel_flip_frames::FlipStripIntent;
        let intents = ph2d_panel_flip_frames::drain_flip_strip_intents();
        if intents.is_empty() {
            return false;
        }
        // O alvo é resolvido UMA vez: o objeto/camada não muda no meio de um drain, e
        // re-resolvê-lo por pedido só daria a chance de discordar de si mesmo.
        let Some(obj) = flip.objects().first().map(|o| o.id) else {
            return false;
        };
        let Some(lid) = active_layer
            .filter(|id| {
                flip.object(obj)
                    .is_some_and(|o: &ph2d_flip::FlipObject| o.layer(*id).is_some())
            })
            .or_else(|| {
                flip.object(obj)
                    .and_then(|o| o.layers().last().map(|l| l.id))
            })
        else {
            return false;
        };
        let Some(o) = flip.object_mut(obj) else {
            return false;
        };
        let mut changed = false;
        for intent in intents {
            changed |= match intent {
                // `move_frame` devolve `false` num destino ocupado. O painel já ENCOSTA a
                // chave na vizinha (nunca pede um destino ocupado), então um `false` aqui
                // significa que a tira e o documento discordaram — e o certo é não mudar
                // nada, que é o que acontece.
                FlipStripIntent::MoveKey { from, to } => {
                    let ok = o.move_frame(lid, from, to);
                    // ⚠️ O light table acompanha, e **só quando a chave de fato andou**:
                    // remapear sobre um movimento recusado moveria o pin para um quadro
                    // onde não há chave nenhuma.
                    if ok {
                        strip.remap_pin_after_move(from, to);
                    }
                    ok
                }
                FlipStripIntent::SetHold { key, frames } => {
                    // O delta tem de ser lido ANTES: depois da escrita, a exposição já é a
                    // nova e a diferença some (o clássico seed-versus-sample).
                    let before = o.layer(lid).map_or(0, |l| l.duration_at(key));
                    let ok = o.set_exposure(lid, key, frames);
                    if ok && before > 0 {
                        strip.remap_pins_after_hold(key, frames as i32 - before as i32);
                    }
                    ok
                }
            };
        }
        changed
    }
}

#[cfg(all(test, feature = "panel-flip-frames"))]
mod tests {
    use super::*;
    use ph2d_flip::{Hold, KeyKind};
    use ph2d_panel_flip_frames::FlipStripIntent;

    /// Um objeto com chaves em 0, 4 e 8 — a fixture da tira.
    fn doc() -> (FlipDoc, LayerId) {
        let mut flip = FlipDoc::default();
        let oid = flip.push_object("Flip");
        let obj = flip.object_mut(oid).expect("objeto");
        let lid = obj.add_layer("Layer 1");
        for key in [0, 4, 8] {
            obj.insert_frame(lid, key, Hold::Implicit, KeyKind::Keyframe);
        }
        (flip, lid)
    }

    fn keys(flip: &FlipDoc, lid: LayerId) -> Vec<i32> {
        flip.objects()[0]
            .layer(lid)
            .expect("camada")
            .cells()
            .iter()
            .map(|(k, _, _)| *k)
            .collect()
    }

    /// 🔴 O pedido do arrasto chega ao documento: a chave anda.
    #[test]
    fn a_move_intent_moves_the_key() {
        let (mut flip, lid) = doc();
        let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();
        ph2d_panel_flip_frames::push_intent_for_tests(FlipStripIntent::MoveKey { from: 4, to: 6 });
        assert!(apply_strip_intents(
            &mut flip,
            Some(lid),
            &mut crate::flip_strip::FlipStrip::default()
        ));
        assert_eq!(keys(&flip, lid), vec![0, 6, 8]);
    }

    /// 🔴 E o da borda muda a EXPOSIÇÃO — que **empurra** as chaves seguintes (a semântica
    /// da tira de exposição, já do modelo). Sem o empurrão, esticar uma chave comeria a
    /// vizinha.
    #[test]
    fn a_hold_intent_pushes_the_following_keys() {
        let (mut flip, lid) = doc();
        let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();
        ph2d_panel_flip_frames::push_intent_for_tests(FlipStripIntent::SetHold {
            key: 0,
            frames: 6,
        });
        assert!(apply_strip_intents(
            &mut flip,
            Some(lid),
            &mut crate::flip_strip::FlipStrip::default()
        ));
        assert_eq!(
            keys(&flip, lid),
            vec![0, 6, 10],
            "a de 4 foi para 6 e a de 8 a acompanhou"
        );
    }

    /// 🔴 **O pin acompanha a chave que o arrasto moveu.**
    ///
    /// Achado da auditoria: as duas features desta wave se quebravam mutuamente. O artista
    /// fixa o quadro 8, arrasta AQUELA célula para o 9, e o fantasma sumia — o pin ficava
    /// apontando um quadro sem chave, e nada na tela dizia por quê.
    #[test]
    fn a_moved_key_carries_its_pin_along() {
        let (mut flip, lid) = doc();
        let mut strip = crate::flip_strip::FlipStrip::default();
        strip.toggle_pin(8);
        let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();
        ph2d_panel_flip_frames::push_intent_for_tests(FlipStripIntent::MoveKey { from: 8, to: 9 });
        assert!(apply_strip_intents(&mut flip, Some(lid), &mut strip));
        assert_eq!(strip.pinned_keys(), &[9], "a referência foi junto");
    }

    /// 🔴 **E acompanha o EMPURRÃO da exposição** — o caso comum, não o exótico: esticar a
    /// primeira chave empurra a fila inteira, então sem isto UM gesto orfanaria todos os
    /// pins à direita de uma vez.
    #[test]
    fn stretching_a_hold_pushes_the_pins_that_the_keys_push() {
        let (mut flip, lid) = doc();
        let mut strip = crate::flip_strip::FlipStrip::default();
        strip.toggle_pin(8); // a última chave
        let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();
        // A chave 0 expunha 4 (0→4); passa a expor 6 ⇒ delta +2 ⇒ 4→6 e 8→10.
        ph2d_panel_flip_frames::push_intent_for_tests(FlipStripIntent::SetHold {
            key: 0,
            frames: 6,
        });
        assert!(apply_strip_intents(&mut flip, Some(lid), &mut strip));
        assert_eq!(keys(&flip, lid), vec![0, 6, 10]);
        assert_eq!(
            strip.pinned_keys(),
            &[10],
            "o pin seguiu a chave que o empurrão moveu"
        );
    }

    /// E um movimento RECUSADO (destino ocupado) não move o pin: remapear ali o mandaria
    /// para um quadro onde não há chave nenhuma.
    #[test]
    fn a_refused_move_leaves_the_pin_where_it_is() {
        let (mut flip, lid) = doc();
        let mut strip = crate::flip_strip::FlipStrip::default();
        strip.toggle_pin(4);
        let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();
        // 4 → 8 é ocupado (a última chave está lá): `move_frame` recusa.
        ph2d_panel_flip_frames::push_intent_for_tests(FlipStripIntent::MoveKey { from: 4, to: 8 });
        assert!(!apply_strip_intents(&mut flip, Some(lid), &mut strip));
        assert_eq!(strip.pinned_keys(), &[4]);
    }

    /// Sem pedidos o documento não é tocado — e o `false` é o que impede um frame ocioso
    /// de virar um passo de undo.
    #[test]
    fn an_empty_drain_changes_nothing() {
        let (mut flip, lid) = doc();
        let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();
        assert!(!apply_strip_intents(
            &mut flip,
            Some(lid),
            &mut crate::flip_strip::FlipStrip::default()
        ));
        assert_eq!(keys(&flip, lid), vec![0, 4, 8]);
    }
}
