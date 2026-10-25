use super::*;

/// ⭐ A tabela cobre o vocabulário inteiro, na ordem dele, sem id repetido.
#[test]
fn toda_linha_autoral_da_mesa_tem_canal() {
    let campos: Vec<u16> = CAMPOS.iter().map(|&(c, _)| c).collect();
    assert_eq!(
        campos,
        m::campos().collect::<Vec<_>>(),
        "a ordem e a cobertura do vocabulário"
    );
    let mut ids: Vec<NodeId> = CAMPOS.iter().flat_map(|(_, i)| i.iter().copied()).collect();
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n, "um id em duas linhas");
    assert_eq!(canal_do_campo(painel::AMIX_DUCK_KEY_BUS[2]).len(), 1);
    assert!(
        canal_do_campo(painel::AMIX_PLAY).is_empty(),
        "o Play Test é da sessão"
    );
    assert!(
        canal_do_campo(painel::AMIX_MASTER_METER).is_empty(),
        "o medidor não é autoria"
    );
}

/// ⭐ O clique com a animação de um OBJECTO em vista não grava no clip dele.
#[test]
fn o_clique_da_mesa_nao_grava_no_clip_de_um_objecto() {
    let inst = Instante {
        t_cut: 0.0,
        solo: false,
    };
    let autorar = |_: u64, _: PropKind| Some(TimelineIntent::BeginEdit);
    let mut doc = TimelineDoc::new();
    let mut w = World::new();
    ph2d_ecs::garante_cena(&mut w);
    let alvo = [painel::SUB_FADER[1]];
    let na_cena = clique(
        &doc,
        &alvo,
        &w,
        inst,
        |_| Vec::new(),
        |_, _| Some(1.0),
        autorar,
    );
    assert!(!na_cena.is_empty(), "controlo: na Cena grava");
    let i = doc.add_clip_for("Heroi".into(), 7).expect("clip");
    doc.set_active(i);
    let no_objecto = clique(
        &doc,
        &alvo,
        &w,
        inst,
        |_| Vec::new(),
        |_, _| Some(1.0),
        autorar,
    );
    assert!(no_objecto.is_empty());
}
