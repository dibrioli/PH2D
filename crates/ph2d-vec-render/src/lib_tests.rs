//! Testes do render da cena vetorial (`dispatch`, `path_screen_bounds`, o FX raster) —
//! arquivo irmao de `lib.rs` pelo teto de LOC (a suite cresceu com os gates do plano 24).

use super::*;

#[test]
fn empty_path_yields_empty_bezpath() {
    let p = VecPath::default();
    assert!(build_bezpath(&p).elements().is_empty());
}

#[test]
fn demo_scene_builds_nonempty_paths() {
    let scene = VecScene::demo();
    for path in scene.paths() {
        assert!(!build_bezpath(path).elements().is_empty());
    }
}

/// Uma cena de UM quadrado preenchido, e o id dele.
fn one_square() -> (VecScene, VecPathId) {
    use ph2d_vec_scene::{Paint, Rgba8, VecVertex};
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 30, 30, 255))),
        ..VecPath::default()
    });
    (scene, id)
}

/// **A geometria viva desenha NO LUGAR da fonte, não por cima dela.** Duas formas na tela
/// onde devia haver uma é o sintoma de um passe que só ACRESCENTA; e o z de um offset tem
/// de ser o z da forma que ele offseta.
#[test]
fn the_live_geometry_draws_instead_of_the_source() {
    let (scene, id) = one_square();
    let (view, xf) = (VecViewState::default(), VecXforms::default());
    let mut a = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &FxImages::new(),
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut a,
    );
    let plain = a.inner().encoding().n_paths;

    // A derivada: DOIS caminhos (um offset pode partir a forma em vários — o donut do
    // smoke devolve oito). Se o `dispatch` acrescentasse em vez de trocar, sairiam três.
    let mut live = LiveGeometry::new();
    let (extra, _) = one_square();
    live.insert(id, extra.paths().to_vec());
    live.insert(id, vec![scene.paths()[0].clone(), scene.paths()[0].clone()]);
    let mut b = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &live,
        &FxImages::new(),
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut b,
    );
    assert_eq!(
        b.inner().encoding().n_paths,
        plain * 2,
        "a derivada tem de SUBSTITUIR a fonte (2 itens = 2 desenhos, não 3)"
    );
}

/// **Uma entrada PRESENTE e VAZIA desenha NADA** — é a aniquilação (o offset comeu a
/// forma). Colapsá-la com "ausente" faria a forma reaparecer inteira no extremo do
/// slider, que é o oposto do que o artista acabou de pedir.
#[test]
fn an_empty_live_entry_draws_nothing() {
    let (scene, id) = one_square();
    let mut live = LiveGeometry::new();
    live.insert(id, Vec::new());
    let mut s = VectorScene::new();
    dispatch(
        &scene,
        &VecViewState::default(),
        &VecXforms::default(),
        &live,
        &FxImages::new(),
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut s,
    );
    assert_eq!(s.inner().encoding().n_paths, 0);
}

/// **Um filtro `Replace` (Blur) desenha a IMAGEM no lugar da forma** — a imagem TOMA o z, a
/// forma não é desenhada. A contagem de caminhos fica igual à nua (1 imagem em vez de 1 forma);
/// o oráculo pega os DOIS modos de falha em torno disso: `0` = a versão que desenhava NADA (o
/// bug que este gate achou), `plain+1` = ignorar `replaced` e desenhar forma E imagem.
#[test]
fn a_replace_filter_draws_the_image_in_place_of_the_shape() {
    let (scene, id) = one_square();
    let (view, xf) = (VecViewState::default(), VecXforms::default());
    let mut plain = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &FxImages::new(),
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut plain,
    );
    let plain_n = plain.inner().encoding().n_paths;
    assert!(plain_n > 0, "a forma nua desenha algo");

    let mut fx = FxImages::new();
    fx.insert(id, one_pixel_image());
    let mut s = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &fx,
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut s,
    );
    assert_eq!(
        s.inner().encoding().n_paths,
        plain_n,
        "Replace desenha a imagem NO LUGAR da forma (nem nada — o bug —, nem forma+imagem)"
    );
}

/// **Uma forma SEM filtro desenha-se a si mesma** — o controle do gate acima: a imagem só entra
/// quando há entrada no mapa, e uma forma alheia na cena não é tocada por um FX que não é dela.
#[test]
fn a_shape_without_a_filter_draws_itself() {
    let (scene, id) = one_square();
    let (view, xf) = (VecViewState::default(), VecXforms::default());
    let mut plain = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &FxImages::new(),
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut plain,
    );
    let plain_n = plain.inner().encoding().n_paths;

    // O FX é de OUTRO caminho (id + 1 não existe na cena) — a forma tem de continuar a desenhar-se.
    let mut fx = FxImages::new();
    fx.insert(id + 1, one_pixel_image());
    let mut s = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &fx,
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut s,
    );
    assert_eq!(
        s.inner().encoding().n_paths,
        plain_n,
        "sem filtro DELA, a forma desenha-se como sempre"
    );
}

/// **`path_screen_bounds` cobre a forma e escala com a câmera.** É o número que dimensiona o
/// scratch e posiciona a `FxImage`; um bbox errado põe o FX no lugar errado. O quadrado
/// `[0,2]²` sem traço dá `(0,0,2,2)`, e a 2× a câmera dá `(0,0,4,4)` — o dobro exato.
#[test]
fn path_screen_bounds_covers_the_shape_and_scales_with_the_camera() {
    let (scene, id) = one_square();
    let xf = VecXforms::default();
    let live = LiveGeometry::new();
    let (x0, y0, x1, y1) =
        path_screen_bounds(&scene, &xf, &live, id, Affine::IDENTITY).expect("bounds");
    assert!(
        x0 <= 0.01 && y0 <= 0.01 && x1 >= 1.99 && y1 >= 1.99,
        "o bbox tem de cobrir o quadrado [0,2]²; deu ({x0},{y0},{x1},{y1})"
    );
    let (sx0, sy0, sx1, sy1) =
        path_screen_bounds(&scene, &xf, &live, id, Affine::scale(2.0)).expect("bounds 2x");
    assert!(
        (sx1 - sx0 - 2.0 * (x1 - x0)).abs() < 0.01 && (sy1 - sy0 - 2.0 * (y1 - y0)).abs() < 0.01,
        "a 2x a câmera o bbox tem de dobrar; deu ({sx0},{sy0},{sx1},{sy1})"
    );
}

/// **O bbox de tela cobre a PONTA DO MITER, não só meia largura.**
///
/// Numa quina de ângulo interno `θ` o miter chega a `½w / sin(θ/2)` do vértice — numa ponta de
/// estrela (36°), **3,24 × ½w**. Inflar por meia largura recortava a ponta contra a borda do
/// scratch do FX raster, e o que se via era **a ponta CEIFADA** (reportado no smoke). O gate mede a
/// ponta ANALÍTICA e exige que ela caiba.
#[test]
fn the_screen_bounds_cover_a_miter_spike_not_just_half_the_width() {
    use ph2d_vec_scene::{Rgba8, StrokeSpec, VecVertex};
    // Uma cunha de 36° apontando para +Y, com a ponta na origem (o `StrokeSpec::new` já traz a
    // junta Miter, que é o default do produto).
    let half = (18.0f64).to_radians().tan();
    let width = 4.0;
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [-half * 10.0, -10.0], [half * 10.0, -10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), width)),
        ..VecPath::default()
    });
    let (_, _, _, y1) = path_screen_bounds(
        &scene,
        &VecXforms::default(),
        &LiveGeometry::new(),
        id,
        Affine::IDENTITY,
    )
    .expect("bounds");
    // A ponta do miter, analítica: `½w / sin(θ/2)` acima do vértice (que está em y = 0).
    let spike = 0.5 * width / (18.0f64).to_radians().sin();
    assert!(
        y1 >= spike - 0.01,
        "o bbox parou em {y1} e a ponta do miter chega a {spike} — a ponta seria CEIFADA contra a \
         borda do scratch"
    );
}

/// Uma `FxImage` mínima de 1×1 para os gates de dispatch — o conteúdo não importa, só a PRESENÇA.
fn one_pixel_image() -> FxImage {
    FxImage {
        image: ph2d_vector::StableImage::from_rgba(std::sync::Arc::new(vec![0u8; 4]), 1, 1)
            .expect("1x1 image"),
        rect: (0.0, 0.0, 1.0, 1.0),
    }
}

/// Spike de escala (ADR-0108 §5) — custo de re-encode NAIVE por frame (CPU,
/// sem dirty-tracking), a fração dominante do custo em escala (achado Rive).
/// `cargo test -p ph2d-vec-render --release -- --ignored --nocapture`
///
/// ⚠️ **Este número REGREDIU 1,7× sem ninguém ver**, porque ele vivia só neste `println!` de um
/// teste `#[ignore]`: o spike de 2026-07-05 registrou 10k formas em **0,77 ms** e a auditoria de
/// 2026-07-29 mediu **1,323**. A causa (uma construção de `BezPath` jogada fora por forma
/// preenchida + o cozido a correr duas vezes) está curada e **agora é gateada por CONTAGEM**, não
/// por relógio — ver `encode_cost_tests`. Medido depois da cura: **0,901 ms a 10k** (1,47×
/// recuperado) · 0,089 a 1k · 4,109 a 50k.
///
/// ⚠️ E o RESÍDUO fica nomeado em vez de arredondado: 0,901 contra os 0,77 do spike é **1,17×** que
/// esta wave não explica. Entre as duas medições nasceram o cozido (ADR-0121), a pilha de Live Path
/// Effects (ADR-0132) e o `LiveGeometry`/`FxImages` no `dispatch` — atribuir esse 1,17× exige a
/// própria medição, e afirmar uma causa sem ela seria o que a auditoria acabou de corrigir.
#[test]
#[ignore = "spike manual de medição; rode em --release --nocapture"]
fn encode_cost_by_n() {
    use std::time::Instant;
    let affine = Affine::IDENTITY;
    println!("\n=== re-encode NAIVE por frame (CPU, sem dirty-tracking) ===");
    for &n in &[1_000usize, 5_000, 10_000, 20_000, 50_000] {
        let scene = VecScene::demo_grid(n);
        let mut target = VectorScene::new();
        target.reset();
        let xf = VecXforms::new();
        dispatch(
            &scene,
            &VecViewState::default(),
            &xf,
            &LiveGeometry::new(),
            &FxImages::new(),
            &WidgetSkins::new(),
            affine,
            &mut target,
        ); // warm
        let iters = 30;
        let t = Instant::now();
        for _ in 0..iters {
            target.reset();
            dispatch(
                &scene,
                &VecViewState::default(),
                &xf,
                &LiveGeometry::new(),
                &FxImages::new(),
                &WidgetSkins::new(),
                affine,
                &mut target,
            );
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        println!(
            "N={:>6}  encode={:>7.3} ms/frame   (teto encode-bound: {:>6.0} fps)",
            n,
            ms,
            1000.0 / ms
        );
    }
}

/// **O `dispatch` HONRA a tinta dos tokens** (plano UI/UX §4/W4).
///
/// ⚠️ Este gate existe porque uma mutação real sobreviveu a todos os outros: os gates de unidade
/// do `painted()` e do resolvedor chamam a porta DIRETAMENTE, então trocar `path.painted(bound)`
/// de volta por `path` no laço de desenho deixava os sete verdes e **todo binding inerte na tela**.
/// A costura entre a porta e o desenho é um fato próprio, e precisa de um oráculo próprio.
///
/// O oráculo não é *"os encodes diferem"* (verdade em qualquer mudança), e sim a IDENTIDADE:
/// desenhar a forma **bindada** ao token tem de produzir exatamente o mesmo encode que desenhar
/// uma forma cujo LITERAL já é aquela cor.
#[test]
fn the_dispatch_draws_the_token_colour_not_the_literal() {
    use ph2d_vec_scene::{BoundPaint, Paint, Rgba8};

    let token = Rgba8::new(11, 222, 33, 255);
    let encode = |scene: &VecScene, view: &VecViewState| {
        let mut t = VectorScene::new();
        dispatch(
            scene,
            view,
            &VecXforms::default(),
            &LiveGeometry::new(),
            &FxImages::new(),
            &WidgetSkins::new(),
            Affine::IDENTITY,
            &mut t,
        );
        t.inner().encoding().draw_data.clone()
    };

    // (a) O literal, sem binding: a referência de que a mutação NÃO pode ser igual.
    let (scene, id) = one_square();
    let literal = encode(&scene, &VecViewState::default());

    // (b) A mesma forma, BINDADA ao token.
    let bound_view = VecViewState {
        bound: vec![BoundPaint {
            path: id,
            fill: Some(token),
            stroke: None,
            alpha: None,
        }],
        ..VecViewState::default()
    };
    let bound = encode(&scene, &bound_view);

    // (c) Uma forma cujo LITERAL já é a cor do token — o alvo.
    let (mut target_scene, target_id) = one_square();
    if let Some(p) = target_scene.path_mut(target_id) {
        p.fill = Some(Paint::Solid(token));
    }
    let target = encode(&target_scene, &VecViewState::default());

    assert_ne!(
        bound, literal,
        "o `dispatch` desenhou o LITERAL — o binding nao chegou ao desenho (o laco perdeu o \
         `painted()`), e os gates da porta continuam verdes"
    );
    assert_eq!(
        bound, target,
        "a forma bindada tem de encodar exatamente como a forma cujo literal ja' e' a cor do token"
    );
}

/// **A PELE de widget SUBSTITUI o desenho** (plano UI/UX W6.2) — no z da forma, como o FX raster.
///
/// ⚠️ Sem este gate, tirar a injeção do `dispatch` deixaria a `widget_live` a cozinhar um mapa
/// que ninguém desenha: os seis gates da ponte ficam VERDES (eles perguntam ao mapa) e o produto
/// mostra o retângulo cru. É o buraco que separa *"a pele foi pintada"* de *"a pele aparece"*.
#[test]
fn the_widget_skin_replaces_the_drawing() {
    let (scene, id) = one_square();
    let (view, xf) = (VecViewState::default(), VecXforms::default());

    let mut plain = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &FxImages::new(),
        &WidgetSkins::new(),
        Affine::IDENTITY,
        &mut plain,
    );
    let bare = plain.inner().encoding().n_paths;
    assert!(bare > 0, "a forma nua desenha");

    // Uma pele com TRÊS caminhos: se o dispatch acrescentasse em vez de trocar, sairiam 3 + o
    // desenho; se a ignorasse, sairia só o desenho.
    let mut skin = VectorScene::new();
    for _ in 0..3 {
        skin.fill_rect(
            ph2d_vector::Rect::new(0.0, 0.0, 4.0, 4.0),
            ph2d_vector::Color::from_rgba8(1, 2, 3, 255),
        );
    }
    let three = skin.inner().encoding().n_paths;
    let mut skins = WidgetSkins::new();
    skins.insert(id, skin);

    let mut dressed = VectorScene::new();
    dispatch(
        &scene,
        &view,
        &xf,
        &LiveGeometry::new(),
        &FxImages::new(),
        &skins,
        Affine::IDENTITY,
        &mut dressed,
    );
    assert_eq!(
        dressed.inner().encoding().n_paths,
        three,
        "a pele nao SUBSTITUIU o desenho (nem apareceu, nem tomou o lugar dele)"
    );
}
