use super::*;

#[test]
fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE").then(|| v.to_owned());
    assert_eq!(armed_level_in(com("1")), Some(1));
    assert_eq!(armed_level_in(com("2")), Some(2));
    assert_eq!(armed_level_in(com("3")), Some(3));
    assert_eq!(armed_level_in(com("4")), Some(4));
    assert_eq!(armed_level_in(com("5")), None);
    assert_eq!(armed_level_in(com("x")), None);
    assert_eq!(armed_level_in(|_| None), None);
}

#[test]
fn o_condutor_joga_corre_e_para_por_esta_ordem() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE_DRIVE").then(|| v.to_owned());
    assert_eq!(conducao_in(com("window")), Some(GameStage::Window));
    assert_eq!(conducao_in(com("fullscreen")), Some(GameStage::Fullscreen));
    assert_eq!(conducao_in(com("editor")), Some(GameStage::Editor));
    assert_eq!(conducao_in(com("x")), None);
    let s = GameStage::Fullscreen;
    let g: Vec<_> = (0..1000).filter_map(|q| gesto_no_quadro(1, s, q)).collect();
    assert_eq!(
        g,
        [
            Gesto::Joga(s),
            Gesto::Direita(true),
            Gesto::Direita(false),
            Gesto::Para
        ]
    );
}

/// ⭐ GEF 19 E7c — **os limites da câmera do nível 1 são as paredes**: em x a borda de FORA de cada
/// parede (a vista nunca mostra o fim do mundo nem corta a parede), em baixo o fundo do chão, e o
/// tecto não corta a parede. (Mutação: `max: [17.0, 9.0]` ⇒ RED.)
#[test]
fn os_limites_da_camara_do_nivel_1_sao_as_paredes() {
    let mut world = ph2d_ecs::World::new();
    let _ = nivel_1(&mut world);
    let limites: Vec<CameraLimits> = world
        .query::<&CameraLimits>()
        .iter(&world)
        .cloned()
        .collect();
    assert_eq!(limites.len(), 1, "uma câmera no nível 1");
    let l = &limites[0];
    let bloco = |nome: &str| {
        let (_, c, h) = BLOCOS
            .iter()
            .find(|(n, _, _)| *n == nome)
            .unwrap_or_else(|| panic!("o bloco {nome}"));
        (*c, *h)
    };
    let ((ce, he), (cd, hd)) = (bloco("Wall Left"), bloco("Wall Right"));
    let (cc, hc) = bloco("Ground Left");
    assert_eq!(
        l.min[0],
        ce[0] - he[0],
        "a esquerda é a borda de fora da parede esquerda"
    );
    assert_eq!(
        l.max[0],
        cd[0] + hd[0],
        "a direita é a borda de fora da parede direita"
    );
    assert_eq!(l.min[1], cc[1] - hc[1], "em baixo, o fundo do chão");
    assert!(
        l.max[1] >= ce[1] + he[1] && l.max[1] >= cd[1] + hd[1],
        "o tecto não corta as paredes"
    );
}
