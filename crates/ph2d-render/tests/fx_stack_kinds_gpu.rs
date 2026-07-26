//! **O CATÁLOGO da pilha de FX raster** (plano 24 W3) — os degraus de DENTRO, o CONTORNO e a COR.
//!
//! Irmão do `fx_stack_gpu.rs` (que cobre o fold, a ordem e a costura de render) pelo teto de LOC, e
//! coeso por assunto: aqui prova-se o que **cada TIPO desenha**.
//!
//! O gate que carrega a wave é o [`every_kind_draws_something`]: ele varre a tabela do
//! `ph2d_ecs::FxOp` e exige que TODO tipo mude a imagem. É o antídoto do modo de falha desta wave —
//! um tipo entra na tabela (ganha nome, botão "Add", card no painel e defaults) e o shader não tem
//! braço para ele, então o artista clica e nada acontece, com toda a suíte verde.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_kinds_gpu -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture, stack_reach};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// A moldura das fixtures: uma forma BRANCA OPACA com margem folgada em toda volta — a premissa do
/// produto (`bbox + stack_reach`), sem a qual o kernel perde peso na borda da textura.
const W: u32 = 160;
const H: u32 = 80;
const BX0: u32 = 40;
const BX1: u32 = 96;
const BY0: u32 = 24;
const BY1: u32 = 56;

fn source(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in BY0..BY1 {
        for x in BX0..BX1 {
            let o = ((y * W + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    make_src(gpu, W, H, &bytes)
}

fn one(kind: u8, sigma_px: f32, tint: [f32; 4], offset_px: [i32; 2]) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px,
        offset_px,
        tint,
        opacity: 1.0,
    }
}

/// Roda uma pilha sobre a forma padrão e devolve os bytes RGBA do resultado.
fn render(gpu: &ph2d_gpu::GpuContext, pass: &mut FxStackPass, ops: &[FxOpGpu]) -> Vec<u8> {
    let src = source(gpu);
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, &src, &dst, W, H, ops);
    readback(gpu, &dst, W, H)
}

fn alpha_at(px: &[u8], x: u32, y: u32) -> i32 {
    i32::from(px[(((y * W + x) * 4) + 3) as usize])
}
fn rgb_at(px: &[u8], x: u32, y: u32) -> [i32; 3] {
    let o = ((y * W + x) * 4) as usize;
    [i32::from(px[o]), i32::from(px[o + 1]), i32::from(px[o + 2])]
}

/// **TODO tipo da tabela desenha alguma coisa.**
///
/// Varre `FxOp::KINDS` — não uma lista escrita à mão —, então um tipo novo entra neste gate no
/// mesmo commit em que entra na tabela. Sem ele, o modo de falha desta wave é mudo: o painel ganha
/// o botão, o card, os defaults e o `Add` chega ao bus, e o shader simplesmente cai no `else`.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn every_kind_draws_something() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    for kind in 0..FxOp::KINDS as u8 {
        // Um degrau com parâmetros VISÍVEIS: cor forte, raio de verdade, e deslocamento para quem
        // o usa (uma sombra sem offset seria um glow, mas ainda assim desenharia).
        let offset = if FxOp::spec(kind).has_offset {
            [6, -6]
        } else {
            [0, 0]
        };
        let out = render(&gpu, &mut pass, &[one(kind, 5.0, RED, offset)]);
        let differ = plain
            .chunks_exact(4)
            .zip(out.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        let total = (W * H) as usize;
        assert!(
            differ * 100 > total,
            "o tipo {} ({kind}) nao mudou a imagem ({differ} de {total} texels) — ele esta na \
             tabela e o shader nao tem braco para ele",
            FxOp::kind_name(kind)
        );
        eprintln!(
            "[kinds] {:<14} mudou {differ:>6} de {total} texels",
            FxOp::kind_name(kind)
        );
    }
}

/// **A sombra de DENTRO mora dentro: escurece a borda por dentro e não vaza um texel para fora.**
///
/// Duas metades, e a ausente é a que importa — sem a máscara pela cobertura da forma, o halo
/// invertido pinta o lado de FORA inteiro (que é onde `1 − a` vale 1), e o efeito vira um retângulo
/// escuro em volta da arte.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_inner_shadow_darkens_the_rim_and_never_leaks_outside() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let out = render(
        &gpu,
        &mut pass,
        &[one(FxOp::INNER_GLOW, 5.0, BLACK, [0, 0])],
    );
    let y = (BY0 + BY1) / 2;
    let rim = rgb_at(&out, BX0 + 1, y)[0];
    let core = rgb_at(&out, (BX0 + BX1) / 2, y)[0];
    eprintln!("[inner] rim {rim} · core {core}");
    assert!(
        rim + 60 < core,
        "a borda de dentro tem de ficar bem mais escura que o miolo (rim {rim}, core {core})"
    );
    assert!(core > 200, "o miolo nao pode escurecer (deu {core})");
    // E o lado de fora fica INTOCADO — alfa exatamente zero, byte a byte, nas quatro margens.
    for (x, y) in [
        (BX0 - 4, y),
        (BX1 + 4, y),
        ((BX0 + BX1) / 2, BY0 - 4),
        ((BX0 + BX1) / 2, BY1 + 4),
    ] {
        assert_eq!(
            alpha_at(&out, x, y),
            0,
            "o halo de DENTRO vazou para fora da forma em ({x},{y}) — falta a mascara pela cobertura"
        );
    }
    // A cobertura da forma não se move: um efeito de dentro não engorda nem come a silhueta.
    assert_eq!(alpha_at(&out, (BX0 + BX1) / 2, y), 255);
}

/// **O contorno alcança exatamente a LARGURA que o slider promete, e para com borda dura.**
///
/// É a afirmação que separa um Outline de um Glow com opacidade alta: o Glow desvanece por ~3σ e
/// nunca tem "largura"; o corte no nível `Φ(−1)` põe a borda a σ px da silhueta, com uma transição
/// de ~1 px. As duas metades num gate só, porque uma sem a outra é satisfeita pelo Glow.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_outline_reaches_its_width_and_stops_hard() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let y = (BY0 + BY1) / 2;
    // Perfil de alfa à DIREITA da fronteira da forma (que fica entre BX1-1 e BX1).
    let profile = |pass: &mut FxStackPass, kind: u8, sigma: f32| -> (f32, usize) {
        let out = render(&gpu, pass, &[one(kind, sigma, RED, [0, 0])]);
        let last = (BX1..W)
            .rfind(|x| alpha_at(&out, *x, y) > 128)
            .map_or(BX1 as f32 - 1.0, |x| x as f32);
        let band = (BX1..W)
            .filter(|x| {
                let a = alpha_at(&out, *x, y);
                a > 25 && a < 230
            })
            .count();
        (last - (BX1 as f32 - 0.5), band)
    };
    for sigma in [4.0f32, 8.0] {
        let (reach, band) = profile(&mut pass, FxOp::OUTLINE, sigma);
        let (glow_reach, glow_band) = profile(&mut pass, FxOp::GLOW, sigma);
        eprintln!(
            "[outline] sigma {sigma}: alcance {reach:.1} px (banda {band}) · glow {glow_reach:.1} \
             (banda {glow_band})"
        );
        assert!(
            (reach - sigma).abs() <= 1.5,
            "o contorno de largura {sigma} alcancou {reach} px"
        );
        assert!(
            band <= 3,
            "a borda do contorno tem de ser DURA (banda de {band} px)"
        );
        assert!(
            glow_band > band * 3,
            "o controle falhou: o glow do mesmo sigma tinha de desvanecer muito mais \
             (banda {glow_band} contra {band})"
        );
    }
}

/// **O Color Overlay repinta e NÃO move um texel de cobertura** — o alfa sai byte-idêntico ao que
/// entrou, inclusive na borda anti-aliased. É isso que o separa de um halo.
///
/// ⚠️ **O laço de FORÇA é o que faz este gate poder falhar.** A primeira versão testava só a
/// opacidade cheia — e ali `alfa × k` com `k = 1` **é** o alfa, então a mutação que escreve a força
/// no canal de cobertura era a identidade e o gate ficava verde sobre ela. O fenômeno só existe
/// numa força intermediária.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_colour_overlay_repaints_without_moving_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    let y = (BY0 + BY1) / 2;
    for strength in [1.0f32, 0.6, 0.25] {
        let mut op = one(FxOp::COLOR_OVERLAY, 0.0, RED, [0, 0]);
        op.opacity = strength;
        let out = render(&gpu, &mut pass, &[op]);
        let moved = (0..(W * H) as usize)
            .filter(|i| plain[i * 4 + 3] != out[i * 4 + 3])
            .count();
        assert_eq!(
            moved, 0,
            "{moved} texels mudaram de COBERTURA sob um Color Overlay de forca {strength}"
        );
        let c = rgb_at(&out, (BX0 + BX1) / 2, y);
        eprintln!("[overlay] forca {strength}: miolo {c:?}");
        // O verde/azul do branco cedem na proporção da força — é o `mix` que a força controla.
        let want = ((1.0 - strength) * 255.0).round() as i32;
        assert!(
            c[0] > 240 && (c[1] - want).abs() <= 6 && (c[2] - want).abs() <= 6,
            "a forca {strength} tinha de dar ~[255,{want},{want}] (deu {c:?})"
        );
    }
}

/// **A margem é um fato do TIPO, e ela é medida em três respostas diferentes** (sem GPU — é
/// aritmética pura, e por isso este roda em toda máquina).
///
/// Quem mora DENTRO não cresce nada; o Outline cresce a LARGURA dele (não o suporte do kernel, que
/// é 3×); o resto cresce o suporte. Pagar `3σ` de textura por um contorno de `σ` seria triplicar a
/// área por nada, e dar margem a um Inner Glow seria pagá-la inteira por nada.
#[test]
fn the_reach_is_a_fact_of_the_kind() {
    let sigma = 8.0;
    let reach = |kind: u8| stack_reach(&[one(kind, sigma, RED, [0, 0])]);
    for kind in [FxOp::INNER_SHADOW, FxOp::INNER_GLOW, FxOp::COLOR_OVERLAY] {
        assert_eq!(
            reach(kind),
            (0, 0, 0, 0),
            "{} desenha so dentro do que recebeu — margem seria textura paga a troco de nada",
            FxOp::kind_name(kind)
        );
    }
    assert_eq!(
        reach(FxOp::BLUR),
        (24, 24, 24, 24),
        "o blur espalha 3 sigma"
    );
    assert_eq!(
        reach(FxOp::OUTLINE),
        (9, 9, 9, 9),
        "o contorno espalha a LARGURA dele (mais 1 px de banda), nao o suporte do kernel"
    );
    // O deslocamento de uma sombra EXTERNA é assimétrico; o de uma INTERNA não conta (a máscara o
    // corta na borda, então ele não pode empurrar a imagem para lado nenhum).
    let outer = stack_reach(&[one(FxOp::DROP_SHADOW, sigma, BLACK, [10, -4])]);
    assert_eq!(outer, (24, 28, 34, 24), "a sombra externa inclina a margem");
    let inner = stack_reach(&[one(FxOp::INNER_SHADOW, sigma, BLACK, [10, -4])]);
    assert_eq!(
        inner,
        (0, 0, 0, 0),
        "a sombra de DENTRO nunca cresce a imagem"
    );
}

/// **O op pontual é MUITO mais barato que um borrão** — a consequência observável de custar UM
/// dispatch e não ler vizinho nenhum. Sem esta medida, `passes_of` podia devolver 2 para todo mundo
/// e nada além de um `eprintln` notaria.
///
/// Razão, não relógio: o número absoluto é da máquina, a razão é do desenho.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_pointwise_op_costs_much_less_than_a_blur() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // ⚠️ Textura GRANDE de propósito: na moldura das outras fixtures (160x80) o custo é quase todo
    // encoder + resolve, e a razão mediria a sobrecarga fixa em vez do trabalho por pixel.
    let (sw, sh) = (512u32, 512u32);
    let src = make_src(&gpu, sw, sh, &vec![255u8; (sw * sh * 4) as usize]);
    let dst = make_output_texture(&gpu, sw, sh);
    let time = |pass: &mut FxStackPass, ops: &[FxOpGpu]| -> f64 {
        for _ in 0..3 {
            pass.run(&gpu, &src, &dst, sw, sh, ops);
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let t = std::time::Instant::now();
        for _ in 0..20 {
            pass.run(&gpu, &src, &dst, sw, sh, ops);
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        t.elapsed().as_secs_f64() * 1000.0 / 20.0
    };
    let six = |kind: u8, sigma: f32| vec![one(kind, sigma, RED, [0, 0]); 6];
    let blur = time(&mut pass, &six(FxOp::BLUR, 8.0));
    let overlay = time(&mut pass, &six(FxOp::COLOR_OVERLAY, 0.0));
    eprintln!("[custo] 6 blurs {blur:.3} ms · 6 color overlays {overlay:.3} ms");
    assert!(
        overlay * 2.0 < blur,
        "seis ops pontuais ({overlay:.3} ms) tinham de custar bem menos que seis borroes \
         ({blur:.3} ms) — `passes_of` deve estar dando dois passes a quem nao borra"
    );
}
