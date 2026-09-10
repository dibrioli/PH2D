//! Os gates do gesto do modo Osso — o que o artista aponta contra o que o documento guarda.

use super::*;
use ph2d_skeleton_render::BonePart;
use ph2d_tool_vector::BoneAction;

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
    let (a, b) = test_segment(&sim, filho);
    assert!(
        (a[0] - 30.0).abs() < 1e-4 && (a[1] - 40.0).abs() < 1e-4,
        "a origem saiu em {a:?} e foi pedida em (30,40)"
    );
    assert!(
        (b[0] - 30.0).abs() < 1e-4 && (b[1] - 55.0).abs() < 1e-4,
        "a ponta saiu em {b:?} e foi pedida em (30,55)"
    );
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
    let d = press(
        &sim,
        &scene,
        &pen,
        [5.0, 0.0],
        1.0,
        None,
        BoneAction::Create,
    );
    assert_eq!(
        d,
        BonePress::Start {
            birth: BoneBirth {
                origin: [5.0, 0.0],
                parent: None
            },
            pick: Some(id)
        },
        "apontar a forma tem de a devolver - sem isto o `Bind` nunca tem sujeito"
    );
    // ⭐ E um press sobre o CORPO de um osso arma um osso NOVO e continua a apontar a forma: desde
    // 2026-09-09 o corpo não consome o press em *Criar* (só a PONTA decide parentesco).
    //
    // ⚠️ O osso ATRAVESSA a elipse de propósito, e o ponto está longe da PONTA dele (`x = 26`, a
    // `20` unidades daqui, contra um raio de ponta de `joint_radius_px(24) = 6`): a fixtura tem de
    // medir o corpo, e um ponto perto da ponta mediria o parentesco.
    create(&mut sim, None, [2.0, 0.0], [26.0, 0.0]).expect("osso");
    assert_eq!(
        press(
            &sim,
            &scene,
            &pen,
            [6.0, 0.0],
            1.0,
            None,
            BoneAction::Create
        ),
        BonePress::Start {
            birth: BoneBirth {
                origin: [6.0, 0.0],
                parent: None
            },
            pick: Some(id)
        },
        "em CRIAR, o CORPO de um osso nao consome o press - comecar um osso em cima de outro e' um \
         gesto legitimo, e a forma por baixo continua a ser o sujeito do `Bind`"
    );
}

/// ⭐⭐⭐ **O PARENTESCO É A PONTA, E SÓ A PONTA** (ordem do dono, 2026-09-09: *«para criar um osso
/// como filho de outro o clique deve acontecer na ponta do osso pai»*).
///
/// ⛔⛔ **A lei anterior lia a SELECÇÃO — e este gate media-a.** Ela dizia *«com um osso aceso, o
/// próximo nasce na ponta dele, arraste-se onde se arrastar»*, e é exactamente isso que o dono
/// mandou tirar: com um esqueleto na cena não havia press nenhum que fizesse uma **raiz** nova
/// (*«não se pode criar ossos fora da cadeia»*).
///
/// ⇒ as duas células que importam, com o MESMO osso aceso nas duas: **na ponta** ⇒ filho, e a
/// origem encaixa nela; **longe dela** ⇒ raiz, e a origem é o ponto cru.
///
/// (Mutação: o `press` ler o `selected` para escolher o pai ⇒ RED na 2ª.)
#[test]
fn only_a_press_on_the_tip_makes_the_new_bone_a_child() {
    let mut sim = SimWorld::default();
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [40.0, 0.0]).expect("osso");
    // NA PONTA (40,0) — a porta do parentesco.
    assert_eq!(
        press(
            &sim,
            &scene,
            &pen,
            [40.0, 0.0],
            1.0,
            None,
            BoneAction::Create
        ),
        BonePress::Start {
            birth: BoneBirth {
                origin: [40.0, 0.0],
                parent: Some(osso)
            },
            pick: None
        },
        "um press na PONTA tem de armar um FILHO, e a origem tem de encaixar nela"
    );
    // ⭐⭐⭐ LONGE dela, **com o osso ACESO**: raiz nova. É a célula do report.
    assert_eq!(
        press(
            &sim,
            &scene,
            &pen,
            [60.0, 40.0],
            1.0,
            Some(osso),
            BoneAction::Create
        ),
        BonePress::Start {
            birth: BoneBirth {
                origin: [60.0, 40.0],
                parent: None
            },
            pick: None
        },
        "com um osso ACESO, um press longe da ponta dele tem de fazer uma RAIZ - senao nao ha' \
         gesto nenhum que crie um osso fora da cadeia, que e' o report do dono"
    );
}

/// ⭐⭐⭐ **RAMIFICAR DO MEIO DE UMA CORRENTE É O MESMO GESTO QUE CONTINUÁ-LA.**
///
/// ⛔ Na lei anterior isto era **inexprimível pelo canvas**: o pai era a selecção, então fazer o 2.º
/// braço de uma espinha exigia ir escolher o osso do meio na Hierarquia e voltar. Aqui a ponta dele
/// está desenhada e o press nela basta.
///
/// ⚠️ **E a ponta do pai é a raiz do FILHO**, no mesmo ponto: a competição é por **proximidade**,
/// e é ela que faz o press escolher o osso cuja PONTA está ali — não o que também passa por ali.
#[test]
fn a_press_on_a_mid_chain_tip_branches_from_that_bone() {
    let mut sim = SimWorld::default();
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    // Uma corrente de 3: 0→10, 10→20, 20→30.
    let ossos = test_chain(&mut sim, 3);
    // A ponta do PRIMEIRO é (10,0) — que é também a raiz do segundo.
    assert_eq!(
        press(
            &sim,
            &scene,
            &pen,
            [10.0, 0.0],
            1.0,
            Some(ossos[2]),
            BoneAction::Create
        ),
        BonePress::Start {
            birth: BoneBirth {
                origin: [10.0, 0.0],
                parent: Some(ossos[0])
            },
            pick: None
        },
        "a ponta do 1o osso tem de ramificar DELE - e nao do 2o, cuja raiz esta' no mesmo ponto"
    );
    // E a ponta do último continua a corrente.
    assert_eq!(
        press(
            &sim,
            &scene,
            &pen,
            [30.0, 0.0],
            1.0,
            None,
            BoneAction::Create
        ),
        BonePress::Start {
            birth: BoneBirth {
                origin: [30.0, 0.0],
                parent: Some(ossos[2])
            },
            pick: None
        },
        "a ponta da corrente tem de a continuar"
    );
}

/// ⭐⭐⭐ **UM GESTO, UM VERBO** — o report de 2026-09-07 (*«do modo como está fica confuso para o
/// usuário»*), dito como decisão.
///
/// ⛔ **Antes a ambiguidade era do PONTEIRO:** o MESMO arrasto criava ou posava consoante o que
/// estava por baixo do cursor. Isso torna **inalcançáveis** dois gestos legítimos — começar um osso
/// *em cima* de outro, e posar um osso *sem medo* de criar um por engano — e obriga o artista a
/// saber o que está debaixo do cursor antes de carregar.
///
/// Este gate mede as **seis** células: três pontos (corpo · ponta · vazio) × dois verbos.
///
/// ⚠️ **A PONTA é a célula que 2026-09-09 acrescentou**, e é a que separa melhor os dois verbos:
/// ali *Criar* ramifica e *Transformar* pega a cinemática inversa — **o mesmo pixel, dois verbos**.
///
/// (Mutação: o `press` ignorar o `action` ⇒ RED em três delas.)
#[test]
fn the_same_press_means_different_things_in_create_and_in_transform() {
    let mut sim = SimWorld::default();
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [40.0, 0.0]).expect("osso");
    // SOBRE o corpo do osso (longe da raiz e da ponta).
    let sobre = [20.0, 0.0];
    assert_eq!(
        press(&sim, &scene, &pen, sobre, 1.0, None, BoneAction::Create),
        BonePress::Start {
            birth: BoneBirth {
                origin: sobre,
                parent: None
            },
            pick: None
        },
        "CRIAR sobre o corpo de um osso: arma um osso NOVO ali, sem parentesco"
    );
    assert_eq!(
        press(&sim, &scene, &pen, sobre, 1.0, None, BoneAction::Transform),
        BonePress::Grab {
            bone: osso,
            part: BonePart::Body
        },
        "TRANSFORMAR sobre um osso: agarra e posa"
    );
    // ⭐ E na PONTA os dois verbos divergem: CRIAR ramifica, TRANSFORMAR pega o *end effector*.
    let ponta = [40.0, 0.0];
    assert_eq!(
        press(&sim, &scene, &pen, ponta, 1.0, None, BoneAction::Create),
        BonePress::Start {
            birth: BoneBirth {
                origin: ponta,
                parent: Some(osso)
            },
            pick: None
        },
        "CRIAR na ponta: ramifica"
    );
    assert_eq!(
        press(&sim, &scene, &pen, ponta, 1.0, None, BoneAction::Transform),
        BonePress::Grab {
            bone: osso,
            part: BonePart::Tip
        },
        "TRANSFORMAR na ponta: cinematica inversa, nunca um osso novo"
    );
    // NO VAZIO, longe de tudo.
    let vazio = [200.0, 200.0];
    assert_eq!(
        press(&sim, &scene, &pen, vazio, 1.0, None, BoneAction::Create),
        BonePress::Start {
            birth: BoneBirth {
                origin: vazio,
                parent: None
            },
            pick: None
        },
        "CRIAR no vazio: marca a origem de um osso novo"
    );
    assert_eq!(
        press(&sim, &scene, &pen, vazio, 1.0, None, BoneAction::Transform),
        BonePress::Pick { path: None },
        "TRANSFORMAR no vazio NAO pode armar osso nenhum - e' o verbo a ser um so'"
    );
}

/// ⭐⭐ **A PRÉ-VISUALIZAÇÃO OBEDECE AO MESMO LIMIAR QUE O `Up`** (Enio, 2026-09-07).
///
/// ⛔ O `Up` recusa um arrasto mais curto que o raio das alças — um osso de comprimento zero não
/// tem eixo, não pesa ponto nenhum e é invisível. Se a pré-visualização não soubesse disso, ela
/// **prometeria** um osso que o `Up` não faz, e o `CLAUDE.md` §5.0 nomeia esse defeito: *uma cena
/// que ensina o contrário do que acontece é pior que uma cena ausente*.
///
/// ⇒ os dois consultam a MESMA função, e este gate mede a lei dela nos dois lados da dobra.
#[test]
fn the_preview_arms_exactly_where_the_release_would_make_a_bone() {
    let o = [10.0, 10.0];
    // 1 unidade de mundo por píxel ⇒ o limiar é 12 unidades.
    assert!(
        !drag_makes_a_bone(o, [10.0, 10.0], 1.0),
        "zero nao faz osso"
    );
    assert!(
        !drag_makes_a_bone(o, [21.0, 10.0], 1.0),
        "11 px esta' abaixo do raio das alcas - o `Up` recusaria"
    );
    assert!(
        drag_makes_a_bone(o, [22.0, 10.0], 1.0),
        "12 px ja' faz osso, e a pre-visualizacao tem de o dizer"
    );
    // E o limiar é de TELA: aproximando o zoom faz-se um osso menor em mundo.
    assert!(
        drag_makes_a_bone(o, [11.0, 10.0], 0.05),
        "com o mundo mais denso por pixel, 1 unidade ja' passa os 12 px"
    );
}

/// ⭐⭐⭐ **DUAS CORRENTES SEPARADAS VIRAM UMA, E A ADOPTADA NÃO SE MEXE** (ordem do dono,
/// 2026-09-09: *«ligando a ponta de um com o fundo de outro ao criar um osso intermediário»*).
///
/// O gesto inteiro, na ordem em que o `Up` o executa: a ponta de `A` arma o press · o release cai na
/// base de `B` · o osso do meio nasce entre as duas · `B` passa a pendurar-se nele.
///
/// ⚠️⚠️ **A pose de MUNDO de `B` e de todos os filhos dele tem de ficar ONDE ESTAVA.** `B` é uma
/// corrente que o artista já posicionou; um `ChildOf` cru somaria a pose do osso novo e o esqueleto
/// inteiro saltaria — o defeito que o [`crate::vec_transform::reparent_keeping_world`] existe para
/// evitar, e cuja ausência se lê como *«juntei os ossos e o boneco explodiu»*.
///
/// ⚠️ **O filho de `B` entra no gate de propósito**: ele é a prova de que a corrente viaja INTEIRA,
/// e de que a compensação de pose se propaga em vez de valer só para o adoptado.
///
/// (Mutação: o `connect` pendurar por `ChildOf` cru em vez da porta que preserva o mundo ⇒ RED.)
#[test]
fn two_separate_chains_become_one_and_the_adopted_one_does_not_move() {
    let mut sim = SimWorld::default();
    let a = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("A");
    // A outra corrente: uma base e um filho, LONGE da primeira e fora do eixo dela.
    let b = create(&mut sim, None, [40.0, 5.0], [50.0, 5.0]).expect("B");
    let c = create(
        &mut sim,
        Some(Entity::from_bits(b)),
        [50.0, 5.0],
        [50.0, 15.0],
    )
    .expect("C");
    let (b_antes, c_antes) = (test_segment(&sim, b), test_segment(&sim, c));

    // O press na ponta de A, e o release na base de B.
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    let BonePress::Start { birth, .. } = press(
        &sim,
        &scene,
        &pen,
        [10.0, 0.0],
        1.0,
        None,
        BoneAction::Create,
    ) else {
        panic!("o press na ponta de A tem de armar um osso");
    };
    assert_eq!(birth.parent, Some(a));
    let (alvo, base) = splice_target(&sim, [40.0, 5.0], 1.0, birth).expect("a base de B e' o alvo");
    assert_eq!(alvo, b);
    assert_eq!(
        base,
        [40.0, 5.0],
        "a ponta do osso novo ENCAIXA na base de B"
    );

    let novo = create(&mut sim, Some(Entity::from_bits(a)), birth.origin, base).expect("o do meio");
    assert!(connect(&mut sim, alvo, novo), "a emenda tem de acontecer");

    // ⭐ Uma corrente só: de A ate' C, passando pelo osso novo.
    let cadeia: Vec<u64> = crate::skeleton_live::chain_to(&sim, c)
        .into_iter()
        .map(|e| e.to_bits())
        .collect();
    assert_eq!(
        cadeia,
        vec![a, novo, b, c],
        "as duas correntes tinham de virar UMA, com o osso do meio a costura'-las"
    );
    // ⭐⭐ E nada se mexeu: B e o filho dele estao onde o artista os pos.
    for (bits, antes, quem) in [(b, b_antes, "B"), (c, c_antes, "o filho de B")] {
        let agora = test_segment(&sim, bits);
        for (p, q) in [(agora.0, antes.0), (agora.1, antes.1)] {
            assert!(
                (p[0] - q[0]).abs() < 1e-4 && (p[1] - q[1]).abs() < 1e-4,
                "{quem} saltou de {q:?} para {p:?} ao ser adoptado — a pose de MUNDO tem de ficar"
            );
        }
    }
    // ⚠️ E a ponta do osso do meio pousa exactamente na base de B: a corrente e' contínua.
    let (_, ponta_do_meio) = test_segment(&sim, novo);
    assert!(
        (ponta_do_meio[0] - b_antes.0[0]).abs() < 1e-4
            && (ponta_do_meio[1] - b_antes.0[1]).abs() < 1e-4,
        "o osso do meio acaba em {ponta_do_meio:?} e a base de B esta' em {:?} — a corrente tem um \
         vao",
        b_antes.0
    );
}

/// ⛔ **O ADOPTADO DEIXA DE SER UMA RAIZ** — o `RootOrder` sai com ele.
///
/// ⚠️ Ele é o desempate entre RAÍZES, e o gate `a_root_bone_is_born_with_an_explicit_root_order` já
/// declara a outra metade da lei (*«um filho não leva `RootOrder`»*). Deixá-lo aqui faria a árvore
/// carregar uma ordem que descreve o que aquele osso já não é.
///
/// (Mutação: o `connect` não remover o `RootOrder` ⇒ RED.)
#[test]
fn the_adopted_root_stops_being_a_root() {
    let mut sim = SimWorld::default();
    let a = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("A");
    let b = create(&mut sim, None, [40.0, 0.0], [50.0, 0.0]).expect("B");
    assert!(
        sim.world().get::<RootOrder>(Entity::from_bits(b)).is_some(),
        "a fixtura nao produz o fenomeno: B tinha de nascer raiz"
    );
    let novo = create(
        &mut sim,
        Some(Entity::from_bits(a)),
        [10.0, 0.0],
        [40.0, 0.0],
    )
    .expect("meio");
    assert!(connect(&mut sim, b, novo));
    assert!(
        sim.world().get::<RootOrder>(Entity::from_bits(b)).is_none(),
        "B foi adoptado e ficou com o `RootOrder` — a arvore passa a carregar uma ordem que \
         descreve o que ele ja' nao e'"
    );
}

/// ⛔⛔ **A EMENDA RECUSA O LAÇO QUE PENDURARIA O APP.**
///
/// Arrastar da ponta de um FILHO para a base do PRÓPRIO ancestral pediria que o ancestral se
/// pendurasse num osso que descende dele. ⚠️ Não é um erro cosmético: uma travessia de `Transform`
/// sobre uma árvore cíclica **não devolve**.
///
/// ⚠️ **A recusa vive no `splice_target`, e é isso que a torna VISÍVEL:** o alvo simplesmente não
/// existe, logo a pré-visualização não encaixa e o osso nasce onde a mão está. Se a recusa vivesse
/// no `connect`, o desenho prometeria a emenda e o release entregaria um osso sem ela.
///
/// (Mutação: o `adopting_would_cycle` devolver `false` ⇒ RED.)
#[test]
fn splicing_refuses_the_loop_that_would_hang_the_app() {
    let mut sim = SimWorld::default();
    let a = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("A");
    let d = create(
        &mut sim,
        Some(Entity::from_bits(a)),
        [10.0, 0.0],
        [20.0, 0.0],
    )
    .expect("D");
    // O press na ponta de D; o release na base de A, que e' ancestral de D.
    let filho = BoneBirth {
        origin: [20.0, 0.0],
        parent: Some(d),
    };
    assert_eq!(
        splice_target(&sim, [0.0, 0.0], 1.0, filho),
        None,
        "adoptar o ancestral fecharia um laco — e uma travessia de Transform sobre um laco nao \
         devolve"
    );
    // ⚠️ E o CONTROLO: a mesma base, com um osso novo SEM pai, e' um alvo legítimo (dar uma raiz
    // nova a uma corrente). Sem isto o gate passaria com a emenda desligada.
    let solto = BoneBirth {
        origin: [0.0, -30.0],
        parent: None,
    };
    assert_eq!(
        splice_target(&sim, [0.0, 0.0], 1.0, solto).map(|(b, _)| b),
        Some(a),
        "sem pai nao ha' laco possivel — a emenda tem de valer"
    );
}

/// ⭐⭐⭐ **O QUE SE DESENHA É O OSSO QUE VAI NASCER** — a porta [`drag_now`], medida nos dois lados
/// da emenda.
///
/// ⛔⛔ **Foi uma MUTAÇÃO SOBREVIVENTE que a fez existir.** A 1.ª redacção desta wave resolvia as
/// três coisas — a emenda, a ponta encaixada e o limiar — **duas vezes**: uma na pré-visualização e
/// outra no release. Apagar o encaixe do lado do DESENHO deixava a suíte inteira verde, e o produto
/// ficava com o defeito mais caro desta família: *o osso acaba no cursor na tela e nasce na bolinha*.
/// ⇒ os dois consumidores lêem esta estrutura, e um gate de costura prova que a lêem.
///
/// (Mutação: o `drag_now` devolver `tip = pointer` com emenda ⇒ RED.)
#[test]
fn what_is_drawn_is_the_bone_that_will_be_born() {
    let mut sim = SimWorld::default();
    let a = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("A");
    // ⚠️ **B é LONGO de propósito**: a bolinha da base encolhe com o comprimento
    // (`joint_radius_px(comp) = min(12, comp/4)`), e com um osso de `10` o alvo tem `2,5` — a 1.ª
    // redacção desta fixtura punha o ponteiro a `3,6` e não produzia o fenómeno que o gate mede.
    create(&mut sim, None, [40.0, 0.0], [80.0, 0.0]).expect("B");
    let nascimento = BoneBirth {
        origin: [10.0, 0.0],
        parent: Some(a),
    };
    // ⚠️ O ponteiro cai PERTO da base de B, e não em cima: é assim que o dedo do artista chega, e
    // é a distância entre os dois pontos que torna o encaixe observável.
    let agora = drag_now(&sim, nascimento, [42.0, 3.0], 1.0);
    assert!(agora.splice.is_some(), "a fixtura nao produz o fenomeno");
    assert_eq!(
        agora.tip,
        [40.0, 0.0],
        "a ponta desenhada tem de ENCAIXAR na base do alvo — no cursor, o artista ve' um osso que \
         nao e' o que vai nascer"
    );
    assert!(agora.armed);
    // ⚠️ E longe de tudo a ponta é o ponteiro, sem encaixe nenhum.
    let solto = drag_now(&sim, nascimento, [10.0, 90.0], 1.0);
    assert_eq!(solto.splice, None);
    assert_eq!(solto.tip, [10.0, 90.0]);
    // ⭐⭐ **E o LIMIAR mede o osso ENCAIXADO, nunca o ponteiro.** Um arrasto que ainda está curto
    // até à base do alvo não arma, mesmo com a mão já longe da origem — e vice-versa.
    let perto = create(&mut sim, None, [14.0, 0.0], [54.0, 0.0]).expect("C, a 4 da origem");
    // O ponteiro a `13` da origem (armaria) e a `9` da base de C, dentro do raio `10` dela.
    let curto = drag_now(&sim, nascimento, [23.0, 0.0], 1.0);
    assert_eq!(
        curto.splice.map(|(b, _)| b),
        Some(perto),
        "a fixtura nao produz o fenomeno: o ponteiro tinha de achar a base de C"
    );
    assert!(
        !curto.armed,
        "o encaixe poe a ponta a 4 unidades da origem (limiar 12) e mesmo assim armou — o limiar \
         esta' a medir o PONTEIRO, que fica a 13"
    );
}
