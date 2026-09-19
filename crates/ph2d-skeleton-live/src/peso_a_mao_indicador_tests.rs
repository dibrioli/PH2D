//! ⭐⭐⭐ **OS GATES DO INDICADOR DO PESO** — o que o artista VÊ, e sobre que arte.
//!
//! # Porque eles são um ficheiro e não os do irmão
//!
//! O [`super::peso_a_mao_tests`] mede o que uma pincelada **FAZ** ao documento (a mancha, a fusão,
//! o tecto, a âncora no repouso). Isto mede o que o app **MOSTRA** antes de ela acontecer: que arte
//! está sob o dedo, quantos pontos aparecem e de que cor. ⛔ São perguntas com fixturas diferentes
//! — a do irmão é um palco mínimo, a destes é a peça REAL da cena do dono — e um ficheiro só passou
//! o tecto de LOC no dia em que o segundo report chegou.
//!
//! ⚠️ **Todos nasceram de um report do dono** (2026-09-19): *«nada fica vermelho e nada fica azul»*
//! e *«os pesos não são aplicados apenas nos nós, mas também nos handles»*.

use super::*;
use ph2d_ecs::{ChildOf, Name, RootOrder, Transform};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook};

/// A `pixels_per_meter` das fixturas — ver o irmão.
const PPM: f32 = 100.0;

/// A distância entre dois pontos.
fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Um osso, com a pose LOCAL — ver [`barra_da_cena`].
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
            ph2d_skeleton_ecs::Bone {
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

fn forma(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(*map.get(&id).expect("a forma tem entidade"))
}

/// ⭐⭐⭐ **A ESCALA DA BARRA DA CENA — a tabela de que o raio de fábrica do pincel foi derivado.**
///
/// ⛔⛔ **Ela existe porque o número que ela justifica estava errado por `2,85 ×`** (report do dono,
/// 2026-09-19: *«os pontos não ficam coloridos»*). O `WEIGHT_RADIUS_DEFAULT` era `20` em unidades
/// de MUNDO — `2 000` px — contra uma peça de `702` px: um clique agarrava todos os pontos dela.
///
/// ⚠️ **O sujeito é a peça REAL da cena** (o `RoundRect` do braço de `PH2D_VEC_BONE_SMOKE=1`), e
/// não um rectângulo de conveniência: *uma tabela medida noutra arte descreve outro programa*.
///
/// ⚠️ **Ela afirma os DOIS extremos e não só um:** o raio de fábrica tem de ser maior que a
/// distância entre dois pontos vizinhos (senão o pincel é um apontador) e muito menor que a peça
/// (senão ele não aponta a sítio nenhum). *Uma sonda que só imprimisse seria uma nota que
/// envelhece* — e esta reprova no dia em que a arte da cena mudar de escala.
#[test]
fn a_escala_da_barra_e_a_que_a_tabela_do_raio_cita() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    // ⚠️ A cadeia real e' feita de poses LOCAIS: so' a raiz esta' no mundo, e cada filho nasce na
    // ponta do pai. Com absolutas ela dobra-se sobre si mesma e os pesos colapsam no primeiro osso
    // — foi o que a 1.a redaccao desta sonda mediu, e lia-se como um defeito do produto.
    let passo = (-1.8f64 - -8.2) / 3.0;
    let mut pai = None;
    for k in 0..3 {
        let pos = if k == 0 {
            [-8.2, 2.5]
        } else {
            [passo as f32, 0.0]
        };
        pai = Some(osso(&mut sim, &format!("Bone {}", k + 1), pos, passo, pai));
    }
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    let pts = crate::peso_a_mao::repousos(&sim, forma(&map, id), PPM);
    let mut d: Vec<[f64; 2]> = Vec::new();
    for q in &pts {
        if !d.iter().any(|x| dist(*x, *q) < 1e-9) {
            d.push(*q);
        }
    }
    assert!(
        d.len() >= 8,
        "a barra da cena tem {} posicoes distintas — a tabela do raio foi medida sobre 14, e com \
         menos que isto ela deixou de a descrever",
        d.len()
    );
    let mut viz = f64::INFINITY;
    let mut peca: f64 = 0.0;
    for i in 0..d.len() {
        for j in (i + 1)..d.len() {
            let s = dist(d[i], d[j]);
            if s > 1e-9 {
                viz = viz.min(s);
            }
            peca = peca.max(s);
        }
    }
    let (viz_px, peca_px) = (viz * f64::from(PPM), peca * f64::from(PPM));
    // O raio de fábrica, repetido AQUI de propósito: esta crate não conhece a `ph2d-tool-vector`
    // (a dependência seria ao contrário), então o que o gate pode afirmar é a FAIXA em que o
    // número tem de cair. ⛔ Mudá-lo lá sem o mudar aqui deixa a tabela do doc a mentir.
    const RAIO_DE_FABRICA_PX: f64 = 40.0;
    assert!(
        RAIO_DE_FABRICA_PX > viz_px * 1.2,
        "o raio de fabrica ({RAIO_DE_FABRICA_PX} px) nao chega a cobrir uma vizinhanca: dois \
         pontos vizinhos desta peca estao a {viz_px:.1} px — o pincel virou um apontador"
    );
    assert!(
        RAIO_DE_FABRICA_PX < peca_px * 0.25,
        "o raio de fabrica ({RAIO_DE_FABRICA_PX} px) e' grande demais para a peca ({peca_px:.1} \
         px): era assim que o `20.0` em unidades de MUNDO agarrava todos os pontos de uma vez"
    );
}

/// **A barra da cena, com a cadeia de 3 ossos** — a fixtura dos gates do report de 2026-09-19.
///
/// ⚠️ **É a peça REAL** (o `RoundRect` do braço de `PH2D_VEC_BONE_SMOKE=1`) e a cadeia é feita de
/// poses LOCAIS: só a raiz está no mundo, e cada filho nasce na ponta do pai. ⛔ Com absolutas ela
/// dobra-se sobre si mesma e todo o peso colapsa no primeiro osso — foi o que a 1.ª redacção destas
/// sondas mediu, e lia-se exactamente como um defeito do produto.
fn barra_da_cena() -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let passo = (-1.8f64 - -8.2) / 3.0;
    let mut pai = None;
    let mut ids = Vec::new();
    for k in 0..3 {
        let pos = if k == 0 {
            [-8.2, 2.5]
        } else {
            [passo as f32, 0.0]
        };
        let e = osso(&mut sim, &format!("Bone {}", k + 1), pos, passo, pai);
        pai = Some(e);
        ids.push(e);
    }
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    // ⛔⛔ **A FORMA CARREGA UM `Sprite`, e sem ele esta fixtura não contém o fenómeno** — no app
    // toda arte vectorial tem um, e a 1.ª redacção da porta escolhia o ramo da mídia por
    // `tem Sprite?`: ali o `SkinnedMesh` não parseia, a porta respondia «não achei» e a tela ficava
    // sem pontos. *Só a FOTOGRAFIA o mostrou — a fixtura de unidade não tinha `Sprite` e estava
    // verde sobre o defeito.*
    let alvo = forma(&map, id);
    sim.world_mut()
        .entity_mut(alvo)
        .insert(ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]));
    (sim, scene, map, id, ids)
}

/// O raio do pincel de fábrica, em unidades de MUNDO a esta `ppm`.
fn raio_de_fabrica() -> f64 {
    40.0 / f64::from(PPM)
}

/// ⭐⭐⭐ **A ARTE SOB O CURSOR É A QUE O CONTÉM — e não a que tem um VÉRTICE perto.**
///
/// ⛔⛔⛔ **O defeito que este gate cura é o report de 2026-09-19** (*«nada fica vermelho e nada
/// fica azul»*): a porta media a distância ao **ponto** posado mais perto, e os pontos de uma forma
/// vivem nos CANTOS dela. Medido nesta barra, o cursor no meio do comprimento achava **nada** em
/// `7` de `9` posições ⇒ sem arte não há pontos, e a tela ficava vazia.
///
/// ⚠️⚠️ **E quem o expôs foi a minha própria cura anterior:** enquanto o raio valia `20` unidades
/// de MUNDO (`2 000` px) a arte era sempre encontrada, por acidente. *Um número a fazer dois
/// trabalhos esconde o defeito do segundo enquanto estiver errado no primeiro* — aqui eram *«até
/// onde o pincel alcança»* e *«que arte está debaixo do dedo»*, e o segundo nunca foi uma distância.
#[test]
fn o_cursor_no_meio_da_arte_acha_a_arte() {
    let (sim, _scene, map, id, _) = barra_da_cena();
    let alvo = forma(&map, id);
    let r = raio_de_fabrica();
    let dentro: Vec<f64> = vec![-8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0];
    for x in &dentro {
        assert_eq!(
            crate::peso_a_mao::pele_sob_o_cursor(&sim, PPM, [*x, 2.5], r),
            Some(alvo),
            "o cursor DENTRO da barra (x={x}) nao achou a arte — e' o report de 19/09, e sem arte \
             nao ha' pontos nenhuns para colorir"
        );
    }
    // ⭐ A metade NEGATIVA: longe da peça ela continua a não ser encontrada, senão o pincel pintaria
    // a arte errada a partir de qualquer sítio do canvas — que é o defeito que a cura podia trazer.
    assert_eq!(
        crate::peso_a_mao::pele_sob_o_cursor(&sim, PPM, [-5.0, 9.0], r),
        None,
        "o cursor LONGE da barra achou-a na mesma — a contencao passou a aceitar o canvas inteiro"
    );
}

/// ⭐⭐⭐ **UM PONTO POR NÓ, E O PESO DAS ALÇAS É O DO NÓ.**
///
/// ⛔⛔ **Report do dono, duas vezes (2026-09-19):** *«parece que os pesos não são aplicados apenas
/// nos nós, mas também nos handles»* e depois *«o algoritmo continua considerando pesos em alças e
/// não apenas nos pontos»*. A 1.ª resposta desta casa curou só o DESENHO (numa forma de cantos
/// arredondados as alças ficam em posições distintas, logo `8` nós apareciam como **`24`
/// pontinhos**) e deixou a LEI como estava — é o que o segundo report veio dizer.
///
/// ⚠️ **As duas metades, porque as curas seriam opostas:** o indicador mostra `8` pontos, e a lei
/// continua a devolver `24` POSIÇÕES — as alças são desenhadas, logo têm de ser deformadas.
/// O que elas deixaram de ter é peso PRÓPRIO.
///
/// ⛔⛔ **A 3.ª metade é uma coerência e NÃO o discriminador, e dizê-lo é o que a torna honesta:**
/// **no repouso** as duas leis concordam nesta arte (as alças de uma quina arredondada ficam a
/// `0,28` da âncora e os pesos derivados dão o mesmo). Quem as separa são dois gates construídos
/// para isso: o [`um_dab_chega_inteiro_as_duas_alcas_do_no`] — **nesta mesma arte, depois de uma
/// pincelada**, que é onde o report do dono vive — e o `ph2d_vec_skin` com uma forma de alças
/// longas a atravessar uma junta. *Os dois trazem o controlo positivo dentro.*
#[test]
fn o_indicador_mostra_um_ponto_por_no_e_a_alca_pesa_o_do_no() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let todos = crate::peso_a_mao::pontos_da_pele(&sim, alvo, ossos[0], PPM);
    let vistos = crate::peso_a_mao::pontos_do_indicador(&sim, PPM, Some(ossos[0]));
    assert_eq!(
        todos.len(),
        vistos.len() * 3,
        "a lei devia guardar TRES pontos por no' (ancora + duas alcas) e o indicador mostrar UM: \
         lei {} · vistos {}",
        todos.len(),
        vistos.len()
    );
    assert!(
        vistos.len() >= 4,
        "o indicador mostrou {} pontos — com menos que isto a fixtura deixou de ter forma",
        vistos.len()
    );
    // ⭐ E o que se mostra é a ÂNCORA, não uma média: cada ponto visto é um dos `3k` da lei.
    for (i, (p, w)) in vistos.iter().enumerate() {
        let ancora = &todos[i * 3];
        assert!(
            dist(*p, ancora.mundo) < 1e-9 && (w - ancora.peso).abs() < 1e-9,
            "o ponto {i} do indicador nao e' a ancora do no' {i}"
        );
    }
    // ⭐ A 3.ª metade: as duas alças de cada nó carregam o peso DELE, e não um seu.
    for k in 0..todos.len() / 3 {
        let (a, entra, sai) = (&todos[k * 3], &todos[k * 3 + 1], &todos[k * 3 + 2]);
        assert!(
            (entra.peso - a.peso).abs() < 1e-12 && (sai.peso - a.peso).abs() < 1e-12,
            "o no' {k} tem alcas com peso proprio ({} / {} contra {}) — o peso deixou de ser do NO'",
            entra.peso,
            sai.peso,
            a.peso
        );
    }
}

/// ⭐⭐⭐ **COM UM OSSO QUE POSSUI METADE DA ARTE, A TELA TEM AS DUAS CORES.**
///
/// ⛔ *«nada fica vermelho e nada fica azul»* — e o que faltava não era a rampa (os dois extremos
/// dela são tokens de hue `25` e `235`, vermelho e azul de verdade): era não haver **ponto nenhum**.
///
/// ⚠️ **O `Bone 2` fica de fora de propósito e é a metade que ensina:** o osso do MEIO de uma cadeia
/// de três sobre oito nós não possui nada, logo ali uma cor só é a resposta CERTA. *Foi ele que o
/// meu passo de smoke mandou clicar.*
#[test]
fn com_o_osso_certo_a_tela_tem_as_duas_cores() {
    let (sim, _scene, map, id, ossos) = barra_da_cena();
    let _ = forma(&map, id);
    for (n, o) in [(1usize, ossos[0]), (3, ossos[2])] {
        let v = crate::peso_a_mao::pontos_do_indicador(&sim, PPM, Some(o));
        let quentes = v.iter().filter(|(_, w)| *w > 0.5).count();
        let frios = v.iter().filter(|(_, w)| *w <= 0.5).count();
        assert!(
            quentes > 0 && frios > 0,
            "Bone {n}: a tela tem UMA cor so' ({quentes} quentes, {frios} frios) — e' o report de \
             19/09 a` letra"
        );
    }
    let meio = crate::peso_a_mao::pontos_do_indicador(&sim, PPM, Some(ossos[1]));
    assert!(
        !meio.is_empty() && meio.iter().all(|(_, w)| *w <= 0.5),
        "o osso do MEIO passou a possuir alguma coisa nesta arte — a lei mudou, e o passo do smoke \
         que o nomeava deixou de estar errado pelo motivo que o handoff regista"
    );
}

/// ⭐⭐⭐ **UM DAB DO ARTISTA CHEGA INTEIRO ÀS DUAS ALÇAS DO NÓ** — a medição que dá razão ao dono.
///
/// ⛔⛔⛔ **Este é o defeito do report de 2026-09-19, medido na arte DELE** (a barra de
/// `PH2D_VEC_BONE_SMOKE=1`). Com o peso por METADE, a mancha do pincel é um bump radial no espaço:
/// ela vale `1` no centro — que é a âncora — e menos nas alças, que estão ao lado. Um dab de
/// `amount = 1` sobre um nó entrega:
///
/// | ponto | lei de hoje (peso do NÓ) | lei de ontem (peso da posição) |
/// |---|---|---|
/// | âncora | `0,5000` | `0,5000` |
/// | alça de entrada | `0,5000` | **`0,2150`** |
/// | alça de saída | `0,5000` | `0,5000` |
///
/// ⭐⭐ **E o mais duro é a assimetria:** das duas alças do MESMO nó, uma seguia e a outra não — é
/// a tangente a partir-se exactamente no ponto que o artista acabou de pintar. *Um peso que o
/// artista não consegue entregar ao nó inteiro num gesto não é um peso que ele controla.*
///
/// ⚠️ **Sem o dab as duas leis CONCORDAM nesta arte** (as alças de uma quina arredondada ficam a
/// `0,28` da âncora e os pesos derivados são iguais) — é a pincelada que as separa. *É por isso que
/// este gate pinta antes de medir, e é por isso que a régua que só olhava o repouso não via nada.*
#[test]
fn um_dab_chega_inteiro_as_duas_alcas_do_no() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();
    // O no' mais perto da ponta esquerda, que e' o que o Bone 1 governa.
    let quem = ossos[2];
    let pts = crate::peso_a_mao::pontos_de_peso(&sim, alvo, quem, PPM);
    let (i, alvo_no) = pts
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.mundo[0].partial_cmp(&b.1.mundo[0]).unwrap())
        .map(|(i, p)| (i, *p))
        .unwrap();
    assert!(
        alvo_no.peso < 0.01,
        "a fixtura precisa de um no' que este osso NAO governa (peso {:.4}), senao o dab satura e \
         as duas leis concordam por acidente",
        alvo_no.peso
    );
    let r = crate::peso_a_mao::pinta(&mut sim, alvo, quem, PPM, alvo_no.mundo, raio, 1.0);
    assert!(
        matches!(r, crate::peso_a_mao::Pincelada::Pintada { .. }),
        "a pincelada foi recusada ({r:?}) — sem mancha este gate nao mede nada"
    );
    // Agora leio a ancora e as duas alcas pela lei de HOJE (peso do no') e pela de ONTEM (peso da posicao).
    let todos = crate::peso_a_mao::pontos_da_pele(&sim, alvo, quem, PPM);
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .unwrap()
        .clone();
    let pele = crate::skin_live::skin_of(&sim, alvo).unwrap();
    let correcoes = skin.correcoes_resolvidas();
    let repousos = crate::peso_a_mao::repousos(&sim, alvo, PPM);
    let mut w = pele.scratch();
    let tendao = skin
        .tendons
        .iter()
        .position(|t| t.bone == *sim.world().get::<ph2d_ecs::StableId>(quem).unwrap())
        .unwrap();
    let mut divergiu = 0;
    for k in (i * 3)..(i * 3 + 3) {
        pele.weights_corrected(repousos[k], None, &mut w, &correcoes);
        let ontem = w.get(tendao).copied().unwrap_or(0.0);
        let hoje = todos[k].peso;
        let nome = ["ancora", "alca-entra", "alca-sai"][k - i * 3];
        eprintln!("[peso] {nome}: hoje {hoje:.4} · lei de ontem {ontem:.4}");
        assert!(
            (hoje - todos[i * 3].peso).abs() < 1e-12,
            "o {nome} do no' nao recebeu o peso do NO' ({hoje:.4} contra {:.4})",
            todos[i * 3].peso
        );
        if (ontem - todos[i * 3].peso).abs() > 0.05 {
            divergiu += 1;
        }
    }
    // ⭐ O CONTROLO: a lei de ontem TEM de discordar em pelo menos uma das metades, senao esta
    // fixtura deixou de conter o fenomeno e as asserções acima passam a ser verdades triviais.
    assert!(
        divergiu >= 1,
        "as duas leis concordam nas tres metades — a fixtura deixou de discriminar, e este gate \
         deixou de afirmar o que o report de 19/09 pediu"
    );

    // ⭐⭐⭐ **A 4.ª metade: a mancha é ANCORADA no NÓ, mesmo com o dedo mais perto de uma alça.**
    // ⛔ «pesos em alças» tinha TRÊS sítios, e este é o que ninguém tinha visto: o
    // [`crate::peso_a_mao::ponto_sob_o_cursor`] escolhia entre TODOS os pontos, logo o centro de uma
    // correcção podia cair numa alça — um sítio cujo peso já ninguém lê.
    let alca = repousos[i * 3 + 1];
    let no = repousos[i * 3];
    let rumo = [(alca[0] - no[0]) * 0.8, (alca[1] - no[1]) * 0.8];
    assert!(
        rumo[0].hypot(rumo[1]) > 1e-6,
        "a alca coincide com a ancora nesta fixtura — o dedo nao consegue ficar mais perto dela, e \
         esta metade nao mede nada"
    );
    let dedo = crate::skin_live::world_of(&sim, alvo).apply([no[0] + rumo[0], no[1] + rumo[1]]);
    let antes = skin.correcoes.len();
    crate::peso_a_mao::pinta(&mut sim, alvo, quem, PPM, dedo, raio, 0.3);
    let depois = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .expect("a pele")
        .correcoes
        .clone();
    assert!(depois.len() >= antes, "a pincelada perdeu manchas");
    let dist = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    assert!(
        depois
            .iter()
            .all(|c| dist(c.centro, no) < dist(c.centro, alca) + 1e-12),
        "uma mancha foi ancorada mais perto de uma ALCA do que do NO' — o centro da correccao caiu \
         num sitio cujo peso ja' ninguem le^"
    );
}

/// ⭐⭐⭐ **O QUE O DEDO APANHA É O QUE O OLHO VÊ** — o instantâneo posado do hit-test é, ponto a
/// ponto, a geometria que o quadro DESENHA.
///
/// ⛔⛔ **Este gate nasceu de uma mutação SOBREVIVENTE:** fazer o [`crate::peso_a_mao::posados`]
/// voltar a pesar cada metade pela posição DELA não reprovava nada. Ele alimenta a silhueta do
/// [`crate::peso_a_mao::pele_sob_o_cursor`], logo uma divergência ali é *o dedo a tocar num sítio
/// com a arte noutro* — o defeito mais caro desta casa, e nenhuma régua o via.
///
/// ⚠️⚠️ **A fixtura tem de conter o fenómeno, e no REPOUSO ela não contém:** as duas leis de peso
/// concordam numa quina arredondada. É preciso **uma pincelada** (a mancha é um bump radial e
/// separa-as) **e** um osso fora da pose de repouso (senão tudo é a identidade). O gate arma as
/// duas coisas e depois compara contra o `recook`, que é o que de facto desenha.
#[test]
fn o_instantaneo_do_hit_test_e_a_geometria_desenhada() {
    let (mut sim, mut scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    // (a) um osso fora do repouso — sem isto toda a pele e' a identidade.
    sim.world_mut()
        .get_mut::<Transform>(ossos[1])
        .expect("o osso tem pose")
        .rotation += 25.0_f32.to_radians();
    // (b) uma pincelada — e' ela que separa a lei do NO' da lei da POSICAO nesta arte.
    let pts = crate::peso_a_mao::pontos_de_peso(&sim, alvo, ossos[2], PPM);
    let no = pts
        .iter()
        .min_by(|a, b| a.mundo[0].partial_cmp(&b.mundo[0]).expect("finito"))
        .copied()
        .expect("a barra tem nos");
    let r = crate::peso_a_mao::pinta(
        &mut sim,
        alvo,
        ossos[2],
        PPM,
        no.mundo,
        raio_de_fabrica(),
        1.0,
    );
    assert!(
        matches!(r, crate::peso_a_mao::Pincelada::Pintada { .. }),
        "sem mancha as duas leis concordam e este gate nao mede nada ({r:?})"
    );

    crate::skin_live::recook(&sim, &mut scene);
    let desenhado = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o caminho da cena");
    let x = crate::skin_live::world_of(&sim, alvo);
    let visto = crate::peso_a_mao::posados(&sim, alvo, PPM);
    // ⛔⛔⛔ **A PREMISSA DESTE GATE MUDOU COM A F30, e a lei que ele defende não.** Ele dizia *«um
    // ponto do instantâneo por ponto DESENHADO»* — e com a lei da curva o desenho tem **mais** nós
    // que a fonte (`24` contra `36`, medido), porque a imagem de uma cúbica por um mapa não-afim
    // precisa de mais pedaços. ⚠️ *E ter menos pontos é a resposta CERTA:* o peso vive nos nós da
    // FONTE, e é só neles que o artista pode pousar uma mancha — um ponto do indicador onde não há
    // peso para corrigir seria um controlo morto. ⇒ a régua passa a ser a que a lei sempre foi:
    // **o instantâneo tem um ponto por ponto da FONTE, e cada âncora dele cai SOBRE o desenho**.
    let repousos_n = crate::peso_a_mao::repousos(&sim, alvo, PPM).len();
    assert_eq!(
        visto.len(),
        repousos_n,
        "o instantaneo do hit-test deixou de ter um ponto por ponto da FONTE"
    );
    // ⭐ **O CONTROLO: a pose TEM de mover a arte**, senão comparar duas identidades passa sempre.
    //
    // ⚠️ **Ele compara o REPOUSO com o INSTANTÂNEO, e não com o desenho** (F30): os dois têm um
    // ponto por ponto da FONTE, enquanto o desenho tem mais — *emparelhar por índice duas listas de
    // tamanhos diferentes compara coisas que não são a mesma*. ⛔ E ele tinha sido **perdido** na
    // 1.ª reescrita deste gate: um gate sem controlo positivo mede o nada e fica verde.
    let andou = crate::peso_a_mao::repousos(&sim, alvo, PPM)
        .iter()
        .zip(&visto)
        .map(|(a, b)| {
            let p = x.apply(*a);
            (p[0] - b[0]).hypot(p[1] - b[1])
        })
        .fold(0.0_f64, f64::max);
    assert!(
        andou > 0.05,
        "a fixtura nao move a arte ({andou:.4}) — comparar duas identidades passa sempre"
    );
    // ⭐ **Cada ÂNCORA do instantâneo cai sobre a curva desenhada.** ⚠️ Só as âncoras: uma alça de
    // Bézier vive FORA da curva por definição, e exigir que ela caia lá acusaria toda tangente.
    let curva = polilinha_mundo(desenhado, x);
    let pior = visto
        .iter()
        .enumerate()
        .filter(|(k, _)| crate::peso_a_mao::dono_do_peso(true, *k) == *k)
        .map(|(_, a)| ph2d_skeleton::dist2_to_polyline(*a, &curva).sqrt())
        .fold(0.0_f64, f64::max);
    assert!(
        pior < 1e-6,
        "o hit-test ve^ uma ancora {pior:.6} fora da curva desenhada — o dedo toca num sitio e o \
         desenho esta' noutro"
    );
}

/// A curva desenhada, amostrada em MUNDO — o que o olho vê.
fn polilinha_mundo(p: &ph2d_vec_scene::VecPath, x: ph2d_vec_scene::Xform) -> Vec<[f64; 2]> {
    const N: usize = 64;
    let cozido = p.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let n = verts.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            for k in 0..=N {
                let t = k as f64 / N as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push(x.apply([
                    w0.mul_add(
                        a.anchor[0],
                        w1.mul_add(
                            a.out_handle[0],
                            w2.mul_add(b.in_handle[0], w3 * b.anchor[0]),
                        ),
                    ),
                    w0.mul_add(
                        a.anchor[1],
                        w1.mul_add(
                            a.out_handle[1],
                            w2.mul_add(b.in_handle[1], w3 * b.anchor[1]),
                        ),
                    ),
                ]));
            }
        }
    }
    out
}
