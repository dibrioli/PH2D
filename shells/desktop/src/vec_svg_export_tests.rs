//! Gates da EXPORTAÇÃO SVG ([`super::svg`]).

use super::*;
use ph2d_vec_scene::{Contour, FillRule, Rgba8, StrokeSpec, VecVertex, VecXforms, VertexKind};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn quadrado(scene: &mut VecScene, r: f64) -> u64 {
    scene.push_path(VecPath {
        verts: vec![v(-r, -r), v(r, -r), v(r, r), v(-r, r)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 2.0)),
        ..VecPath::default()
    })
}

fn sempre_nao(_: u64) -> bool {
    false
}

/// ⭐⭐⭐ **O QUE SAI É A GEOMETRIA, E ELA ESTÁ NO SVG** — o gate de partida: sem `d` não há
/// exportação nenhuma, por mais bonito que o cabeçalho seja.
#[test]
fn the_export_carries_the_geometry_and_the_paint() {
    let mut scene = VecScene::new();
    quadrado(&mut scene, 10.0);
    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);

    assert_eq!(out.formas, 1);
    assert!(out.texto.contains("<svg "), "tem de ser um SVG");
    assert!(
        out.texto.contains("viewBox="),
        "sem viewBox nada abre no lugar certo"
    );
    assert!(
        out.texto.contains(" d=\"M"),
        "o caminho tem de ter geometria"
    );
    assert!(
        out.texto.contains("#ff0000"),
        "a cor de preenchimento viaja"
    );
    assert!(
        out.texto.contains("stroke-width=\"2\""),
        "a largura do traco viaja"
    );
    assert!(
        out.aproximadas.is_empty(),
        "uma cor chapada nao se aproxima"
    );
}

/// ⭐⭐⭐ **A POSE ESTÁ ASSADA NA GEOMETRIA.**
///
/// ⚠️ Sem isto o ficheiro abriria com todas as formas empilhadas na origem — e, pior para quem
/// analisa, as coordenadas não seriam as que o editor mede.
#[test]
fn the_objects_pose_is_baked_into_the_coordinates() {
    let mut scene = VecScene::new();
    let id = quadrado(&mut scene, 10.0);
    let mut xf = VecXforms::new();
    xf.insert(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 500.0, 0.0]));

    let out = svg(&scene, &xf, &sempre_nao, &sempre_nao);

    assert!(
        out.texto.contains("490") && out.texto.contains("510"),
        "a translacao de 500 tem de aparecer nas coordenadas"
    );
    assert!(
        !out.texto.contains("transform="),
        "a pose e' ASSADA, nao um atributo — um transform por elemento discordaria da regua"
    );
}

/// ⭐⭐ **DOIS elementos: o preenchimento só leva os contornos FECHADOS.**
///
/// ⚠️ É a lei do renderer (`build_fill_bezpath`). Um SVG com um elemento só fecharia implicitamente
/// cada contorno ABERTO e abriria regiões que o app não pinta — e uma rede soldada é feita
/// exactamente de contornos abertos.
#[test]
fn an_open_contour_is_stroked_but_never_filled() {
    let mut scene = VecScene::new();
    let mut p = VecPath {
        verts: vec![
            v(-10.0, -10.0),
            v(10.0, -10.0),
            v(10.0, 10.0),
            v(-10.0, 10.0),
        ],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 2.0)),
        fill_rule: FillRule::EvenOdd,
        ..VecPath::default()
    };
    p.subpaths.push(Contour {
        verts: vec![v(-30.0, 30.0), v(30.0, 30.0)],
        closed: false,
    });
    scene.push_path(p);

    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);

    let cheios: Vec<&str> = out
        .texto
        .lines()
        .filter(|l| l.contains("fill-rule="))
        .collect();
    assert_eq!(cheios.len(), 1, "um elemento de preenchimento");
    assert!(
        !cheios[0].contains("-30"),
        "o contorno ABERTO nao pode entrar no preenchimento: {}",
        cheios[0]
    );
    assert!(
        out.texto.contains("-30"),
        "e ainda assim ele tem de ser TRACADO"
    );
    assert!(
        out.texto.contains("evenodd"),
        "a regra de preenchimento viaja"
    );
}

/// ⭐⭐⭐ **UMA TINTA SEM EQUIVALENTE SAI APROXIMADA — E O FICHEIRO DIZ QUAL.**
///
/// ⚠️ *Um exportador que ignora em silêncio é pior do que um que recusa* (a lei do importador
/// `.ase`). E a cor de recurso é a **mesma** que o renderer usa quando o ladrilho não resolve, para
/// quem abre o ficheiro ver o que o app desenharia — nunca uma cor inventada aqui.
#[test]
fn a_paint_without_an_svg_equivalent_is_named_in_the_header() {
    let mut scene = VecScene::new();
    scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)],
        closed: true,
        fill: Some(Paint::MultiPoint {
            points: vec![ph2d_vec_scene::GradientPoint {
                pos: [0.0, 0.0],
                color: Rgba8::new(1, 2, 3, 255),
                jitter: 0.0,
                influence: 1.0,
            }],
        }),
        ..VecPath::default()
    });

    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);

    assert_eq!(out.aproximadas.len(), 1);
    assert!(
        out.texto.contains("APROXIMADO") && out.texto.contains("multi-ponto"),
        "o cabecalho tem de NOMEAR o que se perdeu"
    );
    assert!(
        out.texto.contains("#010203"),
        "e a cor e' a de recurso do proprio app"
    );
}

/// ⛔ **Uma forma ESCONDIDA não é exportada**, e uma ÁREA DE BALDE leva marca.
///
/// ⚠️ A marca é o que deixa quem lê o ficheiro separar a LINHA da TINTA sem adivinhar pela cor —
/// e é a mesma pergunta que o balde faz para decidir o que é parede.
#[test]
fn a_hidden_shape_is_left_out_and_a_bucket_fill_is_marked() {
    let mut scene = VecScene::new();
    let a = quadrado(&mut scene, 10.0);
    let b = quadrado(&mut scene, 20.0);
    let out = svg(&scene, &VecXforms::new(), &|id| id == a, &|id| id == b);

    assert_eq!(out.formas, 1, "a escondida fica de fora");
    assert!(
        out.texto.contains("data-ph2d-fill=\"1\""),
        "a area de balde tem de vir marcada"
    );
    assert!(
        out.texto.contains(&format!("data-ph2d-id=\"{b}\"")),
        "cada forma leva o id que o app lhe da'"
    );
}

/// ⚠️ **O gradiente vai como gradiente**, com as coordenadas na mesma pose da geometria.
#[test]
fn a_linear_gradient_travels_as_a_gradient() {
    let mut scene = VecScene::new();
    scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)],
        closed: true,
        fill: Some(Paint::Linear {
            stops: vec![
                ph2d_vec_scene::GradientStop {
                    offset: 0.0,
                    color: Rgba8::new(255, 0, 0, 255),
                },
                ph2d_vec_scene::GradientStop {
                    offset: 1.0,
                    color: Rgba8::new(0, 0, 255, 255),
                },
            ],
            start: [0.0, 0.0],
            end: [10.0, 10.0],
        }),
        ..VecPath::default()
    });

    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);

    assert!(
        out.texto.contains("<linearGradient"),
        "sem <linearGradient> nao e' um gradiente"
    );
    assert!(
        out.texto.contains("url(#g"),
        "o preenchimento tem de o referir"
    );
    assert!(
        out.texto.contains("stop-color=\"#0000ff\""),
        "as paradas viajam"
    );
    assert!(
        out.aproximadas.is_empty(),
        "um gradiente linear NAO e' aproximacao"
    );
}
