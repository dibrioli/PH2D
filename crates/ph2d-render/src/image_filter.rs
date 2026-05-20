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
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
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
