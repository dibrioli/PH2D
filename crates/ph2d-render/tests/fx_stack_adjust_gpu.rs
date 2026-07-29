//! **COLOR ADJUST, no dispositivo** (plano 24 W8) — matiz, saturação e brilho.
//!
//! Arquivo próprio pelo mesmo motivo dos irmãos: assunto coeso, e os outros estão perto do teto de
//! LOC.
//!
//! # O oráculo NÃO é uma tolerância: é a OUTRA implementação, que já estava no repo
//!
//! O gate que carrega a wave é o [`the_adjust_is_the_law_the_painter_already_ships`]. A lei deste
//! degrau não é nova — é a do `AdjustmentKind::HueSaturationBrightness` que a camada de ajuste do
//! Painter ship há waves, e cujo kernel WGSL passou a morar num arquivo COMPARTILHADO quando
//! ganhou este segundo consumidor. Então o oráculo certo não é *"o resultado parece razoável"*: é
//! **a implementação de CPU daquela crate**, escrita por outra wave, para outro consumidor, sem
//! saber que esta existe. Se as duas discordarem, uma delas está errada — e o artista veria a
//! mesma ficha desenhar duas coisas.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_adjust_gpu --release -- --ignored`.

use ph2d_color::LinearRgba;
use ph2d_color::oklab::OklabColor;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use ph2d_ecs::FxOp;
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, HsbParams, apply_adjustment,
};
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

/// A paleta da fixture — uma coluna por cor, cada uma com croma e luminância diferentes, mais o
/// BRANCO e o CINZA (os acromáticos, que são os pontos fixos da matiz e da saturação).
const PALETTE: [[u8; 3]; 9] = [
    [220, 40, 40],   // vermelho
    [240, 150, 30],  // âmbar
    [230, 220, 60],  // amarelo
    [60, 180, 90],   // verde
    [40, 140, 220],  // azul
    [140, 60, 200],  // roxo
    [170, 128, 105], // terracota apagado — croma BAIXO, o que fica no gamut em toda volta
    [255, 255, 255], // branco
    [128, 128, 128], // cinza
];

/// As `0..VIVID` são as cromáticas VIVAS; a [`MUTED`] é a de croma baixo; as duas últimas são os
/// acromáticos. Índices nomeados porque três gates diferentes fatiam a mesma paleta, e um `0..6`
/// solto apodrece na primeira cor que entrar no meio.
const VIVID: usize = 6;
/// A cor de croma BAIXO — a única cuja rotação fica no gamut em TODA volta (medido).
const MUTED: usize = 6;

const CELL: u32 = 8;
const W: u32 = CELL * PALETTE.len() as u32;
const H: u32 = 16;

/// O limite de paridade GPU↔CPU, em níveis de byte. **É o mesmo do irmão do compositor**
/// (`gpu_adjustment_matches_cpu_reference_each_kind`), e a razão é a mesma: a raiz cúbica do OKLab
/// é `pow(x, 1/3)` no dispositivo e `powf`/`cbrt` no Rust, e o ida-e-volta atravessa duas
/// transferências sRGB.
const MAX_DELTA: i32 = 4;

/// A cobertura da metade de BAIXO da fixture. ⚠️ **Ela existe por um modo de falha concreto:**
/// o scratch é PREMULTIPLICADO, e ajustar `rgb` sem dividir pelo alfa trataria um texel de meia
/// cobertura como uma cor mais ESCURA — a rampa de anti-aliasing giraria de matiz em relação ao
/// miolo. Com toda a fixture opaca, essa troca é indetectável.
const SEMI: u8 = 128;

fn source(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for (i, rgb) in PALETTE.iter().enumerate() {
        for y in 0..H {
            for x in 0..CELL {
                let o = ((y * W + i as u32 * CELL + x) * 4) as usize;
                bytes[o..o + 3].copy_from_slice(rgb);
                bytes[o + 3] = if y < H / 2 { 255 } else { SEMI };
            }
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// Um degrau de ajuste de cor — os três knobs, e nada mais.
fn adjust(hue: f32, sat: f32, bright: f32) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::COLOR_ADJUST,
        sigma_px: 0.0,
        offset_px: [0, 0],
        tint: [0.0, 0.0, 0.0, 1.0],
        tint_b: [1.0; 4],
        opacity: 1.0,
        mode: 0,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
        hue,
        sat,
        bright,
        stops: [[0.0; 4]; 8],
        stop_pos: [[0.0; 4]; 2],
        stop_count: 0,
    }
}

fn render(gpu: &ph2d_gpu::GpuContext, pass: &mut FxStackPass, ops: &[FxOpGpu]) -> Vec<u8> {
    let src = source(gpu);
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, &src, &dst, W, H, ops, &[]);
    readback(gpu, &dst, W, H)
}

/// O centro da célula `i`, na metade OPACA.
fn cell(px: &[u8], i: usize) -> [u8; 4] {
    sample(px, i, H / 4)
}

/// O centro da célula `i`, na metade de meia COBERTURA.
fn cell_semi(px: &[u8], i: usize) -> [u8; 4] {
    sample(px, i, H * 3 / 4)
}

fn sample(px: &[u8], i: usize, y: u32) -> [u8; 4] {
    let x = i as u32 * CELL + CELL / 2;
    let o = ((y * W + x) * 4) as usize;
    [px[o], px[o + 1], px[o + 2], px[o + 3]]
}

/// A resposta da CPU do PAINTER para a mesma cor e os mesmos knobs — byte de entrada a byte de
/// saída, atravessando as mesmas duas transferências que o dispositivo atravessa.
fn painter_cpu(rgb: [u8; 3], h: f32, s: f32, b: f32) -> [u8; 3] {
    let mut acc = [[
        srgb_to_linear_byte(rgb[0]),
        srgb_to_linear_byte(rgb[1]),
        srgb_to_linear_byte(rgb[2]),
        1.0,
    ]];
    apply_adjustment(
        &AdjustmentKind::HueSaturationBrightness,
        &AdjustmentParams::HueSaturationBrightness(HsbParams { h, s, b }),
        &mut acc,
    );
    [
        linear_to_srgb_byte(acc[0][0]),
        linear_to_srgb_byte(acc[0][1]),
        linear_to_srgb_byte(acc[0][2]),
    ]
}

/// **O gate da wave: o degrau desenha o que a camada de ajuste do Painter desenha.**
///
/// ⚠️ A força deste oráculo é que ele não foi escrito para esta wave: `apply_hsb` existe desde a
/// W4 do Painter, num outro crate, para o compositor de camadas. Uma divergência aqui significa
/// que o app passou a ter DUAS respostas para *"o que o slider de matiz faz?"*.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_adjust_is_the_law_the_painter_already_ships() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // Uma varredura pelos três eixos, e uma combinação — um eixo de cada vez isola a culpa.
    let knobs: [(f32, f32, f32); 5] = [
        (0.25, 0.0, 0.0),
        (-0.4, 0.0, 0.0),
        (0.0, -0.6, 0.0),
        (0.0, 0.5, 0.0),
        (0.12, 0.3, -0.2),
    ];
    let mut worst = 0i32;
    for (h, s, b) in knobs {
        let out = render(&gpu, &mut pass, &[adjust(h, s, b)]);
        for (i, rgb) in PALETTE.iter().enumerate() {
            let got = cell(&out, i);
            let want = painter_cpu(*rgb, h, s, b);
            for c in 0..3 {
                let d = i32::from(got[c]) - i32::from(want[c]);
                worst = worst.max(d.abs());
                assert!(
                    d.abs() <= MAX_DELTA,
                    "h={h} s={s} b={b}, cor {rgb:?}: a GPU deu {got:?} e o Painter dá {want:?} \
                     (canal {c}, delta {d}) — as duas metades do app discordam sobre a MESMA lei"
                );
            }
        }
    }
    eprintln!("[fx_adjust] pior delta GPU vs Painter-CPU: {worst} nivel(is) de byte");
}

/// **O neutro é a identidade AO BIT** — o que o artista tem de poder confiar ao atravessar o zero
/// a arrastar.
///
/// ⚠️ **E o que o produz NÃO é o early-out da lei, medido.** Eu escrevi que era, e a mutação que o
/// remove passou por este gate: a sonda irmã mostra **0 de 4096 bytes** de diferença numa rampa
/// sRGB completa sem o ramo. O erro do ida-e-volta OKLab em `f32` fica sob meio nível e a
/// quantização o come; o ramo é exactidão no FLOAT (que compõe numa pilha longa) e custo. Este
/// gate afirma o OBSERVÁVEL, que é o que importa ao artista — e continua a morrer se a lei
/// deixar de ser neutra no zero por qualquer outra via.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_neutral_adjust_is_byte_identical_to_no_adjust_at_all() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let neutral = render(&gpu, &mut pass, &[adjust(0.0, 0.0, 0.0)]);
    let differ = plain
        .chunks_exact(4)
        .zip(neutral.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differ, 0,
        "o ponto neutro do ajuste moveu {differ} texels — ele tem de ser a identidade AO BIT"
    );
}

/// **A matiz GIRA a cor, e não a drena** — é isto que separa uma rotação de croma de um *tint*: o
/// vermelho deixa de ser vermelho e continua uma cor, não um cinza avermelhado.
///
/// ⚠️ O oráculo é medido em **OKLab**, o espaço em que a lei é definida. Em RGB, *"quanto croma
/// sobrou"* é uma pergunta sem resposta estável.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_hue_turns_the_colour_without_draining_it() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let turned = render(&gpu, &mut pass, &[adjust(0.25, 0.0, 0.0)]);
    // As seis primeiras são as cromáticas; as duas últimas são acromáticas de propósito.
    for (i, rgb) in PALETTE.iter().enumerate().take(VIVID) {
        let (a, b) = (cell(&plain, i), cell(&turned, i));
        let (ca, cb) = (chroma(a), chroma(b));
        assert!(
            cb / ca > 0.6,
            "cor {i} {rgb:?}: o croma caiu de {ca:.4} para {cb:.4} — a matiz drenou a cor em vez \
             de a girar"
        );
        let moved = (0..3)
            .map(|c| (i32::from(a[c]) - i32::from(b[c])).abs())
            .max()
            .unwrap_or(0);
        assert!(
            moved > 20,
            "cor {i}: a matiz de um quarto de volta mal moveu a cor ({moved} niveis)"
        );
    }
}

/// **A rotação é RÍGIDA — e o que a estraga é o GAMUT do alvo, não a lei.**
///
/// ⚠️ **Duas vezes a medição me corrigiu neste gate, e a segunda foi a que ensina.** Escrevi
/// primeiro *"o croma é preservado"* para todas as cores: falso, o vermelho da paleta cai a
/// **0,641** num quarto de volta. Reescrevi para *"nas duas que ficam no gamut"*: também falso —
/// eu tinha medido UM ângulo, e o âmbar cai a **0,817** a 3/8 de volta enquanto o azul cai a
/// **0,736** a −1/8. **Estar no gamut é propriedade do par (cor, ângulo), não da cor.**
///
/// Uma rotação rígida em OKLab leva a cor para fora do sRGB, e a viagem de volta a 8 bits corta.
/// A fixture que CONTÉM o fenômeno é então uma cor de croma baixo, que cabe em qualquer direção —
/// medido no giro inteiro, a razão fica em **0,989..1,010**.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_hue_rotation_is_rigid_where_the_result_still_fits_in_the_gamut() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let before = chroma(cell(&plain, MUTED));
    for turn in [0.125f32, 0.25, 0.375, 0.5, -0.125, -0.25, -0.375] {
        let turned = render(&gpu, &mut pass, &[adjust(turn, 0.0, 0.0)]);
        let after = chroma(cell(&turned, MUTED));
        let ratio = after / before;
        assert!(
            (0.97..=1.03).contains(&ratio),
            "{:?} a {turn} volta(s): croma {before:.4} -> {after:.4} (razao {ratio:.3}) — a \
             rotação deixou de ser rígida numa cor que CABE no gamut em toda direção",
            PALETTE[MUTED]
        );
    }
}

/// **A saturação drena até o cinza e dobra o croma** — os dois extremos do knob, medidos.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_saturation_drains_to_grey_and_doubles_the_chroma() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let grey = render(&gpu, &mut pass, &[adjust(0.0, -1.0, 0.0)]);
    let rich = render(&gpu, &mut pass, &[adjust(0.0, 1.0, 0.0)]);
    for i in 0..VIVID {
        let c0 = chroma(cell(&plain, i));
        assert!(
            chroma(cell(&grey, i)) < 0.01,
            "cor {i}: saturação -1 deixou croma {:.4} — tinha de sair CINZA",
            chroma(cell(&grey, i))
        );
        let c2 = chroma(cell(&rich, i));
        assert!(
            c2 > c0,
            "cor {i}: saturação +1 não aumentou o croma ({c0:.4} -> {c2:.4})"
        );
    }
}

/// **O brilho alcança preto e branco EXACTOS nas pontas** — o lerp em luz linear, e a razão de ele
/// não ser um ganho multiplicativo dos dois lados.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_brightness_reaches_exact_black_and_exact_white() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let dark = render(&gpu, &mut pass, &[adjust(0.0, 0.0, -1.0)]);
    let light = render(&gpu, &mut pass, &[adjust(0.0, 0.0, 1.0)]);
    for (i, _) in PALETTE.iter().enumerate() {
        assert_eq!(
            &cell(&dark, i)[..3],
            &[0, 0, 0],
            "cor {i}: brilho -1 não chegou ao preto exacto"
        );
        assert_eq!(
            &cell(&light, i)[..3],
            &[255, 255, 255],
            "cor {i}: brilho +1 não chegou ao branco exacto"
        );
    }
}

/// **Um pixel ACROMÁTICO é imune à matiz e à saturação, e isso é a lei, não um defeito.** Girar a
/// matiz de um pixel sem croma É nada.
///
/// ⚠️ Este gate existe porque a premissa mordeu: a varredura do catálogo tem fixture BRANCA, e o
/// trio `(matiz, saturação, +brilho)` mediu **0 de 12800 texels** — `+brilho` num branco é o
/// próprio branco. Quem for mexer nos knobs desta família tem de saber onde estão os pontos fixos.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn an_achromatic_pixel_is_untouched_by_hue_and_saturation() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let spun = render(&gpu, &mut pass, &[adjust(0.37, 0.8, 0.0)]);
    // As duas últimas células são o branco e o cinza.
    for i in PALETTE.len() - 2..PALETTE.len() {
        let (a, b) = (cell(&plain, i), cell(&spun, i));
        let worst = (0..3)
            .map(|c| (i32::from(a[c]) - i32::from(b[c])).abs())
            .max()
            .unwrap_or(0);
        assert!(
            worst <= 1,
            "a célula acromática {i} andou {worst} níveis sob matiz+saturação: {a:?} -> {b:?}"
        );
    }
}

/// **O ajuste não move um texel de COBERTURA** — é o que o separa de um halo, e é o mesmo contrato
/// que o Color Overlay ao lado honra.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_adjust_never_moves_the_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let out = render(&gpu, &mut pass, &[adjust(0.3, -0.5, 0.4)]);
    for (i, (a, b)) in plain.chunks_exact(4).zip(out.chunks_exact(4)).enumerate() {
        assert_eq!(
            a[3], b[3],
            "o alfa do texel {i} mudou: {} -> {}",
            a[3], b[3]
        );
    }
}

/// O croma OKLab de um pixel sRGB — a grandeza em que a lei está escrita.
fn chroma(px: [u8; 4]) -> f32 {
    let lin = [
        srgb_to_linear_byte(px[0]),
        srgb_to_linear_byte(px[1]),
        srgb_to_linear_byte(px[2]),
    ];
    // ⚠️ A conversão vem do `ph2d-color` — a MESMA que o kernel espelha bit a bit. Reescrevê-la
    // aqui faria o gate medir a minha aritmética contra a minha aritmética.
    let lab = OklabColor::from_linear(LinearRgba::new(lin[0], lin[1], lin[2], 1.0));
    (lab.a * lab.a + lab.b * lab.b).sqrt()
}

/// **O ajuste lê a cor que o texel DE FACTO tem, não a premultiplicada.**
///
/// ⚠️ O scratch da pilha é premultiplicado, então `src.rgb` de um texel a meia cobertura é a cor
/// pela METADE. Ajustá-la assim rodaria a matiz de uma cor mais escura — e como o OKLab não é
/// linear na luminosidade, o resultado difere. O sintoma seria a orla de anti-aliasing a puxar
/// para outra cor que o miolo, que é onde estes efeitos inteiros vivem.
///
/// O oráculo é o mesmo do gate da wave: o Painter, sobre a cor RETA.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_adjust_reads_the_straight_colour_not_the_premultiplied_one() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_adjust] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let (h, s, b) = (0.2_f32, 0.35_f32, 0.0_f32);
    let out = render(&gpu, &mut pass, &[adjust(h, s, b)]);
    for (i, rgb) in PALETTE.iter().enumerate() {
        let got = cell_semi(&out, i);
        assert_eq!(
            got[3], SEMI,
            "a célula {i} perdeu a cobertura de meia altura"
        );
        let want = painter_cpu(*rgb, h, s, b);
        for c in 0..3 {
            let d = i32::from(got[c]) - i32::from(want[c]);
            assert!(
                d.abs() <= MAX_DELTA,
                "cor {rgb:?} a meia cobertura: a GPU deu {got:?} e a cor RETA devia dar {want:?} \
                 (canal {c}, delta {d}) — o ajuste está a ler o premultiplicado"
            );
        }
    }
}

/// **SONDA:** quantos bytes o ida-e-volta OKLab move num degrau NEUTRO, sobre uma rampa sRGB
/// COMPLETA — o número que corrigiu o doc-comment da lei (0 de 4096, com e sem o early-out).
#[test]
#[ignore = "sonda"]
fn probe_the_neutral_round_trip_over_a_full_ramp() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // 256 colunas de 1 px: a rampa inteira em cada canal, mais um cinza.
    let (w, h) = (256u32, 4u32);
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for x in 0..w {
        for y in 0..h {
            let o = ((y * w + x) * 4) as usize;
            let v = u8::try_from(x).unwrap_or(255);
            bytes[o] = v;
            bytes[o + 1] = v.wrapping_mul(3);
            bytes[o + 2] = 255 - v;
            bytes[o + 3] = 255;
        }
    }
    let mut go = |ops: &[FxOpGpu]| {
        let src = make_src(&gpu, w, h, &bytes);
        let dst = make_output_texture(&gpu, w, h);
        pass.run(&gpu, &src, &dst, w, h, ops, &[]);
        readback(&gpu, &dst, w, h)
    };
    let plain = go(&[]);
    let neutral = go(&[adjust(0.0, 0.0, 0.0)]);
    let differ = plain
        .iter()
        .zip(neutral.iter())
        .filter(|(a, b)| a != b)
        .count();
    let worst = plain
        .iter()
        .zip(neutral.iter())
        .map(|(a, b)| i32::from(*a) - i32::from(*b))
        .map(i32::abs)
        .max()
        .unwrap_or(0);
    eprintln!("[probe] rampa completa: {differ} bytes diferem, pior delta {worst}");
}
