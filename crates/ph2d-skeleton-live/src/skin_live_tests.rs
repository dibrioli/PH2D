//! Os gates do [`crate::skin_live`] — o esqueleto como HOST.
//!
//! A **lei** da mistura (peso, órfão, C¹, escala) é do MÓDULO (`ph2d-skeleton`) e está gateada lá. Aqui
//! mede-se o que só existe com um mundo ECS: prender não move nada · dobrar um osso dobra o desenho
//! · a hierarquia É a cinemática (mover o pai leva o filho) · apagar um osso não apaga a forma · os
//! dois verbos de soltar · e um segundo esqueleto não é apanhado por engano.

use super::*;
// ⭐ UMA definição: os dois auxiliares vivem no `test_support` da crate, porque o gate da
// sequência do smoke (na shell) também os usa.
use crate::test_support::{pior_desvio, quadro};
use ph2d_ecs::{ChildOf, Name, RootOrder, Transform};
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

/// Uma cena com UMA forma (um rectângulo deitado de `(0,0)` a `(40,10)`) e um esqueleto de dois
/// ossos ao longo dela. Devolve `(sim, scene, map, id, [osso_raiz, osso_ponta])`.
fn palco() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    (sim, scene, map, id, [raiz, ponta])
}

/// Um osso em `pos` (local do pai), comprimento `len`, força 1.
fn osso(sim: &mut SimWorld, nome: &str, pos: [f32; 2], len: f64, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                ..Transform::IDENTITY
            },
            Name::new(nome),
            RootOrder(0),
            Bone {
                length: len,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// Gira um osso, em graus.
fn gira(sim: &mut SimWorld, e: Entity, graus: f32) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .rotation = graus.to_radians();
}

/// ⭐⭐⭐ **PRENDER NÃO MOVE UM PIXEL.** A pose de repouso é a identidade por construção (doc 47
/// §2.5), então o artista carrega em *Bind* e **nada acontece** — que é exactamente o que ele
/// espera. Uma pele que salta ao ligar é o defeito nº 1 de todo pacote de rig.
#[test]
fn binding_a_shape_moves_nothing() {
    let (mut sim, mut scene, map, id, _) = palco();
    let antes = scene.paths()[0].clone();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio(&antes, &depois);
    assert!(
        pior < 1e-9,
        "prender moveu a forma em {pior} - o `rest` nao esta' a ser o composto `S-1 . B`"
    );
}

/// ⭐⭐⭐ **DOBRAR UM OSSO DOBRA O DESENHO** — a razão de existir da wave.
///
/// A ponta do rectângulo está dentro do alcance do 2.º osso e longe do 1.º; girar o 2.º levanta-a e
/// **deixa a base onde estava**. É a diferença entre um personagem e um recorte de papel.
#[test]
fn bending_a_bone_bends_the_drawing() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    let repouso = quadro(&sim, &mut scene, id);
    gira(&mut sim, ossos[1], 60.0);
    let dobrado = quadro(&sim, &mut scene, id);

    let dir = |p: &VecPath| {
        p.verts_all()
            .map(|v| v.anchor)
            .filter(|a| a[0] > 30.0)
            .fold(0.0_f64, |m, a| m.max(a[1]))
    };
    let esq = |p: &VecPath| {
        p.verts_all()
            .map(|v| v.anchor)
            .filter(|a| a[0] < 2.0)
            .fold(0.0_f64, |m, a| m.max(a[1]))
    };
    assert!(
        dir(&dobrado) - dir(&repouso) > 3.0,
        "a ponta nao subiu: {} -> {}",
        dir(&repouso),
        dir(&dobrado)
    );
    assert!(
        (esq(&dobrado) - esq(&repouso)).abs() < 0.5,
        "a BASE mexeu-se ({} -> {}) - o osso da ponta esta' a mandar em toda a forma",
        esq(&repouso),
        esq(&dobrado)
    );
}

/// ⭐⭐ **A HIERARQUIA É A CINEMÁTICA** — girar o osso-pai leva o filho, sem uma linha de FK escrita
/// nesta linha de código. É o `propagate_transforms` da casa a fazer o trabalho, e é a razão de um
/// osso ser uma ENTIDADE (doc 47 §2.1).
///
/// ⚠️ Se alguém trocar a árvore por uma lista dentro de um componente, é este gate que fica
/// vermelho.
#[test]
fn turning_the_parent_bone_carries_the_child_because_the_tree_is_the_kinematics() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    let repouso = quadro(&sim, &mut scene, id);
    gira(&mut sim, ossos[0], 40.0);
    let posado = quadro(&sim, &mut scene, id);
    let ponta = |p: &VecPath| {
        p.verts_all()
            .map(|v| v.anchor)
            .fold(0.0_f64, |m, a| m.max(a[1]))
    };
    assert!(
        ponta(&posado) - ponta(&repouso) > 8.0,
        "girar a RAIZ tinha de levar a forma inteira: {} -> {}",
        ponta(&repouso),
        ponta(&posado)
    );
}

/// ⛔ **APAGAR UM OSSO NÃO APAGA A FORMA.** O que resta renormaliza-se sozinho; a forma continua a
/// desenhar-se. Sem isto, um `Delete` na Hierarquia colapsaria o desenho na origem — sem uma linha
/// de erro, porque a soma dos pesos seria zero.
#[test]
fn deleting_a_bone_does_not_delete_the_drawing() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    let antes = quadro(&sim, &mut scene, id);
    sim.world_mut().entity_mut(ossos[1]).despawn();
    let depois = quadro(&sim, &mut scene, id);
    assert!(
        depois.verts_all().all(|v| v.anchor[0].is_finite()),
        "a forma virou NaN ao perder um osso"
    );
    assert!(
        pior_desvio(&antes, &depois) < 1e-9,
        "apagar um osso EM REPOUSO mudou a forma - a renormalizacao esta' errada"
    );
}

/// **Os dois verbos de soltar.** *Release* devolve o que o artista desenhou; *Expand* fica com o que
/// ele está a ver. Adivinhar qual deles ele quer é que não.
#[test]
fn releasing_gives_back_the_drawing_and_expanding_keeps_the_pose() {
    for (keep, volta) in [(Keep::Source, true), (Keep::Deformed, false)] {
        let (mut sim, mut scene, map, id, ossos) = palco();
        let autorada = scene.paths()[0].clone();
        bind(&mut sim, &scene, &map, &[id], None);
        gira(&mut sim, ossos[1], 60.0);
        let posada = quadro(&sim, &mut scene, id);
        assert_eq!(release(&mut sim, &mut scene, &map, &[id], keep), 1);
        let ficou = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .expect("o path")
            .clone();
        let alvo = if volta { &autorada } else { &posada };
        assert!(
            pior_desvio(alvo, &ficou) < 1e-9,
            "{keep:?} ficou com a geometria errada"
        );
        // E a pele foi-se: o recook seguinte não pode voltar a deformar.
        let depois = quadro(&sim, &mut scene, id);
        assert!(
            pior_desvio(&ficou, &depois) < 1e-9,
            "{keep:?} deixou pele viva"
        );
    }
}

/// ⚠️ **UM SEGUNDO ESQUELETO NÃO É APANHADO POR ENGANO.** Com um osso apontado, o Bind leva a árvore
/// DELE; sem nenhum, leva tudo (a leitura certa de *"há um esqueleto só"*, que é o caso comum).
#[test]
fn a_second_skeleton_is_only_bound_when_it_is_the_one_pointed_at() {
    let (mut sim, _scene, _map, _id, ossos) = palco();
    let outro = osso(&mut sim, "Other", [200.0, 0.0], 10.0, None);
    // ⚠️ **Comparado como CONJUNTO**: a ORDEM dentro de uma pele não tem sentido (os pesos
    // normalizam-se), só precisa de ser determinística — e ordenar por `to_bits` é ordenar por id
    // de ALOCAÇÃO, que numa fixtura não é o que se quer afirmar. O que importa é *quem* entra.
    let conjunto = |v: Vec<Entity>| {
        let mut s: Vec<u64> = v.into_iter().map(|e| e.to_bits()).collect();
        s.sort_unstable();
        s
    };
    assert_eq!(
        conjunto(skeleton_of(&sim, Some(ossos[1]))),
        conjunto(vec![ossos[0], ossos[1]])
    );
    assert_eq!(
        conjunto(skeleton_of(&sim, Some(outro))),
        conjunto(vec![outro])
    );
    assert_eq!(
        conjunto(skeleton_of(&sim, None)),
        conjunto(vec![ossos[0], ossos[1], outro]),
        "sem semente, o esqueleto e' a cena"
    );
}

/// ⚠️ **A ORDEM dos ossos numa pele é um RÓTULO, não uma lei** — permutá-la devolve o mesmo desenho.
///
/// É este gate que autoriza `skeleton_of` a ordenar por `to_bits`: se a ordem decidisse alguma
/// coisa, ordenar por id de ALOCAÇÃO seria o defeito que o `CLAUDE.md` §5 nomeia. O que ela decide
/// é só a ordem da soma em `f64`, e a medida está aqui.
///
/// ⚠️⚠️ **A permutação passou a ter DUAS metades em 2026-09-15, e a 1.ª redacção só mexia numa.**
/// Enquanto os pesos eram DERIVADOS de uma distância, a lista de tendões era a única coisa
/// ordenada; com os pesos do padrão-ouro GUARDADOS, a coluna `j` da tabela **é** o tendão `j`.
/// Reverter os tendões sem reverter as colunas não permuta nada — produz **dados incoerentes**, e
/// o gate lia `18,5` de desvio a acusar um defeito que não existe. *É o mesmo erro que reverter os
/// triângulos de uma malha sem reverter os vértices.*
///
/// ⇒ ele permuta **as duas metades**, que é o que «permutar os ossos» quer dizer desde que a tabela
/// existe.
#[test]
fn the_order_of_the_bones_in_a_skin_does_not_change_the_drawing() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    bind(&mut sim, &scene, &map, &[id], None);
    gira(&mut sim, ossos[1], 55.0);
    let direita = quadro(&sim, &mut scene, id);

    let e = Entity::from_bits(map[&id]);
    let mut skin = sim
        .world_mut()
        .get_mut::<SkinBind>(e)
        .expect("a pele")
        .clone();
    skin.tendons.reverse();
    // E as COLUNAS da tabela com eles — as duas metades do mesmo facto.
    if let Some(mut g) = crate::skinned_mesh::le(&skin.source)
        && g.ossos() > 0
    {
        let n = g.ossos();
        let revertidos: Vec<f64> = g
            .pesos
            .chunks_exact(n)
            .flat_map(|c| c.iter().rev().copied().collect::<Vec<_>>())
            .collect();
        g.pesos = revertidos;
        skin.source = postcard::to_allocvec(&g).expect("serializa");
    }
    sim.world_mut().entity_mut(e).insert(skin);
    let avessa = quadro(&sim, &mut scene, id);
    let pior = pior_desvio(&direita, &avessa);
    assert!(
        pior < 1e-12,
        "permutar os ossos moveu a forma em {pior} - a ordem esta' a decidir algo"
    );
}

/// ⭐⭐⭐ **PRENDER À CENA INTEIRA DÁ O MESMO DESENHO QUE PRENDER AO ESQUELETO CERTO** — e é isto
/// que autoriza o `Bind` sem cerimónia (o botão prende a tudo quando o artista não apontou um osso).
///
/// ⚠️ **A propriedade é do SUPORTE FINITO, não da bondade da implementação:** um osso longe está
/// fora do raio de todo ponto da forma ⇒ peso `0` ⇒ a normalização devolve exactamente os mesmos
/// números; e um ponto órfão prende-se ao mais PRÓXIMO, que é do esqueleto certo. ⛔ Com a lei
/// global (`1/d²`) isto seria FALSO, e o segundo esqueleto arrastaria a forma um pouco.
#[test]
fn binding_to_the_whole_scene_draws_the_same_as_binding_to_the_right_skeleton() {
    let desenho = |semente: bool| {
        let (mut sim, mut scene, map, id, ossos) = palco();
        // Um segundo esqueleto, LONGE — o que o `Bind` sem semente também apanharia.
        let outro = osso(&mut sim, "Far", [400.0, 0.0], 30.0, None);
        osso(&mut sim, "Far2", [30.0, 0.0], 30.0, Some(outro));
        let raiz = semente.then_some(ossos[0]);
        assert_eq!(bind(&mut sim, &scene, &map, &[id], raiz), 1);
        gira(&mut sim, ossos[1], 50.0);
        quadro(&sim, &mut scene, id)
    };
    let pior = pior_desvio(&desenho(true), &desenho(false));
    assert!(
        pior < 1e-12,
        "o esqueleto LONGE mudou o desenho em {pior} - o suporte deixou de ser finito"
    );
}

/// **Os ossos que o overlay desenha saem da POSE, não do que foi escrito.** Um osso filho herda a
/// pose do pai — se este gate ficar vermelho, o artista vê o osso num sítio e a forma dobra noutro.
#[test]
fn the_drawn_bone_is_where_the_hierarchy_puts_it() {
    let (mut sim, _scene, _map, _id, ossos) = palco();
    gira(&mut sim, ossos[0], 90.0);
    let segs = bone_segments(&sim);
    let (_, a, b) = segs
        .iter()
        .find(|(bits, _, _)| *bits == ossos[1].to_bits())
        .copied()
        .expect("o osso da ponta");
    // Raiz em (0,5) girada 90°: a ponta dela vai para (0,25); o filho sai dali para cima.
    //
    // ⚠️ **A barra é `1e-5` porque a ROTAÇÃO é `f32`** (o `Transform` da casa), e `cos(90°)` em
    // `f32` vale `−4,37e-8`; a 45 unidades de distância isso lê-se como `−1,75e-6`. *Uma barra de
    // `1e-6` aqui não mediria o produto, mediria o tipo do campo de rotação.*
    assert!(
        a[0].abs() < 1e-5 && (a[1] - 25.0).abs() < 1e-5,
        "a origem do osso filho ficou em {a:?}"
    );
    assert!(
        b[0].abs() < 1e-5 && (b[1] - 45.0).abs() < 1e-5,
        "a ponta do osso filho ficou em {b:?}"
    );
}

/// ⭐⭐⭐ **A PELE ATRAVESSA O DESFAZER** — o gate que separa *"a pele existe"* de *"o app a tem"*.
///
/// Irmão do `field3d_snapshot_tests::the_whole_part_survives_the_world_snapshot_round_trip`, e pela
/// mesma razão: o undo e o salvar são **a mesma máquina** (`world_to_snapshot` → `snapshot_to_world`),
/// e ela **re-spawna** o mundo. Uma referência guardada que não sobreviva ao respawn morre **em
/// silêncio** — a forma fica com a última geometria boa e deixa de responder aos ossos.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate restaurava para um `SimWorld::new()` e passava** — porque num
/// mundo virgem o alocador do bevy entrega os mesmos índices na mesma ordem, e os bits **coincidem
/// por acaso**. O [`crate::undo::ProjectState::restore`] faz o contrário e diz-o à letra: ele
/// *despawna* as entidades editáveis e re-spawna **no mesmo mundo** — *"ids do mundo são novos"* —,
/// e é aí que a geração sobe. *Uma fixtura que não produz o fenómeno devolve verde sobre o defeito.*
#[test]
fn a_skin_survives_the_respawn_that_undo_and_save_do() {
    use ph2d_ecs::scene::{
        ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world,
        world_to_snapshot,
    };
    use ph2d_ecs::{TransformPropagationState, WorklistBuf};

    let (mut sim, scene, map, id, _) = palco();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);

    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_skeleton_ecs::register_skeleton_components(&mut reg);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut snap)
        .expect("o snapshot so' falha se um componente registado nao (de)serializa");

    // O caminho do `ProjectState::restore`: despawna o editável e re-spawna NO MESMO mundo.
    let editaveis: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<Transform>().is_some())
        .map(|er| er.id())
        .collect();
    for e in editaveis {
        let _ = sim.world_mut().despawn(e);
    }
    let mut renasceu = sim;
    snapshot_to_world(renasceu.world_mut(), &snap, &reg).expect("restaura");

    let peles: Vec<SkinBind> = renasceu
        .world()
        .iter_entities()
        .filter_map(|er| er.get::<SkinBind>().cloned())
        .collect();
    assert_eq!(peles.len(), 1, "a pele nao atravessou o snapshot");
    let ossos_vivos = renasceu
        .world()
        .iter_entities()
        .filter(|er| er.get::<Bone>().is_some())
        .count();
    assert_eq!(ossos_vivos, 2, "os ossos nao atravessaram o snapshot");

    let vivos = bone_index(&renasceu);
    let resolvidos = peles[0]
        .tendons
        .iter()
        .filter(|t| {
            vivos
                .get(&t.bone)
                .is_some_and(|&e| renasceu.world().get::<Bone>(e).is_some())
        })
        .count();
    assert_eq!(
        resolvidos,
        peles[0].tendons.len(),
        "{resolvidos} de {} tendoes ainda nomeiam um osso vivo depois do respawn - a pele morreu \
         em silencio, e o sintoma e' 'a forma deixou de seguir o esqueleto depois do Ctrl+Z'",
        peles[0].tendons.len()
    );
}

/// ⭐⭐⭐ **UM RIG JÁ AUTORADO NÃO MUDOU UM BIT** — o gate de regressão do *bendy bone*, do lado do
/// PRODUTOR.
///
/// ⚠️ **A referência não é um golden escrito à mão** (isso mediria a minha aritmética): é a mesma
/// pele construída pela porta ANTIGA, `SkinBone::new`, osso a osso. O que se afirma é que as duas
/// portas **coincidem**, e é o que autoriza a `resolve_with` a chamar sempre a nova.
#[test]
fn a_rig_authored_before_bendy_bones_resolves_to_exactly_the_same_skin() {
    let (mut sim, scene, map, id, _) = palco();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let forma = Entity::from_bits(map[&id]);
    let skin = sim
        .world()
        .get::<SkinBind>(forma)
        .expect("a forma esta' presa")
        .clone();
    let index = bone_index(&sim);
    let pele = resolve(&sim, &skin, forma, &index).expect("a pele resolve");

    let shape_inv = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(&sim, forma),
    )
    .inverse()
    .expect("a forma nao e' singular");
    let antiga: Vec<SkinBone> = skin
        .tendons
        .iter()
        .enumerate()
        .filter_map(|(j, b)| {
            let e = *index.get(&b.bone)?;
            let vb = sim.world().get::<Bone>(e).copied()?;
            let mundo = ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(&sim, e),
            );
            // ⚠️ **O `tendon` é escriturado pelo ORÁCULO também, e de propósito.** Ele não é
            // geometria — é *«de que osso autorado esta pose é»*, e a `SkinBone::new` não tem como
            // o saber (ela constrói um osso sozinho). O que este gate afirma é que as duas portas
            // dão a mesma DEFORMAÇÃO; branquear o campo aqui seria fazer o gate mentir sobre o
            // que compara, e cravá-lo a `0` faria o oráculo descrever um rig de um osso só.
            #[expect(clippy::cast_possible_truncation, reason = "o palco tem dois ossos")]
            Some(SkinBone {
                tendon: j as u32,
                ..SkinBone::new(Xform(b.rest), vb.length, vb.strength, mundo, shape_inv)?
            })
        })
        .collect();
    assert_eq!(antiga.len(), 2, "o palco tem dois ossos");
    assert_eq!(pele.bones(), antiga.as_slice());
}

/// ⭐⭐⭐ **UM OSSO COM CURVATURA PRODUZ N SUB-OSSOS, E O DESENHO ARQUEIA** — o controlo positivo do
/// lado do produtor: sem ele, a lei inteira podia estar certa e nunca ser chamada.
///
/// ⚠️ **A régua é a ALTURA DO MIOLO do rectângulo**, e não a da ponta: a curvatura arqueia o corpo
/// e deixa as duas extremidades onde estão (a Bézier acaba na ponta do osso).
#[test]
fn authoring_curvature_makes_the_producer_emit_sub_bones_and_bows_the_art() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let antes = quadro(&sim, &mut scene, id);
    let forma = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(forma).expect("presa").clone();
    assert_eq!(
        resolve(&sim, &skin, forma, &bone_index(&sim))
            .expect("resolve")
            .len(),
        2,
        "recto ⇒ um sub-osso por osso"
    );

    // O artista arqueia o osso da RAIZ.
    {
        let mut b = sim
            .world_mut()
            .get_mut::<Bone>(ossos[0])
            .expect("o osso existe");
        b.segments = 8;
        b.curve = ph2d_skeleton::bend::Bend {
            inn: [0.0, 8.0],
            out: [0.0, 8.0],
        };
    }
    assert_eq!(
        resolve(&sim, &skin, forma, &bone_index(&sim))
            .expect("resolve")
            .len(),
        9,
        "8 sub-ossos do curvo + 1 do recto"
    );

    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio(&antes, &depois);
    assert!(
        pior > 1.0,
        "a curvatura nao chegou ao desenho: pior desvio {pior}"
    );
}

/// ⭐⭐⭐ **AS DUAS MÍDIAS RESPONDEM À MESMA LEI** — é o gate por que esta wave existe.
///
/// ⛔⛔⛔ **Em 2026-09-15 o padrão-ouro entrou só para IMAGENS**, porque ele precisa de uma malha do
/// domínio e uma Bézier não tem uma. O preço foi um rig com **duas leis**: um braço vectorial e um
/// braço em imagem, presos ao mesmo esqueleto, deformavam-se por matemáticas diferentes — e o
/// sintoma seria *«o braço desenhado não acompanha o braço vectorial»*.
///
/// ⚠️ **A régua não é «as duas ficam iguais»** — elas não têm de ficar: uma amostra a arte numa
/// grelha e a outra move pontos de controlo de uma curva. O que se afirma é que **a forma vectorial
/// deixou de responder à lei EUCLIDIANA**, e a prova é directa: a tabela de pesos existe e é
/// coerente com o caminho.
///
/// ⛔ E a metade que impede isto de ser vácuo: um caminho **ABERTO** continua sem tabela, porque não
/// tem interior — e isso é uma resposta, não uma falha.
#[test]
fn as_duas_midias_respondem_a_mesma_lei() {
    let (mut sim, scene, map, id, _) = palco();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let e = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("a pele").clone();
    let g: crate::skinned_mesh::SkinnedPath =
        postcard::from_bytes(&skin.source).expect("a forma guardada");
    assert!(
        g.valida(),
        "a tabela de pesos nao fecha com o caminho ({} pesos para {} pontos de controlo)",
        g.pesos.len(),
        g.pontos()
    );
    assert_eq!(
        g.ossos(),
        skin.tendons.len(),
        "a tabela cobre {} ossos e a pele tem {} tendoes — a coluna `j` deixou de ser o tendao `j`",
        g.ossos(),
        skin.tendons.len()
    );
    // ⭐ E os pesos são uma PARTIÇÃO: cada ponto de controlo soma `1`.
    let n = g.ossos();
    let mut pior = 0.0_f64;
    for c in g.pesos.chunks_exact(n) {
        pior = pior.max((c.iter().sum::<f64>() - 1.0).abs());
    }
    assert!(
        pior < 1e-9,
        "o pior |soma - 1| dos pesos do caminho e' {pior:.2e}: a particao da unidade partiu-se"
    );
}

/// ⛔ **UM CAMINHO ABERTO NÃO TEM INTERIOR, e fica na lei derivada** — o controlo do gate acima.
///
/// ⚠️ Sem ele, o gate irmão passaria com uma lei que inventasse um domínio para qualquer coisa. *Uma
/// linha de construção tem área zero: recusar é a resposta honesta.*
#[test]
fn um_caminho_aberto_fica_na_lei_derivada() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Uma POLILINHA aberta — dois pontos, sem fecho.
    let mut aberto = cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]);
    aberto.closed = false;
    let id = scene.push_path(aberto);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    let e = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("a pele").clone();
    let g: crate::skinned_mesh::SkinnedPath =
        postcard::from_bytes(&skin.source).expect("a forma guardada");
    assert!(
        g.pesos.is_empty(),
        "um caminho ABERTO recebeu {} pesos: alguem inventou um dominio para uma linha",
        g.pesos.len()
    );
    // E o desenho continua a resolver — pela lei derivada.
    let depois = quadro(&sim, &mut scene, id);
    assert!(
        depois.verts_all().all(|v| v.anchor[0].is_finite()),
        "a forma aberta virou NaN ao cair na lei derivada"
    );
}

/// ⭐⭐⭐ **O palco com VÉRTICES NA JUNTA** — e ele existe porque o [`palco`] não distingue as leis.
///
/// ⛔⛔ **O rectângulo do [`palco`] tem os quatro pontos de controlo nos CANTOS, e ali as duas leis
/// concordam** — a diferença entre elas vive **junto da junta**, e um rectângulo de `40 × 10` com a
/// junta a meio não tem vértice nenhum lá. Medido: a separação entre as duas leis é `0,000`, e o
/// gate que as compara não afirmava nada.
///
/// ⇒ este palco põe seis vértices, **dois deles exactamente sobre a junta**, e a arte é **ALTA**
/// (`40 × 30` contra `40 × 10`): a aresta de cima fica a `25` do eixo, para lá do raio de `20` do
/// *bump* — que é onde a lei antiga degenera numa partição dura e as duas mais se afastam.
/// *Uma fixtura que não produz o fenómeno mede outro programa.*
fn palco_com_vertices_na_junta() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let path = VecPath {
        verts: [
            [0.0, 0.0],
            [20.0, 0.0],
            [40.0, 0.0],
            [40.0, 30.0],
            [20.0, 30.0],
            [0.0, 30.0],
        ]
        .into_iter()
        .map(ph2d_vec_scene::VecVertex::corner)
        .collect(),
        closed: true,
        ..VecPath::default()
    };
    let id = scene.push_path(path);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    (sim, scene, map, id, [raiz, ponta])
}

/// ⭐⭐⭐ **A TABELA CHEGA AO DESENHO** — e este gate existe porque uma mutação SOBREVIVEU.
///
/// ⛔⛔⛔ Os dois gates acima provam que a tabela **existe** e que ela **fecha** com o caminho; pôr
/// `usa = false` no [`ph2d_vec_skin::aplica_com`] — isto é, o vector a ignorar os pesos guardados e
/// a cair na lei derivada — deixava **os 29 verdes**. *É o terceiro passo que um `grep` não vê: o
/// painel escreve · alguém lê · **o leitor DECIDE, ou entrega a quem descarta?***
///
/// ⚠️ **A régua tem DUAS metades, e nenhuma basta:** o quadro tem de dar o que a lei GUARDADA dá
/// (`≈ 0`) **e** tem de diferir do que a lei DERIVADA dá. Só a primeira passaria com as duas leis a
/// coincidirem por acaso; só a segunda passaria com o quadro a dar uma terceira coisa qualquer.
#[test]
fn a_tabela_de_pesos_do_caminho_chega_ao_desenho() {
    let (mut sim, mut scene, map, id, ossos) = palco_com_vertices_na_junta();
    assert_eq!(bind(&mut sim, &scene, &map, &[id], None), 1);
    gira(&mut sim, ossos[1], 55.0);
    let desenhado = quadro(&sim, &mut scene, id);

    // As duas leis, pela porta do produto, sobre a MESMA pele e o MESMO caminho guardado.
    let e = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("a pele").clone();
    let g: crate::skinned_mesh::SkinnedPath =
        postcard::from_bytes(&skin.source).expect("a forma guardada");
    assert!(!g.pesos.is_empty(), "a fixtura tem de ter tabela");
    let pele = skin_of(&sim, e).expect("a pele resolve");
    let (mut com, mut sem) = (g.path.clone(), g.path.clone());
    // ⚠️⚠️ **A expectativa constrói-se com a MESMA lei que o quadro corre** (F30, 2026-09-19). Ela
    // usava a `aplica_com` — a lei dos pontos de controlo — e o quadro passou a desenhar a **imagem
    // verdadeira** da curva: o gate acusou `2,3e1` de desvio sobre produto correcto. *Uma régua que
    // compara o produto com uma lei que ele já não corre mede a mudança da lei, não o produto.*
    // ⛔ O que este gate afirma continua a ser o mesmo: **a tabela GUARDADA é que chega ao desenho**,
    // e não a derivada.
    let tol = ph2d_vec_skin::curva::TOLERANCIA;
    ph2d_vec_skin::curva::aplica_pela_curva(&pele, &mut com, &g.pesos, &[], tol);
    ph2d_vec_skin::curva::aplica_pela_curva(&pele, &mut sem, &[], &[], tol);

    let segue_guardada = pior_desvio(&desenhado, &com);
    let segue_derivada = pior_desvio(&desenhado, &sem);
    let separacao = pior_desvio(&com, &sem);
    println!(
        "quadro vs guardada {segue_guardada:.3e} | quadro vs derivada {segue_derivada:.3e} | \
         as duas leis separam {separacao:.3}"
    );
    // ⛔ O CONTROLO: se as duas leis dessem o mesmo, o gate abaixo não afirmaria nada.
    assert!(
        separacao > 0.1,
        "as duas leis entregam o MESMO desenho (separacao {separacao:.3e}) — a fixtura deixou de \
         distinguir a lei guardada da derivada"
    );
    assert!(
        segue_guardada < 1e-9,
        "o quadro NAO desenhou a lei guardada (desviou {segue_guardada:.3e})"
    );
    assert!(
        segue_derivada > 0.1,
        "o quadro desenhou a lei DERIVADA (desviou so' {segue_derivada:.3e} dela) — a tabela de \
         pesos nao chega ao desenho"
    );
}
