//! Os gates do [`super`] — o ponto novo entra na fonte, a tabela fecha, e a forma quase não se move.

use super::*;
use ph2d_ecs::{ChildOf, Name, RootOrder, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::{ShapeKind, VecScene, cook};

/// ⛔⛔⛔ **A F30 MATOU A PREMISSA DOS GATES DA COMPENSAÇÃO, e eles medem-na EXPLICITAMENTE desde
/// então.** Com a lei da curva ligada — o caminho de OMISSÃO — o desenho é a **imagem verdadeira** da
/// fonte, logo partir a fonte não o move (`0,0000 %`) e compensar **estraga** (`11,11 %` da peça).
/// ⇒ a compensação é da lei dos pontos de controlo, e é lá que ela se mede: estes gates chamam as
/// portas com a lei explícita (`recook_com` / `insere_ponto_com`).
///
/// ⚠️⚠️ **Ela chegou a ser um ESTADO GLOBAL com uma porta `forcar_lei` (que já não existe), e isso era um canal entre
/// testes:** o doc dela dizia *«o nextest corre um processo por teste»* — verdade para o `nextest`,
/// **falsa** para o `cargo test`, que corre os testes em THREADS do mesmo processo. A suíte
/// reprovava em conjunto e passava sozinha, que é a assinatura mais cara que há.
pub(super) const LEI_INGENUA: bool = false;

/// ⭐ A lei de OMISSÃO — a que o artista corre. Ver [`ph2d_vec_skin::curva`].
pub(super) const LEI_DA_CURVA: bool = true;

pub(super) fn osso(
    sim: &mut SimWorld,
    nome: &str,
    pos: [f32; 2],
    len: f64,
    pai: Option<Entity>,
) -> Entity {
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

/// Um rectângulo deitado com um braço de dois ossos por cima — a fixtura das outras suítes desta
/// crate, e ela tem tabela de pesos porque a forma é FECHADA.
///
/// ⛔⛔ **Ela prende SEM a subdivisão do bind, e desde 2026-09-20 isso é o PRODUTO** (ordem do
/// dono: *«retire a criação automática de ponto no bind»*). A F28 responde *«o que acontece
/// quando um ponto NOVO entra numa forma GROSSEIRA?»* — e uma forma grosseira voltou a ser o que
/// o artista tem na mão depois de carregar em *Bind*.
///
/// ⚠️⚠️ **A redacção de 19/09 dizia o CONTRÁRIO e ficou aqui um dia:** *«desde a ordem do dono o
/// produto já não produz formas grosseiras — ele subdivide no Bind … com o bind de hoje estes
/// gates ficariam VERDES por vácuo»*. ⇒ *toda a razão de ser desta suíte voltou, e o que a
/// protegeu de um dia inteiro a medir o caminho errado foi ela pedir a lei pelo PARÂMETRO em vez
/// de a herdar do default.*
pub(super) fn palco() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let id = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let n = crate::skin_live::bind_com(&mut sim, &cena, &mapa, &[id], Some(raiz), false);
    assert_eq!(n, 1, "o palco tem de prender");
    (sim, cena, mapa, id, [raiz, ponta])
}

/// ⭐ **O MESMO palco com a SUBDIVISÃO — a lei que saiu do produto em 2026-09-20.**
///
/// ⚠️⚠️ **Ele chamava-se `palco_do_produto` e pedia a subdivisão pelo DEFAULT do
/// [`crate::skin_live::bind`]** — e no dia em que o default virou, ele passou a ser byte-idêntico
/// ao [`palco`] **sem uma linha mudar**. *Dois nomes para o mesmo palco são duas respostas à
/// mesma pergunta, e a que envelhece é a que herda o default:* hoje ele pede a lei pelo
/// parâmetro, como a irmã, e o nome diz o que ele É em vez de para quem ele serve.
///
/// Ele é o **contrafactual** de tudo o que a subdivisão comprava: *a fidelidade de uma forma
/// presa é propriedade da GEOMETRIA, e é ela que a dá.*
pub(super) fn palco_subdividido() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let id = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let n = crate::skin_live::bind_com(&mut sim, &cena, &mapa, &[id], Some(raiz), true);
    assert_eq!(n, 1, "o palco subdividido tem de prender");
    (sim, cena, mapa, id, [raiz, ponta])
}

/// O mesmo palco, mas com a aresta de baixo **DESENHADA** com `pedacos` pedaços antes do bind — o
/// que um artista faz quando quer controlo ali.
///
/// ⚠️ **Os cortes são ANTES do bind**, e é isso que faz a diferença: a tabela de pesos é resolvida
/// sobre o caminho já subdividido, logo cada segmento cobre pouca variação de peso.
pub(super) fn palco_desenhado(
    pedacos: usize,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let id = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    // ⚠️⚠️ **Corta sempre o pedaço QUE SOBRA, e não o primeiro.** A 1.ª redacção fazia
    // `split_segment(.., 0, ..)` em laço e subdividia o primeiro pedaço vezes sem conta, deixando o
    // ÚLTIMO a atravessar a junta inteira — a fixtura chamava-se «desenhada» e media o mesmo
    // segmento grosseiro do outro palco (`gap = 1,0`, salto `15 %`). *Uma fixtura com o nome errado
    // é pior que nenhuma: ela responde à pergunta do vizinho.*
    for i in 0..pedacos.saturating_sub(1) {
        let t = 1.0 / (pedacos - i) as f64;
        ph2d_vec_scene::split_segment(cena.path_mut(id).expect("caminho"), i, t)
            .expect("o desenho subdivide");
    }
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let n = crate::skin_live::bind_com(&mut sim, &cena, &mapa, &[id], Some(raiz), false);
    assert_eq!(n, 1, "o palco desenhado tem de prender");
    (sim, cena, mapa, id, [raiz, ponta])
}

pub(super) fn fonte(
    sim: &SimWorld,
    mapa: &VecEntityMap,
    id: VecPathId,
) -> crate::skinned_mesh::SkinnedPath {
    let e = Entity::from_bits(*mapa.get(&id).expect("entidade"));
    let skin = sim.world().get::<SkinBind>(e).expect("presa");
    crate::skinned_mesh::le(&skin.source).expect("a fonte le-se")
}

pub(super) fn ancoras(cena: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    cena.path(id)
        .map(|p| p.verts_all().map(|v| v.anchor).collect())
        .unwrap_or_default()
}

/// ⭐⭐⭐ **O PONTO NOVO SOBREVIVE AO QUADRO** — a lei da wave, medida pelo caminho do produto.
///
/// ⛔ O controlo está dentro: a sonda `hoje_um_ponto_novo_numa_forma_presa_evapora_se` (na
/// `ph2d-app-skeleton`) mede o **outro** lado — inserir só no documento vivo e ver o `recook`
/// deitá-lo fora. *Sem os dois, este gate não distingue a cura de uma fixtura que nunca conteve o
/// defeito.*
#[test]
fn o_ponto_novo_sobrevive_ao_quadro() {
    let (mut sim, mut cena, mapa, id, _) = palco();
    let antes = ancoras(&cena, id).len();

    assert!(
        insere_ponto_com(&mut sim, &mapa, id, 0, 0.5, LEI_INGENUA).is_some(),
        "a porta recusou uma forma que ESTA' presa"
    );

    assert_eq!(
        fonte(&sim, &mapa, id).path.verts_all().count(),
        antes + 1,
        "a FONTE nao ganhou o ponto — o quadro seguinte deita-o fora"
    );
    crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
    assert_eq!(
        ancoras(&cena, id).len(),
        antes + 1,
        "o ponto evaporou-se no quadro: o desenho e' re-derivado da fonte, e ela nao o tem"
    );
}

/// ⛔ **Uma forma SOLTA não passa por aqui** — a porta devolve `None` e quem chama faz o que sempre
/// fez. *Uma porta que aceitasse tudo tornaria a caneta dependente do esqueleto.*
#[test]
fn uma_forma_solta_nao_e_desta_porta() {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let id = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    assert_eq!(
        insere_ponto_com(&mut sim, &mapa, id, 0, 0.5, LEI_INGENUA),
        None,
        "a porta aceitou uma forma sem pele"
    );
}

/// ⭐⭐⭐ **A TABELA CRESCE COM O CAMINHO, e a linha nova é a MISTURA dos vizinhos.**
///
/// ⚠️ **As três metades são três defeitos diferentes:** a tabela pode fechar e estar deslocada; a
/// linha pode existir e ser lixo; e ela pode ser plausível e não somar `1`, que é o que faz um nó
/// ser entregue ao primeiro osso.
#[test]
fn a_tabela_cresce_e_a_linha_nova_e_a_mistura_dos_vizinhos() {
    let (mut sim, _cena, mapa, id, _) = palco();
    let velha = fonte(&sim, &mapa, id);
    let ossos = velha.ossos();
    assert!(ossos >= 2, "o palco tem de ter tabela, senao nao mede nada");
    let n_velho = velha.path.verts_all().count();

    let ni = insere_ponto_com(&mut sim, &mapa, id, 0, 0.5, LEI_INGENUA).expect("insere");
    let nova = fonte(&sim, &mapa, id);

    assert!(nova.valida(), "a tabela deixou de fechar com o caminho");
    assert_eq!(nova.ossos(), ossos, "a tabela mudou de LARGURA");
    assert_eq!(nova.path.verts_all().count(), n_velho + 1);

    let linha = nova.linha_do_no(ni).expect("a linha do no' novo");
    let a = velha.linha_do_no(0).expect("vizinho a");
    let b = velha.linha_do_no(1).expect("vizinho b");
    for (k, w) in linha.iter().enumerate() {
        let esperado = a[k] + (b[k] - a[k]) * 0.5;
        assert!(
            (w - esperado).abs() < 1e-12,
            "a linha do no' novo nao e' a mistura dos vizinhos no osso {k}: {w} contra {esperado}"
        );
    }
    let soma: f64 = linha.iter().sum();
    assert!(
        (soma - 1.0).abs() < 1e-9,
        "a linha nova soma {soma} e nao 1 — um no' que nao e' de ninguem e' entregue ao primeiro osso"
    );
    // ⛔ E os VIZINHOS não mudaram de peso: uma re-resolução global apagaria a linha de base que o
    // artista corrigiu à mão.
    assert_eq!(
        nova.linha_do_no(0).expect("a"),
        a,
        "o peso do vizinho anterior MUDOU — alguem re-resolveu o global"
    );
}
