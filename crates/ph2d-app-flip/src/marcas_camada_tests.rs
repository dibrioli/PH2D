//! Gates do [`super`] (`marcas_camada.rs`) — o losango das camadas do Flip.

use super::*;
use crate::canais_camada::chave;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_flip::{FlipDoc, LayerId};

const E: u64 = 7;

fn em(t: f64) -> Instante {
    Instante {
        t_cut: t,
        solo: true,
    }
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(0.5)
}

fn add(e: u64, p: PropKind) -> Option<TimelineIntent> {
    Some(TimelineIntent::AddKey {
        entity: e,
        prop: p,
        t: RationalTime::from_seconds(0.5),
        value: AnimValue::Float(0.5),
        interp: Interp::Linear,
    })
}

fn objeto() -> (FlipDoc, LayerId, LayerId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Flip");
    let o = doc.object_mut(oid).unwrap();
    let a = o.add_layer("A");
    let b = o.add_layer("B");
    (doc, a, b)
}

fn id(l: LayerId, k: FlipLayerWidget) -> NodeId {
    flip_layer_widget_id(u64::from(l.0), k)
}

/// ⭐ **A tabela id → grupo**: o slider e o chip partilham o canal; o olho e a mistura têm o
/// seu; o cadeado, o nome e as setas não têm canal (plano 35 §8).
#[test]
fn cada_controlo_da_camada_mostra_o_seu_canal() {
    let (doc, a, b) = objeto();
    let obj = &doc.objects()[0];
    let canal = |l, k| canal_do_campo(obj, id(l, k));
    assert_eq!(
        canal(b, FlipLayerWidget::OpacityNum),
        chave(f::OPACIDADE, b)
    );
    assert_eq!(canal(b, FlipLayerWidget::Opacity), chave(f::OPACIDADE, b));
    assert_eq!(canal(a, FlipLayerWidget::Depth), chave(f::PROFUNDIDADE, a));
    assert_eq!(canal(a, FlipLayerWidget::Visibility), chave(f::VISIVEL, a));
    assert_eq!(canal(a, FlipLayerWidget::Blend), chave(f::MISTURA, a));
    for k in [
        FlipLayerWidget::Lock,
        FlipLayerWidget::Row,
        FlipLayerWidget::MoveUp,
        FlipLayerWidget::MoveDown,
    ] {
        assert_eq!(canal(a, k), None, "{k:?} não é propriedade da cena");
    }
    let todos: usize = CAMPOS.iter().map(|(_, k)| k.len()).sum();
    assert_eq!(todos, 6);
}

/// O mapa marca cada controlo de cada camada; a chave da camada `b` não acende a da `a`.
#[test]
fn o_mapa_e_por_camada() {
    let (doc, a, b) = objeto();
    let obj = &doc.objects()[0];
    let mut tl = TimelineDoc::new();
    tl.fps_display = 24.0;
    let p = PropKind::Field(chave(f::OPACIDADE, b).unwrap());
    tl.insert_key(
        E,
        p,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.5),
        Interp::Linear,
    );
    let m = marcas(&tl, E, obj, em(0.0), na_curva);
    assert_eq!(m.len(), 2 * 6);
    assert_eq!(m[&id(b, FlipLayerWidget::Opacity)], KeyMark::Chave);
    assert_eq!(m[&id(b, FlipLayerWidget::OpacityNum)], KeyMark::Chave);
    assert_eq!(m[&id(a, FlipLayerWidget::Opacity)], KeyMark::Livre);
    assert_eq!(m[&id(b, FlipLayerWidget::Depth)], KeyMark::Livre);
}

/// O clique num losango livre grava o campo daquela camada pela porta da K; com chave neste
/// quadro, apaga-a. Fora da tabela, nada.
#[test]
fn o_clique_grava_o_campo_da_camada() {
    let (doc, a, _) = objeto();
    let obj = &doc.objects()[0];
    let kk = chave(f::PROFUNDIDADE, a).unwrap();
    let lidos = |_| vec![kk];
    let tl = TimelineDoc::new();
    let i = clique(
        &tl,
        &[
            id(a, FlipLayerWidget::Depth),
            id(a, FlipLayerWidget::DepthNum),
        ],
        E,
        obj,
        em(0.5),
        lidos,
        na_curva,
        add,
    );
    let gravados: Vec<PropKind> = i
        .iter()
        .filter_map(|x| match x {
            TimelineIntent::AddKey { prop, .. } => Some(*prop),
            _ => None,
        })
        .collect();
    assert_eq!(gravados, vec![PropKind::Field(kk)]);
    assert!(
        clique(
            &tl,
            &[id(a, FlipLayerWidget::Lock)],
            E,
            obj,
            em(0.5),
            lidos,
            na_curva,
            add
        )
        .is_empty()
    );
}
