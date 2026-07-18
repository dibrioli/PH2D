//! Testes do Edit Mode (`flip_select`), num módulo-irmão pelo cap de LOC do HR-18.
//! Declarado pelo pai via `#[path]`, então `super` é `flip_select`.

use super::*;
use ph2d_flip::{Fill, FlipStroke, Point, Rgba};
use ph2d_vec_scene::Xform;

/// Um traço de line-art, com os pontos dados (largura em px de TELA, como o produto).
fn line(pts: &[(f32, f32)], width: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s
}

/// Uma REGIÃO (o que o balde produz): só cor, sem linha (`hide_stroke`).
fn region(pts: &[(f32, f32)]) -> FlipStroke {
    let mut s = line(pts, 0.0);
    s.closed = true;
    s.hide_stroke = true;
    s.fill = Some(Fill {
        color: Rgba::new(0.8, 0.2, 0.2, 1.0),
        opacity: 1.0,
    });
    s
}

fn drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes = strokes;
    d
}

/// 🔴 **O clique pega a COSTURA de um traço fechado** — o repro do smoke do Enio (§4.A):
/// *"uma linha do triângulo e uma linha do quadrado não são sensíveis à seleção"*.
///
/// A aresta de fechamento (último→primeiro vértice) é desenhada pelo render
/// (`pack::stroke_segments`) e realçada pelo halo, mas o pick iterava
/// `positions().windows(2)` e a perdia: um triângulo tinha **3 linhas na tela e 2
/// clicáveis**, um quadrado 4 e 3. O fill mascarava isso (o `ring_contains` pega o
/// interior), então só uma forma fechada **sem preenchimento** expunha o buraco — que é
/// exatamente o que a cena do §4.A tem.
///
/// Mutação que sangra: `hits` voltar a iterar `positions().windows(2)`.
#[test]
fn a_click_on_the_seam_of_a_closed_stroke_picks_it() {
    // Triângulo SEM fill: a costura é a aresta (0,100)→(0,0). O ponto de mira (0,50) está
    // no meio dela — e a 50 da base e a ~35 da hipotenusa, MUITO além do `MIN_PICK_PX`
    // (5). O isolamento é o ponto do fixture: num triângulo pequeno o alcance do pick
    // alcança a hipotenusa e o teste passa pelo motivo errado (foi o que aconteceu na 1ª
    // tentativa — `feedback_test_with_product_numbers_not_convenient_ones`).
    let seam_mid = Vec2::new(0.0, 50.0);
    let verts = [(0.0, 0.0), (100.0, 0.0), (0.0, 100.0)];
    let mut tri = line(&verts, 4.0);
    tri.closed = true;
    let d = drawing(vec![tri]);
    assert_eq!(
        stroke_at(&d, seam_mid, 1.0, &Xform::IDENTITY),
        Some(0),
        "o clique no meio da COSTURA nao pegou o triangulo (as 3 linhas sao clicaveis)"
    );
    // 🔴 O par de AUSÊNCIA: o MESMO traço ABERTO não tem costura — o clique ali erra.
    // (Sem este, "feche sempre" passaria e a senoide do W8 ganharia uma aresta fantasma
    // ligando as pontas: clicar no vazio entre elas selecionaria o traço.)
    let mut open = line(&verts, 4.0);
    open.closed = false;
    let d_open = drawing(vec![open]);
    assert_eq!(
        stroke_at(&d_open, seam_mid, 1.0, &Xform::IDENTITY),
        None,
        "traco ABERTO nao tem costura: clicar na aresta que ninguem desenhou nao pega nada"
    );
}

/// **O pick pega o traço de CIMA.** Dois traços sobrepostos: o clique tem de escolher o
/// que o usuário VÊ (o último da lista — a ordem da lista é a ordem de z).
///
/// Mutação que sangra: tire o `.rev()` do `stroke_at` e ele passa a pegar o de baixo.
#[test]
fn the_pick_takes_the_topmost_stroke() {
    let d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
    ]);
    assert_eq!(
        stroke_at(&d, Vec2::new(5.0, 0.0), 1.0, &Xform::IDENTITY),
        Some(1),
        "o pick pegou o traco de BAIXO — o usuario clica no que ve"
    );
}

/// **Uma REGIÃO pega pelo INTERIOR.** Ela não tem linha (`hide_stroke`): exigir
/// proximidade da borda tornaria a cor do balde inselecionável, e clicar no meio dela é o
/// gesto óbvio.
///
/// E o BURACO não pega: clicar no furo de um "O" é clicar no que está ATRÁS dele.
#[test]
fn a_region_is_picked_by_its_interior_but_never_by_its_hole() {
    let mut r = region(&[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)]);
    r.holes = vec![vec![
        Vec2::new(8.0, 8.0),
        Vec2::new(12.0, 8.0),
        Vec2::new(12.0, 12.0),
        Vec2::new(8.0, 12.0),
    ]];
    let d = drawing(vec![r]);

    assert_eq!(
        stroke_at(&d, Vec2::new(3.0, 3.0), 1.0, &Xform::IDENTITY),
        Some(0),
        "o miolo da regiao tem de pegar (ela nao tem linha para se aproximar)"
    );
    assert_eq!(
        stroke_at(&d, Vec2::new(10.0, 10.0), 1.0, &Xform::IDENTITY),
        None,
        "o BURACO pegou — clicar no furo de um O e clicar no que esta atras"
    );
}

/// **O raio de pick acompanha o ZOOM.** A espessura do traço é absoluta em px de TELA e a
/// geometria é documento: o mesmo clique, a 3 unidades de uma linha fina, tem de PEGAR
/// com a câmera afastada (onde 3 unidades são poucos px) e ERRAR com ela aproximada.
///
/// É a mesma armadilha que matou o balde três vezes (BUGS #11/#14/#16): um teste com
/// `px_to_world = 1.0` — o único valor em que px de tela vale unidade de documento — não
/// veria erro de unidade nenhum.
///
/// Mutação que sangra: tire o `px_to_world` do `stroke_at` e as duas asserções trocam.
#[test]
fn the_pick_radius_follows_the_zoom() {
    let d = drawing(vec![line(&[(0.0, 0.0), (100.0, 0.0)], 1.0)]);
    let p = Vec2::new(50.0, 3.0); // 3 unidades de documento acima da linha

    // Câmera AFASTADA: 1 px de tela = 4 unidades ⇒ o raio de pick (5 px de tela) cobre
    // 20 unidades. Pega.
    assert_eq!(
        stroke_at(&d, p, 4.0, &Xform::IDENTITY),
        Some(0),
        "afastado, 3 unidades sao menos de 1 px de tela: tinha de pegar"
    );
    // Câmera APROXIMADA: 1 px de tela = 0,1 unidade ⇒ o raio de pick cobre 0,5 unidade.
    // O ponto está a 3 unidades = 30 px de tela. Erra.
    assert_eq!(
        stroke_at(&d, p, 0.1, &Xform::IDENTITY),
        None,
        "aproximado, 3 unidades sao 30 px de tela: nao podia pegar"
    );
}

/// **Clique simples SUBSTITUI; Shift+clique ALTERNA; clique no vazio DESMARCA.**
#[test]
fn click_replaces_shift_toggles_and_empty_clears() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 10.0), (10.0, 10.0)], 4.0),
    ]);

    assert!(apply_pick(&mut d, Some(0), Pick::Replace));
    assert_eq!(d.selected_indices(), vec![0]);

    // Shift: o 1º CONTINUA selecionado.
    assert!(apply_pick(&mut d, Some(1), Pick::Toggle));
    assert_eq!(d.selected_indices(), vec![0, 1]);

    // Shift de novo no mesmo: sai.
    assert!(apply_pick(&mut d, Some(1), Pick::Toggle));
    assert_eq!(d.selected_indices(), vec![0]);

    // Clique simples noutro: o 1º SAI (substitui, não acumula).
    assert!(apply_pick(&mut d, Some(1), Pick::Replace));
    assert_eq!(d.selected_indices(), vec![1]);

    // Vazio sem Shift: limpa. Com Shift: não faz nada (um shift-clique que errou o traço
    // por 2 px não pode apagar a seleção que o usuário montou).
    assert!(apply_pick(&mut d, None, Pick::Replace));
    assert!(d.selected_indices().is_empty());
    assert!(apply_pick(&mut d, Some(0), Pick::Replace));
    assert!(!apply_pick(&mut d, None, Pick::Toggle));
    assert_eq!(d.selected_indices(), vec![0], "o Shift no vazio DESMARCOU");
}

/// **A seleção SOBREVIVE ao balde** — e é isto que uma lista de índices no shell não
/// conseguiria fazer.
///
/// O balde insere o preenchimento **no meio da lista** (`flip_fill`: a cor entra POR BAIXO
/// do line-art). Uma seleção guardada como índice `1` apontaria, depois da inserção, para
/// um traço DIFERENTE — e o próximo ajuste do painel recoloriria a arte errada, em
/// silêncio. Como o `selected` é um atributo que VIAJA com o traço, ele não tem como
/// dessincronizar.
///
/// Mutação que sangra: guarde a seleção como `Vec<usize>` no shell e este teste morre.
#[test]
fn the_selection_survives_the_bucket_inserting_a_stroke_beneath_it() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 10.0), (10.0, 10.0)], 4.0),
    ]);
    apply_pick(&mut d, Some(1), Pick::Replace);
    let before = d.strokes[1].clone();

    // O balde entra ATRÁS do line-art (índice 0) — exatamente como o `fill_click` faz.
    d.strokes
        .insert(0, region(&[(0.0, 0.0), (5.0, 0.0), (5.0, 5.0)]));

    assert_eq!(
        d.selected_indices(),
        vec![2],
        "a insercao deslocou o traco, e a selecao tem de ter ido JUNTO com ele"
    );
    assert_eq!(
        d.strokes[2], before,
        "o traco selecionado nao e mais o mesmo — a selecao apontou para outra arte"
    );
}

/// **Re-clicar a seleção que já existe não muda nada** — senão cada clique repetido
/// viraria um passo de undo vazio (o registro do undo sai do DIFF pós-frame).
#[test]
fn reclicking_the_same_lone_selection_changes_nothing() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0)], 4.0)]);
    assert!(apply_pick(&mut d, Some(0), Pick::Replace));
    assert!(
        !apply_pick(&mut d, Some(0), Pick::Replace),
        "re-clicar a mesma selecao registrou uma mudanca (= um passo de undo vazio)"
    );
}

/// **Uma camada TRAVADA não entrega os traços dela** — nem para seleção (regra do GP).
#[test]
fn a_locked_layer_yields_no_drawing_to_select_in() {
    use ph2d_core::Playhead;
    let mut doc = FlipDoc::default();
    let oid = doc.push_object("Flip");
    let obj = doc.object_mut(oid).unwrap();
    let lid = obj.add_layer("Layer 1");
    obj.insert_frame(
        lid,
        0,
        ph2d_flip::Hold::Implicit,
        ph2d_flip::KeyKind::Keyframe,
    );
    let playhead = Playhead::default();

    assert!(
        visible_drawing(&doc, &playhead, Some(lid)).is_some(),
        "a camada destravada tem de entregar o desenho"
    );
    doc.object_mut(oid).unwrap().layer_mut(lid).unwrap().locked = true;
    assert!(
        visible_drawing(&doc, &playhead, Some(lid)).is_none(),
        "a camada TRAVADA entregou o desenho — e o clique editaria arte protegida"
    );
}

// ── Os ajustes do painel sobre a SELEÇÃO (`apply_style_delta`) ──

fn style() -> ph2d_tool_flip::FlipStyleSnapshot {
    ph2d_tool_flip::FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Edit,
        stroke: [10, 20, 200, 255], // azul
        width_px: 8.0,
        opacity: 1.0,
        ..Default::default()
    }
}

/// 🔴 **A regra que protege a arte: SÓ A MUDANÇA age.**
///
/// Um traço vermelho selecionado, com o painel em azul, **não pode virar azul** — e é
/// exatamente o que aconteceria se o passe reaplicasse o estilo a cada frame. O usuário
/// perderia a arte *só por clicar nela*.
///
/// Mutação que sangra: tire o `if prev == now { return false }` (ou compare contra os
/// defaults em vez de contra o frame anterior) e o vermelho vira azul.
#[test]
fn selecting_a_stroke_does_not_repaint_it_with_the_panel_colour() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0)], 4.0)]);
    d.strokes[0].selected = true;
    let red = Rgba::new(0.9, 0.1, 0.1, 1.0);
    for c in d.strokes[0].colors_mut() {
        *c = red;
    }

    // O painel está em AZUL, mas nada mudou entre um frame e o outro.
    let s = style();
    assert!(
        !apply_style_delta(&mut d, &s, &s),
        "um frame sem mudanca de estilo escreveu no traco"
    );
    assert!(
        d.strokes[0].colors().iter().all(|c| *c == red),
        "o traco VERMELHO virou a cor do painel so por estar selecionado"
    );
}

/// **Mexer no Size não repinta; mexer na cor não reespessa.** Cada campo age sozinho.
#[test]
fn each_field_acts_alone() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0)], 4.0)]);
    d.strokes[0].selected = true;
    let red = Rgba::new(0.9, 0.1, 0.1, 1.0);
    for c in d.strokes[0].colors_mut() {
        *c = red;
    }

    let prev = style();
    let bigger = ph2d_tool_flip::FlipStyleSnapshot {
        width_px: 16.0,
        ..prev
    };
    assert!(apply_style_delta(&mut d, &prev, &bigger));
    assert!(
        d.strokes[0].colors().iter().all(|c| *c == red),
        "mexer no SIZE repintou o traco"
    );
    assert!(
        (d.strokes[0].widths()[0] - ph2d_tool_flip::size_to_world(16.0)).abs() < 1e-3,
        "o Size nao chegou no traco"
    );
}

/// **A espessura preserva o PERFIL da pressão, e o slider é reversível.**
///
/// Um traço de caneta tem largura variável (pressão). Um `w_i := size` chapado destruiria
/// esse desenho em silêncio. O perfil (`w_i / w_max`) é re-imposto sobre a espessura nova,
/// e como escalar preserva razões, arrastar o slider ida-e-volta devolve o traço ORIGINAL.
///
/// Mutação que sangra: troque o `size * (w / max)` por `size` e a razão 1:2 vira 1:1.
#[test]
fn resizing_preserves_the_pressure_profile_and_is_reversible() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)], 4.0)]);
    d.strokes[0].selected = true;
    // Perfil de pressão 1:4:2, em unidades de MUNDO (§4.C.6) e coerente com o Size do
    // estilo `a` (8 → `size_to_world(8)` no pico): é assim que o `build_stroke` autora,
    // e é isso que torna o ida-e-volta EXATO em vez de só proporcional.
    let peak = ph2d_tool_flip::size_to_world(8.0);
    let w = d.strokes[0].widths_mut();
    w[0] = peak * 0.25;
    w[1] = peak;
    w[2] = peak * 0.5;
    let original: Vec<f32> = d.strokes[0].widths().to_vec();

    let a = style(); // width_px = 8
    let b = ph2d_tool_flip::FlipStyleSnapshot {
        width_px: 16.0,
        ..a
    };
    apply_style_delta(&mut d, &a, &b);
    let scaled: Vec<f32> = d.strokes[0].widths().to_vec();
    let peak16 = ph2d_tool_flip::size_to_world(16.0);
    assert!(
        (scaled[0] - peak16 * 0.25).abs() < 1e-6 && (scaled[1] - peak16).abs() < 1e-6,
        "o perfil da pressao (1:4:2) nao sobreviveu ao Size: {scaled:?}"
    );

    // E a volta devolve o traço.
    apply_style_delta(&mut d, &b, &a);
    let back: Vec<f32> = d.strokes[0].widths().to_vec();
    for (o, n) in original.iter().zip(back.iter()) {
        assert!(
            (o - n).abs() < 1e-3,
            "o slider nao e reversivel: {original:?} -> {back:?}"
        );
    }
}

/// **Um traço NÃO selecionado é intocável.** É a fronteira inteira da feature.
#[test]
fn an_unselected_stroke_is_never_touched() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 10.0), (10.0, 10.0)], 4.0),
    ]);
    d.strokes[1].selected = true;
    let before = d.strokes[0].clone();

    let a = style();
    let b = ph2d_tool_flip::FlipStyleSnapshot {
        stroke: [255, 0, 0, 255],
        width_px: 32.0,
        ..a
    };
    assert!(apply_style_delta(&mut d, &a, &b));
    assert_eq!(
        d.strokes[0], before,
        "o traco NAO selecionado foi editado pelo painel"
    );
}

/// 🔴 **A cor da LINHA e a cor do MIOLO são dois atributos independentes** (smoke do Enio,
/// 2026-07-13: *"a cor do stroke muda fill e stroke"*).
///
/// O 1º corte recoloria o miolo junto com a linha ("um traço com fill é uma coisa só") —
/// e isso tira do usuário a única forma de mudar uma sem a outra. São duas decisões de
/// arte, e cada uma tem o seu swatch (o painel ganhou o de Fill no modo Edit).
///
/// Mutação que sangra: volte a escrever `f.color = color` no ramo do `stroke`.
#[test]
fn the_stroke_colour_never_touches_the_fill_and_vice_versa() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)], 4.0)]);
    d.strokes[0].selected = true;
    let green = Rgba::new(0.1, 0.7, 0.2, 1.0);
    d.strokes[0].fill = Some(Fill {
        color: green,
        opacity: 1.0,
    });

    // Mexer no swatch do TRAÇO: a linha muda, o miolo NÃO.
    let a = style();
    let b = ph2d_tool_flip::FlipStyleSnapshot {
        stroke: [255, 0, 0, 255],
        ..a
    };
    assert!(apply_style_delta(&mut d, &a, &b));
    assert_eq!(
        d.strokes[0].fill.unwrap().color,
        green,
        "o swatch do TRACO recoloriu o MIOLO — sao dois atributos"
    );

    // Mexer no swatch do MIOLO: o miolo muda, a linha NÃO.
    let line_now = d.strokes[0].colors()[0];
    let c = ph2d_tool_flip::FlipStyleSnapshot {
        fill_color: [0, 0, 255, 255],
        ..b
    };
    assert!(apply_style_delta(&mut d, &b, &c));
    assert_ne!(
        d.strokes[0].fill.unwrap().color,
        green,
        "o swatch do MIOLO nao chegou no fill"
    );
    assert_eq!(
        d.strokes[0].colors()[0],
        line_now,
        "o swatch do MIOLO recoloriu a LINHA"
    );
}

/// **Uma REGIÃO do balde só responde ao swatch de Fill** — ela não tem linha visível
/// (`hide_stroke`), então o swatch do traço não a alcança.
#[test]
fn a_region_answers_only_to_the_fill_swatch() {
    let mut d = drawing(vec![region(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)])]);
    d.strokes[0].selected = true;
    let before = d.strokes[0].clone();

    let a = style();
    let b = ph2d_tool_flip::FlipStyleSnapshot {
        stroke: [255, 0, 0, 255],
        ..a
    };
    apply_style_delta(&mut d, &a, &b);
    assert_eq!(
        d.strokes[0].colors(),
        before.colors(),
        "o swatch do traco mexeu numa REGIAO (que nao tem linha)"
    );

    let c = ph2d_tool_flip::FlipStyleSnapshot {
        fill_color: [0, 0, 255, 255],
        ..b
    };
    assert!(apply_style_delta(&mut d, &b, &c));
    assert_ne!(
        d.strokes[0].fill.unwrap().color,
        before.fill.unwrap().color,
        "o swatch de Fill nao recoloriu a regiao"
    );
}

/// **O swatch de Fill RECOLORE, não CRIA.** Um traço sem miolo continua sem miolo.
#[test]
fn the_fill_swatch_recolours_but_never_creates_a_fill() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0)], 4.0)]);
    d.strokes[0].selected = true;
    let a = style();
    let b = ph2d_tool_flip::FlipStyleSnapshot {
        fill_color: [0, 0, 255, 255],
        ..a
    };
    apply_style_delta(&mut d, &a, &b);
    assert!(
        d.strokes[0].fill.is_none(),
        "o swatch de Fill INVENTOU um preenchimento num traco que era so linha"
    );
}

// ── O plano do pen-DOWN (`plan_down`): o colapso ADIADO ──

/// 🔴 **Pegar um traço de uma MULTISSELEÇÃO arrasta o GRUPO — não colapsa a seleção.**
///
/// O bug do smoke do Enio (2026-07-13): *"funciona a multisseleção mas não dá pra mover as
/// formas selecionadas juntas. só uma"*.
///
/// A causa: o pen-down num traço fazia `Pick::Replace` SEMPRE — inclusive quando o traço já
/// estava selecionado. A multisseleção morria no instante do toque, e o arrasto levava um
/// traço só. É por isso que todo editor (Illustrator, Figma, Blender) **adia** o colapso
/// para o pen-up: as duas leituras do mesmo gesto (colapsar × arrastar o grupo) só se
/// distinguem pelo que vem DEPOIS do down.
///
/// Mutação que sangra: tire o braço `if drawing.strokes[i].selected` do `plan_down` e a
/// seleção volta a colapsar no toque.
#[test]
fn grabbing_one_of_several_selected_strokes_keeps_the_whole_selection() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 10.0), (10.0, 10.0)], 4.0),
        line(&[(0.0, 20.0), (10.0, 20.0)], 4.0),
    ]);
    d.strokes[0].selected = true;
    d.strokes[2].selected = true;

    // Pen-down no traço 0, que JÁ está selecionado, sem Shift.
    let down = plan_down(&mut d, Some(0), false, false);

    assert_eq!(
        d.selected_indices(),
        vec![0, 2],
        "o toque num traco JA selecionado destruiu a multisselecao — e o arrasto levaria um so"
    );
    assert_eq!(
        down,
        Down::Move {
            collapse_to: Some(0)
        },
        "o gesto tem de ser MOVER (com o colapso ADIADO para o pen-up)"
    );
}

/// **E soltar SEM arrastar colapsa** — "agora só este". É a outra metade da regra: sem ela,
/// clicar num traço de uma multisseleção não teria como reduzi-la.
#[test]
fn releasing_without_dragging_collapses_the_selection_to_the_stroke() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 10.0), (10.0, 10.0)], 4.0),
    ]);
    d.strokes[0].selected = true;
    d.strokes[1].selected = true;

    let Down::Move {
        collapse_to: Some(i),
    } = plan_down(&mut d, Some(0), false, false)
    else {
        panic!("o down num traco selecionado tem de abrir um Move com colapso adiado");
    };
    // O pen-up SEM arrasto executa o colapso (é o que o `flip_edit_canvas_up` faz).
    assert!(apply_pick(&mut d, Some(i), Pick::Replace));
    assert_eq!(d.selected_indices(), vec![0], "o colapso nao aconteceu");
}

/// **Um traço NÃO selecionado vira a seleção no ato** (e não tem colapso a adiar).
#[test]
fn grabbing_an_unselected_stroke_selects_it_at_once() {
    let mut d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.0, 10.0), (10.0, 10.0)], 4.0),
    ]);
    d.strokes[1].selected = true;

    let down = plan_down(&mut d, Some(0), false, false);

    assert_eq!(
        d.selected_indices(),
        vec![0],
        "o traco novo nao virou a selecao"
    );
    assert_eq!(down, Down::Move { collapse_to: None });
}
