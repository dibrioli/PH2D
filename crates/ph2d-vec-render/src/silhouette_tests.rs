//! Os gates da porta `sil` — **quem responde pela forma TRAÇADA**.
//!
//! Esta crate não conhece a booleana (o `Cargo.toml` explica o porquê), então ela não pode
//! *produzir* a união `preenchimento ∪ traço`; o que ela pode — e o que estes gates pinam — é
//! **consumi-la quando alguém a traz** e **continuar recusando quando ninguém traz**.

use super::*;
use ph2d_vec_scene::{Contour, FillRule, Paint, Rgba8, StrokeSpec, VecVertex};

/// `VecPathId` é um alias de `u64` — este helper existe só para o teste ler como o produto.
fn id_of(n: u64) -> VecPathId {
    n
}

/// Um quadrado fechado de lado `2·half`, centrado na origem.
fn square(id: u64, half: f64) -> VecPath {
    let verts = [[-half, -half], [half, -half], [half, half], [-half, half]]
        .into_iter()
        .map(VecVertex::corner)
        .collect::<Vec<_>>();
    VecPath {
        id,
        verts,
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 60, 220, 255))),
        ..VecPath::default()
    }
}

/// Uma cena com os caminhos JÁ com os ids que o teste escolheu — o `push_path` cunha o dele, então
/// o id é reposto depois (o teste pergunta por id, e um id cunhado tornaria a pergunta outra).
fn scene_of(paths: Vec<VecPath>) -> VecScene {
    let mut s = VecScene::default();
    for p in paths {
        let want = p.id;
        let got = s.push_path(p);
        if got != want {
            s.paths_mut().last_mut().expect("acabou de entrar").id = want;
        }
    }
    s
}

/// A resposta que a shell traria: o quadrado CRESCIDO pela meia-largura, sem traço e com
/// preenchimento — o contrato que a `sil` declara.
fn resolved(id: u64, half: f64) -> LiveGeometry {
    let mut m = LiveGeometry::new();
    m.insert(id_of(id), vec![square(id, half)]);
    m
}

/// O pior desvio dos extremos dos segmentos em relação à borda do quadrado de meia-largura `half`.
///
/// ⚠️ **Contar segmentos não serve de oráculo:** o `build_fill_bezpath` emite CÚBICAS (degeneradas
/// numa quina) e o achatamento as parte em `steps(hull)` pedaços — 4 por aresta com esta tolerância.
/// A contagem é detalhe do achatador; o que o campo consome é ONDE a fronteira está.
fn worst_off_square(segs: &[[f32; 4]], half: f64) -> f64 {
    segs.iter()
        .flat_map(|s| [(s[0], s[1]), (s[2], s[3])])
        .map(|(x, y)| (f64::from(x).abs().max(f64::from(y).abs()) - half).abs())
        .fold(0.0_f64, f64::max)
}

/// **O gate.** Uma forma com traço passa a ter silhueta EXATA quando alguém resolve a união — e o
/// que sai descreve a borda RESOLVIDA, não a curva autorada.
///
/// ⚠️ Mutação que sangra: apagar o braço `sil.get(&id)` de `silhouette_segments`. Aí a forma cai no
/// braço da cena, o `push_path` a recusa pelo traço, e o resultado volta a ser **0 segmentos** — o
/// caminho do raster, que é onde o pente do bevel nasce.
#[test]
fn a_stroked_shape_gets_an_exact_silhouette_once_someone_resolves_the_union() {
    let mut p = square(7, 1.0);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.2));
    let scene = scene_of(vec![p]);
    let xf = VecXforms::default();
    let live = LiveGeometry::new();

    let none = silhouette_segments(
        &scene,
        &xf,
        &live,
        &LiveGeometry::new(),
        id_of(7),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    assert!(
        none.is_empty(),
        "sem uniao resolvida uma forma tracada tem de cair no raster (veio {} segmentos)",
        none.len()
    );

    let segs = silhouette_segments(
        &scene,
        &xf,
        &live,
        &resolved(7, 1.1),
        id_of(7),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    assert!(
        !segs.is_empty(),
        "com a uniao resolvida a silhueta nao pode ser vazia"
    );
    // Todo extremo da silhueta senta na borda RESOLVIDA (1,1), nunca na autorada (1,0).
    let worst = worst_off_square(&segs, 1.1);
    assert!(
        worst < 1e-4,
        "a silhueta descreve a curva autorada, nao a resolvida (pior desvio {worst:.5})"
    );
}

/// **Um traço de largura ZERO não é um traço — e a forma tem silhueta exata sem ninguém resolver
/// nada.**
///
/// Report do Enio (2026-07-27, segunda rodada): *"para stroke maior que 0 funciona. Mas para
/// stroke = 0 linhas aparecem"*. O slider de Width chega a `0`, e `0` significa **sem traço** — o
/// `stroke_zero_tests` desta crate já o prova do lado do desenho. Mas o campo perguntava
/// `stroke.is_some()`, que continua **verdadeiro** com largura zero: a forma caía no raster e o
/// pente voltava, agora sem sequer haver tinta de contorno para justificar.
///
/// ⚠️ Mutação que sangra: `push_path` voltar a perguntar `path.stroke.is_some()`. Zero segmentos.
#[test]
fn a_zero_width_stroke_is_not_a_stroke_and_the_shape_keeps_its_exact_silhouette() {
    let mut p = square(21, 1.0);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.0));
    let scene = scene_of(vec![p]);
    let segs = silhouette_segments(
        &scene,
        &VecXforms::default(),
        &LiveGeometry::new(),
        &LiveGeometry::new(),
        id_of(21),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    assert!(
        !segs.is_empty(),
        "largura zero mandou a forma para o raster - o pente volta sem haver tinta de contorno"
    );
    assert!(
        worst_off_square(&segs, 1.0) < 1e-6,
        "a silhueta de largura zero tem de ser a do PREENCHIMENTO, exata"
    );
}

/// **A `sil` é consultada PRIMEIRO — na frente da geometria derivada.** Uma forma pode ter offset
/// vivo *e* traço; a união é feita sobre o que o `dispatch` desenha, então ela é a palavra final.
#[test]
fn the_resolved_union_wins_over_the_derived_geometry() {
    let mut p = square(9, 1.0);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.2));
    let scene = scene_of(vec![p]);
    let mut live = LiveGeometry::new();
    live.insert(id_of(9), vec![square(9, 3.0)]);
    let segs = silhouette_segments(
        &scene,
        &VecXforms::default(),
        &live,
        &resolved(9, 1.1),
        id_of(9),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    let worst = worst_off_square(&segs, 1.1);
    assert!(
        worst < 1e-4,
        "a derivada venceu a uniao resolvida (pior desvio {worst:.5})"
    );
}

/// **A forma SEM traço é byte-idêntica ao mundo pré-`sil`.** Um mapa vazio não muda uma vírgula do
/// caminho que já funcionava — é isto que torna a wave uma adição e não uma reescrita.
#[test]
fn a_plain_shape_is_untouched_by_the_new_door() {
    let scene = scene_of(vec![square(3, 1.0)]);
    let (xf, live, empty) = (
        VecXforms::default(),
        LiveGeometry::new(),
        LiveGeometry::new(),
    );
    let segs = silhouette_segments(
        &scene,
        &xf,
        &live,
        &empty,
        id_of(3),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    assert!(!segs.is_empty(), "um quadrado tem fronteira");
    let worst = worst_off_square(&segs, 1.0);
    assert!(worst < 1e-6, "a forma nua mudou de lugar ({worst:.6})");
}

/// **Uma peça resolvida que chegue com traço (ou sem preenchimento) é RECUSADA.**
///
/// O `push_path` já a recusaria em silêncio, e silêncio aqui seria caro: a forma voltaria ao raster
/// com todos os gates verdes. Este gate torna o contrato do `sil` — *regiões fechadas, sem traço* —
/// executável em vez de prosa, e é o par do `regions_of` da shell, que o normaliza.
#[test]
fn a_resolved_piece_that_still_carries_a_stroke_is_refused() {
    let mut p = square(5, 1.0);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.2));
    let scene = scene_of(vec![p.clone()]);
    let mut bad = LiveGeometry::new();
    bad.insert(id_of(5), vec![p]);
    let segs = silhouette_segments(
        &scene,
        &VecXforms::default(),
        &LiveGeometry::new(),
        &bad,
        id_of(5),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    assert!(
        segs.is_empty(),
        "uma regiao com traco nao e uma silhueta resolvida"
    );
}

/// Sanidade do fixture: um `Contour` extra é um subpath, e a silhueta o inclui — a união devolve
/// compostos quando a forma tem furo, e eles têm de virar segmentos como qualquer outro contorno.
#[test]
fn a_compound_region_contributes_every_contour() {
    let mut outer = square(11, 2.0);
    outer.subpaths.push(Contour {
        verts: [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
    });
    outer.fill_rule = FillRule::EvenOdd;
    let scene = scene_of(vec![square(11, 2.0)]);
    let mut sil = LiveGeometry::new();
    sil.insert(id_of(11), vec![outer]);
    let segs = silhouette_segments(
        &scene,
        &VecXforms::default(),
        &LiveGeometry::new(),
        &sil,
        id_of(11),
        Affine::IDENTITY,
        Affine::IDENTITY,
    );
    // Os DOIS contornos aparecem: há extremo na borda externa (2,0) e na interna (1,0).
    let on = |half: f64| {
        segs.iter()
            .flat_map(|s| [(s[0], s[1]), (s[2], s[3])])
            .any(|(x, y)| (f64::from(x).abs().max(f64::from(y).abs()) - half).abs() < 1e-6)
    };
    assert!(on(2.0) && on(1.0), "faltou um dos contornos do composto");
}
