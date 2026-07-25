use super::*;

#[test]
fn impasto_light_wgsl_parses_and_validates_via_naga() {
    let module = naga::front::wgsl::parse_str(IMPASTO_LIGHT_WGSL)
        .unwrap_or_else(|e| panic!("impasto_light.wgsl failed naga parse: {e:?}"));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = validator.validate(&module);
    assert!(
        r.is_ok(),
        "impasto_light.wgsl failed naga validation: {:?}",
        r.err()
    );
}

/// **The literal-level parity gate**, and the one that needs no device — so it runs on every CI
/// runner, not only the GPU lane.
///
/// Runtime output cannot be bit-identical across backends (a backend may contract `a * b + c` into an
/// FMA), so what CAN be pinned exactly is pinned exactly: the constants. `DEPTH_UNIT_PX` is the
/// height-to-pixel gain the whole model hangs on — a shader carrying a stale copy would render the
/// same painting at a different depth and no gate downstream would say why.
#[test]
fn impasto_light_shader_constants_match_the_cpu_pass() {
    // Kept as string checks against the WGSL source rather than shared constants, because the WGSL is
    // a separate compilation unit: it is exactly the drift a shared `const` cannot catch.
    for (decl, why) in [
        (
            "const DEPTH_UNIT_PX: f32 = 16.0;",
            "the height-to-pixel gain (impasto_light::DEPTH_UNIT_PX)",
        ),
        (
            "const AMBIENT: f32 = 0.35;",
            "the diffuse floor (impasto_light::AMBIENT)",
        ),
        (
            "const SPEC_LUT_LAST: f32 = 255.0;",
            "material::SPEC_LUT - 1",
        ),
        ("const ROUGH_LAST: i32 = 64;", "material::ROUGH_LEVELS - 1"),
        (
            "const FLAT_FLOOR: f32 = 1.0e-4;",
            "the divisor floor in `channel`",
        ),
        ("array<Lamp, 4>", "impasto_rig::MAX_LIGHTS"),
    ] {
        assert!(
            IMPASTO_LIGHT_WGSL.contains(decl),
            "impasto_light.wgsl must declare `{decl}` — {why}"
        );
    }
    assert_eq!(
        IMPASTO_MAX_LIGHTS, 4,
        "the uniform's lamp array and impasto_rig::MAX_LIGHTS are the same number"
    );
}

/// The shader must NOT reach for the fast reciprocal square root. The CPU normalises by `sqrt` and a
/// divide; `inverseSqrt` is allowed to be approximate, and swapping it in would move the normal — and
/// therefore every lit pixel — for a handful of ALU cycles nobody asked for.
#[test]
fn the_normal_is_a_real_sqrt_not_an_approximation() {
    assert!(
        !IMPASTO_LIGHT_WGSL.contains("inverseSqrt"),
        "the normal must divide by a real sqrt, as the CPU pass does"
    );
    assert!(
        IMPASTO_LIGHT_WGSL.contains("max(sqrt("),
        "…and floor the length exactly as `Rig::shade` does"
    );
}

/// Every mis-shaped request is refused BEFORE a GPU resource is touched, so this runs device-free.
/// A plane of the wrong size is the dangerous one: it would light the painting from a relief that is
/// not on it, and the result would look like a bug in the sculpt rather than a bug in the upload.
#[test]
fn a_mis_shaped_request_is_refused_without_a_device() {
    let lamp = ImpastoLamp {
        dir: [0.0, 0.0, 1.0],
        half: [0.0, 0.0, 1.0],
        tint: [1.0; 3],
    };
    let lut = vec![0.0f32; 8];
    let relief = vec![0.0f32; 4];
    let cover = vec![0u8; 4];
    let mats = vec![0u8; 16];
    let base = || ImpastoLightInput {
        width: 2,
        height: 2,
        region: crate::layer_compositor::Region::full(2, 2),
        plane_region: crate::layer_compositor::Region::full(2, 2),
        relief: &relief,
        cover: &cover,
        mat0: &mats,
        mat1: &mats,
        lamps: std::slice::from_ref(&lamp),
        spec_lut: &lut,
        lut_width: 4,
        rough_levels: 2,
    };
    assert_eq!(base().check(), Ok(()), "a well-formed request passes");

    let mut short_relief = base();
    short_relief.relief = &relief[..3];
    assert_eq!(
        short_relief.check(),
        Err(ImpastoLightError::PlaneSize),
        "a short relief plane is refused, not silently read past"
    );

    let mut short_mat = base();
    short_mat.mat1 = &mats[..12];
    assert_eq!(
        short_mat.check(),
        Err(ImpastoLightError::PlaneSize),
        "…and so is a short material plane"
    );

    let mut dark = base();
    dark.lamps = &[];
    assert_eq!(
        dark.check(),
        Err(ImpastoLightError::LampCount),
        "no lamp is a caller bug: the CPU seam hands over no planes at all when the rig is dark"
    );

    let mut bad_lut = base();
    bad_lut.lut_width = 3;
    assert_eq!(bad_lut.check(), Err(ImpastoLightError::LutSize));

    let mut empty = base();
    empty.region.w = 0;
    assert_eq!(
        empty.check(),
        Err(ImpastoLightError::EmptyExtent),
        "an empty region would dispatch nothing and report success"
    );
}
