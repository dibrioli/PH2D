//! **DUOTONE e LUMA TO ALPHA, no dispositivo** (plano 24 W9) — a rampa de duas pontas, e o brilho
//! que vira cobertura.
//!
//! Arquivo próprio pelo mesmo motivo dos irmãos: assunto coeso, e os outros estão perto do teto de
//! LOC. Os dois tipos moram juntos porque respondem à MESMA pergunta sobre a arte (*quão claro é
//! este texel?*) e mandam a resposta para lugares diferentes — um para a COR, o outro para a
//! COBERTURA.
//!
//! # As duas fixtures, e por que a chapa não serve
//!
//! Toda fixture aqui é um **DEGRADÊ**, e não a chapa de cor sólida dos outros arquivos: a lei destes
//! dois é função da luminância da PRÓPRIA arte, então uma chapa é um único ponto do domínio — e no
//! branco puro o Luma to Alpha é literalmente a identidade. Uma fixture que não contém o fenômeno
//! deixa passar exactamente o defeito que o gate existe para pegar.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_duotone_gpu --release -- --ignored`.

use ph2d_color::LinearRgba;
use ph2d_color::oklab::OklabColor;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

/// A moldura: uma barra com margem, e o conteúdo é uma RAMPA de cinza da esquerda para a direita.
const W: u32 = 96;
const H: u32 = 16;
const X0: u32 = 16;
const X1: u32 = 80;
const Y0: u32 = 4;
const Y1: u32 = 12;
/// Quantas colunas de arte a rampa tem.
const SPAN: u32 = X1 - X0;

/// O limite de paridade GPU↔CPU, em níveis de byte. **É o mesmo do irmão do Color Adjust**, e pela
/// mesma razão: a raiz cúbica do OKLab é `pow(x, 1/3)` no dispositivo e `cbrt` no Rust, e o
/// ida-e-volta atravessa duas transferências sRGB.
const MAX_DELTA: i32 = 4;

/// As duas pontas da rampa autorada — o par frio→quente, em sRGB reto.
const SHADOW: [f32; 4] = [0.10, 0.12, 0.35, 1.0];
const HIGHLIGHT: [f32; 4] = [1.0, 0.86, 0.62, 1.0];

/// O valor sRGB da coluna `x` da rampa (preto na esquerda, branco na direita).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ramp_byte(x: u32) -> u8 {
    (255.0 * (x - X0) as f32 / (SPAN - 1) as f32) as u8
}

/// A fixture padrão: rampa OPACA.
fn source(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    make_src(gpu, W, H, &ramp_bytes(255))
}

fn ramp_bytes(alpha: u8) -> Vec<u8> {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in Y0..Y1 {
        for x in X0..X1 {
            let o = ((y * W + x) * 4) as usize;
            let v = ramp_byte(x);
            bytes[o..o + 4].copy_from_slice(&[v, v, v, alpha]);
        }
    }
    bytes
}

/// **A fixture da BORDA ANTI-ALIASED**: cor CONSTANTE, alfa em rampa.
///
/// ⚠️ É a única fixture que separa a nossa lei da do SVG. A matriz do `feColorMatrix` escreve
/// `A' = luma(cor RETA)` ignorando o alfa que estava lá — e a cor reta de uma orla é a MESMA do
/// miolo, então sob aquela lei a rampa de cobertura vira um DEGRAU. Numa fixture opaca as duas leis
/// são indistinguíveis.
fn source_soft_edge(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in Y0..Y1 {
        for x in X0..X1 {
            let o = ((y * W + x) * 4) as usize;
            // Um cinza médio fixo — luminância bem longe de 0 e de 1, para o escalonamento se ver.
            bytes[o..o + 4].copy_from_slice(&[160, 160, 160, ramp_byte(x)]);
        }
    }
    make_src(gpu, W, H, &bytes)
}

fn duotone(shadow: [f32; 4], highlight: [f32; 4], opacity: f32) -> FxOpGpu {
    step(FxOp::DUOTONE, shadow, highlight, opacity)
}

fn luma_to_alpha(opacity: f32) -> FxOpGpu {
    step(FxOp::LUMA_TO_ALPHA, [0.0; 4], [1.0; 4], opacity)
}

fn step(kind: u8, tint: [f32; 4], tint_b: [f32; 4], opacity: f32) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px: 0.0,
        offset_px: [0, 0],
        tint,
        tint_b,
        opacity,
        mode: 0,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
        stops: [[0.0; 4]; 8],
        stop_pos: [[0.0; 4]; 2],
        stop_count: 0,
    }
}

fn render_on(
    gpu: &ph2d_gpu::GpuContext,
    pass: &mut FxStackPass,
    src: &wgpu::Texture,
    ops: &[FxOpGpu],
) -> Vec<u8> {
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, src, &dst, W, H, ops, &[]);
    readback(gpu, &dst, W, H)
}

fn px(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let o = ((y * W + x) * 4) as usize;
    [bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]
}

/// A linha do meio da arte.
const MID: u32 = (Y0 + Y1) / 2;

// ── O ORÁCULO, em CPU ─────────────────────────────────────────────────────────────────────────

/// **A luminância que a rampa usa** — o `L` do OKLab, calculado pela OUTRA implementação (a do
/// `ph2d-color`, escrita para outro consumidor).
fn oklab_l(srgb: [u8; 3]) -> f32 {
    let lin = LinearRgba::new(
        srgb_to_linear_byte(srgb[0]),
        srgb_to_linear_byte(srgb[1]),
        srgb_to_linear_byte(srgb[2]),
        1.0,
    );
    OklabColor::from_linear(lin).l
}

/// O duotone em CPU: a lei escrita como prosa, não como port do shader.
fn duotone_cpu(srgb: [u8; 3], shadow: [f32; 4], highlight: [f32; 4]) -> [u8; 3] {
    let t = oklab_l(srgb).clamp(0.0, 1.0);
    let mut out = [0u8; 3];
    for (i, o) in out.iter_mut().enumerate() {
        // A mistura acontece em LUZ LINEAR, que é o espaço de trabalho da pilha.
        let a = srgb_to_linear_byte((shadow[i] * 255.0).round() as u8);
        let b = srgb_to_linear_byte((highlight[i] * 255.0).round() as u8);
        *o = linear_to_srgb_byte(a + (b - a) * t);
    }
    out
}

// ── DUOTONE ───────────────────────────────────────────────────────────────────────────────────

/// **A rampa é a lei: a luminância de cada texel escolhe um ponto entre as duas pontas.**
///
/// O oráculo é uma implementação de CPU independente sobre a conversão OKLab do `ph2d-color` — a
/// mesma disciplina do irmão do Color Adjust, e a razão é a mesma: *"parece razoável"* não é
/// oráculo para uma lei que tem forma fechada.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_duotone_maps_luminance_onto_the_ramp() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let out = render_on(&gpu, &mut pass, &src, &[duotone(SHADOW, HIGHLIGHT, 1.0)]);

    let mut worst = 0i32;
    for x in X0..X1 {
        let v = ramp_byte(x);
        let want = duotone_cpu([v, v, v], SHADOW, HIGHLIGHT);
        let got = px(&out, x, MID);
        for i in 0..3 {
            worst = worst.max((i32::from(got[i]) - i32::from(want[i])).abs());
        }
        assert!(
            (0..3).all(|i| (i32::from(got[i]) - i32::from(want[i])).abs() <= MAX_DELTA),
            "coluna {x} (cinza {v}): GPU {got:?} contra a lei em CPU {want:?}"
        );
    }
    eprintln!("[duotone] pior delta contra o oraculo de CPU: {worst} nivel(is)");
}

/// **As DUAS pontas são exactas nos extremos** — preto puro dá a cor de sombra, branco puro dá a de
/// luz.
///
/// É a propriedade que faz as duas swatches significarem o que o rótulo diz. Ela não é acidente da
/// implementação: os coeficientes do `L` do OKLab **somam 1**, então acromático preto vale 0 e
/// branco vale 1, exactamente.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_duotone_endpoints_are_exactly_the_two_swatches() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let out = render_on(&gpu, &mut pass, &src, &[duotone(SHADOW, HIGHLIGHT, 1.0)]);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let want = |c: [f32; 4]| {
        [
            (c[0] * 255.0).round() as u8,
            (c[1] * 255.0).round() as u8,
            (c[2] * 255.0).round() as u8,
        ]
    };
    let dark = px(&out, X0, MID);
    let light = px(&out, X1 - 1, MID);
    let (ws, wh) = (want(SHADOW), want(HIGHLIGHT));
    for i in 0..3 {
        assert!(
            (i32::from(dark[i]) - i32::from(ws[i])).abs() <= MAX_DELTA,
            "a ponta ESCURA nao e a swatch de sombra: {dark:?} contra {ws:?}"
        );
        assert!(
            (i32::from(light[i]) - i32::from(wh[i])).abs() <= MAX_DELTA,
            "a ponta CLARA nao e a swatch de luz: {light:?} contra {wh:?}"
        );
    }
    eprintln!("[duotone] pontas: escura {dark:?} clara {light:?}");
}

/// **O Duotone PRESERVA a modelagem; o Color Overlay a destrói — e é por isso que são dois tipos.**
///
/// Sobre a MESMA rampa: a saída do Duotone continua a variar da esquerda para a direita (o volume
/// sobrevive, só a paleta muda), a do Color Overlay é uma chapa. Sem este gate, *"já temos o Color
/// Overlay"* seria uma objeção que nenhum número contradiz.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_duotone_keeps_the_modelling_that_a_colour_overlay_flattens() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);

    let duo = render_on(&gpu, &mut pass, &src, &[duotone(SHADOW, HIGHLIGHT, 1.0)]);
    let flat = render_on(
        &gpu,
        &mut pass,
        &src,
        &[step(FxOp::COLOR_OVERLAY, HIGHLIGHT, [1.0; 4], 1.0)],
    );

    let spread = |b: &[u8]| {
        let (mut lo, mut hi) = (255i32, 0i32);
        for x in X0..X1 {
            let g = i32::from(px(b, x, MID)[1]);
            lo = lo.min(g);
            hi = hi.max(g);
        }
        hi - lo
    };
    let (d, f) = (spread(&duo), spread(&flat));
    assert!(
        d > 100,
        "o Duotone achatou a modelagem: excursao de {d} niveis no verde"
    );
    assert!(
        f <= 1,
        "o Color Overlay deixou de ser uma chapa: excursao de {f} niveis"
    );
    eprintln!("[duotone] excursao no verde: duotone {d} · color overlay {f}");
}

/// **O alfa de CADA ponta é a força DELA — as duas swatches têm quatro canais, não três.**
///
/// Com a ponta clara em alfa ZERO, o extremo claro da rampa fica INTACTO (não há nada a aplicar
/// ali) enquanto o extremo escuro recebe a cor de sombra cheia. Sem este gate o canal alfa das
/// swatches seria um knob morto — o picker o oferece, e nada o leria.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn each_ramp_end_carries_its_own_strength() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let faded = [HIGHLIGHT[0], HIGHLIGHT[1], HIGHLIGHT[2], 0.0];
    let out = render_on(&gpu, &mut pass, &src, &[duotone(SHADOW, faded, 1.0)]);

    let light = px(&out, X1 - 1, MID);
    assert!(
        (0..3).all(|i| i32::from(light[i]) >= 254),
        "a ponta clara em alfa ZERO tinha de deixar o branco INTACTO, e saiu {light:?}"
    );
    let dark = px(&out, X0, MID);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ws = [
        (SHADOW[0] * 255.0).round() as u8,
        (SHADOW[1] * 255.0).round() as u8,
        (SHADOW[2] * 255.0).round() as u8,
    ];
    assert!(
        (0..3).all(|i| (i32::from(dark[i]) - i32::from(ws[i])).abs() <= MAX_DELTA),
        "a ponta escura, opaca, tinha de aplicar-se cheia: {dark:?} contra {ws:?}"
    );
    eprintln!("[duotone] alfa por-ponta: escura {dark:?} · clara {light:?}");
}

/// **O Duotone não move um texel de cobertura** — ele é pontual, como o Color Overlay ao lado.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_duotone_never_moves_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source_soft_edge(&gpu);
    let plain = render_on(&gpu, &mut pass, &src, &[]);
    let out = render_on(&gpu, &mut pass, &src, &[duotone(SHADOW, HIGHLIGHT, 1.0)]);
    for x in 0..W {
        assert_eq!(
            px(&out, x, MID)[3],
            px(&plain, x, MID)[3],
            "o Duotone mexeu no alfa da coluna {x}"
        );
    }
}

// ── LUMA TO ALPHA ─────────────────────────────────────────────────────────────────────────────

/// **O brilho vira cobertura, e a lei é `A' = A · luma`.**
///
/// Numa rampa opaca o alfa de saída TEM de seguir a luminância: zero no preto, cheio no branco, e
/// monotónico entre eles.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_luma_to_alpha_turns_brightness_into_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let out = render_on(&gpu, &mut pass, &src, &[luma_to_alpha(1.0)]);

    let mut worst = 0i32;
    let mut prev = -1i32;
    for x in X0..X1 {
        let v = ramp_byte(x);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let want = (oklab_l([v, v, v]).clamp(0.0, 1.0) * 255.0 + 0.5) as i32;
        let got = i32::from(px(&out, x, MID)[3]);
        worst = worst.max((got - want).abs());
        assert!(
            (got - want).abs() <= MAX_DELTA,
            "coluna {x} (cinza {v}): alfa {got} contra a luminancia {want}"
        );
        assert!(got >= prev, "o alfa nao e monotonico na coluna {x}");
        prev = got;
    }
    // ⚠️ **E a COR RETA tem de sair intacta.** O scratch é premultiplicado, então escalar só o
    // alfa (e não o vetor inteiro) deixaria `rgb` alto demais para a cobertura nova — o `resolve`
    // divide pelo alfa e a arte sairia LAVADA, com o alfa perfeitamente certo. É o gate que separa
    // `src * t` de `vec4(src.rgb, src.a * t)`.
    for x in X0..X1 {
        if i32::from(px(&out, x, MID)[3]) < 8 {
            continue; // onde a cobertura quase sumiu, a cor reta é ruído de quantização.
        }
        let v = i32::from(ramp_byte(x));
        let got = i32::from(px(&out, x, MID)[0]);
        assert!(
            (got - v).abs() <= MAX_DELTA,
            "coluna {x}: a cor RETA mudou ({got} contra {v}) — o rgb nao acompanhou o alfa"
        );
    }
    assert_eq!(px(&out, X0, MID)[3], 0, "o preto puro tinha de sumir");
    assert_eq!(
        px(&out, X1 - 1, MID)[3],
        255,
        "o branco puro tinha de ficar opaco"
    );
    eprintln!("[luma2a] pior delta contra a luminancia: {worst} nivel(is)");
}

/// **A borda ANTI-ALIASED sobrevive — e é aqui que a nossa lei diverge do SVG, de propósito.**
///
/// Fixture: cor CONSTANTE, alfa em rampa. Sob a matriz literal do `feColorMatrix`
/// (`A' = luma(cor reta)`, ignorando o alfa) toda a orla teria o MESMO alfa — a rampa viraria um
/// degrau, e uma silhueta suave ganharia serrilha. Sob `A' = A · luma` a rampa continua rampa.
///
/// ⚠️ Este gate é o único que distingue as duas leis; toda fixture opaca as vê idênticas.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_luma_to_alpha_preserves_the_antialiased_edge() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source_soft_edge(&gpu);
    let out = render_on(&gpu, &mut pass, &src, &[luma_to_alpha(1.0)]);

    // O alfa tem de crescer com a rampa de entrada, e a razão tem de ser CONSTANTE (= a luminância
    // do cinza fixo), que é exactamente o que `A·luma` significa e o que `A' = luma` destrói.
    let l = oklab_l([160, 160, 160]);
    let mut steps = 0;
    for x in X0 + 1..X1 {
        let a_in = f32::from(ramp_byte(x));
        let got = f32::from(px(&out, x, MID)[3]);
        assert!(
            (got - a_in * l).abs() <= 3.0,
            "coluna {x}: alfa {got} contra {} (entrada {a_in} × luma {l:.4})",
            a_in * l
        );
        if px(&out, x, MID)[3] != px(&out, x - 1, MID)[3] {
            steps += 1;
        }
    }
    assert!(
        steps > 40,
        "a orla virou um DEGRAU: so {steps} mudancas de nivel ao longo da rampa"
    );
    eprintln!("[luma2a] a orla tem {steps} degraus de nivel — continua rampa");
}

/// **O Luma to Alpha nunca CRIA cobertura** — é o que mantém a margem em zero.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_luma_to_alpha_never_creates_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source_soft_edge(&gpu);
    let plain = render_on(&gpu, &mut pass, &src, &[]);
    let out = render_on(&gpu, &mut pass, &src, &[luma_to_alpha(1.0)]);
    for y in 0..H {
        for x in 0..W {
            assert!(
                px(&out, x, y)[3] <= px(&plain, x, y)[3],
                "a cobertura CRESCEU em ({x},{y})"
            );
        }
    }
    assert_eq!(
        ph2d_render::stack_reach(&[luma_to_alpha(1.0)]),
        (0, 0, 0, 0),
        "um tipo que nunca cria cobertura nao pede margem"
    );
}

/// **Encadear recupera o SVG, e o contrário é impossível — é este o argumento da divergência.**
///
/// `Luma to Alpha` → `Color Adjust (Brightness −1)` dá o matte PRETO exacto que a matriz do SVG
/// produz. Nenhuma ordem de degraus devolve a cor que a lei do SVG teria apagado, e é por isso que
/// a lei que GUARDA informação é a que compõe.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn chaining_a_brightness_of_minus_one_gives_the_svg_black_matte() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_duotone] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let mut dark = step(FxOp::COLOR_ADJUST, [0.0; 4], [1.0; 4], 1.0);
    dark.bright = -1.0;
    let out = render_on(&gpu, &mut pass, &src, &[luma_to_alpha(1.0), dark]);

    for x in X0..X1 {
        let p = px(&out, x, MID);
        assert!(
            p[0] <= 1 && p[1] <= 1 && p[2] <= 1,
            "coluna {x}: o matte nao ficou preto ({p:?})"
        );
        let v = ramp_byte(x);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let want = (oklab_l([v, v, v]).clamp(0.0, 1.0) * 255.0 + 0.5) as i32;
        assert!(
            (i32::from(p[3]) - want).abs() <= MAX_DELTA,
            "coluna {x}: o matte perdeu a luminancia no alfa ({} contra {want})",
            p[3]
        );
    }
}

/// **A régua da rampa é o `L` do OKLab, e não o `lum()` das leis de mistura — com o número.**
///
/// Sonda, não gate: ela imprime os dois candidatos para o meio-tom, que é onde eles mais discordam.
/// O `lum` do `blend_modes.wgsl` existe e é a resposta CERTA para os modos `Color`/`Luminosity` do
/// W3C; ele é uma luminância de luz LINEAR, então o meio-tom sRGB cairia a um quinto do caminho da
/// rampa e a arte inteira se empilharia na ponta escura.
#[test]
fn measure_the_two_candidate_rulers_for_the_ramp() {
    let mid = [128u8, 128, 128];
    let lin = srgb_to_linear_byte(mid[0]);
    let w3c = 0.30 * lin + 0.59 * lin + 0.11 * lin;
    let ok = oklab_l(mid);
    eprintln!("[regua] cinza sRGB 128 -> linear {lin:.4} · lum(W3C) {w3c:.4} · L(OKLab) {ok:.4}");
    assert!(
        (ok - 0.600).abs() < 0.01 && (w3c - 0.216).abs() < 0.01,
        "os numeros do doc-comment do shader mudaram: lum {w3c:.4} · L {ok:.4}"
    );
}
