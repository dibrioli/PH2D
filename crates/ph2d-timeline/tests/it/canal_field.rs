//! ⭐⭐⭐ **O canal de qualquer propriedade** (`PropKind::Field`, plano 35 §3.0): a timeline anima um
//! campo de família sem o conhecer — publica o valor da curva em `CanaisEscritos` — e um campo
//! DISCRETO segura até a chave seguinte e SUBSTITUI na pilha (ADR-0115 R7), nunca inventa um terceiro.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::canal::familia;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::{CanaisEscritos, FieldKey, Transform, World};
use ph2d_timeline::{ClipLane, ClipStrip, PropKind, StripSource, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// A cor vermelha do preenchimento — contínua.
const VERMELHO: FieldKey = FieldKey::new(familia::VETOR, v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0);
/// O tipo da tinta (sólida · linear · radial …) — discreto.
const TIPO: FieldKey = FieldKey::new(familia::VETOR, v::TINTA_PREENCH + v::OFF_TIPO, 0, 0);

fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    (world, TimelineDoc::new(), e)
}

/// O que a curva mandou escrever neste quadro para `(e, k)` — e esvazia o quadro.
fn escrito(world: &mut World, e: u64, k: FieldKey) -> Option<f32> {
    let mut w = world.get_resource_mut::<CanaisEscritos>()?;
    w.drenar(k.familia)
        .into_iter()
        .rev()
        .find(|&(ent, kk, _)| ent == e && kk == k)
        .map(|(_, _, x)| x)
}

fn flat(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, x: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(e, prop, s(0.0), AnimValue::Float(x), Interp::Linear);
    doc.insert_key(e, prop, s(2.0), AnimValue::Float(x), Interp::Linear);
    doc.set_active(was);
}

/// Dois clipes que se cruzam de 1 s a 2 s, cada um com o campo num valor plano.
fn crossfade(prop: PropKind, a: f32, b: f32) -> (World, TimelineDoc, u64) {
    let (world, mut doc, e) = scene();
    doc.add_clip("B".to_string());
    flat(&mut doc, 0, e, prop, a);
    flat(&mut doc, 1, e, prop, b);
    let mut l = ClipLane::new("Base");
    l.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    l.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(l);
    (world, doc, e)
}

/// A curva de um campo chega à família pelo `CanaisEscritos`, no valor da curva.
#[test]
fn a_curva_de_um_campo_e_publicada_para_a_familia() {
    let (mut world, mut doc, e) = scene();
    let p = PropKind::Field(VERMELHO);
    doc.insert_key(e, p, s(0.0), AnimValue::Float(255.0), Interp::Linear);
    doc.insert_key(e, p, s(2.0), AnimValue::Float(0.0), Interp::Linear);
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert_eq!(escrito(&mut world, e, VERMELHO), Some(127.5));
}

/// Contínuo: o meio do crossfade é a MÉDIA (o mesmo gate R7 dos 19 de sempre).
#[test]
fn um_campo_continuo_cruza_na_media() {
    let (mut world, mut doc, e) = crossfade(PropKind::Field(VERMELHO), 1.0, 3.0);
    apply_from_doc(&mut world, &mut doc, 1.5);
    let got = escrito(&mut world, e, VERMELHO).expect("a pilha calou o campo");
    assert!(
        (got - 2.0).abs() < 1e-4,
        "o meio do crossfade é {got}, não a média 2"
    );
}

/// ⛔ ADR-0115 R7: o discreto NÃO se mistura. De 0 (sólida) para 2 (radial), a média seria 1
/// (linear) — um terceiro que ninguém pôs. Ele substitui: o clipe de maior peso dá o valor.
#[test]
fn um_campo_discreto_substitui_na_pilha_e_nunca_inventa_um_terceiro() {
    let (mut world, mut doc, e) = crossfade(PropKind::Field(TIPO), 0.0, 2.0);
    for (t, quer) in [(1.1, 0.0), (1.4, 0.0), (1.6, 2.0), (1.9, 2.0)] {
        apply_from_doc(&mut world, &mut doc, t);
        let got = escrito(&mut world, e, TIPO).expect("a pilha calou o campo");
        assert_eq!(got, quer, "em t={t} o discreto vale {got}");
    }
}

/// O discreto é `Hold` mesmo quando quem grava pede `Linear` (a porta `interp_of`), e é função
/// PURA do tempo: voltar atrás devolve a chave anterior (o defeito do seek reverso do Godot,
/// `docs/UI_New_and_Simple/vetor_nova_ui/oraculos/godot_interp.saida.txt`).
#[test]
fn o_discreto_segura_ate_a_chave_seguinte_e_e_funcao_do_tempo() {
    let (mut world, mut doc, e) = scene();
    let p = PropKind::Field(TIPO);
    doc.insert_key(e, p, s(0.0), AnimValue::Float(0.0), Interp::Linear);
    doc.insert_key(e, p, s(1.0), AnimValue::Float(2.0), Interp::Linear);
    for (t, quer) in [
        (0.9, 0.0),
        (1.0, 2.0),
        (0.5, 0.0),
        (0.99, 0.0),
        (1.5, 2.0),
        (0.0, 0.0),
    ] {
        apply_from_doc(&mut world, &mut doc, t);
        assert_eq!(escrito(&mut world, e, TIPO), Some(quer), "em t={t}");
    }
}

/// Dois campos do mesmo objecto são canais DISTINTOS (o `FieldKey` entra na identidade do binding).
#[test]
fn dois_campos_do_mesmo_objecto_sao_dois_canais() {
    let (_world, mut doc, e) = scene();
    let a = doc.bind(e, PropKind::Field(VERMELHO));
    let b = doc.bind(e, PropKind::Field(TIPO));
    assert_ne!(a, b);
    assert_eq!(
        doc.bind(e, PropKind::Field(VERMELHO)),
        a,
        "idempotente por campo"
    );
    let outro_contorno = FieldKey::new(familia::VETOR, v::ANCORA_X, 1, 0);
    let mesmo_vertice = FieldKey::new(familia::VETOR, v::ANCORA_X, 0, 0);
    assert_ne!(
        doc.bind(e, PropKind::Field(outro_contorno)),
        doc.bind(e, PropKind::Field(mesmo_vertice))
    );
}

/// Um documento com campos sobrevive à ida e volta pelo formato gravado.
#[test]
fn um_documento_com_campos_vai_e_volta_pelo_formato() {
    let (_world, mut doc, e) = scene();
    doc.insert_key(
        e,
        PropKind::Field(VERMELHO),
        s(0.5),
        AnimValue::Float(9.0),
        Interp::Linear,
    );
    doc.insert_key(
        e,
        PropKind::Field(TIPO),
        s(0.5),
        AnimValue::Float(3.0),
        Interp::Linear,
    );
    let bytes = doc.to_bytes().unwrap();
    let back = TimelineDoc::from_bytes(&bytes).unwrap();
    assert_eq!(back.bindings().len(), 2);
    assert!(
        back.bindings()
            .iter()
            .any(|b| b.prop == PropKind::Field(TIPO))
    );
}

/// ⭐ v20 → v21 sem perda: o documento migrado é o v20 campo a campo, menos a versão.
#[test]
fn um_ficheiro_v20_abre_sem_perda() {
    const V20_BIN: &[u8] = include_bytes!("../fixtures/timeline_doc_v20.bin");
    const V20_JSON: &str = include_str!("../fixtures/timeline_doc_v20.json");
    let doc = TimelineDoc::from_bytes(V20_BIN).expect("o v20 tem de abrir");
    assert_eq!(doc.version, ph2d_timeline::DOC_VERSION);
    let sem_versao = |mut j: serde_json::Value| {
        j.as_object_mut().unwrap().remove("version");
        j
    };
    let antes: serde_json::Value = serde_json::from_str(V20_JSON).unwrap();
    assert_eq!(
        sem_versao(serde_json::to_value(&doc).unwrap()),
        sem_versao(antes)
    );
    assert!(
        doc.clips()[0].autoplay,
        "o que só a v20 tinha atravessa a migração"
    );
    assert_eq!(
        TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap(),
        doc
    );
}
