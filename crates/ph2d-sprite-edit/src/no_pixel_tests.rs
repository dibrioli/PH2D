use super::*;

fn cena(olho: Option<Visibility>) -> (World, Entity, Entity) {
    let mut w = World::new();
    let mut o = w.spawn(Transform::default());
    if let Some(v) = olho {
        o.insert(v);
    }
    let origem = o.id();
    let trabalho = w.spawn(Transform::default()).id();
    abre(&mut w, origem, trabalho, 7);
    (w, origem, trabalho)
}

#[test]
fn o_quadro_de_entrada_nao_fecha_e_sair_do_draw_fecha_uma_vez() {
    let (mut w, origem, trabalho) = cena(Some(Visibility::default()));
    assert!(terminou(&mut w, None).is_none(), "ainda não entrou no Draw");
    assert!(terminou(&mut w, Some(trabalho.to_bits())).is_none());
    assert_eq!(
        terminou(&mut w, None).map(|a| (a.origem, a.key)),
        Some((origem, 7))
    );
    assert!(terminou(&mut w, None).is_none(), "devolve só uma vez");
    assert!(w.get_entity(trabalho).is_err(), "o objecto de trabalho sai");
}

#[test]
fn a_sprite_esconde_se_e_volta_como_estava() {
    let (mut w, origem, _) = cena(None);
    assert!(
        w.get::<Visibility>(origem).is_some_and(|v| v.hidden),
        "sem olho também se esconde"
    );
    encerra(&mut w);
    assert!(w.get::<Visibility>(origem).is_none());

    let escondida = Visibility::hidden();
    let (mut w, origem, _) = cena(Some(escondida));
    encerra(&mut w);
    assert_eq!(w.get::<Visibility>(origem).copied(), Some(escondida));
}
