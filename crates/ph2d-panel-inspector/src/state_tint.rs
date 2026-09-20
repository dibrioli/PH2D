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

/// ⭐⭐ **Uma cor de SCRIPT em bytes** — a ponte `f64` → `f32` → `[u8; 4]`, numa porta só.
///
/// ⚠️ **O Luau mede em `f64` e toda a tinta deste app é `f32`**, logo há uma conversão a mais do
/// que nas irmãs. Ela vive AQUI, com as duas que compõe, porque os seus dois consumidores — o
/// pintor da fileira e a semente que fala com o selector — **têm de concordar ao byte**: a
/// comparação que decide se o barramento recebe uma edição é feita em `u8`, e duas aritméticas
/// diferentes dariam um fluxo de edições que nunca pára.
pub(crate) fn cor_do_script(c: [f64; 4]) -> [u8; 4] {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "um canal e' 0..=1 (conferido na declaracao): o f32 representa-o de sobra"
    )]
    let f = [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32];
    tint_f32_to_u8(f)
}

/// O inverso de [`cor_do_script`] — o que o selector devolveu, pronto a gravar.
pub(crate) fn cor_para_o_script(c: [u8; 4]) -> [f64; 4] {
    let f = tint_u8_to_f32(c);
    [
        f64::from(f[0]),
        f64::from(f[1]),
        f64::from(f[2]),
        f64::from(f[3]),
    ]
}
