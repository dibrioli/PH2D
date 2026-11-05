use super::*;

fn raiz(sim: &mut SimWorld, _: &str) -> u64 {
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::default(), Visibility::default()))
        .id()
        .to_bits()
}

fn dois_pixeis() -> (u32, u32, Vec<u8>) {
    (2, 1, vec![255, 0, 0, 255, 0, 0, 0, 0])
}

fn cena() -> (SimWorld, PixelApp, u64) {
    let mut sim = SimWorld::new();
    let origem = raiz(&mut sim, "");
    (sim, PixelApp::default(), origem)
}

#[test]
fn a_imagem_vai_e_volta_igual_quando_nada_se_pinta() {
    let (mut sim, mut app, origem) = cena();
    let trabalho = abre(&mut sim, &mut app, origem, dois_pixeis(), raiz).expect("abre");
    assert!(terminou(sim.world_mut(), &mut app, Some(trabalho)).is_none());
    let d = terminou(sim.world_mut(), &mut app, None).expect("saiu do Draw");
    assert_eq!((d.origem, d.w, d.h), (origem, 2, 1));
    assert_eq!(d.rgba, dois_pixeis().2);
}

/// O quadro de entrada ainda não está em Draw: isso não pode fechar a edição.
#[test]
fn antes_de_entrar_em_draw_a_edicao_nao_termina() {
    let (mut sim, mut app, origem) = cena();
    abre(&mut sim, &mut app, origem, dois_pixeis(), raiz).expect("abre");
    assert!(terminou(sim.world_mut(), &mut app, None).is_none());
    assert!(app.edicao.is_some());
}

#[test]
fn a_sprite_se_esconde_na_ida_e_volta_como_estava() {
    let (mut sim, mut app, origem) = cena();
    let e = Entity::from_bits(origem);
    let trabalho = abre(&mut sim, &mut app, origem, dois_pixeis(), raiz).expect("abre");
    assert!(!sim.world().get::<Visibility>(e).unwrap().visible);
    assert!(sim.world().get_entity(Entity::from_bits(trabalho)).is_ok());
    encerra(sim.world_mut(), &mut app).expect("encerra");
    assert!(sim.world().get::<Visibility>(e).unwrap().visible);
    assert!(sim.world().get_entity(Entity::from_bits(trabalho)).is_err());
    assert!(app.docs.is_empty(), "o documento de trabalho não fica");
}

/// O primeiro Undo do artista não pode apagar a imagem que ele abriu.
#[test]
fn o_documento_nasce_sem_passos_de_desfazer() {
    let (mut sim, mut app, origem) = cena();
    abre(&mut sim, &mut app, origem, dois_pixeis(), raiz).expect("abre");
    let key = app.edicao.as_ref().unwrap().key;
    assert!(!app.docs.get_mut(&key).unwrap().history.can_undo());
}

#[test]
fn uma_edicao_de_cada_vez() {
    let (mut sim, mut app, origem) = cena();
    abre(&mut sim, &mut app, origem, dois_pixeis(), raiz).expect("abre");
    assert!(abre(&mut sim, &mut app, origem, dois_pixeis(), raiz).is_err());
}
