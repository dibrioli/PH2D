//! Testes da FRONTEIRA da escultura (o que o solver não pode provar sozinho): a
//! conversão de unidades e o roteamento dos oito pincéis. Módulo-irmão pelo cap de LOC.

use super::*;
use ph2d_tool_flip::ReshapeKind as ToolKind;

fn style(kind: ToolKind, width_px: f64, strength: f32) -> FlipStyleSnapshot {
    FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Reshape,
        reshape: kind,
        width_px,
        opacity: strength,
        ..Default::default()
    }
}

/// 🔴 **O raio do pincel é METADE do Size em MUNDO — e o ZOOM não entra na conta**
/// (§4.C.6).
///
/// Duas conversões se compõem, e nenhuma é opcional: o Size é o DIÂMETRO em unidades de
/// mundo (`size_to_world`, a mesma porta do traço e da borracha) e a escala do objeto
/// (ADR-0111) recua o mundo para o espaço local do desenho. Errar qualquer uma dá um
/// pincel que não alcança nada (ou que alcança o desenho inteiro) — foi essa classe de
/// erro que matou o balde no produto (BUGS #10a).
///
/// A **terceira** conversão morreu em §4.C.6: o `px_to_world` do zoom. O pincel de
/// escultura esculpe uma porção fixa de ARTE, então aproximar a câmera não muda o que ele
/// pega — só o tamanho com que você o vê. (O `px_to_local` FICA no `ReshapeParams`, mas
/// só para o DELTA do arrasto, que é gesto de tela por definição.)
///
/// Mutação que sangra: devolver o raio ao `× px_to_local` — o assert do zoom-invariante
/// cai na hora.
#[test]
fn the_brush_radius_is_half_the_size_in_world_and_ignores_the_zoom() {
    let px_to_world = 10.0f32 / 1080.0; // câmera default numa janela 1080p
    let s = style(ToolKind::Smooth, 40.0, 1.0);
    let want = ph2d_tool_flip::size_to_world(40.0) * 0.5;

    // Objeto na identidade: raio = metade do Size, em mundo.
    let p = params_from(&s, px_to_world, &Xform::IDENTITY, false);
    assert!(
        (p.radius - want).abs() < 1e-6,
        "raio {} != {want} (metade do Size, em mundo)",
        p.radius
    );
    assert!((p.px_to_local - px_to_world).abs() < 1e-9);

    // **O ZOOM não muda o raio**: 10× mais perto, o mesmo pedaço de arte.
    let p_zoom = params_from(&s, px_to_world * 0.1, &Xform::IDENTITY, false);
    assert!(
        (p_zoom.radius - want).abs() < 1e-6,
        "o zoom mexeu no raio ({} != {want}): o pincel voltou a ser de TELA",
        p_zoom.radius
    );

    // Objeto ESCALADO 2× pelo gizmo: o `world_to_local` dele encolhe por 0,5, então o
    // MESMO pincel cobre METADE das unidades locais.
    let scaled = Xform([0.5, 0.0, 0.0, 0.5, 0.0, 0.0]);
    let p2 = params_from(&s, px_to_world, &scaled, false);
    assert!(
        (p2.radius - want * 0.5).abs() < 1e-6,
        "a escala do objeto tem de recuar o raio: {} != {}",
        p2.radius,
        want * 0.5
    );
}

/// A força do pincel é o **Strength** do painel (o `opacity` da tool — a borracha faz
/// igual), e o Ctrl é o invert.
#[test]
fn strength_and_ctrl_reach_the_solver() {
    let s = style(ToolKind::Thickness, 20.0, 0.25);
    let p = params_from(&s, 0.01, &Xform::IDENTITY, false);
    assert!((p.strength - 0.25).abs() < 1e-6, "a forca nao chegou");
    assert!(!p.invert);
    let p = params_from(&s, 0.01, &Xform::IDENTITY, true);
    assert!(p.invert, "o Ctrl nao chegou ao solver");
}

/// **Os oito pincéis chegam ao solver — todos.** Um `match` de tradução com um braço
/// errado é invisível para o compilador (os dois enums têm os mesmos nomes) e o
/// usuário só descobre clicando: escolhe Twist e o traço engrossa. Aqui cada variante
/// da tool é mapeada e conferida contra a do solver, uma a uma.
#[test]
fn every_tool_brush_maps_to_its_own_solver_brush() {
    let pairs = [
        (ToolKind::Smooth, ReshapeKind::Smooth),
        (ToolKind::Push, ReshapeKind::Push),
        (ToolKind::Grab, ReshapeKind::Grab),
        (ToolKind::Pinch, ReshapeKind::Pinch),
        (ToolKind::Twist, ReshapeKind::Twist),
        (ToolKind::Thickness, ReshapeKind::Thickness),
        (ToolKind::Strength, ReshapeKind::Strength),
        (ToolKind::Randomize, ReshapeKind::Randomize),
    ];
    for (tool, solver) in pairs {
        let p = params_from(&style(tool, 20.0, 1.0), 0.01, &Xform::IDENTITY, false);
        assert_eq!(p.kind, solver, "{tool:?} caiu no pincel errado");
    }
    // E o mapa é uma BIJEÇÃO: oito pincéis distintos, nenhum colapsado no vizinho.
    let mut kinds: Vec<ReshapeKind> = pairs.iter().map(|(t, _)| kind_of(*t)).collect();
    kinds.sort_by_key(|k| format!("{k:?}"));
    kinds.dedup();
    assert_eq!(
        kinds.len(),
        8,
        "dois pinceis da tool caem no MESMO do solver"
    );
}

/// O falloff multiframe sai daqui em `1.0` (a tira seleciona um quadro só) — e não
/// em `0.0`, que zeraria toda influência e faria os oito pincéis não fazerem nada.
#[test]
fn the_multiframe_falloff_defaults_to_the_active_frame() {
    let p = params_from(
        &style(ToolKind::Push, 20.0, 1.0),
        0.01,
        &Xform::IDENTITY,
        false,
    );
    assert_eq!(p.frame_falloff, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// A COSTURA COM O DOCUMENTO (o que `params_from` sozinho não prova): o autokey.
// ─────────────────────────────────────────────────────────────────────────────

use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipStroke, Hold, KeyKind, Point, Rgba};

/// Um documento com uma linha reta de 5 pontos numa chave no quadro 0.
fn doc_with_line() -> (FlipDoc, LayerId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("O");
    let obj = doc.object_mut(oid).unwrap();
    let l = obj.add_layer("L");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut s = FlipStroke::new();
    for x in 0..5 {
        s.push_point(Point {
            pos: Vec2::new(x as f32, 0.0),
            width: 6.0,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
    }
    obj.drawing_mut(d).unwrap().strokes.push(s);
    (doc, l)
}

fn strokes_at(doc: &FlipDoc, l: LayerId, frame: i32) -> Vec<FlipStroke> {
    let obj = doc.objects().first().unwrap();
    let did = obj.layer(l).unwrap().drawing_at(frame).unwrap();
    obj.drawing(did).unwrap().strokes.clone()
}

/// **O gesto muda o desenho de verdade** (e não só o solver em memória): um Push no
/// meio da linha empurra os pontos do desenho ATIVO.
#[test]
fn a_sculpt_gesture_edits_the_active_drawing() {
    let (mut doc, l) = doc_with_line();
    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 2.0,
        strength: 1.0,
        ..Default::default()
    };
    let mut strip = crate::flip_strip::FlipStrip::default();
    let ph = Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS));
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let (oid_g, mut targets) = reshape_begin(&mut doc, &ph, Some(l), &mut strip, &p, &s, false)
        .expect("ha desenho na camada ativa");
    assert!(
        strokes_at(&doc, l, 0)[0].positions()[2].y > 0.5,
        "o pen-down ja esculpe"
    );

    // E um move continua o gesto no MESMO desenho.
    reshape_sample(&mut doc, oid_g, &mut targets, &p, &s);
    assert!(
        strokes_at(&doc, l, 0)[0].positions()[2].y > 1.0,
        "a 2a amostra tinha de somar (a dose e por AMOSTRA)"
    );
}

/// **Uma camada TRAVADA recusa a escultura** — e o chamador tem como saber (o `None`
/// vira o toast). Sem isto, o pincel "funcionaria" numa camada travada e o cadeado
/// seria decorativo.
#[test]
fn a_locked_layer_refuses_the_sculpt() {
    let (mut doc, l) = doc_with_line();
    let oid = doc.objects().first().unwrap().id;
    doc.object_mut(oid).unwrap().layer_mut(l).unwrap().locked = true;

    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 5.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let got = reshape_begin(
        &mut doc,
        &Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS)),
        Some(l),
        &mut crate::flip_strip::FlipStrip::default(),
        &p,
        &s,
        false,
    );
    assert!(got.is_none(), "a camada travada tinha de RECUSAR");
    assert_eq!(
        strokes_at(&doc, l, 0)[0].positions()[2],
        Vec2::new(2.0, 0.0),
        "e nada pode ter sido esculpido"
    );
}

/// **O autokey da escultura é `Modify` — a chave nova nasce DUPLICATA, nunca em
/// branco.**
///
/// É a regra que a W3 pagou caro para aprender (`docs/Flip/05 §4`): a caneta cria
/// chave em branco; borracha e ESCULTURA duplicam. Se o Reshape criasse uma chave
/// vazia no rabo de um hold, o usuário esculpiria o nada — enquanto o desenho que ele
/// VÊ continuaria intacto num quadro anterior.
///
/// Aqui: uma chave só, no quadro 0, com hold; o playhead está no quadro 5 (dentro do
/// hold, sem chave própria) e o AutoKey está ARMADO. Esculpir tem de produzir uma
/// chave nova no 5 **com o traço dentro** — e esculpido.
#[test]
fn sculpting_inside_a_hold_duplicates_the_visible_drawing_never_a_blank_one() {
    let (mut doc, l) = doc_with_line();
    let mut strip = crate::flip_strip::FlipStrip {
        autokey: true,
        ..Default::default()
    };

    // O playhead no quadro 5 — dentro do hold da chave do quadro 0.
    //
    // **O fps é o do OBJETO** (`DEFAULT_FPS` = 24), não um número conveniente: a 1ª
    // versão deste teste semeou o playhead a 12 fps, o objeto leu o tempo a 24, o
    // gesto esculpiu a chave do quadro **10** — e o teste, lendo o 5, acusou o código
    // de não esculpir. (`feedback_test_with_product_numbers_not_convenient_ones`.)
    let fps = f64::from(ph2d_flip::DEFAULT_FPS);
    let mut ph = Playhead::new(1.0 / fps);
    ph.seek_frame(5, fps);

    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 2.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    reshape_begin(&mut doc, &ph, Some(l), &mut strip, &p, &s, false)
        .expect("o autokey cria a chave");

    let here = strokes_at(&doc, l, 5);
    assert_eq!(
        here.len(),
        1,
        "a chave nova nasceu EM BRANCO: o usuario esculpiria o nada"
    );
    assert!(
        here[0].positions()[2].y > 0.5,
        "a duplicata nasceu, mas o gesto nao a esculpiu"
    );
    // E o quadro 0 (o original) fica INTACTO — a duplicata é que foi esculpida.
    assert_eq!(
        strokes_at(&doc, l, 0)[0].positions()[2],
        Vec2::new(2.0, 0.0),
        "a escultura vazou para o desenho ORIGINAL do hold"
    );
}

/// **O TRAÇO PREENCHIDO: linha e cor são UMA geometria** (a resposta ao Suzanne).
///
/// No Grease Pencil, um material com `stroke + fill` faz o preenchimento ser a
/// triangulação dos pontos da PRÓPRIA curva — por isso, lá, "line e fill parecem um só":
/// não há o que atualizar, é a mesma geometria. Aqui: um traço desenhado com Filled
/// carrega o `fill`, e esculpir a linha move o preenchimento **exatamente** junto,
/// porque não há um segundo objeto para ficar para trás.
///
/// Mutação que sangra: pare de setar o `fill` no `build_stroke` (o traço vira só linha) —
/// ou volte a pular regiões na máscara do solver, e o gesto deixa de mover a cor.
#[test]
fn a_filled_stroke_is_one_geometry_so_sculpting_the_line_moves_the_colour() {
    use ph2d_tool_flip::FlipMode;

    // Um traço desenhado com Shape = Filled.
    let style = FlipStyleSnapshot {
        mode: FlipMode::Draw,
        draw_filled: true,
        fill_color: [200, 150, 90, 255],
        width_px: 8.0,
        // **Smoothing ZERO de propósito.** Com o default (0,5) o active smoothing roda
        // 12 iterações de kernel sobre as amostras — e num "quadrado" de quatro pontos
        // isso colapsa os dois interiores contra a corda, tirando-os de baixo do pincel.
        // O teste ficaria vermelho por culpa DELE, não do código (a armadilha do
        // BUGS #13). O que se afirma aqui é o preenchimento e a escultura, não o
        // alisamento — que tem gates próprios.
        smoothing: 0.0,
        ..Default::default()
    };
    let pts = [
        Vec2::new(0.0, 0.0),
        Vec2::new(4.0, 0.0),
        Vec2::new(4.0, 4.0),
        Vec2::new(0.0, 4.0),
    ];
    let stroke = crate::flip_draw::stroke_from_samples(&style, &pts, &[1.0; 4], &Xform::IDENTITY);
    assert!(
        stroke.fill.is_some(),
        "o traco Filled tem de carregar o proprio preenchimento (o material stroke+fill do GP)"
    );
    assert!(stroke.closed, "uma forma preenchida e fechada");
    assert!(
        !stroke.hide_stroke,
        "e a LINHA continua sendo desenhada: e o mesmo traco, nao uma regiao"
    );

    // Agora esculpe: um Push no meio de uma aresta. A linha anda — e a cor, que é a
    // triangulação DESTES pontos, anda com ela por construção.
    let mut strokes = vec![stroke];
    // ⚠️ **O ponto se ACHA pela POSIÇÃO, nunca pelo índice** (2026-07-27). Este `before`
    // era `positions()[1]` — *"o segundo ponto autorado"*, que era a quina em `(4,0)`
    // enquanto o `stroke_from_samples` entregava os quatro pontos crus. A reamostragem
    // suave (T2.8) densifica o traço pelo arco (medido: **4 → 73 pontos**), então o índice
    // 1 virou `(0.167, 0)` — uma amostra no COMEÇO, a 4 unidades do Push de raio 3. O
    // teste ficou vermelho afirmando o certo sobre o ponto errado.
    //
    // O que ele afirma é *"o gesto move a linha, e a cor vem com ela"* — uma propriedade
    // do ponto **sob o pincel**, não do índice 1. Achá-lo pela distância ao centro do
    // gesto sobrevive a qualquer densidade de amostragem, hoje e na próxima vez que ela
    // mudar ([[reference_topic_fixture_discipline]]).
    let center = Vec2::new(4.0, 0.0);
    let hit = (0..strokes[0].positions().len())
        .min_by(|&a, &b| {
            let d = |i: usize| {
                let q = strokes[0].positions()[i];
                (q.x - center.x).powi(2) + (q.y - center.y).powi(2)
            };
            d(a).total_cmp(&d(b))
        })
        .expect("o traco tem pontos");
    let before = strokes[0].positions()[hit];
    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 3.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(4.0, 0.0),
        delta: Vec2::new(2.0, 0.0),
        pressure: 1.0,
    };
    let mut sess = ph2d_flip_reshape::Session::begin(&strokes, &p, &s);
    assert!(sess.apply(&mut strokes, &p, &s), "o gesto tinha de mexer");
    assert!(
        strokes[0].positions()[hit].x > before.x + 1.0,
        "a linha nao andou"
    );
    assert!(
        strokes[0].fill.is_some(),
        "o preenchimento sobreviveu a escultura (ele E o traco)"
    );
}

/// 🔴 **O sculpt MULTIFRAME: o mesmo gesto esculpe N quadros** (W7).
///
/// É a feature-assinatura do GP para animação. Com chaves marcadas na tira, um traço do
/// pincel age em todas — e o quadro ATIVO recebe influência cheia.
///
/// Mutação que sangra: faça o `reshape_begin` voltar a resolver UM alvo (o `target_drawing`
/// sozinho) e o quadro 5 para de se mexer.
#[test]
fn the_sculpt_reaches_every_selected_key() {
    use ph2d_flip::{Hold, KeyKind};
    let (mut doc, l) = doc_with_line();
    let oid = doc.objects().first().unwrap().id;
    // Uma 2ª chave, no quadro 5, com a MESMA linha (uma cópia — desenho próprio).
    doc.object_mut(oid)
        .unwrap()
        .duplicate_frame(l, 0, 5, ph2d_flip::DupMode::Deep);
    let _ = (Hold::Implicit, KeyKind::Keyframe);

    let strip0 = crate::flip_strip::FlipStrip {
        selection: vec![0, 5], // as DUAS marcadas
        ..Default::default()
    };
    let mut strip = strip0;

    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 5.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let ph = Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS)); // quadro 0 = o ativo

    let (_, targets) = reshape_begin(&mut doc, &ph, Some(l), &mut strip, &p, &s, false)
        .expect("ha desenho na camada ativa");

    assert_eq!(targets.len(), 2, "o gesto nao alcancou as duas chaves");
    assert!(
        strokes_at(&doc, l, 0)[0].positions()[2].y > 0.5,
        "o quadro ATIVO nao foi esculpido"
    );
    assert!(
        strokes_at(&doc, l, 5)[0].positions()[2].y > 0.5,
        "o quadro 5 (marcado, mas nao o ativo) NAO foi esculpido — o multiframe nao chegou nele"
    );
}

/// 🔴 **Um desenho INSTANCIADO por duas chaves é esculpido UMA vez.**
///
/// É a regra que a referência marca com exclamação (`02_referencia §11`: *dedup por
/// Drawing!*). Duas chaves podem apontar o MESMO desenho (o "duplicate as instance", como
/// um ciclo reusa arte). Sem o dedup, o pincel aplicaria **duas vezes no mesmo buffer**: a
/// linha andaria o dobro naquele quadro, e o animador veria a arte se deformar sozinha só
/// nos quadros instanciados — um bug que ninguem atribuiria ao multiframe.
///
/// Mutação que sangra: tire o dedup de `flip_multiframe::targets`.
#[test]
fn an_instanced_drawing_is_sculpted_only_once() {
    let (mut doc, l) = doc_with_line();
    let oid = doc.objects().first().unwrap().id;
    // A chave 5 é uma INSTÂNCIA da 0: o MESMO `DrawingId`.
    doc.object_mut(oid)
        .unwrap()
        .duplicate_frame(l, 0, 5, ph2d_flip::DupMode::Instance);

    let mut strip = crate::flip_strip::FlipStrip {
        selection: vec![0, 5],
        ..Default::default()
    };

    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 5.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let ph = Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS));

    let (_, targets) = reshape_begin(&mut doc, &ph, Some(l), &mut strip, &p, &s, false).unwrap();

    assert_eq!(
        targets.len(),
        1,
        "o desenho instanciado virou DOIS alvos — o pincel o esculpiria em dobro"
    );
    // **E o deslocamento é o de UMA aplicação.** O ponto está SOB o cursor, então a queda
    // do pincel vale 1 e o Push o desloca exatamente pelo delta (1,0 unidade). Se o
    // desenho instanciado fosse esculpido duas vezes, ele andaria **2,0** — e é essa a
    // assinatura do bug, não uma faixa qualquer.
    let y = strokes_at(&doc, l, 0)[0].positions()[2].y;
    assert!(
        (y - 1.0).abs() < 1e-3,
        "o ponto andou {y}: o esperado e 1,0 (uma aplicacao). 2,0 = o pincel bateu DUAS          vezes no mesmo buffer (o dedup caiu)"
    );
}

/// 🔴 **O multiframe é ancorado na ARTE, não no mundo** (W7.2).
///
/// Quadros-alvo podem estar em POSES diferentes — é justamente o que uma pose serve para
/// fazer (um ciclo que ANDA). O cursor aponta para um lugar do mundo, mas a arte de um
/// quadro deslocado está noutro lugar da tela. Se o pincel perseguisse o ponto de MUNDO,
/// ele cairia no vazio para esse quadro, e o multiframe silenciosamente só editaria o
/// ativo sempre que as poses diferissem.
///
/// A semântica certa é a que o animador quer dizer: *"conserta o cotovelo em TODOS os
/// quadros que marquei"* — as mesmas coordenadas na geometria de cada desenho são a mesma
/// parte do personagem.
///
/// (Eu escrevi a compensação de pose primeiro. Este gate a derrubou: no quadro deslocado,
/// nenhum ponto se mexia. **Mutação que sangra:** compensar o `pose_delta` na
/// `InputSample` de cada alvo.)
#[test]
fn multiframe_is_art_anchored_not_world_anchored() {
    let (mut doc, l) = doc_with_line();
    let oid = doc.objects().first().unwrap().id;
    // Chave 5: cópia PROFUNDA (arte própria), deslocada 100 no x — o quadro está longe.
    doc.object_mut(oid)
        .unwrap()
        .duplicate_frame(l, 0, 5, ph2d_flip::DupMode::Deep);
    doc.object_mut(oid)
        .unwrap()
        .translate_frame(l, 5, Vec2::new(100.0, 0.0));

    let mut strip = crate::flip_strip::FlipStrip {
        selection: vec![0, 5],
        ..Default::default()
    };
    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 1.0, // apertado: só o ponto SOB o cursor se mexe
        strength: 1.0,
        ..Default::default()
    };
    // O cursor está sobre o ponto x=2 da arte do quadro ATIVO.
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let ph = Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS));

    let (oid, _targets) = reshape_begin(&mut doc, &ph, Some(l), &mut strip, &p, &s, false).unwrap();

    // Nos DOIS quadros, quem andou é o MESMO ponto da arte (o índice 2) — inclusive no
    // quadro que está a 100 unidades de distância na tela.
    let obj = doc.object(oid).unwrap();
    for (key, label) in [(0, "ativo"), (5, "deslocado")] {
        let did = obj.layer(l).unwrap().drawing_at(key).unwrap();
        let pts = obj.drawing(did).unwrap().strokes[0].positions();
        let moved: Vec<usize> = pts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.y.abs() > 1e-3)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(
            moved,
            vec![2],
            "no quadro {label}: o pincel nao pegou a mesma parte da ARTE"
        );
    }
}
