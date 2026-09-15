//! **As duas conversões de TINTA do Inspector** — irmão do [`super::state`] por CAP de LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** o `state.rs` é o registo dos
//! instantâneos que a shell publica por quadro, e estas duas são **aritmética de cor** — elas não
//! guardam estado nenhum. O tecto foi o que obrigou a olhar, e o sítio certo aparece sozinho.

/// Pack a linear/sRGB f32 RGBA in `[0, 1]` into `[u8; 4]` for the
/// color-swatch fill + `INSP_BLENDER_PICKER` seed. Round-to-nearest
/// (the `+ 0.5` before truncation) so a committed channel and its
/// re-decoded byte agree, and the picker doesn't reopen one step off.
pub(crate) fn tint_f32_to_u8(c: [f32; 4]) -> [u8; 4] {
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit denormalize, not a design token
    [q(c[0]), q(c[1]), q(c[2]), q(c[3])]
}

/// Inverse of [`tint_f32_to_u8`]: the picker round-trips the chosen
/// color through `widget_color(target)` as `[u8; 4]`; this unpacks it
/// back to the `[f32; 4]` the `Sprite` tint channels store.
pub(crate) fn tint_u8_to_f32(c: [u8; 4]) -> [f32; 4] {
    // LITERAL-PX-OK (×4): sRGB 8-bit normalize, not a design token.
    [
        c[0] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
        c[1] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
        c[2] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
        c[3] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
    ]
}
