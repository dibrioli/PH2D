//! `PH2D_SOCKET_SMOKE` — **as três formas de âncora, numa sprite só** ([ADR-0072]).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_SOCKET_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! ⚠️ **Nome distinto do [`crate::anchor_smoke`] de propósito.** Aquele é a cena das âncoras do
//! módulo VETOR (`PH2D_BUILD_SMOKE=52`, plano UI/UX W3) — outro conceito com o mesmo nome. Este é
//! a §12 do Inspector do sprite.
//!
//! Uma sprite com **três** âncoras, uma de cada forma, porque é a comparação que ensina o modelo:
//!
//! | nome | forma | o que se vê |
//! |---|---|---|
//! | `muzzle` | Socket | uma cruz |
//! | `hitbox` | Slice | cruz + retângulo |
//! | `panel_bg` | Região 9-slice | cruz + retângulo + retângulo interno |
//!
//! Cada uma com a **sua cor**, tirada do hash do nome.
//!
//! # ⚠️ Desde 2026-08-22 elas AGARRAM
//!
//! Até aqui a marca era decoração: o ADR §2.3 descrevia alças arrastáveis e o canvas não tinha
//! **uma linha** de tratamento de ponteiro para âncoras. Agora a âncora **aberta na lista** ganha
//! alças, e as outras ficam esmaecidas — dez alças em três âncoras seriam trinta alvos a disputar
//! o mesmo pixel.
//!
//! O que se pode fazer, com a seção §12 aberta e uma linha escolhida na lista:
//!
//! - **arrastar o quadrado do centro** → move a âncora (escreve os campos `Pos`);
//! - **arrastar o quadrado do braço** → roda-a (escreve `Rot`);
//! - **arrastar um canto do retângulo** → redimensiona-o, com o canto oposto quieto.
//!
//! O gesto inteiro é **um** passo de `Ctrl+Z`, e os números do painel andam ao vivo — as duas
//! metades escrevem pela mesma porta.

use ph2d_core::Vec2;
use ph2d_ecs::{NamedAnchor, NamedAnchorList};

/// Lado da sprite de fundo, em pixels.
const SRC_PX: u32 = 128;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_SOCKET_SMOKE").is_some()
}

/// Cria a sprite e as três âncoras. Devolve os bits da entidade.
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<u64> {
    let cell = *next_cell;
    let (_, bits) = crate::image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell,
        SRC_PX,
        1, // preto opaco: as cruzes coloridas leem-se sobre ele
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    )
    .ok()?;
    *next_cell += 1;

    let mut list = NamedAnchorList::new();
    // ⚠️ Pela porta que impõe os caps — nunca `list.0.push`. É a mesma porta que a UI usa, e um
    // smoke que a contornasse encenaria um estado que o gesto não consegue produzir (a cicatriz
    // do `impasto_smoke`).
    let mut muzzle = NamedAnchor::socket("muzzle");
    muzzle.transform.translation = Vec2::new(0.42, 0.18);
    list.insert(muzzle).ok()?;

    let mut hitbox = NamedAnchor::socket("hitbox");
    hitbox.transform.translation = Vec2::new(-0.30, -0.10);
    hitbox.set_bounds(Some([-24.0, -24.0, 48.0, 48.0]));
    list.insert(hitbox).ok()?;

    let mut panel = NamedAnchor::socket("panel_bg");
    panel.transform.translation = Vec2::new(0.0, -0.45);
    panel.set_bounds(Some([-40.0, -16.0, 80.0, 32.0]));
    panel.set_center(Some([-24.0, -8.0, 48.0, 16.0]));
    list.insert(panel).ok()?;

    let e = ph2d_ecs::Entity::from_bits(bits);
    if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
        ent.insert(list);
    }
    Some(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ As três formas têm de estar TODAS presentes — é a comparação que ensina o modelo. Um
    /// smoke só com sockets mostraria uma cruz e não explicaria o que `bounds`/`center` fazem.
    #[test]
    fn the_scene_shows_all_three_shapes() {
        let mut list = NamedAnchorList::new();
        let mut a = NamedAnchor::socket("muzzle");
        a.transform.translation = Vec2::new(0.42, 0.18);
        list.insert(a).unwrap();
        let mut b = NamedAnchor::socket("hitbox");
        b.set_bounds(Some([-24.0, -24.0, 48.0, 48.0]));
        list.insert(b).unwrap();
        let mut c = NamedAnchor::socket("panel_bg");
        c.set_bounds(Some([-40.0, -16.0, 80.0, 32.0]));
        c.set_center(Some([-24.0, -8.0, 48.0, 16.0]));
        list.insert(c).unwrap();

        let kinds: Vec<_> = list.iter().map(ph2d_ecs::NamedAnchor::kind).collect();
        assert_eq!(
            kinds,
            vec![
                ph2d_ecs::AnchorKind::Socket,
                ph2d_ecs::AnchorKind::Slice,
                ph2d_ecs::AnchorKind::NineSliceRegion
            ],
            "a cena tem de mostrar as tres formas"
        );
        // E as três com área não-nula onde ela existe — um retângulo de área zero é invisível.
        for a in list.iter() {
            if let Some(b) = a.bounds {
                assert!(b[2] > 0.0 && b[3] > 0.0, "'{}' tem area nula", a.name);
            }
        }
    }
}
