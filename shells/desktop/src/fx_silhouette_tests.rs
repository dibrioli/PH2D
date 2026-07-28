//! Gates da **silhueta resolvida** — arquivo irmão de `fx_silhouette.rs`.
//!
//! O oráculo é o mapa que o campo de distância vai consumir: quem entra nele, o que cada peça é, e
//! quantas vezes a booleana foi chamada. As mutações que estes gates existem para matar: não
//! resolver a forma traçada (a forma volta ao raster e o pente do bevel volta com ela), resolver
//! quem ninguém lê (custo em toda cena), entregar a peça crua (recusada em silêncio lá na frente) e
//! o memo re-cozer sem que nada tenha mudado (a união passaria a ser paga por FRAME).

use super::FxSilhouette;
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Name, SimWorld, Transform, VecPathRef};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{
    Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex, VecXforms,
};

/// Um quadrado de lado 2 com preenchimento; `stroke` decide se ele leva tinta de contorno.
fn scene(stroke: bool) -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(160, 40, 200, 255))),
        stroke: stroke.then(|| StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.2)),
        ..VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Shape"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id)
}

/// Arma um filtro qualquer — o gate não olha os `ops`, só a PRESENÇA (é ela que diz "alguém lê o
/// campo desta forma").
fn arm_filter(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId) {
    crate::fx_live::set_filter(
        sim,
        map,
        &[id],
        Some(ph2d_ecs::VecFilter {
            ops: vec![ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::BEVEL)],
        }),
    );
}

fn cook(stroke: bool, filter: bool) -> (FxSilhouette, VecPathId) {
    let (scene, mut sim, map, id) = scene(stroke);
    if filter {
        arm_filter(&mut sim, &map, id);
    }
    let mut fx = FxSilhouette::default();
    fx.recook(
        &scene,
        &sim,
        &map,
        &VecXforms::default(),
        &LiveGeometry::new(),
    );
    (fx, id)
}

/// **O gate.** Uma forma TRAÇADA com filtro entra no mapa, e cada peça sai normalizada ao que o
/// consumidor exige: **sem traço** e **com preenchimento**.
///
/// ⚠️ Mutação que sangra: tirar a normalização de `regions_of` (deixar `stroke`/`fill` como a
/// booleana os devolve). O mapa continua populado — e o `push_path` do `silhouette_segments` recusa
/// a peça **em silêncio**, então a forma volta ao raster com este arquivo inteiro verde. É por isso
/// que a asserção é sobre a FORMA de cada peça, não sobre o mapa não estar vazio.
#[test]
fn a_stroked_shape_with_a_filter_gets_regions_the_field_can_consume() {
    let (fx, id) = cook(true, true);
    let regions = fx.live().get(&id).expect("a forma tracada tem de entrar");
    assert!(!regions.is_empty(), "regioes vazias");
    for (i, r) in regions.iter().enumerate() {
        assert!(r.stroke.is_none(), "regiao {i} ainda carrega traco");
        assert!(r.fill.is_some(), "regiao {i} nao e uma regiao (sem fill)");
    }
}

/// **O fato que torna o cinto um cinto.** A booleana devolve HOJE regiões prontas — sem traço, com
/// preenchimento —, e é por isso que a mutação que remove a normalização de `regions_of` sobrevive.
///
/// Sem este gate a normalização seria uma defesa que ninguém sabe se ainda defende alguma coisa; com
/// ele, o dia em que a booleana passar a devolver a peça estilizada falha AQUI, com o nome certo, em
/// vez de a forma voltar ao raster em silêncio.
#[test]
fn the_boolean_already_hands_back_regions_and_this_normalisation_is_a_belt() {
    let (scene, ..) = scene(true);
    let raw = ph2d_vec_boolean::silhouette_paths(&scene.paths()[0]);
    assert!(!raw.is_empty(), "a uniao saiu vazia");
    for (i, r) in raw.iter().enumerate() {
        assert!(
            r.stroke.is_none() && r.fill.is_some(),
            "regiao crua {i} nao satisfaz o contrato do `sil` - a normalizacao deixou de ser cinto \
             e passou a ser conserto (e o gate que a prova esta em `fx_silhouette_tests`)"
        );
    }
}

/// **Sem filtro não se paga a união.** A silhueta serve ao campo de distância, e sem filtro não há
/// campo — uma cena cheia de formas traçadas sem FX não pode custar um sweep.
#[test]
fn a_stroked_shape_without_a_filter_is_not_resolved() {
    let (fx, id) = cook(true, false);
    assert!(fx.live().get(&id).is_none(), "resolveu quem ninguem le");
}

/// **Sem traço não há SEGUNDA resposta.** O `silhouette_segments` já responde exato pela fonte;
/// resolver aqui também seria duas portas para a mesma pergunta, e elas divergiriam no dia em que
/// a booleana mudasse de opinião sobre um caso degenerado.
#[test]
fn a_plain_shape_is_left_to_the_door_that_already_answers_it() {
    let (fx, id) = cook(false, true);
    assert!(fx.live().get(&id).is_none(), "resolveu forma sem traco");
}

/// **O memo torna a união um custo de EDIÇÃO, não de frame.** O re-cook dos FX roda a cada frame de
/// um arrasto de zoom; sem memo uma forma complexa pagaria a booleana por frame.
///
/// ⚠️ O oráculo é a IDENTIDADE do resultado, não um cronômetro: dois cozimentos seguidos sem tocar
/// em nada têm de devolver exatamente as mesmas regiões. Uma mutação que apague o `hit` do memo
/// ainda passaria aqui — e é por isso que existe o irmão abaixo, que conta CHAMADAS.
#[test]
fn cooking_twice_without_touching_anything_gives_the_same_regions() {
    let (scene, mut sim, map, id) = scene(true);
    arm_filter(&mut sim, &map, id);
    let mut fx = FxSilhouette::default();
    let xf = VecXforms::default();
    fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    let first = fx.live().get(&id).cloned().expect("cozeu");
    fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    let second = fx.live().get(&id).cloned().expect("cozeu");
    assert_eq!(
        first, second,
        "o segundo cozimento devolveu outra geometria"
    );
}

/// **O memo ACERTA quando nada muda e ERRA quando a geometria muda.** Conta as entradas no sweep da
/// booleana — o mesmo contador que o gate de FPS do Contour usa.
///
/// ⚠️ Mutação que sangra: `hit` cravado em `false`. O segundo cozimento volta a chamar a booleana, e
/// a contagem de cozimentos do frame parado deixa de ser constante — que é literalmente a diferença
/// entre pagar por edição e pagar por frame.
///
/// ⚠️ **O instrumento é o contador do próprio módulo, não o `__sweep_calls` da booleana** — este
/// conta entradas em `offset_path`, que a união não percorre, então a primeira versão deste gate
/// media zero contra zero e não podia falhar pelo motivo que alegava.
#[test]
fn a_still_frame_costs_nothing_and_a_moved_shape_costs_one_cook() {
    let (mut scene, mut sim, map, id) = scene(true);
    arm_filter(&mut sim, &map, id);
    let mut fx = FxSilhouette::default();
    let xf = VecXforms::default();

    fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    let after_first = fx.cooks();
    assert_eq!(after_first, 1, "o primeiro frame cozinha uma vez");
    for _ in 0..5 {
        fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    }
    assert_eq!(
        fx.cooks(),
        after_first,
        "um frame parado re-cozinhou a uniao"
    );

    // Mexe na geometria: o memo TEM de errar.
    scene.paths_mut()[0].verts[0].anchor[0] -= 0.5;
    fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    assert_eq!(
        fx.cooks(),
        after_first + 1,
        "o memo nao errou depois de a forma mudar - a silhueta descreve a forma de ontem"
    );
}

/// **A união é feita sobre o que o `dispatch` DESENHA.** Havendo geometria derivada (offset vivo,
/// pattern, contour), é ela que entra na booleana — senão o campo descreveria a fonte e o desenho
/// mostraria a derivada, que é a divergência muda que esta família de bugs produz.
#[test]
fn the_union_is_taken_over_the_derived_geometry_when_there_is_one() {
    let (scene, mut sim, map, id) = scene(true);
    arm_filter(&mut sim, &map, id);
    // Uma derivada MUITO maior que a fonte, ainda traçada.
    let mut derived = scene.paths()[0].clone();
    for v in &mut derived.verts {
        v.anchor[0] *= 4.0;
        v.anchor[1] *= 4.0;
    }
    let mut live = LiveGeometry::new();
    live.insert(id, vec![derived]);

    let mut fx = FxSilhouette::default();
    fx.recook(&scene, &sim, &map, &VecXforms::default(), &live);
    let reach = fx
        .live()
        .get(&id)
        .expect("cozeu")
        .iter()
        .flat_map(|r| {
            r.verts
                .iter()
                .chain(r.subpaths.iter().flat_map(|c| c.verts.iter()))
        })
        .map(|v| v.anchor[0].abs().max(v.anchor[1].abs()))
        .fold(0.0_f64, f64::max);
    assert!(
        reach > 3.0,
        "a uniao saiu da FONTE (alcance {reach:.3}), nao da derivada"
    );
}

/// **Perder o traço limpa o memo.** Uma forma que deixou de ser traçada não pode continuar com a
/// resposta velha pendurada — o campo descreveria uma borda de tinta que já não existe.
#[test]
fn dropping_the_stroke_drops_the_resolved_silhouette() {
    let (mut scene, mut sim, map, id) = scene(true);
    arm_filter(&mut sim, &map, id);
    let mut fx = FxSilhouette::default();
    let xf = VecXforms::default();
    fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    assert!(fx.live().contains_key(&id));

    scene.paths_mut()[0].stroke = None;
    fx.recook(&scene, &sim, &map, &xf, &LiveGeometry::new());
    assert!(
        !fx.live().contains_key(&id),
        "a silhueta do traco sobreviveu ao traco"
    );
}

/// **A COSTURA, ponta a ponta: a estrela traçada do report deixa de cair no raster.**
///
/// Os outros gates provam cada metade; este junta as duas na ordem em que o produto as usa —
/// `FxSilhouette::recook` → `silhouette_segments` — porque é exatamente aqui que um id desalinhado
/// ou um espaço trocado quebraria sem ninguém ver: o campo simplesmente voltaria ao raster, que é
/// *pior mas nunca trava*, e o pente do bevel voltaria com ele.
///
/// ⚠️ O oráculo é o CONTRASTE, não um número absoluto: a MESMA forma sem a silhueta resolvida tem de
/// dar **zero** segmentos (é o que acontecia antes desta wave) e com ela tem de dar segmentos que
/// descrevem a borda da TINTA — a meia-largura para fora da curva autorada.
#[test]
fn the_reported_stroked_star_reaches_the_exact_field_instead_of_the_raster() {
    use ph2d_vec_scene::{ShapeKind, cook as cook_shape};
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut star = cook_shape(ShapeKind::Star, [-2.0, -2.0], [2.0, 2.0], &[5.0, 0.45, 0.0]);
    star.fill = Some(Paint::solid(Rgba8::new(160, 40, 200, 255)));
    star.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.2));
    let id = scene.push_path(star);
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Star"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    arm_filter(&mut sim, &map, id);

    let (xf, empty) = (VecXforms::default(), LiveGeometry::new());
    let ident = ph2d_vector::Affine::IDENTITY;
    let before =
        ph2d_vec_render::silhouette_segments(&scene, &xf, &empty, &empty, id, ident, ident);
    assert!(
        before.is_empty(),
        "sem a silhueta resolvida a estrela tracada tinha de cair no raster (veio {})",
        before.len()
    );

    let mut fx = FxSilhouette::default();
    fx.recook(&scene, &sim, &map, &xf, &empty);
    let after =
        ph2d_vec_render::silhouette_segments(&scene, &xf, &empty, fx.live(), id, ident, ident);
    assert!(
        !after.is_empty(),
        "a costura nao fechou: a estrela tracada continua sem silhueta exata"
    );

    // A silhueta descreve a borda da TINTA: o alcance cresce, e cresce ao menos meia-largura.
    let src = scene.paths()[0]
        .verts
        .iter()
        .map(|v| v.anchor[0].hypot(v.anchor[1]))
        .fold(0.0_f64, f64::max);
    let out = after
        .iter()
        .flat_map(|s| [(s[0], s[1]), (s[2], s[3])])
        .map(|(x, y)| f64::from(x).hypot(f64::from(y)))
        .fold(0.0_f64, f64::max);
    assert!(
        out > src + 0.05,
        "a silhueta descreve a curva autorada (raio {src:.4}), nao a borda da tinta ({out:.4})"
    );
}
