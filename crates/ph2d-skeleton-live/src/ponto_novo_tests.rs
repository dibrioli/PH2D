//! Os gates do [`super`] — o ponto novo entra na fonte, a tabela fecha, e a forma quase não se move.

use super::*;
use ph2d_ecs::{ChildOf, Name, RootOrder, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::{ShapeKind, VecScene, cook};

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

/// Um rectângulo deitado com um braço de dois ossos por cima — a fixtura das outras suítes desta
/// crate, e ela tem tabela de pesos porque a forma é FECHADA.
fn palco() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let id = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let n = crate::skin_live::bind(&mut sim, &cena, &mapa, &[id], Some(raiz));
    assert_eq!(n, 1, "o palco tem de prender");
    (sim, cena, mapa, id, [raiz, ponta])
}

/// O mesmo palco, mas com a aresta de baixo **DESENHADA** com `pedacos` pedaços antes do bind — o
/// que um artista faz quando quer controlo ali.
///
/// ⚠️ **Os cortes são ANTES do bind**, e é isso que faz a diferença: a tabela de pesos é resolvida
/// sobre o caminho já subdividido, logo cada segmento cobre pouca variação de peso.
fn palco_desenhado(pedacos: usize) -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
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
    let n = crate::skin_live::bind(&mut sim, &cena, &mapa, &[id], Some(raiz));
    assert_eq!(n, 1, "o palco desenhado tem de prender");
    (sim, cena, mapa, id, [raiz, ponta])
}

fn fonte(sim: &SimWorld, mapa: &VecEntityMap, id: VecPathId) -> crate::skinned_mesh::SkinnedPath {
    let e = Entity::from_bits(*mapa.get(&id).expect("entidade"));
    let skin = sim.world().get::<SkinBind>(e).expect("presa");
    crate::skinned_mesh::le(&skin.source).expect("a fonte le-se")
}

fn ancoras(cena: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
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
        insere_ponto(&mut sim, &mapa, id, 0, 0.5).is_some(),
        "a porta recusou uma forma que ESTA' presa"
    );

    assert_eq!(
        fonte(&sim, &mapa, id).path.verts_all().count(),
        antes + 1,
        "a FONTE nao ganhou o ponto — o quadro seguinte deita-o fora"
    );
    crate::skin_live::recook(&sim, &mut cena);
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
        insere_ponto(&mut sim, &mapa, id, 0, 0.5),
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

    let ni = insere_ponto(&mut sim, &mapa, id, 0, 0.5).expect("insere");
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

/// ⭐⭐⭐ **A FORMA QUASE NÃO SE MOVE, e o «quase» é medido e tem mecanismo.**
///
/// O desenho é uma Bézier cujos pontos de controlo saem de **duas** linhas de pesos, e um corte de
/// de Casteljau na geometria de repouso só comuta com a deformação quando as duas são iguais. ⇒ o
/// desvio é proporcional a quanto o peso varia ao longo daquele segmento, e a barra sai da **medição
/// com a pose DOBRADA**, que é o pior caso do palco.
///
/// ⛔ **A barra é uma FRACÇÃO da peça e não um número absoluto** — uma barra em unidades de mundo
/// mediria o tamanho da fixtura.
#[test]
fn o_ponto_novo_quase_nao_move_a_forma() {
    let (mut sim, mut cena, mapa, id, [_, ponta]) = palco();
    // A pose DOBRADA: é com o osso virado que as duas linhas de peso de um segmento divergem.
    sim.world_mut()
        .get_mut::<Transform>(ponta)
        .expect("Transform")
        .rotation = 0.8;
    crate::skin_live::recook(&sim, &mut cena);
    let antes = ancoras(&cena, id);

    assert!(
        insere_ponto(&mut sim, &mapa, id, 0, 0.5).is_some(),
        "a porta recusou uma forma que ESTA' presa"
    );
    crate::skin_live::recook(&sim, &mut cena);
    let depois = ancoras(&cena, id);
    assert_eq!(depois.len(), antes.len() + 1);

    // As âncoras VELHAS têm de ficar onde estavam — o ponto novo entra entre a 0 e a 1.
    let mut pior = 0.0_f64;
    for (k, p) in antes.iter().enumerate() {
        let q = if k == 0 { depois[0] } else { depois[k + 1] };
        pior = pior.max((p[0] - q[0]).hypot(p[1] - q[1]));
    }
    let diagonal = 40.0_f64.hypot(10.0);
    let fraccao = pior / diagonal;
    eprintln!(
        "[ponto-novo] pior desvio de ancora = {pior:.6} ({:.4} % da peca)",
        fraccao * 100.0
    );
    assert!(
        fraccao < 1e-9,
        "as ancoras VELHAS moveram-se {fraccao} da peca — o corte mexeu em quem nao devia"
    );
}

/// A curva desenhada, amostrada — o que o olho vê, e não os pontos de controlo.
fn polilinha(cena: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    const AMOSTRAS: usize = 24;
    let Some(caminho) = cena.path(id) else {
        return Vec::new();
    };
    let cozido = caminho.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let n = verts.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            for k in 0..AMOSTRAS {
                let t = k as f64 / AMOSTRAS as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w0 * a.anchor[0]
                        + w1 * a.out_handle[0]
                        + w2 * b.in_handle[0]
                        + w3 * b.anchor[0],
                    w0 * a.anchor[1]
                        + w1 * a.out_handle[1]
                        + w2 * b.in_handle[1]
                        + w3 * b.anchor[1],
                ]);
            }
        }
    }
    out
}

/// ⭐⭐⭐ **E a CURVA desenhada — o número que o doc do módulo afirma, nos DOIS lados.**
///
/// As âncoras ficam exactas (o gate acima), mas as ALÇAS mudam: o corte de de Casteljau reescreve o
/// `out` do vizinho anterior e o `in` do seguinte. ⚠️ **É aí que o desvio mora**, e ele é inerente:
/// o desenho cozido é a Bézier dos pontos de controlo **deformados**, e não a imagem verdadeira da
/// curva de repouso pela pele — logo ele já é uma aproximação, e cada pedaço a mais refina-a (ver
/// `a_escada_da_subdivisao_diz_se_o_salto_e_refinamento`, que mede a convergência).
///
/// ⛔⛔ **A barra é sobre a forma DESENHADA e não sobre a grosseira, e a razão está medida:** num
/// rectângulo cru a aresta de baixo é **UM** segmento a atravessar os **dois** ossos, logo as duas
/// pontas dele têm pesos opostos e o primeiro corte vale `~19 %` da peça. *Isso não é o custo de
/// acrescentar um ponto — é o tamanho do erro que aquele único segmento já tinha*, e o corte
/// mostra-o.
///
/// ⚠️⚠️ **E o segmento medido é o PIOR de todos, não o primeiro.** A 1.ª redacção deste gate media o
/// segmento `0` da aresta desenhada e leu **`0,0000 %`** — os dois extremos dele estão ambos dentro
/// do primeiro osso, logo a lei preserva a forma **ao bit por construção** e a barra passava por
/// **vácuo**. Quem o apanhou foi o controlo `gap`: *uma fixtura que não contém o fenómeno não prova
/// que ele não aconteceu*. O sítio onde o peso varia é a **junta**.
///
/// ⭐ O segundo controlo é a pose em REPOUSO: ali o desvio tem de ser **zero ao bit**, senão o que se
/// está a medir é o corte e não a deformação.
#[test]
fn a_curva_desenhada_move_se_o_que_o_peso_varia_no_segmento() {
    /// O salto ao inserir no segmento `seg`, em unidades de mundo.
    fn salto(pedacos: usize, rotacao: f32, seg: usize) -> f64 {
        let (mut sim, mut cena, mapa, id, [_, ponta]) = if pedacos <= 1 {
            palco()
        } else {
            palco_desenhado(pedacos)
        };
        sim.world_mut()
            .get_mut::<Transform>(ponta)
            .expect("Transform")
            .rotation = rotacao;
        crate::skin_live::recook(&sim, &mut cena);
        let antes = polilinha(&cena, id);
        assert!(antes.len() > 8, "a fixtura tem de ter curva para medir");
        assert!(insere_ponto(&mut sim, &mapa, id, seg, 0.5).is_some());
        crate::skin_live::recook(&sim, &mut cena);
        polilinha(&cena, id)
            .iter()
            .map(|p| ph2d_skeleton::dist2_to_polyline(*p, &antes).sqrt())
            .fold(0.0_f64, f64::max)
    }

    const PEDACOS: usize = 8;
    let diagonal = 40.0_f64.hypot(10.0);

    // ⛔ O segmento onde o peso MAIS varia — e a fixtura tem de o conter.
    let (sim, _c, mapa, id, _) = palco_desenhado(PEDACOS);
    let f = fonte(&sim, &mapa, id);
    let (pior_seg, gap) = (0..PEDACOS)
        .map(|k| {
            let (a, b) = (
                f.linha_do_no(k).expect("a"),
                f.linha_do_no(k + 1).expect("b"),
            );
            let g = a
                .iter()
                .zip(b)
                .map(|(x, y)| (x - y).abs())
                .fold(0.0_f64, f64::max);
            (k, g)
        })
        .max_by(|x, y| x.1.total_cmp(&y.1))
        .expect("ha' segmentos");
    assert!(
        gap > 1e-3,
        "nenhum segmento da aresta desenhada tem pesos diferentes nas pontas ({gap}) — a fixtura \
         nao contem o fenomeno, e a barra abaixo passaria por vacuo"
    );

    let repouso = salto(PEDACOS, 0.0, pior_seg);
    let grosseiro = salto(1, 0.8, 0) / diagonal * 100.0;
    let desenhado = salto(PEDACOS, 0.8, pior_seg) / diagonal * 100.0;
    eprintln!(
        "[ponto-novo] desvio da curva: repouso={repouso:.9} · aresta CRUA={grosseiro:.4} % · \
         aresta DESENHADA em {PEDACOS} (pior segmento {pior_seg}, gap {gap:.4})={desenhado:.4} % da peca"
    );
    assert!(
        repouso < 1e-9,
        "em REPOUSO o corte mexeu na curva ({repouso}) — entao o defeito e' do corte e nao da \
         deformacao, e esta regua esta' a medir a coisa errada"
    );
    // ⛔⛔ **CATRACA MEDIDA e não um limite escolhido.** *«Acima de X % o artista vê saltar»* seria
    // um palpite — não há medição nenhuma por trás de um número desses. O que há é a MEDIÇÃO de
    // hoje (`0,911 %` no pior segmento), e a regra desta casa para uma dívida tolerada: **ela só
    // encolhe**. ⚠️ E com o censo de obsolescência ao lado, senão a catraca vira licença.
    const CATRACA_PCT: f64 = 1.0;
    assert!(
        desenhado < CATRACA_PCT,
        "no PIOR segmento de uma aresta desenhada com {PEDACOS} pedacos a curva mexeu \
         {desenhado:.4} % da peca, contra a catraca de {CATRACA_PCT} % — a lei da mistura piorou"
    );
    assert!(
        desenhado > CATRACA_PCT * 0.5,
        "o salto caiu para {desenhado:.4} %, muito abaixo da catraca de {CATRACA_PCT} % — ou alguem \
         melhorou a lei (e entao BAIXE a catraca, com o numero novo escrito aqui), ou a fixtura \
         deixou de conter o fenomeno"
    );
    assert!(
        grosseiro > desenhado * 4.0,
        "a aresta CRUA ({grosseiro:.4} %) deixou de saltar muito mais que a desenhada \
         ({desenhado:.4} %) — a fixtura grosseira ja' nao contem o fenomeno que esta nota explica"
    );
}

/// ⭐⭐⭐ **A MEDIÇÃO QUE DECIDE A LEI: o salto é CORRUPÇÃO ou é REFINAMENTO?**
///
/// ⚠️ **O desenho JÁ é uma aproximação.** O `recook` deforma **pontos de controlo** — a curva cozida
/// é a Bézier desses pontos, e não a imagem verdadeira da curva de repouso pela pele (que é
/// `t ↦ blend(repouso(t), peso(t))`, com o peso a variar ao longo do segmento). ⇒ *acrescentar um
/// ponto aumenta o número de pedaços, e a pergunta é se a aproximação MELHORA*.
///
/// A escada mede `1 → 2 → 4 → 8` pedaços no mesmo segmento e compara cada degrau com o seguinte. Se
/// os desvios **encolherem geometricamente**, a sequência converge e o salto do primeiro degrau é o
/// preço de uma aproximação grosseira a ser refinada — não um defeito da lei.
#[test]
fn a_escada_da_subdivisao_diz_se_o_salto_e_refinamento() {
    let construir = |cortes: usize| -> Vec<[f64; 2]> {
        let (mut sim, mut cena, mapa, id, [_, ponta]) = palco();
        sim.world_mut()
            .get_mut::<Transform>(ponta)
            .expect("Transform")
            .rotation = 0.8;
        // Corta o segmento 0 repetidamente ao meio, sempre na metade esquerda e na direita.
        for _ in 0..cortes {
            let n = fonte(&sim, &mapa, id).path.verts_all().count();
            // Todos os segmentos que vieram do original: eles são os primeiros `n_cortes`.
            let segs: Vec<usize> = (0..n).collect();
            for seg in segs.iter().rev() {
                // só os segmentos da aresta de baixo (entre a âncora 0 e a que era a 1)
                if *seg < cortes_da_aresta(&sim, &mapa, id) {
                    let _ = insere_ponto(&mut sim, &mapa, id, *seg, 0.5);
                }
            }
        }
        crate::skin_live::recook(&sim, &mut cena);
        polilinha(&cena, id)
    };
    let degraus: Vec<Vec<[f64; 2]>> = (0..4).map(construir).collect();
    let diagonal = 40.0_f64.hypot(10.0);
    let mut desvios = Vec::new();
    for k in 0..degraus.len() - 1 {
        let d = degraus[k + 1]
            .iter()
            .map(|p| ph2d_skeleton::dist2_to_polyline(*p, &degraus[k]).sqrt())
            .fold(0.0_f64, f64::max);
        desvios.push(d / diagonal * 100.0);
    }
    eprintln!("[ponto-novo] escada da subdivisao (% da peca): {desvios:?}");
    assert!(
        desvios.windows(2).all(|w| w[1] < w[0]),
        "a escada NAO converge: {desvios:?} — entao acrescentar um ponto nao esta' a refinar a \
         aproximacao, esta' a corrompe-la, e a lei da mistura esta' errada"
    );
}

/// Quantos segmentos a aresta de baixo tem agora (ela começa com `1`).
fn cortes_da_aresta(sim: &SimWorld, mapa: &VecEntityMap, id: VecPathId) -> usize {
    fonte(sim, mapa, id).path.verts_all().count() - 3
}
