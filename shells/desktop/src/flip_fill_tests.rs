//! Testes do balde (`flip_fill`), num módulo-irmão pelo cap de LOC do HR-18.
//! Declarado pelo pai via `#[path]`, então `super` é `flip_fill`.

use super::*;
use ph2d_tool_flip::FillMode as ToolFillMode;

fn style(mode: ToolFillMode) -> FlipStyleSnapshot {
    FlipStyleSnapshot {
        fill_mode: mode,
        fill_color: [200, 100, 50, 255],
        gap_px: 0.0,
        grow: 2.0,
        precision: 1.0,
        ..Default::default()
    }
}

/// Um quadrado de line-art no desenho.
fn boxed_drawing() -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(20.0, 20.0),
        Vec2::new(0.0, 20.0),
    ] {
        s.push_point(Point {
            pos: p,
            width: 0.6,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s.closed = true;
    d.strokes.push(s);
    d
}

/// O caminho feliz: clicar dentro do quadrado cria UM traço de preenchimento — que
/// entra ATRÁS do line-art (a cor vai por baixo da linha).
#[test]
fn clicking_inside_creates_a_fill_behind_the_line_art() {
    let mut d = boxed_drawing();
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("um quadrado fechado preenche");
    assert_eq!(d.strokes.len(), 2);
    assert!(is_fill(&d.strokes[0]), "o fill entra ATRÁS do line-art");
    assert!(!d.strokes[1].hide_stroke, "o line-art continua visível");
    assert!(d.strokes[0].fill.is_some());
}

/// **Unpaint** remove o preenchimento sob o clique — e não toca no line-art.
#[test]
fn unpaint_removes_the_fill_under_the_click_only() {
    let mut d = boxed_drawing();
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    assert_eq!(d.strokes.len(), 2);

    fill_click(
        &mut d,
        &style(ToolFillMode::Unpaint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("há um fill sob o clique");
    assert_eq!(d.strokes.len(), 1, "só o fill saiu");
    assert!(!d.strokes[0].hide_stroke, "o line-art ficou");

    // E despintar o vazio não faz nada (nem entra em pânico).
    assert!(
        fill_click(
            &mut d,
            &style(ToolFillMode::Unpaint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .is_err()
    );
}

/// **Um fill anterior NÃO é fronteira** — senão a 2ª cor nunca entraria por baixo
/// da 1ª, e o `Paint Behind` seria impossível. (Um fechamento de gap, que também é
/// invisível, É fronteira: os dois se distinguem pelo `fill`.)
#[test]
fn an_existing_fill_is_not_a_boundary() {
    let mut d = boxed_drawing();
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    // O 2º clique preenche a MESMA região (o fill velho não a dividiu).
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("o fill anterior não pode barrar o novo");
    assert_eq!(d.strokes.len(), 3, "dois fills + o line-art");
}

/// **Gap Closure:** um "C" aberto não preenche… até o Gap Closure fechá-lo — e o
/// fechamento fica no desenho como traço invisível PERSISTENTE.
#[test]
fn gap_closure_leaves_a_persistent_invisible_stroke() {
    let mut d = FlipDrawing::new();
    // Um "C" e uma parede à direita (as pontas do C apontam para ela).
    let mut c = FlipStroke::new();
    for p in [
        Vec2::new(20.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 20.0),
        Vec2::new(20.0, 20.0),
    ] {
        c.push_point(Point {
            pos: p,
            width: 0.6,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    d.strokes.push(c);
    // A parede fica a 10 unidades das pontas do C — bem além do filtro de
    // vazamento cruzado (3px), então SEM Gap Closure isto vaza de verdade.
    let mut wall = FlipStroke::new();
    for p in [Vec2::new(30.0, -2.0), Vec2::new(30.0, 22.0)] {
        wall.push_point(Point {
            pos: p,
            width: 0.6,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    d.strokes.push(wall);

    // Sem Gap Closure: vaza.
    let err = fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap_err();
    assert_eq!(err, FillError::Leaked, "o C aberto tem de recusar");

    // Com Gap Closure: fecha, e o fechamento FICA no documento.
    let s = FlipStyleSnapshot {
        gap_px: 15.0,
        ..style(ToolFillMode::Paint)
    };
    fill_click(&mut d, &s, Vec2::new(10.0, 10.0), 1.0, &Xform::IDENTITY)
        .expect("com Gap Closure, o C fecha");
    let closures = d
        .strokes
        .iter()
        .filter(|st| st.hide_stroke && st.fill.is_none())
        .count();
    assert!(
        closures >= 1,
        "o fechamento tem de virar traço invisível persistente"
    );
    // E ele é FRONTEIRA no próximo fill: re-preencher (outra cor) funciona SEM o
    // Gap Closure ligado — o vão ficou fechado. É todo o ponto do twist do Harmony.
    let mut plain = style(ToolFillMode::Paint);
    plain.fill_color = [10, 200, 10, 255];
    plain.gap_px = 0.0;
    fill_click(&mut d, &plain, Vec2::new(10.0, 10.0), 1.0, &Xform::IDENTITY)
        .expect("o vão já está fechado: re-preencher não precisa do Gap Closure");
}

/// **O bug que matou o balde no produto (auditoria 2026-07-12).**
///
/// Todos os testes acima passam `px_to_world = 1.0` — o ÚNICO valor em que um px
/// de tela vale uma unidade de documento. Foi exatamente esse valor que escondeu a
/// conversão faltante: com a câmera de verdade (`height_world = 10` numa janela de
/// 1080p, `px_to_world ≈ 0,0093`), a meia-espessura de um traço de 6 px era lida
/// como **3 unidades de mundo** — uma linha de ~324 px atravessando um desenho de
/// 2,8 unidades. O clique caía sempre dentro do traço.
///
/// Este teste usa os números do PRODUTO. Mutação que sangra: tire o `px_to_world`
/// do `boundaries` e ele volta a `OnBoundary`.
#[test]
fn the_bucket_fills_at_the_real_camera_scale() {
    // Câmera default: 10 unidades de mundo na altura de uma janela de 1080p.
    let px_to_world = 10.0 / 1080.0;
    // Um quadrado de ~300 px na tela, desenhado com o pincel de 6 px.
    let side = 300.0 * px_to_world;
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(0.0, 0.0),
        Vec2::new(side, 0.0),
        Vec2::new(side, side),
        Vec2::new(0.0, side),
    ] {
        s.push_point(Point {
            pos: p,
            width: 6.0, // px de TELA (o que o `build_stroke` guarda)
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s.closed = true;
    d.strokes.push(s);

    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(side * 0.5, side * 0.5),
        px_to_world,
        &Xform::IDENTITY,
    )
    .expect("um quadrado fechado no zoom PADRAO tem de preencher");

    assert!(is_fill(&d.strokes[0]), "o fill entra atras do line-art");
    // E a regiao preenchida cobre a maior parte do quadrado (nao uma casquinha).
    let area = {
        let r = d.strokes[0].positions();
        let n = r.len();
        (0..n)
            .map(|i| {
                let (a, b) = (r[i], r[(i + 1) % n]);
                a.x * b.y - b.x * a.y
            })
            .sum::<f32>()
            .abs()
            * 0.5
    };
    let full = side * side;
    assert!(
        area > full * 0.8,
        "o fill cobriu so {:.0}% do quadrado (a espessura ainda esta em unidade errada)",
        area / full * 100.0
    );
}

/// **Reajustar parte da BASE PRISTINA — nunca do resultado anterior.**
///
/// É o coração do alvo vivo (`flip_live`). Um fill com Gap cria fechamentos
/// PERSISTENTES no desenho; se o reajuste rodasse sobre o resultado anterior, cada
/// mexida no slider empilharia mais um jogo de fechamentos e mais um preenchimento —
/// os parâmetros se COMPORIAM em vez de se substituírem, e arrastar o slider para a
/// frente e para trás não voltaria ao mesmo lugar. (É o mesmo erro do offset
/// acumulador do Painter, `docs/Painter/09`.)
///
/// Aqui: um "C" aberto. Preencher com Gap, restaurar a base, preencher de novo com o
/// MESMO Gap → o desenho tem de ficar IDÊNTICO. E o caminho errado (sem restaurar)
/// tem de crescer — senão o teste não estaria provando nada.
#[test]
fn readjusting_from_the_pristine_base_replaces_instead_of_accumulating() {
    // Um quadrado QUASE fechado: falta o pedacinho de (0,4) até (0,0). A ponta em
    // (0,4) aponta para baixo, direto na linha de baixo — que é o vão que o Gap
    // Closure existe para fechar. (Um "C" com as pontas apontando para LONGE do vão
    // não fecha nem no GP: a extensão é na TANGENTE.)
    let open_c = || {
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        for p in [
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 0.0),
            Vec2::new(20.0, 20.0),
            Vec2::new(0.0, 20.0),
            Vec2::new(0.0, 4.0),
        ] {
            s.push_point(Point {
                pos: p,
                width: 0.6,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        d.strokes.push(s);
        d
    };
    let with_gap = |gap: f64| FlipStyleSnapshot {
        gap_px: gap,
        ..style(ToolFillMode::Paint)
    };

    let base = open_c().strokes;

    // 1ª aplicação, com Gap.
    let mut d = open_c();
    fill_click(
        &mut d,
        &with_gap(30.0),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("com Gap, o C fecha e preenche");
    let after_first = d.strokes.len();
    assert!(after_first > base.len(), "o fill tem de ter criado algo");

    // **O caminho CERTO**: restaura a base e reaplica → idêntico.
    d.strokes = base.clone();
    fill_click(
        &mut d,
        &with_gap(30.0),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    assert_eq!(
        d.strokes.len(),
        after_first,
        "reaplicar da base pristina tem de SUBSTITUIR o resultado, nao empilhar outro"
    );

    // **O caminho ERRADO** (sem restaurar): cresce. Se um dia isto parar de crescer,
    // o teste acima virou vácuo — e este aqui avisa.
    let mut wrong = open_c();
    fill_click(
        &mut wrong,
        &with_gap(30.0),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    let once = wrong.strokes.len();
    fill_click(
        &mut wrong,
        &with_gap(30.0),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    assert!(
        wrong.strokes.len() > once,
        "reaplicar SEM restaurar a base deveria acumular — se nao acumula, o gate de \
             cima nao prova nada"
    );
}
