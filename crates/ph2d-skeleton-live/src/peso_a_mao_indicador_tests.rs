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
fn barra_da_cena() -> (SimWorld, VecEntityMap, VecPathId, Vec<Entity>) {
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
    (sim, map, id, ids)
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
    let (sim, map, id, _) = barra_da_cena();
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

/// ⭐⭐⭐ **UM PONTO POR NÓ, e o peso continua a ser aplicado às ALÇAS.**
///
/// ⛔⛔ **Report do dono (2026-09-19):** *«parece que os pesos não são aplicados apenas nos nós, mas
/// também nos handles (alças)»* — **verdade**, e tem de continuar a ser: o esqueleto transforma a
/// âncora E as duas alças, e uma alça parada com a âncora a andar quebrava a curva. O que estava
/// errado era o DESENHO: numa forma de cantos arredondados as alças ficam em posições distintas,
/// logo `8` nós apareciam como **`24` pontinhos**.
///
/// ⚠️ **As duas metades, porque as curas seriam opostas:** o indicador mostra `8`, e a lei continua
/// a devolver `24` — esconder um e apagar o outro leem-se igual numa tabela.
#[test]
fn o_indicador_mostra_um_ponto_por_no_e_a_lei_guarda_as_alcas() {
    let (sim, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let todos = crate::peso_a_mao::pontos_da_pele(&sim, alvo, ossos[0], PPM);
    let vistos = crate::peso_a_mao::pontos_do_indicador(
        &sim,
        PPM,
        Some(ossos[0]),
        Some(alvo),
        None,
        raio_de_fabrica(),
    );
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
    let (sim, map, id, ossos) = barra_da_cena();
    let _ = forma(&map, id);
    for (n, o) in [(1usize, ossos[0]), (3, ossos[2])] {
        let v = crate::peso_a_mao::pontos_do_indicador(
            &sim,
            PPM,
            Some(o),
            None,
            Some([-5.0, 2.5]),
            raio_de_fabrica(),
        );
        let quentes = v.iter().filter(|(_, w)| *w > 0.5).count();
        let frios = v.iter().filter(|(_, w)| *w <= 0.5).count();
        assert!(
            quentes > 0 && frios > 0,
            "Bone {n}: a tela tem UMA cor so' ({quentes} quentes, {frios} frios) — e' o report de \
             19/09 a` letra"
        );
    }
    let meio = crate::peso_a_mao::pontos_do_indicador(
        &sim,
        PPM,
        Some(ossos[1]),
        None,
        Some([-5.0, 2.5]),
        raio_de_fabrica(),
    );
    assert!(
        !meio.is_empty() && meio.iter().all(|(_, w)| *w <= 0.5),
        "o osso do MEIO passou a possuir alguma coisa nesta arte — a lei mudou, e o passo do smoke \
         que o nomeava deixou de estar errado pelo motivo que o handoff regista"
    );
}
