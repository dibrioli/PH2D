//! Os gates dos oito pincéis — um por pincel, cada um afirmando **a propriedade que
//! define aquele pincel** (não "mudou alguma coisa"). Módulo-irmão pelo cap de LOC.

use super::*;
use ph2d_flip::{Point, Rgba};

/// Uma linha horizontal de `n` pontos, espaçados de 1 unidade, começando na origem.
fn line(n: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..n {
        s.push_point(Point {
            pos: Vec2::new(i as f32, 0.0),
            width: 6.0,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
    }
    s
}

/// Uma linha trêmula (zigue-zague de ±0,4 em y).
fn wobbly(n: usize) -> FlipStroke {
    let mut s = line(n);
    for (i, p) in s.positions_mut().iter_mut().enumerate() {
        p.y = if i % 2 == 0 { 0.4 } else { -0.4 };
    }
    s
}

fn params(kind: ReshapeKind, radius: f32) -> ReshapeParams {
    ReshapeParams {
        kind,
        radius,
        strength: 1.0,
        ..Default::default()
    }
}

fn sample(pos: Vec2, delta: Vec2) -> InputSample {
    InputSample {
        pos,
        delta,
        pressure: 1.0,
    }
}

/// Uma sessão de UMA amostra (o caso comum dos testes).
fn stroke_once(strokes: &mut [FlipStroke], p: &ReshapeParams, s: &InputSample) -> bool {
    let mut sess = Session::begin(strokes, p, s);
    sess.apply(strokes, p, s)
}

// ─────────────────────────── a infra (T5.1) ───────────────────────────

/// **A influência é o funil**: máxima no centro, ZERO na borda, e o multiframe a
/// escala. Sem derivada nas pontas (smoothstep) — é o que evita degrau na marca.
#[test]
fn influence_peaks_at_the_centre_and_dies_at_the_rim() {
    let p = params(ReshapeKind::Smooth, 10.0);
    let s = sample(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
    assert!((influence(&p, &s, Vec2::new(0.0, 0.0)) - 1.0).abs() < 1e-6);
    assert_eq!(
        influence(&p, &s, Vec2::new(10.0, 0.0)),
        0.0,
        "na borda: zero"
    );
    assert_eq!(influence(&p, &s, Vec2::new(11.0, 0.0)), 0.0, "fora: zero");
    let mid = influence(&p, &s, Vec2::new(5.0, 0.0));
    assert!((mid - 0.5).abs() < 1e-6, "meio raio = meio efeito: {mid}");

    // Força e pressão entram multiplicando…
    let half = ReshapeParams { strength: 0.5, ..p };
    assert!((influence(&half, &s, Vec2::new(0.0, 0.0)) - 0.5).abs() < 1e-6);
    let soft = sample(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
    let soft = InputSample {
        pressure: 0.25,
        ..soft
    };
    assert!((influence(&p, &soft, Vec2::new(0.0, 0.0)) - 0.25).abs() < 1e-6);
    // …e o falloff MULTIFRAME também (é isto que o mantém vivo até a T5.7 chegar:
    // se ele virasse um parâmetro morto, retrofitá-lo custaria os oito pincéis).
    let far_frame = ReshapeParams {
        frame_falloff: 0.5,
        ..p
    };
    assert!((influence(&far_frame, &s, Vec2::new(0.0, 0.0)) - 0.5).abs() < 1e-6);
}

/// **A máscara é congelada no pen-down.** Um traço que aparece no meio do gesto
/// (tween, outra ferramenta, undo) NÃO é esculpido — arrastar o pincel para cima
/// dele não o recruta. *A máscara define O QUE; o traço define QUANTO.*
#[test]
fn the_mask_is_frozen_at_pen_down() {
    let mut strokes = vec![line(5)];
    let p = params(ReshapeKind::Push, 10.0);
    let mut sess = Session::begin(&strokes, &p, &sample(Vec2::new(2.0, 0.0), Vec2::ZERO));

    // Um traço NOVO entra no desenho no meio do gesto, bem debaixo do pincel.
    strokes.push(line(5));
    let before = strokes[1].positions().to_vec();
    sess.apply(
        &mut strokes,
        &p,
        &sample(Vec2::new(2.0, 0.0), Vec2::new(0.0, 3.0)),
    );

    assert_eq!(
        strokes[1].positions(),
        &before[..],
        "o traco que nasceu DEPOIS do pen-down foi esculpido"
    );
    assert_ne!(
        strokes[0].positions()[2],
        Vec2::new(2.0, 0.0),
        "e o traco original tem de ter sido esculpido"
    );
}

/// **A ESCULTURA MOVE A COR JUNTO COM A LINHA** (smoke do Enio 2026-07-13, com o
/// Suzanne do Blender ao lado: *"o fill é atualizado em tempo real como se line e fill
/// fossem um só"*).
///
/// No Grease Pencil o sculpt edita **todas** as curvas (`retrieve_editable_strokes` só
/// exclui material travado) e o preenchimento é a **triangulação dos pontos da própria
/// curva** — mover os pontos re-tria o fill no mesmo frame. Um pincel que movesse a
/// linha e deixasse a cor para trás não seria escultura: seria uma ferramenta de
/// quebrar o desenho.
///
/// Mutação que sangra: volte a pular as regiões na máscara (`!is_region`) e a cor fica
/// exatamente onde estava.
#[test]
fn sculpting_carries_the_filled_region_along_with_the_line() {
    let mut fill = line(5);
    fill.hide_stroke = true;
    fill.fill = Some(ph2d_flip::Fill {
        color: Rgba::WHITE,
        opacity: 1.0,
    });
    let mut strokes = vec![fill, line(5)]; // a região + a linha
    let p = params(ReshapeKind::Push, 10.0);
    let changed = stroke_once(
        &mut strokes,
        &p,
        &sample(Vec2::new(2.0, 0.0), Vec2::new(0.0, 5.0)),
    );
    assert!(changed);
    assert!(
        strokes[0].positions()[2].y > 1.0,
        "a REGIAO ficou para tras: a cor descola da linha"
    );
    assert!(
        strokes[1].positions()[2].y > 1.0,
        "e a linha tem de ter andado tambem"
    );
}

/// **E o BURACO vai junto** — senão a rosquinha se abre.
///
/// Os buracos vivem fora do SoA (`FlipStroke.holes`), e é exatamente por isso que eles
/// são fáceis de esquecer: nenhum laço sobre `positions_mut()` os alcança. Um "O" cujo
/// contorno anda e cujo furo fica parado vira uma mancha sólida deslocada.
#[test]
fn sculpting_carries_the_holes_of_a_region_too() {
    for kind in [ReshapeKind::Push, ReshapeKind::Grab] {
        let mut region = line(5);
        region.hide_stroke = true;
        region.fill = Some(ph2d_flip::Fill {
            color: Rgba::WHITE,
            opacity: 1.0,
        });
        region.holes = vec![vec![
            Vec2::new(1.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(3.0, 1.0),
            Vec2::new(1.0, 1.0),
        ]];
        let mut strokes = vec![region];
        let p = params(kind, 10.0);
        let s = sample(Vec2::new(2.0, 0.0), Vec2::new(0.0, 4.0));
        let mut sess = Session::begin(&strokes, &p, &s);
        sess.apply(&mut strokes, &p, &s);
        let hole = &strokes[0].holes[0];
        assert!(
            hole.iter().all(|h| h.y > 0.5),
            "{kind:?}: o BURACO ficou para tras (a rosquinha se abriu): {hole:?}"
        );
    }
}

/// **Mas os pincéis de ATRIBUTO não tocam a região.** O `width` do contorno de um fill
/// não é a espessura de uma linha: é a **dilatação da cor por baixo do line-art**
/// (BUGS #15). Engrossá-lo com o Thicken empurraria a cor para FORA do desenho — um
/// pincel de espessura que produz vazamento de cor.
#[test]
fn the_attribute_brushes_do_not_touch_a_region() {
    for kind in [ReshapeKind::Thickness, ReshapeKind::Strength] {
        let mut region = line(5);
        region.hide_stroke = true;
        region.fill = Some(ph2d_flip::Fill {
            color: Rgba::WHITE,
            opacity: 1.0,
        });
        let widths = region.widths().to_vec();
        let ops = region.opacities().to_vec();
        let mut strokes = vec![region];
        let p = params(kind, 10.0);
        let changed = stroke_once(&mut strokes, &p, &sample(Vec2::new(2.0, 0.0), Vec2::ZERO));
        assert!(!changed, "{kind:?}: mexeu na regiao");
        assert_eq!(
            strokes[0].widths(),
            &widths[..],
            "{kind:?}: mudou a dilatacao"
        );
        assert_eq!(strokes[0].opacities(), &ops[..]);
    }
}

/// **Determinismo (HR-5):** a mesma sequência de amostras dá o mesmo resultado, bit a
/// bit — inclusive no Randomize (o hash é função de `(amostra, índice)`).
#[test]
fn the_same_gesture_gives_the_same_geometry() {
    for kind in ReshapeKind::ALL {
        let run = || {
            let mut strokes = vec![wobbly(9)];
            let p = params(kind, 4.0);
            let mut sess = Session::begin(&strokes, &p, &sample(Vec2::new(4.0, 0.0), Vec2::ZERO));
            for i in 0..5 {
                let s = sample(Vec2::new(4.0 + i as f32 * 0.5, 0.0), Vec2::new(0.5, 0.2));
                sess.apply(&mut strokes, &p, &s);
            }
            strokes
        };
        assert_eq!(run(), run(), "{kind:?} nao e deterministico");
    }
}

// ─────────────────────────── os pincéis ───────────────────────────

/// **Smooth (T5.2):** alisa o tremor **sem encolher as pontas**.
#[test]
fn smooth_flattens_the_wobble_without_shortening_the_stroke() {
    let mut strokes = vec![wobbly(21)];
    let before = strokes[0].positions().to_vec();
    let p = params(ReshapeKind::Smooth, 100.0); // alcança o traço todo
    let s = sample(Vec2::new(10.0, 0.0), Vec2::ZERO);
    let mut sess = Session::begin(&strokes, &p, &s);
    for _ in 0..4 {
        sess.apply(&mut strokes, &p, &s);
    }
    let after = strokes[0].positions();

    let wob_before: f32 = before.iter().map(|p| p.y.abs()).sum();
    let wob_after: f32 = after.iter().map(|p| p.y.abs()).sum();
    assert!(
        wob_after < wob_before * 0.5,
        "o tremor tinha de cair: {wob_before:.2} -> {wob_after:.2}"
    );
    assert_eq!(
        after[0], before[0],
        "a ponta inicial ANDOU (o traco encolheu)"
    );
    assert_eq!(
        after[20], before[20],
        "a ponta final ANDOU (o traco encolheu)"
    );
}

/// **Push (T5.3):** os pontos sob o pincel andam COM o cursor; os de fora, não.
#[test]
fn push_carries_the_points_under_the_brush_and_no_others() {
    let mut strokes = vec![line(11)];
    let p = params(ReshapeKind::Push, 2.0);
    let s = sample(Vec2::new(5.0, 0.0), Vec2::new(0.0, 1.0));
    assert!(stroke_once(&mut strokes, &p, &s));
    let pts = strokes[0].positions();
    assert!(
        pts[5].y > 0.9,
        "o ponto sob o cursor andou quase tudo: {}",
        pts[5].y
    );
    assert!(
        pts[4].y > 0.0 && pts[4].y < pts[5].y,
        "o vizinho anda MENOS"
    );
    assert_eq!(pts[0].y, 0.0, "o ponto fora do raio nao pode andar");
    assert_eq!(pts[10].y, 0.0);
}

/// **Push parado não faz nada** — a dose é o MOVIMENTO (`mouse_delta · influence`).
#[test]
fn push_does_nothing_when_the_cursor_is_still() {
    let mut strokes = vec![line(5)];
    let p = params(ReshapeKind::Push, 10.0);
    assert!(!stroke_once(
        &mut strokes,
        &p,
        &sample(Vec2::new(2.0, 0.0), Vec2::ZERO)
    ));
}

/// **Grab (T5.3) — a distinção de UX que o plano exige.**
///
/// O Grab **segura** o que estava sob o pincel no pen-down e o carrega, mesmo que o
/// cursor se afaste; o Push **varre** — quando o cursor sai de perto, o ponto para.
/// É o mesmo gesto, e o resultado é oposto: por isso os dois existem.
#[test]
fn grab_keeps_carrying_what_it_grabbed_while_push_lets_it_go() {
    // O mesmo caminho: o cursor sai de cima do ponto 5 e vai para longe.
    let path = [
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];
    let mut carried = 0.0f32;
    let mut swept = 0.0f32;
    for kind in [ReshapeKind::Grab, ReshapeKind::Push] {
        let mut strokes = vec![line(11)];
        let p = params(kind, 1.5);
        let start = sample(Vec2::new(5.0, 0.0), Vec2::ZERO);
        let mut sess = Session::begin(&strokes, &p, &start);
        let mut cursor = start.pos;
        for d in path {
            cursor += d;
            sess.apply(&mut strokes, &p, &sample(cursor, d));
        }
        let y = strokes[0].positions()[5].y;
        if kind == ReshapeKind::Grab {
            carried = y;
        } else {
            swept = y;
        }
    }
    assert!(
        carried > 3.5,
        "o Grab tinha de CARREGAR o ponto os 4 passos: y = {carried:.2}"
    );
    assert!(
        swept < 1.5,
        "o Push tinha de LARGAR o ponto quando o cursor se afastou: y = {swept:.2}"
    );
    assert!(carried > swept * 2.0, "os dois pinceis ficaram iguais");
}

/// **Pinch (T5.5):** aperta em direção ao cursor; com Ctrl, infla. E é LENTO
/// (`influence²/25` = no máximo 4% por amostra) — não colapsa a forma num ponto.
#[test]
fn pinch_pulls_towards_the_cursor_and_inverted_pushes_away() {
    let dist = |st: &FlipStroke, c: Vec2| -> f32 {
        let d = st.positions()[0] - c;
        (d.x * d.x + d.y * d.y).sqrt()
    };
    let cursor = Vec2::new(5.0, 0.0);
    let mut strokes = vec![line(11)];
    let before = dist(&strokes[0], cursor);

    let p = params(ReshapeKind::Pinch, 10.0);
    let s = sample(cursor, Vec2::ZERO);
    let mut sess = Session::begin(&strokes, &p, &s);
    for _ in 0..10 {
        sess.apply(&mut strokes, &p, &s);
    }
    let after = dist(&strokes[0], cursor);
    assert!(
        after < before,
        "o Pinch tinha de APROXIMAR: {before} -> {after}"
    );
    assert!(
        after > before * 0.5,
        "e devagar (10 amostras nao podem colapsar a forma): {after}"
    );

    let mut inflated = vec![line(11)];
    let pi = ReshapeParams { invert: true, ..p };
    let mut sess = Session::begin(&inflated, &pi, &s);
    for _ in 0..10 {
        sess.apply(&mut inflated, &pi, &s);
    }
    assert!(
        dist(&inflated[0], cursor) > before,
        "invertido, o Pinch tinha de AFASTAR"
    );
}

/// **Twist (T5.5):** rotação RÍGIDA ao redor do cursor — a distância de cada ponto ao
/// cursor é preservada (é o que separa torcer de arrastar), e o Ctrl inverte o SENTIDO.
///
/// O sentido se mede pelo produto vetorial `radial × radial'` (positivo =
/// anti-horário), **não** pelo sinal de `y`: o ponto está à ESQUERDA do cursor, e ali
/// um giro anti-horário desce o `y`. (A 1ª versão deste teste assertava `y > 0` e
/// acusou um código que estava certo — a armadilha do BUGS #13: *confira que o teste
/// descreve o que você acha que descreve*.)
#[test]
fn twist_rotates_rigidly_around_the_cursor() {
    let cursor = Vec2::new(5.0, 0.0);
    let radius_of = |st: &FlipStroke, i: usize| -> f32 {
        let d = st.positions()[i] - cursor;
        (d.x * d.x + d.y * d.y).sqrt()
    };
    /// Sentido do giro do ponto `i` ao redor do cursor: `radial × radial'`.
    fn turn(before: Vec2, after: Vec2, cursor: Vec2) -> f32 {
        let (a, b) = (before - cursor, after - cursor);
        a.x * b.y - a.y * b.x
    }

    let p = params(ReshapeKind::Twist, 10.0);
    let s = sample(cursor, Vec2::ZERO);
    let mut strokes = vec![line(11)];
    let before = strokes[0].positions()[0];
    let r_before = radius_of(&strokes[0], 0);
    let mut sess = Session::begin(&strokes, &p, &s);
    for _ in 0..30 {
        sess.apply(&mut strokes, &p, &s);
    }
    let after = strokes[0].positions()[0];
    assert!(
        turn(before, after, cursor) > 0.1,
        "30 amostras de 1 grau tinham de girar o ponto (anti-horario)"
    );
    assert!(
        (radius_of(&strokes[0], 0) - r_before).abs() < 0.01,
        "a rotacao nao e rigida: o raio ate o cursor mudou"
    );

    let mut back = vec![line(11)];
    let pi = ReshapeParams { invert: true, ..p };
    let mut sess = Session::begin(&back, &pi, &s);
    for _ in 0..30 {
        sess.apply(&mut back, &pi, &s);
    }
    assert!(
        turn(before, back[0].positions()[0], cursor) < -0.1,
        "invertido, o Twist tinha de girar para o OUTRO lado"
    );
}

/// **Thickness (T5.4):** aditivo — engrossa, e invertido afina até ZERO (nunca
/// negativo). Um passo PROPORCIONAL nunca sairia do zero; o aditivo recupera.
#[test]
fn thickness_adds_and_inverted_thins_down_to_zero() {
    let mut strokes = vec![line(11)];
    let p = params(ReshapeKind::Thickness, 3.0);
    let s = sample(Vec2::new(5.0, 0.0), Vec2::ZERO);
    let mut sess = Session::begin(&strokes, &p, &s);
    for _ in 0..5 {
        sess.apply(&mut strokes, &p, &s);
    }
    let w = strokes[0].widths();
    assert!(w[5] > 6.0 + 5.0 * 0.5, "engrossou: {}", w[5]);
    assert_eq!(w[0], 6.0, "fora do raio, intacto");

    // Invertido: afina — e PARA no zero (largura negativa não existe).
    let pi = ReshapeParams { invert: true, ..p };
    let mut sess = Session::begin(&strokes, &pi, &s);
    for _ in 0..100 {
        sess.apply(&mut strokes, &pi, &s);
    }
    assert_eq!(strokes[0].widths()[5], 0.0, "afinou ate zero, sem negativo");
}

/// **Strength (T5.4):** a opacidade sobe/desce em passos de `0.125 · influência`, e
/// fica clampada em `[0,1]`.
#[test]
fn strength_raises_and_inverted_erases_the_opacity() {
    let mut strokes = vec![line(11)];
    for o in strokes[0].opacities_mut() {
        *o = 0.5;
    }
    let p = params(ReshapeKind::Strength, 3.0);
    let s = sample(Vec2::new(5.0, 0.0), Vec2::ZERO);
    let mut sess = Session::begin(&strokes, &p, &s);
    sess.apply(&mut strokes, &p, &s);
    let o = strokes[0].opacities();
    assert!((o[5] - 0.625).abs() < 1e-5, "um passo = +0,125: {}", o[5]);
    assert_eq!(o[0], 0.5, "fora do raio, intacto");

    // Sobe até 1 e para (clamp).
    let mut sess = Session::begin(&strokes, &p, &s);
    for _ in 0..50 {
        sess.apply(&mut strokes, &p, &s);
    }
    assert_eq!(strokes[0].opacities()[5], 1.0);

    // Invertido: apaga até 0 e para.
    let pi = ReshapeParams { invert: true, ..p };
    let mut sess = Session::begin(&strokes, &pi, &s);
    for _ in 0..50 {
        sess.apply(&mut strokes, &pi, &s);
    }
    assert_eq!(strokes[0].opacities()[5], 0.0);
}

/// **Randomize (T5.6):** o ruído é **perpendicular ao movimento** — a linha ondula em
/// vez de engrossar. Aqui: cursor andando em **y**, então o desvio tem de aparecer em
/// **x** (e a componente ao longo do movimento fica intacta).
#[test]
fn randomize_jitters_only_perpendicular_to_the_mouse_movement() {
    let mut strokes = vec![line(11)];
    let p = params(ReshapeKind::Randomize, 10.0);
    // O cursor sobe (+y): o `sideways` é o eixo x.
    let s = sample(Vec2::new(5.0, 0.0), Vec2::new(0.0, 1.0));
    let mut sess = Session::begin(&strokes, &p, &s);
    for _ in 0..5 {
        sess.apply(&mut strokes, &p, &s);
    }
    let pts = strokes[0].positions();
    let moved_x: f32 = (0..11).map(|i| (pts[i].x - i as f32).abs()).sum();
    let moved_y: f32 = pts.iter().map(|p| p.y.abs()).sum();
    assert!(
        moved_x > 0.5,
        "o jitter perpendicular nao aconteceu: {moved_x}"
    );
    assert_eq!(
        moved_y, 0.0,
        "o jitter vazou para a direcao DO MOVIMENTO (tinha de ser so perpendicular)"
    );
}

/// **Randomize é re-semeado por AMOSTRA:** parado, o pincel continua vibrando (passeio
/// browniano). Uma semente por GESTO congelaria o deslocamento e o efeito morreria ao
/// parar o cursor — que é exatamente quando o artista quer bagunçar mais.
#[test]
fn randomize_keeps_wandering_while_the_cursor_is_held_still() {
    let mut strokes = vec![line(11)];
    let p = params(ReshapeKind::Randomize, 10.0);
    let s = sample(Vec2::new(5.0, 0.0), Vec2::new(0.0, 1.0));
    let mut sess = Session::begin(&strokes, &p, &s);

    sess.apply(&mut strokes, &p, &s);
    let after_1 = strokes[0].positions().to_vec();
    sess.apply(&mut strokes, &p, &s); // MESMA amostra, cursor parado
    let after_2 = strokes[0].positions().to_vec();

    assert_ne!(
        after_1, after_2,
        "com a semente por GESTO o traco congelaria: a 2a amostra tem de mexer de novo"
    );
}

/// **O auto-masking pela SELEÇÃO** (W6 — o Edit Mode).
///
/// É a promessa central do Edit Mode: num desenho cheio, alisar uma linha não pode alisar
/// o que passa perto dela. Se HÁ seleção, o gesto toca só os selecionados.
///
/// E **seleção vazia = tudo** — é essa metade da regra que torna a feature aditiva: quem
/// nunca abriu o Edit Mode não vê diferença nenhuma no Sculpt.
///
/// Mutação que sangra: tire o `.filter(|(_, st)| !any_selected || st.selected)` do
/// `Session::begin` e a máscara volta a pegar os dois traços.
#[test]
fn the_sculpt_touches_only_the_selected_strokes_when_there_is_a_selection() {
    let params = ReshapeParams {
        kind: ReshapeKind::Smooth,
        radius: 1000.0, // o pincel cobre os DOIS traços — só a seleção pode separá-los
        strength: 1.0,
        ..Default::default()
    };
    let s0 = sample(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

    // Sem seleção: a máscara pega os dois (o comportamento de sempre).
    let mut strokes = vec![wobbly(12), wobbly(12)];
    let s = Session::begin(&strokes, &params, &s0);
    assert_eq!(
        s.mask.len(),
        2,
        "sem selecao, o Sculpt tem de continuar pegando TUDO (a feature e aditiva)"
    );

    // Com um selecionado: só ele.
    strokes[1].selected = true;
    let s = Session::begin(&strokes, &params, &s0);
    assert_eq!(
        s.mask.len(),
        1,
        "com selecao, o Sculpt tem de tocar SO o selecionado"
    );
    assert_eq!(
        s.mask.first().copied(),
        Some(1),
        "o Sculpt esculpiu o traco ERRADO (o nao-selecionado)"
    );
}

/// 🔴 **A máscara por PONTO (W8): com seleção PARCIAL, os pontos não-selecionados saem
/// BYTE-IDÊNTICOS do gesto** — posição, largura e opacidade — e os selecionados mudam.
///
/// É a "máscara fina" do domínio Point: o pincel roda livre e o `apply` devolve o que
/// ele não podia tocar (é o que faz valer para os oito pincéis sem mudar nenhum).
///
/// Mutação que sangra: tirar a restauração do `apply` (o `saved`) — o Smooth alisa o
/// traço inteiro e os congelados se movem.
#[test]
fn a_partial_point_selection_freezes_the_unselected_points() {
    let p = params(ReshapeKind::Smooth, 1000.0); // o pincel cobre tudo
    let s0 = sample(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
    let mut strokes = vec![wobbly(12)];
    // Seleciona SÓ a metade da frente (índices 0..6).
    for i in 0..6 {
        strokes[0].set_point_selected(i, true);
    }
    let before = strokes[0].positions().to_vec();
    let widths_before = strokes[0].widths().to_vec();

    let mut sess = Session::begin(&strokes, &p, &s0);
    assert!(sess.apply(&mut strokes, &p, &s0), "o gesto tem de agir");

    let after = strokes[0].positions().to_vec();
    // (a) Os NÃO-selecionados (6..12) não se moveram um bit.
    for i in 6..12 {
        assert_eq!(
            before[i], after[i],
            "o ponto congelado {i} se moveu — a mascara por ponto nao segurou"
        );
    }
    assert_eq!(
        widths_before,
        strokes[0].widths(),
        "largura de ponto congelado mudou"
    );
    // (b) Ao menos um SELECIONADO mudou (senão a máscara virou um no-op e o gate é
    // falso-zero — a lição do BUGS #17).
    assert!(
        (0..6).any(|i| before[i] != after[i]),
        "nenhum ponto selecionado se moveu — o gesto nao agiu onde devia"
    );
}

/// **O Grab com seleção parcial só agarra os pontos selecionados — e os BURACOS ficam**
/// (não têm seleção própria; só andam com o traço inteiro).
#[test]
fn a_partial_point_selection_limits_what_the_grab_grabs() {
    let p = params(ReshapeKind::Grab, 1000.0);
    let s0 = sample(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
    let mut st = wobbly(8);
    st.holes
        .push(vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)]);
    st.set_point_selected(2, true);
    st.set_point_selected(3, true);
    let strokes = vec![st];
    let sess = Session::begin(&strokes, &p, &s0);
    assert_eq!(
        sess.grabbed(),
        2,
        "o Grab parcial tem de agarrar SO os 2 pontos selecionados (sem os buracos)"
    );
}
