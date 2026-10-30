use std::collections::BTreeSet;

use super::*;

/// A tabela cobre o vocabulário inteiro, sem id repetido; o slider e o chip da gravidade mostram o
/// GRUPO dos dois eixos.
#[test]
fn toda_linha_do_painel_tem_canal() {
    let campos: Vec<u16> = CAMPOS.iter().map(|&(c, _)| c).collect();
    let mut vocab = m::CAMPOS.to_vec();
    vocab.push(m::CAMADAS);
    assert_eq!(campos, vocab, "a ordem e a cobertura do vocabulário");
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

/// ⭐ **A matriz de camadas é UM losango** (F41): o número de células CONTA-SE nas duas pontas (o
/// vocabulário e o painel), qualquer célula mostra o grupo das 36, e o clique grava as 36 — com as
/// células publicadas, como a shell as passa (`campos_lidos`).
#[test]
fn o_losango_da_matriz_grava_as_36_celulas() {
    let n = ph2d_physics_ecs::MAX_LAYERS * (ph2d_physics_ecs::MAX_LAYERS + 1) / 2;
    assert_eq!(usize::from(m::CELULAS), n);
    assert_eq!(painel::PHYSICS_LAYER_CELL.len(), n);
    assert_eq!(crate::canais_mundo::celulas().count(), n);
    let celulas: BTreeSet<u16> = crate::canais_mundo::celulas().map(|c| c.0).collect();
    assert_eq!(celulas, (0..m::CELULAS).collect(), "cada célula, um canal");
    for &id in painel::PHYSICS_LAYER_CELL.iter() {
        assert_eq!(canal_do_campo(id).len(), n);
    }
    let inst = Instante {
        t_cut: 0.0,
        solo: false,
    };
    let mut w = World::new();
    ph2d_ecs::garante_cena(&mut w);
    let todas: Vec<FieldKey> = (0..m::CELULAS).map(crate::canais_mundo::kc).collect();
    let visto = std::cell::RefCell::new(BTreeSet::new());
    let autorar = |_: u64, p: PropKind| {
        visto.borrow_mut().insert(p);
        Some(TimelineIntent::BeginEdit)
    };
    let doc = TimelineDoc::new();
    let canto = [painel::PHYSICS_LAYER_CELL[0]];
    let _ = clique(
        &doc,
        &canto,
        &w,
        inst,
        |_| todas.clone(),
        |_, _| Some(1.0),
        autorar,
    );
    let esperado: BTreeSet<PropKind> = todas.into_iter().map(PropKind::Field).collect();
    assert_eq!(visto.into_inner(), esperado);
}
