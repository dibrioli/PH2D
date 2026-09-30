//! ⚠️ **A BARRA DA CENA DO DONO, num sítio só** — a peça real do `PH2D_VEC_BONE_SMOKE=1`, com o
//! esqueleto de três ossos que a governa.
//!
//! ⛔ Ela atravessa dois módulos de teste (o que o pincel MOSTRA e o que ele FAZ **entre** os nós), e
//! *uma cópia por ficheiro divergiria no primeiro ajuste* — que é a mesma razão do irmão
//! [`crate::esqueletos_tests_support`]. ⚠️ **E a fixtura é a peça do DONO de propósito:** é nela que
//! os oito nós ficam todos nas duas pontas, que é o que faz o meio da barra ser o caso difícil.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook};

/// A `pixels_per_meter` das fixturas — ver o irmão.
pub(crate) const PPM: f32 = 100.0;

/// Um osso, com a pose LOCAL — ver [`barra_da_cena`].
pub(crate) fn osso(
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

pub(crate) fn forma(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(*map.get(&id).expect("a forma tem entidade"))
}

/// **A barra da cena, com a cadeia de 3 ossos** — a fixtura dos gates do report de 2026-09-19.
///
/// ⚠️ **É a peça REAL** (o `RoundRect` do braço de `PH2D_VEC_BONE_SMOKE=1`) e a cadeia é feita de
/// poses LOCAIS: só a raiz está no mundo, e cada filho nasce na ponta do pai. ⛔ Com absolutas ela
/// dobra-se sobre si mesma e todo o peso colapsa no primeiro osso — foi o que a 1.ª redacção destas
/// sondas mediu, e lia-se exactamente como um defeito do produto.
pub(crate) fn barra_da_cena() -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
    barra_da_cena_com(true)
}

/// ⭐⭐⭐ **A mesma barra presa pela PORTA DO PRODUTO** — o [`crate::skin_live::bind`], sem
/// parâmetro nenhum.
///
/// ⛔⛔ **Ela existe por uma MUTAÇÃO SOBREVIVENTE (2026-09-20):** o gate que afirma *«o produto
/// deixa os oito pontos»* pedia `barra_da_cena_com(false)` e ficava **VERDE** com o `bind` a
/// subdividir outra vez — *um gate que chama a porta interna afirma que a lei existe, nunca que o
/// gesto a usa*, e é a lei que esta casa já escreveu três vezes noutras famílias.
///
/// ⚠️ Toda metade de gate que fale do **PRODUTO** entra por aqui; a irmã parametrizada fica para
/// o contrafactual.
pub(crate) fn barra_da_cena_do_produto()
-> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
    monta_a_barra(None)
}

/// ⭐⭐ **A mesma barra com a SUBDIVISÃO DO BIND como parâmetro.**
///
/// `subdividir = false` é o mundo de **antes de 2026-09-19** e o de **depois de 2026-09-20**;
/// `true` é a lei que o dono mandou retirar do gesto e que fica como **contrafactual** (e como o
/// sujeito de toda forma GRAVADA entre os dois dias). ⚠️ Ele é a porta do CONTROLO — quem afirma
/// sobre o produto usa a [`barra_da_cena_do_produto`].
pub(crate) fn barra_da_cena_com(
    subdividir: bool,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
    monta_a_barra(Some(subdividir))
}

/// A barra e o esqueleto; `None` prende pela porta do produto, `Some(b)` pela parametrizada.
fn monta_a_barra(
    subdividir: Option<bool>,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
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
    match subdividir {
        None => crate::skin_live::bind(&mut sim, &scene, &map, &[id], None),
        Some(b) => crate::skin_live::bind_com(&mut sim, &scene, &map, &[id], None, b),
    };
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
pub(crate) fn raio_de_fabrica() -> f64 {
    40.0 / f64::from(PPM)
}

/// ⭐⭐ **O BRAÇO DA DOBRA FORTE** — a peça e os ossos do `PH2D_VEC_BONE_SMOKE=4`, nas proporções
/// DELA: comprimento `6×` a espessura, três ossos de `5/3` de espessura entre as duas tampas, criados
/// pela mesma porta ([`crate::bone::create`]), com o traço de `traco × espessura`.
///
/// ⛔ **A [`barra_da_cena_com`] NÃO contém o gancho da dobra** (medido, report do dono de
/// 2026-09-30): com ossos de `2,13` espessuras a dobra fica LISA em toda pose de `110°`–`140°` ×
/// `70°`–`100°` (nenhum nó do assado vira mais de `0,7°`), e com os da cena um nó do assado vira
/// `180°` no vinco. *Uma fixtura com outras proporções mede outra dobra.*
pub(crate) fn braco_da_dobra_forte(
    traco: f64,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
    let (l, t) = (6.0_f64, 1.0_f64);
    let mut peca = cook(
        ShapeKind::RoundRect,
        [-l / 2.0, -t / 2.0],
        [l / 2.0, t / 2.0],
        &[t / 2.0],
    );
    if traco > 0.0 {
        peca.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(120, 70, 25, 255),
            traco * t,
        ));
    }
    let x0 = -l / 2.0 + t / 2.0;
    let passo = (l - t) / 3.0;
    let juntas: Vec<[f64; 2]> = (0..=3_u32)
        .map(|k| [x0 + passo * f64::from(k), 0.0])
        .collect();
    peca_presa(peca, &juntas)
}

/// **Uma peça presa a uma CADEIA de ossos pela porta do produto** — `juntas` são as pontas da
/// cadeia no mundo (`n + 1` pontos para `n` ossos), cada osso criado por [`crate::bone::create`] como
/// o gesto do editor o cria, e o bind é o do produto. A forma carrega o `Sprite` pela razão do
/// [`barra_da_cena_com`].
pub(crate) fn peca_presa(
    peca: ph2d_vec_scene::VecPath,
    juntas: &[[f64; 2]],
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<Entity>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(peca);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let mut pai = None;
    let mut ids = Vec::new();
    for par in juntas.windows(2) {
        let e =
            Entity::from_bits(crate::bone::create(&mut sim, pai, par[0], par[1]).expect("osso"));
        pai = Some(e);
        ids.push(e);
    }
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    let alvo = forma(&map, id);
    sim.world_mut()
        .entity_mut(alvo)
        .insert(ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]));
    (sim, scene, map, id, ids)
}
