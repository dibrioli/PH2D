//! Shared numeric + geometry helpers for the Color Equalization pipeline:
//! the `clamp8` rounding clamp used by every stage, plus the preview
//! aspect-fit + bilinear RGBA8 resize primitives.

pub(crate) fn clamp8(v: f32) -> u8 {
    if v < 0.0 {
        0
    } else if v >= 255.0 {
        255
    } else {
        (v + 0.5) as u8
    }
}

/// Aspect-fit `(sw, sh)` inside a `max_dim × max_dim` box without
/// upscaling. The preview cap uses this to bound CLAHE work per slider
/// drag (briefing PREVIEW cap 512²).
pub fn aspect_fit_within(sw: u32, sh: u32, max_dim: u32) -> (u32, u32) {
    if sw == 0 || sh == 0 || max_dim == 0 {
        return (sw.max(1), sh.max(1));
    }
    if sw <= max_dim && sh <= max_dim {
        return (sw, sh);
    }
    if sw >= sh {
        let dh = ((sh as u64 * max_dim as u64) / sw as u64).max(1) as u32;
        (max_dim, dh)
    } else {
        let dw = ((sw as u64 * max_dim as u64) / sh as u64).max(1) as u32;
        (dw, max_dim)
    }
}

/// Bilinear-interpolating RGBA8 resize, own implementation (no `image`
/// dep). Maps each destination pixel back to a fractional source position
/// and bilinearly samples the four neighbours per channel (alpha
/// included).
pub fn resize_bilinear_rgba(
    src_pixels: &[ph2d_color::SrgbRgba],
    sw: u32,
    sh: u32,
    dw: u32,
    dh: u32,
) -> Vec<u8> {
    let src: &[u8] = bytemuck::cast_slice(src_pixels);
    let mut dst = vec![0u8; (dw as usize) * (dh as usize) * 4];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return dst;
    }
    let sx_scale = sw as f32 / dw as f32;
    let sy_scale = sh as f32 / dh as f32;
    for y in 0..dh {
        let sy = (y as f32 + 0.5) * sy_scale - 0.5;
        let sy0 = sy.floor().max(0.0) as i32;
        let sy1 = (sy0 + 1).min(sh as i32 - 1);
        let sy0_c = sy0.clamp(0, sh as i32 - 1);
        let wy = (sy - sy0 as f32).clamp(0.0, 1.0);
        for x in 0..dw {
            let sx = (x as f32 + 0.5) * sx_scale - 0.5;
            let sx0 = sx.floor().max(0.0) as i32;
            let sx1 = (sx0 + 1).min(sw as i32 - 1);
            let sx0_c = sx0.clamp(0, sw as i32 - 1);
            let wx = (sx - sx0 as f32).clamp(0.0, 1.0);
            let dst_off = ((y as usize) * (dw as usize) + x as usize) * 4;
            for c in 0..4 {
                let p00 = src[((sy0_c as usize) * (sw as usize) + sx0_c as usize) * 4 + c] as f32;
                let p10 = src[((sy0_c as usize) * (sw as usize) + sx1 as usize) * 4 + c] as f32;
                let p01 = src[((sy1 as usize) * (sw as usize) + sx0_c as usize) * 4 + c] as f32;
                let p11 = src[((sy1 as usize) * (sw as usize) + sx1 as usize) * 4 + c] as f32;
                let top = p00 + wx * (p10 - p00);
                let bot = p01 + wx * (p11 - p01);
                dst[dst_off + c] = clamp8(top + wy * (bot - top));
            }
        }
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4×4 RGBA8 with a single solid colour + opaque alpha.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn clamp8_handles_overflow_and_underflow() {
        assert_eq!(clamp8(-10.0), 0);
        assert_eq!(clamp8(0.0), 0);
        assert_eq!(clamp8(127.4), 127);
        assert_eq!(clamp8(127.6), 128);
        assert_eq!(clamp8(255.0), 255);
        assert_eq!(clamp8(999.0), 255);
    }

    #[test]
    fn resize_bilinear_identity() {
        let src = solid(4, 4, [100, 150, 200]);
        let dst = resize_bilinear_rgba(bytemuck::cast_slice(&src), 4, 4, 4, 4);
        for (a, b) in dst.iter().zip(src.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn resize_bilinear_halves_dims() {
        let src = solid(8, 8, [100, 150, 200]);
        let dst = resize_bilinear_rgba(bytemuck::cast_slice(&src), 8, 8, 4, 4);
        assert_eq!(dst.len(), 4 * 4 * 4);
        // Solid colour → bilinear is identity-coloured.
        assert_eq!(&dst[..4], &[100, 150, 200, 255]);
    }

    #[test]
    fn aspect_fit_within_caps_larger_dimensions() {
        assert_eq!(aspect_fit_within(1024, 512, 512), (512, 256));
        assert_eq!(aspect_fit_within(512, 1024, 512), (256, 512));
        assert_eq!(aspect_fit_within(400, 300, 512), (400, 300));
    }
}
