//! **A TURBULÊNCIA, no dispositivo** (plano 24 W6b) — a imagem deformada por um campo de ruído.
//!
//! Arquivo próprio pelo mesmo motivo dos irmãos: o assunto é coeso e os outros estão perto do teto
//! de LOC.
//!
//! # O oráculo é a BORDA, e é ela porque é o que o artista vê
//!
//! O campo de deslocamento não é observável (nenhum passe o escreve numa textura); o que sai é a
//! imagem deformada. Então a fixture é um **meio-plano opaco** e a medição é a **posição sub-pixel
//! da borda linha a linha** — uma curva `x(y)` que É o contorno desenhado. Dela saem todas as
//! perguntas desta wave: *quão longe a tinta anda* (amplitude), *quão grandes são as ondulações*
//! (cruzamentos de zero), *quanto grão fino há dentro delas* (a 2ª diferença), *é outro desenho?*
//! (semente) e *tem vinco?* (o modo).
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_turbulence_gpu -- --ignored --nocapture`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 128;
const H: u32 = 160;
/// Onde a borda da fixture cai, em texels.
const EDGE: u32 = 64;

/// Um degrau de turbulência.
fn turb(amount_px: f32, scale_px: f32, detail: u8, seed: u8, mode: u8) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::TURBULENCE,
        sigma_px: amount_px,
        offset_px: [0, 0],
        tint: [0.0, 0.0, 0.0, 1.0],
        opacity: 1.0,
        mode,
        blend: 0,
        noise_scale_px: scale_px,
        detail,
        seed,
    }
}

/// Um Glow **INERTE** (`tint.a = 0`, `opacity = 0`): `out = over + halo·(1−a)` com `a = 0` devolve
/// a entrada AO BIT, mas o `op_reach` dele é `3σ`.
///
/// Existe para uma coisa só: mudar a MARGEM da pilha sem mudar um pixel. É o instrumento do gate
/// de ancoragem — sem ele não há como perguntar *"o padrão depende do tamanho do scratch?"*.
fn inert_glow(sigma_px: f32) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::GLOW,
        sigma_px,
        offset_px: [0, 0],
        tint: [1.0, 1.0, 1.0, 0.0],
        opacity: 0.0,
        mode: FxOp::MODE_PROXIMITY,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
    }
}

/// Um meio-plano OPACO à esquerda de `edge`, com anti-aliasing de meio texel na fronteira — alfa
/// RETO, como o Vello entrega.
fn half_plane(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32, edge: f64) -> wgpu::Texture {
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let cov = (edge - (f64::from(x) + 0.5) + 0.5).clamp(0.0, 1.0);
            let o = ((y * w + x) * 4) as usize;
            bytes[o] = 235;
            bytes[o + 1] = 175;
            bytes[o + 2] = 60;
            bytes[o + 3] = (cov * 255.0).round() as u8;
        }
    }
    make_src(gpu, w, h, &bytes)
}

/// Roda a pilha e devolve os bytes (sRGB, alfa reto).
fn run(
    gpu: &ph2d_gpu::GpuContext,
    src: &wgpu::Texture,
    w: u32,
    h: u32,
    ops: &[FxOpGpu],
) -> Vec<u8> {
    let mut pass = FxStackPass::new(gpu);
    let dst = make_output_texture(gpu, w, h);
    pass.run(gpu, src, &dst, w, h, ops, &[]);
    readback(gpu, &dst, w, h)
}

/// **A CURVA DA BORDA**: para cada linha, o `x` sub-pixel onde o alfa cruza a meia-cobertura.
///
/// Linhas onde a borda saiu do quadro devolvem `None` — um `0.0` silencioso ali seria um pico
/// gigante que a estatística leria como estrutura.
fn edge_curve(px: &[u8], w: u32, h: u32) -> Vec<Option<f64>> {
    (0..h)
        .map(|y| {
            let a = |x: u32| f64::from(px[(((y * w + x) * 4) + 3) as usize]);
            (1..w)
                .find(|&x| a(x) < 128.0 && a(x - 1) >= 128.0)
                .map(|x| {
                    let (hi, lo) = (a(x - 1), a(x));
                    // Interpolação linear até o cruzamento dos 128 — é ela que dá o sub-pixel.
                    let t = if (hi - lo).abs() < 1e-9 {
                        0.0
                    } else {
                        (hi - 128.0) / (hi - lo)
                    };
                    f64::from(x - 1) + t
                })
        })
        .collect()
}

/// Quantas linhas do topo e da base a análise DESCARTA.
///
/// ⚠️ **Não é folga por gosto: é a margem que o produto reserva.** O deslocamento tem componente
/// em `y`, então uma linha a menos de `Amount` da borda do scratch amostra FORA dele — e lá o
/// `tap_img` devolve transparente, por construção. Na fixture a forma toca as bordas (um
/// meio-plano tem de atravessar a altura toda), então essas linhas ganham uma faixa vazia que a
/// detecção lê como *a borda está aqui*: medido, a curva ia de `[17,75; 65,28]` com o miolo INTEIRO
/// em 63,3 — um punhado de linhas de fronteira inflava a amplitude em 3×, e era isso, e só isso,
/// que reprovava três gates desta wave. No produto a `stack_reach` já reserva essa margem.
const CORE: usize = 24;

/// As linhas do MIOLO, onde a borda de fato existe.
fn defined(curve: &[Option<f64>]) -> Vec<f64> {
    defined_rows(curve, CORE, curve.len() - CORE)
}

/// O mesmo, numa janela explícita — a aresta DIAGONAL sai do quadro antes do fim, então a janela
/// dela é do gate, não da constante.
fn defined_rows(curve: &[Option<f64>], lo: usize, hi: usize) -> Vec<f64> {
    let core: Vec<f64> = curve[lo..hi].iter().filter_map(|c| *c).collect();
    // Controle positivo: se a detecção quebrar, uma lista curta passaria calada por toda
    // estatística abaixo.
    assert!(
        core.len() + 4 >= hi - lo,
        "a borda sumiu em {} linhas do miolo",
        hi - lo - core.len()
    );
    core
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// A amplitude do desvio da borda em relação à reta — *quão longe a tinta anda*.
fn amplitude(v: &[f64]) -> f64 {
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}

/// Quantas vezes a curva cruza a própria média — *quantas ondulações cabem na altura*.
fn crossings(v: &[f64]) -> usize {
    let m = mean(v);
    v.windows(2)
        .filter(|w| (w[0] - m).signum() != (w[1] - m).signum())
        .count()
}

/// A rugosidade: a média de |2ª diferença| dividida pela média de |1ª diferença|. Mede QUEBRA de
/// inclinação, e é adimensional — não confunde *mais estrutura fina* com *mais amplitude*.
fn roughness(v: &[f64]) -> f64 {
    let d1: f64 = v.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (v.len() - 1) as f64;
    let d2: f64 = v
        .windows(3)
        .map(|w| (w[2] - 2.0 * w[1] + w[0]).abs())
        .sum::<f64>()
        / (v.len() - 2) as f64;
    if d1 < 1e-9 { 0.0 } else { d2 / d1 }
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_zero_amount_is_byte_identical_to_no_turbulence_at_all() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let none = run(&gpu, &src, W, H, &[]);
    let zero = run(
        &gpu,
        &src,
        W,
        H,
        &[turb(0.0, 24.0, 3, 0, FxOp::MODE_SMOOTH)],
    );
    let diff = none.iter().zip(&zero).filter(|(a, b)| a != b).count();
    assert_eq!(
        diff, 0,
        "Amount 0 tem de ser no-op AO BYTE — é o que faz um degrau recém-criado, ou um knob \
         zerado, não mudar a arte"
    );
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_turbulence_moves_the_edge_by_about_the_amount_it_was_given() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let flat = defined(&edge_curve(&run(&gpu, &src, W, H, &[]), W, H));
    assert!(
        amplitude(&flat) < 0.02,
        "a fixture tem de ser RETA: amplitude {:.4}",
        amplitude(&flat)
    );
    for amount in [4.0f32, 8.0, 16.0] {
        let px = run(
            &gpu,
            &src,
            W,
            H,
            &[turb(amount, 24.0, 3, 0, FxOp::MODE_SMOOTH)],
        );
        let c = defined(&edge_curve(&px, W, H));
        let a = amplitude(&c);
        let lo = c.iter().cloned().fold(f64::MAX, f64::min);
        let hi = c.iter().cloned().fold(f64::MIN, f64::max);
        eprintln!("amount {amount:>5.1} px -> amplitude {a:.3} px · faixa [{lo:.2}, {hi:.2}]");
        // O campo vive em `[-1,1]` com média zero, então o desvio-padrão da borda é uma FRAÇÃO do
        // Amount — nunca o próprio número (isso seria um campo saturado). A faixa afirma o que
        // importa: o knob governa a escala, e não sobra nem falta uma ordem de grandeza.
        assert!(
            a > f64::from(amount) * 0.08 && a < f64::from(amount) * 0.75,
            "amount {amount} deu amplitude {a:.3}, fora da faixa proporcional"
        );
    }
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_size_is_the_size_of_the_ripples() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    // UMA oitava: com várias, a estrutura fina acrescenta cruzamentos e a razão mede a soma em vez
    // do tamanho. A fixture tem de conter o fenômeno E MAIS NADA.
    let n = |scale: f32| {
        let px = run(
            &gpu,
            &src,
            W,
            H,
            &[turb(8.0, scale, 1, 0, FxOp::MODE_SMOOTH)],
        );
        crossings(&defined(&edge_curve(&px, W, H)))
    };
    let (fine, coarse) = (n(16.0), n(48.0));
    eprintln!("size 16 px -> {fine} cruzamentos · size 48 px -> {coarse}");
    assert!(
        fine >= coarse * 2,
        "triplicar o tamanho tem de RAREAR as ondulações: {fine} vs {coarse}"
    );
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_detail_adds_fine_structure_without_changing_the_scale() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let r = |detail: u8| {
        let px = run(
            &gpu,
            &src,
            W,
            H,
            &[turb(10.0, 40.0, detail, 0, FxOp::MODE_SMOOTH)],
        );
        roughness(&defined(&edge_curve(&px, W, H)))
    };
    let (one, four) = (r(1), r(4));
    eprintln!("detail 1 -> rugosidade {one:.4} · detail 4 -> {four:.4}");
    assert!(
        four > one * 1.5,
        "somar oitavas tem de acrescentar estrutura FINA: {one:.4} -> {four:.4}"
    );
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn another_seed_is_another_drawing_of_the_same_kind() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let curve = |seed: u8| {
        let px = run(
            &gpu,
            &src,
            W,
            H,
            &[turb(8.0, 32.0, 3, seed, FxOp::MODE_SMOOTH)],
        );
        defined(&edge_curve(&px, W, H))
    };
    let (a, b) = (curve(0), curve(7));
    let apart = mean(
        &a.iter()
            .zip(&b)
            .map(|(x, y)| (x - y).abs())
            .collect::<Vec<_>>(),
    );
    let (aa, ab) = (amplitude(&a), amplitude(&b));
    eprintln!("semente 0 vs 7 -> distância média {apart:.3} px · amplitudes {aa:.3} / {ab:.3}");
    // OUTRO desenho…
    assert!(
        apart > aa * 0.5,
        "as duas sementes desenharam quase o mesmo"
    );
    // …do MESMO tipo: a semente não é um segundo controle de intensidade.
    assert!(
        (aa - ab).abs() < aa * 0.5,
        "trocar a semente mudou a AMPLITUDE ({aa:.3} vs {ab:.3}) — ela deve mudar só o desenho"
    );
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_creased_mode_breaks_the_slope_where_the_smooth_one_rolls() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let r = |mode: u8| {
        let px = run(&gpu, &src, W, H, &[turb(10.0, 40.0, 2, 3, mode)]);
        roughness(&defined(&edge_curve(&px, W, H)))
    };
    let (smooth, creased) = (r(FxOp::MODE_SMOOTH), r(FxOp::MODE_CREASED));
    eprintln!("smooth -> rugosidade {smooth:.4} · creased -> {creased:.4}");
    // `Σ|n|` tem VINCO onde cada oitava cruza o zero — quebra de inclinação, que é exatamente o
    // que a 2ª diferença mede. É a propriedade que dá nome ao modo.
    assert!(
        creased > smooth * 1.2,
        "o modo Creased tem de vincar: {smooth:.4} -> {creased:.4}"
    );
}

#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_warp_reads_between_texels_instead_of_snapping_to_one() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    // Uma borda DURA (sem anti-aliasing nenhum): a fonte só tem alfa 0 e 255.
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..EDGE {
            let o = ((y * W + x) * 4) as usize;
            bytes[o] = 235;
            bytes[o + 1] = 175;
            bytes[o + 2] = 60;
            bytes[o + 3] = 255;
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let px = run(
        &gpu,
        &src,
        W,
        H,
        &[turb(8.0, 32.0, 3, 0, FxOp::MODE_SMOOTH)],
    );
    let mid = px
        .chunks_exact(4)
        .filter(|p| p[3] > 8 && p[3] < 247)
        .count();
    eprintln!("texels de cobertura PARCIAL na saída: {mid}");
    // Amostrar pelo vizinho mais próximo só sabe copiar 0 ou 255: qualquer valor intermediário só
    // pode ter vindo de uma leitura ENTRE texels. É o que separa uma onda de uma escada.
    assert!(
        mid > 50,
        "a borda saiu quantizada ({mid} texels parciais) — a amostragem virou nearest"
    );
}

/// **OS DOIS EIXOS SÃO CAMPOS INDEPENDENTES, não um campo usado duas vezes.**
///
/// Um só campo nos dois eixos desloca tudo ao longo da DIAGONAL — e uma forma que é constante
/// naquela direção fica **exatamente onde estava**. É isso que a fixture explora: uma aresta a 45°
/// é invariante sob deslocamento diagonal, então com um campo só ela sai lisa, e com dois ela
/// ondula. A aresta VERTICAL dos outros gates não distingue os dois casos (ela ignora o `dy` por
/// construção), e é por isso que este gate precisa de uma fixture própria.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_two_axes_are_independent_fields_not_one_field_used_twice() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    // Opaco acima da reta `y = x`: constante ao longo de (1,1).
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let d = (f64::from(y) + 0.5) - (f64::from(x) + 0.5);
            let o = ((y * W + x) * 4) as usize;
            bytes[o] = 235;
            bytes[o + 1] = 175;
            bytes[o + 2] = 60;
            bytes[o + 3] = ((d + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    // A borda diagonal cruza a linha da esquerda para a direita, então a curva `x(y)` é uma RETA
    // inclinada: o que interessa é o desvio dela, não a amplitude bruta.
    let wobble = |ops: &[FxOpGpu]| {
        // A janela é do GATE: a diagonal entra em `y = x`, então ela só está dentro do quadro
        // enquanto `x < W`.
        let c = defined_rows(&edge_curve(&run(&gpu, &src, W, H, ops), W, H), CORE, 112);
        let n = c.len() as f64;
        // Remove a tendência linear (a própria inclinação da aresta) e mede o que sobra.
        let (sx, sy) = (n * (n - 1.0) / 2.0, c.iter().sum::<f64>());
        let sxx = (0..c.len()).map(|i| (i * i) as f64).sum::<f64>();
        let sxy = c.iter().enumerate().map(|(i, v)| i as f64 * v).sum::<f64>();
        let slope = (n * sxy - sx * sy) / (n * sxx - sx * sx);
        let icpt = (sy - slope * sx) / n;
        let res: Vec<f64> = c
            .iter()
            .enumerate()
            .map(|(i, v)| v - (slope * i as f64 + icpt))
            .collect();
        amplitude(&res)
    };
    let flat = wobble(&[]);
    let warped = wobble(&[turb(10.0, 32.0, 3, 0, FxOp::MODE_SMOOTH)]);
    eprintln!("aresta a 45° -> desvio {flat:.4} px sem turbulência · {warped:.4} px com");
    assert!(
        warped > 1.0 && warped > flat * 10.0,
        "a aresta diagonal mal se mexeu ({warped:.4} px): o deslocamento está todo na diagonal, \
         ou seja os dois eixos leem o MESMO campo"
    );
}

/// **O PADRÃO É PREGADO NA FORMA, NÃO NA TEXTURA.**
///
/// A grade do ruído é ancorada em `org` = a margem que a pilha reservou. Sem esse termo, ela fica
/// ancorada no canto do scratch — e a margem é função de TODA a pilha, então mexer no raio de um
/// Glow (ou só arrastar o Amount) faria o padrão inteiro **andar** por baixo da forma, um efeito
/// colateral entre degraus que ninguém consegue atribuir.
///
/// O instrumento é o [`inert_glow`]: ele muda a margem sem mudar um pixel.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn the_noise_is_pinned_to_the_shape_not_to_the_scratch() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let t = turb(8.0, 32.0, 3, 0, FxOp::MODE_SMOOTH);
    let glow = inert_glow(4.0);
    // A margem que o instrumento acrescenta — perguntada à MESMA porta que a dimensiona.
    let pad = ph2d_render::stack_reach(&[glow]).0;
    assert!(pad > 0, "o instrumento tem de MUDAR a margem");

    // A: a forma na origem, num scratch justo.
    let a_px = run(&gpu, &half_plane(&gpu, W, H, f64::from(EDGE)), W, H, &[t]);
    // B: o MESMO desenho deslocado de `pad`, num scratch maior, sob uma pilha cuja margem cresceu
    // exatamente `pad` — que é o que a shell faz quando um degrau a mais entra na pilha.
    let (bw, bh) = (W + 2 * pad, H + 2 * pad);
    let b_src = half_plane(&gpu, bw, bh, f64::from(EDGE + pad));
    let b_px = run(&gpu, &b_src, bw, bh, &[glow, t]);

    let a = edge_curve(&a_px, W, H);
    let b = edge_curve(&b_px, bw, bh);
    // Compara o MIOLO (longe da borda do quadro, onde os dois scratches recortam diferente).
    let m = 16usize;
    let mut worst = 0.0f64;
    for y in m..(H as usize - m) {
        let (Some(av), Some(bv)) = (a[y], b[y + pad as usize]) else {
            continue;
        };
        worst = worst.max((av - (bv - f64::from(pad))).abs());
    }
    eprintln!("margem +{pad} px -> pior desvio da borda: {worst:.4} px");
    assert!(
        worst < 0.25,
        "o padrão ANDOU quando a margem mudou (pior desvio {worst:.3} px): a grade do ruído está \
         ancorada no scratch, não na forma"
    );
}

/// **O que uma oitava CUSTA** — a medição que decide o `FxOp::MAX_DETAIL`, executável, em vez de
/// um número escolhido por conforto.
#[test]
#[ignore = "medição, não gate"]
fn measure_the_turbulence_octave_cost() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    const N: u32 = 512;
    let src = half_plane(&gpu, N, N, f64::from(N / 2));
    let mut pass = FxStackPass::new(&gpu);
    let dst = make_output_texture(&gpu, N, N);
    eprintln!("--- custo por oitava, {N}x{N} ---");
    let mut prev = 0.0f64;
    for detail in [1u8, 2, 3, 4, 6, 8, 12] {
        let ops = [turb(10.0, 40.0, detail, 0, FxOp::MODE_SMOOTH)];
        // Aquece (compilação de pipeline, alocação das work textures).
        pass.run(&gpu, &src, &dst, N, N, &ops, &[]);
        let _ = readback(&gpu, &dst, N, N);
        let t0 = std::time::Instant::now();
        const REPS: u32 = 20;
        for _ in 0..REPS {
            pass.run(&gpu, &src, &dst, N, N, &ops, &[]);
        }
        let _ = readback(&gpu, &dst, N, N);
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(REPS);
        eprintln!(
            "  detail {detail:>2} -> {ms:7.4} ms/passe   (+{:.4})",
            ms - prev
        );
        prev = ms;
    }
}

/// **Quanto uma oitava a mais ainda MOVE a imagem** — a outra metade da pergunta do teto: um limite
/// legítimo diz de que recurso ele é, e este diz *"além daqui a oitava custa e não desenha"*.
#[test]
#[ignore = "medição, não gate"]
fn measure_what_an_extra_octave_still_moves() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let curve = |detail: u8| {
        let px = run(
            &gpu,
            &src,
            W,
            H,
            &[turb(10.0, 40.0, detail, 0, FxOp::MODE_SMOOTH)],
        );
        defined(&edge_curve(&px, W, H))
    };
    eprintln!("--- o que a oitava N acrescenta (Size 40 px) ---");
    let mut prev = curve(1);
    for detail in 2u8..=10 {
        let now = curve(detail);
        let moved = mean(
            &prev
                .iter()
                .zip(&now)
                .map(|(a, b)| (a - b).abs())
                .collect::<Vec<_>>(),
        );
        eprintln!(
            "  detail {:>2} -> a borda moveu {moved:.4} px em relação a {}",
            detail,
            detail - 1
        );
        prev = now;
    }
}

/// **Os números da CENA DE SMOKE (`PH2D_BUILD_SMOKE=35`)** — medidos antes de a mensagem dela os
/// afirmar, que é a regra desta linha.
///
/// A cena fala MUNDO e este harness fala PIXEL, então os valores entram aqui já convertidos ao
/// zoom que a cena abre (`ZOOM` px por unidade de mundo). Um número afirmado na tela sem uma
/// medição por trás é a coisa que esta jornada já viu mentir duas vezes.
#[test]
#[ignore = "medição, não gate"]
fn measure_the_smoke_scene_pairs() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador: pulando");
        return;
    };
    /// Pixels por unidade de MUNDO na cena (as estrelas medem 1,35 e ocupam ~135 px).
    const ZOOM: f32 = 100.0;
    let src = half_plane(&gpu, W, H, f64::from(EDGE));
    let m = |amount: f32, scale: f32, detail: u8, mode: u8| {
        let ops = [turb(amount * ZOOM, scale * ZOOM, detail, 0, mode)];
        let c = defined(&edge_curve(&run(&gpu, &src, W, H, &ops), W, H));
        (amplitude(&c), crossings(&c), roughness(&c))
    };
    eprintln!("--- os quatro pares da cena =35 (zoom {ZOOM} px/mundo) ---");
    for (name, amount, scale, detail, mode) in [
        ("1a Amount 0.00", 0.00f32, 0.25f32, 3u8, FxOp::MODE_SMOOTH),
        ("1b Amount 0.25", 0.25, 0.25, 3, FxOp::MODE_SMOOTH),
        ("2a Size 0.12", 0.08, 0.12, 3, FxOp::MODE_SMOOTH),
        ("2b Size 0.50", 0.08, 0.50, 3, FxOp::MODE_SMOOTH),
        ("3a Detail 1", 0.08, 0.30, 1, FxOp::MODE_SMOOTH),
        ("3b Detail 6", 0.08, 0.30, 6, FxOp::MODE_SMOOTH),
        ("4a Smooth", 0.10, 0.30, 3, FxOp::MODE_SMOOTH),
        ("4b Creased", 0.10, 0.30, 3, FxOp::MODE_CREASED),
    ] {
        let (a, c, r) = m(amount, scale, detail, mode);
        eprintln!("  {name:<16} amplitude {a:6.2} px · {c:2} ondulações · rugosidade {r:.3}");
    }
}
