//! ⭐⭐⭐ **A SONDA: o que acontece hoje quando o artista acrescenta um ponto a uma forma PRESA?**
//!
//! ⚠️ **Ela existe para medir uma afirmação MINHA antes de construir sobre ela.** A F26 deixou ao
//! dono duas saídas para *«pintar peso entre os vértices de uma forma vectorial»*, e escreveu sobre
//! a primeira: *«acrescentar vértices com a caneta … o gesto já existe nesta casa. **Custo: zero de
//! arquitectura**»*. ⛔ *Uma presença afirmada sem olhar o caminho do produto é um palpite com cara
//! de medição* — a mesma família que este repo já pagou nos dois sentidos.
//!
//! O que a sonda corre é o caminho do artista, na ordem dele: prender · inserir um ponto · deixar o
//! quadro passar.
//!
//! ⭐⭐ **DEPOIS DA CURA ela continua verde, e passa a ser o CONTROLO NEGATIVO dela.** A cura
//! ([`ph2d_skeleton_live::ponto_novo`]) acrescentou uma ROTA — a caneta reporta onde inseriu e a
//! shell leva isso à geometria autorada. ⛔ A rota velha, escrever só no documento vivo, continua a
//! evaporar-se, e é isso que os gates da lei precisam de ter do outro lado: *sem esta sonda eles não
//! distinguem a cura de uma fixtura que nunca conteve o defeito.*

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook};

/// Um osso em `pos` (local do pai), comprimento `len`, alcance 1.
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
                ..Bone::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// O palco: um rectângulo deitado com um braço de dois ossos por cima.
fn palco() -> (
    SimWorld,
    VecScene,
    ph2d_vec_entities::entities::VecEntityMap,
    VecPathId,
    Entity,
) {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = ph2d_vec_entities::entities::VecEntityMap::new();
    let caminho = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
    let _ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, cena, mapa, caminho, raiz)
}

fn pontos(cena: &VecScene, id: VecPathId) -> usize {
    cena.path(id).map_or(0, |p| p.verts_all().count())
}

/// ⛔⛔⛔ **MEDIDO: escrito SÓ no documento vivo, o ponto novo evapora-se no quadro seguinte.**
///
/// O `recook` corre uma vez por quadro e reconstrói o desenho a partir da geometria **autorada** que
/// o `bind` guardou (`SkinBind::source`) — logo tudo o que a caneta escreve no documento vivo é
/// deitado fora ao fim de um quadro. ⚠️ **Não há erro, não há aviso e não há recusa:** o artista vê
/// o ponto aparecer sob o dedo e desaparecer sozinho.
///
/// ⇒ *A primeira saída da F26 NÃO custava zero de arquitectura*, e esta sonda é a medição que o
/// disse. ⭐ Ela fica como **controlo negativo** da cura: a rota que passa pela
/// [`ph2d_skeleton_live::ponto_novo::insere_ponto`] sobrevive, esta não.
#[test]
fn hoje_um_ponto_novo_numa_forma_presa_evapora_se() {
    let (mut sim, mut cena, mapa, caminho, raiz) = palco();
    let antes = pontos(&cena, caminho);

    let n = ph2d_skeleton_live::skin_live::bind(&mut sim, &cena, &mapa, &[caminho], Some(raiz));
    assert_eq!(n, 1, "o palco tem de prender, senao nao mede nada");
    // ⚠️⚠️ **O que o quadro devolve é a FONTE, e ela voltou a ser a forma que o artista desenhou.**
    // A 1.ª redacção comparava o `depois` com o ANTES; em 2026-09-19 a subdivisão do bind matou
    // essa premissa e esta linha passou a exigir `na_fonte > antes`; em **2026-09-20** o dono
    // mandou *«retire a criação automática de ponto no bind»* e ela morreu outra vez, **para o
    // lado de onde tinha vindo**.
    // ⭐ *A sonda atravessou as duas ordens opostas por medir a FONTE em vez de um literal — o que
    // muda aqui é só o SENTIDO da comparação, e ele é afirmado em vez de presumido.*
    let na_fonte = ph2d_ecs::Entity::try_from_bits(*mapa.get(&caminho).expect("entidade"))
        .and_then(|e| sim.world().get::<ph2d_skeleton_ecs::SkinBind>(e))
        .and_then(|sk| ph2d_skeleton_live::skinned_mesh::le(&sk.source))
        .map_or(0, |g| g.path.verts_all().count());
    assert_eq!(
        na_fonte, antes,
        "o bind mexeu na contagem de pontos da forma ({na_fonte} contra {antes}) — a subdivisao \
         voltou ao caminho de produto, e esta sonda passa a medir outra coisa"
    );

    // ⚠️ **É a porta que a caneta chama**: o `insert_on_selected_segment` do `ph2d-vec-edit` faz
    // `split_segment(scene.path_mut(sel), seg, t)` e mais nada — ela é o núcleo do gesto.
    let inserido =
        ph2d_vec_scene::split_segment(cena.path_mut(caminho).expect("o caminho existe"), 0, 0.5);
    assert!(inserido.is_some(), "a insercao tem de acontecer");
    let com_o_ponto = pontos(&cena, caminho);
    assert_eq!(
        com_o_ponto,
        antes + 1,
        "a caneta nao inseriu: a sonda estaria a medir outra coisa"
    );

    // O quadro passa.
    ph2d_skeleton_live::skin_live::recook(&sim, &mut cena);
    let depois = pontos(&cena, caminho);

    eprintln!(
        "[sonda-ponto-novo] antes={antes} com_o_ponto={com_o_ponto} depois_do_quadro={depois}"
    );
    assert_eq!(
        depois, na_fonte,
        "ESTA SONDA MUDOU DE VEREDITO: o ponto novo SOBREVIVEU ao quadro. Se alguem curou isto, \
         apague esta sonda e escreva o gate da lei nova — uma sonda que mede um defeito curado \
         defende-o"
    );
    // ⭐ E a metade que o `na_fonte` sozinho não diz: o ponto que a caneta escreveu no documento
    // vivo **não está lá**. *Sem ela, uma fonte que por acaso tivesse `com_o_ponto` nós leria como
    // «sobreviveu».*
    assert_ne!(
        depois, com_o_ponto,
        "o ponto escrito so' no documento vivo sobreviveu ao quadro"
    );
}

/// ⭐⭐ **E a segunda metade, que diz PORQUE a cura não é só «escrever no `source`».**
///
/// A tabela de pesos é achatada (`pesos[c * ossos + j]`, três pontos de controlo por vértice) e é
/// calculada **uma vez**, no `bind`. Um vértice novo não tem linha nenhuma nela — e o `recook` tem
/// uma cerca explícita para isso: *«uma tabela que não fecha com o caminho cai na lei derivada em
/// vez de ser lida deslocada»*.
///
/// ⇒ mesmo que a caneta escrevesse no `source`, a forma inteira **mudaria de lei** no quadro
/// seguinte: ela deixaria de usar os pesos que o artista corrigiu à mão e voltaria aos automáticos.
/// *É este o custo real da primeira saída, e ele não é zero.*
///
/// ⭐ **É esta a metade que a [`ph2d_skeleton_live::ponto_novo::insere_na_fonte`] paga:** ela faz o
/// corte **e** cresce a tabela, com a linha do nó novo a ser a mistura das dos dois vizinhos. Esta
/// sonda mede o que acontece a quem fizer só a primeira metade.
#[test]
fn a_tabela_de_pesos_nao_cobre_um_vertice_novo() {
    let (mut sim, cena, mapa, caminho, raiz) = palco();
    ph2d_skeleton_live::skin_live::bind(&mut sim, &cena, &mapa, &[caminho], Some(raiz));

    let e = ph2d_ecs::Entity::from_bits(*mapa.get(&caminho).expect("a forma tem entidade"));
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(e)
        .cloned()
        .expect("prendeu");
    let mut guardado = ph2d_skeleton_live::skinned_mesh::le(&skin.source).expect("a fonte le-se");
    assert!(
        guardado.valida(),
        "a fonte guardada tem de fechar, senao a sonda mede uma pele partida"
    );
    let ossos = guardado.ossos();
    let pontos_antes = guardado.pontos();

    // O artista acrescenta um ponto — e imaginemos que ele chegava ao `source`.
    ph2d_vec_scene::split_segment(&mut guardado.path, 0, 0.5).expect("insere");

    eprintln!(
        "[sonda-ponto-novo] pesos={} ossos={ossos} pontos_antes={pontos_antes} \
         pontos_depois={} fecha={}",
        guardado.pesos.len(),
        guardado.pontos(),
        guardado.valida()
    );
    assert!(
        !guardado.valida(),
        "ESTA SONDA MUDOU DE VEREDITO: a tabela passou a fechar com um vertice a mais. Se alguem \
         ensinou o `split_segment` a crescer a tabela, apague esta sonda e gateie a lei nova"
    );
}
