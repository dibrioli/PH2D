//! **OS MODOS DE QUEDA** — o Inner Shadow deslocado e os dois modos do Glow.
//!
//! Arquivo próprio porque o `fx_stack_kinds_gpu.rs` está perto do teto de LOC e porque o assunto é
//! coeso: *o que "perto da borda" quer dizer*, e o que muda quando o degrau é DESLOCADO.
//!
//! Rodar: `cargo test -p ph2d-render --test fx_stack_modes_gpu -- --ignored`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{
    SLOPE, make_src, oblique_segments, oblique_signed, oblique_source, readback, try_headless_gpu,
};

const W: u32 = 96;
const H: u32 = 96;
/// A banda, em texels.
const BAND: f32 = 16.0;

fn one(kind: u8, sigma: f32, tint: [f32; 4], off: [i32; 2], mode: u8) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px: sigma,
        offset_px: off,
        tint,
        opacity: 1.0,
        mode,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
    }
}

/// A luminância MÉDIA por profundidade (distância para DENTRO da aresta), em passos de 1 texel.
///
/// Só texels totalmente cobertos entram: a fileira do contorno tem alfa parcial e a composição
/// sobre o fundo a moveria, o que mediria a moldura em vez do efeito.
fn depth_profile(px: &[u8], max_depth: usize) -> Vec<f64> {
    let mut acc = vec![(0.0f64, 0u32); max_depth + 1];
    for y in 3..H - 3 {
        for x in 3..W - 3 {
            let d = oblique_signed(x, y);
            if d < 0.0 || d > max_depth as f64 {
                continue;
            }
            let o = ((y * W + x) * 4) as usize;
            if px[o + 3] < 250 {
                continue;
            }
            let l = 0.299 * f64::from(px[o])
                + 0.587 * f64::from(px[o + 1])
                + 0.114 * f64::from(px[o + 2]);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let b = d.round() as usize;
            acc[b].0 += l;
            acc[b].1 += 1;
        }
    }
    acc.iter()
        .map(|(s, n)| if *n == 0 { -1.0 } else { s / f64::from(*n) })
        .collect()
}

fn inner_shadow(gpu: &ph2d_gpu::GpuContext, off: [i32; 2]) -> Vec<u8> {
    let src = oblique_source(gpu, W, H);
    let dst = make_output_texture(gpu, W, H);
    let mut pass = FxStackPass::new(gpu);
    pass.run(
        gpu,
        &src,
        &dst,
        W,
        H,
        &[one(
            FxOp::INNER_SHADOW,
            BAND,
            [0.0, 0.0, 0.0, 1.0],
            off,
            FxOp::MODE_CONTOUR,
        )],
        &oblique_segments(W),
    );
    readback(gpu, &dst, W, H)
}

/// **UMA SOMBRA INTERNA É MAIS ESCURA NA BORDA — inclusive deslocada.**
///
/// ⚠️ **O defeito que este gate fecha, no número que o expôs.** Em modo Contour a força era
/// `1 − smoothstep(0, w, dist)` com `dist` SEM SINAL, então um texel cujo ponto amostrado caía FORA
/// da forma voltava a ter distância grande e a sombra DESVANECIA justamente do lado onde devia
/// estar saturada: a banda **descolava do contorno**. Medido com deslocamento 8 (luminância por
/// profundidade, tinta crua ≈ 180): `110 96 81 64 45 24 3 9 31 52 …` — o ponto mais escuro ficava
/// **7 texels dentro** e a borda saía **3,6× mais clara** que ele. Com o sinal: `0 0 0 0 0 0 0 9
/// 31 52 …`.
///
/// ⚠️ **A fixture varre TRÊS deslocamentos, e o zero é metade do gate:** sem deslocamento
/// `sdist == dist` para todo texel que contribui, então a lei antiga e a nova coincidem ali — se o
/// gate só medisse o zero, ele ficaria verde sobre o defeito inteiro.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_inner_shadow_is_darkest_at_the_edge_even_when_offset() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_modes] sem adapter — skip");
        return;
    };
    for off in [0i32, 8, 16] {
        let px = inner_shadow(&gpu, [0, off]);
        let prof = depth_profile(&px, 25);
        let vals: Vec<String> = prof
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, v)| **v >= 0.0)
            .map(|(i, v)| format!("{i}:{v:.0}"))
            .collect();
        eprintln!(
            "[fx_stack_modes] inner shadow off={off}: {}",
            vals.join(" ")
        );

        let live: Vec<(usize, f64)> = prof
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, v)| **v >= 0.0)
            .map(|(i, v)| (i, *v))
            .collect();
        assert!(live.len() > 15, "perfil curto demais: {}", live.len());
        let darkest = live
            .iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .expect("perfil não vazio");
        assert!(
            darkest.0 <= 1,
            "off={off}: o ponto mais escuro está a {} texels da borda (luminância {:.0}) — a \
             sombra descolou do contorno, que é a assinatura da distância SEM SINAL",
            darkest.0,
            darkest.1
        );
        // …e ela clareia dali para dentro, sem voltar a escurecer.
        for w in live.windows(2) {
            assert!(
                w[1].1 >= w[0].1 - 1.5,
                "off={off}: a sombra volta a ESCURECER da profundidade {} ({:.0}) para {} ({:.0})",
                w[0].0,
                w[0].1,
                w[1].0,
                w[1].1
            );
        }
    }
}

/// **O DESLOCAMENTO EMPURRA A SOMBRA: ele satura tantos texels quanto o seu comprimento.**
///
/// É o que o artista de facto controla com o par Offset — e é a metade que o gate acima não afirma
/// (lá basta ser monótona, e uma sombra que ignorasse o deslocamento também seria). Medido: sem
/// deslocamento nenhum texel satura, com 8 saturam ~7 e com 16 saturam ~15.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_offset_saturates_the_shadow_as_far_as_it_pushes() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_modes] sem adapter — skip");
        return;
    };
    let saturated = |off: i32| -> usize {
        let px = inner_shadow(&gpu, [0, off]);
        depth_profile(&px, 25)
            .iter()
            .skip(1)
            .take_while(|v| **v >= 0.0 && **v < 2.0)
            .count()
    };
    // ⚠️ **O deslocamento é um VETOR e a profundidade é PERPENDICULAR à aresta** — contra uma
    // aresta oblíqua o que satura é a PROJEÇÃO. A minha primeira expectativa era `off` cru e o
    // gate nasceu vermelho com 14 onde eu pedia 16: o produto estava certo e o oráculo é que tinha
    // esquecido o cosseno. `cos(atan(0,43874)) = 0,9157`.
    let projected = |off: f64| off / (1.0 + SLOPE * SLOPE).sqrt();
    let (a, b, c) = (saturated(0), saturated(8), saturated(16));
    eprintln!(
        "[fx_stack_modes] texels saturados: off=0 {a} · off=8 {b} (previsto {:.1}) · off=16 {c} \
         (previsto {:.1})",
        projected(8.0),
        projected(16.0)
    );
    assert_eq!(
        a, 0,
        "sem deslocamento a sombra não pode saturar ({a} texels): o máximo dela é a borda"
    );
    for (off, got) in [(8.0f64, b), (16.0, c)] {
        let want = projected(off);
        assert!(
            (want - got as f64).abs() <= 1.5,
            "o deslocamento {off:.0} tinha de saturar a própria PROJEÇÃO ({want:.1} texels) e \
             saturou {got}"
        );
    }
}

/// **O HALO DO CONTOUR É FUNÇÃO DA DISTÂNCIA E MAIS NADA; O DO PROXIMITY NÃO É.**
///
/// É a diferença inteira entre os dois modos, e é a razão de o Glow passar a ter a escolha.
///
/// ⚠️ **A minha primeira redação deste gate contava a história ERRADA** — copiei o enredo do irmão
/// de DENTRO (*"a reentrância quase não acende"*) sem medir. Num halo EXTERNO o sinal se inverte:
/// medido a 3,5 texels da borda, o Proximity dá **156 na reentrância contra 110 na aresta reta**
/// (ali há forma dos DOIS lados, então há mais silhueta borrada por perto) e **45 numa quina
/// CONVEXA** (ali há menos). Quem fica no escuro num halo externo é a PONTA, não o vão — 111
/// níveis de espalhamento à MESMA distância.
///
/// O que sobrevive à medição é a lei, e ela é exata: à mesma distância, o Contour dá o mesmo halo
/// nos três sítios — 202, 202, 202, ao nível. A fixture é uma CRUZ (quinas reentrantes E convexas) com a
/// SILHUETA entregue, e as três sondas ficam à mesma distância da borda; sem isso o gate compararia
/// distâncias em vez da lei.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_contour_glow_is_a_function_of_distance_alone_and_proximity_is_not() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_modes] sem adapter — skip");
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
                bytes[o..o + 4].copy_from_slice(&[235, 175, 60, 255]);
            }
        }
    }
    let src = make_src(&gpu, cw, ch, &bytes);
    let mut pass = FxStackPass::new(&gpu);
    // ⚠️ **A cruz entrega a SILHUETA**, e sem isso o gate não mede a lei: no caminho do raster o
    // campo é propagado por JFA e erra JUSTAMENTE numa quina (medido — a quina lia 215 contra 255
    // da reta, um vão de 40 que é da propagação, não do modo). Com o pé exato as duas sondas ficam
    // as duas a 3,5 texels da borda, que é o que o gate afirma.
    let seg = |a: (f32, f32), b: (f32, f32)| [a.0, a.1, b.0, b.1];
    let (lo, hi) = (14.0f32, 114.0f32);
    let (fv0, fv1, fh0, fh1) = (v0 as f32, v1 as f32, h0 as f32, h1 as f32);
    let cross = [
        seg((lo, fh0), (fv0, fh0)),
        seg((fv0, fh0), (fv0, lo)),
        seg((fv0, lo), (fv1, lo)),
        seg((fv1, lo), (fv1, fh0)),
        seg((fv1, fh0), (hi, fh0)),
        seg((hi, fh0), (hi, fh1)),
        seg((hi, fh1), (fv1, fh1)),
        seg((fv1, fh1), (fv1, hi)),
        seg((fv1, hi), (fv0, hi)),
        seg((fv0, hi), (fv0, fh1)),
        seg((fv0, fh1), (lo, fh1)),
        seg((lo, fh1), (lo, fh0)),
    ];
    // A sonda: 3 px FORA da forma, na quina reentrante e no meio de uma aresta reta.
    let probe = |pass: &mut FxStackPass, mode: u8| -> (i32, i32, i32) {
        let dst = make_output_texture(&gpu, cw, ch);
        pass.run(
            &gpu,
            &src,
            &dst,
            cw,
            ch,
            &[one(FxOp::GLOW, 12.0, [1.0, 1.0, 1.0, 1.0], [0, 0], mode)],
            &cross,
        );
        let px = readback(&gpu, &dst, cw, ch);
        let at = |x: u32, y: u32| -> i32 { i32::from(px[((y * cw + x) * 4 + 3) as usize]) };
        // Quina reentrante (na diagonal do "L") · aresta reta do braço vertical · quina CONVEXA
        // (a ponta do braço). As três a 3,5 texels da borda, as três FORA da forma.
        (at(v1 + 3, h1 + 3), at(v1 + 3, h0 - 14), at(v0 - 3, 14 - 3))
    };
    // ⚠️ **A premissa, declarada — porque eu a violei.** A minha primeira sonda "de aresta reta"
    // caía a (v0+20, h1+3), que está DENTRO do braço vertical: ela lia o alfa da FORMA (255) e não
    // o do halo, e o gate acusava 53 níveis de diferença sobre um produto correto. O controle
    // atropelado pelo experimento, outra vez.
    for (x, y, name) in [
        (v1 + 3, h1 + 3, "quina reentrante"),
        (v1 + 3, h0 - 14, "aresta reta"),
        (v0 - 3, 14 - 3, "quina convexa"),
    ] {
        assert_eq!(
            bytes[((y * cw + x) * 4 + 3) as usize],
            0,
            "a sonda da {name} caiu DENTRO da forma — ela mediria a cobertura, não o halo"
        );
    }
    let (px_notch, px_edge, px_tip) = probe(&mut pass, FxOp::MODE_PROXIMITY);
    let (cn_notch, cn_edge, cn_tip) = probe(&mut pass, FxOp::MODE_CONTOUR);
    eprintln!(
        "[fx_stack_modes] alfa do halo a 3,5 texels — PROXIMITY reentrancia {px_notch} · reta \
         {px_edge} · ponta {px_tip}  ||  CONTOUR {cn_notch} · {cn_edge} · {cn_tip}"
    );
    assert!(
        cn_edge > 150,
        "o Contour tinha de acender a aresta reta ({cn_edge})"
    );
    // A LEI do Contour: à mesma distância, o MESMO halo nos três sítios.
    let cont_spread = [cn_notch, cn_edge, cn_tip]
        .iter()
        .max()
        .copied()
        .unwrap_or(0)
        - [cn_notch, cn_edge, cn_tip]
            .iter()
            .min()
            .copied()
            .unwrap_or(0);
    assert!(
        cont_spread <= 6,
        "no Contour os três sítios estão à mesma distância e o halo varia {cont_spread} níveis \
         ({cn_notch} · {cn_edge} · {cn_tip}) — a banda deixou de seguir o CONTORNO"
    );
    // E o controle: no Proximity ele varia MUITO à mesma distância. Se os dois concordassem, um
    // dos modos não existiria.
    let prox_spread = [px_notch, px_edge, px_tip]
        .iter()
        .max()
        .copied()
        .unwrap_or(0)
        - [px_notch, px_edge, px_tip]
            .iter()
            .min()
            .copied()
            .unwrap_or(0);
    assert!(
        prox_spread > 40,
        "o Proximity varia só {prox_spread} níveis à mesma distância ({px_notch} · {px_edge} · \
         {px_tip}) — sem essa variação a escolha não estaria a escolher nada"
    );
    // E o motivo pelo qual o artista quer o Contour: a PONTA é onde o halo por proximidade morre.
    assert!(
        px_tip < px_edge && px_tip + 40 < cn_tip,
        "a quina convexa tinha de ser o ponto FRACO do Proximity ({px_tip} contra {px_edge} na \
         reta e {cn_tip} no Contour)"
    );
}

/// **O HALO DO CONTOUR PÁRA EXATAMENTE NA LARGURA AUTORADA.**
///
/// A queda vale zero em `w` por construção, e é isso que autoriza o `op_reach` deste caso a ser `w`
/// em vez do suporte `3σ` do borrão. Um alcance que mentisse recortaria o halo na borda da textura;
/// um halo que passasse de `w` seria recortado pela margem — os dois lados da mesma afirmação.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_contour_glow_reaches_its_width_and_stops() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_modes] sem adapter — skip");
        return;
    };
    let src = oblique_source(&gpu, W, H);
    let mut pass = FxStackPass::new(&gpu);
    // ⚠️ **Os DOIS caminhos do campo.** Com silhueta o pé é exato; sem ela o campo é semeado da
    // COBERTURA e propagado por JFA — e essa semeadura só acontece se o Glow entrar no
    // `seeds_shell`, que é a casca que dá os dois lados da fronteira. Sem isso o halo externo não
    // teria semente nenhuma e o modo Contour desenharia NADA numa forma com traço (o caso que cai
    // no raster). Um gate que só passasse geometria deixaria essa metade sem prova.
    for (name, geom) in [
        ("com geometria", oblique_segments(W)),
        ("sem geometria", Vec::new()),
    ] {
        let dst = make_output_texture(&gpu, W, H);
        pass.run(
            &gpu,
            &src,
            &dst,
            W,
            H,
            &[one(
                FxOp::GLOW,
                BAND,
                [1.0, 1.0, 1.0, 1.0],
                [0, 0],
                FxOp::MODE_CONTOUR,
            )],
            &geom,
        );
        let px = readback(&gpu, &dst, W, H);
        // Alfa médio a `d` texels FORA da aresta.
        let outward = |d: f64| -> f64 {
            let (mut s, mut n) = (0.0, 0u32);
            for y in 3..H - 3 {
                for x in 3..W - 3 {
                    if (oblique_signed(x, y) + d).abs() > 0.4 {
                        continue;
                    }
                    s += f64::from(px[((y * W + x) * 4 + 3) as usize]);
                    n += 1;
                }
            }
            if n == 0 { -1.0 } else { s / f64::from(n) }
        };
        let (near, mid, edge, beyond) = (
            outward(1.0),
            outward(f64::from(BAND) * 0.5),
            outward(f64::from(BAND) - 1.0),
            outward(f64::from(BAND) + 2.0),
        );
        eprintln!(
            "[fx_stack_modes] halo contour ({name}): 1px {near:.0} · meio {mid:.0} · {}px {edge:.0} · \
         além {beyond:.0}",
            BAND - 1.0
        );
        assert!(
            near > 230.0,
            "{name}: o halo tinha de ser opaco junto à borda ({near:.0})"
        );
        assert!(
            (60.0..200.0).contains(&mid),
            "{name}: no meio da banda o halo tinha de estar a meio caminho ({mid:.0})"
        );
        assert!(
            beyond < 4.0,
            "{name}: além da largura autorada não pode sobrar halo ({beyond:.0}) — o `op_reach` \
         promete `w`"
        );
        assert!(
            edge < mid,
            "{name}: o halo tinha de cair para fora ({edge:.0} vs {mid:.0})"
        );
    }
}

/// **O GLOW EM PROXIMITY É EXATAMENTE O QUE ERA** — a opção nova não pode repintar o que já existe.
///
/// ⚠️ Um Glow salvo antes desta wave carrega `mode = 0`, que É o Proximity, e o `FxOp::new` do
/// Glow arma o mesmo — mas nada disso é garantia de que o CAMINHO não mudou. O que este gate afirma
/// é o caminho: com Proximity o degrau continua a ser um borrão (dois dispatches), e não o campo de
/// distância; o `plan_of` passou a perguntar pelo MODO, e uma condição mal escrita ali levaria todo
/// Glow para a banda em silêncio.
#[test]
fn a_proximity_glow_is_still_a_blur_not_a_distance_band() {
    let glow = |mode: u8| one(FxOp::GLOW, 12.0, [1.0; 4], [0, 0], mode);
    // A MARGEM é a impressão digital observável do caminho: um borrão espalha o suporte do kernel
    // (3σ), uma banda espalha a largura autorada. O oráculo do borrão é o BLUR com o mesmo sigma —
    // não um número escrito à mão, que envelheceria com o `kernel_half`.
    let reach = |op: FxOpGpu| ph2d_render::stack_reach(&[op]).0;
    let blur = reach(one(FxOp::BLUR, 12.0, [0.0; 4], [0, 0], 0));
    let prox = reach(glow(FxOp::MODE_PROXIMITY));
    let cont = reach(glow(FxOp::MODE_CONTOUR));
    eprintln!("[fx_stack_modes] margem: blur {blur} · glow prox {prox} · glow contour {cont}");
    assert_eq!(
        prox, blur,
        "o Glow em Proximity tem de continuar a espalhar como um borrão"
    );
    assert!(
        cont < prox,
        "o Glow em Contour espalha a LARGURA ({cont}), não o suporte ({prox})"
    );
}
