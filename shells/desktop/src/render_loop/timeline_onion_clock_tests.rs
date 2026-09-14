use super::*;

/// ⭐⭐⭐ **SEM instante de clip não há fantasma nenhum** — a metade `None` do relógio. ⛔ O clip pode
/// não tocar no instante da vista (ou tocar duas vezes numa pilha), e aí não há «agora» de que o
/// passado e o futuro sejam vizinhos.
#[test]
fn without_a_clip_instant_the_onion_publishes_nothing() {
    let (sim, mut present, osso, _arte) = rig_com_pele();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0f32), (4.0, 1.2)] {
        doc.insert_key(
            osso,
            PropKind::Rotation,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    // ⛔ O CONTROLO é a MESMA cena COM instante — senão este gate afirmaria que a fixtura está vazia.
    let mut ghosts = |t| {
        let mut out = ph2d_render::LiftedInstances::default();
        super::super::collect_onion_ghosts(
            &settings(),
            &sim,
            &mut present,
            &doc,
            Some(osso),
            t,
            PPM,
            &mut out,
        );
        out.len()
    };
    assert_eq!(
        ghosts(None),
        0,
        "sem instante de clip nao ha' vizinho nenhum"
    );
    assert!(
        ghosts(Some(2.0)) > 0,
        "controlo: com instante tinha de ghostar"
    );
}
