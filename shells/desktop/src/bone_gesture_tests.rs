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
            origin: [5.0, 0.0],
            pick: Some(id)
        },
        "apontar a forma tem de a devolver - sem isto o `Bind` nunca tem sujeito"
    );
    // E um press sobre um OSSO nao e' uma escolha de forma.
    // ⚠️ O ponto é LONGE da raiz de propósito: o raio da junta é `joint_radius_px(24) = 6` a este
    // zoom (o tecto de `comp/4` mandando), e num ponto perto da raiz TODO press seria junta — a
    // fixtura mediria o verbo errado.
    let osso = create(&mut sim, None, [3.0, 4.0], [27.0, 4.0]).expect("osso");
    assert_eq!(
        press(
            &sim,
            &scene,
            &pen,
            [20.0, 4.0],
            1.0,
            None,
            BoneAction::Create
        ),
        BonePress::Select { bone: osso },
        "em CRIAR, tocar num osso so' o ACENDE - e' assim que se escolhe onde ramificar"
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
    let d = press(
        &sim,
        &scene,
        &pen,
        [60.0, 40.0],
        1.0,
        Some(osso),
        BoneAction::Create,
    );
    assert_eq!(
        d,
        BonePress::Start {
            origin: [10.0, 0.0],
            pick: None
        },
        "o filho tem de crescer da ponta do pai"
    );
    // Sem osso aceso: um osso SOLTO nasce onde a mao pousou.
    let f = press(
        &sim,
        &scene,
        &pen,
        [60.0, 40.0],
        1.0,
        None,
        BoneAction::Create,
    );
    assert_eq!(
        f,
        BonePress::Start {
            origin: [60.0, 40.0],
            pick: None
        },
        "sem pai, a origem e' o ponto apontado"
    );
}

/// ⭐⭐⭐ **O REALCE ACENDE EXACTAMENTE O QUE O CLIQUE PEGA** — a lei que o `pick_hovered_object`
/// já declara para as formas (*«um realce que acendesse outra coisa que a que o clique pega seria
/// pior que não haver realce nenhum»*), e que aqui é mais apertada: as duas alças estão **uma
/// dentro da outra** e executam VERBOS diferentes.
///
/// ⚠️ Ele varre o osso de ponta a ponta e compara, ponto a ponto, o que o hover diz com o que o
/// `press` decide. Uma segunda varredura escrita ao lado do realce passaria neste gate só por
/// coincidência — é por isso que o `hover` chama o `hit`/`grabbed_the_joint`, e não uma cópia.
///
/// (Mutação: o `hover` decidir a junta por outro raio ⇒ RED.)
#[test]
fn the_hover_lights_exactly_what_the_click_would_grab() {
    let mut sim = SimWorld::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [100.0, 0.0]).expect("osso");
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    let mut viu_junta = false;
    let mut viu_corpo = false;
    for i in 0..=100 {
        let p = [f64::from(i), 0.0];
        let h = hover(&sim, p, 1.0, Some(osso)).expect("o ponteiro esta' sobre o osso");
        assert_eq!(h.bone, osso);
        let BonePress::Grab { bone, part } = press(
            &sim,
            &scene,
            &pen,
            p,
            1.0,
            Some(osso),
            BoneAction::Transform,
        ) else {
            panic!("o press devia agarrar o osso em {p:?}");
        };
        assert_eq!(
            (h.bone, h.part),
            (bone, part),
            "em {p:?} o realce e o clique discordam - o artista ve' um verbo e recebe outro"
        );
        viu_junta |= h.part == BonePart::Joint;
        viu_corpo |= h.part == BonePart::Body;
    }
    // ⚠️ O controlo: a varredura tem de produzir os DOIS estados, senão o gate compara um lado só.
    assert!(
        viu_junta && viu_corpo,
        "a fixtura nao produziu as duas metades (junta={viu_junta}, corpo={viu_corpo})"
    );
}

/// ⭐⭐ **A JUNTA TEM A TOLERÂNCIA DA CASA, não metade dela** — o report de 2026-09-06 (*«a bolinha
/// e sua área sensível ao mouse precisa ser maior pois está difícil selecioná-la»*).
///
/// O corpo do osso já pedia emprestado o `HANDLE_HIT_PX = 12` do `input_dispatch` *«para o dedo do
/// artista ter sempre a mesma tolerância»*, e a junta ficava com `6` — **o alvo menor por dentro do
/// maior**. Este gate mede o que o dedo alcança, e ⚠️ **mede-o no MESMO número que o desenho usa**:
/// se os dois divergirem, o realce acende num sítio e o clique pega noutro.
///
/// (Mutação: `BONE_JOINT_R_PX` de volta a `6.0` ⇒ RED.)
#[test]
fn the_joint_gets_the_houses_finger_tolerance_and_the_dot_is_that_same_number() {
    let mut sim = SimWorld::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [100.0, 0.0]).expect("osso");
    // A tolerância da casa, declarada no `input_dispatch` e emprestada pelo `BONE_HIT_PX`.
    assert!(
        grabbed_the_joint(Some(&sim), osso, [11.0, 0.0], 1.0),
        "a 11 px da raiz o dedo ainda tem de apanhar a junta - ela recebe a tolerancia da casa (12)"
    );
    assert!(
        !grabbed_the_joint(Some(&sim), osso, [13.0, 0.0], 1.0),
        "a 13 px a junta ja' acabou, senao ela come o corpo"
    );
    // ⚠️ E o alvo é o DESENHO: uma segunda constante aqui separaria o dedo do olho.
    assert!(
        (ph2d_skeleton_render::joint_radius_px(100.0) - 12.0).abs() < 1e-12,
        "o raio DESENHADO deixou de ser o mesmo que o dedo procura"
    );
}

/// ⛔ **Num osso CURTO a junta encolhe, e o recurso é o verbo de GIRAR.** Duas juntas de raio `r`
/// comem `2r` do comprimento; sem tecto, um osso curto fica todo junta e não há onde agarrar para
/// rodar — a cerca que o `grabbed_the_joint` já declarava por escrito.
///
/// (Mutação: tirar o `.min(comp * 0.25)` ⇒ RED.)
#[test]
fn a_short_bone_keeps_half_of_itself_grabbable_for_rotation() {
    let mut sim = SimWorld::default();
    let curto = create(&mut sim, None, [0.0, 0.0], [20.0, 0.0]).expect("curto");
    // Raio 5 (20/4) ⇒ o meio do osso, a 10, é CORPO e não junta.
    assert!(
        !grabbed_the_joint(Some(&sim), curto, [10.0, 0.0], 1.0),
        "num osso de 20 px a junta chegou ao meio - nao sobra corpo para girar"
    );
    assert!(
        grabbed_the_joint(Some(&sim), curto, [4.0, 0.0], 1.0),
        "a junta encolheu demais e deixou de ser agarravel"
    );
    // A lei, nos dois lados da dobra: longo ⇒ o tecto da casa; curto ⇒ um quarto do comprimento.
    assert!((ph2d_skeleton_render::joint_radius_px(200.0) - 12.0).abs() < 1e-12);
    assert!((ph2d_skeleton_render::joint_radius_px(20.0) - 5.0).abs() < 1e-12);
}

/// ⛔ **A ALÇA DA FORÇA SÓ É AGARRÁVEL ONDE ELA É PINTADA** — no osso em FOCO, e em mais nenhum.
///
/// ⚠️ *Uma alça agarrável onde nada está desenhado é pior que uma alça ausente*: o artista carrega
/// no vazio e o app faz uma coisa que ele não pediu. O `draw_influence` pinta a região de UM osso
/// (a selecção) e o `hover` recebe esse mesmo `foco` — este gate afirma que os dois concordam.
/// ⚠️⚠️ **A 1.ª redacção deste gate SOBREVIVEU à mutação** (pôr a alça em todo osso apanhado, e não
/// só no do foco), e a razão é a 3.ª leitura da memória: *a fixtura não produzia o fenómeno*. Ela
/// punha a alça a `20` unidades do eixo — fora do raio de acerto do CORPO (`12`) —, então sem foco
/// o `hit` também devolvia `None` e os dois lados concordavam por acidente.
///
/// ⇒ a força desce para `0,3` (raio `6`), e a alça passa a cair **dentro** do corpo. Aí as duas
/// respostas divergem: com foco é `Influence`, sem foco é `Body`. É esse par que o gate mede.
#[test]
fn the_strength_handle_exists_only_on_the_focused_bone() {
    let mut sim = SimWorld::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [20.0, 0.0]).expect("osso");
    {
        let mut b = sim
            .world_mut()
            .get_mut::<ph2d_skeleton_ecs::Bone>(Entity::from_bits(osso))
            .expect("Bone");
        b.strength = 0.3; // raio 6 ⇒ a alça cai DENTRO do raio de acerto do corpo (12)
    }
    // Meio do osso (10,0) mais 6 na perpendicular.
    let alca = [10.0, 6.0];
    assert_eq!(
        hover(&sim, alca, 1.0, Some(osso)).map(|h| h.part),
        Some(BonePart::Influence),
        "com o osso em foco a alca tem de ganhar o ponto"
    );
    assert_eq!(
        hover(&sim, alca, 1.0, None).map(|h| h.part),
        Some(BonePart::Body),
        "sem foco nao ha' mancha desenhada - o mesmo ponto tem de ser CORPO, e o clique girar"
    );
    // E o caso longe continua a valer: sem foco, nem sequer há osso ali.
    {
        let mut b = sim
            .world_mut()
            .get_mut::<ph2d_skeleton_ecs::Bone>(Entity::from_bits(osso))
            .expect("Bone");
        b.strength = 1.0;
    }
    assert_eq!(hover(&sim, [10.0, 20.0], 1.0, None), None);
}

/// ⛔ **A ALÇA DA PONTA SÓ EXISTE EM QUEM FECHA A CORRENTE.** Numa junta interior a ponta de um
/// osso **é** a raiz do seguinte, e ali já há uma bolinha com outro verbo (deslocar) — duas alças
/// no mesmo pixel a fazer coisas diferentes é o defeito que o realce por parte existe para evitar.
#[test]
fn only_the_bone_that_closes_a_chain_offers_the_end_effector() {
    let mut sim = SimWorld::default();
    let ossos = test_chain(&mut sim, 3);
    // A ponta do 1.º osso (10,0) é a raiz do 2.º: ali NÃO há alça de ponta.
    assert_eq!(
        hover(&sim, [10.0, 0.0], 1.0, None).map(|h| h.part),
        Some(BonePart::Joint),
        "numa junta interior o verbo e' DESLOCAR, nao IK"
    );
    // A ponta do último (30,0) fecha a corrente.
    assert_eq!(
        hover(&sim, [30.0, 0.0], 1.0, None).map(|h| h.part),
        Some(BonePart::Tip),
        "a ponta da corrente tem de oferecer o end effector"
    );
    assert_eq!(crate::skeleton_live::chain_ends(&sim), vec![ossos[2]]);
}

/// ⭐⭐⭐ **UM GESTO, UM VERBO** — o report de 2026-09-07 (*«do modo como está fica confuso para o
/// usuário»*), dito como decisão.
///
/// ⛔ **Antes a ambiguidade era do PONTEIRO:** o MESMO arrasto criava ou posava consoante o que
/// estava por baixo do cursor. Isso torna **inalcançáveis** dois gestos legítimos — começar um osso
/// *em cima* de outro, e posar um osso *sem medo* de criar um por engano — e obriga o artista a
/// saber o que está debaixo do cursor antes de carregar.
///
/// Este gate mede as **quatro** células: dois pontos (sobre osso · no vazio) × dois verbos.
///
/// (Mutação: o `press` ignorar o `action` ⇒ RED em duas delas.)
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
        BonePress::Select { bone: osso },
        "CRIAR sobre um osso: so' acende, e o arrasto seguinte faz um filho"
    );
    assert_eq!(
        press(&sim, &scene, &pen, sobre, 1.0, None, BoneAction::Transform),
        BonePress::Grab {
            bone: osso,
            part: BonePart::Body
        },
        "TRANSFORMAR sobre um osso: agarra e posa"
    );
    // NO VAZIO, longe de tudo.
    let vazio = [200.0, 200.0];
    assert_eq!(
        press(&sim, &scene, &pen, vazio, 1.0, None, BoneAction::Create),
        BonePress::Start {
            origin: vazio,
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

/// ⭐⭐⭐ **QUE ALÇAS PEGAM FORA DO MODO OSSO — a linha é o VERBO, não a alça.**
///
/// ⛔⛔ **Achado da auditoria de 2026-09-08:** as alças do osso são pintadas e **acendem sob o rato
/// nos 14 modos de vector**, e o `Down` só era lido dentro do `DrawMode::Bone` ⇒ o artista via o
/// arco de limite acender e arrastá-lo não fazia nada.
///
/// ⚠️ **O critério não é «é uma alça», é «este verbo existe noutra ferramenta?»** — girar e
/// deslocar um osso o gizmo de sprite já faz, e roubar-lhos aqui trocaria a lei do arrasto da seta
/// em silêncio; a força, as duas paredes do limite e a cinemática inversa **não têm outra porta**.
#[test]
fn only_the_verbs_no_other_tool_can_express_are_grabbed_outside_bone_mode() {
    use crate::bone_gesture::grabbable_outside_bone_mode as pega;
    use ph2d_skeleton_render::BonePart;
    for (parte, porque) in [
        (BonePart::Influence, "a forca nao tem outra porta"),
        (
            BonePart::LimitMin,
            "a parede horaria do limite nao tem outra porta",
        ),
        (
            BonePart::LimitMax,
            "a parede anti-horaria do limite nao tem outra porta",
        ),
        (BonePart::Tip, "a cinematica inversa nao tem outra porta"),
    ] {
        assert!(
            pega(parte),
            "{parte:?} acende sob o dedo em todo modo e nao pegaria: {porque}"
        );
    }
    for (parte, quem) in [
        (BonePart::Body, "o gizmo de sprite ja' GIRA"),
        (BonePart::Joint, "o gizmo de sprite ja' DESLOCA"),
    ] {
        assert!(
            !pega(parte),
            "{parte:?} passaria a roubar o arrasto da seta: {quem}"
        );
    }
}
