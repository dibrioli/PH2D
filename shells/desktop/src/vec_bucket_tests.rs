//! Gates da COSTURA do **BALDE** — o que a lei pura não alcança: a cena, a pose e o cache.

use super::*;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: ph2d_vec_scene::VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// ⚠️ **A chave é o CONTEÚDO, não a contagem.** Mover uma forma não muda quantas há, e um cache
/// que não visse isso acenderia uma face onde já não há linha nenhuma.
#[test]
fn the_cache_key_follows_the_geometry_not_the_count() {
    let a = vec![(vec![v(0.0, 0.0), v(10.0, 0.0)], false)];
    let b = vec![(vec![v(0.0, 0.0), v(10.0, 0.5)], false)];
    assert_ne!(chave(&a), chave(&b), "mover uma ponta nao mudou a chave");
    assert_eq!(chave(&a), chave(&a.clone()), "a chave tem de ser estavel");
}

/// ⭐⭐⭐ **AS DUAS ALÇAS entram na chave** — a lei que o Enio nomeou em 2026-09-01:
///
/// > *"O nó de uma solda é um só para todas as linhas. As alças daquele nó devem servir
/// > simultaneamente para o stroke e para os preenchimentos, senão é impossível que sejam
/// > transformados juntos."*
///
/// ⛔⛔ **A 1.ª redacção deste gate media UMA das duas** (`out_handle`), e o produto tinha
/// exactamente esse buraco: arrastar a alça de ENTRADA de um nó mudava o traço e o preenchimento
/// **não sabia** — a chave do cache não via a mudança, a rede não era refeita, e a área ficava com
/// a curva de antes. *Um gate que mede metade da população aprova a metade que não mediu.*
#[test]
fn the_key_sees_both_handles_move() {
    let base = vec![(vec![v(0.0, 0.0), v(10.0, 0.0)], false)];
    let antes = chave(&base);
    let mut saida = base.clone();
    saida[0].0[0].out_handle = [3.0, 4.0];
    assert_ne!(antes, chave(&saida), "a alca de SAIDA nao entra na chave");
    let mut entrada = base.clone();
    entrada[0].0[1].in_handle = [7.0, -4.0];
    assert_ne!(
        antes,
        chave(&entrada),
        "a alca de ENTRADA nao entra na chave — o traco muda e o preenchimento fica com a curva \
         de antes"
    );
}

/// ⛔ **Um caminho ESCONDIDO não cerca nada** — ele estaria a preencher contra uma parede
/// invisível.
#[test]
fn a_hidden_path_is_not_a_wall() {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0)],
        ..VecPath::default()
    });
    let (todos, tags) = contornos_mundo(&scene, &VecXforms::new(), &|_| false);
    assert_eq!(
        tags.len(),
        todos.len(),
        "cada contorno tem de trazer a etiqueta dele"
    );
    assert_eq!(todos.len(), 1);
    let (nenhum, _) = contornos_mundo(&scene, &VecXforms::new(), &|x| x == id);
    assert!(nenhum.is_empty(), "o escondido entrou na rede");
}

/// ⚠️⚠️ **A POSE entra na conta**: dois traços só se cruzam depois de o `Transform` os pôr no
/// lugar, e medir na geometria local diria que eles não se encontram.
#[test]
fn the_contours_come_out_in_world_space() {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0)],
        ..VecPath::default()
    });
    let mut xf = VecXforms::new();
    xf.insert(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 100.0, 0.0]));
    let (c, _) = contornos_mundo(&scene, &xf, &|_| false);
    assert_eq!(c[0].0[0].anchor, [100.0, 0.0], "a pose nao entrou");
}

/// ⛔⛔ **UM PREENCHIMENTO NÃO É PAREDE** — a metade de CENA da lei.
///
/// Report do Enio (2026-09-01): *"ao usar o balde nas áreas coloridas, ele para de funcionar nas
/// áreas não coloridas."* O mecanismo (arestas coincidentes envenenam o passeio) está medido na
/// lei pura; aqui prova-se que a shell **reconhece** o preenchimento e o mantém fora.
#[test]
fn a_bucket_fill_is_not_a_wall() {
    use ph2d_ecs::{Name, SimWorld, Transform, VecBucketFill, VecPathRef};
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let mut scene = VecScene::new();
    let parede = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0)],
        ..VecPath::default()
    });
    let area = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)],
        closed: true,
        ..VecPath::default()
    });
    for (id, fill) in [(parede, false), (area, true)] {
        let mut e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(id)));
        if fill {
            e.insert(VecBucketFill::new([5.0, 2.0], Vec::new()));
        }
        map.insert(id, e.id().to_bits());
    }
    let fills = preenchimentos(&sim, &map);
    assert_eq!(fills.len(), 1, "a shell nao reconheceu o preenchimento");
    assert_eq!(fills[0].0, area);
    assert_eq!(fills[0].2.seed, [5.0, 2.0], "a semente nao voltou inteira");

    // ⚠️ **A MESMA porta que o produto usa** (`fora_da_rede`) — um fecho escrito aqui testaria o
    // fecho deste teste, e foi assim que a mutação que apagava o termo do preenchimento sobreviveu.
    let so_fill: std::collections::BTreeSet<u64> = fills.iter().map(|(id, _, _)| *id).collect();
    let (paredes, _) = contornos_mundo(&scene, &VecXforms::new(), &|id| {
        fora_da_rede(false, so_fill.contains(&id))
    });
    assert!(
        fora_da_rede(false, true),
        "um preenchimento tem de ficar FORA"
    );
    assert!(fora_da_rede(true, false), "um escondido tem de ficar FORA");
    assert!(!fora_da_rede(false, false), "uma linha comum e' PAREDE");
    assert_eq!(
        paredes.len(),
        1,
        "o preenchimento entrou na rede como parede"
    );
    assert!(
        !paredes[0].1,
        "a parede que sobrou tem de ser a linha ABERTA"
    );
}

/// ⭐⭐⭐ **A ÁREA RE-COZIDA DESCE AO ESPAÇO DO CAMINHO** (report do Enio, 2026-09-01, com foto:
/// *"o preenchimento está nascendo deslocado para fora do stroke"*).
///
/// ⚠️ A rede fala MUNDO; o documento guarda LOCAL. Depois de o `settle_origins` mudar a origem da
/// entidade para o centro da caixa dela, escrever mundo naquele `VecPath` desloca-o **pelo centro
/// dele** — e era por isso que cada área saía com um desvio DIFERENTE.
#[test]
fn the_recooked_area_comes_down_to_the_paths_own_space() {
    let mundo = vec![v(100.0, 50.0), v(140.0, 50.0), v(140.0, 90.0)];
    // A pose que o assentamento deixa: a origem no centro da caixa.
    let xf = ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 120.0, 70.0]);
    let local = para_local(mundo.clone(), &xf).expect("a pose e' invertivel");
    assert_eq!(
        local[0].anchor,
        [-20.0, -20.0],
        "a area nao desceu ao local"
    );
    // …e de volta ao mundo pela pose, ela está EXACTAMENTE onde a rede a pôs.
    for (l, m) in local.iter().zip(&mundo) {
        assert_eq!(xf.apply(l.anchor), m.anchor, "o ida-e-volta nao fecha");
    }
    // ⚠️ Identidade é o caminho comum (a forma acabada de nascer) e não paga nada.
    assert_eq!(
        para_local(mundo.clone(), &ph2d_vec_scene::Xform::IDENTITY),
        Some(mundo)
    );
}

/// ⛔⛔ **O preenchimento é DERIVADO, e a vista di-lo** — a outra metade da lei do Enio
/// (2026-09-01). Sem esta linha, os nós dele desenham-se e agarram-se: um segundo conjunto de
/// alças empilhado sobre o das linhas que o produzem.
#[test]
fn a_bucket_fill_is_published_as_derived_and_is_not_pickable() {
    use ph2d_ecs::{Name, SimWorld, Transform, VecBucketFill, VecPathRef};
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let (parede, area) = (7u64, 9u64);
    for (id, fill) in [(parede, false), (area, true)] {
        let mut e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(id)));
        if fill {
            e.insert(VecBucketFill::new([0.0, 0.0], Vec::new()));
        }
        map.insert(id, e.id().to_bits());
    }
    let view = crate::vec_entities::view_state(&sim, &map);
    assert!(
        view.is_derived(area),
        "o preenchimento nao foi publicado como derivado"
    );
    assert!(!view.is_derived(parede), "a parede nao pode ser derivada");
    assert!(
        !view.is_pickable(area),
        "os nos do preenchimento continuam agarraveis"
    );
    assert!(view.is_pickable(parede), "a parede deixou de ser agarravel");
}

/// ⭐⭐⭐ **UMA REGIÃO QUE PARTIU SAI COMO **UM** OBJECTO, com um contorno por pedaço.**
///
/// Report do Enio (2026-09-02): *"quando atravessamos uma linha com um nó, os preenchimentos se
/// quebram"*. A metade da cura que se vê é esta: as duas metades chegam ao documento como o
/// contorno **primário** e um **subpath** do mesmo caminho — ⛔ e não como dois objectos, que
/// encheriam a Hierarquia à primeira travessia (o outro report do mesmo dia).
#[test]
fn a_region_that_split_comes_out_as_one_object_with_a_contour_per_piece() {
    let quadrado = (
        vec![
            v(-10.0, -10.0),
            v(10.0, -10.0),
            v(10.0, 10.0),
            v(-10.0, 10.0),
        ],
        true,
    );
    let linha = (vec![v(-20.0, 0.0), v(20.0, 0.0)], false);
    let rede = ph2d_vec_fill::rede(&[quadrado, linha]);
    let faces: Vec<_> = rede.faces().into_iter().filter(|f| f.area > 0.0).collect();
    assert_eq!(faces.len(), 2, "a fixtura tem de partir o quadrado em duas");

    let (primeiro, subs) =
        geometria_local(&rede, &faces, &[0, 1], &ph2d_vec_scene::Xform::IDENTITY)
            .expect("as duas faces dao geometria");

    assert!(
        primeiro.len() >= 3,
        "o contorno primario e' uma das metades"
    );
    assert_eq!(
        subs.len(),
        1,
        "a OUTRA metade vem como subpath do mesmo caminho"
    );
    assert!(subs[0].closed, "um preenchimento e' sempre fechado");
    assert!(subs[0].verts.len() >= 3);
}

/// ⛔ **Sem face nenhuma não sai forma nenhuma** — é o que faz um preenchimento sem dono CONGELAR
/// onde está, em vez de ser reescrito com geometria vazia.
#[test]
fn a_fill_that_won_no_face_produces_no_shape() {
    let rede = ph2d_vec_fill::rede(&[(vec![v(0.0, 0.0), v(10.0, 0.0)], false)]);
    let faces: Vec<_> = rede.faces().into_iter().filter(|f| f.area > 0.0).collect();
    assert!(
        geometria_local(&rede, &faces, &[], &ph2d_vec_scene::Xform::IDENTITY).is_none(),
        "sem faces, nada a escrever"
    );
}

/// Um quadrado FECHADO com `fill`, com a quina inferior-esquerda em `(x, y)` e lado `l`.
fn forma_pintada(scene: &mut VecScene, x: f64, y: f64, l: f64) -> u64 {
    scene.push_path(VecPath {
        verts: vec![
            v(x, y),
            v(x + l, y),
            v(x + l, y + l),
            v(x, y + l),
        ],
        closed: true,
        fill: Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
            10, 20, 30, 255,
        ))),
        ..VecPath::default()
    })
}

/// ⭐⭐⭐ **O BALDE SOBREPÕE-SE AO QUE JÁ PINTOU** — o preenchimento antigo é APAGADO, não empilhado.
///
/// Report do Enio (2026-09-04). ⚠️⚠️ **Sem isto o defeito não era «não faz nada»:** a forma nova
/// nascia por cima e no quadro seguinte as duas tinham âncoras na MESMA face; o empate de
/// [`crate::vec_bucket_claim::donos`] desce ao índice do documento, logo **a velha ganhava** e a
/// nova ficava congelada e invisível.
#[test]
fn a_bucket_fill_already_under_the_click_is_deleted_not_stacked() {
    let mut scene = VecScene::new();
    let velho = forma_pintada(&mut scene, 0.0, 0.0, 10.0);
    let (baldes, formas) = tinta_sob(
        &scene,
        &VecXforms::new(),
        &|id| id == velho,
        [5.0, 5.0],
    );
    assert_eq!(baldes, vec![velho], "o preenchimento velho tem de sair");
    assert!(formas.is_empty(), "ele nao e' uma forma, e' a propria regiao");
}

/// ⭐⭐⭐ **TODAS as formas pintadas sob o ponto perdem o `fill`, e não só a da frente.**
///
/// A lei é do Enio, no mesmo dia: *"na área de intersecção de duas formas sem weld, teremos dois
/// preenchimentos sobrepostos. Eles devem ser deletados para entrar o do balde."*
///
/// ⛔ A fixtura é assimétrica de propósito — o ponto está na intersecção das DUAS, e uma terceira
/// forma pintada fica de fora: um gate que só medisse a intersecção não distinguiria *"todas as que
/// contêm o ponto"* de *"todas as que existem"*.
#[test]
fn every_shape_fill_under_the_click_is_cleared_not_only_the_front_one() {
    let mut scene = VecScene::new();
    let a = forma_pintada(&mut scene, 0.0, 0.0, 10.0);
    let b = forma_pintada(&mut scene, 5.0, 5.0, 10.0);
    let longe = forma_pintada(&mut scene, 100.0, 100.0, 10.0);
    let (baldes, mut formas) = tinta_sob(&scene, &VecXforms::new(), &|_| false, [7.0, 7.0]);
    formas.sort_unstable();
    assert!(baldes.is_empty(), "nenhuma delas e' preenchimento do balde");
    assert_eq!(
        formas,
        {
            let mut esperado = vec![a, b];
            esperado.sort_unstable();
            esperado
        },
        "as DUAS que se sobrepoem no ponto tem de sair, e a de longe tem de ficar (longe={longe})"
    );
}

/// ⛔ **Uma forma SEM `fill` não é tinta** — o balde não lhe toca, e ela continua a ser a parede que
/// cerca a região que ele está a pintar.
#[test]
fn a_shape_without_fill_is_never_touched_by_the_bucket() {
    let mut scene = VecScene::new();
    let so_traco = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 10.0)],
        closed: true,
        ..VecPath::default()
    });
    let (baldes, formas) = tinta_sob(&scene, &VecXforms::new(), &|_| false, [5.0, 5.0]);
    assert!(
        baldes.is_empty() && formas.is_empty(),
        "a forma sem preenchimento ({so_traco}) nao tem tinta nenhuma para apagar"
    );
}

/// ⚠️⚠️ **A POSE entra na conta.** O clique é MUNDO e a geometria guardada é LOCAL: sem descer o
/// ponto ao espaço do caminho, uma forma arrastada para longe seria apagada por um clique que nunca
/// esteve dentro dela.
#[test]
fn the_click_comes_down_to_the_paths_own_space_before_asking() {
    let mut scene = VecScene::new();
    let id = forma_pintada(&mut scene, 0.0, 0.0, 10.0);
    let mut xf = VecXforms::new();
    // Uma translação de 100 em x: `[a, b, c, d, e, f]` com `(e, f)` a translação.
    xf.insert(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 100.0, 0.0]));
    let (_, dentro) = tinta_sob(&scene, &xf, &|_| false, [105.0, 5.0]);
    assert_eq!(dentro, vec![id], "no MUNDO o clique caiu dentro dela");
    let (_, fora) = tinta_sob(&scene, &xf, &|_| false, [5.0, 5.0]);
    assert!(
        fora.is_empty(),
        "a forma ja' nao esta' na origem — medir em LOCAL apagaria a tinta errada"
    );
}

/// ⭐⭐⭐ **AS DUAS ACÇÕES SÃO DIFERENTES** — o preenchimento do balde é APAGADO, a forma é LIMPA.
///
/// ⚠️ **É a metade que a lei pura não prova.** O `tinta_sob` só diz *quem*; trocar os dois lados
/// (apagar a forma e limpar o preenchimento) passaria por ele — e apagaria a PAREDE que cerca a
/// região que se está a pintar.
#[test]
fn the_bucket_erases_its_own_paint_and_only_strips_the_shapes() {
    let mut scene = VecScene::new();
    let velho = forma_pintada(&mut scene, 0.0, 0.0, 10.0);
    let forma = forma_pintada(&mut scene, 0.0, 0.0, 10.0);
    let (apagados, limpos) =
        apagar_tinta_sob(&mut scene, &VecXforms::new(), &|id| id == velho, [5.0, 5.0]);
    assert_eq!((apagados, limpos), (1, 1));
    assert!(
        scene.paths().iter().all(|p| p.id != velho),
        "o preenchimento do balde tem de DESAPARECER — ele e' a propria regiao"
    );
    let sobrevivente = scene
        .paths()
        .iter()
        .find(|p| p.id == forma)
        .expect("a forma tem de SOBREVIVER: ela e' a parede que cerca a regiao");
    assert!(
        sobrevivente.fill.is_none(),
        "…mas sem a tinta antiga, que e' o que o balde veio substituir"
    );
    assert!(
        !sobrevivente.verts.is_empty(),
        "e com a geometria intacta"
    );
}
