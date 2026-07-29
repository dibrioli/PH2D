//! **GROW / SHRINK, no dispositivo** (plano 24 W7) — a silhueta engorda ou afina.
//!
//! Arquivo próprio pelo mesmo motivo dos irmãos: o assunto é coeso e os outros estão perto do teto
//! de LOC.
//!
//! # O oráculo é ONDE A BORDA FICOU, e é ele porque é o que a operação É
//!
//! Uma morfologia é uma afirmação sobre um LUGAR: *a fronteira anda `r` ao longo da própria
//! normal*. Então a medição é a **posição sub-pixel do contorno** — num meio-plano ela dá o sinal e
//! a magnitude de uma vez, e numa QUINA ela responde a pergunta que separa este motor do
//! `feMorphology` do SVG: o elemento estruturante é um DISCO ou um RETÂNGULO?
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_morphology_gpu --release -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 128;
const H: u32 = 128;
/// Onde a borda do meio-plano cai, em texels.
const EDGE: f64 = 64.0;

/// Um degrau de morfologia — o único knob dele é o `grow_px`, COM SINAL.
fn morph(grow_px: f32) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::MORPHOLOGY,
        sigma_px: 0.0,
        offset_px: [0, 0],
        tint: [0.0, 0.0, 0.0, 1.0],
        opacity: 1.0,
        mode: 0,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px,
    }
}

/// Um contorno de largura `w`, preto — o degrau que a morfologia tem de ENGORDAR em vez de
/// recortar.
fn outline(w: f32) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::OUTLINE,
        sigma_px: w,
        offset_px: [0, 0],
        tint: [0.0, 0.0, 0.0, 1.0],
        opacity: 1.0,
        mode: 0,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
    }
}

/// Um meio-plano opaco com borda antialiasada em `edge` (alfa RETO, como o Vello entrega).
fn half_plane(gpu: &ph2d_gpu::GpuContext, edge: f64) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let cov = (edge - (f64::from(x) + 0.5) + 0.5).clamp(0.0, 1.0);
            let o = ((y * W + x) * 4) as usize;
            bytes[o] = 235;
            bytes[o + 1] = 175;
            bytes[o + 2] = 60;
            bytes[o + 3] = (cov * 255.0).round() as u8;
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// A MESMA aresta, como segmento vertical — a geometria que o campo exato consome.
fn half_plane_segments() -> Vec<[f32; 4]> {
    let far = (W as f32) * 4.0;
    vec![[EDGE as f32, -far, EDGE as f32, far]]
}

/// Uma CAIXA opaca `[lo, hi]²`, com cobertura EXATA (a área do texel dentro da caixa — separável
/// num retângulo alinhado aos eixos, então não há supersampling a aproximar nada).
fn box_source(gpu: &ph2d_gpu::GpuContext, lo: f64, hi: f64) -> wgpu::Texture {
    let cov1 = |i: u32| {
        let (a, b) = (f64::from(i), f64::from(i) + 1.0);
        (b.min(hi) - a.max(lo)).clamp(0.0, 1.0)
    };
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let o = ((y * W + x) * 4) as usize;
            bytes[o] = 235;
            bytes[o + 1] = 175;
            bytes[o + 2] = 60;
            bytes[o + 3] = (cov1(x) * cov1(y) * 255.0).round() as u8;
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// Roda a pilha (sem geometria) e devolve os bytes (sRGB, alfa reto).
fn run(gpu: &ph2d_gpu::GpuContext, src: &wgpu::Texture, ops: &[FxOpGpu]) -> Vec<u8> {
    run_with_geom(gpu, src, ops, &[])
}

/// Roda a pilha COM a silhueta em segmentos — o caminho do produto quando a forma é vetorial.
fn run_with_geom(
    gpu: &ph2d_gpu::GpuContext,
    src: &wgpu::Texture,
    ops: &[FxOpGpu],
    segs: &[[f32; 4]],
) -> Vec<u8> {
    let mut pass = FxStackPass::new(gpu);
    let dst = make_output_texture(gpu, W, H);
    pass.run(gpu, src, &dst, W, H, ops, segs);
    readback(gpu, &dst, W, H)
}

fn alpha(px: &[u8], x: u32, y: u32) -> f64 {
    f64::from(px[(((y * W + x) * 4) + 3) as usize])
}

/// **ONDE O CONTORNO FICOU**, na linha `y`: o `x` sub-pixel onde o alfa cruza a meia-cobertura.
fn contour_x(px: &[u8], y: u32) -> f64 {
    let x = (1..W)
        .find(|&x| alpha(px, x, y) < 128.0 && alpha(px, x - 1, y) >= 128.0)
        .expect("a borda tem de estar no quadro");
    let (hi, lo) = (alpha(px, x - 1, y), alpha(px, x, y));
    let t = if (hi - lo).abs() < 1e-9 {
        0.0
    } else {
        (hi - 128.0) / (hi - lo)
    };
    f64::from(x - 1) + t
}

/// A média do contorno sobre o miolo das linhas (um meio-plano é invariante em `y`, então a média
/// só tira o ruído de quantização de 8 bits).
fn contour(px: &[u8]) -> f64 {
    let rows: Vec<f64> = (16..H - 16).map(|y| contour_x(px, y)).collect();
    rows.iter().sum::<f64>() / rows.len() as f64
}

// ── O NEUTRO ──────────────────────────────────────────────────────────────────────────────────

/// **Amount 0 é o degrau que não faz NADA** — byte a byte.
///
/// ⚠️ O slider é BIPOLAR: o artista atravessa o zero a arrastar. Re-derivar a cobertura do campo
/// devolveria um anti-aliasing *quase* igual ao da fonte, e "quase" numa passagem que acontece a
/// todo arrasto é um pisca que ninguém consegue atribuir.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_zero_amount_is_byte_identical_to_no_morphology_at_all() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let src = half_plane(&gpu, EDGE);
    let bare = run(&gpu, &src, &[]);
    let zero = run(&gpu, &src, &[morph(0.0)]);
    let diff = bare.iter().zip(&zero).filter(|(a, b)| a != b).count();
    assert_eq!(diff, 0, "Amount 0 tem de ser byte-idêntico à pilha vazia");
}

// ── O SINAL E A MAGNITUDE ─────────────────────────────────────────────────────────────────────

/// **O contorno anda o que se pediu, e o sinal é a direção** — a wave inteira numa sweep.
///
/// ⚠️ **A tolerância admite a RÉGUA, e a régua foi MEDIDA, não suposta** (a sonda
/// `measure_where_each_law_puts_the_contour`): o campo semeado pelo raster põe a fronteira ~0,5 px
/// adiante numa aresta DURA alinhada aos eixos quando o JFA propaga longe — e não é desta operação,
/// é da régua: um **Outline** de 8 px, lei completamente diferente, mede `+8,494` no mesmo caminho,
/// contra `+8,000` pelo pé exato da geometria. A fixture é o pior caso do estimador de sub-texel
/// (rampa de anti-aliasing de exatamente 1 texel, com a diferença central a ler amostras saturadas).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_contour_moves_by_the_amount_it_was_given_and_the_sign_is_the_direction() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let src = half_plane(&gpu, EDGE);
    let base = contour(&run(&gpu, &src, &[]));
    let mut prev = f64::NEG_INFINITY;
    for amount in [-12.0_f32, -8.0, -3.0, -1.0, 1.0, 3.0, 8.0, 12.0] {
        let moved = contour(&run(&gpu, &src, &[morph(amount)]));
        let walked = moved - base;
        eprintln!(
            "amount {amount:+6.1} px -> andou {walked:+7.3} px (erro {:+.3})",
            walked - f64::from(amount)
        );
        assert!(
            (walked - f64::from(amount)).abs() < 0.6,
            "grow {amount} devia andar {amount} px, andou {walked:.3}"
        );
        assert!(
            walked > prev,
            "o contorno tem de andar MONOTONICAMENTE com o knob: {walked:.3} veio depois de {prev:.3}"
        );
        prev = walked;
    }
}

/// **Crescer `r` e contornar `r` põem a fronteira NO MESMO LUGAR** — porque são o mesmo conjunto.
///
/// Um contorno de largura `r` é, por definição, a silhueta dilatada por `r` (o próprio comentário
/// dele diz *"isto é uma DILATAÇÃO de verdade (`d <= w`)"*), e um Grow de `r` é a mesma dilatação a
/// carregar a arte em vez de uma cor chapada. **Duas leis independentes sobre o mesmo campo**, e é
/// isso que torna este oráculo mais forte que uma tolerância contra o ideal: ele não pergunta se a
/// morfologia acerta um número, pergunta se ela concorda com a dilatação que o módulo já ship.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_grow_and_the_outline_agree_on_where_the_dilated_boundary_is() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let src = half_plane(&gpu, EDGE);
    for r in [3.0_f32, 8.0, 12.0] {
        let by_outline = contour(&run(&gpu, &src, &[outline(r)]));
        let by_grow = contour(&run(&gpu, &src, &[morph(r)]));
        eprintln!("r = {r:.0}: contorno {by_outline:.3} · grow {by_grow:.3}");
        assert!(
            (by_outline - by_grow).abs() < 0.05,
            "Grow({r}) e Outline({r}) descrevem o MESMO conjunto e discordaram: \
             {by_grow:.3} contra {by_outline:.3}"
        );
    }
}

// ── O ELEMENTO ESTRUTURANTE ───────────────────────────────────────────────────────────────────

/// **A quina cresce em DISCO, não em retângulo** — e é o que separa este motor do `feMorphology`.
///
/// O SVG dilata com um elemento estruturante RETANGULAR (o `radius` é `rx`,`ry`), então uma quina a
/// 90° cresce `r` em cada eixo e o alcance na DIAGONAL sai `r√2` — a quina fica quadrada. Com o
/// campo de distância euclidiano o conjunto novo é `{d ≤ r}`, ou seja um quarto de círculo: o
/// alcance diagonal é `r`, e é essa a forma que o Photoshop tem de oferecer como opção
/// (*Preserve: Roundness*) e que um artista espera de um "engordar".
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_structuring_element_is_a_disc_not_a_rectangle() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let (lo, hi) = (40.0_f64, 88.0_f64);
    let src = box_source(&gpu, lo, hi);
    let r = 10.0_f64;
    let px = run(&gpu, &src, &[morph(r as f32)]);
    // Anda pela diagonal a partir da quina e acha onde a cobertura cai a meio.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let at = |t: f64| {
        let (x, y) = (hi + t * s, hi + t * s);
        alpha(&px, x.floor() as u32, y.floor() as u32)
    };
    let mut reach = 0.0;
    let mut t = 0.0;
    while t < r * 2.0 {
        if at(t) < 128.0 {
            reach = t;
            break;
        }
        t += 0.25;
    }
    let square = r * std::f64::consts::SQRT_2;
    eprintln!(
        "alcance diagonal na quina: {reach:.2} px (disco = {r:.2} · retângulo = {square:.2})"
    );
    assert!(
        (reach - r).abs() < 1.5,
        "a quina devia crescer {r} px na diagonal (disco), cresceu {reach:.2}"
    );
    assert!(
        reach < square - 2.0,
        "a quina cresceu {reach:.2}, que é o alcance de um RETÂNGULO ({square:.2})"
    );
}

// ── A COR DA ÁREA NOVA ────────────────────────────────────────────────────────────────────────

/// **A orla que nasceu veste a cor da borda de onde ela nasceu**, nunca preto transparente.
///
/// Crescer cria área onde a fonte não tem tinta nenhuma; a resposta vem da MESMA porta que o
/// feather usa (`straight_colour`), e uma segunda cópia divergiria exatamente aqui.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_ring_that_grew_wears_the_shapes_colour() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let src = half_plane(&gpu, EDGE);
    let px = run(&gpu, &src, &[morph(6.0)]);
    // Um texel bem dentro da orla nova (a fonte ali era 100% vazia).
    let x = EDGE as u32 + 3;
    let o = (((H / 2) * W + x) * 4) as usize;
    let (r, g, b, a) = (
        f64::from(px[o]),
        f64::from(px[o + 1]),
        f64::from(px[o + 2]),
        f64::from(px[o + 3]),
    );
    eprintln!("orla nova em x={x}: rgba({r:.0}, {g:.0}, {b:.0}, {a:.0})");
    assert!(a > 250.0, "a orla devia ser opaca, saiu alfa {a:.0}");
    for (got, want, name) in [(r, 235.0, "R"), (g, 175.0, "G"), (b, 60.0, "B")] {
        assert!(
            (got - want).abs() < 8.0,
            "a orla devia vestir a tinta da forma ({name} = {want}), veste {got:.0}"
        );
    }
}

/// Um meio-plano cuja tinta VARIA ao longo de `x` — a fixture que separa os dois braços da porta
/// de cor. Numa forma monocromática *"a minha cor"* e *"a cor da borda"* são o mesmo número, e um
/// gate escrito sobre ela fica verde com a porta inteira substituída por uma constante (medido).
fn gradient_half_plane(gpu: &ph2d_gpu::GpuContext, edge: f64) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let cov = (edge - (f64::from(x) + 0.5) + 0.5).clamp(0.0, 1.0);
            let o = ((y * W + x) * 4) as usize;
            // Vermelho sobe de 20 a 240 ao longo do eixo que cruza a fronteira.
            let r = 20.0 + 220.0 * (f64::from(x) / edge).clamp(0.0, 1.0);
            bytes[o] = r.round() as u8;
            bytes[o + 1] = 40;
            bytes[o + 2] = 40;
            bytes[o + 3] = (cov * 255.0).round() as u8;
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// **A forma guarda as cores DELA; só a orla nova é que empresta a da borda.**
///
/// ⚠️ Os dois braços de `straight_colour` respondem perguntas diferentes — *"tenho tinta?"* e
/// *"de quem eu herdo?"* — e colapsá-los repinta a IMAGEM INTEIRA com a cor do contorno. Com a
/// fixture monocromática dos irmãos essa mutação sobrevive: é preciso uma tinta que varie ao longo
/// do eixo que cruza a fronteira.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_shape_keeps_its_own_colours_and_only_the_new_ring_borrows() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let src = gradient_half_plane(&gpu, EDGE);
    let px = run(&gpu, &src, &[morph(6.0)]);
    let red = |x: u32| f64::from(px[(((H / 2) * W + x) * 4) as usize]);
    // Bem DENTRO: a tinta local (o gradiente em x=20 dá ~89), nunca a da borda (~240).
    let inside = red(20);
    // Na ORLA nova: a tinta da borda, que é o topo do gradiente.
    let ring = red(EDGE as u32 + 3);
    // ⚠️ E o ENCOLHER é o caso que de facto separa os dois braços: os texels que sobrevivem estão
    // DENTRO do alcance do JFA, então o `off` deles aponta para a borda ANTIGA. Colapsada a porta,
    // a faixa inteira que a operação preservou é repintada com a cor do contorno — no miolo o
    // defeito esconde-se, porque um texel fora do alcance do salto amostra a si próprio.
    let shrunk = run(&gpu, &src, &[morph(-12.0)]);
    // x=50 fica DENTRO do novo contorno (que caiu para 52) e a 14 px da borda antiga — ou
    // seja dentro do alcance do salto, que é onde a porta de cor tem de escolher certo.
    let near_edge = f64::from(shrunk[(((H / 2) * W + 50) * 4) as usize]);
    eprintln!(
        "dentro (x=20) R={inside:.0} · orla nova R={ring:.0} · encolhido (x=50) R={near_edge:.0}"
    );
    assert!(
        (near_edge - 192.0).abs() < 12.0,
        "ao ENCOLHER, a faixa preservada guarda a cor dela (~192), veio {near_edge:.0}"
    );
    assert!(
        (inside - 89.0).abs() < 12.0,
        "o miolo tem de guardar a cor DELE (~89), veio {inside:.0}"
    );
    assert!(
        ring > 200.0,
        "a orla tem de vestir a cor da BORDA (~240), veio {ring:.0}"
    );
}

// ── CONTRA O QUÊ ELA MEDE ─────────────────────────────────────────────────────────────────────

/// **A morfologia mede a IMAGEM que recebeu, não a FORMA** — o gate que pina a decisão de desenho.
///
/// ⚠️ **A fixture TEM de trazer geometria, e é o ponto inteiro.** Sem segmentos o produtor já semeia
/// o campo pela cobertura, então o defeito não existe e o gate ficaria verde sobre nada. Com
/// geometria disponível o finalize do campo resolve pelo pé EXATO da silhueta — o desenho certo
/// para o contorno, o feather e o bevel, e a resposta errada para esta.
///
/// A pilha é `Outline(8) → Grow(3)`. Medindo a imagem, o contorno final está a `8 + 3` da aresta;
/// medindo a forma, a morfologia RECORTA o contorno de volta a `3` e a pilha deixa de compor.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_morphology_measures_the_image_it_received_not_the_shape() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let src = half_plane(&gpu, EDGE);
    let segs = half_plane_segments();
    let (w, r) = (8.0_f32, 3.0_f32);
    let just_outline = contour(&run_with_geom(&gpu, &src, &[outline(w)], &segs));
    let grown = contour(&run_with_geom(&gpu, &src, &[outline(w), morph(r)], &segs));
    eprintln!(
        "contorno sozinho {just_outline:.2} · depois de Grow({r}) {grown:.2} (a forma cai em {EDGE})"
    );
    assert!(
        (just_outline - (EDGE + f64::from(w))).abs() < 1.5,
        "o contorno de {w} px devia acabar em {}, acabou em {just_outline:.2}",
        EDGE + f64::from(w)
    );
    let walked = grown - just_outline;
    assert!(
        (walked - f64::from(r)).abs() < 0.6,
        "o Grow devia ENGORDAR o contorno em {r} px (medindo a imagem), andou {walked:.2} — \
         medir a FORMA teria recortado para {}",
        EDGE + f64::from(r)
    );
}

/// SONDA (não é gate): o mesmo alcance pedido a um Outline e a um Grow, com e sem geometria — para
/// separar o que é lei da morfologia do que é a régua do campo.
#[test]
#[ignore = "sonda"]
fn measure_where_each_law_puts_the_contour() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = half_plane(&gpu, EDGE);
    let segs = half_plane_segments();
    let base = contour(&run(&gpu, &src, &[]));
    eprintln!("a fonte mede {base:.3} (a aresta cai em {EDGE}, logo o zero da régua e' {base:.3})");
    for w in [3.0_f32, 8.0] {
        let o_geom = contour(&run_with_geom(&gpu, &src, &[outline(w)], &segs));
        let o_rast = contour(&run(&gpu, &src, &[outline(w)]));
        let m_geom = contour(&run_with_geom(&gpu, &src, &[morph(w)], &segs));
        let m_rast = contour(&run(&gpu, &src, &[morph(w)]));
        eprintln!(
            "alcance {w:.0}: OUTLINE geom {:+.3} raster {:+.3} | GROW geom {:+.3} raster {:+.3}",
            o_geom - base,
            o_rast - base,
            m_geom - base,
            m_rast - base
        );
    }
}
