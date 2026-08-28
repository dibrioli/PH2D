//! ⭐⭐ **O VÍNCULO AO DESENHO VÊ-SE E SOLTA-SE?** (W57) — os gates da terceira família cuja
//! disponibilidade não é constante.
//!
//! ⚠️ Irmão do [`crate::field3d_profile_reach_tests`] pela mesma fronteira: aquele pergunta *o
//! painel oferece o que o motor sabe fazer?*, este pergunta *o vínculo que a W55 criou é
//! **alcançável** — vê-se sem abrir painel, e há gesto para o largar e para o refazer?*

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_ecs::FieldProfileSource;

use crate::field3d_scene::acts::{ACT_LINK, ACT_UNLINK, LINK_BADGE, acts_for, link_badges};

fn a_square() -> Profile {
    Profile::new(
        vec![vec![[-0.1, -0.1], [0.1, -0.1], [0.1, 0.1], [-0.1, 0.1]]],
        FillRule::NonZero,
        1.0e-3,
    )
    .expect("um quadrado é um perfil")
}

/// A cena nascida do documento, e a **folha** dela — a mesma porta do irmão
/// [`crate::field3d_reach_tests`]: quem faz nascer é o `sync_scene`, nunca um `spawn` à mão.
fn leaf_of(doc: &FieldDoc) -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(doc), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let leaf = ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|&e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(e)
                    .map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            )
        })
        .expect("a folha");
    (sim, vec![leaf])
}

/// Uma peça de **uma forma pendurada numa união** — e a união é load-bearing: o `can_detach` exige
/// um **pai** (a raiz *é* a peça e não se apaga), então uma folha solta receberia a fileira vazia e
/// o gate mediria dois nadas.
fn under_a_union(p: Primitive) -> (SimWorld, Vec<Entity>) {
    use ph2d_field::{Blend, Node, NodeKind, Op};
    leaf_of(
        &FieldDoc::new(
            vec![
                ph2d_field_eval::leaf(p, Xform::IDENTITY),
                Node::new(
                    Xform::IDENTITY,
                    NodeKind::Combine {
                        op: Op::Union(Blend::Sharp),
                        children: vec![NodeId(0)],
                    },
                ),
            ],
            NodeId(1),
        )
        .expect("a peça"),
    )
}

/// Uma peça com **uma extrusão** — o caso normal do artista.
fn a_drawn_shape() -> (SimWorld, Vec<Entity>) {
    under_a_union(Primitive::Extrude {
        profile: a_square(),
        half_height: 0.1,
        round: 0.0,
    })
}

/// Uma peça com uma **esfera** — a forma que não tem onde pôr um perfil.
fn a_plain_shape() -> (SimWorld, Vec<Entity>) {
    under_a_union(Primitive::Sphere { radius: 0.2 })
}

fn link(world: &mut bevy_ecs::world::World, e: Entity, path: u64) {
    world.entity_mut(e).insert(FieldProfileSource {
        path,
        level: ph2d_field::DEFAULT_PROFILE_RESOLUTION,
    });
}

/// Corre um corpo com o estado do módulo **armado e reposto** — o `profile_pick` é global ao
/// processo, e um teste que o deixasse sujo mudaria o vizinho.
fn with_pick<R>(pick: Option<u64>, f: impl FnOnce() -> R) -> R {
    crate::field3d_smoke::set_armed_by_panel(true);
    crate::field3d_smoke::note_profile(pick);
    let out = f();
    crate::field3d_smoke::note_profile(None);
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());
    out
}

/// ⭐⭐ **LARGAR só é oferecido a quem TEM vínculo** — e a pergunta é feita ao componente.
#[test]
fn unlink_is_offered_only_to_a_shape_that_follows_a_drawing() {
    let (mut sim, sel) = a_drawn_shape();
    with_pick(None, || {
        assert!(
            !acts_for(sim.world(), &sel).contains(&ACT_UNLINK),
            "uma extrusão SOLTA recebeu «Unlink» — um botão que não tem o que largar"
        );
        link(sim.world_mut(), sel[0], 7);
        assert!(
            acts_for(sim.world(), &sel).contains(&ACT_UNLINK),
            "uma extrusão LIGADA não recebeu «Unlink» — o vínculo existe e não há como o desfazer"
        );
    });
}

/// ⭐⭐ **LIGAR precisa das DUAS pontas**: um contorno escolhido, e uma forma que saiba o que fazer
/// com ele. ⛔ Um `Sphere` ligado a um desenho é estado inalcançável — o recozimento escreveria um
/// perfil onde a forma não tem onde o pôr.
#[test]
fn link_is_offered_only_with_an_outline_picked_and_a_shape_that_takes_one() {
    let (sim, sel) = a_drawn_shape();
    with_pick(None, || {
        assert!(
            !acts_for(sim.world(), &sel).contains(&ACT_LINK),
            "sem contorno escolhido, «Link Drawing» não tem a que ligar"
        );
    });
    with_pick(Some(7), || {
        assert!(
            acts_for(sim.world(), &sel).contains(&ACT_LINK),
            "com um contorno escolhido, uma extrusão tem de poder ligar-se a ele"
        );
    });
    let (sim, sel) = a_plain_shape();
    with_pick(Some(7), || {
        assert!(
            !acts_for(sim.world(), &sel).contains(&ACT_LINK),
            "uma ESFERA recebeu «Link Drawing» — ela não tem onde pôr um perfil"
        );
    });
}

/// ⭐⭐⭐ **O SLOT RESOLVE-SE EM CHAVE, NUNCA EM NÚMERO** — a lei que a fileira variável exige.
///
/// ⛔ Com `Unlink` presente, o slot `3` é *largar*; sem ele, o slot `3` **não existe**. Um despacho
/// que casasse `0`/`1`/`2` por número faria o botão executar o verbo do vizinho no dia em que a
/// fileira mudasse de tamanho — **sem erro de compilação e sem teste vermelho**. Este gate mede a
/// única coisa que o impede: a lista publicada e a lista despachada são **a mesma função**.
#[test]
fn the_act_slot_resolves_to_a_key_not_to_a_number() {
    let (mut sim, sel) = a_drawn_shape();
    with_pick(Some(7), || {
        let plain = acts_for(sim.world(), &sel);
        link(sim.world_mut(), sel[0], 7);
        let linked = acts_for(sim.world(), &sel);
        assert!(
            linked.len() > plain.len(),
            "a fileira não cresceu com o vínculo — o gate mediria duas listas iguais"
        );
        let slot = linked
            .iter()
            .position(|k| *k == ACT_UNLINK)
            .expect("«Unlink» está na lista");
        // ⭐⭐ **O MESMO NÚMERO, DOIS VERBOS.** Sem vínculo o slot `3` é *ligar*; com vínculo é
        // *largar*. É esta a colisão exacta — e a 1.ª versão deste gate exigia o contrário
        // (`slot >= plain.len()`, "o verbo novo vai para o fim"), que é uma afirmação sobre a
        // ORDEM da lista e não sobre o perigo. *Um gate escrito contra o sintoma que eu imaginei,
        // e não contra o que a lista de facto faz, mede a minha suposição.*
        assert_eq!(
            plain.get(slot).copied(),
            Some(ACT_LINK),
            "a fileira curta não tem outro verbo no slot {slot} — sem isso o despacho por número \
             seria inofensivo e este gate não mediria nada"
        );
        assert_eq!(
            linked.get(slot).copied(),
            Some(ACT_UNLINK),
            "o slot {slot} deixou de ser «Unlink»"
        );
        // ⭐⭐⭐ **E O DESPACHO TEM DE OBEDECER À MESMA LISTA.** ⛔ Sem esta metade o gate mede a
        // lista publicada e não o botão: uma mutação que devolvia o despacho ao `ACTS` fixo
        // **SOBREVIVEU** — a fileira continuava certa e o clique no slot `3` caía fora do array e
        // não fazia nada. *Um gate que lê a lista não prova o botão; a mesma família da W56f
        // («uma lei que o caminho do produto não chama não é uma lei»).*
        let _ = ph2d_panel_model3d::drain_intents();
        ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::Act {
            slot,
        });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &sel,
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            sim.world().get::<FieldProfileSource>(sel[0]).is_none(),
            "o clique no slot {slot} não largou o desenho — o despacho está a resolver o slot por \
             outra lista que não a publicada"
        );
    });
}

/// ⭐⭐ **O SELO diz-se na Hierarquia** — o vínculo vê-se sem abrir painel.
#[test]
fn a_linked_shape_wears_the_badge_and_a_loose_one_does_not() {
    let (mut sim, sel) = a_drawn_shape();
    crate::field3d_smoke::set_armed_by_panel(true);
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    // ⚠️ **A pergunta é sobre o selo do VÍNCULO, e não sobre o mapa estar vazio.** Ela era
    // `is_empty()` enquanto o vínculo e o isolamento eram os únicos selos; a W97 pôs o **verbo** em
    // toda linha que participa da receita, de propósito. *Uma mudança de modelo obriga a
    // re-perguntar o que cada gate ainda mede* — e o que este sempre mediu está agora escrito.
    assert_ne!(
        link_badges().get(&sel[0].to_bits()).copied(),
        Some(LINK_BADGE),
        "uma extrusão SOLTA está a usar o selo do vínculo"
    );
    link(sim.world_mut(), sel[0], 7);
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    assert_eq!(
        link_badges().get(&sel[0].to_bits()).copied(),
        Some(LINK_BADGE),
        "a forma que segue o desenho não recebeu selo — quem olha a árvore não vê diferença \
         nenhuma entre uma extrusão viva e uma fotografia dela"
    );
}

/// ⭐⭐⭐ **UM NÓ ISOLADO QUE SEGUE UM DESENHO MOSTRA `ISO`, NÃO `LNK`** — a precedência, e ela é a
/// decisão que carrega a wave do selo de isolamento (2026-08-25).
///
/// ⛔ **O campo do selo é UM por linha**, e o comentário do merge no `render_loop` afirmava que *"as
/// duas famílias nunca caem na mesma entidade"* — verdade enquanto as famílias eram forma vetorial e
/// nó do modelador. ⚠️ **Estas duas caem**: um nó isolado pode seguir um desenho, e sem uma regra
/// escrita quem ganhava era a ordem de inserção no mapa — *uma decisão de produto tomada por um
/// `extend`*.
///
/// ⭐ Ganha o `ISO`, e a razão não é gosto: *o `LNK` é uma propriedade daquele nó; o `ISO` é um
/// estado da VISTA que explica por que todo o resto desapareceu.* Quando as duas competem, a
/// pergunta que o artista tem é a segunda.
#[test]
fn an_isolated_linked_node_shows_the_isolation_not_the_link() {
    crate::field3d_smoke::forget_isolation();
    let (mut sim, sel) = a_drawn_shape();
    crate::field3d_smoke::set_armed_by_panel(true);
    link(sim.world_mut(), sel[0], 7);
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    // O controle: sem isolamento, o mesmo nó usa o selo do vínculo.
    assert_eq!(
        link_badges().get(&sel[0].to_bits()).copied(),
        Some(LINK_BADGE),
        "a fixtura só prova a precedência se o nó de facto usasse o outro selo"
    );

    assert!(crate::field3d_smoke::toggle_isolate(Some(sel[0].to_bits())));
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    assert_eq!(
        link_badges().get(&sel[0].to_bits()).copied(),
        Some(crate::field3d_scene::acts::ISOLATE_BADGE),
        "com as duas famílias na mesma linha, quem tem de aparecer é o estado da VISTA"
    );
    crate::field3d_smoke::forget_isolation();
}

/// Uma peça com uma **escultura** cujo campo o registo não conhece — o estado em que um projeto
/// abre quando a malha mudou de sítio.
fn a_lost_sculpture(key: &str) -> (SimWorld, Vec<Entity>) {
    use ph2d_field::{Blend, Node, NodeKind, NodeShape, Op};
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(
        &mut sim,
        Some(
            &FieldDoc::new(
                vec![
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Sampled {
                            key: key.to_string(),
                        },
                    ),
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Combine {
                            op: Op::Union(Blend::Sharp),
                            children: vec![NodeId(0)],
                        },
                    ),
                ],
                NodeId(1),
            )
            .expect("a peça"),
        ),
        0.0,
    );
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let leaf = ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|&e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(e)
                    .map(|n| &n.shape),
                Some(NodeShape::Sampled { .. })
            )
        })
        .expect("a escultura");
    (sim, vec![leaf])
}

/// ⭐⭐⭐ **RELIGAR aparece só para a escultura que PERDEU o arquivo** (W76).
///
/// ⚠️ **As três metades são o gate.** Só *«aparece quando falta»* e o verbo estaria sempre lá, a
/// oferecer conserto a quem não está partido; só *«não aparece quando está lá»* e ele não apareceria
/// nunca. E a terceira é a chave **`scene:`**: ela nomeia a escultura **viva** da cena, que não veio
/// de arquivo nenhum — pedir um `.obj` para a substituir seria mandar o artista procurar o que nunca
/// existiu.
#[test]
fn relink_is_offered_only_to_a_sculpture_that_lost_its_file() {
    use crate::field3d_scene::acts::ACT_RELINK;

    let (sim, sel) = a_lost_sculpture("/tmp/uma-malha-que-nao-existe.obj");
    assert!(
        acts_for(sim.world(), &sel).contains(&ACT_RELINK),
        "a escultura sem arquivo TEM de oferecer o conserto"
    );

    // Com o campo no registo, não há o que religar. ⚠️ Uma esfera analítica chega: o que se mede
    // é a **presença** da chave, e voxelizar uma malha aqui seria pagar 200 ms para responder
    // «sim».
    struct Ball;
    impl ph2d_field_eval::hybrid::Sampled for Ball {
        fn at(&self, p: [f32; 3]) -> f32 {
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 0.2
        }
        fn bounding_radius(&self) -> f32 {
            0.2
        }
    }
    crate::field3d_smoke::register_sampled(
        "/tmp/uma-malha-que-nao-existe.obj",
        std::sync::Arc::new(Ball),
    );
    let (sim, sel) = a_lost_sculpture("/tmp/uma-malha-que-nao-existe.obj");
    assert!(
        !acts_for(sim.world(), &sel).contains(&ACT_RELINK),
        "uma escultura que está lá não tem o que consertar"
    );

    // ⚠️ E a escultura VIVA da cena nunca o oferece, mesmo ausente do registo.
    let (sim, sel) = a_lost_sculpture(crate::field3d_import::SCENE_KEY);
    assert!(
        !acts_for(sim.world(), &sel).contains(&ACT_RELINK),
        "uma chave `scene:` não é um arquivo — quem a repõe é o `+ Sculpt from scene`"
    );
}

/// ⭐⭐ **E o pedido atravessa até à CHAVE do nó** (W76) — os saltos que um teste alcança.
///
/// ⚠️ **O salto do meio não é alcançável**: escolher o arquivo é um diálogo nativo, e ele precisa de
/// um app. O que este gate prova são os dois que sobram — que o verbo **pede** com a entidade certa,
/// e que a resposta **reescreve a chave** —, e é a reescrita que faz a cura durar: ela é o que o
/// arquivo do projeto guarda e o que a próxima abertura vai ler.
#[test]
fn the_relink_answer_rewrites_the_key_on_the_node() {
    use ph2d_field::NodeShape;
    let _ = crate::field3d_smoke::take_relink_request();
    let _ = crate::field3d_smoke::take_relinked();

    let (mut sim, sel) = a_lost_sculpture("/tmp/a-que-sumiu.obj");
    let e = sel[0];
    crate::field3d_smoke::ask_relink_sculpt(e.to_bits());
    assert_eq!(
        crate::field3d_smoke::take_relink_request(),
        Some(e.to_bits()),
        "o verbo pede pela entidade escolhida"
    );

    crate::field3d_smoke::ask_relinked(e.to_bits(), "/tmp/a-nova.obj".to_string());
    crate::field3d_scene::sync_scene(&mut sim, None, 0.0);
    assert_eq!(
        sim.world()
            .get::<ph2d_field_ecs::FieldNode>(e)
            .map(|n| match &n.shape {
                NodeShape::Sampled { key } => key.clone(),
                _ => "não é escultura".to_string(),
            }),
        Some("/tmp/a-nova.obj".to_string()),
        "a chave nova tem de ficar NO NÓ — senão a peça abre hoje e falha outra vez amanhã"
    );
}

/// ⭐⭐⭐ **TODO VERBO DA FILEIRA TEM DE TER RÓTULO** — e a lista é VARRIDA, não escrita à mão.
///
/// ⚠️ **O `ph2d_i18n::tr` de uma chave desconhecida devolve a PRÓPRIA chave** (`leak_key`, *"o
/// identificador cru é feio de propósito"*), e o painel pinta o que ele devolve
/// (`ph2d_panel_model3d::paint`). ⇒ um verbo sem entrada no i18n **não falha em lado nenhum**: ele
/// aparece na tela a dizer `panel.model3d.act.<algo>`, e todo gate de alcance continua verde —
/// eles perguntam *"o verbo é oferecido?"*, nunca *"ele diz o quê?"*.
///
/// ⭐ **Foi assim que a W76 shipou:** o `ACT_RELINK` nasceu com o gesto, o despacho e três provas de
/// mutação, e **sem a linha no `model3d.rs`**. O botão que fecha o beco do arquivo perdido dizia
/// `panel.model3d.act.relink`.
///
/// ⚠️ **A lista é lida do FONTE, de propósito.** Um array `ALL_ACT_KEYS` escrito à mão ao lado dos
/// `push` seria a mesma doença um nível acima — *um `match` exaustivo não guarda a lista de um
/// laço*: o 7.º verbo entraria por um `push` e escaparia ao gate sem erro nenhum.
#[test]
fn every_act_the_row_can_emit_says_something_other_than_its_own_key() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/field3d_scene_acts.rs"),
    )
    .expect("o irmão das ações existe");
    let keys: Vec<String> = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .filter(|l| l.contains("const ACT_") && l.contains(": &str = \""))
        .filter_map(|l| l.split('"').nth(1).map(str::to_string))
        .collect();
    assert!(
        keys.len() >= 6,
        "a varredura tem de achar os verbos; achou {keys:?} — se o formato do `const` mudou, é o \
         gate que ficou cego, não o módulo que ficou limpo"
    );
    let mudos: Vec<&String> = keys
        .iter()
        .filter(|k| ph2d_i18n::tr(k) == k.as_str())
        .collect();
    assert!(
        mudos.is_empty(),
        "estes verbos vão pintar o próprio identificador na tela (falta a linha em \
         `crates/ph2d-i18n/src/model3d.rs`): {mudos:?}"
    );
}

/// ⭐⭐⭐ **A PRECEDÊNCIA DO SELO, e ela tem UMA excepção** (W97) — `BSE` cede ao `LNK`, os outros
/// verbos não.
///
/// # ⚠️ A regra, numa frase
///
/// > *O selo diz o que a linha não consegue dizer sozinha.*
///
/// `BSE` é **derivável da posição** — é a primeira linha que conta —, então cedê-lo ao vínculo não
/// custa informação nenhuma. `SUB`/`INT`/`UNI` **não** são deriváveis de nada na tela, e sem eles a
/// receita ganha um buraco: uma sequência com uma linha ilegível deixa de se ler **inteira**, ao
/// contrário de uma marca de proveniência, que é um facto independente por linha.
///
/// ⛔ **Um segundo selo por linha é a cura de verdade, e o preço está MEDIDO:** o campo é
/// `HierarchyEntity::badge: Option<String>` e os produtores são vários (o vetor, a sprite, a física,
/// este módulo) — é mudança de tipo foundational com N escritores, e não «uma função». É esse o
/// gatilho para rever isto.
#[test]
fn only_the_base_badge_yields_to_the_link_badge() {
    use ph2d_field::{Blend, Op};

    let (mut sim, sel) = a_drawn_shape();
    crate::field3d_smoke::set_armed_by_panel(true);
    link(sim.world_mut(), sel[0], 7);

    // A forma é a ÚNICA do grupo, logo é a base — e a base cede.
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    assert_eq!(
        link_badges().get(&sel[0].to_bits()).copied(),
        Some(LINK_BADGE),
        "a BASE ligada a um desenho tinha de mostrar o vínculo — `BSE` repete o que a posição diz"
    );

    // ⭐ **O controlo que dá sentido ao gate:** a MESMA linha, com um verbo que não é derivável,
    // deixa de ceder. Sem isto, «cede sempre» passaria aqui.
    let pai = sim
        .world()
        .get::<bevy_ecs::hierarchy::ChildOf>(sel[0])
        .expect("a forma tem o grupo por pai")
        .0;
    let irmao = ph2d_field_ecs::add_leaf(
        sim.world_mut(),
        pai,
        Primitive::Sphere { radius: 0.2 },
        [0.0, 0.0, 0.0],
    )
    .expect("nasce ao lado");
    // A ordem manda: o irmão nasce depois, então a extrusão continua a ser a base. Ela só deixa de o
    // ser quando é ELA a falar — e aí o verbo ganha.
    ph2d_field_ecs::set_verb(sim.world_mut(), irmao, Some(Op::Difference(Blend::Sharp)))
        .expect("é um nó");
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    link(sim.world_mut(), irmao, 9);
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    assert_eq!(
        link_badges().get(&irmao.to_bits()).copied(),
        Some("SUB"),
        "uma forma que CORTA e segue um desenho tem de mostrar o corte — o buraco na receita custa \
         mais do que a marca de proveniência"
    );
    crate::field3d_smoke::set_armed_by_panel(false);
}
