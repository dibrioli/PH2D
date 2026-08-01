//! **A TINTA QUE O PERCURSO DERRUBA** — os GATES: o que a lei tem de sustentar, em números que um
//! commit não pode mover em silêncio.
//!
//! ⚠️ **Filho de [`super`] de propósito:** o oráculo é o mesmo `true_area`, e as fixtures (`screen`,
//! `art`, `BLACK`) são as do binning. Um módulo irmão precisaria de uma segunda cópia da referência,
//! e duas referências para uma pergunta é como elas divergem.
//!
//! As SONDAS (`measure_*`, `#[ignore]`) moram no irmão `ink_drop_probes.rs`. ⚠️ **O corte é de
//! RESPONSABILIDADE, não de tamanho:** um gate AFIRMA e uma sonda PERGUNTA — misturados, a saída de
//! uma varredura não diz qual dos dois falhou.

use super::*;
use crate::binning::bin_segments;

/// ⭐ **A CURA DO 3a, PINADA** — o que era um pin do DEFEITO virou o guard da correção.
///
/// A grade da quadratura passou a resolver a **JANELA** em vez do segmento (doc 12 §22.10), e as
/// duas metades que o pin antigo media trocaram de valor: o flanco continua exato e **a tampa
/// deixou de ser zero**.
///
/// ⚠️ **O flanco tem de ser INCLINADO.** Num traço horizontal o `sd` do flanco cai exatamente em
/// ±0,5 — nunca há pixel meio-coberto ali —, então uma fixture horizontal mede `área = 0` e o gate
/// afirma o vazio. É também a razão de os piores pixels das sondas terem caído todos numa TAMPA.
#[test]
fn the_walk_no_longer_drops_the_ink_at_a_cap() {
    let sc = screen(96.0, 96.0);
    let ver = |g: &crate::pack::FlipGpuData, p: [f32; 2]| {
        let bins = bin_segments(g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        let ti = bins.tile_of_pixel(p[0], p[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let real = true_area(run, g, &sc, style.tip, p, 64);
        let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
        (real, cover)
    };
    // (1) O FLANCO segue exato — é o controle que separa *a lei erra na borda* de *a lei erra
    // perto de um EXTREMO*, e sem ele o número da outra metade não diz nada.
    let inclinado = art(&[(&[[20.0, 20.0], [70.0, 62.0]], 9.0, false, BLACK)]);
    let (real_flanco, cover_flanco) = ver(&inclinado, [47.5, 46.5]);
    assert!(
        real_flanco > 0.0 && (cover_flanco - real_flanco).abs() < 2.0 / 255.0,
        "o flanco deveria ser exato: area {real_flanco:.4} vs cover {cover_flanco:.4}"
    );
    // (2) A TAMPA deposita. Antes da cura este pixel media EXATAMENTE zero contra uma área de
    // 0,1028 — 26,21/255 de tinta derrubada.
    let reto = art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]);
    let (real_tampa, cover_tampa) = ver(&reto, [87.5, 41.5]);
    assert!(
        real_tampa > 0.05,
        "a fixture perdeu a tampa: area {real_tampa:.4} (o gate mediria o vazio)"
    );
    assert!(
        cover_tampa > 0.02,
        "⭐ A TINTA DA TAMPA VOLTOU A SER DERRUBADA: cover {cover_tampa:.4} contra area \
         {real_tampa:.4}. A grade voltou a ser ancorada no SEGMENTO?"
    );
}

/// ⭐ **O RESÍDUO QUE A LEI DE ÁREA CRIOU, pinado com o número — e ele NÃO é regressão.**
///
/// A lei de área (§22.7) enxerga um pixel cuja **QUINA** entra na silhueta mesmo com o centro a mais
/// de meio pixel de distância (`sd` até `√2/2`). O `p_eval` — que escolhe onde amostrar o PERFIL —
/// é 1-D ao longo da normal, e a derivação dele (*a parte coberta é `v ∈ [sd − ½, 0]`*) tem
/// intervalo **vazio** quando `sd > ½`: ele pousa FORA da silhueta, `τ` sai 0 e o depósito devolve
/// `None`.
///
/// ⚠️ **Não é regressão:** com a lei antiga esses pixels tinham `edge = 0,5 − sd ≤ 0` e eram
/// descartados no early-out — derrubados do mesmo jeito, só que sem ninguém saber. O que a lei de
/// área fez foi **tornar visível** a discordância entre a cobertura 2-D e a amostra 1-D.
///
/// Medido: **13 pixels de 1115** num zigue-zague de 24 juntas, área ≤ 2,9% (14,94/255).
///
/// ⚠️ **E em 2026-07-31 a pergunta foi FECHADA por medição, não por escolha:** as duas curas que a
/// §22.10 nomeou (capar o alcance dos planos; mover o `p_eval` para dentro) ganharam uma terceira,
/// melhor que as duas — amostrar no **CENTROIDE** da parte coberta, derivado do mesmo polígono da lei
/// de área. Ela foi construída e o oráculo supersampleado a **reprovou**: empate nos flancos, pior
/// nas junções (96,38 contra 61,86/255 de pior caso). Detalhe e tabela em
/// `measure_which_profile_sample_point_is_closer_to_the_truth`.
///
/// ⚠️ **O que aquela medição estabeleceu é maior que o veredito de uma cura:** o pior erro da lei que
/// SHIPA contra a verdade é **22 a 62/255**, e este resíduo vale **≤ 14,94**. *Ele é menor que o erro
/// da aproximação que o curaria.* Enquanto o ponto de amostra for UM ponto, mexer nele troca um
/// artefato pequeno e conhecido por um maior e difuso — a cura de verdade é atacar a aproximação
/// (supersamplear o perfil, ou um segundo tap), que é outra wave, com preço de device próprio.
#[test]
fn the_area_law_can_claim_a_corner_the_profile_cannot_sample_and_this_is_its_number() {
    let sc = screen(96.0, 96.0);
    let g = art(&[(
        &(0..24)
            .map(|k| [20.0 + k as f32 * 2.5, if k % 2 == 0 { 40.0 } else { 52.0 }])
            .collect::<Vec<_>>(),
        6.0,
        false,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
    let p = [17.5_f32, 58.5];
    let ti = bins.tile_of_pixel(p[0], p[1]).expect("dentro");
    let run = bins.segs_of(ti);
    let sl = stroke_silhouette(run, &g, &sc, style.tip, p).expect("tocado");
    // A premissa declarada: o centro está FORA de meio pixel, então só a quina entra.
    assert!(
        sl.sd > 0.5 && sl.sd < core::f32::consts::FRAC_1_SQRT_2,
        "a fixture perdeu o fenomeno: sd = {} (precisa de meio-pixel a raiz(2)/2)",
        sl.sd
    );
    assert!(
        sl.planes.coverage() > 0.01,
        "a lei de area deveria enxergar a quina: {}",
        sl.planes.coverage()
    );
    assert!(
        stroke_deposit(run, &g, &sc, p).is_none(),
        "⭐ O RESIDUO DA QUINA FOI CORRIGIDO — atualize a nota da §22.10 do doc 12 e este gate."
    );
}

/// ⭐ **A BORDA MACIA ENDURECE SOB AUTO-SOBREPOSIÇÃO, E ESTE É O NÚMERO** — o item 5 da fila, medido
/// no percurso.
///
/// ⚠️ **Descrição, não veredito.** Acumular ao passar por cima é o que tinta faz, e é o que o
/// build-up do GIMP entrega; a terceira lei (`Soft` do Krita) é um MODO, não um conserto. O gate
/// existe para que o número não precise ser re-derivado quando a decisão de produto vier.
///
/// A álgebra do percurso é a mesma família: `α = 1 − exp(−τ)` é uma TAXA rumo a um teto, então
/// dobrar `τ` a cada passada encolhe a faixa em que `α` sobe de 10% a 90%.
#[test]
fn a_soft_edge_hardens_when_the_stroke_crosses_itself_and_this_is_its_number() {
    let (w, h) = (128.0_f32, 64.0_f32);
    let sc = screen(w, h);
    let banda = |passadas: u32| -> f32 {
        let mut pts: Vec<[f32; 2]> = vec![[24.0, 32.0]];
        for k in 0..passadas {
            pts.push([if k % 2 == 0 { 104.0 } else { 24.0 }, 32.0]);
        }
        let mut g = art(&[(&pts, 18.0, false, BLACK)]);
        g.strokes[0].hardness = 0.0;
        let bins = bin_segments(&g, &sc, 16);
        let col: Vec<f32> = (0..h as u32)
            .map(|y| crate::binning::walk_pixel(&bins, &g, &sc, [64.5, y as f32 + 0.5])[3])
            .collect();
        let amax = col.iter().copied().fold(0.0_f32, f32::max);
        let cruza = |alvo: f32| -> f32 {
            for y in 0..col.len() - 1 {
                if col[y] < alvo && col[y + 1] >= alvo {
                    return y as f32 + (alvo - col[y]) / (col[y + 1] - col[y]);
                }
            }
            0.0
        };
        cruza(0.9 * amax) - cruza(0.1 * amax)
    };
    let (uma, quinze) = (banda(1), banda(15));
    // A premissa: com o pincel mais macio a borda de UMA passada é larga. Sem isto o gate mede a
    // razão entre dois números duros e não diz nada sobre maciez.
    assert!(
        uma > 3.0,
        "a fixture perdeu a maciez: banda de uma passada {uma:.3} px"
    );
    assert!(
        quinze < uma * 0.6,
        "⭐ O ENDURECIMENTO MUDOU — uma passada {uma:.3} px, quinze {quinze:.3}. Se a terceira lei \
         entrou, atualize a nota da §22.11 do doc 12; se nao, algo mexeu na lei da tinta."
    );
}
