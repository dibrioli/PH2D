//! **GRADIENT MAP, no dispositivo** (plano 24 W11) — a rampa de N stops.
//!
//! # Este arquivo carrega DUAS afirmações, e a primeira é a wave inteira
//!
//! **(1) Um Gradient Map de dois stops nas pontas é o DUOTONE, ao byte** — não *"parecido"*, não
//! *"dentro de um limite"*: os mesmos bytes, do mesmo dispositivo, na mesma pilha. É o que torna
//! esta wave uma GENERALIZAÇÃO em vez de um segundo efeito que responde à mesma pergunta. Duas leis
//! vizinhas que discordassem sobre *"quão claro é este texel"* dariam ao artista duas fichas que
//! desenham coisas diferentes com os mesmos números, e ninguém lê um número numa screenshot.
//!
//! **(2) A LEI DA RAMPA é a que o app já ship** — o `gradient_map_lut` do `ph2d-painter-effects`,
//! escrito por outra wave, para a camada de ajuste do Painter, sem saber que este consumidor
//! existiria. O oráculo é aquela crate, e não *"o resultado parece razoável"*.
//!
//! ⚠️ **E a RÉGUA divergem de propósito, com o número medido.** O Painter mede claridade em
//! **Rec.601 sobre bytes de display**; esta pilha mede em **`L` do OKLab** (a régua que o Duotone e
//! o Luma to Alpha desta linha já usam, e a única perceptualmente uniforme das três que o app
//! carrega). Medido em sRGB 128: Rec.601 dá **0,502** e o OKLab **0,600**. Então a paridade que
//! este arquivo afirma é sobre *"que cor vive na posição `t`"* — a metade genuinamente
//! compartilhada — e **não** sobre *"que `t` este pixel tem"*, que é a divergência declarada.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_gradient_map_gpu --release -- --ignored`.

use ph2d_color::LinearRgba;
use ph2d_color::oklab::OklabColor;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use ph2d_ecs::FxOp;
use ph2d_painter_effects::adjustments::{
    ColorStop, GradientInterp, GradientMapParams, gradient_map_lut,
};
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

/// A moldura, idêntica à do irmão do Duotone: barra com margem, conteúdo em RAMPA de cinza.
///
/// ⚠️ **A fixture TEM de ser um degradê.** A lei é função da luminância da própria arte, então uma
/// chapa de cor sólida é **um único ponto do domínio** — e é justamente entre os stops que uma
/// rampa de N pontos pode estar errada.
const W: u32 = 96;
const H: u32 = 16;
const X0: u32 = 16;
const X1: u32 = 80;
const Y0: u32 = 4;
const Y1: u32 = 12;
const SPAN: u32 = X1 - X0;
const MID: u32 = (Y0 + Y1) / 2;

/// O limite de paridade GPU↔CPU em níveis de byte — **o mesmo dos irmãos do Duotone e do Color
/// Adjust, e pela mesma razão**: a raiz cúbica do OKLab é `pow(x, 1/3)` no dispositivo e `cbrt` no
/// Rust, e o ida-e-volta atravessa duas transferências sRGB.
const MAX_DELTA: i32 = 4;

/// As duas pontas que o irmão do Duotone usa — reusadas VERBATIM, porque o gate de subsunção
/// compara os dois efeitos e um par diferente não compararia nada.
const SHADOW: [f32; 4] = [0.10, 0.12, 0.35, 1.0];
const HIGHLIGHT: [f32; 4] = [1.0, 0.86, 0.62, 1.0];

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ramp_byte(x: u32) -> u8 {
    (255.0 * (x - X0) as f32 / (SPAN - 1) as f32) as u8
}

fn source(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in Y0..Y1 {
        for x in X0..X1 {
            let o = ((y * W + x) * 4) as usize;
            let v = ramp_byte(x);
            bytes[o..o + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    make_src(gpu, W, H, &bytes)
}

/// Um degrau de Gradient Map, montado **pela porta única do componente**.
///
/// ⚠️ `ramp_for_device` é a MESMA função que o `resolve_ops` do shell chama — se o gate ordenasse
/// por conta própria, ele mediria uma rampa que o produto nunca desenha.
fn gm(stops: &[([f32; 4], f32)], mode: u8, opacity: f32) -> FxOpGpu {
    let mut op = FxOp::new(FxOp::GRADIENT_MAP);
    op.stop_count = stops.len() as u8;
    for (i, (colour, pos)) in stops.iter().enumerate() {
        op.stops[i] = *colour;
        op.stop_pos[i] = *pos;
    }
    let (packed_stops, packed_pos, count) = op.ramp_for_device();
    FxOpGpu {
        kind: FxOp::GRADIENT_MAP,
        opacity,
        mode,
        stops: packed_stops,
        stop_pos: packed_pos,
        stop_count: count,
        ..blank()
    }
}

fn duotone(shadow: [f32; 4], highlight: [f32; 4], opacity: f32) -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::DUOTONE,
        tint: shadow,
        tint_b: highlight,
        opacity,
        ..blank()
    }
}

fn blank() -> FxOpGpu {
    FxOpGpu {
        kind: FxOp::BLUR,
        sigma_px: 0.0,
        offset_px: [0, 0],
        tint: [0.0; 4],
        tint_b: [0.0; 4],
        opacity: 1.0,
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

fn render(
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

// ── O ORÁCULO ─────────────────────────────────────────────────────────────────────────────────

/// **A régua desta pilha** — o `L` do OKLab, calculado pela OUTRA implementação (`ph2d-color`).
fn oklab_l(srgb: [u8; 3]) -> f32 {
    let lin = LinearRgba::new(
        srgb_to_linear_byte(srgb[0]),
        srgb_to_linear_byte(srgb[1]),
        srgb_to_linear_byte(srgb[2]),
        1.0,
    );
    OklabColor::from_linear(lin).l
}

/// **A rampa do PAINTER, amostrada em `t` exacto.** A tabela dele tem 256 entradas; a nossa lei é
/// contínua, então o oráculo interpola LINEARMENTE entre as duas entradas vizinhas — e para uma
/// rampa que já é linear por trechos isso reproduz a lei exacta em todo `t` que não caia dentro da
/// janela de `1/255` que contém um stop.
///
/// ⚠️ **Por isso a fixture usa vãos LARGOS** (≥ 0,25): num vão estreito a inclinação da rampa é
/// enorme e a quantização da tabela dominaria o limite — o gate mediria a tabela, não a lei.
fn painter_ramp(stops: &[([f32; 4], f32)], interp: GradientInterp, t: f32) -> [f32; 3] {
    let params = GradientMapParams {
        stops: stops
            .iter()
            .map(|(c, pos)| ColorStop {
                offset: *pos,
                color: [
                    (c[0] * 255.0).round() as u8,
                    (c[1] * 255.0).round() as u8,
                    (c[2] * 255.0).round() as u8,
                    255,
                ],
            })
            .collect(),
        interpolation: interp,
    };
    let lut = gradient_map_lut(&params);
    let f = t.clamp(0.0, 1.0) * 255.0;
    let i0 = f.floor() as usize;
    let i1 = (i0 + 1).min(255);
    let frac = f - i0 as f32;
    core::array::from_fn(|c| lut[i0][c] + (lut[i1][c] - lut[i0][c]) * frac)
}

// ── (1) A SUBSUNÇÃO — o gate que carrega a wave ────────────────────────────────────────────────

/// **Dois stops nas pontas SÃO o Duotone, ao byte.**
///
/// Não há épsilon aqui de propósito: os dois caminhos rodam no MESMO dispositivo, sobre a MESMA
/// fixture, com a MESMA régua e a MESMA aritmética de mistura — a única diferença é de onde as duas
/// cores vieram (`tint`/`tint_b` contra `stops[0]`/`stops[1]`). Qualquer divergência é um defeito,
/// não ruído de ponto flutuante.
///
/// ⚠️ **E a opacidade entra no laço** porque o `k` do Duotone é `opacity × lerp(tint.a, tint_b.a)`:
/// se o Gradient Map lesse a força de outro lugar, um valor não-cheio separaria os dois.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn two_stops_at_the_ends_are_the_duotone_to_the_byte() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    for opacity in [1.0_f32, 0.6, 0.25] {
        let want = render(
            &gpu,
            &mut pass,
            &src,
            &[duotone(SHADOW, HIGHLIGHT, opacity)],
        );
        let got = render(
            &gpu,
            &mut pass,
            &src,
            &[gm(&[(SHADOW, 0.0), (HIGHLIGHT, 1.0)], 0, opacity)],
        );
        let differing = want.iter().zip(&got).filter(|(a, b)| a != b).count();
        assert_eq!(
            differing,
            0,
            "opacity {opacity}: {differing} de {} bytes diferem — o Gradient Map de dois stops \
             DEIXOU de ser o Duotone, e o artista tem duas fichas que desenham coisas diferentes \
             com os mesmos números",
            want.len()
        );
    }
    eprintln!(
        "[fx_gm] subsunção do Duotone: 0 de {} bytes diferem",
        W * H * 4
    );
}

// ── (2) A LEI DA RAMPA é a que o app já ship ──────────────────────────────────────────────────

/// **A rampa de N stops é a do `gradient_map_lut`** — a implementação de CPU de outra crate.
///
/// A comparação é feita em `t`, que é onde as duas metades do app concordam; o `t` de cada pixel
/// sai da régua DESTA pilha (OKLab), que é a divergência declarada no cabeçalho.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_ramp_is_the_law_the_painter_already_ships() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    // Vãos largos, e um stop FORA das pontas em cada rampa — é entre stops que uma rampa de N
    // pontos pode estar errada, e um conjunto que só tem pontas é indistinguível do Duotone.
    let ramps: [&[([f32; 4], f32)]; 3] = [
        &[(SHADOW, 0.0), ([0.9, 0.2, 0.2, 1.0], 0.5), (HIGHLIGHT, 1.0)],
        &[
            ([0.0, 0.0, 0.0, 1.0], 0.0),
            ([0.2, 0.5, 0.9, 1.0], 0.35),
            ([0.95, 0.95, 0.2, 1.0], 0.7),
            ([1.0, 1.0, 1.0, 1.0], 1.0),
        ],
        // Uma rampa que NÃO cobre `[0,1]`: fora do vão os extremos estendem PLANO, e é a mesma
        // escolha do Painter (e do Photoshop) — uma rampa curta não inventa cor além das pontas.
        &[([0.1, 0.6, 0.3, 1.0], 0.3), ([0.95, 0.4, 0.1, 1.0], 0.75)],
    ];
    let mut worst = 0i32;
    for (r, stops) in ramps.iter().enumerate() {
        for mode in [0u8, 1] {
            let interp = if mode == 1 {
                GradientInterp::Smooth
            } else {
                GradientInterp::Linear
            };
            let out = render(&gpu, &mut pass, &src, &[gm(stops, mode, 1.0)]);
            for x in X0..X1 {
                let v = ramp_byte(x);
                let want_lin = painter_ramp(stops, interp, oklab_l([v, v, v]));
                let got = px(&out, x, MID);
                for c in 0..3 {
                    let want = linear_to_srgb_byte(want_lin[c]);
                    let d = i32::from(got[c]) - i32::from(want);
                    worst = worst.max(d.abs());
                    assert!(
                        d.abs() <= MAX_DELTA,
                        "rampa {r} modo {mode} coluna {x} (cinza {v}): a GPU deu {got:?} e o \
                         Painter dá {want} no canal {c} (delta {d}) — as duas metades do app \
                         discordam sobre a MESMA rampa"
                    );
                }
            }
        }
    }
    eprintln!("[fx_gm] pior delta GPU vs Painter-CPU: {worst} nivel(is) de byte");
}

/// **O modo Smooth suaviza ENTRE stops, não a rampa inteira** — e é isso que o distingue de uma
/// curva global aplicada ao `t`.
///
/// O oráculo é a DERIVADA nos stops: com smoothstep por trecho a rampa chega em cada stop interno
/// com inclinação zero (um patamar visível), o que uma suavização global não produz.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_smooth_mode_flattens_the_ramp_at_every_stop_not_only_at_the_ends() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    // Um stop no MEIO, e vãos largos dos dois lados.
    let stops: &[([f32; 4], f32)] = &[
        ([0.0, 0.0, 0.0, 1.0], 0.0),
        ([0.5, 0.5, 0.5, 1.0], 0.5),
        ([1.0, 1.0, 1.0, 1.0], 1.0),
    ];
    // A coluna cuja luminância cai mais perto do stop do meio.
    let mid_col = (X0..X1)
        .min_by(|a, b| {
            let (la, lb) = (
                (oklab_l([ramp_byte(*a); 3]) - 0.5).abs(),
                (oklab_l([ramp_byte(*b); 3]) - 0.5).abs(),
            );
            la.total_cmp(&lb)
        })
        .expect("a rampa tem colunas");
    let mut slope = [0f32; 2];
    for (i, mode) in [0u8, 1].iter().enumerate() {
        let out = render(&gpu, &mut pass, &src, &[gm(stops, *mode, 1.0)]);
        let (a, b) = (px(&out, mid_col - 3, MID), px(&out, mid_col + 3, MID));
        slope[i] = (f32::from(b[0]) - f32::from(a[0])).abs();
    }
    assert!(
        slope[1] < slope[0] * 0.5,
        "a inclinação no stop do meio é {:.1} em Linear e {:.1} em Smooth — o Smooth tem de \
         ACHATAR ali (é o que 'suave entre stops' significa); inclinação igual significa que a \
         suavização está no lugar errado, ou não está em lugar nenhum",
        slope[0],
        slope[1]
    );
    eprintln!(
        "[fx_gm] inclinação no stop interno: Linear {:.1} · Smooth {:.1} nivel(is)/6px",
        slope[0], slope[1]
    );
}

// ── A AUTORIA, e os degenerados ───────────────────────────────────────────────────────────────

/// **A ordem em que o artista clicou não muda um byte.** A porta única ordena uma cópia; o
/// documento guarda a ordem de autoria para o punho sob o dedo não trocar de stop no meio do
/// arrasto.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_authoring_order_of_the_stops_does_not_change_a_byte() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let ordered: &[([f32; 4], f32)] = &[
        ([0.0, 0.0, 0.0, 1.0], 0.0),
        ([0.2, 0.5, 0.9, 1.0], 0.35),
        ([0.95, 0.95, 0.2, 1.0], 0.7),
        ([1.0, 1.0, 1.0, 1.0], 1.0),
    ];
    let scrambled: &[([f32; 4], f32)] = &[
        ([0.95, 0.95, 0.2, 1.0], 0.7),
        ([1.0, 1.0, 1.0, 1.0], 1.0),
        ([0.0, 0.0, 0.0, 1.0], 0.0),
        ([0.2, 0.5, 0.9, 1.0], 0.35),
    ];
    let want = render(&gpu, &mut pass, &src, &[gm(ordered, 0, 1.0)]);
    let got = render(&gpu, &mut pass, &src, &[gm(scrambled, 0, 1.0)]);
    let differing = want.iter().zip(&got).filter(|(a, b)| a != b).count();
    assert_eq!(
        differing, 0,
        "{differing} bytes diferem entre a MESMA rampa autorada em duas ordens — arrastar um stop \
         por cima do vizinho passaria a mudar o desenho"
    );
}

/// **Sem stop nenhum a rampa é a do `gradient_sample` do Painter — e ela NÃO é o default de dois
/// stops.** As duas afirmações são uma só, e a segunda é o achado.
///
/// ⚠️ **Eu escrevi que este degenerado *"cai no default"* e a medição derrubou a frase: são 73
/// níveis de byte de diferença.** Não é defeito: são duas leis distintas, e o Painter tem
/// **exactamente as mesmas duas** — o ramo vazio dele devolve `srgb_to_linear(t)` (trata o `t` como
/// um valor de DISPLAY) enquanto dois stops preto→branco linearizam as pontas e devolvem `t` em luz
/// LINEAR. Herdamos as duas verbatim, e é isso que este gate PINA: quem "consertar" um dos ramos
/// para casar com o outro passa a divergir da crate que é o nosso oráculo.
///
/// ⚠️ **Consequência de produto, e é ela que torna o degenerado inofensivo:** `FxOp::new` nasce com
/// DOIS stops e o trilho do painel tem piso em dois (uma rampa com menos de duas pontas não é uma
/// rampa), então `stop_count == 0` não é estado de autoria — é o degenerado bem-definido de um
/// degrau em branco, e existe para o shader nunca ler uma lista vazia.
///
/// ⚠️ **E a rampa preto→branco NÃO é neutra**, sob nenhuma das duas leis: um Gradient Map é um
/// RECOLORIDOR como o Duotone e o Color Overlay, e adicioná-lo muda o desenho (é o default do
/// Photoshop, pelo mesmo motivo — *"a minha arte mapeada na minha rampa"*). O número medido sai no
/// `eprintln`, e o doc-comment do modelo cita este gate em vez de afirmar neutralidade.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn no_stops_is_the_painters_empty_ramp_which_is_not_the_two_stop_default() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    let mut empty = gm(&[], 0, 1.0);
    empty.stop_count = 0;
    let got = render(&gpu, &mut pass, &src, &[empty]);
    // (a) A lei do ramo vazio é a do Painter, dentro do limite de paridade.
    let mut worst_vs_painter = 0i32;
    for x in X0..X1 {
        let v = ramp_byte(x);
        let want_lin = painter_ramp(&[], GradientInterp::Linear, oklab_l([v, v, v]));
        let px_got = px(&got, x, MID);
        for c in 0..3 {
            let want = linear_to_srgb_byte(want_lin[c]);
            let d = i32::from(px_got[c]) - i32::from(want);
            worst_vs_painter = worst_vs_painter.max(d.abs());
            assert!(
                d.abs() <= MAX_DELTA,
                "coluna {x} (cinza {v}): a GPU deu {px_got:?} e o ramo vazio do Painter dá {want}                  no canal {c} (delta {d})"
            );
        }
    }
    // (b) E ela DIFERE do default explícito — o número que a minha frase errada escondia.
    let two_stop = render(
        &gpu,
        &mut pass,
        &src,
        &[gm(
            &[([0.0, 0.0, 0.0, 1.0], 0.0), ([1.0, 1.0, 1.0, 1.0], 1.0)],
            0,
            1.0,
        )],
    );
    let gap = (X0..X1)
        .flat_map(|x| {
            let (a, b) = (px(&got, x, MID), px(&two_stop, x, MID));
            (0..3)
                .map(move |c| (i32::from(a[c]) - i32::from(b[c])).abs())
                .collect::<Vec<_>>()
        })
        .max()
        .unwrap_or(0);
    assert!(
        gap > 32,
        "a rampa vazia e a preto→branco explícita diferem só {gap} nivel(is) — se convergiram, uma          das duas deixou de ser a lei do Painter, e o oráculo desta wave é aquela crate"
    );
    let mid_grey = (X0..X1)
        .min_by_key(|x| i32::from(ramp_byte(*x)).abs_diff(128) as i32)
        .expect("a rampa tem colunas");
    eprintln!(
        "[fx_gm] ramo vazio vs Painter: {worst_vs_painter} nivel(is) · vs default de 2 stops: \
         {gap} nivel(is) · o default recolore: cinza {} entra e {:?} sai",
        ramp_byte(mid_grey),
        px(&two_stop, mid_grey, MID)
    );
}

/// **Cada stop carrega a PRÓPRIA força** (o alfa dele) — irmão exacto do gate do Duotone
/// `each_ramp_end_carries_its_own_strength`. Sem isto o alfa de um stop do meio seria um knob morto,
/// e um knob morto ensina o artista a desconfiar dos vivos.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn each_stop_carries_its_own_strength() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let src = source(&gpu);
    // O stop do MEIO é transparente (força zero); as pontas são opacas.
    let stops: &[([f32; 4], f32)] = &[(SHADOW, 0.0), ([0.9, 0.2, 0.2, 0.0], 0.5), (HIGHLIGHT, 1.0)];
    let out = render(&gpu, &mut pass, &src, &[gm(stops, 0, 1.0)]);
    let plain = render(&gpu, &mut pass, &src, &[]);
    // A coluna cuja luminância cai no stop do meio tem de estar (quase) intocada; a da ponta, não.
    let mid_col = (X0..X1)
        .min_by(|a, b| {
            (oklab_l([ramp_byte(*a); 3]) - 0.5)
                .abs()
                .total_cmp(&(oklab_l([ramp_byte(*b); 3]) - 0.5).abs())
        })
        .expect("a rampa tem colunas");
    let (at_mid, src_mid) = (px(&out, mid_col, MID), px(&plain, mid_col, MID));
    let untouched: i32 = (0..3)
        .map(|c| (i32::from(at_mid[c]) - i32::from(src_mid[c])).abs())
        .max()
        .unwrap_or(0);
    let (at_end, src_end) = (px(&out, X1 - 1, MID), px(&plain, X1 - 1, MID));
    let moved: i32 = (0..3)
        .map(|c| (i32::from(at_end[c]) - i32::from(src_end[c])).abs())
        .max()
        .unwrap_or(0);
    assert!(
        untouched <= 8,
        "o stop de força ZERO moveu o texel em {untouched} nivel(is) — o alfa por-stop não é lido"
    );
    assert!(
        moved > 40,
        "a ponta opaca só moveu {moved} nivel(is) — a fixture não contém o fenômeno, então o gate \
         acima não prova nada"
    );
    eprintln!("[fx_gm] força por-stop: no zero {untouched} · na ponta opaca {moved}");
}

/// **O Gradient Map nunca move a COBERTURA** — ele recolore, e o alfa que entra é o que sai. Irmão
/// exacto do gate do Duotone, e a metade que o separa do Luma to Alpha.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_gradient_map_never_moves_coverage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_gm] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // Fixture de BORDA: cor constante, alfa em rampa — é onde uma lei que mexesse no alfa apareceria.
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for y in Y0..Y1 {
        for x in X0..X1 {
            let o = ((y * W + x) * 4) as usize;
            bytes[o..o + 4].copy_from_slice(&[160, 160, 160, ramp_byte(x)]);
        }
    }
    let src = make_src(&gpu, W, H, &bytes);
    let out = render(
        &gpu,
        &mut pass,
        &src,
        &[gm(
            &[(SHADOW, 0.0), ([0.9, 0.2, 0.2, 1.0], 0.5), (HIGHLIGHT, 1.0)],
            0,
            1.0,
        )],
    );
    for y in Y0..Y1 {
        for x in X0..X1 {
            assert_eq!(
                px(&out, x, y)[3],
                ramp_byte(x),
                "o alfa mudou em ({x},{y}) — um recoloridor não tem voto sobre a cobertura"
            );
        }
    }
}
