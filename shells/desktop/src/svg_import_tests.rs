//! Gates do lado da SHELL: o desenho aterra onde o gesto pediu, cada forma leva o nome do ficheiro
//! e os `<g>` dele viram grupos da Hierarquia.

use super::{SvgImportResult, import_svg, is_svg_file};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld};
use ph2d_vec_scene::VecScene;
use std::path::PathBuf;

/// Escreve um `.svg` no directório temporário e devolve o caminho.
///
/// ⚠️ **Com nome derivado do teste**: dois testes a partilhar um ficheiro em `/tmp` correm em
/// paralelo no nextest e um apanha o do outro — é uma corrida que este repo já arquivou por engano
/// como flake de carga.
fn ficheiro(nome: &str, texto: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ph2d_gate_{nome}.svg"));
    std::fs::write(&p, texto).expect("escrever a fixtura");
    p
}

fn cena() -> (SimWorld, VecScene, crate::vec_entities::VecEntityMap) {
    (
        SimWorld::default(),
        VecScene::new(),
        crate::vec_entities::VecEntityMap::new(),
    )
}

fn nomes(sim: &SimWorld) -> Vec<String> {
    let mut v: Vec<String> = sim
        .world()
        .iter_entities()
        .filter_map(|e| e.get::<Name>().map(|n| n.0.clone()))
        .collect();
    v.sort();
    v
}

/// ⭐ **A extensão é reclamada por ESTE importador, e não pela grelha de imagens.**
///
/// ⚠️ Mutação que tem de sangrar: pôr `svg` no `SUPPORTED_IMAGE_EXTENSIONS`. O ficheiro entraria
/// como SPRITE — pixels em vez de curvas — e o artista receberia o contrário do que pediu.
#[test]
fn an_svg_is_a_drawing_and_never_an_image() {
    assert!(is_svg_file(std::path::Path::new("a/b/logo.SVG")));
    assert!(is_svg_file(std::path::Path::new("logo.svgz")));
    assert!(!is_svg_file(std::path::Path::new("logo.png")));
    assert!(
        !ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.contains(&"svg"),
        "um .svg que entre pela porta das imagens vira uma sprite de pixels"
    );
    let filtros = crate::import_router::dialog_filters();
    assert!(
        filtros
            .iter()
            .any(|(_, exts)| exts.contains(&"svg") && exts.len() == 2),
        "o dialogo tem de OFERECER a linha do SVG: {filtros:?}"
    );
    assert!(
        filtros[0].1.contains(&"svg"),
        "e o «All supported» tem de o conter — foi por faltar nessa linha que o .gif ficou \
         invisivel durante meses"
    );
}

/// ⭐⭐⭐ **O DESENHO ATERRA ONDE O GESTO PEDIU** — e não na origem do mundo.
#[test]
fn the_drawing_lands_where_the_gesture_asked() {
    let (mut sim, mut scene, mut map) = cena();
    let p = ficheiro(
        "aterra",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <rect id="q" x="0" y="0" width="100" height="100" fill="#123456"/></svg>"##,
    );
    let SvgImportResult::Ok { shapes, size, .. } =
        import_svg(&mut sim, &mut scene, &mut map, &p, [7.0, -3.0], 50.0)
    else {
        panic!("a fixtura tem de entrar");
    };
    assert_eq!(shapes, 1);
    assert!(
        (size[0] - 2.0).abs() < 1e-6,
        "100 px a 50 px/m = 2: {size:?}"
    );
    let xs: Vec<f64> = scene.paths()[0].verts.iter().map(|v| v.anchor[0]).collect();
    let cx = (xs.iter().copied().fold(f64::INFINITY, f64::min)
        + xs.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        * 0.5;
    // ⚠️ A geometria pode ter sido assente (o `settle_origins` move o pivô para a entidade), então
    // o que se mede é o CENTRO no mundo: geometria + pose.
    let pose = f64::from(
        sim.world()
            .get::<ph2d_ecs::Transform>(Entity::from_bits(
                *map.values().next().expect("uma entidade"),
            ))
            .expect("tem Transform")
            .translation
            .x,
    );
    assert!(
        (cx + pose - 7.0).abs() < 1e-5,
        "o desenho tem de ficar centrado no ponto do gesto: {} contra 7.0",
        cx + pose
    );
}

/// ⭐ **Cada forma leva o `id` do ficheiro**, e um `<g id>` vira um grupo com os filhos dentro.
///
/// ⚠️ Mutação que tem de sangrar: deixar o nome de fábrica (`Path {id}`). A Hierarquia de um
/// logótipo importado passaria a ser uma lista de números, e o artista perderia a única pista que
/// o autor do ficheiro lhe deixou.
#[test]
fn each_shape_carries_the_files_own_name_and_a_group_becomes_a_group() {
    let (mut sim, mut scene, mut map) = cena();
    let p = ficheiro(
        "nomes",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <g id="cabeca">
          <rect id="olho" x="0" y="0" width="10" height="10" fill="#111"/>
          <rect id="boca" x="20" y="20" width="10" height="10" fill="#222"/>
        </g></svg>"##,
    );
    let SvgImportResult::Ok { bits, .. } =
        import_svg(&mut sim, &mut scene, &mut map, &p, [0.0, 0.0], 100.0)
    else {
        panic!("a fixtura tem de entrar");
    };
    let ns = nomes(&sim);
    for esperado in ["olho", "boca", "cabeca"] {
        assert!(ns.iter().any(|n| n == esperado), "falta {esperado}: {ns:?}");
    }
    // O objecto de topo é o grupo, e as duas formas são filhas dele.
    let grupo = Entity::from_bits(bits);
    let filhos = sim
        .world()
        .iter_entities()
        .filter(|e| e.get::<ChildOf>().is_some_and(|c| c.0 == grupo))
        .count();
    assert_eq!(filhos, 2, "as duas formas tem de estar DENTRO do grupo");
}

/// ⭐⭐ **UM ficheiro é UM objecto** — sem isto um logótipo de 40 formas aterra como 40 raízes e
/// não há gesto que o mova inteiro.
#[test]
fn a_file_with_many_loose_shapes_still_lands_as_one_object() {
    let (mut sim, mut scene, mut map) = cena();
    let p = ficheiro(
        "umobjecto",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <rect x="0" y="0" width="10" height="10" fill="#111"/>
        <rect x="20" y="20" width="10" height="10" fill="#222"/>
        <rect x="40" y="40" width="10" height="10" fill="#333"/></svg>"##,
    );
    let SvgImportResult::Ok { bits, shapes, .. } =
        import_svg(&mut sim, &mut scene, &mut map, &p, [0.0, 0.0], 100.0)
    else {
        panic!("a fixtura tem de entrar");
    };
    assert_eq!(shapes, 3);
    let topo = Entity::from_bits(bits);
    assert!(
        sim.world().get::<ChildOf>(topo).is_none(),
        "o objecto devolvido e' a RAIZ do desenho"
    );
    let raizes = sim
        .world()
        .iter_entities()
        .filter(|e| e.get::<Name>().is_some() && e.get::<ChildOf>().is_none())
        .count();
    assert_eq!(raizes, 1, "uma raiz so' — o desenho inteiro");
}

/// ⛔ **Um `.svg` sem forma desenhável é RECUSADO em voz alta**, e a recusa diz o que o ficheiro
/// tinha. Um documento vazio leria como *"importou e o desenho estava em branco"*.
#[test]
fn a_file_with_nothing_drawable_is_refused_and_says_what_it_had() {
    let (mut sim, mut scene, mut map) = cena();
    let p = ficheiro(
        "sotexto",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <text x="10" y="20" font-size="10">so texto</text></svg>"##,
    );
    let SvgImportResult::Err { error, .. } =
        import_svg(&mut sim, &mut scene, &mut map, &p, [0.0, 0.0], 100.0)
    else {
        panic!("um ficheiro so' com texto nao tem forma nenhuma");
    };
    assert!(
        error.contains("<text>"),
        "a recusa tem de NOMEAR o que la' estava: {error}"
    );
}

/// ⭐⭐⭐ **O GRUPO NASCE EM CIMA DO DESENHO, e não na origem do mundo.**
///
/// ⚠️ É a ORDEM entre duas portas que já existiam: o `settle_origins` só toca em formas **sem pai**
/// e na identidade, então agrupar antes dele punha um `ChildOf` em cada uma e elas ficavam **para
/// sempre** com o pivô na origem — e o grupo, cuja pose é a média das poses dos membros, nascia lá
/// também, com o gizmo longe do desenho.
///
/// ⛔ **Nenhum outro gate desta wave o vê**: o `the_drawing_lands_where_the_gesture_asked` soma
/// geometria + pose, e essa soma está certa nas DUAS ordens. O que muda é **de quem** é a pose.
///
/// Mutação que tem de sangrar: tirar a chamada ao `settle_origins` do `import_svg`. É o mesmo
/// defeito que o report do Enio de 30/08 curou para o verbo *Group*, por outra porta.
#[test]
fn the_group_is_born_on_top_of_the_drawing_not_at_the_world_origin() {
    let (mut sim, mut scene, mut map) = cena();
    let p = ficheiro(
        "posedogrupo",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <rect x="0" y="0" width="20" height="20" fill="#111"/>
        <rect x="60" y="60" width="20" height="20" fill="#222"/></svg>"##,
    );
    let SvgImportResult::Ok { bits, .. } =
        import_svg(&mut sim, &mut scene, &mut map, &p, [12.0, -8.0], 10.0)
    else {
        panic!("a fixtura tem de entrar");
    };
    let pose = sim
        .world()
        .get::<ph2d_ecs::Transform>(Entity::from_bits(bits))
        .expect("o grupo tem Transform")
        .translation;
    let dist = ((pose.x - 12.0).powi(2) + (pose.y + 8.0).powi(2)).sqrt();
    assert!(
        dist < 6.0,
        "o grupo tem de nascer junto do desenho (que aterrou em (12, -8)), e nasceu em \
         ({}, {}) — distancia {dist}",
        pose.x,
        pose.y
    );
}
