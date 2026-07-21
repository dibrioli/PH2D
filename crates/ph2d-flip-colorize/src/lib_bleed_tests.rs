//! Gates do **Bleed / selo / quina** do Colorize (6º smoke, 2026-07-20) — irmão do
//! `lib_tests.rs`, separado pelo teto de LOC do workspace (700).
//!
//! Todos são auto-contidos (constroem a própria arte), então a separação é limpa: o
//! `lib_tests.rs` fica com os gates do CORTE e os helpers compartilhados.

use super::{DEFAULT_SQUEEZE, Scribble, colorize_with, seal_from_bleed, squeeze_from_bleed};
use ph2d_core::Vec2;

/// **O slider Bleed mapeia no pedágio de aperto** (6º smoke): `0.5` devolve o
/// [`DEFAULT_SQUEEZE`] EXATO (o meio do slider = o comportamento aprovado no 5º smoke),
/// `0` cola (pedágio alto), `1` vaza (pedágio baixo), e é MONOTÔNICO decrescente. Sem
/// transcendental (HR-5): exato nas oitavas, interpolado entre elas.
///
/// Mutação que sangra: inverter o `(1.0 - bleed)` (a direção do slider).
#[test]
fn the_bleed_slider_maps_to_the_squeeze_toll() {
    assert_eq!(
        squeeze_from_bleed(0.5),
        DEFAULT_SQUEEZE,
        "o meio do slider tem de devolver o pedágio DEFAULT (o 5º smoke), exato"
    );
    // Direção: bleed alto = vaza = pedágio BAIXO; bleed baixo = cola = pedágio ALTO.
    assert!(
        squeeze_from_bleed(1.0) < squeeze_from_bleed(0.0),
        "Bleed 1 (vaza) tem de dar pedágio menor que Bleed 0 (cola)"
    );
    // Monotônico (decrescente) e dentro da faixa medida [2⁸, 2¹⁶].
    let mut prev = u32::MAX;
    for i in 0..=10 {
        let sq = squeeze_from_bleed(i as f32 / 10.0);
        assert!(sq < prev, "o pedágio tem de cair a cada passo de Bleed");
        assert!((256..=65536).contains(&sq), "fora da faixa medida: {sq}");
        prev = sq;
    }
    // Clamp: fora de [0,1] não estoura.
    assert_eq!(squeeze_from_bleed(-1.0), squeeze_from_bleed(0.0));
    assert_eq!(squeeze_from_bleed(2.0), squeeze_from_bleed(1.0));
}

/// 🔴 **O Bleed MOVE a lente — o ajuste que o Trap não dava** (6º smoke: *"trap 0 e trap
/// máximo vazam parecidos. se há ajustes possíveis coloque no painel"*).
///
/// Na cena do smoke (divisor fora-do-centro em x=1 com vão aberto, um rabisco de cada lado),
/// a LENTE é o menor `x` que a cor da DIREITA alcança à esquerda do divisor. Com o Bleed
/// baixo (cola) ela fica perto da linha; com o Bleed alto (vaza) ela entra fundo. É o
/// controle CONTÍNUO que funciona com o vão ABERTO — onde o Trap, binário, não muda nada até
/// selar (pinado no gate irmão `the_trap_is_binary_the_bleed_is_continuous`).
///
/// Mutação que sangra: ignorar o `squeeze` no `Scratch::new` (a lente para de responder).
#[test]
fn the_bleed_moves_the_lens_through_an_open_gap() {
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
    for (a, b) in [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5)),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5)),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5)),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5)),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6)),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5)),
    ] {
        let pts = seg_(a, b, 24);
        let n = pts.len();
        strokes.push((pts, vec![0.26; n], false));
    }
    let scribbles = vec![
        Scribble {
            label: 0,
            points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg_(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
            width: 0.15,
        },
    ];
    let lens = |bleed: f32| -> f32 {
        let sq = squeeze_from_bleed(bleed);
        colorize_with(&strokes, &scribbles, 40.0, 0.0, sq)
            .iter()
            .filter(|r| r.label == 1)
            .flat_map(|r| r.fill.outer.iter())
            .fold(f32::MAX, |m, p| m.min(p.x))
    };
    let (deep, mid, hug) = (lens(1.0), lens(0.5), lens(0.0));
    // Vaza (bleed 1) entra MAIS fundo (min_x menor) que cola (bleed 0); o meio fica entre.
    assert!(
        deep < mid - 0.05 && mid < hug - 0.05,
        "o Bleed tem de mover a lente: vaza {deep:.3} < meio {mid:.3} < cola {hug:.3}"
    );
    // E a amplitude é VISÍVEL (não sub-pixel): pelo menos meia unidade de mundo do vazado ao
    // colado (o Trap, binário, dá zero até selar).
    assert!(
        hug - deep > 0.5,
        "a faixa do Bleed tem de ser ampla o bastante para valer um slider (Δ={:.3})",
        hug - deep
    );
}

/// **O Trap é BINÁRIO; o Bleed é CONTÍNUO** — por que os DOIS estão no painel (6º smoke).
///
/// O Enio viu *"trap 0 e trap máximo vazam parecidos"*: enquanto o Trap não SELA o vão, ele
/// não muda a lente NADA (a bola ou passa ou não passa). O Bleed, no mesmo vão aberto, varia
/// a lente continuamente. Este gate pina a diferença de NATUREZA — é o que justifica dois
/// controles em vez de um.
#[test]
fn the_trap_is_binary_the_bleed_is_continuous() {
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5)),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5)),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5)),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5)),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6)),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5)),
    ]
    .into_iter()
    .map(|(a, b)| {
        let pts = seg_(a, b, 24);
        let n = pts.len();
        (pts, vec![0.26; n], false)
    })
    .collect();
    let scribbles = vec![
        Scribble {
            label: 0,
            points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg_(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
            width: 0.15,
        },
    ];
    let lens_trap = |trap: f32| -> f32 {
        colorize_with(&strokes, &scribbles, 40.0, trap, DEFAULT_SQUEEZE)
            .iter()
            .filter(|r| r.label == 1)
            .flat_map(|r| r.fill.outer.iter())
            .fold(f32::MAX, |m, p| m.min(p.x))
    };
    // Gap 1,2 doc = 48 px de buffer a esta precisão ⇒ sela em raio > 24. Abaixo disso, a
    // lente é IDÊNTICA (o Trap não faz nada — o que o Enio viu).
    assert_eq!(
        lens_trap(6.0),
        lens_trap(18.0),
        "o Trap ABAIXO do selo tem de dar a MESMA lente (binário — o report do Enio)"
    );
    // Acima do selo, a lente some (a cor não entra mais pelo vão — vira dois componentes).
    assert!(
        lens_trap(30.0) > lens_trap(18.0) + 0.1,
        "o Trap ACIMA do selo tem de fechar a lente (a cor cola no divisor)"
    );
}

/// 🔴 **O `Bleed 0` SELA o vão** (6º smoke seg.: *"funciona, contudo, mesmo com bleed 0 ainda
/// há vazamento"*; Enio 2026-07-20 escolheu *"Bleed 0 SELA o vão"*).
///
/// O pedágio SATURA — o `Bleed 0` só com `squeeze` fica em `+0.915` (um bojo ainda vaza,
/// porque um vão largo é passagem por design). A cura que o Enio pediu é tratar o vão como
/// FECHADO, e `seal_from_bleed` alimenta a trapped-ball no `Bleed` baixo: no `0`, o raio
/// (`1.0` doc × precisão = 40 px aqui) fecha o vão de 48 px, os dois lados viram componentes
/// SEPARADOS, e a fronteira cola na linha (`+0.95`, a borda do vão). No MEIO (`0.5`) o selo é 0
/// e o vão fica aberto — o 5º smoke intacto.
///
/// Mutação que sangra: `seal_from_bleed → 0` (o selo some) ⇒ o `Bleed 0` volta ao `+0.915` do
/// squeeze puro, abaixo do limiar de selagem.
#[test]
fn the_bleed_zero_seals_the_gap() {
    let seg_ = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5)),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5)),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5)),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5)),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6)),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5)),
    ]
    .into_iter()
    .map(|(a, b)| {
        let pts = seg_(a, b, 24);
        let n = pts.len();
        (pts, vec![0.26; n], false)
    })
    .collect();
    let scribbles = vec![
        Scribble {
            label: 0,
            points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg_(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8),
            width: 0.15,
        },
    ];
    let precision = 40.0f32;
    // A LENTE do produto: o `Bleed` alimenta a bola (`seal`) E o pedágio (`squeeze`), Trap 0.
    let lens = |bleed: f32| -> f32 {
        let trap = seal_from_bleed(bleed) * precision;
        let sq = squeeze_from_bleed(bleed);
        colorize_with(&strokes, &scribbles, precision, trap, sq)
            .iter()
            .filter(|r| r.label == 1)
            .flat_map(|r| r.fill.outer.iter())
            .fold(f32::MAX, |m, p| m.min(p.x))
    };
    // O vão do divisor está em x=1. SELADO, a lente cola na linha (a borda do vão, ~+0.95).
    let sealed = lens(0.0);
    assert!(
        sealed >= 0.93,
        "o Bleed 0 tem de SELAR o vão (lente na linha ~+0.95, medido); deu {sealed:.3}"
    );
    // O MEIO fica ABERTO — a seepage do 5º smoke, bem à esquerda da linha.
    let mid = lens(0.5);
    assert!(
        mid < 0.75,
        "o Bleed 0.5 tem de deixar o vão ABERTO (o 5º smoke); deu {mid:.3}"
    );
    // E é o SELO que fecha o último trecho: sem ele (Trap 0) o squeeze puro para em ~+0.915.
    let squeeze_only = colorize_with(
        &strokes,
        &scribbles,
        precision,
        0.0,
        squeeze_from_bleed(0.0),
    )
    .iter()
    .filter(|r| r.label == 1)
    .flat_map(|r| r.fill.outer.iter())
    .fold(f32::MAX, |m, p| m.min(p.x));
    assert!(
        sealed > squeeze_only + 0.02,
        "o selo tem de fechar além do que o squeeze sozinho alcança \
         (selado {sealed:.3} vs squeeze puro {squeeze_only:.3})"
    );
}

/// 🔴 **A cor preenche a QUINA que o artista desenhou** (report do Enio 2026-07-20: um
/// triângulo preto na quina, cortado por uma diagonal limpa a 45°).
///
/// As quatro paredes de uma caixa desenhada à mão são traços **SEPARADOS**, e o tremor deixa
/// as pontas sem coincidir: na cena do smoke o buraco entre os EIXOS mede 0,023 e 0,040 doc
/// nas quinas esquerdas. A tinta é rasterizada no eixo (raio 0, a âncora imune ao zoom do
/// BUGS #14), então um buraco de ≳2 px de buffer **não fecha sozinho**. A trapped-ball sela
/// esse buraco no NÚCLEO — mas a dilatação de volta contava SALTOS e o atravessava; numa quina
/// convexa o núcleo de FORA chega em ~`r` saltos e o de DENTRO em ~`2r`, o exterior ganha a
/// corrida e leva uma cunha (a bissetriz de hop-count numa grade 4-conexa = a reta a 45°).
///
/// ⚠️ **O fixture TEM de conter o fenômeno, e por isso ele mede o próprio vão.** A precisão
/// decide se o buraco existe: a 40 os mesmos vãos viram 0,9–1,6 px, a cápsula os fecha, e o
/// gate fica verde por inércia sobre o bug de pé (foi exatamente o que aconteceu na 1ª
/// tentativa de reproduzir). Uma caixa de traço ÚNICO fechado também nunca reproduz.
///
/// Oráculo de APARÊNCIA: a partir de cada quina que o artista desenhou, ande a diagonal para
/// dentro; a região do rabisco tem de cobrir o ponto logo ali. Ele não consulta `segment`,
/// componente, EDT nem a fila — pergunta *"a quina está pintada?"*.
///
/// Mutação que sangra: devolver a dilatação (`segment.rs` 4a) à BFS FIFO de saltos.
#[test]
fn the_colour_fills_the_corner_the_artist_drew() {
    // Tremor determinístico — o MESMO da cena de smoke (`flip_colorize_smoke::hand`).
    let hh = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let wall = |a: Vec2, b: Vec2, sd: usize| -> Vec<Vec2> {
        (0..24)
            .map(|i| {
                let t = i as f32 / 23.0;
                Vec2::new(
                    a.x + (b.x - a.x) * t + hh(i + sd) * 0.05,
                    a.y + (b.y - a.y) * t + hh(i + sd + 91) * 0.05,
                )
            })
            .collect()
    };
    // QUATRO paredes separadas (é isto que cria os buracos de quina) + o divisor do smoke.
    let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
    for (a, b, sd) in [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6), 41),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5), 53),
    ] {
        let pts = wall(a, b, sd);
        let n = pts.len();
        strokes.push((pts, vec![0.13; n], false));
    }
    let seg_ = |a: Vec2, b: Vec2| -> Vec<Vec2> {
        (0..8)
            .map(|i| {
                let t = i as f32 / 7.0;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let scribbles = vec![Scribble {
        label: 0,
        points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5)),
        width: 0.15,
    }];
    let precision = 160.0f32;

    // O fixture CONTÉM o fenômeno? O buraco entre os eixos das duas paredes da quina
    // inferior-esquerda tem de ser largo o bastante para a cápsula de raio 0 NÃO o fechar.
    let bl_bottom = strokes[0].0[0]; // ponta inicial da parede de baixo
    let bl_left = *strokes[3].0.last().expect("parede esquerda"); // ponta final da esquerda
    let gap_px = (bl_bottom - bl_left).length() * precision;
    assert!(
        gap_px >= 2.0,
        "o fixture NÃO contém o fenômeno: o vão da quina é {gap_px:.2} px de buffer e a \
         cápsula de raio 0 o fecha sozinha — suba a precisão ou afaste as pontas"
    );

    let regs = colorize_with(&strokes, &scribbles, precision, 1.0 * precision, 65536);
    let inside = |poly: &[Vec2], p: Vec2| -> bool {
        let mut c = false;
        let n = poly.len();
        for i in 0..n {
            let j = (i + n - 1) % n;
            let (a, b) = (poly[i], poly[j]);
            if ((a.y > p.y) != (b.y > p.y)) && (p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x)
            {
                c = !c;
            }
        }
        c
    };
    // As duas quinas ESQUERDAS (as do lado do rabiscso vermelho), 0,10 doc para dentro pela
    // diagonal. A cunha medida no defeito tinha 0,33 doc — dez vezes esta margem.
    for (cx, cy, nome) in [
        (-4.0f32, 2.5f32, "superior-esquerda"),
        (-4.0, -2.5, "inferior-esquerda"),
    ] {
        let p = Vec2::new(cx + 0.10, cy - 0.10 * cy.signum());
        assert!(
            regs.iter()
                .any(|r| r.label == 0 && inside(&r.fill.outer, p)),
            "a quina {nome} não foi pintada em ({:.2}, {:.2}) — a dilatação atravessou o \
             buraco que a bola recusou e o exterior levou a cunha",
            p.x,
            p.y
        );
    }
}

/// 🔴 **O selo é um DEGRAU, nunca uma rampa** (report do Enio 2026-07-20: *"o bleed baixo está
/// tirando tinta das quinas antes de resolver o vazamento"* — com a lente AINDA na tela).
///
/// A trapped-ball é **binária**: abaixo do raio que sela o vão ela não muda a lente NADA (o
/// gate irmão `the_trap_is_binary_the_bleed_is_continuous` pina isso), mas **já erode a arte**
/// — as quinas. Então uma RAMPA sobre ela entrega, na faixa do meio, só o efeito colateral: a
/// bola grande o bastante para comer a quina e pequena demais para fechar o vão. O degrau
/// apaga essa zona — ou o vão está aberto e **a bola não existe** (`0`), ou está selado
/// (`SEAL_DOC`). Nunca um raio intermediário, que é puro dano.
///
/// Mutação que sangra: voltar à rampa (`((KNEE-b)/KNEE).max(0) * SEAL_DOC`) ⇒ aparecem raios
/// intermediários.
#[test]
fn the_seal_is_a_step_not_a_ramp() {
    // Varre o slider inteiro: todo valor devolve OU zero OU o raio cheio, nada entre.
    let full = seal_from_bleed(0.0);
    assert!(full > 0.0, "o Bleed 0 tem de selar");
    for i in 0..=1000 {
        let b = i as f32 / 1000.0;
        let s = seal_from_bleed(b);
        assert!(
            s == 0.0 || (s - full).abs() < 1e-6,
            "raio INTERMEDIÁRIO em bleed={b:.3}: {s} (nem 0 nem {full}) — \
             é a zona que só entrega dano: erode a quina sem fechar o vão"
        );
    }
    // E é monotônico no sentido certo: selado embaixo, aberto em cima.
    assert_eq!(seal_from_bleed(1.0), 0.0, "Bleed 1 = vão ABERTO, sem bola");
    assert_eq!(
        seal_from_bleed(0.5),
        0.0,
        "o meio (5º smoke) não carrega bola"
    );
}

/// 🔴 **O selo do `Bleed 0` NÃO extrapola para fora do retângulo** (report do Enio 2026-07-20:
/// *"Antes o trap controlava a extrapolação para fora do retangulo. Agora nada tira a
/// extrapolação"*).
///
/// A trapped-ball erode `trap_px` da tinta; com a **margem da grade FIXA** (`MARGIN_PX`), uma
/// bola grande deixa o FUNDO (a moldura de papel fora da caixa) sem espaço para o próprio
/// núcleo, e a dilatação o engole para dentro de uma cor — a cor VAZA para além da caixa. E o
/// selo do `Bleed 0` alimenta um raio que **cresce com a precisão** (o zoom), então num zoom
/// aproximado (precisão alta) a bola fica maior que a margem e a regressão aparece. O fix é a
/// **margem crescer com o raio** (`margin_px = MARGIN_PX.max(trap_px)`), medido aqui a precisão
/// 160 (onde o bug se manifesta) com a tremor real da cena do smoke.
///
/// Mutação que sangra: `margin_px = MARGIN_PX` (fixa) ⇒ a `RED` inunda a caixa inteira
/// (`max_x ≈ +4.3`, do outro lado do divisor) em vez de parar no divisor (`+1.05`).
#[test]
fn the_bleed_zero_seal_does_not_extrapolate_past_the_box() {
    let hh = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let seg_ = |a: Vec2, b: Vec2, n: usize, seed: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(
                    a.x + (b.x - a.x) * t + hh(i + seed) * 0.05,
                    a.y + (b.y - a.y) * t + hh(i + seed + 91) * 0.05,
                )
            })
            .collect()
    };
    let mut strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = Vec::new();
    for (a, b, sd) in [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29),
        (Vec2::new(1.0, -2.5), Vec2::new(1.0, -0.6), 41),
        (Vec2::new(1.0, 0.6), Vec2::new(1.0, 2.5), 53),
    ] {
        let pts = seg_(a, b, 24, sd);
        let n = pts.len();
        strokes.push((pts, vec![0.13; n], false));
    }
    let scribbles = vec![
        Scribble {
            label: 0,
            points: seg_(Vec2::new(-2.0, -1.5), Vec2::new(-2.0, 1.5), 8, 3),
            width: 0.15,
        },
        Scribble {
            label: 1,
            points: seg_(Vec2::new(2.6, -1.5), Vec2::new(2.6, 1.5), 8, 5),
            width: 0.15,
        },
    ];
    // Precisão 160 = um zoom aproximado (a regressão só aparece quando o raio > margem fixa).
    let precision = 160.0f32;
    let trap = seal_from_bleed(0.0) * precision; // o raio do Bleed 0 (= 160 px aqui)
    let regs = colorize_with(
        &strokes,
        &scribbles,
        precision,
        trap,
        squeeze_from_bleed(0.0),
    );
    let red_max_x = regs
        .iter()
        .filter(|r| r.label == 0)
        .flat_map(|r| r.fill.outer.iter())
        .fold(f32::MIN, |m, p| m.max(p.x));
    // O VERMELHO é a cor da ESQUERDA. Selado, ele para no divisor (x=1). Se o fundo colapsar,
    // ele inunda a caixa inteira e cruza para a direita (max_x ≈ +4.3).
    assert!(
        red_max_x < 2.0,
        "o Bleed 0 fez a cor extrapolar (vermelho alcança x={red_max_x:.2}; devia parar no \
         divisor em ~+1.05). A margem da grade não cresceu com o raio da bola."
    );
}
