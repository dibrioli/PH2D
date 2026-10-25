use super::*;

/// A tabela cobre o vocabulário inteiro, sem id repetido; o slider e o chip da gravidade mostram o
/// GRUPO dos dois eixos.
#[test]
fn toda_linha_do_painel_tem_canal() {
    let campos: Vec<u16> = CAMPOS.iter().map(|&(c, _)| c).collect();
    assert_eq!(
        campos,
        m::CAMPOS.to_vec(),
        "a ordem e a cobertura do vocabulário"
    );
    let mut ids: Vec<NodeId> = CAMPOS.iter().flat_map(|(_, i)| i.iter().copied()).collect();
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n, "um id em duas linhas");
    assert_eq!(canal_do_campo(painel::PHYSICS_GRAVITY_Y_NUM).len(), 2);
    assert_eq!(canal_do_campo(painel::PHYSICS_SUBSTEPS).len(), 1);
    assert!(canal_do_campo(painel::PHYSICS_SHOW_COLLIDERS).is_empty());
}

/// ⭐ O clique com a animação de um OBJECTO em vista não grava no clip dele.
#[test]
fn o_clique_da_cena_nao_grava_no_clip_de_um_objecto() {
    let inst = Instante {
        t_cut: 0.0,
        solo: false,
    };
    let autorar = |_: u64, _: PropKind| Some(TimelineIntent::BeginEdit);
    let mut doc = TimelineDoc::new();
    let mut w = World::new();
    ph2d_ecs::garante_cena(&mut w);
    let na_cena = clique(
        &doc,
        &[painel::PHYSICS_GRAVITY_Y],
        &w,
        inst,
        |_| Vec::new(),
        |_, _| Some(0.0),
        autorar,
    );
    assert!(!na_cena.is_empty(), "controlo: na Cena grava");
    let i = doc.add_clip_for("Heroi".into(), 7).expect("clip");
    doc.set_active(i);
    let no_objecto = clique(
        &doc,
        &[painel::PHYSICS_GRAVITY_Y],
        &w,
        inst,
        |_| Vec::new(),
        |_, _| Some(0.0),
        autorar,
    );
    assert!(no_objecto.is_empty());
}
