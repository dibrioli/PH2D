//! Os gates dos **pontos de passagem** — filho de `connector_live_tests` (teto de LOC).
//!
//! O escape manual so vale se ele MANDA. Um waypoint que o roteador ignorasse seria pior que nao
//! ter waypoint nenhum: o usuario finca o ponto, a linha nao muda, e ele conclui que o editor esta
//! quebrado.

use super::*;

/// **A ROTA PASSA PELO WAYPOINT** — e essa e a feature inteira numa asserção.
///
/// O escape manual so vale se ele MANDA: um ponto de passagem que o roteador ignorasse seria pior
/// que nao ter ponto nenhum. O gate exige que a polilinha cozida encoste no ponto fincado, e que a
/// rota deixe de ser a que ela era sem ele.
#[test]
fn the_cooked_route_actually_goes_through_the_waypoint() {
    let (mut sim, mut scene, map, conn, [a, b]) = scene_with_connector();
    let mut cache = SideCache::new();

    // Sem waypoint: A e B estao alinhados, entao a rota e uma reta em y = 0.5.
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let straight = poly(&scene, conn);
    assert!(
        straight.iter().all(|p| (p[1] - 0.5).abs() < 1e-6),
        "a rota sem waypoint deveria ser reta: {straight:?}"
    );

    // Finca um ponto BEM acima da linha.
    let w = [5.0, 6.0];
    let mut c = VecConnector::between(a, b);
    c.waypoints = vec![w];
    assert!(attach(&mut sim, &map, conn, &c));
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let bent = poly(&scene, conn);

    // A rota ENCOSTA no ponto fincado.
    let touches = bent.iter().any(|p| (p[0] - w[0]).hypot(p[1] - w[1]) < 1e-6);
    assert!(
        touches,
        "a rota nao passou pelo ponto de passagem {w:?} — o escape manual nao manda: {bent:?}"
    );
    // E ela subiu ate la (nao continuou reta, ignorando o ponto).
    assert!(
        bent.iter().any(|p| p[1] > 5.0),
        "a rota ignorou o waypoint e seguiu reta: {bent:?}"
    );
    // As duas pontas continuam nas formas — dobrar a rota nao pode soltar a linha.
    let (s, t) = ends(&scene, conn);
    assert!(on_border([0.0, 0.0], [2.0, 1.0], s), "soltou de A: {s:?}");
    assert!(on_border([8.0, 0.0], [10.0, 1.0], t), "soltou de B: {t:?}");
}

/// Dois waypoints sao percorridos NA ORDEM. Sem isso, a linha iria ao 2o, voltaria ao 1o e
/// seguiria — o mais visivel dos bugs, e o mais facil de nao testar.
#[test]
fn two_waypoints_are_visited_in_order() {
    let (mut sim, mut scene, map, conn, [a, b]) = scene_with_connector();
    let mut cache = SideCache::new();
    let (w1, w2) = ([3.0, 5.0], [7.0, -4.0]);
    let mut c = VecConnector::between(a, b);
    c.waypoints = vec![w1, w2];
    assert!(attach(&mut sim, &map, conn, &c));
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let pts = poly(&scene, conn);

    let idx = |w: [f64; 2]| {
        pts.iter()
            .position(|p| (p[0] - w[0]).hypot(p[1] - w[1]) < 1e-6)
    };
    let (i1, i2) = (idx(w1), idx(w2));
    assert!(
        i1.is_some() && i2.is_some(),
        "a rota pulou um dos pontos: {pts:?}"
    );
    assert!(
        i1 < i2,
        "a rota visitou os waypoints FORA DE ORDEM (vai e volta): {pts:?}"
    );
}
