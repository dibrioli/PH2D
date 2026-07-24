use super::*;

#[test]
fn max_layers_for_budget_4k_and_degenerate() {
    // 4K RGBA8 = 33,177,600 B/slice; 512 MiB / that = 16 slices.
    assert_eq!(
        max_layers_for_budget(3840, 2160, LAYER_CACHE_BUDGET_BYTES),
        16
    );
    // Zero-area canvas → 0 (no divide-by-zero).
    assert_eq!(max_layers_for_budget(0, 0, LAYER_CACHE_BUDGET_BYTES), 0);
    // Tiny canvas is bounded by HARD_CAP_LAYERS, not the byte budget.
    assert_eq!(
        max_layers_for_budget(1, 1, LAYER_CACHE_BUDGET_BYTES),
        HARD_CAP_LAYERS
    );
}

/// The POD mirrors must be exactly the size the WGSL structs are, because a
/// storage-buffer array is read by STRIDE: a Rust struct one word wider than its
/// shader twin does not fail to bind — it reads every op from the wrong offset,
/// and the picture is garbage with no error anywhere.
///
/// `GpuOp` grew 16 → 32 when the coverage modifiers landed (`mask_slot`, `flags`
/// and two pads); the WGSL `Op` grew the same four words in the same order.
#[test]
fn gpu_pod_sizes_match_wgsl() {
    assert_eq!(core::mem::size_of::<GpuOp>(), 32);
    assert_eq!(core::mem::size_of::<GpuGlobals>(), 32);
}

/// The `Op` field ORDER is the binding contract, and `size_of` alone cannot see
/// it: swap `mask_slot` with `flags` and the struct is still 32 bytes while every
/// op silently reads a slice index as a bitfield.
///
/// So the WGSL declaration is read as text and its member order compared with
/// this file's. Cheap, CPU-only, and it runs everywhere — the same discipline as
/// `shader_blend_modes_bit_identical_with_rust`.
#[test]
fn gpu_op_field_order_matches_the_wgsl_struct() {
    let src = LAYER_COMPOSITE_WGSL;
    let start = src
        .find("struct Op {")
        .expect("the WGSL declares `struct Op`");
    let body = &src[start..][..src[start..].find('}').expect("closed")];
    let fields: Vec<&str> = body
        .lines()
        .skip(1)
        .filter_map(|l| l.trim().split(':').next())
        .filter(|n| !n.is_empty() && !n.starts_with("//"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "kind",
            "layer_slot",
            "blend_mode",
            "opacity",
            "mask_slot",
            "flags",
            "_pad0",
            "_pad1",
        ],
        "WGSL `Op` members drifted from the Rust `GpuOp` layout"
    );
}

/// The decode LUT the shader binds must equal the canonical
/// `srgb_to_linear_byte` for every byte — this is what makes the GPU decode
/// bit-identical to the CPU `compositor::decode` (the table replaced the
/// in-shader `pow`, so this gate, not the textual one, pins decode parity).
#[test]
fn srgb_lut_matches_cpu_transfer() {
    let lut = build_srgb_lut();
    assert_eq!(lut.len(), 256);
    for b in 0..=255u8 {
        assert_eq!(
            lut[b as usize].to_bits(),
            srgb_to_linear_byte(b).to_bits(),
            "LUT[{b}] drifted from srgb_to_linear_byte",
        );
    }
    // The shader recovers the byte index via `round(raw * 255)`; assert the
    // WGSL still indexes the table that way (guards a refactor that breaks
    // the unorm→byte recovery).
    assert!(LAYER_COMPOSITE_WGSL.contains("srgb_lut[u32(raw.r * 255.0 + 0.5)]"));
}

#[test]
fn validate_op_list_balance_and_depth() {
    let layer = LayerOp::Layer {
        mask: None,
        clipping: false,
        key: 1,
        blend_mode: 0,
        opacity: 1.0,
    };
    // Balanced.
    assert!(
        validate_op_list(&[
            layer,
            LayerOp::PushGroup,
            layer,
            LayerOp::PopGroup {
                blend_mode: 0,
                opacity: 1.0
            },
        ])
        .is_ok()
    );
    // Unbalanced (push without pop).
    assert_eq!(
        validate_op_list(&[LayerOp::PushGroup]),
        Err(LayerCompositeError::MalformedOpList)
    );
    // Pop without push.
    assert_eq!(
        validate_op_list(&[LayerOp::PopGroup {
            blend_mode: 0,
            opacity: 1.0
        }]),
        Err(LayerCompositeError::MalformedOpList)
    );
    // Too deep (> MAX_GROUP_DEPTH nested pushes).
    let mut deep = Vec::new();
    for _ in 0..MAX_STACK {
        deep.push(LayerOp::PushGroup);
    }
    assert_eq!(
        validate_op_list(&deep),
        Err(LayerCompositeError::MalformedOpList)
    );
}

#[test]
fn flatten_layer_ops_resolves_slots_and_reuses_scratch() {
    let slot_of = |k: u64| match k {
        9 => 1,
        7 => 3,
        _ => 0,
    };
    let ops = vec![
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 9,
            blend_mode: 1,
            opacity: 0.5,
        },
        LayerOp::Adjustment {
            mask: None,
            kind: 0, // ADJ_HSB
            params: [0.25, 0.5, -0.1],
            blend_mode: 0,
            opacity: 0.75,
        },
        LayerOp::PushGroup,
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 7,
            blend_mode: 11,
            opacity: 1.0,
        },
        LayerOp::PopGroup {
            blend_mode: 6,
            opacity: 0.8,
        },
    ];
    let mut scratch = GpuOpScratch::new();
    flatten_layer_ops(&ops, slot_of, |k| Some(slot_of(k)), &mut scratch);
    assert_eq!(scratch.ops.len(), 5);
    assert_eq!(scratch.ops[0].kind, OP_LAYER);
    assert_eq!(scratch.ops[0].layer_slot, 1); // key 9 → slice 1
    assert_eq!(scratch.ops[0].blend_mode, 1);
    // The adjustment op carries its params index in layer_slot + its own blend/opacity.
    assert_eq!(scratch.ops[1].kind, OP_ADJUSTMENT);
    assert_eq!(scratch.ops[1].layer_slot, 0); // first (only) adjustment → params[0]
    assert!((scratch.ops[1].opacity - 0.75).abs() < 1e-6);
    assert_eq!(scratch.adj.len(), 1);
    assert_eq!(scratch.adj[0].kind, 0);
    assert!((scratch.adj[0].p0 - 0.25).abs() < 1e-6);
    assert!((scratch.adj[0].p2 + 0.1).abs() < 1e-6);
    assert_eq!(scratch.ops[2].kind, OP_PUSH_GROUP);
    assert_eq!(scratch.ops[3].layer_slot, 3); // key 7 → slice 3
    assert_eq!(scratch.ops[4].kind, OP_POP_GROUP);
    assert_eq!(scratch.ops[4].blend_mode, 6);

    // Re-flattening into the same scratch must not grow capacity (HR-3).
    let cap = scratch.capacity();
    flatten_layer_ops(&ops, slot_of, |k| Some(slot_of(k)), &mut scratch);
    assert_eq!(scratch.capacity(), cap);
}

#[test]
fn distinct_layer_count_dedupes() {
    let ops = vec![
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 2,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::PushGroup,
    ];
    assert_eq!(distinct_layer_count(&ops), 2);
}

// ── WGSL reflection / parity gates (CPU-only, no GPU) ─────────────────

#[test]
fn layer_composite_wgsl_parses_via_naga() {
    let r = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL);
    assert!(
        r.is_ok(),
        "layer_composite.wgsl failed naga parse: {:?}",
        r.err()
    );
}

#[test]
fn layer_composite_wgsl_validates_via_naga() {
    let module = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL).expect("must parse");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let r = validator.validate(&module);
    assert!(
        r.is_ok(),
        "layer_composite.wgsl failed naga validation: {:?}",
        r.err()
    );
}

#[test]
fn shader_workgroup_size_is_8x8x1() {
    let module = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL).expect("must parse");
    // Both entry points (flat + grouped) must use the 8×8 tiling.
    for name in ["cs_flat", "cs_grouped"] {
        let ep = module
            .entry_points
            .iter()
            .find(|ep| ep.name == name)
            .unwrap_or_else(|| panic!("{name} entry point"));
        assert_eq!(ep.workgroup_size, [WORKGROUP_EDGE, WORKGROUP_EDGE, 1]);
    }
}

fn wgsl_struct_span(name: &str) -> usize {
    use naga::TypeInner;
    let module = naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL).expect("must parse");
    for (_, ty) in module.types.iter() {
        if let TypeInner::Struct { span, .. } = ty.inner
            && ty.name.as_deref() == Some(name)
        {
            return span as usize;
        }
    }
    panic!("`{name}` struct not found in layer_composite.wgsl");
}

#[test]
fn shader_struct_sizes_match_rust_abi() {
    assert_eq!(wgsl_struct_span("Op"), core::mem::size_of::<GpuOp>());
    assert_eq!(
        wgsl_struct_span("Globals"),
        core::mem::size_of::<GpuGlobals>()
    );
    // Spatial pass-graph (W4) globals — Rust POD ↔ WGSL uniform mirrors.
    assert_eq!(
        wgsl_struct_span("SegGlobals"),
        core::mem::size_of::<SegGlobals>()
    );
    assert_eq!(
        wgsl_struct_span("BlurGlobals"),
        core::mem::size_of::<BlurGlobals>()
    );
    assert_eq!(
        wgsl_struct_span("CombineGlobals"),
        core::mem::size_of::<CombineGlobals>()
    );
    assert_eq!(
        wgsl_struct_span("EncodeGlobals"),
        core::mem::size_of::<EncodeGlobals>()
    );
    assert_eq!(
        wgsl_struct_span("ChromaGlobals"),
        core::mem::size_of::<ChromaGlobals>()
    );
}

#[test]
fn shader_op_kind_discriminants_match_wgsl() {
    // The Rust `OP_*` and WGSL `OP_*` must agree, and the WGSL `switch`
    // arms (`case 0u/1u/2u`) dispatch on them.
    assert!(LAYER_COMPOSITE_WGSL.contains("const OP_LAYER: u32 = 0u;"));
    assert!(LAYER_COMPOSITE_WGSL.contains("const OP_PUSH_GROUP: u32 = 1u;"));
    assert!(LAYER_COMPOSITE_WGSL.contains("const OP_POP_GROUP: u32 = 2u;"));
    assert!(LAYER_COMPOSITE_WGSL.contains("const OP_ADJUSTMENT: u32 = 3u;"));
    assert_eq!(OP_LAYER, 0);
    assert_eq!(OP_PUSH_GROUP, 1);
    assert_eq!(OP_POP_GROUP, 2);
    assert_eq!(OP_ADJUSTMENT, 3);
    // OP_SPATIAL (4) is the flatten placeholder for a spatial adjustment; the
    // segment loops + cs_flat treat it as a no-op (default arm), so the WGSL has
    // no `case 4u` — pin the Rust discriminant so the placeholder stays distinct.
    assert_eq!(OP_SPATIAL, 4);
}

/// Pin every numeric literal the GPU blend math shares with the Rust
/// source-of-truth (`ph2d_painter_brush::blend` + `ph2d_color::srgb`) to
/// zero-ULP agreement — the same discipline as
/// `shader_oklab_coefficients_bit_identical_with_rust`. Each literal must
/// (a) appear verbatim in the WGSL and (b) parse to the exact f32 bits of
/// the Rust constant it mirrors. Catches a drift like `0.59 → 0.587`
/// (textual) or a transfer-function typo (bit-level).
#[test]
fn shader_blend_modes_bit_identical_with_rust() {
    // (wgsl literal that must appear verbatim, expected f32 value)
    let pairs: &[(&str, f32)] = &[
        // sRGB ENCODE transfer (ph2d_color::srgb::linear_to_srgb_byte) —
        // the decode direction is the bit-exact LUT (gate below), so only
        // the encode literals live in the WGSL now.
        ("12.92", 12.92),
        ("0.055", 0.055),
        ("1.055", 1.055),
        ("2.4", 2.4),
        ("0.0031308", 0.003_130_8),
        // W3C HSL luminosity coefficients (blend::lum).
        ("0.3 * c.r", 0.3),
        ("0.59 * c.g", 0.59),
        ("0.11 * c.b", 0.11),
        // W3C soft-light cubic approximation (blend::soft_light).
        ("16.0 * cb", 16.0),
        ("12.0)", 12.0),
        // Exclusion / linear-burn / linear-light shared constants.
        ("2.0 * cb * cs", 2.0),
        // f32::EPSILON guard threshold (blend::apply ao <= EPSILON).
        ("1.1920929e-7", f32::EPSILON),
    ];
    for (lit, expected) in pairs {
        // Verbatim presence.
        assert!(
            LAYER_COMPOSITE_WGSL.contains(lit),
            "literal `{lit}` not found verbatim in layer_composite.wgsl"
        );
        // Bit-exact parse of the leading numeric token of the literal.
        let token: String = lit
            .chars()
            .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '-' | '+'))
            .collect();
        let parsed: f32 = token
            .parse()
            .unwrap_or_else(|e| panic!("WGSL literal token `{token}` failed f32 parse: {e:?}"));
        assert_eq!(
            parsed.to_bits(),
            expected.to_bits(),
            "blend literal drift: WGSL `{token}` is 0x{:08x}, Rust `{expected}` is 0x{:08x}",
            parsed.to_bits(),
            expected.to_bits(),
        );
    }
}

/// Pin the W4 adjustment literals (OKLab matrices + B/C pivot) bit-identical
/// to the Rust source they mirror (`ph2d_color::oklab` + `ph2d_painter_brush
/// ::adjustments`). Mirror of the blend-mode gate: a future edit that re-
/// introduces the full-precision OKLab spec coefficients (which drift the
/// GPU↔CPU adjustment parity past the ±4-byte bound — see the GPU gate
/// `gpu_adjustment_matches_cpu_reference_each_kind`) is caught here, on the
/// no-GPU CI lane.
#[test]
fn shader_adjustment_coefficients_bit_identical_with_rust() {
    let pairs: &[(&str, f32)] = &[
        // B/C perceptual mid-gray pivot (apply_brightness_contrast PIVOT).
        ("0.21404114", 0.214_041_14),
        // OKLab from_linear (ph2d_color::oklab::OklabColor::from_linear).
        ("0.41222147", 0.412_221_47),
        ("0.5363325", 0.536_332_5),
        ("0.2119035", 0.211_903_5),
        ("0.6299787", 0.629_978_7),
        ("0.21045426", 0.210_454_26),
        ("1.9779985", 1.977_998_5),
        ("0.80867577", 0.808_675_77),
        // OKLab to_linear (OklabColor::to_linear).
        ("0.39633778", 0.396_337_78),
        ("1.2914855", 1.291_485_5),
        ("4.0767417", 4.076_741_7),
        ("2.6097574", 2.609_757_4),
        ("1.7076147", 1.707_614_7),
    ];
    for (lit, expected) in pairs {
        assert!(
            LAYER_COMPOSITE_WGSL.contains(lit),
            "adjustment literal `{lit}` not found verbatim in layer_composite.wgsl"
        );
        let parsed: f32 = lit
            .parse()
            .unwrap_or_else(|e| panic!("adjustment literal `{lit}` failed f32 parse: {e:?}"));
        assert_eq!(
            parsed.to_bits(),
            expected.to_bits(),
            "adjustment literal drift: WGSL `{lit}` is 0x{:08x}, Rust is 0x{:08x}",
            parsed.to_bits(),
            expected.to_bits(),
        );
    }
}

/// The four HSL (non-separable) modes + the two compositing specials must
/// be dispatched by the right discriminants — guards a copy-paste that
/// silently routes a mode to the wrong arm.
#[test]
fn shader_dispatches_hsl_and_specials_by_canonical_discriminant() {
    // HSL group = 16..=19 (Hue/Saturation/Color/Luminosity).
    for code in ["case 16u", "case 17u", "case 18u", "case 19u"] {
        assert!(
            LAYER_COMPOSITE_WGSL.contains(code),
            "missing HSL arm {code}"
        );
    }
    // Specials: Behind=20, Clear=21 (handled before the blend switch).
    assert!(
        LAYER_COMPOSITE_WGSL.contains("if mode == 20u"),
        "missing Behind special"
    );
    assert!(
        LAYER_COMPOSITE_WGSL.contains("if mode == 21u"),
        "missing Clear special"
    );
    // is_hsl must match exactly the 16..=19 set.
    assert!(
        LAYER_COMPOSITE_WGSL.contains("mode == 16u || mode == 17u || mode == 18u || mode == 19u")
    );
}
