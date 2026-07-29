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

/// **A fixture da VARREDURA** — a mesma forma, mas em DEGRADÊ (preto à esquerda, branco à direita)
/// em vez da chapa branca das outras.
///
/// ⚠️ **Uma chapa não pode conter o fenômeno de um tipo cuja lei é função da PRÓPRIA arte.** O Luma
/// to Alpha mapeia brilho → cobertura, e sobre branco puro a luminância é 1: ele é a IDENTIDADE
/// ali, então a varredura o reportaria como *"não desenha nada"* sobre um produto perfeitamente
/// correto — exactamente a falha que ela existe para produzir, com o sinal trocado. O Duotone tem o
/// mesmo problema pelo outro lado (branco cai na ponta clara da rampa e mais nada varia).
///
/// Trocar a chapa por um degradê teria mudado o que TODOS os outros gates deste arquivo medem, e
/// os comentários deles estão calibrados no branco; a varredura ganha a fixture DELA.
fn source_ramp(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in BY0..BY1 {
        for x in BX0..BX1 {
            let o = ((y * W + x) * 4) as usize;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let v = (255.0 * f32::from(u8::try_from(x - BX0).unwrap_or(255))
                / f32::from(u8::try_from(BX1 - BX0 - 1).unwrap_or(1))) as u8;
            bytes[o..o + 4].copy_from_slice(&[v, v, v, 255]);
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
        // ⚠️ **A mesma pergunta à TABELA que o `grow_px` e o `hue` fazem mais abaixo.** Deixar a
        // segunda ponta no branco neutro enquanto a primeira é forte é meia rampa autorada — e no
        // extremo claro da fixture o Duotone cairia exactamente nela, ou seja no ponto onde a lei
        // devolve a entrada.
        tint_b: if FxOp::spec(kind).color_b_label.is_some() {
            [0.0, 0.35, 1.0, 1.0]
        } else {
            [1.0; 4]
        },
        opacity: 1.0,
        // O modo default do TIPO — é o que o produto arma, e é o que o gate tem de medir.
        mode: if FxOp::spec(kind).modes.is_empty() {
            0
        } else {
            FxOp::new(kind).mode
        },
        blend: 0,
        // Um campo de ruído que se veja, para quem lê ruído (nos outros o shader nem olha).
        noise_scale_px: 24.0,
        detail: 3,
        seed: 0,
        // ⚠️ **O knob VISÍVEL de um tipo é resposta da TABELA, não do nome do parâmetro.** Este
        // construtor dava `sigma_px` a todo mundo, e o Grow / Shrink não tem raio nenhum: o
        // comprimento dele é o `grow`, então ele entrava na varredura no ponto NEUTRO e "não
        // desenhava nada". A pergunta é a mesma que a linha do `offset` já faz.
        grow_px: if FxOp::spec(kind).grow_label.is_some() {
            sigma_px
        } else {
            0.0
        },
        // ⚠️ **A mesma pergunta à TABELA que a linha do `grow_px` acima, e pelo mesmo motivo:** o
        // Color Adjust não tem raio nenhum, então dar-lhe `sigma_px` o deixaria no ponto NEUTRO
        // (onde a lei devolve a entrada AO BIT, de propósito) e ele entraria na varredura "sem
        // desenhar nada".
        //
        // ⚠️ **E o BRILHO vai para BAIXO, por uma lição MEDIDA.** Um ajuste pontual tem pontos
        // FIXOS por construção — matiz e saturação não movem um pixel sem croma, e `+brilho` é
        // `out + (1−out)·b`, que em BRANCO é exactamente branco. Quando a varredura corria sobre a
        // chapa branca eu escrevi aqui que o brilho *"move um pixel de qualquer cor"* e ela mediu
        // **0 de 12800**. Hoje a fixture da varredura é um degradê (`source_ramp`), então os dois
        // sinais moveriam a maior parte dela; para baixo continua sendo a escolha certa, porque só
        // o preto puro é ponto fixo e ele é UMA coluna da rampa.
        hue: if FxOp::spec(kind).adjust_labels.is_some() {
            0.25
        } else {
            0.0
        },
        sat: if FxOp::spec(kind).adjust_labels.is_some() {
            0.6
        } else {
            0.0
        },
        bright: if FxOp::spec(kind).adjust_labels.is_some() {
            -0.25
        } else {
            0.0
        },
    }
}

/// Roda uma pilha sobre a forma padrão e devolve os bytes RGBA do resultado.
fn render(gpu: &ph2d_gpu::GpuContext, pass: &mut FxStackPass, ops: &[FxOpGpu]) -> Vec<u8> {
    let src = source(gpu);
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, &src, &dst, W, H, ops, &[]);
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
    // A fixture da varredura é o DEGRADÊ — ver `source_ramp`.
    let render_ramp = |pass: &mut FxStackPass, ops: &[FxOpGpu]| {
        let src = source_ramp(&gpu);
        let dst = make_output_texture(&gpu, W, H);
        pass.run(&gpu, &src, &dst, W, H, ops, &[]);
        readback(&gpu, &dst, W, H)
    };
    let plain = render_ramp(&mut pass, &[]);
    for kind in 0..FxOp::KINDS as u8 {
        // Um degrau com parâmetros VISÍVEIS: cor forte, raio de verdade, e deslocamento para quem
        // o usa (uma sombra sem offset seria um glow, mas ainda assim desenharia).
        let offset = if FxOp::spec(kind).offset_labels.is_some() {
            [6, -6]
        } else {
            [0, 0]
        };
        let out = render_ramp(&mut pass, &[one(kind, 5.0, RED, offset)]);
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

/// **Um degrau de DENTRO nunca move a COBERTURA — nenhum texel, nem na borda anti-aliased.**
///
/// Foi o gate que faltava, e o defeito que ele pega foi reportado na tela: um **rim claro de 1 px**
/// em volta da forma. O halo era composto como uma CAMADA por cima (`halo + over*(1−halo.a)`), o
/// que SOMA alfa — na borda com `over.a = 0,5` e `halo.a = 0,25` o resultado era 0,625 — e como o
/// `resolve` des-premultiplica, dividir por um alfa maior CLAREIA. O gate antigo só olhava o miolo
/// (255) e o lado de fora (0): os dois estão certos mesmo com o bug, porque o fenômeno vive
/// exatamente na fatia de alfa fracionário.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn an_inner_op_never_moves_the_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // Uma borda com alfa FRACIONÁRIO em rampa — é onde o bug morava, e a fixture tem de o conter.
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in BY0..BY1 {
        for x in BX0..BX1 {
            #[allow(clippy::cast_possible_truncation)]
            let a = ((x - BX0) * 255 / (BX1 - BX0 - 1)).min(255) as u8;
            let o = ((y * W + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[a, a, a, a]);
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let dst = make_output_texture(&gpu, W, H);
    let run = |pass: &mut FxStackPass, ops: &[FxOpGpu]| {
        pass.run(&gpu, &src, &dst, W, H, ops, &[]);
        readback(&gpu, &dst, W, H)
    };
    let plain = run(&mut pass, &[]);
    for kind in [FxOp::INNER_SHADOW, FxOp::INNER_GLOW] {
        for strength in [1.0f32, 0.5] {
            let mut o = one(kind, 5.0, BLACK, [4, -4]);
            o.opacity = strength;
            let out = run(&mut pass, &[o]);
            let moved: Vec<usize> = (0..(W * H) as usize)
                .filter(|i| plain[i * 4 + 3] != out[i * 4 + 3])
                .collect();
            assert!(
                moved.is_empty(),
                "{} (forca {strength}) moveu a cobertura de {} texels — o primeiro em ({}, {}): {} -> {}",
                FxOp::kind_name(kind),
                moved.len(),
                moved[0] as u32 % W,
                moved[0] as u32 / W,
                plain[moved[0] * 4 + 3],
                out[moved[0] * 4 + 3]
            );
        }
    }
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
        // ⚠️ Meio pixel de tolerância, e é a convenção da sonda (a ÚLTIMA coluna acima do limiar),
        // não folga: com a dilatação sobre o campo de distância a borda cai em `w` exatamente, e é
        // isso que faz o slider "Width" prometer o que entrega. Um bar de ±1,5 px não distinguia a
        // correção de meio texel do JFA — a mutação que a remove passava por ele.
        assert!(
            (reach - (sigma - 0.5)).abs() <= 0.25,
            "o contorno de largura {sigma} alcancou {reach} px (esperado {})",
            sigma - 0.5
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
        //
        // ⚠️ **E a proporção é de LUZ, não de bytes.** A pilha mistura em espaço linear, então a
        // expectativa é `encode(1 − força)`: a 0,6 isso dá 170, não 102. O gate aprendeu a lei
        // certa em vez de afrouxar uma tolerância — e com isso passou a ser uma segunda testemunha
        // da convenção linear (o número antigo era o de um `mix` sobre bytes codificados).
        let want = i32::from(ph2d_color::srgb::linear_to_srgb_byte(1.0 - strength));
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
            pass.run(&gpu, &src, &dst, sw, sh, ops, &[]);
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let t = std::time::Instant::now();
        for _ in 0..20 {
            pass.run(&gpu, &src, &dst, sw, sh, ops, &[]);
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        t.elapsed().as_secs_f64() * 1000.0 / 20.0
    };
    let six = |kind: u8, sigma: f32| vec![one(kind, sigma, RED, [0, 0]); 6];
    // ⚠️ **O custo MARGINAL, e a subtração é o que torna o gate honesto.** Toda pilha paga um
    // `ingest` e um `resolve` fixos, então uma razão bruta dilui a diferença que se quer medir — e
    // dilui MAIS quando o overhead cresce, o que é exatamente o que aconteceu quando o ingest
    // nasceu. O que este gate afirma é sobre o `plan_of`: quem não borra gasta UM dispatch, não
    // dois. Medir só os ops isola essa afirmação da moldura.
    let base = time(&mut pass, &[]);
    let blur = time(&mut pass, &six(FxOp::BLUR, 8.0)) - base;
    let overlay = time(&mut pass, &six(FxOp::COLOR_OVERLAY, 0.0)) - base;
    eprintln!(
        "[custo] moldura {base:.3} ms · 6 blurs +{blur:.3} ms · 6 color overlays +{overlay:.3} ms"
    );
    assert!(
        overlay * 2.0 < blur,
        "seis ops pontuais (+{overlay:.3} ms) tinham de custar bem menos que seis borroes \
         (+{blur:.3} ms) — `passes_of` deve estar dando dois passes a quem nao borra"
    );
}

/// **Opacidade 0 é no-op em TODO tipo** — byte a byte, contra a pilha vazia.
///
/// Varre a tabela, então um tipo novo entra aqui sozinho. ⚠️ O Blur **falhava**: ele fazia
/// `borrado × opacidade`, então opacidade 0 não era *este efeito não contribui* e sim **a forma
/// desaparece**. Os outros seis já eram no-op por construção — foi a varredura que separou os dois
/// casos, e é por isso que ela varre em vez de escolher um tipo.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn an_op_at_zero_opacity_is_a_no_op_for_every_kind() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let plain = render(&gpu, &mut pass, &[]);
    for kind in 0..FxOp::KINDS as u8 {
        let mut o = one(kind, 6.0, RED, [5, -5]);
        o.opacity = 0.0;
        let out = render(&gpu, &mut pass, &[o]);
        let differ = plain
            .chunks_exact(4)
            .zip(out.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            differ,
            0,
            "o tipo {} mudou {differ} texels com opacidade ZERO — um degrau desligado pelo knob tem de ser o mesmo que nao estar la",
            FxOp::kind_name(kind)
        );
    }
}

/// **O modo CONTOUR põe sombra na REENTRÂNCIA; o PROXIMITY quase não põe** — e essa é a diferença
/// inteira entre os dois, reportada na tela (*"a estrela tem sombra só nas pontas"*).
///
/// A proximidade mede *quanto de FORA há por perto*: numa reentrância o fora subtende um ângulo
/// pequeno, então o número é pequeno **mesmo encostado na borda**. A distância à borda não tem
/// ângulo nenhum: é 0 em todo ponto do contorno.
///
/// A fixture é uma CRUZ — quatro quinas reentrantes e quatro pontas —, e as duas sondas ficam à
/// MESMA distância da borda (≈3 px): sem isso o gate compararia distâncias diferentes e não a lei.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_contour_mode_shadows_a_reentrant_corner_and_proximity_barely_does() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let (cw, ch) = (128u32, 128u32);
    let (v0, v1, h0, h1) = (44u32, 84u32, 44u32, 84u32);
    let mut bytes = vec![0u8; (cw * ch * 4) as usize];
    for y in 0..ch {
        for x in 0..cw {
            let horizontal = (14..114).contains(&x) && (h0..h1).contains(&y);
            let vertical = (v0..v1).contains(&x) && (14..114).contains(&y);
            if horizontal || vertical {
                let o = ((y * cw + x) * 4) as usize;
                bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
    let src = make_src(&gpu, cw, ch, &bytes);
    let dst = make_output_texture(&gpu, cw, ch);
    let mut pass = FxStackPass::new(&gpu);
    // Sem deslocamento: a pergunta é sobre a LEI da banda, não sobre a direção da luz.
    let probe = |pass: &mut FxStackPass, mode: u8| -> (i32, i32) {
        let mut o = one(FxOp::INNER_GLOW, 8.0, BLACK, [0, 0]);
        o.mode = mode;
        pass.run(&gpu, &src, &dst, cw, ch, &[o], &[]);
        let px = readback(&gpu, &dst, cw, ch);
        let at = |x: u32, y: u32| i32::from(px[((y * cw + x) * 4) as usize]);
        // Reentrante: 2 texels na diagonal para dentro da quina (distância 2,83 da borda).
        // Reta: 3 texels acima da aresta de baixo do braço, longe de qualquer quina.
        (at(v0 + 2, h0 + 2), at(v0 - 20, h0 + 3))
    };
    let (prox_notch, prox_edge) = probe(&mut pass, FxOp::MODE_PROXIMITY);
    let (cont_notch, cont_edge) = probe(&mut pass, FxOp::MODE_CONTOUR);
    // ⚠️ **A profundidade de uma sombra é uma grandeza de LUZ, e é aí que estas barras moram.**
    // Em bytes codificados a transferência comprime o topo da faixa, então a MESMA diferença de
    // sombra lê menor perto do branco — medido, os dois modos separavam 33 contra 19 níveis (fator
    // 1,7) e em luz linear separam 0,2445 contra 0,0940 (fator **2,6**). Medir na grandeza certa
    // deixou o gate mais afiado, não mais frouxo.
    let lin = |b: i32| f64::from(ph2d_color::srgb::srgb_to_linear_byte(b.clamp(0, 255) as u8));
    let prox_gap = lin(prox_notch) - lin(prox_edge);
    let cont_gap = (lin(cont_notch) - lin(cont_edge)).abs();
    eprintln!(
        "[contorno] PROXIMITY reentrancia {prox_notch} vs aresta {prox_edge} (vao {prox_gap:.4}) · \
         CONTOUR reentrancia {cont_notch} vs aresta {cont_edge} (vao {cont_gap:.4}) \
         (0 = sombra cheia, 255 = sem sombra; vaos em luz linear)"
    );
    // A lei do CONTOUR: à mesma distância da borda, a mesma sombra — na quina e na reta.
    assert!(
        cont_gap <= 0.14,
        "no modo Contour a reentrancia ({cont_notch}) tinha de escurecer como a aresta \
         ({cont_edge}) — a banda segue o CONTORNO (vao {cont_gap:.4} em luz)"
    );
    // E o controle: no PROXIMITY a reentrância é MUITO mais clara — é o defeito reportado, e ele
    // continua lá de propósito (é o outro modo, não um bug).
    assert!(
        prox_gap >= 0.18,
        "o modo Proximity tinha de deixar a reentrancia bem mais CLARA que a aresta \
         (deu {prox_notch} contra {prox_edge}, vao {prox_gap:.4} em luz) — se os dois modos \
         concordam, um deles nao existe"
    );
    // E as duas leis escurecem de verdade a aresta reta (senão o gate acima passaria com tudo claro).
    //
    // ⚠️ Também em LUZ, e pela mesma razão — mais o fato que a pilha linear tornou visível: reduzir
    // metade da LUZ é o byte 188, não 128. Um sombreamento correto lê mais claro em bytes do que a
    // aritmética de gama fazia crer, e a barra tem de falar da grandeza que a sombra é. Medido:
    // Contour deixa 0,3185 da luz, Proximity 0,6105.
    assert!(
        lin(cont_edge) < 0.70 && lin(prox_edge) < 0.70,
        "a aresta reta tinha de escurecer nos dois modos (luz {:.4} / {:.4})",
        lin(cont_edge),
        lin(prox_edge)
    );
}

/// **A banda do modo Contour é função da DISTÂNCIA, e de mais nada.**
///
/// ⚠️ **O oráculo aqui é um BUCKET, não uma linha de sonda, e a diferença decidiu a wave.** Andar
/// "paralelo à aresta" numa grade de texels obriga a arredondar o `y`, e o arredondamento sozinho
/// move a sonda ±0,5 px através de uma banda cujo gradiente é ~32 níveis/px: o gate media **34
/// níveis de oscilação** sobre um campo que podia estar perfeito. Agora ele agrupa TODOS os texels
/// cuja distância VERDADEIRA (analítica, o meio-plano é conhecido) cai numa fatia estreita, e exige
/// que a sombra deles concorde — que é a propriedade de verdade: *à mesma distância, a mesma
/// sombra*, independentemente de onde o texel esteja ao longo da aresta.
///
/// ⚠️ E a aresta é **obliqua de propósito** (21,8°): a 45° o texel-semente mais próximo é o mesmo
/// para toda a linha `x + y` constante, então a discretização some POR SIMETRIA e o gate ficaria
/// verde sobre um produto que penteia.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_contour_band_is_a_function_of_distance_alone() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let (dw, dh) = (128u32, 128u32);
    let (nx, ny) = (0.371_4f64, 0.928_5f64); // normal unitária ≈ 21,8°
    let sd_at = |x: u32, y: u32| (f64::from(x) - 64.0).mul_add(nx, (f64::from(y) - 64.0) * ny);
    let mut bytes = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        for x in 0..dw {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = ((0.5 - sd_at(x, y)).clamp(0.0, 1.0) * 255.0).round() as u8;
            let o = ((y * dw + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[a, a, a, a]);
        }
    }
    let src = make_src(&gpu, dw, dh, &bytes);
    let dst = make_output_texture(&gpu, dw, dh);
    let mut pass = FxStackPass::new(&gpu);
    let mut o = one(FxOp::INNER_GLOW, 8.0, BLACK, [0, 0]);
    o.mode = FxOp::MODE_CONTOUR;
    pass.run(&gpu, &src, &dst, dw, dh, &[o], &[]);
    let px = readback(&gpu, &dst, dw, dh);
    // Todos os texels a 3,0..3,1 px para DENTRO da aresta, longe das bordas da textura.
    let mut band: Vec<i32> = Vec::new();
    for y in 20..108u32 {
        for x in 20..108u32 {
            let d = -sd_at(x, y);
            if (2.85..3.15).contains(&d) {
                band.push(i32::from(px[((y * dw + x) * 4) as usize]));
            }
        }
    }
    let (lo, hi) = (
        *band.iter().min().expect("banda"),
        *band.iter().max().expect("banda"),
    );
    eprintln!(
        "[diagonal] {} texels a 3,0-3,1 px da aresta: sombra {lo}..{hi} ({} niveis)",
        band.len(),
        hi - lo
    );
    assert!(
        band.len() > 10,
        "a fatia tem de conter texels ({})",
        band.len()
    );
    assert!(
        hi < 160,
        "o controle POSITIVO falhou: a fatia tem de cair DENTRO da banda (deu {lo}..{hi})"
    );
    assert!(
        hi - lo <= 6,
        "a sombra varia {} niveis entre texels à MESMA distância ({lo}..{hi}) — o campo depende de \
         onde o texel está ao longo da aresta, e isso é o PENTE que o olho vê",
        hi - lo
    );
}

/// **O contorno é REDONDO na quina, e isso é uma propriedade da representação — não um ajuste.**
///
/// O pedido de *"opção de arredondar ou não"* esbarra numa derivação, e o gate existe para gravá-la
/// junto do número: numa quina convexa de ângulo interno `θ`, a ponta de um **miter** fica a
/// `w/sin(θ/2)` do vértice, enquanto qualquer junção redonda alcança exactamente `w`. Numa ponta de
/// estrela (`θ ≈ 36°`) isso é **3,24 × w**.
///
/// Ora, o nosso contorno (como QUALQUER dilatação morfológica) é uma soma de Minkowski `A ⊕ S`: para
/// esticar 3,24 w na quina, o `S` teria de conter um ponto a 3,24 w naquela direção — e aí engordaria
/// 3,24 w **também na aresta reta**. ⇒ **Nenhuma dilatação é `w` na reta e `3,24 w` na ponta.** O que
/// decide um miter são as DIREÇÕES das duas arestas que se encontram, e isso não está no campo de
/// alfa: é geometria, e mora na pilha de Effects (`VecOffset { join }`), não aqui.
///
/// O gate mede o alcance na PONTA contra o alcance na ARESTA: iguais ⇒ redondo. Se algum dia
/// alguém "consertar" a quina por acidente, é aqui que aparece.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_outline_is_round_at_a_corner_because_a_dilation_cannot_be_anything_else() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let (tw, th) = (192u32, 192u32);
    // Uma cunha de 36° apontando para +X, com a ponta em (120, 96) e a base à esquerda.
    let (tipx, tipy) = (120.0f64, 96.0f64);
    let half = (18.0f64).to_radians().tan();
    let mut bytes = vec![0u8; (tw * th * 4) as usize];
    for y in 0..th {
        for x in 0..tw {
            let dx = tipx - f64::from(x);
            let dy = f64::from(y) - tipy;
            if dx > 0.0 && dx < 90.0 && dy.abs() <= dx * half {
                let o = ((y * tw + x) * 4) as usize;
                bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
    let src = make_src(&gpu, tw, th, &bytes);
    let dst = make_output_texture(&gpu, tw, th);
    let mut pass = FxStackPass::new(&gpu);
    let w = 10.0f32;
    pass.run(
        &gpu,
        &src,
        &dst,
        tw,
        th,
        &[one(FxOp::OUTLINE, w, RED, [0, 0])],
        &[],
    );
    let px = readback(&gpu, &dst, tw, th);
    let alpha = |x: u32, y: u32| i32::from(px[(((y * tw + x) * 4) + 3) as usize]);
    // Alcance ALÉM da ponta, ao longo do eixo (o bissetor).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tip = tipx as u32;
    let tip_reach = (tip..tw)
        .rfind(|x| alpha(*x, 96) > 128)
        .map_or(0.0, |x| f64::from(x) - tipx);
    // Alcance acima de uma aresta, medido na PERPENDICULAR dela (a cunha é inclinada 18°).
    let (ex, ey) = (60u32, 96 - (60.0 * half).round() as u32);
    let edge_reach = (0..40u32).take_while(|k| alpha(ex, ey - k) > 128).count() as f64
        * (18.0f64).to_radians().cos();
    let miter = f64::from(w) / (18.0f64).to_radians().sin();
    eprintln!(
        "[quina] alcance na PONTA {tip_reach:.1} px · na ARESTA {edge_reach:.1} px · \
         um miter pediria {miter:.1} px (largura {w})"
    );
    assert!(
        tip_reach > 1.0,
        "a PONTA ficou sem contorno nenhum ({tip_reach:.1} px) — foi exatamente isso que o corte \
         num campo BORRADO fazia numa quina de 36 graus, e é por isso que o contorno passou a ser \
         uma dilatação sobre o campo de DISTÂNCIA"
    );
    assert!(
        (tip_reach - edge_reach).abs() <= 2.5,
        "o contorno tem de alcançar o MESMO tanto na ponta ({tip_reach:.1}) e na aresta \
         ({edge_reach:.1}) — é o que 'redondo' significa"
    );
    assert!(
        tip_reach < miter * 0.6,
        "a ponta alcançou {tip_reach:.1}, perto dos {miter:.1} de um miter — se isto acontecer, a \
         derivação acima está errada e o comentário deste gate tem de mudar junto"
    );
}

/// **O FEATHER amacia a BORDA e não toca o MIOLO — é isso que o separa de um Blur.**
///
/// A fixture tem LISTRAS dentro da forma, e é a única razão de o gate poder falhar: numa forma de
/// cor lisa um borrão também não muda o miolo (não há o que misturar), então uma fixture uniforme
/// deixaria o Feather e o Blur indistinguíveis. A rampa é CENTRADA na fronteira (a forma ganha
/// alfa parcial para FORA e perde para dentro), que é o que um feather faz e um recorte não.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_feather_softens_the_edge_and_leaves_the_interior_alone() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in BY0..BY1 {
        for x in BX0..BX1 {
            // Listras de período 4 px: o DETALHE que um borrão destrói e um feather preserva.
            let v = if (x / 2) % 2 == 0 { 255u8 } else { 60u8 };
            let o = ((y * W + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let dst = make_output_texture(&gpu, W, H);
    let mut pass = FxStackPass::new(&gpu);
    let run = |pass: &mut FxStackPass, ops: &[FxOpGpu]| {
        pass.run(&gpu, &src, &dst, W, H, ops, &[]);
        readback(&gpu, &dst, W, H)
    };
    let plain = run(&mut pass, &[]);
    let feathered = run(&mut pass, &[one(FxOp::FEATHER, 8.0, RED, [0, 0])]);
    let blurred = run(&mut pass, &[one(FxOp::BLUR, 8.0, RED, [0, 0])]);
    // O contraste das listras no MIOLO (longe de qualquer borda).
    let contrast = |px: &[u8]| {
        let y = (BY0 + BY1) / 2;
        let band: Vec<i32> = (BX0 + 20..BX1 - 20).map(|x| rgb_at(px, x, y)[0]).collect();
        band.iter().max().copied().unwrap_or(0) - band.iter().min().copied().unwrap_or(0)
    };
    eprintln!(
        "[feather] contraste do miolo: nu {} · feather {} · blur {}",
        contrast(&plain),
        contrast(&feathered),
        contrast(&blurred)
    );
    assert_eq!(
        contrast(&feathered),
        contrast(&plain),
        "o feather tem de deixar o MIOLO intacto"
    );
    assert!(
        contrast(&blurred) * 3 < contrast(&plain),
        "o controle falhou: um borrão do mesmo raio tinha de lavar as listras ({} contra {})",
        contrast(&blurred),
        contrast(&plain)
    );
    // ⚠️ E DENTRO da banda a cor de cada texel continua a DELE — só o alfa muda. Sem esta metade,
    // "pinte o miolo com a cor da borda" passava: no miolo o campo nem existe (o JFA é limitado),
    // então a mutação era a identidade exatamente onde a sonda de contraste olhava.
    let y = (BY0 + BY1) / 2;
    for x in BX1 - 6..BX1 - 1 {
        assert_eq!(
            rgb_at(&feathered, x, y),
            rgb_at(&plain, x, y),
            "o feather repintou o texel {x} (dentro da banda) — ele muda a COBERTURA, não a cor"
        );
    }
    // A rampa é CENTRADA: a forma ganha alfa para FORA e perde para dentro.
    let (out3, edge, in3) = (
        alpha_at(&feathered, BX1 + 2, y),
        alpha_at(&feathered, BX1 - 1, y),
        alpha_at(&feathered, BX1 - 6, y),
    );
    eprintln!("[feather] alfa: fora+2 {out3} · na borda {edge} · dentro-6 {in3}");
    assert!(
        out3 > 20,
        "o feather tem de sangrar para FORA da silhueta (deu {out3}) — senão é um recorte, não uma \
         rampa centrada"
    );
    assert!(
        in3 > edge && edge > out3,
        "a rampa tem de subir de fora para dentro ({out3} < {edge} < {in3})"
    );
    assert_eq!(
        alpha_at(&feathered, (BX0 + BX1) / 2, y),
        255,
        "e o miolo continua opaco"
    );
}

/// **O BEVEL acende a face virada para a LUZ e escurece a oposta; trocar a luz TROCA os dois.**
///
/// A metade que importa é a inversão: um gate que só olhasse "um lado ficou claro" passaria com um
/// efeito que clareia sempre o mesmo lado, ignorando o knob de luz.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_bevel_lights_the_rim_that_faces_the_light_and_flips_with_it() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let y = (BY0 + BY1) / 2;
    // ⚠️ Forma CINZA, e é a única razão de o gate poder falhar: sobre branco o realce não tem para
    // onde subir (255 é o teto), e a metade "acende" ficaria verde sobre um efeito que só escurece.
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for yy in BY0..BY1 {
        for xx in BX0..BX1 {
            let o = ((yy * W + xx) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[128, 128, 128, 255]);
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let dst = make_output_texture(&gpu, W, H);
    // Luz vinda da ESQUERDA e depois da DIREITA, no mesmo relevo.
    let probe = |pass: &mut FxStackPass, light: [i32; 2]| -> (i32, i32, i32, i32) {
        pass.run(
            &gpu,
            &src,
            &dst,
            W,
            H,
            &[one(FxOp::BEVEL, 8.0, BLACK, light)],
            &[],
        );
        let out = readback(&gpu, &dst, W, H);
        (
            rgb_at(&out, BX0 + 2, y)[0],
            rgb_at(&out, BX1 - 3, y)[0],
            rgb_at(&out, (BX0 + BX1) / 2, y)[0],
            // Mais fundo no rebordo: é aqui que se vê o relevo MORRER para o miolo.
            rgb_at(&out, BX0 + 6, y)[0],
        )
    };
    let (l_left, l_right, l_core, l_deep) = probe(&mut pass, [-8, 0]);
    let (r_left, r_right, r_core, _) = probe(&mut pass, [8, 0]);
    eprintln!(
        "[bevel] luz da ESQUERDA: rim esq {l_left} · rim dir {l_right} · miolo {l_core}\n\
         [bevel] luz da DIREITA: rim esq {r_left} · rim dir {r_right} · miolo {r_core}"
    );
    assert!(
        l_left > l_core + 10 && l_right + 10 < l_core,
        "com a luz da esquerda o rim ESQUERDO acende ({l_left}) e o direito escurece ({l_right}), \
         contra o miolo ({l_core})"
    );
    assert!(
        r_right > r_core + 10 && r_left + 10 < r_core,
        "e trocar a luz TROCA os dois (esq {r_left}, dir {r_right}, miolo {r_core})"
    );
    assert_eq!(l_core, r_core, "o miolo não é tocado por nenhuma das duas");
    // ⚠️ E o relevo DECAI para dentro. Sem esta metade, um bevel de intensidade constante passava:
    // no miolo o campo nem existe (o JFA é limitado), então ele já estava intacto de graça.
    assert!(
        l_deep < l_left - 20 && l_deep > l_core,
        "o realce tem de MORRER para o miolo: rim {l_left}, a 6 px {l_deep}, miolo {l_core}"
    );
}

/// **O CONTORNO também é função da distância, e de mais nada** — irmão do gate da banda, do outro
/// lado da fronteira. Mede a fatia de texels FORA da forma, à mesma distância verdadeira: se o
/// campo dependesse de onde o texel está ao longo da aresta, o serrilhado apareceria aqui.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_outline_edge_is_a_function_of_distance_alone() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_kinds] sem adapter — skip");
        return;
    };
    let (dw, dh) = (128u32, 128u32);
    let (nx, ny) = (0.371_4f64, 0.928_5f64);
    let sd_at = |x: u32, y: u32| (f64::from(x) - 64.0).mul_add(nx, (f64::from(y) - 64.0) * ny);
    let mut bytes = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        for x in 0..dw {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = ((0.5 - sd_at(x, y)).clamp(0.0, 1.0) * 255.0).round() as u8;
            let o = ((y * dw + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[a, a, a, a]);
        }
    }
    let src = make_src(&gpu, dw, dh, &bytes);
    let dst = make_output_texture(&gpu, dw, dh);
    let mut pass = FxStackPass::new(&gpu);
    pass.run(
        &gpu,
        &src,
        &dst,
        dw,
        dh,
        &[one(FxOp::OUTLINE, 8.0, RED, [0, 0])],
        &[],
    );
    let px = readback(&gpu, &dst, dw, dh);
    // A fatia a ~4 px para FORA (o contorno de largura 8 ainda é opaco lá) e a ~7,9 (a borda dele).
    for (lo_d, hi_d, what) in [
        (3.85, 4.15, "no meio do contorno"),
        (7.4, 7.7, "na borda dele"),
    ] {
        let mut band: Vec<i32> = Vec::new();
        for y in 20..108u32 {
            for x in 20..108u32 {
                let d = sd_at(x, y);
                if (lo_d..hi_d).contains(&d) {
                    band.push(i32::from(px[(((y * dw + x) * 4) + 3) as usize]));
                }
            }
        }
        let (lo, hi) = (
            *band.iter().min().expect("banda"),
            *band.iter().max().expect("banda"),
        );
        eprintln!(
            "[contorno-borda] {} texels {what}: alfa {lo}..{hi} ({} niveis)",
            band.len(),
            hi - lo
        );
        assert!(band.len() > 10, "a fatia tem de conter texels");
        assert!(
            hi - lo <= 8,
            "o alfa do contorno varia {} niveis entre texels à MESMA distância ({what}: {lo}..{hi})",
            hi - lo
        );
    }
}
