//! **O lápis desenha onde a mão está** — o gesto contra a ordem REAL do frame.
//!
//! Estes gates nasceram VERMELHOS num defeito reportado pelo Enio (*"pencil está com offset do
//! mouse"*). A causa não estava no lápis: estava na lista de gestos que o `settle_origins` pula.
//! Ela enumerava *"a caneta e a ferramenta de forma"* — as duas que existiam quando foi escrita —
//! e o lápis chegou como o TERCEIRO. Assentar um path no meio de um gesto que reescreve a
//! geometria em MUNDO a cada frame soma geometria + `Transform`, e a tinta sai deslocada do cursor
//! **pelo ponto onde o arrasto começou**.
//!
//! # Por que os gates da wave não o apanharam
//!
//! Os 22 gates de motor provam o AJUSTE (a curva passa pelas amostras) e os 5 arch-gates provam a
//! ORDEM do dispatch. Nenhum dos dois vê a POSE: a geometria estava certa e o afim da entidade
//! desmentia-a um passe depois. O que faltava era um gate que corresse os passes do frame **na
//! ordem do produto** e comparasse o pixel com o dedo — que é o que este arquivo é.
//!
//! ⚠️ **O 1º frame do gesto sai CORRECTO**, e é isso que torna o defeito difícil de ver num teste
//! de unidade: o `settle` corre ANTES do render, então no frame em que ele assenta o path a
//! geometria já é local e o afim já a leva de volta ao lugar. A divergência começa no frame
//! SEGUINTE, quando o lápis reescreve mundo por cima do local. Um gate que medisse um frame só
//! ficaria verde sobre o produto vermelho — daí a asserção correr **sobre o gesto inteiro**.

/// A dinâmica das fixtures: **um rato** (pressão cheia) e um relógio que anda um passo por
/// amostra. Premissa, não asserção — estes gates testam a GEOMETRIA e o enquadramento do lápis;
/// a largura variável tem gates próprios (`ph2d_vec_edit::pencil_width`).
fn tick() -> ph2d_vec_edit::pencil_width::PenDynamics {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    ph2d_vec_edit::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: u128::from(N.fetch_add(1, Ordering::Relaxed)) * 4_000_000,
    }
}

use ph2d_ecs::SimWorld;
use ph2d_vec_scene::VecScene;

use crate::vec_entities::VecEntityMap;

/// Unidades de mundo por pixel de tela — a régua da câmera default (4 unidades em ~900 px).
const PX_TO_WORLD: f64 = 0.0045;

/// A mão: um arrasto curto **longe da origem do mundo**, que é onde o artista de facto desenha.
///
/// ⚠️ **A fixture TEM de estar longe do centro.** O deslocamento É o ponto de partida do arrasto,
/// então um traço desenhado na origem mede erro zero **com o defeito instalado** — a fixture
/// óbvia é exactamente a que não contém o fenómeno.
fn hand() -> Vec<[f64; 2]> {
    (0..40)
        .map(|i| {
            let t = f64::from(i) / 39.0;
            [1.5 + 0.5 * t, 0.5 + 0.2 * (t * 3.0).sin()]
        })
        .collect()
}

/// Um frame do produto, na ordem do `render_loop`: `sync` → `settle_origins` → `build`.
///
/// A lista de gestos vem da **porta** (`gesture_paths`), não de um literal escrito aqui: um gate
/// que montasse a sua própria lista provaria a intenção do gate e não o que o produto passa.
fn frame(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    pencil: &ph2d_vec_edit::Pencil,
) -> ph2d_vec_scene::VecXforms {
    let pen = ph2d_vec_edit::PenTool::default();
    let shape = ph2d_vec_edit::ShapeTool::default();
    let drawing = crate::vec_transform::gesture_paths(&pen, &shape, pencil);
    crate::vec_entities::sync(sim, scene, map);
    crate::vec_transform::settle_origins(sim, scene, map, &drawing);
    crate::vec_transform::build(sim, map)
}

/// Onde o último ponto do traço é DESENHADO: geometria local ∘ afim da entidade.
fn drawn_tip(
    scene: &VecScene,
    xf: &ph2d_vec_scene::VecXforms,
    id: ph2d_vec_scene::VecPathId,
) -> [f64; 2] {
    let local = scene
        .path(id)
        .and_then(|p| p.verts.last().map(|v| v.anchor))
        .expect("o traço vivo tem de estar na cena");
    ph2d_vec_scene::xform_of(xf, id).apply(local)
}

/// **A tinta fica sob o dedo, do 1º ao último move.**
///
/// Mutação que sangra: tirar o lápis da porta `gesture_paths` (voltar a enumerar só a caneta e a
/// forma) ⇒ **erro 1,5897 unidades de mundo ≈ 353 px** a partir do 2º move, crescendo com a
/// distância à origem.
#[test]
fn the_pencil_draws_where_the_hand_is_through_the_whole_gesture() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let path = hand();

    let id = pencil.on_press(&mut scene, path[0], PX_TO_WORLD, tick());
    let mut worst = 0.0_f64;
    for (n, p) in path[1..].iter().enumerate() {
        pencil.on_drag(&mut scene, *p, tick());
        let xf = frame(&mut sim, &mut scene, &mut map, &pencil);
        let tip = drawn_tip(&scene, &xf, id);
        let err = ((tip[0] - p[0]).powi(2) + (tip[1] - p[1]).powi(2)).sqrt();
        assert!(
            err < 1e-9,
            "move {n}: a mão está em {p:?} e a tinta saiu em {tip:?} (erro {err:.4} unidades de \
             mundo ≈ {:.0} px) — o gesto foi ASSENTADO no meio: geometria de mundo + `Transform`",
            err / PX_TO_WORLD
        );
        worst = worst.max(err);
    }
    assert!(worst < 1e-9, "pior erro do gesto: {worst}");
}

/// **O traço fica onde a mão o deixou depois de soltar** — e o assentamento então acontece,
/// porque a partir daí a geometria pára de ser reescrita.
///
/// As duas metades são um gate só de propósito: o valor de assentar é o pivô no centro da forma
/// (ADR-0112), e um "conserto" que simplesmente nunca assentasse o lápis passaria na primeira
/// metade e falharia aqui.
#[test]
fn the_committed_stroke_keeps_its_place_and_then_settles() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let path = hand();

    let id = pencil.on_press(&mut scene, path[0], PX_TO_WORLD, tick());
    for p in &path[1..] {
        pencil.on_drag(&mut scene, *p, tick());
        frame(&mut sim, &mut scene, &mut map, &pencil);
    }
    assert!(
        pencil.on_release(&mut scene),
        "o gesto foi um traço, não um clique"
    );

    // O frame seguinte: já não há gesto vivo, então o `settle` PODE assentar.
    let xf = frame(&mut sim, &mut scene, &mut map, &pencil);
    let tip = drawn_tip(&scene, &xf, id);
    let last = path[path.len() - 1];
    let err = ((tip[0] - last[0]).powi(2) + (tip[1] - last[1]).powi(2)).sqrt();
    assert!(
        err < 1e-9,
        "depois de soltar, a ponta do traço saiu em {tip:?} e a mão terminou em {last:?} \
         (erro {err:.4})"
    );

    // E o pivô foi para o centro da forma: a geometria deixou de ser mundo.
    let center_local = scene
        .path_bbox(id)
        .map(|(lo, hi)| [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
        .expect("o traço tem bbox");
    assert!(
        center_local[0].abs() < 1e-6 && center_local[1].abs() < 1e-6,
        "o traço commitado NÃO foi assentado (centro local {center_local:?}) — o pivô ficou no \
         centro do mundo, e o gizmo giraria a forma em torno de um ponto fora dela"
    );
}

/// **A porta anuncia o gesto do lápis** — a metade estrutural, sem relógio e sem cena.
///
/// Ela e o gate de comportamento não são redundantes: um passe novo que voltasse a deslocar o
/// traço por outro caminho passaria por aqui e cairia lá; e uma lista montada à mão num chamador
/// futuro passaria lá (a fixture usa a porta) e cairia aqui.
#[test]
fn the_gesture_door_announces_a_live_pencil_stroke() {
    let mut scene = VecScene::new();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let pen = ph2d_vec_edit::PenTool::default();
    let shape = ph2d_vec_edit::ShapeTool::default();

    assert!(
        crate::vec_transform::gesture_paths(&pen, &shape, &pencil).is_empty(),
        "sem gesto vivo a porta tem de estar vazia — anunciar um path ocioso proibiria o \
         assentamento dele para sempre"
    );
    let id = pencil.on_press(&mut scene, [1.5, 0.5], PX_TO_WORLD, tick());
    assert_eq!(
        crate::vec_transform::gesture_paths(&pen, &shape, &pencil),
        vec![id],
        "o traço VIVO do lápis não é anunciado — o `settle_origins` não o enxerga e assenta-o no \
         meio do gesto"
    );
    pencil.cancel(&mut scene);
    assert!(
        crate::vec_transform::gesture_paths(&pen, &shape, &pencil).is_empty(),
        "o gesto morreu e a porta continua a anunciá-lo"
    );
}

/// **O gesto ARMA o perfil de largura enquanto o traço está aberto** (W1d) — é isto que faz o
/// artista ver a espessura ao desenhar, e não descobri-la no release.
///
/// A cadeia é a do produto: a mão dita as amostras → o lápis deriva as paradas → o frame as
/// pendura na forma viva. Um elo faltando deixa o componente ausente, e nenhum gate de unidade
/// do `pencil_width` notaria (eles chamam `width_stops` à mão).
#[test]
fn a_speed_varying_gesture_arms_a_live_width_profile() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let path = hand();
    // Metade lenta, metade rápida: as POSIÇÕES são as mesmas do gate acima, e só o relógio muda.
    let mut t_ns = 0u128;
    let mut dyn_at = |i: usize| {
        t_ns += if i < path.len() / 2 {
            8_000_000
        } else {
            1_000_000
        };
        ph2d_vec_edit::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns,
        }
    };

    let id = pencil.on_press(&mut scene, path[0], PX_TO_WORLD, dyn_at(0));
    for (i, p) in path[1..].iter().enumerate() {
        pencil.on_drag(&mut scene, *p, dyn_at(i + 1));
    }
    frame(&mut sim, &mut scene, &mut map, &pencil);
    // O passo que o `render_loop` dá entre o `sync` e o cozimento.
    let stops = pencil.width_stops(ph2d_vec_edit::pencil_width::WidthSource::Speed);
    crate::profile_live::arm(&mut sim, &map, &[id], &stops);

    let armed = crate::profile_live::spec_of(&sim, &map, id).expect("o gesto não armou perfil");
    assert!(
        armed.at(0.85) < armed.at(0.15),
        "o trecho rápido não saiu mais fino: {:.3} contra {:.3}",
        armed.at(0.85),
        armed.at(0.15)
    );
}

/// **A fonte `Uniform` não pendura nada.** É o produto de antes do W1d, e o neutro é a AUSÊNCIA —
/// um componente com oito multiplicadores `1.0` em toda forma desenhada seria o documento a
/// acumular relações invisíveis.
#[test]
fn the_uniform_source_arms_no_profile() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let path = hand();
    let id = pencil.on_press(&mut scene, path[0], PX_TO_WORLD, tick());
    for p in &path[1..] {
        pencil.on_drag(&mut scene, *p, tick());
    }
    frame(&mut sim, &mut scene, &mut map, &pencil);
    let stops = pencil.width_stops(ph2d_vec_edit::pencil_width::WidthSource::Uniform);
    crate::profile_live::arm(&mut sim, &map, &[id], &stops);
    assert!(
        crate::profile_live::spec_of(&sim, &map, id).is_none(),
        "a fonte Uniform pendurou um perfil"
    );
}
