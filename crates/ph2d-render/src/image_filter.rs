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

/// Build a sprite sampler from a packed `RenderInstance::sampling` key
/// (Sprite Inspector v2 W3.T3.11): `filter_tag (low byte) | repeat_tag
/// << 8`, where the tags are the `ph2d_ecs::FilterMode`/`RepeatMode`
/// enum discriminants. The mipmapped/aniso variants now resolve to real
/// trilinear / anisotropic minification — both the atlas and the
/// individual store carry a full mip chain since Phase 2 (2026-06-18).
pub fn sampler_from_tags(device: &wgpu::Device, filter_tag: u8, repeat_tag: u8) -> wgpu::Sampler {
    // FilterMode: 1 Nearest · 2 Linear · 3 NearestMipmap · 4 LinearMipmap
    // · 5 NearestAniso · 6 LinearAniso (0 Inherit → Linear fallback).
    let filter = match filter_tag {
        1 | 3 | 5 => wgpu::FilterMode::Nearest,
        _ => wgpu::FilterMode::Linear,
    };
    // Trilinear (Linear mip blend) for every mipmapped/aniso variant now
    // that the sprite textures carry a real mip chain; plain Nearest(1)/
    // Linear(2) keep Nearest mip selection (effectively single-level).
    let mipmap_filter = match filter_tag {
        3..=6 => wgpu::MipmapFilterMode::Linear,
        _ => wgpu::MipmapFilterMode::Nearest,
    };
    // Anisotropy needs mag+min+mipmap ALL Linear (wgpu/Metal rule), so only
    // LinearAniso(6) qualifies; NearestAniso(5) keeps point mag/min and
    // falls back to 1× (it still gets the trilinear mip blend above).
    let anisotropy_clamp = if filter_tag == 6 { 16 } else { 1 };
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
