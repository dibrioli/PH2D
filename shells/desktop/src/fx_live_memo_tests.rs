//! Gates da **CHAVE do memo** do FX raster — arquivo irmão de `fx_live_memo.rs`.
//!
//! O oráculo é a decisão que o `recook` toma na 1ª varredura, e ela é literalmente **um `==` entre
//! duas chaves**: igual ⇒ os pixels que estão na textura servem; diferente ⇒ re-coza. Por isso os
//! gates comparam CHAVES em vez de espiarem a GPU — a decisão inteira mora aqui, e mora aqui de
//! propósito (ela vivia dentro do `recook`, que precisa de dispositivo, então nenhum teste headless
//! a alcançava e o defeito abaixo atravessou a wave inteira com 18 gates verdes ao lado).
//!
//! O defeito medido (2026-07-29): **mudar a cor do preenchimento de uma forma filtrada não mudava a
//! tela.** A pilha resolvida era a mesma, a caixa era a mesma, então o memo acertava.
//!
//! ⚠️ **Um gate foi escrito e DESCARTADO em vez de shipado:** *"armar um Live Path Effect re-cozinha
//! a forma"*. Ele passa com a chave ANTIGA (a `(pilha, w, h)`), porque **todo efeito de caminho move
//! a caixa** — então ele não podia falhar pelo motivo que alegava, e a `effects` viaja para dentro da
//! chave pelo `VecPath` inteiro de qualquer modo (não há lista de campos a esquecer). Verde por
//! construção; a razão fica escrita aqui para ninguém o re-escrever achando que prova algo.

use super::job_for;
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{FxOp, Name, SimWorld, Transform, VecFilter, VecPathRef};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{
    Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex, VecXforms, Xform,
};
use ph2d_vector::Affine;

/// Um quadrado preenchido de lado 2, com uma pilha de FX armada. `Xform` identidade, câmera
/// identidade — o estado de qualquer forma recém-criada.
fn fixture() -> (VecScene, SimWorld, VecEntityMap, VecXforms, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(160, 40, 200, 255))),
        ..VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Shape"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    crate::fx_live::set_filter(
        &mut sim,
        &map,
        &[id],
        Some(VecFilter {
            ops: vec![FxOp::new(FxOp::BEVEL)],
        }),
    );
    (scene, sim, map, VecXforms::new(), id)
}

/// A chave que o `recook` computaria para esta forma neste frame.
fn key(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
    id: VecPathId,
) -> super::FxKey {
    let live = LiveGeometry::new();
    let sil = LiveGeometry::new();
    job_for(scene, sim, map, xforms, &live, &sil, Affine::IDENTITY, id)
        .expect("a forma tem filtro, caixa e pilha nao-vazia")
        .key
}

/// **CONTROLE: nada mudou ⇒ o memo ACERTA.**
///
/// Sem esta metade, a cura do defeito poderia ser *"a chave nunca casa"* — e os outros gates ficariam
/// todos verdes sobre um produto que re-cozinha a cena inteira em todo frame, que é exatamente o
/// custo que o memo existe para não pagar.
#[test]
fn a_frame_where_nothing_changed_hits_the_memo() {
    let (scene, sim, map, xforms, id) = fixture();
    let a = key(&scene, &sim, &map, &xforms, id);
    let b = key(&scene, &sim, &map, &xforms, id);
    assert!(
        a == b,
        "a chave mudou sem ninguem mexer em nada — o memo nunca acerta e toda forma filtrada \
         re-cozinha em todo frame"
    );
}

/// **A COR DO PREENCHIMENTO entra na chave** (o defeito reportado).
///
/// Mutação que tem de sangrar: a chave voltar a ser `(ops, w, h)` — era o código que shipava, e
/// mudar a cor do fill de uma forma filtrada não mudava um pixel na tela.
#[test]
fn changing_the_fill_colour_re_cooks_the_shape() {
    let (mut scene, sim, map, xforms, id) = fixture();
    let before = key(&scene, &sim, &map, &xforms, id);
    scene.path_mut(id).expect("a forma existe").fill =
        Some(Paint::solid(Rgba8::new(0, 200, 90, 255)));
    let after = key(&scene, &sim, &map, &xforms, id);
    assert!(
        before != after,
        "a cor do fill mudou e o memo ACERTOU — a textura continua a mostrar a cor antiga, sem erro \
         e sem warning (medido 2026-07-29)"
    );
}

/// **O TRAÇO entra na chave.** Irmão do de cima, e não é redundante: um traço novo muda a caixa
/// (`w`/`h` crescem pela largura), então a chave ANTIGA já o pegava — o que este gate pina é o caso
/// em que a largura NÃO muda: só a cor do traço.
#[test]
fn changing_the_stroke_colour_re_cooks_the_shape() {
    let (mut scene, sim, map, xforms, id) = fixture();
    scene.path_mut(id).expect("a forma existe").stroke =
        Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.2));
    let before = key(&scene, &sim, &map, &xforms, id);
    scene.path_mut(id).expect("a forma existe").stroke =
        Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.2));
    let after = key(&scene, &sim, &map, &xforms, id);
    assert!(
        before != after,
        "a cor do TRACO mudou (mesma largura, mesma caixa) e o memo ACERTOU"
    );
}

/// **Mover um vértice INTERIOR entra na chave.**
///
/// O caso que a caixa não vê: os extremos ficam onde estavam, então `w`/`h` não se movem — mas o
/// desenho dentro da célula é outro.
#[test]
fn moving_an_interior_vertex_re_cooks_the_shape() {
    let (mut scene, sim, map, xforms, id) = fixture();
    // Cinco vértices: o do meio da aresta de baixo é interior aos extremos em X **e** em Y.
    {
        let p = scene.path_mut(id).expect("a forma existe");
        p.verts = [
            [-1.0, -1.0],
            [0.0, -0.5],
            [1.0, -1.0],
            [1.0, 1.0],
            [-1.0, 1.0],
        ]
        .map(VecVertex::corner)
        .to_vec();
    }
    let before = key(&scene, &sim, &map, &xforms, id);
    let (w0, h0) = (before.w, before.h);
    scene.path_mut(id).expect("a forma existe").verts[1] = VecVertex::corner([0.0, 0.5]);
    let after = key(&scene, &sim, &map, &xforms, id);
    assert_eq!(
        (after.w, after.h),
        (w0, h0),
        "a fixture nao contem o fenomeno: a caixa MUDOU, entao a chave antiga tambem pegaria isto e \
         o gate nao prova nada"
    );
    assert!(
        before != after,
        "um vertice interior mudou de lugar (mesma caixa) e o memo ACERTOU"
    );
}

/// **TRANSLADAR a forma NÃO re-cozinha** — e esta é a propriedade que faz o memo valer a pena.
///
/// O conteúdo da célula é a forma desenhada em `-ex0,-ey0`: mover a forma (ou panhar a câmera) dá a
/// MESMA arte na MESMA posição dentro da célula, e o que muda é o `rect` onde a célula é desenhada
/// — recomputado todo frame, fora da chave.
///
/// ⚠️ Mutação que tem de sangrar: pôr o afim INTEIRO na chave (a cura ingênua). Ela faz **toda forma
/// filtrada re-cozinhar em todo frame de pan**, que é precisamente o gesto onde o memo se paga.
#[test]
fn translating_the_shape_does_not_re_cook_it() {
    let (scene, sim, map, mut xforms, id) = fixture();
    let before = key(&scene, &sim, &map, &xforms, id);
    xforms.insert(id, Xform([1.0, 0.0, 0.0, 1.0, 37.0, -11.0]));
    let after = key(&scene, &sim, &map, &xforms, id);
    assert!(
        before == after,
        "transladar a forma re-cozinhou o FX dela — panhar a cena passa a re-cozinhar toda forma \
         filtrada, em todo frame"
    );
}

/// **ESCALAR a forma re-cozinha.**
///
/// ⚠️ **Honestidade sobre o que este gate prova:** escalar também move `w`/`h`, então a chave ANTIGA
/// já o pegava e nenhuma mutação na parte linear o faz sangrar. Ele fica como **contraponto do de
/// cima** — para que *"a translação não conta"* nunca seja implementado como *"o afim não conta"* —
/// e não como prova de que `cam`/`screen` estão na chave (ver o doc do módulo: uma mudança linear que
/// muda os pixels muda a caixa também, então esses dois campos são cinto, não gate).
#[test]
fn scaling_the_shape_re_cooks_it() {
    let (scene, sim, map, mut xforms, id) = fixture();
    let before = key(&scene, &sim, &map, &xforms, id);
    xforms.insert(id, Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
    let after = key(&scene, &sim, &map, &xforms, id);
    assert!(before != after, "escalar a forma nao re-cozinhou o FX dela");
}

/// **A geometria DERIVADA entra na chave.**
///
/// Quando um produtor vivo (offset, contour, pattern, blend) substitui a forma, é ELE que é
/// desenhado na célula — e a forma autorada nem é lida. Um vão aqui seria a mesma doença uma camada
/// acima: o offset muda de cor e a textura mostra a antiga.
///
/// ⚠️ A fixture usa duas geometrias derivadas de **caixa idêntica** e cor diferente, de propósito: um
/// offset que muda de tamanho move `w`/`h` e seria pego pela chave antiga — o fenômeno que este gate
/// tem de conter é exactamente o que a caixa NÃO vê.
#[test]
fn changing_the_derived_geometry_re_cooks_the_shape() {
    let (scene, sim, map, xforms, id) = fixture();
    let sil = LiveGeometry::new();
    let derived = |c: Rgba8| {
        let mut m = LiveGeometry::new();
        m.insert(
            id,
            vec![VecPath {
                verts: [[-1.5, -1.5], [1.5, -1.5], [1.5, 1.5], [-1.5, 1.5]]
                    .map(VecVertex::corner)
                    .to_vec(),
                closed: true,
                fill: Some(Paint::solid(c)),
                ..VecPath::default()
            }],
        );
        m
    };
    let key_with = |live: &LiveGeometry| {
        job_for(
            &scene,
            &sim,
            &map,
            &xforms,
            live,
            &sil,
            Affine::IDENTITY,
            id,
        )
        .expect("a forma tem filtro e caixa")
        .key
    };
    let a = key_with(&derived(Rgba8::new(200, 30, 30, 255)));
    let b = key_with(&derived(Rgba8::new(30, 30, 200, 255)));
    assert_eq!(
        (a.w, a.h),
        (b.w, b.h),
        "a fixture nao contem o fenomeno: as duas geometrias derivadas tem caixas diferentes, entao \
         a chave antiga tambem as distinguiria"
    );
    assert!(
        a != b,
        "a geometria DERIVADA mudou (mesma caixa) e o memo ACERTOU — a textura mostra o offset da \
         era anterior"
    );
}
