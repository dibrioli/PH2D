//! **Nenhum kernel escreve fora da ponta** — a varredura por KERNEL da lei de [`crate::tip`].
//!
//! ⚠️ **Este arquivo existe porque o gate estava no lugar errado.** O corte *"fora do quadrado
//! unitário não há dab"* foi gateado em 2026-08-09 pelo produto (o carimbo de grade enche a célula e
//! nada mais), e um gate de produto só cobre a rota que o roteador ESCOLHE para aquela configuração —
//! mediu uma, e as outras quatro shiparam com o defeito. O Enio voltou com a mesma foto no mesmo dia.
//!
//! A lei não é do roteador, é do KERNEL: *um dab escreve nos pixels cujo CENTRO cai dentro da ponta, e
//! em nenhum outro*. Aqui ela é afirmada uma vez por kernel público que consome um stamp, sem passar
//! por pincel, rota ou método de traço — então um kernel novo entra nesta lista e o roteador não tem
//! voto sobre a cobertura.
//!
//! ⚠️ **O oráculo é a definição, não a aritmética do kernel:** a ponta de imagem é `|tex| ≤ 1` no
//! referencial DELA ([`crate::texture::shape::shape_value`]), com `tex = Rot(−α)·(p + 0.5 − c)/r · Size
//! + Offset` e `α` = o Angle da Shape + a rotação do dab. O pixel `p` está dentro sse o CENTRO dele está
//! dentro da ponta ([`in_tip`], escrita aqui por trigonometria, sem passar pelo amostrador).
//!
//! ⛔⛔ **Este oráculo já MENTIU uma vez (Bug #52):** ele dizia `|p + 0.5 − c| ≤ r` nos eixos da TELA e
//! chamava-lhe «a mesma frase» do `shape_value`. Não era — e a fixtura (0°, Size 1, sem Offset) era o
//! ÚNICO caso em que as duas coincidem, então o gate ficou verde sobre a caixa que cortava a ponta
//! rodada e a de Size < 1. *Uma fixtura alinhada aos eixos não mede uma base:* por isso
//! [`no_kernel_cuts_or_overflows_a_rotated_or_scaled_tip`] corre cada kernel rodado, com Size 0,6 e com
//! Offset.

use crate::spec::BrushSpec;
use crate::stamp_color::{
    ColorStampMask, DynDab, FusedDab, accumulate_color_stamp_coverage,
    accumulate_color_stamps_fused, accumulate_color_stamps_fused_batch,
    accumulate_shape_layers_rgba_batch, blit_color_stamp, render_color_stamp_mask,
};
use crate::texture::{ImageMask, TextureKind};

const W: u32 = 192;
const H: u32 = 192;
const CX: f32 = 96.0;
const CY: f32 = 96.0;
const R: f32 = 20.0;
const STAMP: u32 = 24;

/// A janela que a lei permite: os pixels cujo CENTRO cai dentro da ponta, INCLUSIVE.
fn lawful_box() -> (i64, i64, i64, i64) {
    let lo = |c: f32| (c - R - 0.5).ceil() as i64;
    let hi = |c: f32| (c + R - 0.5).floor() as i64;
    (lo(CX), hi(CX), lo(CY), hi(CY))
}

/// O retângulo INCLUSIVO dos texels que o kernel escreveu, lido de um buffer que nasceu a zero.
fn written_box(buf: &[u8], stride: usize) -> (i64, i64, i64, i64) {
    let (mut x0, mut x1, mut y0, mut y1) = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);
    for y in 0..i64::from(H) {
        for x in 0..i64::from(W) {
            if buf[(y as usize) * (W as usize) * stride + (x as usize) * stride..][..stride]
                .iter()
                .any(|&b| b != 0)
            {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    (x0, x1, y0, y1)
}

/// Um pincel com silhueta de Shape por IMAGEM — a única que tem aro OPACO, e por isso a única que
/// expõe o clamp. Com um falloff macio o aro é `0` e a fixture ficaria verde sobre o defeito.
fn opaque_spec() -> BrushSpec {
    let mut spec = BrushSpec::default();
    spec.shape.kind = TextureKind::Image;
    spec
}

fn opaque_layer() -> Vec<u8> {
    vec![255u8; (STAMP * STAMP) as usize]
}

fn opaque_color_stamp() -> ColorStampMask {
    let lum = opaque_layer();
    let layer = ImageMask {
        lum: &lum,
        width: STAMP,
        height: STAMP,
    };
    render_color_stamp_mask(
        &opaque_spec(),
        &[layer],
        &[[1.0, 1.0, 1.0]],
        &[None],
        &[1.0],
        None,
        None,
        STAMP,
    )
}

/// **Nenhum kernel escreve fora da ponta.** Um por rota do carimbo — o roteador não tem voto aqui.
///
/// **Mutação que tem de sangrar:** devolver texels fora do quadrado unitário em
/// [`crate::tip::axis_taps`], ou clampar a coordenada antes de a perguntar (foi assim que os quatro
/// amostradores viviam, e é o defeito que o Enio fotografou).
#[test]
fn no_kernel_writes_outside_the_tip() {
    let want = lawful_box();
    let mut bad: Vec<String> = Vec::new();
    let mut check = |name: &str, got: (i64, i64, i64, i64)| {
        if got != want {
            bad.push(format!("{name}: escreveu {got:?}"));
        }
    };
    let spec = opaque_spec();
    let stamp = opaque_color_stamp();
    let lum = opaque_layer();
    let layer = ImageMask {
        lum: &lum,
        width: STAMP,
        height: STAMP,
    };

    // (1) O carimbo em tons de cinza.
    let mask = crate::stamp::render_stamp_mask(&spec, None, Some(&layer), None, STAMP);
    let mut canvas = vec![0u8; (W * H * 4) as usize];
    let _ = crate::stamp::blit_stamp(
        &mut canvas,
        W,
        H,
        [CX, CY],
        R,
        &mask,
        &spec,
        1.0,
        false,
        None,
    );
    check("blit_stamp", written_box(&canvas, 4));

    // (2) O carimbo COLORIDO — a rota do report (Use Texture Colors).
    let mut canvas = vec![0u8; (W * H * 4) as usize];
    let _ = blit_color_stamp(
        &mut canvas,
        W,
        H,
        [CX, CY],
        R,
        &stamp,
        &spec,
        1.0,
        false,
        [0.0; 3],
    );
    check("blit_color_stamp", written_box(&canvas, 4));

    // (3) O acumulador de cobertura de UMA camada.
    let mut cov = vec![0u8; (W * H) as usize];
    let _ = accumulate_color_stamp_coverage(&mut cov, W, H, [CX, CY], R, &stamp, 1.0);
    check("accumulate_color_stamp_coverage", written_box(&cov, 1));

    // (4) O acumulador FUNDIDO (várias camadas, um dab).
    let mut covs = vec![vec![0u8; (W * H) as usize]];
    let _ = accumulate_color_stamps_fused(
        &mut covs,
        W,
        H,
        [CX, CY],
        R,
        std::slice::from_ref(&stamp),
        1.0,
    );
    check("accumulate_color_stamps_fused", written_box(&covs[0], 1));

    // (5) O acumulador fundido em LOTE.
    let mut covs = vec![vec![0u8; (W * H) as usize]];
    let _ = accumulate_color_stamps_fused_batch(
        &mut covs,
        W,
        H,
        &[FusedDab {
            center: [CX, CY],
            radius: R,
            coverage: 1.0,
        }],
        std::slice::from_ref(&stamp),
    );
    check(
        "accumulate_color_stamps_fused_batch",
        written_box(&covs[0], 1),
    );

    // (6) O acumulador RGBA em LOTE — a rota que o `stamp_dabs_cached_color_rgba` toma.
    let mut accs = vec![vec![0u8; (W * H * 4) as usize]];
    let _ = accumulate_shape_layers_rgba_batch(
        &mut accs,
        W,
        H,
        &[DynDab {
            center: [CX, CY],
            radius: R,
            coverage: 1.0,
            spec,
            shape_basis: crate::texture::shape_basis(
                &spec.shape,
                &mut 0u64,
                [1.0, 1.0],
                spec.footprint_deform(),
                crate::texture::ShapeFrame::Static,
            ),
            grain_basis: None,
            colors: vec![[1.0, 1.0, 1.0]],
            color_shift: [0.0; 3],
        }],
        &[layer],
        &[None],
        None,
        None,
    );
    check(
        "accumulate_shape_layers_rgba_batch",
        written_box(&accs[0], 4),
    );
    assert!(
        bad.is_empty(),
        "estes kernels escreveram fora da ponta (a janela legal e {want:?}): {}",
        bad.join(" | ")
    );
}

/// O pixel `(x, y)` está dentro da ponta de imagem — a DEFINIÇÃO, por trigonometria: a pegada roda por
/// `−φ` (o ângulo do dab + a rotação sorteada `dab_deg`) e estica o eixo menor por `1/(1 − Flatten)`;
/// depois a Shape roda por `−θ` (o Angle dela), escala por Size e desloca por Offset. Dentro sse `|tex| ≤ 1`.
fn in_tip(spec: &BrushSpec, dab_deg: f32, x: i64, y: i64) -> bool {
    let rel = [(x as f32 + 0.5 - CX) / R, (y as f32 + 0.5 - CY) / R];
    let (sf, cf) = (f32::from(spec.dab_angle_deg) + dab_deg)
        .to_radians()
        .sin_cos();
    let f = [
        rel[0] * cf + rel[1] * sf,
        (-rel[0] * sf + rel[1] * cf) / (1.0 - spec.dab_flatten),
    ];
    let (s, c) = f32::from(spec.shape.angle_deg).to_radians().sin_cos();
    let tex = [
        (f[0] * c + f[1] * s) * spec.shape.size[0] + spec.shape.offset[0],
        (-f[0] * s + f[1] * c) * spec.shape.size[1] + spec.shape.offset[1],
    ];
    tex[0].abs() <= 1.0 && tex[1].abs() <= 1.0
}

/// O retângulo INCLUSIVO dos pixels dentro da ponta.
fn tip_box(spec: &BrushSpec, dab_deg: f32) -> (i64, i64, i64, i64) {
    let (mut x0, mut x1, mut y0, mut y1) = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);
    for y in 0..i64::from(H) {
        for x in 0..i64::from(W) {
            if in_tip(spec, dab_deg, x, y) {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    (x0, x1, y0, y1)
}

/// `(kernel, exacto?, caixa escrita)`.
type Escrita = (&'static str, bool, (i64, i64, i64, i64));

/// O que cada kernel escreveu para `spec` (rotação do dab `dab_deg`), por nome. Exactos = os que
/// amostram a ponta por pixel; os outros lêem um carimbo assado de `STAMP_FINO` texels.
fn every_kernel(spec: &BrushSpec, dab_deg: f32) -> Vec<Escrita> {
    const STAMP_FINO: u32 = 160;
    // O kernel por-pixel lê o raio do PINCEL; os outros recebem-no — os dois têm de ser `R`.
    let spec = &BrushSpec {
        radius_px: R,
        ..*spec
    };
    let lum = vec![255u8; (STAMP_FINO * STAMP_FINO) as usize];
    let layer = ImageMask {
        lum: &lum,
        width: STAMP_FINO,
        height: STAMP_FINO,
    };
    let (s, c) = dab_deg.to_radians().sin_cos();
    let rotor = [c, s];
    let basis = crate::texture::shape_basis(
        &spec.shape,
        &mut 0u64,
        [1.0, 1.0],
        spec.dab_footprint(rotor),
        crate::texture::ShapeFrame::Static,
    );
    let n = (W * H) as usize;
    let mut out = Vec::new();

    // Os por-pixel: valem em qualquer rotação do dab.
    let mut canvas = vec![0u8; n * 4];
    let _ = crate::stamp_dab_textured_masked(
        &mut canvas,
        W,
        H,
        [CX, CY],
        spec,
        1.0,
        false,
        None,
        None,
        Some(crate::ShapeInput {
            basis: &basis,
            image: Some(&layer),
            ramp_lut: None,
        }),
        None,
        None,
        rotor,
        None,
    );
    out.push(("stamp_dab_textured_masked", true, written_box(&canvas, 4)));
    let mut accs = vec![vec![0u8; n * 4]];
    let _ = accumulate_shape_layers_rgba_batch(
        &mut accs,
        W,
        H,
        &[DynDab {
            center: [CX, CY],
            radius: R,
            coverage: 1.0,
            spec: *spec,
            shape_basis: basis,
            grain_basis: None,
            colors: vec![[1.0, 1.0, 1.0]],
            color_shift: [0.0; 3],
        }],
        &[layer],
        &[None],
        None,
        None,
    );
    out.push((
        "accumulate_shape_layers_rgba_batch",
        true,
        written_box(&accs[0], 4),
    ));
    if dab_deg != 0.0 {
        return out; // os assados e o canvas-cached são de orientação constante (o roteador sabe)
    }
    let mut canvas = vec![0u8; n * 4];
    let (mut tex, mut ready) = (vec![0u8; n], vec![0u8; n]);
    let _ = crate::blit_canvas_cached(
        &mut canvas,
        &mut tex,
        &mut ready,
        W,
        H,
        [CX, CY],
        R,
        spec,
        None,
        Some(&layer),
        None,
        1.0,
        false,
        None,
    );
    out.push(("blit_canvas_cached", true, written_box(&canvas, 4)));

    // Os assados: o domínio cresce com a ponta e quem amostra percorre `r · reach`.
    let mask = crate::stamp::render_stamp_mask(spec, None, Some(&layer), None, STAMP_FINO);
    let mut canvas = vec![0u8; n * 4];
    let _ = crate::stamp::blit_stamp(
        &mut canvas,
        W,
        H,
        [CX, CY],
        R,
        &mask,
        spec,
        1.0,
        false,
        None,
    );
    out.push(("blit_stamp", false, written_box(&canvas, 4)));
    let ramp = vec![[0.0, 0.0, 0.0, 1.0]; 256];
    let mut canvas = vec![0u8; n * 4];
    let _ = crate::blit_stamp_ramped(
        &mut canvas,
        W,
        H,
        [CX, CY],
        R,
        &mask,
        spec,
        1.0,
        false,
        &ramp,
        crate::RampAlphaMode::None,
    );
    out.push(("blit_stamp_ramped", false, written_box(&canvas, 4)));
    let stamp = render_color_stamp_mask(
        spec,
        &[layer],
        &[[1.0, 1.0, 1.0]],
        &[None],
        &[1.0],
        None,
        None,
        STAMP_FINO,
    );
    let mut canvas = vec![0u8; n * 4];
    let _ = blit_color_stamp(
        &mut canvas,
        W,
        H,
        [CX, CY],
        R,
        &stamp,
        spec,
        1.0,
        false,
        [0.0; 3],
    );
    out.push(("blit_color_stamp", false, written_box(&canvas, 4)));
    let mut cov = vec![0u8; n];
    let _ = accumulate_color_stamp_coverage(&mut cov, W, H, [CX, CY], R, &stamp, 1.0);
    out.push((
        "accumulate_color_stamp_coverage",
        false,
        written_box(&cov, 1),
    ));
    let mut covs = vec![vec![0u8; n]];
    let _ = accumulate_color_stamps_fused(
        &mut covs,
        W,
        H,
        [CX, CY],
        R,
        std::slice::from_ref(&stamp),
        1.0,
    );
    out.push((
        "accumulate_color_stamps_fused",
        false,
        written_box(&covs[0], 1),
    ));
    let dab = [FusedDab {
        center: [CX, CY],
        radius: R,
        coverage: 1.0,
    }];
    let mut covs = vec![vec![0u8; n]];
    let _ =
        accumulate_color_stamps_fused_batch(&mut covs, W, H, &dab, std::slice::from_ref(&stamp));
    out.push((
        "accumulate_color_stamps_fused_batch",
        false,
        written_box(&covs[0], 1),
    ));
    let mut accs = vec![vec![0u8; n * 4]];
    let _ = crate::accumulate_color_stamps_rgba_batch(
        &mut accs,
        W,
        H,
        &dab,
        std::slice::from_ref(&stamp),
    );
    out.push((
        "accumulate_color_stamps_rgba_batch",
        false,
        written_box(&accs[0], 4),
    ));
    out
}

/// ⭐⭐ **Bug #52, por KERNEL: a ponta rodada, de Size < 1 ou deslocada sai INTEIRA e não transborda.**
/// Os por-pixel escrevem exactamente a caixa da ponta; os assados, a menos de 1 px (a orla bilinear de um
/// texel de `≤ 0,6` px). Antes da cura todos paravam em `± r` — a Size 0,6 isso corta 13 px de cada lado.
///
/// **Mutação que tem de sangrar:** [`crate::tip::dab_extent`] devolver `base` (a caixa `± raio`), ou
/// [`crate::tip::stamp_domain`] devolver `1`.
#[test]
fn no_kernel_cuts_or_overflows_a_rotated_or_scaled_tip() {
    let caso = |angle: u16, size: f32, offset: [f32; 2]| {
        let mut s = opaque_spec();
        s.shape.angle_deg = angle;
        s.shape.size = [size, size];
        s.shape.offset = offset;
        s
    };
    let casos = [
        ("Angle 30°", caso(30, 1.0, [0.0, 0.0]), 0.0),
        ("Size 0,6", caso(0, 0.6, [0.0, 0.0]), 0.0),
        (
            "Offset + Size 0,8 + Angle 20°",
            caso(20, 0.8, [0.3, -0.2]),
            0.0,
        ),
        (
            "Jitter Rotate 45° (dab) + Size 0,7",
            caso(0, 0.7, [0.0, 0.0]),
            45.0,
        ),
        // ⚠️ Assimétrico DE PROPÓSITO: com a ponta centrada e redonda a rotação de `+φ` e a de `−φ` dão a
        // mesma caixa (o quadrado é simétrico), e a mutação do sinal em `unapply_linear` sobrevivia.
        (
            "Flatten 0,4 + Offset + Jitter 30°",
            {
                let mut s = caso(15, 0.8, [0.4, 0.1]);
                s.dab_flatten = 0.4;
                s
            },
            30.0,
        ),
    ];
    let mut bad: Vec<String> = Vec::new();
    for (nome, spec, dab_deg) in casos {
        let want = tip_box(&spec, dab_deg);
        for (kernel, exact, got) in every_kernel(&spec, dab_deg) {
            let tol = if exact { 0 } else { 1 };
            let off = [
                got.0 - want.0,
                got.1 - want.1,
                got.2 - want.2,
                got.3 - want.3,
            ];
            if off.iter().any(|d| d.abs() > tol) {
                bad.push(format!(
                    "{nome} · {kernel}: escreveu {got:?}, a ponta é {want:?}"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// **O custo do alcance (Bug #52), A/B no MESMO processo:** a rota dinâmica (`stamp_dab_textured_masked`)
/// com a caixa da ponta contra a caixa `± raio` de antes ([`crate::tip::CAIXA_DO_RAIO`]), intercaladas em
/// blocos rodados, resumo = o MÍNIMO. A área visitada só cresce onde a ponta cresce: a `0°`/Size 1 é a
/// mesma caixa. Correr: `cargo test -p ph2d-painter-brush --release --lib measure_o_custo -- --ignored
/// --nocapture`.
#[test]
#[ignore = "medição"]
fn measure_o_custo_do_alcance() {
    const N: u32 = 256;
    const RAIO: f32 = 40.0;
    const BLOCOS: usize = 7;
    const DABS: usize = 40;
    let lum = vec![255u8; 64 * 64];
    let layer = ImageMask {
        lum: &lum,
        width: 64,
        height: 64,
    };
    for (nome, angle, size, dab_deg) in [
        ("0°, Size 1", 0u16, 1.0f32, 0.0f32),
        ("Jitter 45°, Size 1", 0, 1.0, 45.0),
        ("0°, Size 0,6", 0, 0.6, 0.0),
    ] {
        let mut spec = opaque_spec();
        spec.radius_px = RAIO;
        spec.shape.angle_deg = angle;
        spec.shape.size = [size, size];
        let (s, c) = dab_deg.to_radians().sin_cos();
        let basis = crate::texture::shape_basis(
            &spec.shape,
            &mut 0u64,
            [1.0, 1.0],
            spec.dab_footprint([c, s]),
            crate::texture::ShapeFrame::Static,
        );
        let mut canvas = vec![0u8; (N * N * 4) as usize];
        let mut corre = |antes: bool| {
            crate::tip::CAIXA_DO_RAIO.with(|f| f.set(antes));
            let t0 = std::time::Instant::now();
            let mut area = 0u64;
            for _ in 0..DABS {
                if let Some(r) = crate::stamp_dab_textured_masked(
                    &mut canvas,
                    N,
                    N,
                    [128.0, 128.0],
                    &spec,
                    1.0,
                    false,
                    None,
                    None,
                    Some(crate::ShapeInput {
                        basis: &basis,
                        image: Some(&layer),
                        ramp_lut: None,
                    }),
                    None,
                    None,
                    [c, s],
                    None,
                ) {
                    area = u64::from(r.w) * u64::from(r.h);
                }
            }
            crate::tip::CAIXA_DO_RAIO.with(|f| f.set(false));
            (t0.elapsed().as_secs_f64() * 1e6 / DABS as f64, area)
        };
        let (mut cura, mut antes) = ((f64::MAX, 0u64), (f64::MAX, 0u64));
        for b in 0..BLOCOS {
            for v in [b % 2 == 0, b % 2 != 0] {
                let (us, a) = corre(v);
                let alvo = if v { &mut antes } else { &mut cura };
                *alvo = (alvo.0.min(us), a);
            }
        }
        println!(
            "{nome}: antes {:.1} µs/dab ({} px) · cura {:.1} µs/dab ({} px) · {:.2}×",
            antes.0,
            antes.1,
            cura.0,
            cura.1,
            cura.0 / antes.0
        );
    }
}
