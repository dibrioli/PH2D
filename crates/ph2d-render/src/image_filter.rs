//! Maps the app-wide [`ImageFilterMode`] onto the wgpu sampler filter
//! used by every sprite texture (atlas + individual).
//!
//! The enum itself lives in `ph2d-host` (zero-dep, shared by render and
//! editor-core). This module is just the render-side mapping so the
//! sampler descriptors in [`crate::atlas`] and [`crate::individual`]
//! don't each hardcode a `FilterMode` — they call [`wgpu_filter`] with
//! the current mode, guaranteeing atlas and individual stay in sync.

pub use ph2d_host::ImageFilterMode;

/// The `wgpu::FilterMode` (mag + min) that `mode` selects for sprite
/// sampling. `Nearest` for crisp pixel art, `Linear` for smooth.
pub fn wgpu_filter(mode: ImageFilterMode) -> wgpu::FilterMode {
    match mode {
        ImageFilterMode::PixelArt => wgpu::FilterMode::Nearest,
        ImageFilterMode::Smooth => wgpu::FilterMode::Linear,
    }
}

/// Build the canonical sprite sampler for `mode`. Both the atlas and
/// the individual-texture store call this so there is exactly ONE
/// sampler descriptor in the codebase — no more divergent hardcoded
/// `Linear` (atlas) vs `Nearest` (individual).
pub fn create_sprite_sampler(
    device: &wgpu::Device,
    mode: ImageFilterMode,
    label: &str,
) -> wgpu::Sampler {
    let filter = wgpu_filter(mode);
    // Anisotropic filtering sharpens MINIFICATION (zoomed-out / oblique sprites)
    // well beyond plain trilinear — it takes multiple taps along the axis of
    // greatest compression instead of one isotropic mip sample, so a high-res
    // canvas viewed small keeps crisp edges instead of the trilinear "safe blur".
    // Requires all three filters Linear + a real mip chain — both true for the
    // mipmapped individual-texture store AND the mipmapped atlas (Phase 2); a
    // no-op fallback only on Nearest (pixel art, which can't use aniso anyway).
    // `16×` is the universal wgpu/Metal max.
    let anisotropy_clamp = if filter == wgpu::FilterMode::Linear {
        16
    } else {
        1
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        // Linear between mip levels = trilinear minification (no level "popping").
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp,
        ..Default::default()
    })
}

/// **A lei de ampliação de uma tag de `FilterMode`, PURA — `true` = ponto (pixel duro).**
///
/// `1 Nearest · 3 NearestMipmap · 5 NearestAniso` ampliam por ponto; todo o resto (incluindo
/// `0 Inherit`, cujo fallback é linear) interpola.
///
/// # Por que isto é uma função e não três linhas dentro do `sampler_from_tags`
///
/// ⚠️ Enquanto a lei viveu lá dentro, **medi-la exigia um `wgpu::Device`** — e por isso ninguém a
/// media. O painel do Inspector acendia «Linear» para as tags 3 e 5 (auditoria de 2026-08-21,
/// `docs/Sprite_projeto/20` §2.2): ele dizia o contrário do que o ecrã desenhava, e nenhum gate
/// podia notar, porque o único sítio que sabia a resposta precisava de GPU para responder.
///
/// Agora a lei é `const`, mora num sítio, o `sampler_from_tags` consome-a, e o gate
/// `the_filter_segmented_tells_the_truth_about_what_renders` (shell) mede-a **sem adapter nenhum**.
pub const fn filter_tag_magnifies_by_point(filter_tag: u8) -> bool {
    matches!(filter_tag, 1 | 3 | 5)
}

/// **A lei de MINIFICAÇÃO entre níveis de mip de uma tag, PURA — `true` = mistura trilinear.**
///
/// `3 NearestMipmap · 4 LinearMipmap · 5 NearestAniso · 6 LinearAniso` misturam dois níveis; o
/// `1 Nearest` e o `2 Linear` escolhem um só (as texturas de sprite têm cadeia de mip desde a
/// Fase 2, então «Nearest mip» é efectivamente nível único), e o `0 Inherit` cai com eles.
///
/// Irmã da [`filter_tag_magnifies_by_point`], e extraída pela MESMA razão: enquanto ela vivia
/// dentro do `sampler_from_tags`, medi-la exigia um `wgpu::Device`.
pub const fn filter_tag_blends_mips(filter_tag: u8) -> bool {
    matches!(filter_tag, 3..=6)
}

/// **A lei de ANISOTROPIA de uma tag, PURA** — quantas amostras o sampler pode tomar ao longo do
/// eixo de maior compressão. `1` = isotrópico (o neutro do wgpu), `16` = o máximo universal
/// wgpu/Metal.
///
/// # ⛔⛔ Só a tag `6 LinearAniso` pode pedir anisotropia, e isso NÃO é uma escolha nossa
///
/// A regra é do wgpu (e do Metal por baixo): `anisotropy_clamp > 1` exige `mag_filter`,
/// `min_filter` **e** `mipmap_filter` os três em `Linear`. A tag `5 NearestAniso` amplia por
/// PONTO por definição — é isso que o nome dela promete ao artista —, logo *ampliar por ponto* e
/// *pedir anisotropia* são pedidos **contraditórios**, e o que a máquina pode entregar é um dos
/// dois.
///
/// ⇒ O sampler que a `5` produz é **campo a campo idêntico** ao da `3 NearestMipmap`, e há gate a
/// prová-lo ([`the_near_aniso_mode_is_the_near_mip_mode`]). Ela não é um modo caro, nem um modo
/// por implementar: ela é um item de menu que **não pode existir**, e o remate desta cura é
/// retirá-la do selector — ⚠️ trabalho em `ph2d-ecs` (o enum) e `ph2d-panel-inspector` (os
/// rótulos), **crates que esta linha não possui**. Este gate é a rede que impede que ela volte a
/// ser reintroduzida como se fosse um modo distinto.
///
/// [`the_near_aniso_mode_is_the_near_mip_mode`]: self::tests::the_near_aniso_mode_is_the_near_mip_mode
pub const fn filter_tag_anisotropy(filter_tag: u8) -> u16 {
    if filter_tag == 6 { 16 } else { 1 }
}

/// O maior tag de filtro que `ph2d_ecs::FilterMode::from_tag` sabe distinguir.
///
/// ⚠️ **Existe para quem OFERECE filtros num menu.** Um consumidor que clampasse
/// por um literal próprio ou ofereceria um modo que o `from_tag` devolve como
/// `Inherit` (item de menu morto), ou pararia antes do último (modo inalcançável)
/// — e nenhum dos dois dá erro. O número não é auto-evidente: ele é
/// **verificado** por [`the_filter_tag_ceiling_is_the_last_distinct_mode`], que
/// afirma que este tag é concreto e que o seguinte já cai no fallback.
pub const FILTER_TAG_MAX: u8 = 6;

/// O maior tag de repetição que `ph2d_ecs::RepeatMode::from_tag` distingue.
/// Mesmo papel do [`FILTER_TAG_MAX`], com o mesmo gate ao lado.
pub const REPEAT_TAG_MAX: u8 = 3;

/// Build a sprite sampler from a packed `RenderInstance::sampling` key
/// (Sprite Inspector v2 W3.T3.11): `filter_tag (low byte) | repeat_tag
/// << 8`, where the tags are the `ph2d_ecs::FilterMode`/`RepeatMode`
/// enum discriminants. The mipmapped/aniso variants now resolve to real
/// trilinear / anisotropic minification — both the atlas and the
/// individual store carry a full mip chain since Phase 2 (2026-06-18).
pub fn sampler_from_tags(device: &wgpu::Device, filter_tag: u8, repeat_tag: u8) -> wgpu::Sampler {
    let filter = if filter_tag_magnifies_by_point(filter_tag) {
        wgpu::FilterMode::Nearest
    } else {
        wgpu::FilterMode::Linear
    };
    let mipmap_filter = if filter_tag_blends_mips(filter_tag) {
        wgpu::MipmapFilterMode::Linear
    } else {
        wgpu::MipmapFilterMode::Nearest
    };
    let anisotropy_clamp = filter_tag_anisotropy(filter_tag);
    // RepeatMode: 1 Disabled (clamp) · 2 Enabled (repeat) · 3 Mirror
    // (0 Inherit → clamp fallback).
    let address = match repeat_tag {
        2 => wgpu::AddressMode::Repeat,
        3 => wgpu::AddressMode::MirrorRepeat,
        _ => wgpu::AddressMode::ClampToEdge,
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("ph2d-render per-node sprite sampler"),
        address_mode_u: address,
        address_mode_v: address,
        address_mode_w: address,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        anisotropy_clamp,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O TETO DOS TAGS É O ÚLTIMO MODO CONCRETO** — derivado do `from_tag`, que
    /// é a lei, e não escrito ao lado dela.
    ///
    /// ⚠️ Falsificado nos DOIS sentidos: um modo novo no `ph2d-ecs` sem mexer aqui
    /// deixa o teto baixo (e o menu de quem o lê fica sem o modo novo), e um teto
    /// alto demais oferece um tag que o `from_tag` devolve como `Inherit`.
    #[test]
    fn the_filter_tag_ceiling_is_the_last_distinct_mode() {
        use ph2d_ecs::{FilterMode, RepeatMode};
        assert_ne!(
            FilterMode::from_tag(FILTER_TAG_MAX),
            FilterMode::Inherit,
            "FILTER_TAG_MAX tem de ser um modo CONCRETO"
        );
        assert_eq!(
            FilterMode::from_tag(FILTER_TAG_MAX + 1),
            FilterMode::Inherit,
            "o tag seguinte ja' tem de cair no fallback — senao o teto esta' baixo"
        );
        assert_ne!(RepeatMode::from_tag(REPEAT_TAG_MAX), RepeatMode::Inherit);
        assert_eq!(
            RepeatMode::from_tag(REPEAT_TAG_MAX + 1),
            RepeatMode::Inherit
        );
    }

    /// ⛔⛔ **`Near+Aniso` (tag 5) É `Near+Mip` (tag 3) — o mesmo sampler, campo a campo.**
    ///
    /// O menu *Texture Filter* do Inspector oferece sete modos e **seis** são distinguíveis. A
    /// `5 NearestAniso` promete anisotropia sobre ampliação por ponto, e o wgpu recusa a
    /// combinação: `anisotropy_clamp > 1` exige `mag`+`min`+`mipmap` os três `Linear`. Ver
    /// [`filter_tag_anisotropy`] para o mecanismo.
    ///
    /// # Por que a régua são as TRÊS leis, e não o `Sampler`
    ///
    /// ⚠️ Um `wgpu::Sampler` precisa de `Device` — precisa de **adapter**, logo o gate seria
    /// `#[ignore]` e o CI nunca o correria (a família documentada no CLAUDE.md §5.0). As três leis
    /// puras SÃO o descritor: elas são os únicos campos que o `filter_tag` decide (o endereçamento
    /// vem do `repeat_tag`, e o resto é `Default`). Comparar as leis é comparar o descritor, sem
    /// GPU nenhuma.
    ///
    /// ⚠️ **E o gate afirma as duas metades.** Sem a segunda — que os outros pares de tags
    /// vizinhas **diferem** — um `filter_tag_anisotropy` que devolvesse `1` para tudo passaria, e
    /// um que devolvesse sempre o mesmo `mipmap_filter` também.
    #[test]
    fn the_near_aniso_mode_is_the_near_mip_mode() {
        // As três leis que o `filter_tag` decide, na ordem em que o descritor as consome.
        let law = |tag: u8| {
            (
                filter_tag_magnifies_by_point(tag),
                filter_tag_blends_mips(tag),
                filter_tag_anisotropy(tag),
            )
        };
        const NEAREST_MIPMAP: u8 = 3;
        const NEAREST_ANISO: u8 = 5;
        assert_eq!(
            law(NEAREST_ANISO),
            law(NEAREST_MIPMAP),
            "a tag 5 (Near+Aniso) deixou de ser indistinguivel da 3 (Near+Mip). Se isto ficou \
             VERMELHO porque o wgpu passou a aceitar anisotropia com ampliacao por ponto, a nota \
             de `filter_tag_anisotropy` caducou e a opcao pode voltar ao selector"
        );
        assert_eq!(
            filter_tag_anisotropy(NEAREST_ANISO),
            1,
            "o modo Near+Aniso pediu anisotropia: o wgpu rejeita `anisotropy_clamp > 1` sem os \
             tres filtros Lineares, e a validacao mata a criacao do sampler"
        );

        // A metade JUSTA: as leis distinguem de facto os modos que SÃO distintos.
        assert_ne!(law(1), law(NEAREST_MIPMAP), "Nearest e Near+Mip colidiram");
        assert_ne!(law(2), law(4), "Linear e Lin+Mip colidiram");
        assert_ne!(law(4), law(6), "Lin+Mip e Lin+Aniso colidiram");
        assert_eq!(
            filter_tag_anisotropy(6),
            16,
            "o unico modo que PODE pedir anisotropia deixou de a pedir"
        );
    }

    #[test]
    fn pixel_art_maps_to_nearest() {
        assert_eq!(
            wgpu_filter(ImageFilterMode::PixelArt),
            wgpu::FilterMode::Nearest
        );
    }

    #[test]
    fn smooth_maps_to_linear() {
        assert_eq!(
            wgpu_filter(ImageFilterMode::Smooth),
            wgpu::FilterMode::Linear
        );
    }
}
