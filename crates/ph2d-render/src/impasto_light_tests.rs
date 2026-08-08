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

/// **O `Globals` do WGSL MEDE exatamente o `Globals` do Rust** — e a ordem dos campos é a mesma.
///
/// ⚠️ **Este gate nasceu de um PANIC**, e o mecanismo vale mais que a asserção. A W10.7 deu ao
/// uniform um bit novo (`has_form_occ`) ocupando a vaga que era `pad1`; o Rust trocou o nome da
/// vaga, o WGSL **acrescentou o campo e deixou o `pad1` para trás**, e o alinhamento de 16 bytes do
/// `Lamp` arredondou o struct de lá para **240 bytes contra os 224 daqui**. O wgpu recusa isso no
/// dispatch — como **panic**, não como erro devolvido —, então TODO documento com relevo na rota de
/// GPU, o bake da escultura e a re-acendida de um objeto assado morriam no primeiro quadro.
///
/// ⚠️ **Nenhum gate de unidade via, e os seis que veriam são `#[ignore]`** (`tests/impasto_light_gpu.rs`,
/// que precisa de adapter). É por isso que este mora aqui e **sem device**: uma incompatibilidade de
/// ABI entre duas declarações do mesmo buffer é aritmética, não é uma pergunta para a placa de vídeo.
///
/// ⚠️ **A lista de nomes é a TERCEIRA cópia da ordem dos campos, de propósito.** Ela não é
/// redundância: é o que torna a terceira edição **obrigatória** em vez de silenciosa — um campo novo
/// que só entre em dois dos três lugares reprova aqui, dizendo qual falta.
#[test]
fn the_wgsl_globals_measures_exactly_the_rust_globals() {
    /// A ordem dos campos do [`Globals`] do Rust. Ver o ⚠️ acima.
    const RUST_ORDER: [&str; 9] = [
        "lamps",
        "n",
        "ox",
        "oy",
        "rw",
        "rh",
        "has_form",
        "has_form_occ",
        "pad2",
    ];
    /// Onde o uniform mora — o mesmo par que o `bind_group` do [`ImpastoLightPass::run`] escreve.
    const UNIFORM: naga::ResourceBinding = naga::ResourceBinding {
        group: 0,
        binding: 7,
    };

    let module = naga::front::wgsl::parse_str(IMPASTO_LIGHT_WGSL).expect("parse");
    let var = module
        .global_variables
        .iter()
        .map(|(_, v)| v)
        .find(|v| v.binding == Some(UNIFORM))
        // **Controle positivo**: sem ele, mover o uniform para outro binding faria este gate passar
        // por vácuo — verde sobre um shader que ninguém conferiu.
        .expect("o shader declara um uniform em @group(0) @binding(7)");
    let naga::TypeInner::Struct { members, span } = &module.types[var.ty].inner else {
        panic!("o uniform do binding 7 tem de ser um struct");
    };

    let names: Vec<&str> = members
        .iter()
        .map(|m| m.name.as_deref().unwrap_or("<sem nome>"))
        .collect();
    assert_eq!(
        names, RUST_ORDER,
        "os campos do `Globals` do WGSL sairam da ordem do `Globals` do Rust"
    );
    assert_eq!(
        *span as usize,
        std::mem::size_of::<Globals>(),
        "o `Globals` do WGSL mede {span} bytes e o do Rust mede {} — o wgpu recusa o dispatch, \
         como PANIC, em TODA rota que acende relevo na GPU",
        std::mem::size_of::<Globals>()
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
    //
    // ⚠️ `AMBIENT` é a exceção, e ela é DERIVADA da constante de propósito. Uma string escrita à mão
    // pega o shader driftando e é CEGA à outra direção — o número em Rust mudar e o shader ficar
    // parado. Isso era teórico enquanto havia um dono e um shader; com o rig morando em `ph2d-light` e
    // a malha do módulo 3D acendendo pela MESMA lei, são três lugares, e o piso ambiente é justamente
    // o que os dois consumidores têm de dobrar igual.
    let ambient_decl = format!("const AMBIENT: f32 = {};", ph2d_light::AMBIENT);
    for (decl, why) in [
        (
            "const DEPTH_UNIT_PX: f32 = 16.0;",
            "the height-to-pixel gain (impasto_light::DEPTH_UNIT_PX)",
        ),
        (
            ambient_decl.as_str(),
            "the diffuse floor (ph2d_light::AMBIENT)",
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
        IMPASTO_MAX_LIGHTS,
        ph2d_light::MAX_LIGHTS,
        "the uniform's lamp array and the rig's own MAX_LIGHTS are the same number"
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
        // O mundo sem escultura — e a doação é opcional exatamente para que ele continue existindo.
        form: None,
        form_occlusion: None,
    };
    assert_eq!(base().check(), Ok(()), "a well-formed request passes");

    // ⚠️ E o plano de FORMA é conferido pela mesma porta: um plano curto chegaria ao `write_texture`,
    // onde a falha é um erro de driver em vez de uma recusa com nome.
    let short_form = vec![0f32; 4 * 4 - 1];
    let mut bad_form = base();
    bad_form.form = Some(&short_form);
    assert_eq!(
        bad_form.check(),
        Err(ImpastoLightError::PlaneSize),
        "a short form plane is refused, not silently read past"
    );
    let good_form = vec![0f32; 4 * 4];
    let mut ok_form = base();
    ok_form.form = Some(&good_form);
    assert_eq!(
        ok_form.check(),
        Ok(()),
        "quatro floats por texel é a forma certa"
    );

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
