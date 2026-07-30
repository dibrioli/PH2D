//! Gates dos **DOIS FATORES FORA DO `τ`** — irmão de `tau_tests.rs`, que prova a LEI.
//!
//! A cobertura de um traço é `cover = (1 − exp(−τ)) · edge · fade`, e este arquivo é sobre os dois
//! termos que **não** são o `τ`:
//!
//! - o **fade sub-pixel**, o par do piso de largura (`mask *= smoothstep(0,1,thickness)` no raster);
//! - a **tampa CHATA**, que entra pelo `edge` — no raster ela é a ausência de geometria (o quad não
//!   estende) e aqui é a interseção com um semi-plano.
//!
//! ⚠️ **Pendurado sob `binning::tests` de propósito** (irmão, não filho de `tau_tests`): as fixtures
//! (`screen`, `art`, `push_tapered`, `BLACK`) são as do binning, e duplicá-las daria duas cenas para
//! uma pergunta.

use super::*;
use crate::binning::{ScreenSpace, bin_segments};
use crate::pack::FlipGpuData;

/// O fade é o `smoothstep(0, 1, x)` da WGSL, pinado contra valores computados **FORA** do codebase
/// (`x²·(3−2x)` à mão) — chamar a própria função seria o oráculo sempre-verde.
#[test]
fn the_sub_pixel_fade_is_the_wgsl_smoothstep() {
    for (x, esperado) in [
        (-1.0_f32, 0.0_f32),
        (0.0, 0.0),
        (0.15, 0.060_75), // 0,0225 · 2,70
        (0.25, 0.156_25), // 0,0625 · 2,50
        (0.3, 0.216),     // 0,0900 · 2,40
        (0.5, 0.5),       // 0,2500 · 2,00
        (0.8, 0.896),     // 0,6400 · 1,40
        (1.0, 1.0),
        (1.3, 1.0),
        (99.0, 1.0),
    ] {
        let v = crate::tau::sub_pixel_fade(x);
        assert!(
            (v - esperado).abs() < 1e-6,
            "fade({x}) = {v}, esperado {esperado}"
        );
    }
    // Monótona — sem isto, uma curva errada mas ancorada nos pontos passaria.
    let mut prev = -1.0_f32;
    for k in 0..=200 {
        let x = k as f32 / 200.0;
        let v = crate::tau::sub_pixel_fade(x);
        assert!(v >= prev, "o fade nao e' monotono em {x}");
        prev = v;
    }
}

/// ⚠️ **O ATALHO do caso comum é EXATO, e é isso que o torna aceitável:** o [`crate::tau::stroke_tau`]
/// pula o `sub_pixel_fade` quando as DUAS pontas do segmento medem ≥ 1 px, e a licença é que toda
/// amostra entre elas é uma combinação convexa — logo também ≥ 1 —, onde a função devolve `1.0`
/// **exato** (o `clamp` satura e `1·1·(3−2) = 1`). Um traço de espessura normal não paga um ciclo por
/// esta wave, e não paga um ulp.
#[test]
fn the_shortcut_for_a_normal_width_segment_is_exact() {
    for a in 0..24 {
        for b in 0..24 {
            let (wa, wb) = (1.0 + a as f32 * 0.37, 1.0 + b as f32 * 0.61);
            for k in 0..=64 {
                let f = k as f32 / 64.0;
                let lerp = wa * (1.0 - f) + wb * f;
                assert_eq!(
                    crate::tau::sub_pixel_fade(lerp),
                    1.0,
                    "o atalho mente em wa {wa} wb {wb} f {f} (lerp {lerp})"
                );
            }
        }
    }
}

/// O pico de alfa que o percurso da CPU deixa numa linha reta de largura `width`.
fn peak_alpha(sc: &ScreenSpace, h: f32, width: f32) -> f32 {
    let g = art(&[(&[[8.0, 16.0], [56.0, 16.0]], width, false, BLACK)]);
    let bins = bin_segments(&g, sc, 16);
    let mut best = 0.0_f32;
    for y in 0..h as u32 {
        best = best.max(walk_pixel(&bins, &g, sc, [32.5, y as f32 + 0.5])[3]);
    }
    best
}

/// 🔴 **A LINHA SUB-PIXEL DESBOTA em vez de sair GROSSA — e o oráculo é EXATO, não aproximado.**
///
/// Abaixo do piso (`MIN_WIDTH_PX = 1,3`) o raio clampado é o MESMO para toda largura, então a
/// GEOMETRIA é idêntica (mesmo `edge`, mesmo `τ`) e a única coisa que muda de uma largura para outra
/// é o fade. Isso dá uma forma fechada: `α(w) == sub_pixel_fade(w) · α(1,3)`.
///
/// ⚠️ **A `hardness` é 1,0** (o `art` a fixa) e isso é a fixture contendo o fenômeno: ali
/// `f = F_MAX` e `1 − exp(−τ)` já está **SATURADO**, o único regime onde escalar o `τ` em vez da
/// cobertura seria indistinguível de não fazer nada — que é exatamente a confusão que este gate
/// existe para pegar.
#[test]
fn a_sub_pixel_line_fades_instead_of_going_out_thick() {
    let (w, h) = (64.0, 32.0);
    let sc = screen(w, h);
    let cheia = peak_alpha(&sc, h, crate::binning::MIN_WIDTH_PX);
    assert!(
        cheia > 0.5,
        "a fixture nao contem o fenomeno: a linha de piso mede α {cheia:.4}"
    );
    for width in [0.15_f32, 0.3, 0.5, 0.8, 1.0] {
        let esperado = crate::tau::sub_pixel_fade(width) * cheia;
        let medido = peak_alpha(&sc, h, width);
        assert!(
            (medido - esperado).abs() < 1e-4,
            "a linha de {width} px nao desbotou: α {medido:.4} contra {esperado:.4} \
             — o fade escala a COBERTURA, nunca o τ"
        );
    }
}

/// 🔴 **O FADE É DO DAB, NÃO DO TRAÇO** — e é este gate que separa as duas leituras.
///
/// Num traço que afina de 8 px a 0,2 px a barriga sai CHEIA e a agulha FRACA. Se o fade fosse lido
/// do TRAÇO (uma largura só — a do primeiro ponto, digamos), a agulha de 0,2 px sairia com a tinta
/// de 8: o defeito que o fade existe para remover, escondido na ponta em vez de na linha toda.
#[test]
fn the_needle_tip_of_a_taper_is_faint_and_the_belly_is_not() {
    let (w, h) = (64.0, 32.0);
    let sc = screen(w, h);
    let mut g = FlipGpuData::default();
    push_tapered(&mut g, &[[8.0, 16.0], [56.0, 16.0]], &[8.0, 0.2]);
    let bins = bin_segments(&g, &sc, 16);
    let alpha_em = |x: f32| -> f32 {
        let mut best = 0.0_f32;
        for y in 0..h as u32 {
            best = best.max(walk_pixel(&bins, &g, &sc, [x, y as f32 + 0.5])[3]);
        }
        best
    };
    let (barriga, agulha) = (alpha_em(12.5), alpha_em(54.5));
    println!("  barriga α {barriga:.4}   agulha α {agulha:.4}");
    assert!(
        barriga > 0.9,
        "a barriga do traço desbotou sem motivo: α {barriga:.4}"
    );
    // Medido: barriga 1,0000 · agulha 0,3205; com o fade lido do TRAÇO a agulha vai a 1,0000, e a
    // barra fica confortavelmente entre os dois (uma barra colada no medido falha por ruído, uma
    // colada na mutação nao falha por nada).
    assert!(
        agulha < 0.6,
        "a agulha saiu com a tinta da barriga: α {agulha:.4} — o fade esta sendo lido do TRAÇO"
    );
}

// ————————————————————————————— a tampa chata —————————————————————————————

/// Monta um traço aberto com as flags de tampa dadas.
fn capped(pts: &[[f32; 2]], width: f32, flags: u32) -> FlipGpuData {
    let mut g = art(&[(pts, width, false, BLACK)]);
    g.strokes[0].flags |= flags;
    g
}

/// O alcance horizontal da tinta na linha `y` — `(x mínimo, x máximo)` acima de meia cobertura.
fn ink_span(g: &FlipGpuData, sc: &ScreenSpace, w: f32, y: f32) -> Option<(u32, u32)> {
    let bins = bin_segments(g, sc, 16);
    let (mut lo, mut hi) = (u32::MAX, 0u32);
    for x in 0..w as u32 {
        if walk_pixel(&bins, g, sc, [x as f32 + 0.5, y])[3] > 0.5 {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    (lo != u32::MAX).then_some((lo, hi))
}

/// 🔴 **A TAMPA CHATA CORTA, e é a única feature desta lista que o percurso tem de expressar por um
/// MECANISMO DIFERENTE do rasterizador.**
///
/// Lá a tampa é a AUSÊNCIA de geometria (o quad não estende); aqui não há quad, então ela é a
/// interseção com um semi-plano — um `max` sobre o `sd`. Um traço reto de raio 6 tem de acabar no
/// PONTO com tampa chata e ~6 px depois dele com tampa redonda.
#[test]
fn a_flat_cap_cuts_the_ribbon_where_the_stroke_ends() {
    let (w, h) = (64.0, 32.0);
    let sc = screen(w, h);
    let pts = [[16.0, 16.0], [48.0, 16.0]];
    let redonda = ink_span(&capped(&pts, 12.0, 0), &sc, w, 16.5).expect("tinta redonda");
    let chata = ink_span(
        &capped(
            &pts,
            12.0,
            crate::pack::FLAG_START_FLAT | crate::pack::FLAG_END_FLAT,
        ),
        &sc,
        w,
        16.5,
    )
    .expect("tinta chata");
    println!("  redonda {redonda:?}   chata {chata:?}");
    // A tampa redonda estende ~um raio para cada lado; a chata acaba no ponto.
    assert!(
        redonda.0 <= 11 && redonda.1 >= 52,
        "a tampa redonda nao estendeu: {redonda:?}"
    );
    assert!(
        chata.0 >= 15 && chata.1 <= 48,
        "a tampa chata nao cortou: {chata:?}"
    );
    // ⚠️ E cada ponta é INDEPENDENTE: só o começo chato deixa o fim redondo.
    let so_comeco = ink_span(
        &capped(&pts, 12.0, crate::pack::FLAG_START_FLAT),
        &sc,
        w,
        16.5,
    )
    .expect("tinta so-comeco");
    assert!(
        so_comeco.0 >= 15 && so_comeco.1 >= 52,
        "as duas pontas nao sao independentes: {so_comeco:?}"
    );
}

/// ⚠️ **A truncagem é por-SEGMENTO, nunca um semi-plano global** — e a diferença é ARTE que
/// desaparece. Um traço que volta e passa POR CIMA do próprio começo cortado **pinta** ali (é o que
/// os quads do rasterizador fazem: só o do PRIMEIRO segmento não estende). Um semi-plano global
/// apagaria a tinta da volta inteira do lado de fora do plano.
#[test]
fn a_stroke_that_folds_back_over_its_own_flat_cap_still_paints_there() {
    let (w, h) = (64.0, 48.0);
    let sc = screen(w, h);
    // Começa em (32,10) subindo, vira e volta ATRAVESSANDO x < 32 na altura y = 10.
    let pts = [[32.0, 10.0], [32.0, 30.0], [10.0, 10.0]];
    let flags = crate::pack::FLAG_START_FLAT | crate::pack::FLAG_END_FLAT;
    let g = capped(&pts, 10.0, flags);
    let bins = bin_segments(&g, &sc, 16);
    // Um ponto ATRÁS do plano do começo (x < 32) mas sobre a perna de volta.
    let atras = walk_pixel(&bins, &g, &sc, [22.5, 20.5])[3];
    println!("  a perna de volta atras do plano: α {atras:.4}");
    assert!(
        atras > 0.5,
        "a truncagem virou um semi-plano GLOBAL: α {atras:.4} atras do plano do começo"
    );
    // E o plano do começo AINDA corta o primeiro segmento (o controle).
    let cortado = walk_pixel(&bins, &g, &sc, [32.5, 6.5])[3];
    assert!(
        cortado < 0.5,
        "o plano do começo nao corta mais nada: α {cortado:.4} antes do primeiro ponto"
    );
}

/// ⚠️ **A tampa é dos EXTREMOS, e de mais nada** — uma JUNÇÃO interna não é uma ponta.
///
/// O gate nasceu de uma mutação que sobreviveu: trocar `cap_head == Some(seg.a)` por
/// `cap_head.is_some()` faz **cada** segmento cortar nas próprias pontas, e o que isso produz não é
/// um semi-plano global — é um **entalhe em toda quina**, porque no lado de fora da curva os dois
/// semi-planos vizinhos se somam e abrem uma fatia. Os dois probes que eu tinha (a perna de volta e
/// o corte do começo) não passavam por junção nenhuma.
#[test]
fn an_internal_joint_of_a_flat_capped_stroke_has_no_notch() {
    let (w, h) = (64.0, 64.0);
    let sc = screen(w, h);
    // Um "L" em mundo: (16,16) → (40,16) → (40,40). Em PIXEL a junção cai em (40, 48) e a perna
    // vertical sobe; o lado de FORA da curva é `px > 40 && py > 48`.
    let pts = [[16.0, 16.0], [40.0, 16.0], [40.0, 40.0]];
    let flags = crate::pack::FLAG_START_FLAT | crate::pack::FLAG_END_FLAT;
    let g = capped(&pts, 10.0, flags);
    let bins = bin_segments(&g, &sc, 16);
    // (43,51) dista 4,24 px da junção — dentro do disco de raio 5 que a quina tem de cobrir.
    let quina = walk_pixel(&bins, &g, &sc, [43.5, 51.5])[3];
    println!("  a quina externa: α {quina:.4}");
    assert!(
        quina > 0.5,
        "a junção interna ganhou um ENTALHE: α {quina:.4} — a tampa esta cortando todo segmento"
    );
}

/// ⚠️ **Tampa chata num traço FECHADO é inerte** — um anel não tem ponta, e o `flip.wgsl` gateia em
/// `!closed` pelo mesmo motivo. As flags podem estar marcadas (o `stroke_flags` as escreve sem olhar
/// o `closed`), e nada pode mudar por causa disso.
#[test]
fn a_closed_stroke_ignores_the_flat_cap_flags() {
    let (w, h) = (64.0, 64.0);
    let sc = screen(w, h);
    let pts = [[16.0, 16.0], [48.0, 16.0], [48.0, 48.0], [16.0, 48.0]];
    let limpo = art(&[(&pts, 10.0, true, BLACK)]);
    let mut marcado = limpo.clone();
    marcado.strokes[0].flags |= crate::pack::FLAG_START_FLAT | crate::pack::FLAG_END_FLAT;
    // ⚠️ **E o anel PONTILHADO é o que de fato prova o guard.** No anel contínuo a mutação (tirar o
    // `!closed`) sobreviveu, e não por buraco de gate — por ÁLGEBRA: o meio-disco que o plano tira do
    // primeiro segmento está inteiro dentro do disco que a **tampa redonda do segmento de FECHO**
    // cobre no mesmo ponto, então o `min` sobre os segmentos devolve o mesmo número. Nas CONTAS não:
    // a conta do arco 0 pertence ao PRIMEIRO segmento e o de fecho não a possui (o `bead_range` dele
    // é meio-aberto e `tail` é `None` num anel), então metade dela desapareceria.
    let mut anel = ph2d_flip::FlipStroke::new();
    for [x, y] in pts {
        anel.push_point(ph2d_flip::Point {
            pos: ph2d_core::Vec2::new(x, y),
            width: 10.0,
            opacity: 1.0,
            color: ph2d_flip::Rgba::new(0.0, 0.0, 0.0, 1.0),
        });
    }
    anel.closed = true;
    anel.hardness = 1.0;
    anel.tip = ph2d_flip::StrokeTip::Dots;
    anel.dot_spacing = 2.0;
    let mut dd = ph2d_flip::FlipDrawing::default();
    dd.strokes.push(anel);
    let pontilhado = crate::pack::pack_drawing(&dd);
    let mut pont_marcado = pontilhado.clone();
    pont_marcado.strokes[0].flags |= crate::pack::FLAG_START_FLAT | crate::pack::FLAG_END_FLAT;

    for (limpo, marcado, nome) in [
        (&limpo, &marcado, "continuo"),
        (&pontilhado, &pont_marcado, "pontilhado"),
    ] {
        let bl = bin_segments(limpo, &sc, 16);
        let bm = bin_segments(marcado, &sc, 16);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                assert_eq!(
                    walk_pixel(&bl, limpo, &sc, p),
                    walk_pixel(&bm, marcado, &sc, p),
                    "as flags de tampa mudaram o anel {nome} em ({x}, {y})"
                );
            }
        }
    }
}

/// 📏 SONDA — quantas PASSAGENS o partidor acha em cada ponto do X, e com que cobertura cada uma.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_pass_split_of_an_x() {
    let sc = screen(64.0, 64.0);
    let pernas = [[12.0, 12.0], [52.0, 52.0], [52.0, 12.0], [12.0, 52.0]];
    let mut pts: Vec<[f32; 2]> = Vec::new();
    for w in pernas.windows(2) {
        for k in 0..24 {
            let t = k as f32 / 24.0;
            pts.push([
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ]);
        }
    }
    pts.push(pernas[3]);
    let mut g = art(&[(&pts, 9.0, false, BLACK)]);
    g.strokes[0].flags |= crate::pack::FLAG_SELF_OVERLAP;
    for p in &mut g.points {
        p.opacity = 0.5;
    }
    let bins = bin_segments(&g, &sc, 16);
    for (nome, px, py) in [
        ("braço", 20.5, 20.5),
        ("cruz", 32.5, 32.5),
        ("meio-perna", 50.5, 32.5),
    ] {
        let p = [px, py];
        let Some(ti) = bins.tile_of_pixel(px, py) else {
            continue;
        };
        let run = bins.segs_of(ti);
        let mut s = 0;
        let mut fatias = Vec::new();
        while s < run.len() {
            let e = crate::dabs::pass_end(run, s);
            let cover = stroke_deposit(&run[s..e], &g, &sc, p).map_or(0.0, |d| d.cover);
            if cover > 0.0 {
                fatias.push((s, e, (cover * 100.0).round() as i32));
            }
            s = e;
        }
        let quebras: Vec<usize> = (0..run.len().saturating_sub(1))
            .filter(|&k| run[k + 1].a != run[k].b)
            .collect();
        println!(
            "  {nome:<11} run {:>3} segs -> {} passagem(ns) com tinta: {fatias:?}   quebras de cadeia: {quebras:?}",
            run.len(),
            fatias.len()
        );
    }
}

// ————————————————————————————— o self overlap —————————————————————————————

/// Um "X" de UM traço só, denso e a opacidade em `op`. Sem o bit se `overlap` é falso.
fn crossing_x(overlap: bool, op: f32) -> FlipGpuData {
    let pernas = [[12.0, 12.0], [52.0, 52.0], [52.0, 12.0], [12.0, 52.0]];
    let mut pts: Vec<[f32; 2]> = Vec::new();
    for w in pernas.windows(2) {
        for k in 0..24 {
            let t = k as f32 / 24.0;
            pts.push([
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ]);
        }
    }
    pts.push(pernas[3]);
    let mut g = art(&[(&pts, 9.0, false, BLACK)]);
    if overlap {
        g.strokes[0].flags |= crate::pack::FLAG_SELF_OVERLAP;
    }
    for p in &mut g.points {
        p.opacity = op;
    }
    g
}

/// 🔴 **O SELF OVERLAP compõe as PASSAGENS — e SÓ no cruzamento.**
///
/// ⚠️ **A partição é ÁLGEBRA, não gosto:** em opacidade 1 os dois casos coincidem
/// (`1 − Π exp(−τ_p) = 1 − exp(−τ)`), então a diferença é inteira sobre **como o `opacity` entra**, e
/// isso exige o `τ` de cada passagem. Esse gate mede as três regiões de um X de um traço só:
/// o braço (uma passagem), a junção (uma passagem, mas onde a 1ª versão desta wave CORTAVA) e o
/// cruzamento (duas).
#[test]
fn the_self_overlap_composes_only_where_the_stroke_crosses_itself() {
    let sc = screen(64.0, 64.0);
    let alpha = |overlap: bool, p: [f32; 2]| {
        let g = crossing_x(overlap, 0.5);
        let bins = bin_segments(&g, &sc, 16);
        walk_pixel(&bins, &g, &sc, p)[3]
    };
    let (braco, cruz, juncao) = ([20.5, 20.5], [32.5, 32.5], [50.5, 32.5]);
    for p in [braco, juncao] {
        let (off, on) = (alpha(false, p), alpha(true, p));
        assert!(
            (on - off).abs() < 1e-4,
            "a partição cortou onde NAO ha cruzamento em {p:?}: OFF {off:.4} ON {on:.4}"
        );
    }
    let (off, on) = (alpha(false, cruz), alpha(true, cruz));
    println!("  cruzamento  OFF α {off:.4}   ON α {on:.4}");
    // Duas passagens de cobertura cheia a opacidade 0,5: `1 − (1−0,5)² = 0,75`.
    assert!(
        (off - 0.5).abs() < 0.02,
        "o cruzamento sem a flag nao satura em `opacity`: {off:.4}"
    );
    assert!(
        (on - 0.75).abs() < 0.02,
        "o cruzamento com a flag nao compôs duas passagens: {on:.4} (esperado 0,75)"
    );
}

/// ⚠️ **Em opacidade 1 a flag só move o OMBRO — o MIOLO é intocado, e isso é a álgebra da lei.**
///
/// No interior saturado `1 − Π exp(−τ_p) = 1 − exp(−Σ τ_p)`: a partição não muda nada. O que sobra é
/// o `edge`, que passa a ser **por-passagem** — e dois ombros parciais compostos dão mais que a união
/// deles.
///
/// ⚠️ **A 1ª versão deste gate afirmava "a flag NÃO muda NADA em opacidade 1" e nasceu VERMELHA
/// (pior |Δ| 1,21e-1). A medição do PRODUTO decidiu contra o gate, não contra o código:** o
/// **rasterizador** muda mais que o percurso ali — pior Δalfa **+63 em 16 px** contra **+31 em 12 px**
/// —, então o efeito é da semântica `over`, não desta implementação. A afirmação certa é a que este
/// gate faz agora: onde a flag mexe, o pixel é de BORDA.
#[test]
fn at_full_opacity_the_self_overlap_only_moves_the_antialiased_shoulder() {
    let sc = screen(64.0, 64.0);
    let (a, b) = (crossing_x(false, 1.0), crossing_x(true, 1.0));
    let (ba, bb) = (bin_segments(&a, &sc, 16), bin_segments(&b, &sc, 16));
    let (mut mexidos, mut pior_no_miolo) = (0u32, 0.0_f32);
    for y in 0..64 {
        for x in 0..64 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let (va, vb) = (walk_pixel(&ba, &a, &sc, p), walk_pixel(&bb, &b, &sc, p));
            let d = (0..4).fold(0.0_f32, |m, c| m.max((va[c] - vb[c]).abs()));
            if d <= 1e-4 {
                continue;
            }
            mexidos += 1;
            // ⚠️ A asserção: um pixel que a flag mexe é de BORDA (o alfa `OFF` está estritamente
            // entre 0 e 1). Se ela mexesse o miolo, a partição não seria `1 − Π exp(−τ_p)`.
            assert!(
                va[3] > 1e-4 && va[3] < 1.0 - 1e-4,
                "a flag mexeu no MIOLO em ({x}, {y}): alfa OFF {:.4}, |Δ| {d:.4}",
                va[3]
            );
            pior_no_miolo = pior_no_miolo.max(d);
        }
    }
    println!("  opacidade 1: {mexidos} px de BORDA mexidos, pior |Δ| {pior_no_miolo:.3e}");
    assert!(
        mexidos > 0,
        "a fixture nao contem o fenomeno: a flag nao mexeu pixel nenhum em opacidade 1"
    );
}

/// ⚠️ **A LIMITAÇÃO, medida e nomeada:** um cruzamento que nunca sai do LADRILHO fica contíguo na
/// lista e lê como UMA passagem — a flag não compõe ali. A degradação é a conservadora (volta ao
/// comportamento `OFF`, o *first-wins* histórico do GP), e é a mesma postura dos tetos do
/// `neighbors.rs`. Este gate **pina o limite** para ninguém o descobrir por acidente.
#[test]
fn a_crossing_that_never_leaves_the_tile_reads_as_one_pass() {
    let sc = screen(64.0, 64.0);
    // Um lacinho de ~10 px, inteiro dentro de um ladrilho de 16.
    let pts = [
        [26.0, 30.0],
        [34.0, 30.0],
        [34.0, 38.0],
        [26.0, 38.0],
        [30.0, 34.0],
        [38.0, 34.0],
    ];
    let mut off = art(&[(&pts, 5.0, false, BLACK)]);
    for p in &mut off.points {
        p.opacity = 0.5;
    }
    let mut on = off.clone();
    on.strokes[0].flags |= crate::pack::FLAG_SELF_OVERLAP;
    let (bo, bn) = (bin_segments(&off, &sc, 16), bin_segments(&on, &sc, 16));
    let p = [30.5, 34.5];
    let (a_off, a_on) = (
        walk_pixel(&bo, &off, &sc, p)[3],
        walk_pixel(&bn, &on, &sc, p)[3],
    );
    println!("  lacinho num ladrilho só:  OFF α {a_off:.4}   ON α {a_on:.4}");
    assert!(
        (a_on - a_off).abs() < 1e-4,
        "a limitação MUDOU (o lacinho passou a compor: OFF {a_off:.4} ON {a_on:.4}) — \
         se isso e' de propósito, reescreva este gate; se nao, o partidor esta cortando por acidente"
    );
}
