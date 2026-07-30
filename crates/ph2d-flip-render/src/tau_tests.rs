//! Gates da **LEI DA TINTA** (`tau.rs`) — irmão de `binning_tests.rs`, que prova a estrutura de
//! aceleração; aqui prova-se a RESPOSTA.
//!
//! ⚠️ **Pendurado sob `binning::tests` de propósito:** as fixtures (`screen`, `art`,
//! `push_tapered`, `BLACK`) são as do binning, e duplicá-las daria duas cenas para uma pergunta.

use super::*;
use crate::binning::{BinSeg, ScreenSpace, bin_segments};
use crate::pack::FlipGpuData;
use crate::tau::{dab_weight, f_of};

/// ⚠️ **O CONTROLE DE TODOS OS SMOKES** (§8 do handoff): em `hardness = 1` o traço não pode mudar.
///
/// A integral tem de reproduzir a união dura — e a medição diz **onde** ela não reproduz: só a
/// borda, onde a união é um degrau e a integral é uma rampa de largura sub-pixel.
#[test]
fn at_hardness_one_the_integral_is_the_hard_union() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    let g = art(&[
        (&[[10.0, 20.0], [86.0, 44.0]], 11.0, false, BLACK),
        (
            &[[20.0, 50.0], [50.0, 12.0], [78.0, 50.0]],
            7.0,
            false,
            BLACK,
        ),
    ]);
    let bins = bin_segments(&g, &sc, 16);
    let (mut differ, mut worst_band) = (0usize, 0.0f32);
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                continue;
            };
            let list = bins.segs_of(ti);
            let mut i = 0;
            while i < list.len() {
                let sid = list[i].stroke;
                let mut j = i;
                while j < list.len() && list[j].stroke == sid {
                    j += 1;
                }
                let run = &list[i..j];
                let tau_cover = stroke_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                let hard = hard_union_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                if (tau_cover - hard).abs() > 1.0 / 255.0 {
                    differ += 1;
                    // Quão longe da silhueta esse pixel está? (a distância ao contorno)
                    let band = edge_distance(run, &g, &sc, p);
                    worst_band = worst_band.max(band.abs());
                }
                i = j;
            }
        }
    }
    // Medido: a discordância vive numa casca de menos de meio pixel em torno da silhueta.
    assert!(
        worst_band < 0.75,
        "a integral discorda a {worst_band:.3} px da borda em {differ} pixels — nao e' so a borda"
    );
}

/// Distância com sinal do pixel à silhueta do traço (negativa = dentro).
fn edge_distance(run: &[BinSeg], g: &FlipGpuData, sc: &ScreenSpace, p: [f32; 2]) -> f32 {
    let mut best = f32::MAX;
    for seg in run {
        let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
        let sa = sc.point_px(pa.pos);
        let sb = sc.point_px(pb.pos);
        let (t, cx, cy) = closest_on_seg(p, sa, sb);
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        let r = sc.radius_px(pa.width) * (1.0 - t) + sc.radius_px(pb.width) * t;
        best = best.min(dist - r);
    }
    best
}

/// A curva de UM dab **é** a do Painter — conferida contra a função REAL dele, não contra uma
/// reescrita. É esta âncora que faz o motor novo mirar no depósito digital que o Enio pediu.
#[test]
fn the_dab_weight_is_the_painters_falloff() {
    for hi in 0..20 {
        let hardness = hi as f32 / 20.0;
        for di in 0..=100 {
            let dn = di as f32 / 100.0;
            let ours = dab_weight(dn, hardness);
            let theirs = {
                let h = hardness.clamp(0.0, 1.0);
                if h >= 1.0 {
                    f32::from(dn < 1.0)
                } else {
                    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
                    ph2d_painter_brush::Falloff::Smooth.weight(remapped)
                }
            };
            assert_eq!(ours, theirs, "divergiu em dn={dn}, hardness={hardness}");
        }
    }
}

/// **A IDENTIDADE QUE TROCA O PRODUTO POR UMA SOMA** — a peça inteira do motor novo.
/// `1 − Π(1−w_k)` e `1 − exp(−Σ f(d_k))` são o MESMO número; é a segunda forma que é comutativa,
/// sem ordem e sem teto.
#[test]
fn the_sum_of_f_is_the_product_of_the_dabs() {
    for hi in 1..20 {
        let hardness = hi as f32 / 20.0;
        let dns = [0.05f32, 0.2, 0.35, 0.5, 0.65, 0.8, 0.95];
        let product: f32 = dns.iter().map(|d| 1.0 - dab_weight(*d, hardness)).product();
        let prof = crate::tau::DabProfile {
            hardness,
            airbrush: false,
        };
        let sum: f32 = dns.iter().map(|d| f_of(*d, prof)).sum();
        let (a, b) = (1.0 - product, 1.0 - (-sum).exp());
        assert!(
            (a - b).abs() < 2e-6,
            "hardness {hardness}: produto {a} != exp(-soma) {b}"
        );
    }
}

/// **O DEFEITO QUE CUSTOU A SAGA.** Onde o traço cruza a si mesmo há MAIS caminho perto do pixel,
/// então `τ` é estritamente maior — a lei responde ao cruzamento por construção, sem canal
/// lateral, sem teto e sem depth. O motor de hoje integra uma reta fictícia, que não tem
/// cruzamento nenhum para ver.
#[test]
fn the_crossing_carries_more_tau_than_a_single_arm() {
    let (w, h) = (96.0, 96.0);
    let sc = screen(w, h);
    // Um X: as duas pernas se cruzam no centro.
    let g = art(&[(
        &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
        9.0,
        false,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let tau_at = |p: [f32; 2]| {
        let ti = bins.tile_of_pixel(p[0], p[1]).unwrap();
        let list = bins.segs_of(ti);
        crate::tau::stroke_tau(
            list,
            &g,
            &sc,
            crate::tau::StrokeStyle {
                profile: crate::tau::DabProfile {
                    hardness: 0.4,
                    airbrush: false,
                },
                tip: crate::tau::TipShape::Continuous,
            },
            p,
        )
        .map_or(0.0, |ink| ink.tau)
    };
    let crossing = tau_at([48.0, 48.0]);
    let single_arm = tau_at([48.0, 24.0]);
    assert!(
        crossing > single_arm * 1.2,
        "o cruzamento nao acumulou: {crossing:.3} contra {single_arm:.3} de um braço só"
    );
}

/// **A LEI É FATO DO CAMINHO, NÃO DA DENSIDADE DA POLILINHA** — a doença que esta linha curou
/// quatro vezes, agora pinada no PRODUTO e não só na sonda: o MESMO caminho amostrado em 4 e em
/// 40 pontos tem de pintar a mesma imagem.
#[test]
fn the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    // ⚠️ A MESMA GEOMETRIA, amostrada de dois jeitos — a versão fina insere pontos **sobre** as
    // pernas da grossa. Uma senoide reamostrada mediria a CORDA, não a lei: com 4 pontos ela é
    // outro desenho, e o gate estaria medindo geometria diferente e chamando de dependência de
    // amostragem (a fixture que eu escrevi primeiro, e que falhou por isso).
    let coarse: Vec<[f32; 2]> = vec![[12.0, 20.0], [50.0, 46.0], [84.0, 18.0]];
    let mut fine: Vec<[f32; 2]> = Vec::new();
    for leg in coarse.windows(2) {
        for i in 0..12 {
            let t = i as f32 / 12.0;
            fine.push([
                leg[0][0] + (leg[1][0] - leg[0][0]) * t,
                leg[0][1] + (leg[1][1] - leg[0][1]) * t,
            ]);
        }
    }
    fine.push(*coarse.last().unwrap());
    assert_eq!(
        (coarse.len(), fine.len()),
        (3, 25),
        "a fixture mudou de forma"
    );

    // ⚠️ **Dureza MACIA, e a escolha é medida.** Em `hardness = 1` a cobertura é um DEGRAU: a
    // borda é resolvida até um passo de quadratura (~0,06 px), e um pixel cujo centro cai nessa
    // casca **flipa 255 de uma vez**. Um gate de densidade ali mede o degrau, não a lei — medido:
    // pior desvio 254,8/255 com `SUB = 2` e 1,06 com `SUB = 4`, contra os números abaixo em 0,4.
    // (A metade dura tem gate PRÓPRIO: `at_hardness_one_the_integral_is_the_hard_union`.)
    let mut gc = art(&[(&coarse, 9.0, false, BLACK)]);
    let mut gf = art(&[(&fine, 9.0, false, BLACK)]);
    gc.strokes[0].hardness = 0.4;
    gf.strokes[0].hardness = 0.4;
    let bc = bin_segments(&gc, &sc, 16);
    let bf = bin_segments(&gf, &sc, 16);
    let mut worst = 0.0f32;
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let a = walk_pixel(&bc, &gc, &sc, p)[3];
            let b = walk_pixel(&bf, &gf, &sc, p)[3];
            worst = worst.max((a - b).abs());
        }
    }
    // O resíduo que sobra é a GRADE de quadratura (cada segmento arredonda o próprio `n`), não a
    // lei. Medido: sub-nível de byte.
    assert!(
        worst * 255.0 < 1.0,
        "a densidade mexeu na tinta: pior desvio {:.2}/255",
        worst * 255.0
    );
}

/// A cobertura segue o raio **LOCAL**, não uma média do segmento — a pressão é o caso normal, e
/// com raio médio um traço que afina sairia com espessura constante.
#[test]
fn the_coverage_follows_the_local_radius_of_a_tapering_stroke() {
    let (w, h) = (128.0, 64.0);
    let sc = screen(w, h);
    let mut g = FlipGpuData::default();
    push_tapered(&mut g, &[[10.0, 32.0], [118.0, 32.0]], &[24.0, 6.0]);
    let bins = bin_segments(&g, &sc, 16);
    let half_at = |x: f32| -> f32 {
        let mut best = 0.0f32;
        for k in 0..400 {
            let dy = k as f32 * 0.1;
            if walk_pixel(&bins, &g, &sc, [x, 32.0 + dy])[3] > 0.5 {
                best = dy;
            }
        }
        best
    };
    let (thick, thin) = (half_at(20.0), half_at(108.0));
    let ratio = thick / thin.max(1e-3);
    // As larguras autoradas nesses x são ~22,2 e ~7,8 (o lerp), razão ~2,85.
    assert!(
        (2.4..3.4).contains(&ratio),
        "o traço nao afinou: meia-largura {thick:.1} contra {thin:.1} (razao {ratio:.2})"
    );
}

/// ⚠️ **A REGRA DO GP que o `flip.wgsl` documenta:** *um traço a opacity 0,5 não escurece sobre si
/// mesmo*. É por isso que o `opacity` multiplica DEPOIS da cobertura e **nunca entra no `f`** — se
/// entrasse, o cruzamento acumularia opacidade e a regra cairia.
#[test]
fn opacity_scales_the_ink_and_never_darkens_the_crossing() {
    let (w, h) = (96.0, 96.0);
    let sc = screen(w, h);
    let mut g = art(&[(
        &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
        11.0,
        false,
        BLACK,
    )]);
    for p in &mut g.points {
        p.opacity = 0.5;
    }
    let bins = bin_segments(&g, &sc, 16);
    let arm = walk_pixel(&bins, &g, &sc, [48.0, 24.0])[3];
    let crossing = walk_pixel(&bins, &g, &sc, [48.0, 48.0])[3];
    assert!(
        (arm - 0.5).abs() < 1.0 / 255.0,
        "opacity 0,5 nao virou meia tinta: {arm:.4}"
    );
    assert!(
        (crossing - arm).abs() < 1.0 / 255.0,
        "o cruzamento ESCURECEU: {crossing:.4} contra {arm:.4} do braço"
    );
}

/// Uma FILEIRA DE CONTAS de verdade — empacotada pelo `pack_drawing`, porque o `art` acima carimba
/// `arc_len = 0` em todo ponto e o *tip* vive exatamente desse número. Pontos em `x = 20, 40, 60`
/// (arcos 0, 20, 40) com `dot_spacing = 2` sobre largura 10 ⇒ pitch 20 ⇒ **toda conta cai
/// exatamente num PONTO**, que é o regime onde a posse meio-aberta do `bead_range` decide.
fn bead_row(tip: ph2d_flip::StrokeTip, hardness: f32) -> FlipGpuData {
    let mut st = ph2d_flip::FlipStroke::new();
    for x in [20.0_f32, 40.0, 60.0] {
        st.push_point(ph2d_flip::Point {
            pos: ph2d_core::Vec2::new(x, 40.0),
            width: 10.0,
            opacity: 1.0,
            color: ph2d_flip::Rgba::new(0.0, 0.0, 0.0, 1.0),
        });
    }
    st.hardness = hardness;
    st.tip = tip;
    st.dot_spacing = 2.0;
    let mut d = ph2d_flip::FlipDrawing::default();
    d.strokes.push(st);
    crate::pack::pack_drawing(&d)
}

/// 🔴 **UMA CONTA, UM CARIMBO** — a conta que cai numa JUNÇÃO é somada uma vez só.
///
/// ⚠️ **Nenhum gate de paridade CPU×device pode provar isto:** contar duas vezes é um erro que os
/// dois motores cometem IGUAL, então a comparação entre eles fica verde. O oráculo é a lei — a
/// densidade de UM dab a esta distância — e ela é um número fechado.
#[test]
fn the_bead_at_a_joint_is_stamped_once() {
    let sc = screen(80.0, 80.0);
    let g = bead_row(ph2d_flip::StrokeTip::Dots, 0.5);
    let bins = bin_segments(&g, &sc, 16);
    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
    // 3 px acima da conta do MEIO (arco 20, `x = 40`), que é um ponto do traço. `dn = 3/5 = 0,6`;
    // as vizinhas ficam a 20 px ⇒ `dn > 1` ⇒ zero. Logo τ tem de ser o de UM dab.
    let p = [40.0, 43.0];
    let ti = bins
        .tile_of_pixel(p[0], p[1])
        .expect("o ladrilho da conta do meio");
    let tau = crate::tau::stroke_tau(bins.segs_of(ti), &g, &sc, style, p)
        .expect("tinta")
        .tau;
    let um = crate::tau::f_bead_of(0.6, style.profile);
    assert!(
        (tau - um).abs() < 1e-4,
        "a conta da juncao foi carimbada {:.3}x (τ {tau:.4} contra {um:.4} de um dab)",
        tau / um
    );
}

/// 🔴 **A CONTA DA PONTA EXISTE** — o último ponto de um traço aberto não tem segmento seguinte
/// para adotar a conta que cai ali, e sem a exceção do [`crate::dabs::bead_range`] ela desaparece
/// justamente quando o arco total é múltiplo da pitch (o caso de todo traço em números redondos).
#[test]
fn the_bead_at_the_tip_of_the_stroke_is_stamped() {
    let sc = screen(80.0, 80.0);
    let g = bead_row(ph2d_flip::StrokeTip::Dots, 0.5);
    let bins = bin_segments(&g, &sc, 16);
    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
    // 3 px ADIANTE da última conta (arco 40, `x = 60`): só ela alcança (a anterior está a 23 px).
    let p = [63.0, 40.0];
    let ti = bins.tile_of_pixel(p[0], p[1]).expect("o ladrilho da ponta");
    let tau = crate::tau::stroke_tau(bins.segs_of(ti), &g, &sc, style, p)
        .expect("a conta da ponta nao carimbou nada")
        .tau;
    let um = crate::tau::f_bead_of(0.6, style.profile);
    assert!(
        (tau - um).abs() < 1e-4,
        "a conta da ponta nao e um dab: τ {tau:.4} contra {um:.4}"
    );
}

/// 🔴 **UM PONTILHADO NÃO É UMA LINHA** — e o vão é onde a prova mora. Sem isto um
/// `TipShape::of` que devolvesse sempre `Continuous` passaria em todo gate de valor acima.
#[test]
fn the_dotted_row_has_gaps_where_the_full_line_has_ink() {
    let sc = screen(80.0, 80.0);
    let vao = [30.0, 40.0]; // meio caminho entre as contas de `x = 20` e `x = 40`
    let tau_at = |g: &FlipGpuData| {
        let bins = bin_segments(g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        bins.tile_of_pixel(vao[0], vao[1])
            .and_then(|ti| crate::tau::stroke_tau(bins.segs_of(ti), g, &sc, style, vao))
            .map_or(0.0, |ink| ink.tau)
    };
    let cheia = tau_at(&bead_row(ph2d_flip::StrokeTip::Continuous, 0.5));
    let contas = tau_at(&bead_row(ph2d_flip::StrokeTip::Dots, 0.5));
    assert!(cheia > 1.0, "a linha cheia nao pintou o vao: τ {cheia:.4}");
    assert_eq!(
        contas, 0.0,
        "o vao entre contas recebeu tinta: τ {contas:.4}"
    );
}

/// 🔴 **UM CARIMBO QUADRADO É UM QUADRADO** — a QUINA dele tem tinta, e é ela que a janela da
/// quadratura tem de alcançar.
///
/// ⚠️ **Este gate existe porque a paridade CPU×device NÃO pode achar o defeito.** A janela do pixel
/// era um DISCO de raio `rmax`, e a quina de um quadrado fica a `r√2` — então os dois motores
/// perdiam a quina IGUAL, e a comparação entre eles seria verde. O que a denunciou foi o defeito
/// cair **em cima** da fronteira `disc <= 0`, onde a GPU contrai em FMA e o ulp discordava: um
/// sintoma de precisão apontando para um buraco de geometria.
///
/// O ponto de prova é a diagonal a `0,8 r` em cada eixo: `dn` de QUADRADO é 0,8 (tinta), `dn` de
/// DISCO é 1,13 (nada) — então o gate também é o discriminante entre as duas formas de conta.
///
/// ⚠️ **E é a quina de TRÁS, o que não é detalhe de fixture.** A janela da quadratura é clampada ao
/// SEGMENTO, e a conta pertence ao segmento que COMEÇA nela; um pixel adiante do começo mantém a
/// janela aberta mesmo com alcance curto (a diagonal foi salva pelo alargamento de uma conta),
/// enquanto o pixel ATRÁS colapsa `t1 ≤ t0` e o segmento é descartado inteiro. Medido: com a quina
/// da frente a mutação do `dab_reach` **passa**; com a de trás ela sangra. *O lado é parte da
/// fixture.*
#[test]
fn the_square_bead_is_a_square_not_a_disc() {
    let sc = screen(80.0, 80.0);
    // A conta do meio está em `(40, 40)` com `r = 5`, e o segmento que a possui vai para a
    // DIREITA; a diagonal a 4 px para a esquerda e para cima.
    let quina = [36.0, 44.0];
    let tau_at = |g: &FlipGpuData| {
        let bins = bin_segments(g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        bins.tile_of_pixel(quina[0], quina[1])
            .and_then(|ti| crate::tau::stroke_tau(bins.segs_of(ti), g, &sc, style, quina))
            .map_or(0.0, |ink| ink.tau)
    };
    let quadrado = tau_at(&bead_row(ph2d_flip::StrokeTip::Squares, 0.5));
    let disco = tau_at(&bead_row(ph2d_flip::StrokeTip::Dots, 0.5));
    let esperado = crate::tau::f_bead_of(
        0.8,
        crate::tau::DabProfile {
            hardness: 0.5,
            airbrush: false,
        },
    );
    assert!(
        (quadrado - esperado).abs() < 1e-4,
        "a quina do quadrado nao recebeu o carimbo: τ {quadrado:.4} contra {esperado:.4} \
         (a janela da quadratura alcanca `r√2`?)"
    );
    assert_eq!(
        disco, 0.0,
        "a mesma quina recebeu tinta com conta REDONDA: τ {disco:.4} (a fixture nao distingue as \
         duas formas)"
    );
}

// ————————————————— a ANTIDERIVADA: medir o risco antes de construir —————————————————

/// `H(y, u) = ∫₀^u f(√(y² + v²)) dv` — a antiderivada universal da §21.5, por quadratura FINA.
///
/// ⚠️ Isto **não é** a LUT: é o ORÁCULO dela. A pergunta desta sonda é a do risco 1 — a substituição
/// `s = r·u` supõe `r` CONSTANTE no segmento, e um traço de pressão viola isso. A resolução da tabela
/// é outra pergunta, e medir as duas juntas não diria qual erra.
fn h_exact(y: f32, u: f32, prof: crate::tau::DabProfile) -> f64 {
    if u == 0.0 {
        return 0.0;
    }
    let n = 4000;
    let (a, b) = (0.0_f64, f64::from(u));
    let h = (b - a) / f64::from(n);
    let mut acc = 0.0_f64;
    for k in 0..n {
        let v = a + (f64::from(k) + 0.5) * h;
        let dn = (f64::from(y) * f64::from(y) + v * v).sqrt();
        acc += f64::from(crate::tau::f_of(dn as f32, prof)) * h;
    }
    acc
}

/// `τ` de um traço via a ANTIDERIVADA, com `r` congelado no MEIO de cada segmento.
fn tau_via_antiderivative(
    run: &[BinSeg],
    g: &FlipGpuData,
    sc: &ScreenSpace,
    prof: crate::tau::DabProfile,
    p: [f32; 2],
    k: u32,
) -> f64 {
    let mut tau = 0.0_f64;
    for seg in run {
        let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
        let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
        let v = [sb[0] - sa[0], sb[1] - sa[1]];
        let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if len <= 1e-6 {
            continue;
        }
        let dir = [v[0] / len, v[1] / len];
        let w = [p[0] - sa[0], p[1] - sa[1]];
        let t_foot = w[0] * dir[0] + w[1] * dir[1];
        let perp = (w[0] * (-dir[1]) + w[1] * dir[0]).abs();
        let (ra, rb) = (sc.radius_px(pa.width), sc.radius_px(pb.width));
        // ⚠️ **A antiderivada é EXATA para `r` constante**, então a cura do risco 1 é SUBDIVIDIR: `k`
        // pedaços com `r` congelado no meio de cada um. Duas leituras por pedaço, contra ~40 amostras
        // da quadratura — o `k` que fecha o erro é o que decide se a wave vale.
        for j in 0..k {
            let (fa, fb) = (j as f32 / k as f32, (j + 1) as f32 / k as f32);
            let (sa_j, sb_j) = (len * fa, len * fb);
            let r = ra + (rb - ra) * (fa + fb) * 0.5;
            let pitch = (crate::tau::PAINTER_SPACING * 2.0 * r).max(0.25);
            let y = perp / r;
            let (u0, u1) = ((sa_j - t_foot) / r, (sb_j - t_foot) / r);
            tau += f64::from(r / pitch) * (h_exact(y, u1, prof) - h_exact(y, u0, prof));
        }
    }
    tau
}

/// 📏 **SONDA — o RISCO 1 da §21.5: a antiderivada supõe `r` constante no segmento.**
///
/// Compara `τ` da quadratura que SHIPA contra `τ` da antiderivada com `r` congelado no meio do
/// segmento, em três regimes. O que decide a wave é a coluna do traço de PRESSÃO.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_whether_the_antiderivative_survives_a_varying_radius() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    let prof = crate::tau::DabProfile {
        hardness: 0.5,
        airbrush: false,
    };
    let style = crate::tau::StrokeStyle {
        profile: prof,
        tip: crate::tau::TipShape::Continuous,
    };
    // ⚠️ **A fixture afilada é REAMOSTRADA, e a 1ª versão desta sonda não era:** com 2 pontos o `r`
    // varia 8× dentro de UM segmento, o que não é regime nenhum do produto — o `resample_smooth`
    // densifica a `0,4 × largura`, ou seja segmentos de ~`0,8r`. Medir no traço de 2 pontos respondia
    // sobre um desenho que o motor nunca vê (o erro saía 74,6 em τ).
    // ⚠️ **O passo é `0,4 × largura` LOCAL — e a 2ª versão desta sonda usava passo UNIFORME**, o que
    // no fim fino dá segmentos de `3,2r` em vez de `0,8r`: ela media uma reamostragem que o produto
    // não produz. É a mesma convenção que o `measure_ribbon_budget` do `neighbors_tests` já usa
    // (`step = 0,8·R` com `R` local), porque é o que o `resample_smooth` faz.
    let densificado = |xs: (f32, f32), ws: (f32, f32)| -> FlipGpuData {
        let (mut pts, mut wds) = (Vec::new(), Vec::new());
        let mut x = xs.0;
        loop {
            let t = (x - xs.0) / (xs.1 - xs.0);
            let wl = ws.0 + (ws.1 - ws.0) * t;
            pts.push([x, 32.0]);
            wds.push(wl);
            if x >= xs.1 - 1e-4 {
                break;
            }
            x = (x + 0.4 * wl).min(xs.1);
        }
        let mut g = FlipGpuData::default();
        push_tapered(&mut g, &pts, &wds);
        g
    };
    println!("  cena                           o pior |Δα| (de 255) por nº de subdivisões");
    for (nome, g) in [
        (
            "reto, largura CONSTANTE",
            art(&[(&[[12.0, 32.0], [84.0, 32.0]], 12.0, false, BLACK)]),
        ),
        ("afilado 24->3, 2 PONTOS (irreal)", {
            let mut g = FlipGpuData::default();
            push_tapered(&mut g, &[[12.0, 32.0], [84.0, 32.0]], &[24.0, 3.0]);
            g
        }),
        (
            "afilado 24->3, REAMOSTRADO",
            densificado((12.0, 84.0), (24.0, 3.0)),
        ),
    ] {
        let bins = bin_segments(&g, &sc, 16);
        let (mut pior_t, mut pior_a, mut n) = ([0.0_f64; 4], [0.0_f64; 4], 0u32);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                // ⚠️ **Fora das TAMPAS.** A quadratura que shipa soma o meio dab do `end_dab` (§13) e
                // a antiderivada não o tem; incluir a ponta mediria a AUSÊNCIA desse termo (≈ F_MAX/2
                // = 8 em τ, o que a 1ª versão desta sonda reportou como 9,0) em vez da substituição.
                let st = g.strokes[0];
                let p0 = sc.point_px(g.points[st.first_point as usize].pos);
                let pn = sc.point_px(g.points[(st.first_point + st.point_count - 1) as usize].pos);
                let r0 = sc.radius_px(g.points[st.first_point as usize].width) + 2.0;
                let rn = sc
                    .radius_px(g.points[(st.first_point + st.point_count - 1) as usize].width)
                    + 2.0;
                let perto = |q: [f32; 2], raio: f32| {
                    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) <= raio * raio
                };
                if perto(p0, r0) || perto(pn, rn) {
                    continue;
                }
                let Some(ink) = crate::tau::stroke_tau(run, &g, &sc, style, p) else {
                    continue;
                };
                for (ki, kk) in [1_u32, 2, 4, 8].into_iter().enumerate() {
                    let alvo = tau_via_antiderivative(run, &g, &sc, prof, p, kk);
                    let (a_q, a_h) = (
                        1.0 - (-f64::from(ink.tau)).exp(),
                        1.0 - (-alvo.max(0.0)).exp(),
                    );
                    pior_t[ki] = pior_t[ki].max((f64::from(ink.tau) - alvo).abs());
                    pior_a[ki] = pior_a[ki].max((a_q - a_h).abs() * 255.0);
                }
                n += 1;
            }
        }
        println!(
            "  {nome:29}  |Δα| k=1 {:6.2}  k=2 {:6.2}  k=4 {:6.2}  k=8 {:6.2}   ({n} px)",
            pior_a[0], pior_a[1], pior_a[2], pior_a[3]
        );
        let _ = pior_t;
    }
}
