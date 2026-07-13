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

/// **Clicar dentro de uma FORMA FECHADA pinta a própria forma** (o Suzanne).
///
/// Nada de contorno vetorizado: o preenchimento vira o `fill` do traço que já está lá, e
/// a cor passa a ser a triangulação dos pontos DELE. Um conjunto de vértices só — nada
/// para dessincronizar, em nenhum zoom, nunca (smoke do Enio 2026-07-13).
///
/// Mutação que sangra: remova o `filled_shape_target` e o balde volta a criar um traço
/// novo, com vértices próprios — e este teste vê dois traços em vez de um.
#[test]
fn clicking_inside_a_closed_shape_paints_the_shape_itself() {
    let mut d = boxed_drawing();
    let before = d.strokes[0].positions().to_vec();
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("um quadrado fechado preenche");

    assert_eq!(
        d.strokes.len(),
        1,
        "o balde criou um traco NOVO em vez de pintar a forma que ja existia"
    );
    let s = &d.strokes[0];
    assert!(s.fill.is_some(), "a forma nao ganhou a cor");
    assert!(
        !s.hide_stroke,
        "e ela continua sendo LINE-ART (a linha aparece)"
    );
    assert_eq!(
        s.positions(),
        &before[..],
        "os pontos da linha mudaram — a cor tem de usar OS MESMOS"
    );
}

/// **Unpaint tira a COR — e nunca a linha.**
///
/// Uma região (o traço invisível que o balde cria entre várias linhas) some inteira: ela
/// é só cor. Mas um traço PREENCHIDO é line-art que por acaso carrega cor — apagá-lo
/// levaria o desenho junto. O Unpaint tira o `fill` e deixa a linha.
#[test]
fn unpaint_removes_the_colour_but_never_the_line() {
    let mut d = boxed_drawing();
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    assert!(d.strokes[0].fill.is_some());

    fill_click(
        &mut d,
        &style(ToolFillMode::Unpaint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("ha cor sob o clique");
    assert_eq!(d.strokes.len(), 1, "a LINHA foi apagada junto com a cor");
    assert!(d.strokes[0].fill.is_none(), "a cor nao saiu");
    assert!(!d.strokes[0].hide_stroke, "e a linha continua la");

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

/// **Repintar a forma TROCA a cor** — não empilha um segundo preenchimento.
#[test]
fn painting_a_shape_twice_just_changes_its_colour() {
    let mut d = boxed_drawing();
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .unwrap();
    let first = d.strokes[0].fill.unwrap().color;

    let mut other = style(ToolFillMode::Paint);
    other.fill_color = [10, 200, 10, 255];
    fill_click(&mut d, &other, Vec2::new(10.0, 10.0), 1.0, &Xform::IDENTITY).unwrap();

    assert_eq!(d.strokes.len(), 1, "o 2o clique empilhou um traco");
    let second = d.strokes[0].fill.unwrap().color;
    assert_ne!(first, second, "a cor nao trocou");
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

    assert!(
        d.strokes[0].fill.is_some(),
        "a forma fechada tem de pintar A SI MESMA (nenhum contorno vetorizado)"
    );
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

/// **E a região entre VÁRIOS traços continua sendo vetorizada** — porque ali não existe
/// "a curva" para carregar a cor.
///
/// É o caso de colorir entre linhas (o uso de produção), e é também o que o balde do
/// Grease Pencil faz: um traço novo, invisível, com o contorno que o solver traçou. A
/// diferença entre os dois caminhos não é um detalhe de implementação — é a resposta à
/// pergunta *"esta região É uma forma?"*.
#[test]
fn a_region_bounded_by_several_strokes_still_gets_a_traced_contour() {
    let mut d = FlipDrawing::new();
    // Duas linhas em "L" + duas fechando: quatro traços ABERTOS formando um quadrado.
    for (a, b) in [
        (Vec2::new(0.0, 0.0), Vec2::new(20.0, 0.0)),
        (Vec2::new(20.0, 0.0), Vec2::new(20.0, 20.0)),
        (Vec2::new(20.0, 20.0), Vec2::new(0.0, 20.0)),
        (Vec2::new(0.0, 20.0), Vec2::new(0.0, 0.0)),
    ] {
        let mut s = FlipStroke::new();
        for p in [a, b] {
            s.push_point(Point {
                pos: p,
                width: 0.6,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        d.strokes.push(s);
    }
    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("a regiao entre os quatro tracos preenche");

    assert_eq!(d.strokes.len(), 5, "faltou o traco de preenchimento");
    assert!(
        is_fill(&d.strokes[0]),
        "a regiao multi-traco tem de virar um traco de fill VETORIZADO (nenhuma das \
         quatro linhas e 'a forma')"
    );
}

/// **Uma forma desenhada À MÃO pinta a si mesma** — mesmo sendo `closed = false`.
///
/// Era o bug do smoke do Enio (2026-07-13): a caneta só fecha o traço no modo
/// `Shape: Filled`, então uma forma desenhada à mão, com as pontas se encontrando na tela,
/// tem `closed = false`. O balde exigia `closed` para reconhecer a forma — logo, **no
/// produto o auto-preenchimento NUNCA disparava**, e todo fill do usuário caía no contorno
/// vetorizado (o que dessincroniza dos vértices da linha e o zoom amplia).
///
/// Mutação que sangra: devolva o `s.closed` ao filtro do `filled_shape_target` e este teste
/// vê dois traços (o vetorizado) em vez de um.
#[test]
fn a_hand_drawn_shape_paints_itself_even_though_it_is_not_closed() {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(20.0, 20.0),
        Vec2::new(0.0, 20.0),
        Vec2::new(0.2, 0.3), // a mão volta ao começo, mas NÃO fecha o traço
    ] {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    // `closed` é FALSE — é o que `flip_draw::build_stroke` produz fora do Shape: Filled.
    assert!(!s.closed);
    d.strokes.push(s);

    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(10.0, 10.0),
        1.0,
        &Xform::IDENTITY,
    )
    .expect("a forma desenhada a mao preenche");

    assert_eq!(
        d.strokes.len(),
        1,
        "o balde criou um contorno VETORIZADO em vez de pintar a forma — e e por isso \
         que os vertices dessincronizam no produto"
    );
    assert!(
        d.strokes[0].fill.is_some(),
        "a forma desenhada a mao tinha de pintar A SI MESMA"
    );
}
