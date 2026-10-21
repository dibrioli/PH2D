//! Os gates da [`super`] — a família FLIP: o funil do painel, e a curva que chega ao documento
//! sem chegar à fotografia.

use super::*;

/// Um documento com um objecto de duas camadas (`a` em baixo, `b` em cima) e a entidade dele.
fn cena() -> (SimWorld, FlipDoc, u64, FlipObjectId, LayerId, LayerId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Flip");
    let obj = doc.object_mut(oid).expect("objecto");
    let a = obj.add_layer("A");
    let b = obj.add_layer("B");
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(FlipObjectRef(oid.0)).id().to_bits();
    (sim, doc, e, oid, a, b)
}

fn camada(doc: &FlipDoc, oid: FlipObjectId, l: LayerId) -> &FlipLayer {
    doc.object(oid).and_then(|o| o.layer(l)).expect("camada")
}

/// Todo campo lê de volta o que se escreveu, pelo funil do painel.
#[test]
fn cada_campo_le_o_que_escreveu() {
    let (_, mut doc, _, oid, _, b) = cena();
    for (campo, alvo) in [
        (f::VISIVEL, 0.0),
        (f::OPACIDADE, 0.25),
        (f::MISTURA, 3.0),
        (f::PROFUNDIDADE, 0.4),
    ] {
        let kk = chave(campo, b).expect("cabe");
        let (_, ficou) = escrever(&mut doc, oid, kk, alvo).expect("campo");
        assert_eq!(ficou, alvo, "campo {campo}");
        assert_eq!(ler(camada(&doc, oid, b), campo), Some(alvo));
    }
}

/// As escolhas arredondam e prendem-se à tabela; os números passam pelo `clamp` do painel; o olho
/// que já está como a curva manda não é clicado (o interruptor inverteria).
#[test]
fn a_escrita_segue_o_dominio_do_painel() {
    let (_, mut doc, _, oid, a, _) = cena();
    let k = |c| chave(c, a).expect("cabe");
    assert_eq!(
        escrever(&mut doc, oid, k(f::OPACIDADE), 1.7),
        Some((1.0, 1.0))
    );
    let max = f32::from(ph2d_painter_effects::MAX_BLEND_MODES - 1);
    assert_eq!(
        escrever(&mut doc, oid, k(f::MISTURA), 99.0).map(|x| x.1),
        Some(max)
    );
    assert_eq!(
        escrever(&mut doc, oid, k(f::VISIVEL), 0.7),
        Some((1.0, 1.0))
    );
    assert_eq!(
        escrever(&mut doc, oid, k(f::VISIVEL), 0.3),
        Some((1.0, 0.0))
    );
    assert!(edicao(camada(&doc, oid, a), f::VISIVEL, 0.0).is_none());
    assert_eq!(chave(f::OPACIDADE, LayerId(70_000)), None, "o i é u16");
}

/// ⭐ **O MESMO funil que o painel**: a opacidade e a mistura escritas pelo canal dão a camada que
/// o painel dá ao emitir o evento da linha.
#[test]
fn o_canal_e_a_edicao_do_painel_dao_a_mesma_camada() {
    let (_, mut pelo_canal, _, oid, _, b) = cena();
    let mut pelo_painel = pelo_canal.clone();
    escrever(&mut pelo_canal, oid, chave(f::OPACIDADE, b).unwrap(), 0.4);
    escrever(&mut pelo_canal, oid, chave(f::MISTURA, b).unwrap(), 1.0);
    let id = |k| flip_layer_widget_id(u64::from(b.0), k);
    let mut alvo = FlipTarget::on(oid, None);
    for ev in [
        PanelEvent::SetValue(id(FlipLayerWidget::Opacity), 0.4),
        PanelEvent::SelectOption(id(FlipLayerWidget::Blend), "1".to_owned()),
    ] {
        crate::layers::apply_panel_event(
            &ev,
            &mut pelo_painel,
            &mut alvo,
            &ph2d_core::Playhead::default(),
            false,
        );
    }
    assert_eq!(pelo_canal, pelo_painel);
}

/// ⭐⭐ **Ponta-a-ponta**: a curva escreve em `CanaisEscritos` → `aplicar` muda a camada pelo
/// funil, o eco fica, `publicar` lê o novo valor — e a fotografia (`substituir`) vê o AUTORADO,
/// devolvendo o vivo depois (`repor`).
#[test]
fn a_curva_chega_a_camada_e_nao_a_fotografia() {
    let (mut sim, mut doc, e, oid, a, b) = cena();
    let op = chave(f::OPACIDADE, b).unwrap();
    let vis = chave(f::VISIVEL, a).unwrap();
    {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        w.escrever(e, op, 0.25);
        w.escrever(e, vis, 0.0);
    }
    let mut conducao = ConducaoFlip::default();
    assert!(aplicar(&mut sim, &mut doc, &mut conducao));
    assert_eq!(camada(&doc, oid, b).opacity, 0.25);
    assert!(!camada(&doc, oid, a).visible);
    publicar(&mut sim, &doc, &[e]);
    let l = sim.world().resource::<CanaisLidos>();
    assert_eq!(l.valor(e, op), Some(0.25));
    assert_eq!(l.campos(e).count(), 2 * f::CAMPOS.len());

    let vivo = conducao.substituir(&sim, &mut doc);
    assert_eq!(
        camada(&doc, oid, b).opacity,
        1.0,
        "a fotografia vê o autorado"
    );
    assert!(camada(&doc, oid, a).visible);
    conducao.repor(&sim, &mut doc, vivo);
    assert_eq!(camada(&doc, oid, b).opacity, 0.25, "e o vivo volta");
    assert!(!camada(&doc, oid, a).visible);

    // Parado, a curva já não muda nada: o valor dela passa a ser documento.
    conducao.settle();
    conducao.settle();
    assert!(conducao.is_empty());
}

/// A camada apagada deixa de publicar; uma entidade sem objecto Flip não publica nada.
#[test]
fn so_publica_o_que_existe() {
    let (mut sim, mut doc, e, oid, a, _) = cena();
    publicar(&mut sim, &doc, &[e]);
    assert_eq!(
        sim.world().resource::<CanaisLidos>().campos(e).count(),
        2 * f::CAMPOS.len()
    );
    assert!(doc.object_mut(oid).unwrap().remove_layer(a));
    publicar(&mut sim, &doc, &[e]);
    assert_eq!(
        sim.world().resource::<CanaisLidos>().campos(e).count(),
        f::CAMPOS.len()
    );
    let solta = sim.world_mut().spawn(()).id().to_bits();
    publicar(&mut sim, &doc, &[solta]);
    assert_eq!(
        sim.world().resource::<CanaisLidos>().campos(solta).count(),
        0
    );
}
