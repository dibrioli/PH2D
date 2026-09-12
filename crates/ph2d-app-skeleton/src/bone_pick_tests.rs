//! Os gates de **o que o ponteiro APONTA** — o realce contra o que o clique de facto pega.
//!
//! ⚠️ Eles saíram do `bone_gesture_tests.rs` em 2026-09-09, no mesmo corte que separou
//! [`crate::bone_pick`] do [`crate::bone_gesture`]: *a régua mora ao lado da lei que ela mede*.
//!
//! ⚠️ **As fixturas partilhadas (`test_chain`, `test_segment`) ficaram no módulo do GESTO**, que é
//! quem as produz — uma cópia por ficheiro divergiria no primeiro ajuste.

use crate::bone_gesture::{BonePress, create, press, test_chain};
use crate::bone_pick::{free_root_at, grabbed_the_joint, hit, hover, is_a_free_chain_root, tip_at};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_render::BonePart;
use ph2d_tool_vector::BoneAction;

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

/// ⭐⭐⭐ **DUAS PONTAS SOB O DEDO: GANHA A MAIS PERTO, e não a primeira da lista.**
///
/// ⛔ O raio de uma ponta é o da bolinha DESENHADA, logo cresce com o comprimento do osso: um osso
/// longo oferece a ponta dele a `12 px` de distância, e um osso curto pendurado nela cabe inteiro
/// dentro desse raio. Por ordem de varredura, o pai — criado primeiro — ganharia sempre, e ramificar
/// da ponta do FILHO curto seria inexprimível.
///
/// ⚠️ É a mesma regra que as três alças do osso em foco já pagaram: *ganha o que está mais perto do
/// dedo* é a única que não escolhe uma vítima.
///
/// ⚠️⚠️ **Ele corre as DUAS ORDENS DE CRIAÇÃO, e a 1.ª redacção não corria — a mutação SOBREVIVEU.**
/// Medido: o `Entity::to_bits` **desce** a cada entidade nova (`4294967294`, depois `4294967293`) e
/// o `bone_segments` ordena por bits **crescentes** ⇒ o último osso criado vem **primeiro** na
/// lista. Com o curto criado por último, «o primeiro dentro do raio» e «o mais perto» davam a mesma
/// resposta, e a fixtura media uma coincidência. *Uma ordem que só coincide com a certa não é a
/// certa* — e é a lei que este ficheiro já tinha pago com as alças testadas por ordem.
///
/// (Mutação: o `tip_at` devolver o primeiro dentro do raio ⇒ RED numa das duas ordens.)
#[test]
fn when_two_tips_are_under_the_finger_the_nearest_one_wins() {
    // ⚠️ `curto_primeiro` diz qual dos dois é CRIADO antes — e é só isso que muda a posição deles na
    // lista que o `tip_at` varre. A geometria é idêntica nas duas células.
    for curto_primeiro in [false, true] {
        let mut sim = SimWorld::default();
        let faz_longo = |s: &mut SimWorld| create(s, None, [0.0, 0.0], [40.0, 0.0]);
        let faz_curto = |s: &mut SimWorld| create(s, None, [40.0, 0.0], [45.0, 0.0]);
        let (longo, curto) = if curto_primeiro {
            let c = faz_curto(&mut sim).expect("curto");
            (faz_longo(&mut sim).expect("longo"), c)
        } else {
            let l = faz_longo(&mut sim).expect("longo");
            (l, faz_curto(&mut sim).expect("curto"))
        };
        // Em (45,0) as DUAS pontas estão dentro do raio DESENHADO: a do longo a `5` de `10`
        // (`joint_radius_px(40)`), a do curto a `0` de `1,25` (`joint_radius_px(5)`).
        assert_eq!(
            tip_at(&sim, [45.0, 0.0], 1.0).map(|(b, _)| b),
            Some(curto),
            "(curto_primeiro={curto_primeiro}) a ponta do osso CURTO esta' debaixo do dedo e perdeu \
             para a do longo — ramificar dela seria inexprimivel"
        );
        // E a `5` unidades para trás só a do longo cabe.
        assert_eq!(
            tip_at(&sim, [40.0, 0.0], 1.0).map(|(b, _)| b),
            Some(longo),
            "(curto_primeiro={curto_primeiro}) na ponta do longo o alvo tem de ser ele"
        );
    }
}

/// ⭐⭐⭐ **EM *CRIAR*, O REALCE ACENDE EXACTAMENTE O OSSO DE QUE O CLIQUE VAI RAMIFICAR.**
///
/// ⛔⛔ **Sem esta lei o realce acenderia o osso ERRADO no ponto que decide o parentesco:** numa
/// corrente contínua a ponta do osso `k` é a raiz do osso `k+1`, e o `hover` de *Transformar*
/// responde ali *«a JUNTA do `k+1`»* — o artista veria acender o filho e o press ramificaria do pai.
///
/// ⇒ os dois saem da MESMA porta ([`tip_at`]), e este gate varre a corrente inteira a comparar. Uma
/// segunda varredura ao lado do realce passaria aqui só por coincidência.
///
/// (Mutação: o `hover` não ramificar por `action` ⇒ RED na primeira junta interior.)
#[test]
fn in_create_the_hover_lights_exactly_the_bone_the_click_would_branch_from() {
    let mut sim = SimWorld::default();
    let scene = ph2d_vec_scene::VecScene::new();
    let pen = ph2d_vec_edit::PenTool::default();
    let ossos = test_chain(&mut sim, 3);
    let mut viu_ponta = 0;
    // Varre o eixo da corrente inteira, para lá dela, meia unidade de cada vez.
    for i in 0..=80 {
        let p = [f64::from(i) * 0.5, 0.0];
        let h = hover(&sim, p, 1.0, Some(ossos[2]), BoneAction::Create);
        let BonePress::Start { birth, .. } = press(
            &sim,
            &scene,
            &pen,
            p,
            1.0,
            Some(ossos[2]),
            BoneAction::Create,
        ) else {
            panic!("em CRIAR todo press arma um osso — em {p:?} nao armou");
        };
        assert_eq!(
            h.map(|x| x.bone),
            birth.parent,
            "em {p:?} o realce e o parentesco discordam — o artista ve' acender um osso e o filho \
             nasce de outro"
        );
        if let Some(x) = h {
            assert_eq!(
                x.part,
                BonePart::Tip,
                "em CRIAR o unico alvo e' a PONTA — acender outra alca promete um verbo que este \
                 modo nao executa"
            );
            viu_ponta += 1;
        }
    }
    assert!(
        viu_ponta >= 3,
        "a varredura acendeu {viu_ponta} pontas numa corrente de 3 — a fixtura nao produz o \
         fenomeno que o gate mede"
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
        let h = hover(&sim, p, 1.0, Some(osso), BoneAction::Transform)
            .expect("o ponteiro esta' sobre o osso");
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
        hover(&sim, alca, 1.0, Some(osso), BoneAction::Transform).map(|h| h.part),
        Some(BonePart::Influence),
        "com o osso em foco a alca tem de ganhar o ponto"
    );
    assert_eq!(
        hover(&sim, alca, 1.0, None, BoneAction::Transform).map(|h| h.part),
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
    assert_eq!(
        hover(&sim, [10.0, 20.0], 1.0, None, BoneAction::Transform),
        None
    );
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
        hover(&sim, [10.0, 0.0], 1.0, None, BoneAction::Transform).map(|h| h.part),
        Some(BonePart::Joint),
        "numa junta interior o verbo e' DESLOCAR, nao IK"
    );
    // A ponta do último (30,0) fecha a corrente.
    assert_eq!(
        hover(&sim, [30.0, 0.0], 1.0, None, BoneAction::Transform).map(|h| h.part),
        Some(BonePart::Tip),
        "a ponta da corrente tem de oferecer o end effector"
    );
    assert_eq!(ph2d_skeleton_live::skin_live::chain_ends(&sim), vec![ossos[2]]);
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
    use crate::bone_pick::grabbable_outside_bone_mode as pega;
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

/// ⭐⭐⭐ **SÓ UMA CORRENTE QUE COMEÇA SOLTA OFERECE A BASE PARA A EMENDA** (ordem do dono,
/// 2026-09-09).
///
/// ⛔⛔ **A base de um osso do MEIO é, no mesmo pixel, a PONTA do pai dele** — e ali a lei da ponta
/// já fala (*«daqui nasce um filho»*). Oferecer as duas coisas no mesmo ponto reabre exactamente a
/// ambiguidade que a lei da ponta veio curar, um nível acima.
///
/// ⇒ este gate mede as duas leituras **no mesmo ponto**: a ponta responde, a base cala-se.
///
/// (Mutação: o `free_root_at` deixar de filtrar por `is_a_free_chain_root` ⇒ RED.)
#[test]
fn only_a_chain_that_starts_free_offers_its_base_for_a_splice() {
    let mut sim = SimWorld::default();
    let ossos = test_chain(&mut sim, 3);
    // A base do PRIMEIRO é livre — ele abre a corrente.
    assert_eq!(
        free_root_at(&sim, [0.0, 0.0], 1.0).map(|(b, _)| b),
        Some(ossos[0]),
        "a base da corrente tem de se oferecer — e' ela que a emenda adopta"
    );
    // Em (10,0) vivem a PONTA do 1º e a BASE do 2º. Só a ponta responde.
    assert_eq!(
        tip_at(&sim, [10.0, 0.0], 1.0).map(|(b, _)| b),
        Some(ossos[0]),
        "a ponta do 1o osso continua a ser a porta do parentesco"
    );
    assert_eq!(
        free_root_at(&sim, [10.0, 0.0], 1.0),
        None,
        "a base de um osso do MEIO nao pode oferecer-se: ela E' a ponta do pai, no mesmo pixel, e \
         dois verbos num pixel e' o defeito que a lei da ponta veio curar"
    );
    assert!(is_a_free_chain_root(&sim, ossos[0]));
    assert!(!is_a_free_chain_root(&sim, ossos[1]));
}
