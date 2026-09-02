//! Os gates do retrato de um Prefab ([`super`]).
//!
//! ⚠️ **O oráculo é o PIXEL**, e não «a função devolveu `Some`»: um compositor que desenhasse uma
//! peça só, ou que espelhasse tudo, devolveria `Some` na mesma.

use super::compose;
use ph2d_asset_index::Thumb;
use ph2d_ecs::{ChildOf, Entity, SimWorld, SpritePixels, Transform};

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn solid(c: [u8; 4]) -> Thumb {
    Thumb {
        rgba: std::sync::Arc::new(c.repeat(16 * 16)),
        w: 16,
        h: 16,
    }
}

/// Uma receita com peças em `(x, y)`, cada uma 1×1, e a cor que a miniatura dela devolve.
fn recipe(pieces: &[([f32; 2], [u8; 4])]) -> (SimWorld, Entity, Vec<Entity>, Vec<[u8; 4]>) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::MasterRoot))
        .id();
    let mut ents = Vec::new();
    let mut colors = Vec::new();
    for (i, (at, c)) in pieces.iter().enumerate() {
        let e = sim
            .world_mut()
            .spawn((
                Transform::from_translation(ph2d_core::Vec2::new(at[0], at[1])),
                ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
                SpritePixels(ph2d_asset::AssetId::from_digest([i as u8; 32])),
                ChildOf(root),
            ))
            .id();
        ents.push(e);
        colors.push(*c);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, root, ents, colors)
}

fn art(colors: Vec<[u8; 4]>) -> impl FnMut(ph2d_asset::AssetId) -> Option<Thumb> {
    move |id| {
        let b = id.as_bytes()[0] as usize;
        colors.get(b).copied().map(solid)
    }
}

fn pixel(t: &Thumb, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * t.w + x) * 4) as usize;
    [t.rgba[i], t.rgba[i + 1], t.rgba[i + 2], t.rgba[i + 3]]
}

/// ⭐⭐⭐ **AS DUAS PEÇAS APARECEM** — é a razão de existir do retrato.
///
/// Até 2026-09-01 o cartão mostrava a peça MAIOR, e um prefab de duas peças aparecia como uma.
///
/// (Mutação: compor só a primeira peça ⇒ RED.)
#[test]
fn both_pieces_show_up_in_the_portrait() {
    let (sim, root, pieces, colors) = recipe(&[([-1.0, 0.0], RED), ([1.0, 0.0], BLUE)]);
    let t = compose(&sim, root, &pieces, art(colors)).expect("o retrato");
    let mid = t.h / 2;
    let left = pixel(&t, 1, mid);
    let right = pixel(&t, t.w - 2, mid);
    assert_eq!(
        left[0], 255,
        "a peca da esquerda nao foi desenhada: {left:?}"
    );
    assert_eq!(
        right[2], 255,
        "a peca da direita nao foi desenhada: {right:?}"
    );
}

/// ⛔⛔ **O RETRATO NÃO É ESPELHADO** — a peça da esquerda no mundo fica à esquerda no retrato, e a
/// de cima fica em cima.
///
/// ⚠️ **É o defeito que passa por bom até alguém comparar:** o Y do mundo cresce para cima e o de
/// uma imagem cresce para baixo, e sem a inversão o retrato sai virado sem nada o denunciar.
///
/// (Mutação: tirar o `h - y` do `blit` ⇒ RED na metade vertical.)
#[test]
fn the_portrait_is_not_mirrored_on_either_axis() {
    let (sim, root, pieces, colors) = recipe(&[([-1.0, 1.0], RED), ([1.0, -1.0], BLUE)]);
    let t = compose(&sim, root, &pieces, art(colors)).expect("o retrato");
    // A vermelha está em cima-à-esquerda no MUNDO ⇒ em cima-à-esquerda na IMAGEM (linha 0).
    let top_left = pixel(&t, 1, 1);
    let bottom_right = pixel(&t, t.w - 2, t.h - 2);
    assert_eq!(
        top_left[0], 255,
        "a peca de cima-a-esquerda nao esta' la': {top_left:?}"
    );
    assert_eq!(
        bottom_right[2], 255,
        "a peca de baixo-a-direita nao esta' la': {bottom_right:?}"
    );
}

/// ⚠️ **A escala é UNIFORME** — uma disposição larga não estica a peça quadrada.
///
/// Sem isto o objecto que o artista tenta reconhecer sai deformado, que é a única coisa que um
/// retrato tem de não fazer.
#[test]
fn a_wide_layout_does_not_stretch_the_pieces() {
    let (sim, root, pieces, colors) = recipe(&[([-4.0, 0.0], RED), ([4.0, 0.0], BLUE)]);
    let t = compose(&sim, root, &pieces, art(colors)).expect("o retrato");
    assert!(
        t.w > t.h,
        "uma disposicao larga tem de dar um retrato largo: {}x{}",
        t.w,
        t.h
    );
    // A caixa é 10×1 em mundo ⇒ o retrato tem de manter essa razão (a menos de arredondamento).
    let ratio = t.w as f32 / t.h as f32;
    assert!(
        (7.0..=13.0).contains(&ratio),
        "o aspecto do retrato ({ratio:.2}) nao segue o da disposicao (10:1)"
    );
}

/// ⛔ **Sem peça com pixels não há retrato** — e o cartão fica com a cor dominante.
///
/// ⚠️ Inventar um cinzento diria que o prefab tem retrato. *Ele não tem.*
#[test]
fn a_recipe_with_no_pixels_has_no_portrait() {
    let mut sim = SimWorld::new();
    let root = sim.world_mut().spawn((Transform::IDENTITY,)).id();
    let child = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ChildOf(root)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(compose(&sim, root, &[child], |_| None).is_none());
}

/// ⛔⛔ **O retrato é DETERMINÍSTICO** — duas composições do mesmo mundo dão os mesmos bytes.
///
/// ⚠️ Sem a ordem total das peças elas trocavam de camada entre quadros ao sabor da ordem de
/// arquétipo, e **um cartão que pisca é pior que um cartão parcial**. ⚠️ E o memo do pintor
/// revalida por identidade de `Arc`: bytes iguais com `Arc` novo custam um reenvio ao atlas, mas
/// bytes DIFERENTES a cada quadro fariam o cartão tremer.
#[test]
fn the_portrait_is_deterministic() {
    let (sim, root, pieces, colors) = recipe(&[([-1.0, 0.0], RED), ([1.0, 0.0], BLUE)]);
    let a = compose(&sim, root, &pieces, art(colors.clone())).expect("a");
    let b = compose(&sim, root, &pieces, art(colors)).expect("b");
    assert_eq!(
        a.rgba, b.rgba,
        "duas composicoes iguais deram bytes diferentes"
    );
    assert_eq!((a.w, a.h), (b.w, b.h));
}

/// ⚠️ **Uma peça sem miniatura em cache é SALTADA, não é um retrato falhado** — o orçamento de
/// redução pode ter acabado no quadro, e o retrato parcial que sobra é melhor que nenhum.
#[test]
fn a_piece_without_art_is_skipped_and_the_rest_still_draws() {
    let (sim, root, pieces, _colors) = recipe(&[([-1.0, 0.0], RED), ([1.0, 0.0], BLUE)]);
    // Só a segunda tem arte.
    let t = compose(&sim, root, &pieces, |id| {
        (id.as_bytes()[0] == 1).then(|| solid(BLUE))
    })
    .expect("o retrato parcial");
    assert!(t.w > 0 && t.h > 0);
}
