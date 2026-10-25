//! ⭐ F31 §A: a chave posicional do Vetor (v23) passa ao id no `upkeep`, pelo espelho
//! [`IdsDoVetor`], e o nome da linha continua a dizer a POSIÇÃO.

use super::*;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{FieldKey, ItemId, SimWorld};
use ph2d_timeline::PropKind;

const V23_BIN: &[u8] = include_bytes!("../../ph2d-timeline/tests/fixtures/timeline_doc_v23.bin");
const VERMELHO: u16 = v::TINTA_PREENCH + v::OFF_SOLIDA;

fn ids(tintas: &[u32], efeitos: &[u32], filtros: &[u32]) -> IdsDaForma {
    let f = |x: &[u32]| x.iter().map(|&n| ItemId(n)).collect();
    IdsDaForma {
        tintas: f(tintas),
        efeitos: f(efeitos),
        filtros: f(filtros),
    }
}

/// O dono da fixtura (`wire_id` 44), vivo.
fn mundo() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Name::new("forma"),
            StableId(44),
        ))
        .id()
        .to_bits();
    (sim, e)
}

fn chaves(tl: &TimelineState) -> Vec<FieldKey> {
    tl.doc
        .bindings()
        .iter()
        .filter_map(|b| match b.prop {
            PropKind::Field(k) => Some(k),
            _ => None,
        })
        .collect()
}

/// As duas chaves (valores em 0 s e 1 s) da trilha de `k`.
fn valores(tl: &TimelineState, k: FieldKey) -> Vec<ph2d_anim::AnimValue> {
    let b = tl
        .doc
        .bindings()
        .iter()
        .find(|b| b.prop == PropKind::Field(k));
    let t = tl
        .doc
        .active_clip()
        .track(b.expect("ligada").target)
        .expect("trilha");
    t.keys().iter().map(|k| k.value).collect()
}

/// ⭐ O v23 lê-se, cura-se no dono e — com o espelho — cada chave de entrada/efeito/filtro passa ao
/// id da posição que endereçava, com a MESMA trilha; a base, o contorno e o índice fora da lista
/// ficam. (Mutação: a migração mantém a posição ⇒ as chaves de item não aparecem.)
#[test]
fn o_doc_v23_le_e_migra_para_o_id() {
    let (mut sim, e) = mundo();
    let mut tl = install_from_project(V23_BIN).expect("o v23 abre");
    // Sem espelho: espera.
    assert!(!upkeep(&mut tl, sim.world_mut()));
    assert!(chaves(&tl).iter().all(|k| k.item.is_none()));
    let f = |x: f32| ph2d_anim::AnimValue::Float(x);
    let antes = valores(&tl, FieldKey::new(VETOR, v::EFEITO_PARAM, 1, 2));

    let espelho = IdsDoVetor([(e, ids(&[10, 20], &[30, 31], &[40]))].into());
    sim.world_mut().insert_resource(espelho);
    assert!(!upkeep(&mut tl, sim.world_mut()));
    let ks = chaves(&tl);
    let item = |campo, id, j| FieldKey::do_item(VETOR, campo, ItemId(id), j);
    for esperada in [
        item(VERMELHO, 10, 0),
        item(v::PILHA_OPACIDADE, 20, 0),
        item(v::EFEITO_PARAM, 31, 2),
        item(v::FILTRO_RAIO, 40, 0),
        // Ficam: a base, o contorno, e a tinta do traço da entrada 2 (a lista só tem 2: órfã).
        FieldKey::new(VETOR, VERMELHO, 0, 0),
        FieldKey::new(VETOR, v::ANCORA_X, 1, 3),
        FieldKey::new(VETOR, v::TINTA_TRACO + v::OFF_SOLIDA, 3, 0),
    ] {
        assert!(ks.contains(&esperada), "{esperada:?} em {ks:?}");
    }
    assert_eq!(ks.len(), 7);
    assert_eq!(valores(&tl, item(v::EFEITO_PARAM, 31, 2)), antes);
    assert_eq!(valores(&tl, item(VERMELHO, 10, 0)), [f(10.0), f(20.0)]);
    // Idempotente.
    let doc = tl.doc.clone();
    upkeep(&mut tl, sim.world_mut());
    assert_eq!(tl.doc, doc);
}

/// A chave nova já ligada não FUNDE: a antiga fica como está, e a nova com a trilha dela.
#[test]
fn a_chave_nova_ja_ligada_nao_funde() {
    let (mut sim, e) = mundo();
    let mut tl = TimelineState::new();
    let (antiga, nova) = (
        PropKind::Field(FieldKey::new(VETOR, v::FILTRO_RAIO, 0, 0)),
        PropKind::Field(FieldKey::do_item(VETOR, v::FILTRO_RAIO, ItemId(40), 0)),
    );
    let s = ph2d_anim::RationalTime::from_seconds;
    let f = ph2d_anim::AnimValue::Float;
    tl.doc
        .insert_key(e, antiga, s(0.0), f(1.0), ph2d_anim::Interp::Linear);
    tl.doc
        .insert_key(e, nova, s(0.0), f(2.0), ph2d_anim::Interp::Linear);
    sim.world_mut()
        .insert_resource(IdsDoVetor([(e, ids(&[], &[], &[40]))].into()));
    upkeep(&mut tl, sim.world_mut());
    let props: Vec<_> = tl.doc.bindings().iter().map(|b| b.prop).collect();
    assert_eq!(props, [antiga, nova]);
}

/// ⭐ O nome da linha de uma chave de item é o que o dope-sheet dava com o `i` antigo: a tinta da
/// entrada `k` lê-se `k+1`, o efeito e o filtro `e`/`op` (o 0 sem número). (Mutação: o nome volta a
/// ler o `i` da chave ⇒ «Fill» sem número.)
#[test]
fn o_nome_da_linha_da_entrada_e_a_posicao() {
    let (mut sim, e) = mundo();
    sim.world_mut().insert_resource(IdsDoVetor(
        [(e, ids(&[10, 20], &[30, 31], &[40, 41]))].into(),
    ));
    let track = |k: FieldKey| ph2d_timeline::TrackView {
        target: ph2d_anim::AnimTarget::new(u64::from(k.campo)),
        prop: PropKind::Field(k),
        entity: e,
        missing: false,
        keys: Vec::new(),
        buffer_ghost: None,
        pre: ph2d_anim::Extrap::Hold,
        post: ph2d_anim::Extrap::Hold,
        expr: None,
    };
    let item = |campo, id| FieldKey::do_item(VETOR, campo, ItemId(id), 0);
    let ks = [
        item(VERMELHO, 20),
        item(v::EFEITO_LIGADO, 30),
        item(v::EFEITO_PARAM, 31),
        item(v::FILTRO_RAIO, 41),
        item(v::FILTRO_TIPO, 99), // o espelho não o tem: sem nome
    ];
    let mut view = ph2d_timeline::TimelineViewSnapshot {
        tracks: ks.iter().copied().map(track).collect(),
        ..Default::default()
    };
    publish_object_names(&mut view, sim.world());
    let tr = |k: &str| ph2d_i18n::tr(k).to_owned();
    let nome = |k: FieldKey| view.channel_name(PropKind::Field(k)).map(str::to_owned);
    assert_eq!(
        nome(ks[0]),
        Some(format!("{} 2", tr("canal.vetor.preenchimento")))
    );
    assert_eq!(nome(ks[1]), Some(tr("canal.vetor.efeito_ligado")));
    assert_eq!(nome(ks[2]), Some(format!("{} 1", tr("canal.vetor.efeito"))));
    assert_eq!(nome(ks[3]), Some(format!("{} 1", tr("canal.vetor.filtro"))));
    assert_eq!(nome(ks[4]), None);
}
