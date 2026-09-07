//! ⭐⭐⭐ **O ISOLAMENTO do *Edit Prefab*** (Enio, 2026-09-07: *«o canvas deve ser borrado levemente
//! assim como todos os objetos nele, e o Prefab aparece no centro do canvas acima de tudo, livre do
//! blur»*).
//!
//! O desenho parte-se em **duas metades exactas**, e cada uma tem porta própria:
//! 1. [`super::dispatch`] — o MUNDO, sem a receita. É esta textura que o vidro jateado borra.
//! 2. [`super::dispatch_isolated`] — SÓ a receita, numa cena que o presente compõe **depois** do
//!    borrão. É isto que a deixa nítida acima de tudo.
//!
//! ⚠️ **A régua é a PARTIÇÃO** — as duas metades somadas desenham exactamente o que o desenho de
//! sempre desenha. Uma barra sobre cada metade sozinha deixa passar os dois defeitos que importam:
//! a forma que **nenhuma** das duas desenha (some do ecrã) e a que **as duas** desenham (a nítida
//! por cima do borrão de si própria, com um halo à volta).
//!
//! ⛔ **E sem receita aberta o desenho é BYTE-IDÊNTICO ao de sempre** — é a metade que impede esta
//! feature de mexer no caminho comum.

use ph2d_vec_scene::{
    Paint, Rgba8, StrokeSpec, VecPath, VecScene, VecVertex, VecViewState, VecXforms,
};
use ph2d_vector::{Affine, VectorScene};

/// Uma cena com dois quadrados, cada um com o seu id.
fn cena() -> VecScene {
    let mut s = VecScene::new();
    for _ in 0..2 {
        let id = s.push_path(quadrado());
        let _ = id;
    }
    s
}

fn quadrado() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0)),
        ..VecPath::default()
    }
}

/// Desenha o MUNDO com este estado de vista e devolve `(marcas, composições)`.
fn conta(scene: &VecScene, view: &VecViewState) -> (u32, u32) {
    conta_com(scene, view, false)
}

/// Desenha a RECEITA com este estado de vista.
fn conta_receita(scene: &VecScene, view: &VecViewState) -> (u32, u32) {
    conta_com(scene, view, true)
}

fn conta_com(scene: &VecScene, view: &VecViewState, receita: bool) -> (u32, u32) {
    let mut t = VectorScene::new();
    let porta = if receita {
        super::dispatch_isolated
    } else {
        super::dispatch
    };
    porta(
        scene,
        view,
        &VecXforms::default(),
        &super::LiveGeometry::default(),
        &super::FxImages::default(),
        &super::WidgetSkins::default(),
        &super::PatternTiles::default(),
        &super::BrushArts::default(),
        &super::DilatedPaints::default(),
        Affine::IDENTITY,
        &mut t,
    );
    let e = t.inner().encoding();
    (e.n_paths, e.n_clips)
}

/// ⛔⛔⛔ **SEM receita aberta, nada muda** — nem uma marca, nem uma composição, e a segunda porta
/// não desenha coisa nenhuma.
///
/// *Uma feature de vista que mexe no caminho comum é uma regressão com nome bonito.*
#[test]
fn without_an_open_prefab_the_drawing_is_untouched() {
    let s = cena();
    let vazio = VecViewState::default();
    let base = conta(&s, &vazio);
    assert!(base.0 > 0, "a fixtura nao desenhou nada — a regua e' vazia");
    assert_eq!(
        conta_receita(&s, &vazio),
        (0, 0),
        "a porta da receita desenhou com a lista VAZIA — o presente comporia uma cena a mais \
         por cima do vidro em todo quadro"
    );
}

/// ⭐⭐⭐ **AS DUAS METADES SÃO UMA PARTIÇÃO EXACTA** — nem forma perdida, nem forma desenhada duas
/// vezes.
///
/// ⚠️ **A soma é a régua, e as duas metades sozinhas não a substituem:** uma barra em cada uma
/// deixa passar a forma que ninguém desenha (some do ecrã) e a que ambas desenham (a nítida com um
/// halo do próprio borrão à volta — o artefacto que esta arquitectura existe para evitar).
///
/// **Mutação que deve sangrar:** o `dispatch` deixar de saltar a receita, ou o `dispatch_isolated`
/// deixar de filtrar por `is_isolated`.
#[test]
fn the_world_and_the_recipe_partition_the_drawing() {
    let s = cena();
    let ids: Vec<_> = s.paths().iter().map(|p| p.id).collect();
    let (base, _) = conta(&s, &VecViewState::default());
    let view = VecViewState {
        isolated: vec![ids[0]],
        ..VecViewState::default()
    };
    let (mundo, _) = conta(&s, &view);
    let (receita, _) = conta_receita(&s, &view);
    assert!(
        mundo > 0 && receita > 0,
        "uma das metades ficou vazia: mundo {mundo}, receita {receita}"
    );
    assert_eq!(
        mundo + receita,
        base,
        "as duas metades nao somam o desenho de sempre ({mundo} + {receita} contra {base}) — \
         ha' forma perdida ou desenhada duas vezes"
    );
}

/// ⭐⭐ **Com TUDO aberto, o mundo fica vazio e a receita desenha tudo** — o caso extremo da mesma
/// partição, e o que prova que a lei é *«tudo menos as levantadas»* e não uma lista escrita à mão.
#[test]
fn isolating_everything_empties_the_world_and_fills_the_recipe() {
    let s = cena();
    let ids: Vec<_> = s.paths().iter().map(|p| p.id).collect();
    let (base, _) = conta(&s, &VecViewState::default());
    let view = VecViewState {
        isolated: ids,
        ..VecViewState::default()
    };
    assert_eq!(
        conta(&s, &view).0,
        0,
        "o mundo ainda desenhou com tudo levantado — ele apareceria borrado POR BAIXO da receita"
    );
    assert_eq!(
        conta_receita(&s, &view).0,
        base,
        "a receita nao desenhou tudo o que foi levantado"
    );
}

/// ⚠️ **O olho da Hierarquia ganha à receita aberta.** Uma peça escondida continua escondida — as
/// duas portas leem o mesmo `is_hidden`, e a que se esquecesse dele traria de volta uma peça que o
/// artista apagou da vista.
#[test]
fn a_hidden_piece_stays_hidden_on_both_sides_of_the_glass() {
    let s = cena();
    let ids: Vec<_> = s.paths().iter().map(|p| p.id).collect();
    let view = VecViewState {
        isolated: vec![ids[0]],
        hidden: vec![ids[0], ids[1]],
        ..VecViewState::default()
    };
    assert_eq!(
        conta(&s, &view).0,
        0,
        "o mundo desenhou uma forma escondida"
    );
    assert_eq!(
        conta_receita(&s, &view).0,
        0,
        "a receita desenhou uma peca escondida"
    );
}
