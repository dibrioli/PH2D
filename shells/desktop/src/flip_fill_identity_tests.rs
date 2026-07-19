//! **A IDENTIDADE DA COR: um traço, um conjunto de vértices.**
//!
//! Estes gates defendem a propriedade que o smoke do Enio aprovou em 2026-07-13, com o
//! Suzanne na tela, e que **estava desprotegida até 2026-07-18**:
//!
//! > *"Não há dois conjuntos de vértices para dessincronizar: há um só. Esculpir a linha
//! > move a cor junto, de graça, para sempre, em qualquer zoom."*
//!
//! Quem a entrega é o `filled_shape_target`: quando a região É o interior de um traço
//! fechado, o balde põe `fill` **no próprio traço**. A cor não é uma cópia da geometria —
//! ela É a geometria.
//!
//! # Por que estes gates existem (e por que quase não existiram)
//!
//! A fatia **R3** do plano `docs/Flip/10_regiao_por_curvas.md` propunha **aposentar** o
//! `filled_shape_target`, alegando que a rota do arranjo o tinha subsumido. A alegação vale
//! para a **dilatação** (a rota das curvas põe a fronteira no eixo, logo `s = 0` — há gate
//! provando isso) e **não vale para a identidade dos vértices**: `curve_region` *deriva* a
//! fronteira dos vértices das linhas, e o `fill_stroke` os **COPIA** para um traço novo.
//! É um snapshot, não uma identidade compartilhada.
//!
//! # O discriminador é a SELEÇÃO, não o pincel
//!
//! ⚠️ **A 1ª leitura desta medição estava ERRADA, e o erro é instrutivo.** Ela comparou a
//! rota do traço próprio numa FORMA FECHADA com a rota das curvas numa CÉLULA DE GRADE —
//! arte onde a primeira nunca dispara. Maçã com laranja: o número saía do fixture, não da
//! rota. Medido na **mesma arte** (forma fechada de mão, linha de 0,4 de largura), com o
//! `filled_shape_target` ligado e desligado:
//!
//! | gesto | traço próprio | curvas, nada selecionado | curvas, **só a linha selecionada** |
//! |---|---|---|---|
//! | Push | 0,0000 | 0,0000 | **2,1397** |
//! | Grab | 0,0000 | 0,0000 | **2,3134** |
//! | Smooth | 0,0000 | 0,0000 | **0,8972** |
//! | Pinch | 0,0000 | 0,0000 | **0,3344** |
//! | Twist | 0,0000 | 0,0000 | **0,1604** |
//!
//! **Sem seleção as duas rotas empatam em zero** — os vértices copiados nascem em cima dos
//! originais e todo pincel os move junto. O que separa as duas é o **auto-masking do W6**:
//! `Session::begin` filtra por `!any_selected || st.selected`, então com a linha
//! selecionada uma região que seja um traço À PARTE sai da máscara e fica inteira para
//! trás — até **5,8 larguras de linha**. Na rota do traço próprio não há o que mascarar: a
//! cor é um CAMPO do traço selecionado.
//!
//! Corolário para quem escrever gate aqui: **uma fixture sem seleção fica verde nas duas
//! rotas** e não prova nada.
//!
//! ⚠️ E o gate que existia (`clicking_inside_a_closed_shape_paints_the_shape_itself`) pina
//! que a **rota dispara** — um gate SOBRE a rota, então ele seria apagado junto com ela e o
//! produto ficaria verde. **Um gate que mora dentro do que ele defende não defende nada.**

use crate::flip_fill::fill_click;
use crate::flip_fill::tests::style;
use crate::flip_fill_target::max_dist_to_ring;
use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_reshape::{InputSample, ReshapeKind, ReshapeParams, Session};
use ph2d_tool_flip::{FillMode as ToolFillMode, FlipStyleSnapshot};
use ph2d_vec_scene::Xform;

/// Uma forma fechada desenhada À MÃO (trêmula, `closed = false` — como a mão a deixa).
fn hand_blob() -> FlipDrawing {
    let wob = |i: usize| ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    // 41 pontos: o último REPETE o primeiro — a mão que encosta a ponta no começo.
    // (`closed` continua FALSE, que é como o `build_stroke` deixa um traço de mão.)
    for i in 0..=40 {
        let j = i % 40;
        let t = j as f32 / 40.0 * std::f32::consts::TAU;
        let r = 10.0 + wob(j) * 0.6;
        s.push_point(Point {
            pos: Vec2::new(10.0 + r * t.cos(), 10.0 + r * t.sin()),
            width: 0.4,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    d.strokes.push(s);
    d
}

/// A grade trêmula (a célula do meio é delimitada por QUATRO traços) — a rota `curve_region`.
fn grid_drawing() -> FlipDrawing {
    let wob = |i: usize| ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let mut d = FlipDrawing::new();
    for (k, horiz, off) in [
        (0usize, true, 0.0f32),
        (1, true, 10.0),
        (2, false, 0.0),
        (3, false, 10.0),
    ] {
        let mut s = FlipStroke::new();
        for i in 0..24 {
            let t = -1.0 + i as f32 * (12.0 / 23.0);
            let j = wob(i + k * 37) * 0.5;
            s.push_point(Point {
                pos: if horiz {
                    Vec2::new(t, off + j)
                } else {
                    Vec2::new(off + j, t)
                },
                width: 0.4,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        d.strokes.push(s);
    }
    d
}

fn st0() -> FlipStyleSnapshot {
    FlipStyleSnapshot {
        grow: 0.0,
        ..style(ToolFillMode::Paint)
    }
}

/// O anel do preenchimento que existe no desenho.
fn region_ring(d: &FlipDrawing) -> Vec<Vec2> {
    d.strokes
        .iter()
        .rev()
        .find(|s| s.fill.is_some())
        .expect("o balde pintou")
        .positions()
        .to_vec()
}

/// Todos os pontos de LINE-ART (o que o artista vê como "a linha").
fn art_pts(d: &FlipDrawing) -> Vec<Vec2> {
    d.strokes
        .iter()
        .filter(|s| !s.hide_stroke)
        .flat_map(|s| s.positions().to_vec())
        .collect()
}

/// Roda o motor de escultura REAL: pen-down + N moves, como o `flip_reshape` faz.
fn sculpt(d: &mut FlipDrawing, kind: ReshapeKind, at: Vec2, dir: Vec2, steps: usize) {
    let p = ReshapeParams {
        kind,
        radius: 6.0,
        strength: 1.0,
        invert: false,
        frame_falloff: 1.0,
        px_to_local: 0.01,
    };
    let mut pos = at;
    let s0 = InputSample {
        pos,
        delta: Vec2::ZERO,
        pressure: 1.0,
    };
    let mut sess = Session::begin(&d.strokes, &p, &s0);
    sess.apply(&mut d.strokes, &p, &s0);
    for _ in 0..steps {
        let next = pos + dir;
        let s = InputSample {
            pos: next,
            delta: dir,
            pressure: 1.0,
        };
        sess.apply(&mut d.strokes, &p, &s);
        pos = next;
    }
}

/// Quanto a cor "descolou" da arte: a maior distância de um ponto do ANEL à linha mais
/// próxima, e da linha ao anel. Zero = a cor está exatamente sobre a arte.
fn divorce(d: &FlipDrawing) -> (f32, usize) {
    let ring = region_ring(d);
    let art = art_pts(d);
    (max_dist_to_ring(&ring, &art), ring.len())
}

/// **A PROPRIEDADE: esculpir a linha move a cor junto.**
///
/// Os cinco pincéis, um por um. O Smooth está aqui porque é o que separa a identidade da
/// coincidência — ver o cabeçalho.
#[test]
fn sculpting_a_filled_shape_carries_its_colour() {
    for kind in [
        ReshapeKind::Push,
        ReshapeKind::Grab,
        ReshapeKind::Smooth,
        ReshapeKind::Pinch,
        ReshapeKind::Twist,
    ] {
        let mut d = hand_blob();
        fill_click(&mut d, &st0(), Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY)
            .expect("preenche a forma fechada");
        // PRESENÇA: sem isto, "não descolou" fica verde num desenho sem cor nenhuma.
        assert!(
            d.strokes.iter().any(|s| s.fill.is_some()),
            "{kind:?}: premissa do gate — o balde tem de ter pintado alguma coisa"
        );
        // ⚠️ **SÓ o line-art selecionado — é o gesto real e é o discriminador.** Sem
        // seleção as duas rotas empatam em zero (tabela no cabeçalho) e este gate ficaria
        // verde mesmo com a cor virando um traço à parte.
        for s in &mut d.strokes {
            s.selected = s.fill.is_none();
        }
        sculpt(&mut d, kind, Vec2::new(5.0, 2.0), Vec2::new(0.0, 0.35), 8);

        let (div, n) = divorce(&d);
        assert!(
            div <= 1e-4,
            "{kind:?}: a cor descolou {div} da arte ({n} pontos no anel) — a rota do \
             traco proprio nao pode ter dois conjuntos de vertices"
        );
    }
}

/// **O IRMÃO NEGATIVO: na rota das curvas o divórcio EXISTE.**
///
/// Sem ele o gate acima pode ficar verde por acidente — bastaria uma fixture em que as
/// duas rotas coincidem. Este mede o contrário e **exige** o descolamento: é ele que prova
/// que a métrica enxerga o fenômeno.
#[test]
fn on_the_curve_route_the_colour_does_come_loose() {
    let mut d = grid_drawing();
    fill_click(&mut d, &st0(), Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");
    sculpt(
        &mut d,
        ReshapeKind::Smooth,
        Vec2::new(5.0, 0.0),
        Vec2::new(0.2, 0.0),
        8,
    );

    let (div, _) = divorce(&d);
    assert!(
        div > 0.1,
        "a rota das curvas COPIA os vertices, entao o Smooth tem de separar a cor da arte \
         — veio {div}, e se isso virou zero a metrica parou de medir o fenomeno"
    );
}

/// **A CATRACA: o divórcio acumula e nunca volta.**
///
/// É o que separa "ruído numérico" de "defeito". Na rota do traço próprio o número é zero
/// em toda sessão; na outra ele sobe monotonamente (medido: 0,13 → 0,21 → … → 0,42).
#[test]
fn repeated_sculpting_never_pulls_the_colour_off_a_filled_shape() {
    let mut d = hand_blob();
    fill_click(&mut d, &st0(), Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");
    for s in &mut d.strokes {
        s.selected = s.fill.is_none(); // o discriminador — ver o cabeçalho
    }
    for round in 0..6 {
        sculpt(
            &mut d,
            ReshapeKind::Smooth,
            Vec2::new(5.0, 2.0),
            Vec2::new(0.1, 0.1),
            6,
        );
        let (div, _) = divorce(&d);
        assert!(
            div <= 1e-4,
            "sessao {round}: a cor acumulou {div} de descolamento — a catraca comecou"
        );
    }
}

/// **A PREMISSA da tabela: sem seleção as duas rotas empatam.**
///
/// É o controle que dá sentido ao gate acima. Se um dia isto virar vermelho, o
/// discriminador mudou de lugar e a tabela do cabeçalho parou de descrever o produto —
/// e aí o gate acima pode estar verde pelo motivo errado.
#[test]
fn with_nothing_selected_both_routes_carry_the_colour() {
    let mut d = hand_blob();
    fill_click(&mut d, &st0(), Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");
    sculpt(
        &mut d,
        ReshapeKind::Grab,
        Vec2::new(5.0, 2.0),
        Vec2::new(0.0, 0.35),
        8,
    );
    let (div, _) = divorce(&d);
    assert!(
        div <= 1e-4,
        "sem selecao a cor tem de acompanhar em QUALQUER rota; veio {div}"
    );
}

/// **Preencher não desloca o índice do line-art** — o tween pareia POR ÍNDICE.
///
/// A rota das curvas insere a região no começo do vetor; se ela disparasse numa forma
/// fechada, preencher a chave A e não a B faria o inbetween parear **linha com região**.
#[test]
fn filling_a_closed_shape_does_not_shift_the_line_art_index() {
    let mut d = hand_blob();
    let before: Vec<Vec2> = d.strokes[0].positions().to_vec();
    fill_click(&mut d, &st0(), Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");
    let after: Vec<Vec2> = d.strokes[0].positions().to_vec();
    assert_eq!(
        before, after,
        "o traco de line-art saiu do indice 0 — o tween pareia por indice, entao \
         preencher uma chave e nao a outra parearia LINHA com REGIAO"
    );
}
