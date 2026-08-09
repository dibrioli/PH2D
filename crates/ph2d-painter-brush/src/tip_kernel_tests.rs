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
//! ⚠️ **O oráculo é a definição, não a aritmética do kernel:** o pixel `p` está dentro sse
//! `|p + 0.5 − c| <= r`. É a mesma frase que a rota por-pixel sempre disse
//! ([`crate::texture::shape::shape_value`] devolve `0` para `|tex| > 1`) — e o código antigo a violava,
//! escrevendo uma coluna a mais à direita e uma linha a mais abaixo (o retângulo do blit é
//! assimétrico: `floor(c − r)` de um lado, `ceil(c + r) + 1` do outro).

use crate::spec::BrushSpec;
use crate::stamp_color::{
    ColorStampMask, DynDab, FusedDab, accumulate_color_stamp_coverage,
    accumulate_color_stamps_fused, accumulate_color_stamps_fused_batch,
    accumulate_shape_layers_rgba_batch, blit_color_stamp, render_color_stamp_mask,
};
use crate::texture::{ImageMask, TextureKind};

const W: u32 = 96;
const H: u32 = 96;
const CX: f32 = 40.0;
const CY: f32 = 40.0;
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
    let _ = crate::stamp::blit_stamp(&mut canvas, W, H, [CX, CY], R, &mask, &spec, 1.0, false);
    check("blit_stamp", written_box(&canvas, 4));

    // (2) O carimbo COLORIDO — a rota do report (Use Texture Colors).
    let mut canvas = vec![0u8; (W * H * 4) as usize];
    let _ = blit_color_stamp(&mut canvas, W, H, [CX, CY], R, &stamp, &spec, 1.0, false);
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
