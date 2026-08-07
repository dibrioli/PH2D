//! **O contorno é o que se vê E o que se clica** — os gates de [`super::stroke_outline`].
//!
//! ⚠️ **Todos dirigem `on_canvas_pointer`**, a porta do artista: o defeito reportado é do ROTEADOR de
//! Down (`stroke_multi::maybe_switch_or_new_shape`), e um gate que chamasse o hit-test direto ficaria
//! verde sobre um roteador que nunca o consulta.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// O raio da elipse VIVA — quem responde *"qual figura está em mãos agora?"* nos gates de roteamento.
fn live_circle_radius(t: &crate::tool::PainterTool) -> Option<f32> {
    match t.capture_shape().as_deref() {
        Some(crate::undo::ShapeEditState::Ellipse(e)) => Some(e.rx),
        _ => None,
    }
}

/// O raio do polígono VIVO — o irmão de [`live_circle_radius`] para a família de N lados.
fn live_polygon_radius(t: &crate::tool::PainterTool) -> Option<f32> {
    match t.capture_shape().as_deref() {
        Some(crate::undo::ShapeEditState::Polygon(p)) => Some(p.rx),
        _ => None,
    }
}

/// Desenha um círculo COMPLETO (Down → Move → Up) centrado em `c` com raio `r`.
fn draw_circle(t: &mut crate::tool::PainterTool, c: [f32; 2], r: f32) {
    t.on_canvas_pointer(cp(c, PointerPhase::Down));
    t.on_canvas_pointer(cp([c[0] + r * 0.5, c[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([c[0] + r, c[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([c[0] + r, c[1]], PointerPhase::Up));
}

/// **A cena do report:** três círculos grandes, cada um com um INTERIOR largo — que é onde o defeito
/// vive (a caixa de um círculo de raio `r` cobre `4/π` da área dele, e o interior inteiro contava como
/// acerto).
///
/// ⚠️ **A geometria é parte do gate, e a 1ª versão dela era um CONFUNDIDOR:** os círculos estavam tão
/// próximos que o centro do primeiro caía a **1,66 px do contorno do segundo** — o gate do quadrado
/// central passava pelo ramo do CONTORNO, e a mutação *"tire o quadrado central"* sobrevivia. Aqui todo
/// centro e todo ponto de sonda fica a **≥ 45 px de QUALQUER contorno**, então cada ramo é medido
/// sozinho.
fn three_big_circles() -> crate::tool::PainterTool {
    let side = 512u32;
    let mut t = tool(side, PaintMedia::Digital, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    draw_circle(&mut t, [140.0, 140.0], 90.0);
    draw_circle(&mut t, [380.0, 140.0], 90.0);
    draw_circle(&mut t, [140.0, 380.0], 90.0);
    t
}

/// **O report, ao pé da letra** (Enio, 2026-08-07): *"se clicar dentro de uma forma já desenhada, não
/// aceita desenhar outra"*.
///
/// O clique cai FUNDO dentro do primeiro círculo — longe do contorno e longe do quadrado central —, e
/// tem de começar uma figura NOVA, não reativar aquela. Nasce VERMELHO com o `bbox_contains` de volta:
/// o interior da caixa cobre esse ponto e o roteador reativa a parqueada.
#[test]
fn clicking_deep_inside_a_parked_shape_starts_a_new_one() {
    let mut t = three_big_circles();
    let parked_before = t.paint.parked_shapes.len();

    // 45 px à direita do centro do 1º círculo: dentro dele, fora do quadrado central, e a 45 px do
    // contorno mais próximo — nada DESENHADO está ali.
    let inside = [185.0, 140.0];
    t.on_canvas_pointer(cp(inside, PointerPhase::Down));
    t.on_canvas_pointer(cp([inside[0] + 25.0, inside[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([inside[0] + 25.0, inside[1]], PointerPhase::Up));

    assert_eq!(
        t.paint.parked_shapes.len(),
        parked_before + 1,
        "clicar DENTRO de uma figura tem de parquear a ativa e comecar outra, nao reativar a de baixo"
    );
    // E a figura viva é a NOVA: raio ~25, não os ~120 de nenhuma das três.
    let r = live_circle_radius(&t).expect("uma elipse viva");
    assert!(
        (20.0..40.0).contains(&r),
        "a figura viva devia ser a recem-criada (raio ~25), veio r={r:.1}"
    );
}

/// O CONTROLE da lei: encostar no CONTORNO de uma figura parqueada continua reativando-a.
///
/// Sem este par, *"nunca reative nada"* passaria no gate acima e destruiria o gesto de re-edição.
#[test]
fn clicking_a_parked_shapes_outline_reactivates_it() {
    let mut t = three_big_circles();
    let parked_before = t.paint.parked_shapes.len();

    // O ponto mais à direita do contorno do 1º círculo (centro 140,140 · raio 90).
    let on_edge = [230.0, 140.0];
    t.on_canvas_pointer(cp(on_edge, PointerPhase::Down));

    assert_eq!(
        t.paint.parked_shapes.len(),
        parked_before,
        "reativar TROCA a ativa pela parqueada: a contagem de parqueadas nao muda"
    );
    let r = live_circle_radius(&t).expect("uma elipse viva");
    assert!(
        r > 70.0,
        "a figura viva devia ser a parqueada de raio 90, veio r={r:.1}"
    );
}

/// O quadrado central com o glifo de Operation É desenhado ⇒ tem de responder.
///
/// ⚠️ Ele fica bem no meio da figura, ou seja **dentro** da região que o gate de cima exige que NÃO
/// reative — as duas leis convivem porque a segunda região é o que o badge pinta, e ela é pequena.
#[test]
fn clicking_a_parked_shapes_centre_glyph_reactivates_it() {
    let mut t = three_big_circles();
    let parked_before = t.paint.parked_shapes.len();

    t.on_canvas_pointer(cp([140.0, 140.0], PointerPhase::Down)); // o centro exato do 1º círculo

    assert_eq!(t.paint.parked_shapes.len(), parked_before);
    let r = live_circle_radius(&t).expect("uma elipse viva");
    assert!(
        r > 70.0,
        "o quadrado central da parqueada devia reativa-la, veio r={r:.1}"
    );
}

/// O badge carrega o **CONTORNO**, não uma caixa — o que o artista vê é a figura.
#[test]
fn the_badge_carries_the_shapes_outline_not_a_box() {
    let t = three_big_circles();
    let badges = t.stroke_op_badges();
    assert_eq!(badges.len(), 2, "duas figuras parqueadas");

    for b in &badges {
        assert!(b.closed, "um circulo fecha");
        assert!(
            b.outline.len() > 32,
            "uma caixa tem 4 pontos; um contorno de circulo tem dezenas ({} veio)",
            b.outline.len()
        );
        // Todo ponto do contorno está no MESMO raio do centro: é a figura, não a caixa dela.
        let rs: Vec<f32> = b
            .outline
            .iter()
            .map(|p| {
                let (dx, dy) = (p[0] - b.center[0], p[1] - b.center[1]);
                (dx * dx + dy * dy).sqrt()
            })
            .collect();
        let (lo, hi) = rs
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        assert!(
            hi - lo < 2.0,
            "o contorno de um circulo tem raio constante; veio [{lo:.1}, {hi:.1}]"
        );
    }
}

/// **TODA aresta do contorno é alcançável — inclusive a de FECHO.**
///
/// ⚠️ O segmento que liga o último ponto ao primeiro não sai do `windows(2)`: ele tem de ser fechado à
/// mão (`ShapeOutline::hit`). Num círculo de mil pontos esquecê-lo é um vão invisível; num polígono de
/// cinco lados é **uma aresta inteira** morta sob o mouse — daí a fixture ser um polígono, e o gate
/// varrer TODAS as arestas em vez de escolher uma.
#[test]
fn every_edge_of_a_parked_outline_is_reachable() {
    let scene = || {
        let mut t = tool(512, PaintMedia::Digital, 8.0);
        t.paint.brush.stroke_method = StrokeMethod::Polygon;
        draw_circle(&mut t, [160.0, 260.0], 90.0); // o polígono parqueado
        draw_circle(&mut t, [380.0, 260.0], 60.0); // …e o que fica ativo
        t
    };

    let t = scene();
    let badges = t.stroke_op_badges();
    let b = badges.first().expect("um poligono parqueado");
    assert!(b.closed && b.outline.len() >= 3);

    let n = b.outline.len();
    let mids: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let (a, c) = (b.outline[i], b.outline[(i + 1) % n]);
            [(a[0] + c[0]) * 0.5, (a[1] + c[1]) * 0.5]
        })
        .collect();
    drop(t);

    for (i, m) in mids.iter().enumerate() {
        let mut probe = scene();
        probe.on_canvas_pointer(cp(*m, PointerPhase::Down));
        let r = live_polygon_radius(&probe);
        assert!(
            r.is_some_and(|r| r > 80.0),
            "o meio da aresta {i} de {n} nao reativou a figura parqueada (raio vivo {r:?})"
        );
    }
}

/// Um contorno ABERTO diz que é aberto — é esse bit que decide `stroke_box` vs `stroke_open` na shell,
/// e fechá-lo por engano desenha um segmento que a figura não tem.
#[test]
fn an_open_shape_reports_an_open_outline() {
    let side = 512u32;
    let mut t = tool(side, PaintMedia::Digital, 8.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    // Uma curva aberta, fechada com o Up…
    t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([200.0, 160.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([200.0, 160.0], PointerPhase::Up));
    // …e uma segunda figura, para parquear a primeira.
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    draw_circle(&mut t, [400.0, 400.0], 50.0);

    let b = t
        .stroke_op_badges()
        .into_iter()
        .next()
        .expect("a curva parqueada");
    assert!(!b.closed, "uma curva autorada nasce ABERTA");
    assert!(b.outline.len() >= 2);
}

/// **A figura EM CRIAÇÃO já tem gizmo** (Enio: *"o gizmo está invisível ao ser criado"*).
///
/// Com o gesto rascunhando a tinta (`super::shape_draft`), o overlay é a única coisa na tela durante o
/// arrasto — e ele era `None` até o Up. As ALÇAS ficam de fora: nenhum Down as alcança no meio do
/// arrasto, e alça desenhada que não responde é chrome morto.
#[test]
fn a_shape_being_created_already_has_a_gizmo() {
    let side = 256u32;
    let mut t = tool(side, PaintMedia::Digital, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([168.0, 128.0], PointerPhase::Move));

    let ov = t
        .ellipse_overlay()
        .expect("o contorno existe desde o 1o pixel do arrasto");
    assert!(!ov.editing, "ainda no arrasto de criacao");
    assert!(
        ov.perimeter.len() > 8,
        "o contorno da elipse em criacao ({} pontos)",
        ov.perimeter.len()
    );
    // …e ele descreve o raio que a mão puxou (~40), não um resto de estado.
    let r = ov
        .perimeter
        .iter()
        .map(|p| {
            let (dx, dy) = (p[0] - 128.0, p[1] - 128.0);
            (dx * dx + dy * dy).sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!((35.0..45.0).contains(&r), "raio do contorno veio {r:.1}");
}

/// O mesmo para o POLÍGONO e para a CURVA — as outras duas famílias que abrem por arrasto.
#[test]
fn every_shape_family_shows_its_line_while_it_is_drawn() {
    for method in [StrokeMethod::Polygon, StrokeMethod::Arc] {
        let mut t = tool(256, PaintMedia::Digital, 10.0);
        t.paint.brush.stroke_method = method;
        t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([168.0, 148.0], PointerPhase::Move));

        let n = match method {
            StrokeMethod::Polygon => {
                let ov = t
                    .polygon_overlay()
                    .expect("poligono em criacao tem contorno");
                assert!(!ov.editing);
                ov.perimeter.len()
            }
            _ => {
                let ov = t.curve_overlay().expect("curva em criacao tem spine");
                assert!(
                    ov.transform_gizmo.is_none() && ov.points.is_empty(),
                    "na fase de desenho a curva publica SO o spine"
                );
                ov.spine.len()
            }
        };
        assert!(n >= 2, "{method:?} publicou {n} pontos de linha");
    }
}

// ── DELETE: apagar a figura em mãos ──────────────────────────────────────────────────────────────

/// **O pedido** (Enio, 2026-08-07): *"permita usar del para deletar a forma selecionada por último"*.
///
/// A figura em mãos sai; as parqueadas ficam de pé, na tela.
#[test]
fn delete_drops_the_shape_in_hand_and_leaves_the_others_standing() {
    let mut t = three_big_circles();
    let parked_before = t.paint.parked_shapes.len();
    let painted_before = painted_texels(&t);
    assert!(t.capture_shape().is_some(), "ha uma figura em maos");

    assert!(t.delete_active_shape(), "a tecla foi consumida");

    assert!(
        t.capture_shape().is_none(),
        "nada fica selecionado depois — a proxima e a que o artista clicar"
    );
    assert_eq!(
        t.paint.parked_shapes.len(),
        parked_before,
        "as parqueadas nao foram tocadas"
    );
    assert_eq!(
        t.stroke_op_badges().len(),
        parked_before,
        "…e continuam com contorno"
    );
    // ⚠️ **A lista não é a TELA.** A 1ª versão parava nos badges, e a mutação *"não re-carimbe o
    // conjunto que sobrou"* passava por ela: os parqueados continuavam na lista com a tinta da figura
    // apagada ainda na tela. O oráculo do que o artista vê são os texels.
    let after = painted_texels(&t);
    assert!(
        after < painted_before,
        "a tinta da figura apagada continua na tela ({after} de {painted_before} texels)"
    );
    assert!(
        after > 0,
        "as parqueadas sumiram junto — o re-carimbo do conjunto que sobrou nao rodou"
    );
}

/// Sem figura em mãos a tecla **não é consumida** — ela segue para quem mais a quiser.
///
/// ⚠️ É o controle que impede a rota de virar um Delete que engole tudo: sem ele, `delete_active_shape`
/// cravado em `true` passaria, e o artista perderia o Delete de todo o resto do app.
#[test]
fn delete_with_no_shape_in_hand_is_not_consumed() {
    let mut t = tool(256, PaintMedia::Digital, 10.0);
    assert!(
        !t.delete_active_shape(),
        "sem figura viva a tecla tem de cair para o proximo dono"
    );
}

/// Apagar é **UM passo de undo**, e o undo traz a figura de volta.
#[test]
fn deleting_a_shape_is_one_undo_step_that_brings_it_back() {
    let mut t = three_big_circles();
    let before = t.capture_shape().expect("a figura em maos");
    let painted_before = painted_texels(&t);

    assert!(t.delete_active_shape());
    assert!(
        painted_texels(&t) < painted_before,
        "a tinta dela saiu da tela"
    );

    assert!(t.undo_last(), "um Ctrl+Z");
    let back = t.capture_shape().expect("a figura voltou para as maos");
    assert_eq!(
        format!("{back:?}"),
        format!("{before:?}"),
        "…e voltou IGUAL"
    );
}

/// Quantos texels deixaram de ser papel branco — o oráculo do que o artista vê.
fn painted_texels(t: &crate::tool::PainterTool) -> usize {
    t.canvas_rgba
        .iter()
        .step_by(4)
        .zip(t.canvas_rgba.iter().skip(1).step_by(4))
        .filter(|(r, g)| **r != 255 || **g != 255)
        .count()
}

// ── A LINHA COZIDA: o Fillet/Chamfer aparece na PRÓPRIA linha ────────────────────────────────────
//
// Enio, 2026-08-07: *"Line tem em seu gizmo a possibilidade de criar Chamfer e Fillet. Contudo, agora
// que não temos mais o preview da tinta, o preview do chamfer e do fillet precisa acontecer na própria
// linha."* Com o gesto rascunhado (`shape_draft`) o contorno é a ÚNICA coisa na tela, e ele desenhava a
// polilinha AFIADA — o arredondamento ficava invisível exatamente enquanto a alça que o cria é
// arrastada.

/// Uma polilinha de 3 pontos (UMA quina interior) com o Fillet daquela quina ARRASTADO pela alça — o
/// caminho do artista, nunca `corner_mods` escrito à mão.
fn line_with_a_filleted_corner() -> crate::tool::PainterTool {
    let mut t = tool(512, PaintMedia::Digital, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_shape_grab_tol_px(8.0);
    for p in [[120.0, 120.0], [360.0, 120.0], [360.0, 360.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    assert!(t.line_finish_points(), "fim da criação → fase de edição");
    let fh = t.line_overlay().expect("sessão viva").corner_gizmos[0].fillet_handle;
    let target = [fh[0] + 60.0, fh[1]]; // arrastar para a DIREITA cresce o raio
    t.on_canvas_pointer(cp(fh, PointerPhase::Down));
    t.on_canvas_pointer(cp(target, PointerPhase::Move));
    t.on_canvas_pointer(cp(target, PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().corner_gizmos[0].active,
        1,
        "a quina está filetada"
    );
    t
}

/// A menor distância de `p` à polilinha `spine` (px).
fn dist_to(spine: &[[f32; 2]], p: [f32; 2]) -> f32 {
    super::stroke_router::min_dist2_to_polyline(p, spine)
        .expect("a linha tem pontos")
        .sqrt()
}

/// **O gate da wave:** com a quina filetada, o contorno que a shell desenha PASSA LONGE da quina
/// afiada — ele é o arco, não o bico.
///
/// ⚠️ O oráculo é a **distância da quina AUTORADA ao contorno**, não `outline != points`: uma
/// desigualdade de vetores fica verde para qualquer diferença, inclusive uma que o olho não vê.
///
/// **Mutação que deve sangrar:** `outline: ed.points.clone()` no `line_overlay` (o desenho de volta ao
/// bico) ⇒ a distância volta a 0.
#[test]
fn the_line_gizmo_draws_the_cooked_corner_not_the_sharp_one() {
    let t = line_with_a_filleted_corner();
    let ov = t.line_overlay().expect("sessão viva");
    let sharp = ov.points[1];
    let d = dist_to(&ov.outline, sharp);
    assert!(
        d > 5.0,
        "a quina afiada tem de ficar FORA do contorno cozido (dist {d:.2} px) — o gizmo está \
         desenhando o bico, e o fillet fica invisível enquanto se arrasta a alça"
    );
    // …e a alça continua NA quina afiada: é onde se pega, e ela não pode migrar para o arco.
    assert_eq!(
        ov.points[1],
        [360.0, 120.0],
        "a fonte autorada não se move — o cozido é derivado dela (ADR-0121)"
    );
}

/// **A lei do não-SALTO:** o contorno da figura VIVA e o da mesma figura PARQUEADA são a MESMA linha.
///
/// O `stroke_outline` já cozinhava a quina para a parqueada enquanto o overlay vivo desenhava o bico —
/// então parquear uma linha filetada e reativá-la fazia a forma **saltar** na tela. As duas metades
/// perguntam à mesma porta (`line_corner::cooked_path`) e por isso não podem mais divergir.
///
/// **Mutação que deve sangrar:** trocar o `cooked_path` de um dos dois lados por `expand` (que fecha o
/// laço) ou pelos pontos crus.
#[test]
fn the_live_line_and_the_parked_line_are_the_same_outline() {
    let t = line_with_a_filleted_corner();
    let live = t.line_overlay().expect("sessão viva").outline;
    let st = t.capture_shape().expect("a figura está em mãos");
    let parked = t
        .shape_state_outline(&st)
        .expect("a mesma figura, vista como parqueada");
    assert_eq!(
        live, parked.points,
        "o contorno vivo e o parqueado descrevem linhas diferentes — reativar a figura a faz SALTAR"
    );
}

/// **O controle:** sem quina modificada o cozido é os pontos VERBATIM — uma polilinha comum não se
/// move um pixel com esta wave.
#[test]
fn a_plain_polyline_cooks_to_itself() {
    let mut t = tool(512, PaintMedia::Digital, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_shape_grab_tol_px(8.0);
    for p in [[120.0, 120.0], [360.0, 120.0], [360.0, 360.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    assert!(t.line_finish_points());
    let ov = t.line_overlay().expect("sessão viva");
    assert_eq!(
        ov.outline, ov.points,
        "sem Fillet/Chamfer o contorno É a polilinha autorada"
    );
}
