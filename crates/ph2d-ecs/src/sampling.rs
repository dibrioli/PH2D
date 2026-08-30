//! Texture sampling components — Sprite Inspector v2 W3 (spec
//! [`09_sampling_e_material.md`](../../../docs/Sprite_projeto/09_sampling_e_material.md)
//! §9.1–9.2). Both are **hierarchical**: an entity's value overrides
//! the inherited one; `Inherit` defers to the nearest ancestor that
//! sets it, falling back to the project default.
//!
//! Material & Blend (spec §9.4–9.7) are a *W4* deliverable and not
//! defined here.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Per-node texture filter (Godot per-node filter, spec §9.1).
/// Hierarchical: `Inherit` reads the nearest ancestor override, then
/// the project default.
///
/// # ⛔⛔ A tag `5` existe no ARQUIVO e não existe no ENUM — e a falta é a cura
///
/// Ela era `NearestAniso`, e era um item de menu **fisicamente inalcançável**: o wgpu (e o Metal
/// por baixo) exige `mag`+`min`+`mipmap` os três `Linear` para `anisotropy_clamp > 1`, enquanto
/// *ampliar por ponto* é precisamente o que o nome dela promete ao artista. ⇒ O sampler que ela
/// produzia era **campo a campo idêntico** ao da `3 NearestMipmap`, e há gate a prová-lo do outro
/// lado ([`ph2d_render::image_filter`]`::tests::the_near_aniso_mode_is_the_near_mip_mode`).
///
/// ⚠️ **Os números NÃO se renumeraram, e essa é a metade que protege o disco.** A tag é o formato:
/// o `TextureFilter` viaja no `.ph2dproj` como **postcard** pelo registry de cena (um byte, medido),
/// e a lei de anisotropia do renderer é escrita sobre o literal `6`
/// ([`ph2d_render::image_filter::filter_tag_anisotropy`]). Encolher o `LinearAniso` para `5` faria
/// duas coisas em silêncio: todo ficheiro gravado com `Lin+Aniso` passaria a ler outro modo, e o
/// único modo que **pode** pedir anisotropia deixaria de a pedir.
///
/// ⇒ O `LinearAniso` fica em `6` por discriminante explícito, o `5` é lido por
/// [`Self::from_tag`] como `NearestMipmap` (que é o que ele **já era** na máquina), e o `serde`
/// desta enum passa pela tag em vez do índice de variante — vide o `impl Serialize` abaixo.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FilterMode {
    /// Defer to the ancestor / project default (component default).
    #[default]
    Inherit = 0,
    /// No filtering — ideal pixel-art.
    Nearest = 1,
    /// Bilinear — ideal vector UI / smooth.
    Linear = 2,
    /// Mipmapped, nearest within mip.
    NearestMipmap = 3,
    /// Mipmapped, linear within mip (trilinear).
    LinearMipmap = 4,
    // ⛔ 5 = `NearestAniso`, RETIRADO — vide o doc do enum. O buraco é deliberado e o
    //    `from_tag` responde por ele; ⛔ não o reaproveite para um modo novo, senão todo ficheiro
    //    gravado antes de hoje passa a desenhar outra coisa.
    /// Anisotropic + linear. ⚠️ **O único modo que pode pedir anisotropia**, e o `6` é lido como
    /// literal pela lei do renderer.
    LinearAniso = 6,
}

/// **O `serde` desta enum é a TAG, nunca o índice de variante** — e é isso que deixa uma variante
/// sair sem partir um ficheiro gravado.
///
/// ⚠️ **Medido antes de escrever** (`postcard`, 2026-08-30): a codificação de índice de variante é
/// um varint `u32`, que para `0..=6` é **um byte igual ao índice** — exactamente o que
/// `serialize_u8` emite. ⇒ para todo valor que alguma vez foi gravado, os bytes são os **mesmos**;
/// o que muda é que passam a ser os mesmos *por lei* em vez de por coincidência de ordenação.
impl Serialize for FilterMode {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(self.tag())
    }
}

impl<'de> Deserialize<'de> for FilterMode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::from_tag(u8::deserialize(d)?))
    }
}

/// Per-node texture filter override.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureFilter(pub FilterMode);

/// Per-node texture wrap mode (spec §9.2). Hierarchical like
/// [`FilterMode`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    /// Defer to the ancestor / project default.
    #[default]
    Inherit,
    /// Clamp to `[0, 1]`; outside pixels clamp the border.
    Disabled,
    /// Repeat tile (wrap).
    Enabled,
    /// Mirror-repeat (alternate).
    Mirror,
}

/// Per-node texture repeat override.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureRepeat(pub RepeatMode);

/// Per-sprite UV tiling/scroll transform (spec §9.2 use cases: tiling
/// without a TileMap, background scrolling). The sampled UV inside the
/// sprite's own sub-rect is `wrap(quad_uv * scale + offset)`, where the
/// wrap mode comes from the resolved [`RepeatMode`] — so `scale > 1`
/// tiles and `offset` scrolls, wrapping (Repeat) / mirroring (Mirror) /
/// clamping (Disabled) inside the sprite rect (no atlas bleed). Optional
/// component; absence = identity (`scale [1,1]`, `offset [0,0]`).
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UvTransform {
    pub scale: [f32; 2],
    pub offset: [f32; 2],
}

impl UvTransform {
    /// Identity (no tiling / no scroll).
    pub const IDENTITY: Self = Self {
        scale: [1.0, 1.0],
        offset: [0.0, 0.0],
    };
}

impl Default for UvTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl FilterMode {
    /// Resolve `self` against an inherited value: a concrete mode wins;
    /// `Inherit` defers to `inherited`. Used by the extract's
    /// ancestor walk.
    pub fn resolve(self, inherited: FilterMode) -> FilterMode {
        match self {
            FilterMode::Inherit => inherited,
            concrete => concrete,
        }
    }

    /// Enum discriminant as a `u8` tag (Inspector §9 segmented / snapshot
    /// / the renderer's packed sampling key). `0 Inherit … 6 LinearAniso`, **com o `5` vago**.
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// Inverse of [`Self::tag`]; out-of-range → `Inherit`.
    ///
    /// ⚠️ **O `5` é o tag APOSENTADO e continua a LER** — ele era `NearestAniso`, e cai em
    /// `NearestMipmap`, que é o sampler que a máquina de facto lhe dava. *Um ficheiro gravado com
    /// ele abre desenhando exactamente o que desenhava antes desta cura;* recusá-lo, ou deixá-lo
    /// cair no `Inherit`, seria mudar o que o artista vê ao abrir um projecto velho.
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => FilterMode::Nearest,
            2 => FilterMode::Linear,
            3 | 5 => FilterMode::NearestMipmap,
            4 => FilterMode::LinearMipmap,
            6 => FilterMode::LinearAniso,
            _ => FilterMode::Inherit,
        }
    }
}

impl RepeatMode {
    /// Enum discriminant as a `u8` tag. `0 Inherit · 1 Disabled · 2
    /// Enabled · 3 Mirror`.
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// Inverse of [`Self::tag`]; out-of-range → `Inherit`.
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => RepeatMode::Disabled,
            2 => RepeatMode::Enabled,
            3 => RepeatMode::Mirror,
            _ => RepeatMode::Inherit,
        }
    }
}

impl RepeatMode {
    pub fn resolve(self, inherited: RepeatMode) -> RepeatMode {
        match self {
            RepeatMode::Inherit => inherited,
            concrete => concrete,
        }
    }
}

/// Resolve the effective [`FilterMode`] for `entity` by walking the
/// `ChildOf` chain (Godot per-node hierarchy, spec §9.1): the nearest
/// ancestor-or-self with a concrete (non-`Inherit`) [`TextureFilter`]
/// wins; if every node up the chain is `Inherit` / absent, fall back to
/// `project_default`. Allocation-free; the chain is shallow.
pub fn resolve_texture_filter(
    world: &World,
    entity: Entity,
    project_default: FilterMode,
) -> FilterMode {
    let mut node = Some(entity);
    while let Some(n) = node {
        let here = world
            .get::<TextureFilter>(n)
            .map_or(FilterMode::Inherit, |f| f.0);
        if here != FilterMode::Inherit {
            return here;
        }
        node = world.get::<ChildOf>(n).map(|c| c.parent());
    }
    project_default
}

/// Resolve the effective [`RepeatMode`] for `entity` (mirror of
/// [`resolve_texture_filter`], spec §9.2).
pub fn resolve_texture_repeat(
    world: &World,
    entity: Entity,
    project_default: RepeatMode,
) -> RepeatMode {
    let mut node = Some(entity);
    while let Some(n) = node {
        let here = world
            .get::<TextureRepeat>(n)
            .map_or(RepeatMode::Inherit, |r| r.0);
        if here != RepeatMode::Inherit {
            return here;
        }
        node = world.get::<ChildOf>(n).map(|c| c.parent());
    }
    project_default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_defaults_to_inherit() {
        assert_eq!(TextureFilter::default().0, FilterMode::Inherit);
        assert_eq!(TextureRepeat::default().0, RepeatMode::Inherit);
    }

    #[test]
    fn resolve_prefers_concrete_over_inherited() {
        assert_eq!(
            FilterMode::Inherit.resolve(FilterMode::Nearest),
            FilterMode::Nearest
        );
        assert_eq!(
            FilterMode::Linear.resolve(FilterMode::Nearest),
            FilterMode::Linear
        );
        assert_eq!(
            RepeatMode::Inherit.resolve(RepeatMode::Enabled),
            RepeatMode::Enabled
        );
        assert_eq!(
            RepeatMode::Disabled.resolve(RepeatMode::Enabled),
            RepeatMode::Disabled
        );
    }

    #[test]
    fn resolve_walks_childof_to_nearest_concrete() {
        use bevy_ecs::hierarchy::ChildOf;
        let mut w = World::new();
        // root(Linear) → mid(Inherit) → leaf(absent): leaf resolves to
        // the root's Linear.
        let root = w.spawn(TextureFilter(FilterMode::Linear)).id();
        let mid = w
            .spawn((ChildOf(root), TextureFilter(FilterMode::Inherit)))
            .id();
        let leaf = w.spawn(ChildOf(mid)).id();
        assert_eq!(
            resolve_texture_filter(&w, leaf, FilterMode::Nearest),
            FilterMode::Linear
        );
        // A concrete override closer to the leaf wins.
        let leaf2 = w
            .spawn((ChildOf(mid), TextureFilter(FilterMode::Nearest)))
            .id();
        assert_eq!(
            resolve_texture_filter(&w, leaf2, FilterMode::Linear),
            FilterMode::Nearest
        );
    }

    #[test]
    fn resolve_falls_back_to_project_default() {
        let mut w = World::new();
        let e = w.spawn_empty().id();
        assert_eq!(
            resolve_texture_filter(&w, e, FilterMode::Linear),
            FilterMode::Linear
        );
        assert_eq!(
            resolve_texture_repeat(&w, e, RepeatMode::Enabled),
            RepeatMode::Enabled
        );
    }

    /// **Toda variante VIVA de [`FilterMode`], exaustiva por construção** — acrescentar ou tirar
    /// uma torna o `match` abaixo não-exaustivo e isto deixa de compilar.
    fn every_live_mode() -> Vec<FilterMode> {
        let all = vec![
            FilterMode::Inherit,
            FilterMode::Nearest,
            FilterMode::Linear,
            FilterMode::NearestMipmap,
            FilterMode::LinearMipmap,
            FilterMode::LinearAniso,
        ];
        for m in &all {
            // ⚠️ Sem braço `_`, de propósito: é este `match` que faz uma variante nova parar o
            // build em vez de nascer sem entrada nas duas leis abaixo.
            match m {
                FilterMode::Inherit
                | FilterMode::Nearest
                | FilterMode::Linear
                | FilterMode::NearestMipmap
                | FilterMode::LinearMipmap
                | FilterMode::LinearAniso => {}
            }
        }
        all
    }

    /// ⛔⛔ **O TAG APOSENTADO CONTINUA A LER, E LÊ O QUE SEMPRE DESENHOU.**
    ///
    /// A `5` era `NearestAniso` e a máquina dava-lhe o sampler da `3`. Um ficheiro gravado com ela
    /// tem de abrir desenhando o mesmo — nem recusa, nem queda para `Inherit` (que trocaria pixel
    /// duro por interpolação em silêncio, no controlo cuja razão de existir é o pixel duro).
    #[test]
    fn the_retired_tag_still_reads_and_lands_on_the_mode_it_always_was() {
        const RETIRED: u8 = 5;
        assert_eq!(
            FilterMode::from_tag(RETIRED),
            FilterMode::NearestMipmap,
            "o tag 5 (o antigo Near+Aniso) deixou de ler como Near+Mip — todo .ph2dproj gravado \
             com ele passa a desenhar outra coisa ao abrir"
        );
        // E pelo ARQUIVO, que é onde o defeito de facto aparece: um byte `5` no blob do
        // `TextureFilter` continua a chegar como `NearestMipmap`.
        let gravado: Vec<u8> = vec![RETIRED];
        assert_eq!(
            postcard::from_bytes::<TextureFilter>(&gravado)
                .expect("um TextureFilter gravado com o tag aposentado tem de DESSERIALIZAR")
                .0,
            FilterMode::NearestMipmap
        );
        // ⚠️ **A metade JUSTA:** nenhuma variante viva reclama o `5`. Sem ela, este gate ficaria
        // verde num enum que simplesmente renumerou tudo para baixo.
        for m in every_live_mode() {
            assert_ne!(
                m.tag(),
                RETIRED,
                "{m:?} ocupou o tag aposentado — o buraco no 5 e' o que mantem o 6 no sitio"
            );
        }
    }

    /// ⛔⛔ **O FORMATO DE ARQUIVO É A TAG** — golden byte a byte, e o `6` é load-bearing.
    ///
    /// ⚠️ O `serde` derivado escreveria o **índice de variante**, e tirar a `NearestAniso` do meio
    /// puxaria o `LinearAniso` de `6` para `5` **sem uma linha de erro**: todo ficheiro com
    /// `Lin+Aniso` passaria a ler `Near+Mip`, e a lei de anisotropia do renderer (escrita sobre o
    /// literal `6`) deixaria de encontrar quem a pede. É por isso que o `impl Serialize` é manual.
    #[test]
    fn the_wire_format_is_the_tag_of_the_mode_not_its_position() {
        for m in every_live_mode() {
            assert_eq!(
                postcard::to_allocvec(&m).unwrap(),
                vec![m.tag()],
                "{m:?} nao se grava como o proprio tag"
            );
        }
        // O golden que importa, escrito por extenso: o único modo que pode pedir anisotropia.
        assert_eq!(
            postcard::to_allocvec(&TextureFilter(FilterMode::LinearAniso)).unwrap(),
            vec![6u8],
            "o Lin+Aniso saiu do byte 6 — a lei `filter_tag_anisotropy` do renderer le' esse \
             literal, entao o unico modo anisotropico do app fica sem quem o peca"
        );
        // ⚠️ **A metade JUSTA:** modos distintos gravam bytes distintos. Sem ela, um `serialize`
        // que emitisse sempre `0` passaria as duas asserções de cima na variante `Inherit`.
        let mut vistos: Vec<Vec<u8>> = Vec::new();
        for m in every_live_mode() {
            let b = postcard::to_allocvec(&m).unwrap();
            assert!(!vistos.contains(&b), "{m:?} grava os bytes de outro modo");
            vistos.push(b);
        }
    }

    #[test]
    fn modes_serde_round_trip() {
        for m in every_live_mode() {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<FilterMode>(&b).unwrap(), m);
        }
        for m in [
            RepeatMode::Inherit,
            RepeatMode::Disabled,
            RepeatMode::Enabled,
            RepeatMode::Mirror,
        ] {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<RepeatMode>(&b).unwrap(), m);
        }
    }
}
