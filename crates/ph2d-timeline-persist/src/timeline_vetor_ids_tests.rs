//! ⭐ F31 §A: o nome da linha de uma chave de item do Vetor diz a POSIÇÃO, pelo espelho
//! [`IdsDoVetor`].

use super::*;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{FieldKey, ItemId, SimWorld};
use ph2d_timeline::PropKind;

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
