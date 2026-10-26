use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{FieldKey, ItemId, Name, SimWorld};
use ph2d_timeline::{PropKind, TimelineState};

const K: FieldKey = FieldKey::do_item(VETOR, v::PILHA_OPACIDADE, ItemId(20), 0);

fn quadros(tl: &mut TimelineState, sim: &mut SimWorld) {
    for _ in 0..3 {
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut tl.doc, 0.5);
        crate::upkeep(tl, sim.world_mut());
    }
}

/// Uma forma com a tinta 20 animada, gravada e reaberta; `tintas` = o espelho depois do load
/// (`None` = a família ainda não publicou a forma).
fn reaberto(tintas: Option<&[u32]>) -> TimelineState {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((ph2d_ecs::Transform::IDENTITY, Name::new("forma"))).id().to_bits();
    let mut tl = TimelineState::new();
    for (t, x) in [(0.0, 1.0), (1.0, 2.0)] {
        tl.doc.insert_key(
            e,
            PropKind::Field(K),
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(x),
            ph2d_anim::Interp::Linear,
        );
    }
    crate::upkeep(&mut tl, sim.world_mut());
    let mut tl = crate::install_from_project(&tl.doc.to_bytes().expect("grava")).expect("lê");
    if let Some(tintas) = tintas {
        let f = IdsDaForma {
            tintas: tintas.iter().map(|&n| ItemId(n)).collect(),
            efeitos: Vec::new(),
            filtros: Vec::new(),
        };
        sim.world_mut().insert_resource(IdsDoVetor([(e, f)].into()));
    }
    quadros(&mut tl, &mut sim);
    tl
}

fn tem(tl: &TimelineState) -> bool {
    tl.doc.bindings().iter().any(|b| b.prop == PropKind::Field(K))
}

#[test]
fn o_item_que_o_ficheiro_trouxe_e_ja_nao_existe_sai_no_load() {
    let tl = reaberto(Some(&[10]));
    assert!(!tem(&tl), "nenhum undo o pode devolver: sai");
    assert!(!tl.itens_por_decidir, "decidido");
}

#[test]
fn o_item_vivo_fica_e_a_decisao_acaba() {
    let tl = reaberto(Some(&[10, 20]));
    assert!(tem(&tl));
    assert!(!tl.itens_por_decidir, "nada fica por decidir: a lei da sessão volta");
}

#[test]
fn sem_espelho_da_forma_espera() {
    let tl = reaberto(None);
    assert!(tem(&tl), "sem a família publicar, não se decide");
    assert!(tl.itens_por_decidir, "e a decisão fica para o quadro em que ela publicar");
}
