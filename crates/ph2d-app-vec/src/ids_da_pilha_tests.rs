use super::*;
use ph2d_ecs::canal_vetor_ids::IdsDoVetor;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_trim::TrimSpec;
use ph2d_vec_scene::{Paint, PaintEntry, Rgba8};

fn tinta() -> PaintEntry {
    PaintEntry::fill(Paint::solid(Rgba8::new(1, 2, 3, 255)))
}

fn efeito() -> FxEntry {
    FxEntry::new(PathEffect::Trim(TrimSpec::default()))
}

/// Uma cena com uma forma ligada a uma entidade.
fn palco(paints: Vec<PaintEntry>, effects: Vec<FxEntry>) -> (SimWorld, VecScene, u64) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        paints,
        effects,
        ..VecPath::default()
    });
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    (sim, scene, e)
}

fn pilha(scene: &VecScene) -> Vec<ItemId> {
    ids(&scene.paths()[0]).collect()
}

/// ⭐ O contador não sabe dos ids da cena (ela não está no mundo): antes de alocar, ele sobe acima
/// do maior deles — senão a entrada nova repetiria o id de outra forma.
#[test]
fn o_passe_reconcilia_o_contador_com_os_ids_da_cena() {
    let mut velha = tinta();
    velha.id = ItemId(40);
    let (mut sim, mut scene, _) = palco(vec![velha, tinta()], vec![efeito()]);
    sim.world_mut().insert_resource(StableIdCounter::default());
    assert_eq!(atribuir(&mut sim, &mut scene), 2);
    let ids = pilha(&scene);
    assert_eq!(ids[0], ItemId(40));
    assert!(ids[1].0 > 40 && ids[2].0 > 40, "{ids:?}");
    assert_ne!(ids[1], ids[2]);
    // Idempotente: nada falta ⇒ nada aloca.
    assert_eq!(atribuir(&mut sim, &mut scene), 0);
    assert_eq!(pilha(&scene), ids);
}

/// Um id repetido no MESMO caminho (tinta e efeito juntos — a cópia de uma entrada) é refeito; o
/// primeiro fica.
#[test]
fn um_id_repetido_no_mesmo_caminho_e_refeito() {
    let mut a = tinta();
    a.id = ItemId(5);
    let b = a.clone();
    let mut fx = efeito();
    fx.id = ItemId(5);
    let (mut sim, mut scene, _) = palco(vec![a, b], vec![fx]);
    assert_eq!(atribuir(&mut sim, &mut scene), 2);
    let ids = pilha(&scene);
    assert_eq!(ids[0], ItemId(5));
    assert!(
        ids[1] != ids[0] && ids[2] != ids[0] && ids[1] != ids[2],
        "{ids:?}"
    );
    assert!(ids.iter().all(|i| !i.is_none()));
}

/// O espelho: publica-se depois de atribuir, não se publica com um id por atribuir, e a entidade
/// que sai sai do mapa.
#[test]
fn o_espelho_publica_so_a_forma_completa() {
    let (mut sim, mut scene, e) = palco(vec![tinta()], vec![efeito()]);
    publicar(&mut sim, &scene);
    assert!(sim.world().resource::<IdsDoVetor>().0.is_empty());
    passe(&mut sim, &mut scene);
    let ids = pilha(&scene);
    let m = sim.world().resource::<IdsDoVetor>().0.clone();
    assert_eq!(m[&e].tintas, ids[..1]);
    assert_eq!(m[&e].efeitos, ids[1..]);
    sim.world_mut().despawn(Entity::from_bits(e));
    publicar(&mut sim, &scene);
    assert!(sim.world().resource::<IdsDoVetor>().0.is_empty());
}

/// ⭐ D30: a tinta de base que mudou de cor solta-se no passe de TODO quadro, antes da captura do
/// undo: a cor e a ligação entram no MESMO passo, e Desfazer devolve as duas.
#[test]
fn o_passe_solta_a_base_mexida_e_nao_solta_a_que_o_estilo_pinta() {
    let (mut sim, mut scene, _) = palco(Vec::new(), Vec::new());
    let id = scene.paths()[0].id;
    let cor = Rgba8::new(1, 2, 3, 255);
    scene.path_mut(id).expect("forma").fill = Some(Paint::Solid(Rgba8::new(0, 0, 0, 255)));
    let e = scene.add_estilo("Ink", cor);
    assert!(scene.link_estilo((id, ItemId::BASE_FILL), e));
    assert!(scene.set_estilo_color(e, Rgba8::new(9, 9, 9, 255)));
    passe(&mut sim, &mut scene);
    assert!(
        scene.estilo_de((id, ItemId::BASE_FILL)).is_some(),
        "o estilo a pintar soltou"
    );
    scene.path_mut(id).expect("forma").fill = Some(Paint::Solid(cor));
    passe(&mut sim, &mut scene);
    assert_eq!(
        scene.estilo_de((id, ItemId::BASE_FILL)),
        None,
        "o passe não soltou"
    );
}
