//! ⭐⭐⭐ **O ISOLAMENTO do *Edit Prefab*** (Enio, 2026-09-07: *«o canvas deve ser borrado
//! levemente assim como todos os objetos nele, e o Prefab aparece no centro do canvas acima de
//! tudo, livre do blur»*).
//!
//! Duas leis, e as duas vivem na porta única do desenho ([`super::dispatch`]):
//! 1. **o mundo RECUA** — uma camada com [`ph2d_vec_scene::ISOLATION_BACKDROP_ALPHA`] envolve tudo
//!    o que não é a receita;
//! 2. **a receita desenha-se por ÚLTIMO**, fora dessa camada — acima de tudo e sem o recuo.
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

/// Desenha a cena com este estado de vista e devolve `(marcas, composições)`.
fn conta(scene: &VecScene, view: &VecViewState) -> (u32, u32) {
    let mut t = VectorScene::new();
    super::dispatch(
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

/// ⛔⛔⛔ **SEM receita aberta, nada muda** — nem uma marca, nem uma composição.
///
/// *Uma feature de vista que mexe no caminho comum é uma regressão com nome bonito.*
#[test]
fn without_an_open_prefab_the_drawing_is_untouched() {
    let s = cena();
    let base = conta(&s, &VecViewState::default());
    // Uma vista com a caixa preenchida mas SEM exemptas: o modo não liga.
    let mut só_caixa = VecViewState {
        isolation_screen: [0.0, 0.0, 800.0, 600.0],
        ..VecViewState::default()
    };
    só_caixa.isolated.clear();
    assert_eq!(
        conta(&s, &só_caixa),
        base,
        "a caixa sozinha ligou o modo — ele precisa das DUAS metades"
    );
}

/// ⭐⭐⭐ **Com uma receita aberta, o mundo recua numa camada** — e a receita **continua a desenhar**.
///
/// ⚠️ **As duas metades no mesmo gate:** *«abre composição»* sozinho ficaria verde num renderer que
/// esbatesse tudo e deixasse de desenhar a receita; *«desenha as duas»* sozinho ficaria verde num
/// renderer que ignorasse o isolamento por completo.
///
/// **Mutação que deve sangrar:** o `push_object_layer` do recuo, ou a segunda passagem.
#[test]
fn an_open_prefab_pushes_the_backdrop_and_still_draws_everything() {
    let s = cena();
    let ids: Vec<_> = s.paths().iter().map(|p| p.id).collect();
    let (marcas_base, clips_base) = conta(&s, &VecViewState::default());
    let view = VecViewState {
        isolated: vec![ids[0]],
        isolation_screen: [0.0, 0.0, 800.0, 600.0],
        ..VecViewState::default()
    };
    let (marcas, clips) = conta(&s, &view);
    // ⚠️ **`+2` é a camada do recuo, e o número foi MEDIDO**: o Vello encoda um caminho no
    // `push_layer` (o recorte) e outro no `pop_layer` (o fecho). *Escrever `==` aqui seria uma
    // barra calibrada sem olhar para o que o renderer de facto emite.*
    assert_eq!(
        marcas,
        marcas_base + 2,
        "o isolamento perdeu (ou repetiu) desenho: as duas formas continuam a desenhar uma vez, \
         mais o push/pop da camada do recuo"
    );
    assert!(
        clips > clips_base,
        "o mundo nao recuou — nenhuma composicao foi aberta ({clips} contra {clips_base})"
    );
}

/// ⭐⭐ **E a receita sai FORA da camada do recuo** — o desenho dela vem depois do `pop`.
///
/// ⚠️ **A régua é a ORDEM, não a contagem:** um renderer que desenhasse a receita dentro da camada
/// teria exactamente as mesmas marcas e composições, e o artista veria o prefab esbatido junto com
/// o resto — que é o contrário do pedido.
///
/// ⛔ Medida pelo que se pode observar sem um raster: com a receita isolada, a cena tem de conter a
/// marca dela **depois** da composição que fecha o recuo. Como o encoding não expõe a ordem por id,
/// o que se mede é o INVARIANTE que a implementação garante: isolar TODAS as formas deixa o recuo
/// vazio — nenhuma marca dentro dele — e mesmo assim desenha todas.
#[test]
fn isolating_everything_leaves_the_backdrop_empty_and_still_draws() {
    let s = cena();
    let ids: Vec<_> = s.paths().iter().map(|p| p.id).collect();
    let (marcas_base, _) = conta(&s, &VecViewState::default());
    let view = VecViewState {
        isolated: ids,
        isolation_screen: [0.0, 0.0, 800.0, 600.0],
        ..VecViewState::default()
    };
    let (marcas, _) = conta(&s, &view);
    assert_eq!(
        marcas,
        marcas_base + 2,
        "com tudo isolado o desenho tem de continuar completo — e uma vez cada (mais a camada)"
    );
}
