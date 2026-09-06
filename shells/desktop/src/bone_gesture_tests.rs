//! Os gates do gesto do modo Osso — o que o artista aponta contra o que o documento guarda.

use super::*;

fn mundo(sim: &SimWorld, bits: u64) -> ([f64; 2], [f64; 2]) {
    crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .map(|(_, a, b)| (a, b))
        .expect("o osso")
}

/// ⭐⭐ **O OSSO NASCE ONDE O ARTISTA APONTOU, seja qual for a pose do pai.**
///
/// O pai está girado 90° **e** escalado 2×; o filho é pedido em MUNDO e tem de sair exactamente
/// ali. Compor ângulos e dividir comprimentos à mão acertaria neste caso e falharia com escala
/// não-uniforme — levar os DOIS pontos ao espaço local acerta em qualquer afim.
#[test]
fn a_bone_is_born_exactly_where_the_pointer_asked_whatever_the_parent_pose_is() {
    let mut sim = SimWorld::default();
    let raiz = Entity::from_bits(create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("raiz"));
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(raiz)
            .expect("Transform");
        t.rotation = std::f32::consts::FRAC_PI_2;
        t.scale = ph2d_core::Vec2::new(2.0, 2.0);
    }
    let filho = create(&mut sim, Some(raiz), [30.0, 40.0], [30.0, 55.0]).expect("filho");
    let (a, b) = mundo(&sim, filho);
    assert!(
        (a[0] - 30.0).abs() < 1e-4 && (a[1] - 40.0).abs() < 1e-4,
        "a origem saiu em {a:?} e foi pedida em (30,40)"
    );
    assert!(
        (b[0] - 30.0).abs() < 1e-4 && (b[1] - 55.0).abs() < 1e-4,
        "a ponta saiu em {b:?} e foi pedida em (30,55)"
    );
}

/// **Apontar um osso acha-o; apontar ao lado não.** O raio é o mesmo das alças do vetor.
#[test]
fn pointing_at_a_bone_finds_it_and_pointing_beside_it_does_not() {
    let mut sim = SimWorld::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("osso");
    // 1 unidade de mundo por píxel ⇒ o raio é 12 unidades.
    assert_eq!(hit(&sim, [5.0, 3.0], 1.0), Some(osso));
    assert_eq!(
        hit(&sim, [5.0, 40.0], 1.0),
        None,
        "40 unidades esta' fora do raio"
    );
    // E o raio SEGUE o zoom: com o mundo dez vezes mais denso por píxel, 40 cabe.
    assert_eq!(hit(&sim, [5.0, 40.0], 10.0), Some(osso));
}

/// ⛔ **Um osso raiz nasce com `RootOrder` explícito.** Sem ele a árvore desempata por bits de
/// alocação, e o undo passa a registar um passo espúrio por quadro (BUGS #15) — um defeito que não
/// se vê no osso, vê-se no `Ctrl+Z` do app inteiro.
#[test]
fn a_root_bone_is_born_with_an_explicit_root_order() {
    let mut sim = SimWorld::default();
    let a = Entity::from_bits(create(&mut sim, None, [0.0, 0.0], [1.0, 0.0]).expect("a"));
    let b = Entity::from_bits(create(&mut sim, None, [0.0, 5.0], [1.0, 5.0]).expect("b"));
    fn ord(sim: &SimWorld, e: Entity) -> Option<u32> {
        sim.world().get::<RootOrder>(e).map(|o| o.0)
    }
    assert!(
        ord(&sim, a).is_some() && ord(&sim, b).is_some(),
        "faltou o RootOrder"
    );
    assert_ne!(
        ord(&sim, a),
        ord(&sim, b),
        "duas raizes com a mesma ordem sao um empate"
    );
    // E um osso FILHO não é raiz — pôr-lhe `RootOrder` seria descrevê-lo como o que ele não é.
    let f = Entity::from_bits(create(&mut sim, Some(a), [1.0, 0.0], [2.0, 0.0]).expect("f"));
    assert!(sim.world().get::<ChildOf>(f).is_some());
    assert!(ord(&sim, f).is_none(), "um filho nao leva RootOrder");
}

/// ⭐⭐⭐ **AGARRAR O CORPO GIRA; AGARRAR A JUNTA DESLOCA.** As duas metades, porque uma sozinha
/// deixa metade do rig inalcançável — sem a rotação não se posa, sem o deslocamento o esqueleto
/// nunca sai de onde nasceu.
#[test]
fn grabbing_the_body_turns_the_bone_and_grabbing_the_joint_moves_it() {
    let mut sim = SimWorld::default();
    let osso = Entity::from_bits(create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("osso"));
    // Apontar para cima: a ponta sobe, a ORIGEM fica.
    assert!(pose(&mut sim, osso, [0.0, 7.0], false));
    let (a, b) = mundo(&sim, osso.to_bits());
    assert!(
        a[0].abs() < 1e-5 && a[1].abs() < 1e-5,
        "a origem andou: {a:?}"
    );
    assert!(
        b[0].abs() < 1e-4 && (b[1] - 10.0).abs() < 1e-4,
        "a ponta devia ir para (0,10) e foi para {b:?}"
    );
    // Pela junta: a origem vai para o ponteiro e o osso leva a direcção consigo.
    assert!(pose(&mut sim, osso, [4.0, 4.0], true));
    let (a2, b2) = mundo(&sim, osso.to_bits());
    assert!(
        (a2[0] - 4.0).abs() < 1e-5 && (a2[1] - 4.0).abs() < 1e-5,
        "a junta nao foi para o ponteiro: {a2:?}"
    );
    assert!(
        (b2[1] - 14.0).abs() < 1e-4,
        "deslocar mudou a DIRECCAO do osso: {b2:?}"
    );
}

/// ⛔ **Apontar para a PRÓPRIA origem não move nada.** Ali não há direcção, e um `atan2(0,0)` daria
/// um ângulo arbitrário — o osso saltaria no instante em que o ponteiro cruzasse a junta.
#[test]
fn aiming_at_the_bones_own_origin_does_nothing() {
    let mut sim = SimWorld::default();
    let osso = Entity::from_bits(create(&mut sim, None, [3.0, 1.0], [9.0, 1.0]).expect("osso"));
    let antes = mundo(&sim, osso.to_bits());
    assert!(!pose(&mut sim, osso, [3.0, 1.0], false));
    assert_eq!(antes, mundo(&sim, osso.to_bits()));
}

/// ⚠️ **Dois ossos nunca partilham o NOME** — a referência durável deste app é o hash do `Name`,
/// então dois "Bone" seriam o mesmo sujeito para a timeline e para todo binding.
#[test]
fn two_bones_never_share_a_name() {
    let mut sim = SimWorld::default();
    fn nome(sim: &SimWorld, bits: u64) -> String {
        sim.world()
            .get::<Name>(Entity::from_bits(bits))
            .map(|n| n.as_str().to_string())
            .expect("Name")
    }
    let a = create(&mut sim, None, [0.0, 0.0], [1.0, 0.0]).expect("a");
    let b = create(&mut sim, None, [0.0, 5.0], [1.0, 5.0]).expect("b");
    assert_ne!(nome(&sim, a), nome(&sim, b));
}

/// ⭐⭐⭐ **UM CLIQUE SOBRE UMA FORMA, NA FERRAMENTA DE OSSO, SELECCIONA-A** — o report do Enio de
/// 2026-09-06 (*"o bind não funciona"*), reduzido à decisão que o produzia.
///
/// ⛔ **O botão *Bind* age sobre a selecção de FORMAS**, e nesta ferramenta um press nunca
/// seleccionava nada: a secção era pintada, o botão acendia, o clique chegava ao barramento — e a
/// shell recusava por não haver sujeito. ⚠️ **Nenhum gate desta linha o via**: o `seam_bone` prova
/// que o clique SAI do painel, e os gates da pele provam o que o `bind` faz **depois** de receber
/// as formas. Entre os dois faltava *a ferramenta consegue produzir esse sujeito?*
#[test]
fn a_press_over_a_shape_in_the_bone_tool_picks_it_so_bind_has_a_subject() {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(crate::build_smoke::shape(
        ph2d_vec_scene::ShapeKind::Ellipse,
        [2.0, -2.0],
        [8.0, 2.0],
        &[],
        [180, 140, 220],
    ));
    let pen = ph2d_vec_edit::PenTool::default();
    // No MIOLO da forma, longe de osso nenhum.
    let d = press(&sim, &scene, &pen, [5.0, 0.0], 1.0, None);
    assert_eq!(
        d,
        BonePress::Start {
            origin: [5.0, 0.0],
            pick: Some(id)
        },
        "apontar a forma tem de a devolver - sem isto o `Bind` nunca tem sujeito"
    );
    // E um press sobre um OSSO nao e' uma escolha de forma: e' agarrar o osso.
    // ⚠️ O osso é LONGO de propósito: a `BONE_JOINT_R_PX` vale 6 unidades a este zoom, então num
    // osso curto TODO ponto é a junta — e a fixtura mediria o verbo errado.
    let osso = create(&mut sim, None, [3.0, 4.0], [27.0, 4.0]).expect("osso");
    let g = press(&sim, &scene, &pen, [20.0, 4.0], 1.0, None);
    assert_eq!(
        g,
        BonePress::Grab {
            bone: osso,
            joint: false
        },
        "sobre o CORPO de um osso o press agarra-o, nao aponta a forma"
    );
}

/// ⭐⭐⭐ **COM UM OSSO ACESO, O PRÓXIMO NASCE NA PONTA DELE — arraste-se onde se arrastar.**
///
/// ⛔ **A 1.ª lei desta wave era um ENCAIXE POR PROXIMIDADE, e este gate mostrou-a INALCANÇÁVEL:**
/// a ponta está sobre o segmento do osso, logo todo press dentro do raio de encaixe cai também
/// dentro do raio de ACERTO — o ramo `Grab` ganhava sempre e o encaixe nunca corria. *Um encaixe
/// que exige pontaria dentro do alvo que ele quer evitar não é um encaixe.*
#[test]
fn with_a_bone_selected_the_next_one_grows_from_its_tip() {
    let mut sim = SimWorld::default();
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("osso");
    // Longe do osso (senão o press agarra-o), e a origem sai na PONTA dele na mesma.
    let d = press(&sim, &scene, &pen, [60.0, 40.0], 1.0, Some(osso));
    assert_eq!(
        d,
        BonePress::Start {
            origin: [10.0, 0.0],
            pick: None
        },
        "o filho tem de crescer da ponta do pai"
    );
    // Sem osso aceso: um osso SOLTO nasce onde a mao pousou.
    let f = press(&sim, &scene, &pen, [60.0, 40.0], 1.0, None);
    assert_eq!(
        f,
        BonePress::Start {
            origin: [60.0, 40.0],
            pick: None
        },
        "sem pai, a origem e' o ponto apontado"
    );
}
