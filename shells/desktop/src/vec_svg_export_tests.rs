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

/// ⭐⭐⭐ **O QUE ESTÁ EM CIMA NO MUNDO FICA EM CIMA NO FICHEIRO.**
///
/// ⛔ Este gate não existia, e é por isso que **todo SVG exportado saía espelhado** desde 02/09: o
/// exportador escrevia coordenadas de mundo cruas (`Y` a subir) dentro de um `<svg>` (`Y` a
/// descer), e os seis gates deste ficheiro mediam tinta, pose, marca e cabeçalho — nenhum media
/// **para que lado**.
///
/// ⚠️ Mutação que tem de sangrar: apagar o `bake_xform(..., world_to_svg(..))`. Todos os outros
/// gates deste ficheiro continuam verdes.
#[test]
fn what_is_up_in_the_world_is_up_in_the_file() {
    let mut scene = VecScene::new();
    // Um triângulo com a ponta no ALTO do mundo (y = +10) e a base em baixo (y = -10).
    scene.push_path(VecPath {
        verts: vec![v(0.0, 10.0), v(10.0, -10.0), v(-10.0, -10.0)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(0, 0, 0, 255))),
        ..VecPath::default()
    });
    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);
    let ys = ys_do_d(&out.texto);
    // ⚠️ A comparação é contra os EXTREMOS, e não contra o vizinho: num vértice de quina os dois
    // handles coincidem com a âncora, então o `d` traz o mesmo `y` três vezes seguidas — a 1.ª
    // redacção deste gate pedia `ys[0] < ys[1]` e reprovava sobre produto CERTO.
    let (lo, hi) = (
        ys.iter().copied().fold(f64::INFINITY, f64::min),
        ys.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    assert!(
        (ys[0] - lo).abs() < 1e-9 && lo < hi,
        "o `M` do caminho e' a PONTA, e no SVG (Y a descer) ela tem de ser o menor y: \
         primeiro={} lo={lo} hi={hi}\n{}",
        ys[0],
        out.texto
    );
}

/// Os `y` do primeiro `d="…"` do ficheiro, na ordem em que aparecem.
///
/// ⚠️ O separador leva um ESPAÇO à frente de propósito: `data-ph2d-id="` acaba em `d="`, e sem ele
/// este ajudante lia o número do id e devolvia lista vazia — a 1.ª redacção do gate reprovou
/// exactamente assim, sobre produto certo.
fn ys_do_d(texto: &str) -> Vec<f64> {
    let d = texto
        .split(" d=\"")
        .nth(1)
        .expect("ha' um caminho")
        .split('"')
        .next()
        .expect("o d fecha");
    let nums: Vec<f64> = d
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    nums.iter().skip(1).step_by(2).copied().collect()
}

/// ⭐⭐⭐ **EXPORTAR E VOLTAR A IMPORTAR DEVOLVE A MESMA GEOMETRIA.**
///
/// É o oráculo que a lei dos eixos merece: as duas direcções são escritas uma vez cada
/// ([`ph2d_vec_svg::world_to_svg`] / [`ph2d_vec_svg::svg_to_world`]) e este gate atravessa o
/// produto inteiro — o `build_contours` do renderer, o texto do ficheiro, o parser do usvg e a
/// travessia do importador.
///
/// ⚠️ **A centragem do importador é parte da ida-e-volta**: ele devolve o desenho centrado na
/// origem (é o que a shell precisa para o largar), então o que se compara é a FORMA — cada âncora
/// menos o centro da caixa.
#[test]
fn a_drawing_that_goes_out_as_svg_comes_back_the_same_shape() {
    let mut scene = VecScene::new();
    scene.push_path(VecPath {
        verts: vec![v(0.0, 12.0), v(9.0, -6.0), v(-9.0, -6.0)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))),
        ..VecPath::default()
    });
    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);
    let de_volta = ph2d_vec_svg::import(
        out.texto.as_bytes(),
        &ph2d_vec_svg::Options {
            pixels_per_meter: ph2d_vec_svg::EXPORT_PIXELS_PER_UNIT,
        },
    )
    .expect("o nosso proprio ficheiro tem de entrar");
    assert_eq!(de_volta.shapes.len(), 1);
    let v: Vec<[f64; 2]> = de_volta.shapes[0]
        .path
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();
    let centro = |eixo: usize| {
        let lo = v.iter().map(|p| p[eixo]).fold(f64::INFINITY, f64::min);
        let hi = v.iter().map(|p| p[eixo]).fold(f64::NEG_INFINITY, f64::max);
        (lo + hi) * 0.5
    };
    let (cx, cy) = (centro(0), centro(1));
    let mut vistos: Vec<[f64; 2]> = v.iter().map(|p| [p[0] - cx, p[1] - cy]).collect();
    // O centro da caixa do triangulo original: x = 0, y = (12 + -6)/2 = 3.
    let mut esperados = vec![[0.0, 9.0], [9.0, -9.0], [-9.0, -9.0]];
    let ordena = |v: &mut Vec<[f64; 2]>| {
        v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN na geometria"));
    };
    ordena(&mut vistos);
    ordena(&mut esperados);
    for (a, b) in vistos.iter().zip(&esperados) {
        assert!(
            (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3,
            "a forma tem de voltar igual: {vistos:?} contra {esperados:?}"
        );
    }
}

/// ⭐⭐⭐ **CADA CAMADA DA PILHA VIRA UM ELEMENTO** (v20), com a opacidade e a mistura dela — o SVG
/// exprime as duas, então a pilha atravessa o ficheiro sem perda.
///
/// ⚠️ **E cada gradiente leva um `id` PRÓPRIO.** Um `<linearGradient id="g7">` repetido faz o
/// segundo referenciar o primeiro **em silêncio**, e todas as camadas sairiam com a mesma rampa.
#[test]
fn every_layer_of_the_stack_becomes_its_own_element() {
    let mut scene = VecScene::new();
    let mut base = VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(1, 2, 3, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(4, 5, 6, 255), 1.0)),
        ..VecPath::default()
    };
    let mut extra =
        ph2d_vec_scene::PaintEntry::stroke(StrokeSpec::new(Rgba8::new(200, 100, 50, 255), 7.0));
    extra.opacity = ph2d_vec_scene::Opacity::new(0.5);
    extra.blend = ph2d_vec_scene::BlendMode::Multiply;
    base.paints = vec![extra];
    scene.push_path(base);

    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);
    let elementos = out.texto.matches("<path ").count();
    assert_eq!(
        elementos, 3,
        "chao (fill + stroke) + a camada:\n{}",
        out.texto
    );
    assert!(
        out.texto.contains("stroke-width=\"7\""),
        "a largura da CAMADA tem de sair, e nao a da base:\n{}",
        out.texto
    );
    assert!(
        out.texto.contains("opacity=\"0.5\"") && out.texto.contains("mix-blend-mode:multiply"),
        "a opacidade e a mistura da camada atravessam:\n{}",
        out.texto
    );
    assert!(
        out.aproximadas.is_empty(),
        "nada disto e' aproximacao: {:?}",
        out.aproximadas
    );
}

/// ⛔ **UM MODO QUE O CSS NÃO TEM SAI NOMEADO**, e não como `normal` calado — o ficheiro não pode
/// afirmar uma composição que o documento não faz.
#[test]
fn a_blend_mode_css_does_not_have_is_named_not_silently_normal() {
    let mut scene = VecScene::new();
    let mut base = VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(1, 2, 3, 255))),
        ..VecPath::default()
    };
    let mut extra = ph2d_vec_scene::PaintEntry::fill(Paint::Solid(Rgba8::new(9, 9, 9, 255)));
    extra.blend = ph2d_vec_scene::BlendMode::Add; // o Linear Dodge: sem nome em CSS
    base.paints = vec![extra];
    scene.push_path(base);

    let out = svg(&scene, &VecXforms::new(), &sempre_nao, &sempre_nao);
    assert!(
        !out.texto.contains("mix-blend-mode"),
        "sem nome em CSS, nao se escreve nenhum:\n{}",
        out.texto
    );
    assert!(
        out.aproximadas.iter().any(|(_, o)| o.contains("mistura")),
        "e a perda tem de aparecer no cabecalho: {:?}",
        out.aproximadas
    );
}
