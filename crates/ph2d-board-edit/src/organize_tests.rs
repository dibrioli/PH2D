//! **Organizar** (W5) pelo caminho de quem usa: frames, grupos, travar, ordem, alinhar, distribuir.
//!
//! Oráculo: `@excalidraw/excalidraw` 0.18.1 (MIT), corrido por
//! `docs/MiroClone/ferramentas/excalidraw_oracle/corre_organizar.mjs` (2026-10-07, duas corridas
//! byte a byte iguais). Os números das fixtures `saidas/organizar_<caso>.json`, transcritos:
//!
//! | caso | antes | depois |
//! |---|---|---|
//! | C1 esquerda/direita/topo/fundo | Ra 400,250 113×71 · Rb 537,380 61×97 · Rc 459,520 173×41 | x 400·400·400 / x 519·571·459 / y 250·250·250 / y 490·464·520 |
//! | C1 centro h / v | idem | x 459,5·485,5·429,5 / y 370·357·385 |
//! | C2 grupo + solto, esquerda | Ga 400,250 · Gb 450,350 (grupo) · Lc 337,520 | Ga 337 · Gb 387 (o grupo inteiro) |
//! | C3 distribuir h | D1 337 41 · D2 413 113 · D3 601 29 · D4 700 87 | D2 438 · D3 611 (vão 60, pontas paradas) |
//! | C4 rodado 0,5 rad (120×40 em 500,300) | Ref 537,450 80×60 | esquerda x 497,757 · topo y 273,683 (a caixa envolvente) |
//! | D5a `Ctrl+]` em E0, grupo g (E2,E3) a seguir | E0 · E2 · E3 · E1 · E4 | E2 · E3 · E0 · E1 · E4 (salta o grupo inteiro) |
//!
//! ⛔ **Divergência escolhida** (D1/D2): o passo do Excalidraw troca com o VIZINHO DA LISTA mesmo
//! sem se tocarem (E0 passa E1, que está longe). O nosso segue o produto do dono, o Miro (help
//! «Working with objects»: *«Bring forward: Move an object in front of the object directly in front
//! of it»*): passa o primeiro de cima que lhe TOCA — o passo tem sempre efeito à vista.

use super::*;
use ph2d_board_model::GroupId;

const R: f64 = 1e-3;

fn put_rect(w: &mut World, id_bx: [f64; 4]) -> ElementId {
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: w.ed.style.clone(),
        text: Default::default(),
    };
    let el = Element::new_shape(w.doc.mint_id(), w.doc.z_on_top(), shape, id_bx);
    let id = el.id;
    BoardOp::Put(el).apply(&mut w.doc);
    id
}

fn frame(w: &mut World, from: [f64; 2], to: [f64; 2]) -> ElementId {
    w.ed.tool = Tool::Frame;
    w.drag(from, to, CTRL);
    let id = *w.ed.selection().iter().next().expect("nasceu seleccionado");
    assert!(w.el(id).as_frame().is_some());
    id
}

fn xs(w: &World, ids: &[ElementId]) -> Vec<f64> {
    ids.iter().map(|id| w.el(*id).x).collect()
}

fn ys(w: &World, ids: &[ElementId]) -> Vec<f64> {
    ids.iter().map(|id| w.el(*id).y).collect()
}

fn near(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < R)
}

fn three(w: &mut World) -> [ElementId; 3] {
    [
        put_rect(w, [400.0, 250.0, 113.0, 71.0]),
        put_rect(w, [537.0, 380.0, 61.0, 97.0]),
        put_rect(w, [459.0, 520.0, 173.0, 41.0]),
    ]
}

/// ⭐ Oráculo C1: a referência é a caixa da SELECÇÃO, nos seis sentidos.
#[test]
fn align_takes_the_selection_box_as_reference_like_the_oracle() {
    let cases: [(Align, [f64; 3], bool); 6] = [
        (Align::Left, [400.0, 400.0, 400.0], true),
        (Align::Right, [519.0, 571.0, 459.0], true),
        (Align::CenterH, [459.5, 485.5, 429.5], true),
        (Align::Top, [250.0, 250.0, 250.0], false),
        (Align::Bottom, [490.0, 464.0, 520.0], false),
        (Align::CenterV, [370.0, 357.0, 385.0], false),
    ];
    for (a, want, horizontal) in cases {
        let mut w = world();
        let ids = three(&mut w);
        w.ed.select(&w.doc, ids);
        assert!(w.cmd(Command::Align(a)), "{a:?} mexeu");
        let got = if horizontal {
            xs(&w, &ids)
        } else {
            ys(&w, &ids)
        };
        assert!(near(&got, &want), "{a:?}: {got:?} ≠ oráculo {want:?}");
        assert!(w.cmd(Command::Undo));
        assert!(near(&xs(&w, &ids), &[400.0, 537.0, 459.0]), "UM passo");
    }
}

/// Oráculo C2: um grupo alinha-se INTEIRO (as distâncias de dentro ficam).
#[test]
fn a_group_aligns_as_one_unit() {
    let mut w = world();
    let ga = put_rect(&mut w, [400.0, 250.0, 113.0, 71.0]);
    let gb = put_rect(&mut w, [450.0, 350.0, 61.0, 97.0]);
    let lc = put_rect(&mut w, [337.0, 520.0, 173.0, 41.0]);
    w.ed.select(&w.doc, [ga, gb]);
    assert!(w.cmd(Command::Group));
    w.ed.select(&w.doc, [ga, gb, lc]);
    assert!(w.cmd(Command::Align(Align::Left)));
    assert!(near(&xs(&w, &[ga, gb, lc]), &[337.0, 387.0, 337.0]));
}

/// Oráculo C3: vãos IGUAIS entre caixas, as duas pontas paradas (vão 60).
#[test]
fn distribute_makes_equal_gaps_and_keeps_the_ends() {
    let mut w = world();
    let ids = [
        put_rect(&mut w, [337.0, 300.0, 41.0, 60.0]),
        put_rect(&mut w, [413.0, 280.0, 113.0, 90.0]),
        put_rect(&mut w, [601.0, 310.0, 29.0, 50.0]),
        put_rect(&mut w, [700.0, 295.0, 87.0, 70.0]),
    ];
    w.ed.select(&w.doc, ids);
    assert!(w.cmd(Command::Distribute(Axis::Horizontal)));
    assert!(near(&xs(&w, &ids), &[337.0, 438.0, 611.0, 700.0]));
    assert!(
        near(&ys(&w, &ids), &[300.0, 280.0, 310.0, 295.0]),
        "só o eixo pedido"
    );
    w.ed.select(&w.doc, ids[..2].iter().copied());
    assert!(
        !w.cmd(Command::Distribute(Axis::Horizontal)),
        "com dois não há o que distribuir"
    );
}

/// Oráculo C4: um rodado entra pela caixa ENVOLVENTE.
#[test]
fn a_rotated_element_aligns_by_its_bounding_box() {
    for (a, want) in [(Align::Left, 497.757), (Align::Top, 273.683)] {
        let mut w = world();
        let rot = put_rect(&mut w, [500.0, 300.0, 120.0, 40.0]);
        let mut el = w.el(rot).clone();
        el.angle = 0.5;
        BoardOp::Put(el).apply(&mut w.doc);
        let r = put_rect(&mut w, [537.0, 450.0, 80.0, 60.0]);
        w.ed.select(&w.doc, [rot, r]);
        assert!(w.cmd(Command::Align(a)));
        let got = if a == Align::Left {
            w.el(r).x
        } else {
            w.el(r).y
        };
        assert!((got - want).abs() < R, "{a:?}: {got} ≠ {want}");
    }
}

fn z_order(w: &World) -> Vec<ElementId> {
    w.doc.live_in_z_order().iter().map(|e| e.id).collect()
}

/// A cena D do oráculo: E0..E4.
fn order_scene(w: &mut World) -> [ElementId; 5] {
    [
        put_rect(w, [400.0, 250.0, 100.0, 80.0]),
        put_rect(w, [700.0, 250.0, 80.0, 80.0]),
        put_rect(w, [450.0, 290.0, 100.0, 80.0]),
        put_rect(w, [900.0, 400.0, 60.0, 60.0]),
        put_rect(w, [470.0, 300.0, 60.0, 60.0]),
    ]
}

/// ⭐ Um passo à frente passa o primeiro de cima que lhe TOCA (o Miro — divergência escolhida do
/// oráculo D1, cabeçalho); atrás, o primeiro de baixo que lhe toca; sem nenhum, nada muda.
#[test]
fn a_step_forward_passes_the_first_one_above_that_touches_it() {
    let mut w = world();
    let [e0, e1, e2, e3, e4] = order_scene(&mut w);
    w.ed.select(&w.doc, [e0]);
    assert!(w.cmd(Command::Order(Order::Forward)));
    assert_eq!(z_order(&w), vec![e1, e2, e0, e3, e4]);
    w.ed.select(&w.doc, [e4]);
    assert!(w.cmd(Command::Order(Order::Backward)));
    assert_eq!(
        z_order(&w),
        vec![e1, e2, e4, e0, e3],
        "E4 passa para trás de E0 (que subiu): o primeiro de baixo que lhe toca; E3, longe, não conta"
    );
    w.ed.select(&w.doc, [e3]);
    assert!(
        !w.cmd(Command::Order(Order::Forward)),
        "E3 não toca ninguém: nada a fazer"
    );
}

/// Oráculo D5a: um vizinho de um grupo salta-se INTEIRO; à frente/atrás de tudo vai às pontas.
#[test]
fn a_step_jumps_a_whole_group_and_front_back_go_to_the_ends() {
    let mut w = world();
    let e0 = put_rect(&mut w, [400.0, 250.0, 100.0, 80.0]);
    let e2 = put_rect(&mut w, [450.0, 290.0, 100.0, 80.0]);
    let e3 = put_rect(&mut w, [520.0, 320.0, 60.0, 60.0]);
    let e1 = put_rect(&mut w, [700.0, 250.0, 80.0, 80.0]);
    let e4 = put_rect(&mut w, [470.0, 300.0, 60.0, 60.0]);
    w.ed.select(&w.doc, [e2, e3]);
    assert!(w.cmd(Command::Group));
    w.ed.select(&w.doc, [e0]);
    assert!(w.cmd(Command::Order(Order::Forward)));
    assert_eq!(
        z_order(&w),
        vec![e2, e3, e0, e1, e4],
        "o oráculo, número a número"
    );
    w.ed.select(&w.doc, [e0]);
    assert!(w.cmd(Command::Order(Order::Front)));
    assert_eq!(*z_order(&w).last().unwrap(), e0);
    w.ed.select(&w.doc, [e4, e1]);
    assert!(w.cmd(Command::Order(Order::Back)));
    assert_eq!(z_order(&w)[..2], [e1, e4], "a selecção mantém a ordem dela");
    assert!(w.cmd(Command::Undo));
    assert_eq!(*z_order(&w).last().unwrap(), e0, "UM passo");
}

/// ⭐ Oráculo B: um clique num membro apanha o grupo de FORA; o duplo-clique entra um nível; um
/// clique no vazio sai; desagrupar tira só o de fora.
#[test]
fn a_click_picks_the_outer_group_and_a_double_click_enters_one_level() {
    let mut w = world();
    let a = put_rect(&mut w, [0.0, 0.0, 50.0, 50.0]);
    let b = put_rect(&mut w, [100.0, 0.0, 50.0, 50.0]);
    let c = put_rect(&mut w, [200.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [a, b]);
    assert!(w.cmd(Command::Group));
    w.ed.select(&w.doc, [a, b, c]);
    assert!(w.cmd(Command::Group));
    let (inner, outer): (GroupId, GroupId) = (w.el(a).groups()[0], w.el(a).groups()[1]);
    assert_eq!(w.el(c).groups(), vec![outer], "[de dentro, de fora]");
    w.click([-500.0, -500.0], NONE);
    w.click([25.0, 25.0], NONE);
    assert_eq!(w.ed.selection().len(), 3, "o clique apanha o grupo de fora");
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(25.0, 25.0, NONE)));
    assert_eq!(
        *w.ed.selection(),
        BTreeSet::from([a, b]),
        "entrou: o grupo de dentro"
    );
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(25.0, 25.0, NONE)));
    assert_eq!(
        *w.ed.selection(),
        BTreeSet::from([a]),
        "mais um: o elemento"
    );
    w.click([-500.0, -500.0], NONE);
    w.click([25.0, 25.0], NONE);
    assert_eq!(w.ed.selection().len(), 3, "o vazio saiu do grupo");
    assert!(w.cmd(Command::Ungroup));
    assert_eq!(w.el(a).groups(), vec![inner]);
    assert!(w.el(c).groups().is_empty());
}

/// Um grupo arrasta-se inteiro por um clique num membro.
#[test]
fn dragging_a_member_moves_the_whole_group() {
    let mut w = world();
    let a = put_rect(&mut w, [0.0, 0.0, 50.0, 50.0]);
    let b = put_rect(&mut w, [100.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [a, b]);
    assert!(w.cmd(Command::Group));
    w.click([-500.0, -500.0], NONE);
    w.drag([25.0, 25.0], [25.0, 225.0], CTRL);
    assert!(near(&ys(&w, &[a, b]), &[200.0, 200.0]));
}

/// O Miro: *«You can't group … objects located in different frames»* — e a recusa chega à vista.
#[test]
fn grouping_across_frames_is_refused_with_a_notice() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [300.0, 300.0]);
    let inside = put_rect(&mut w, [10.0, 10.0, 50.0, 50.0]);
    let mut el = w.el(inside).clone();
    el.frame = Some(f);
    BoardOp::Put(el).apply(&mut w.doc);
    let outside = put_rect(&mut w, [500.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [inside, outside]);
    assert!(!w.cmd(Command::Group));
    assert_eq!(w.ed.take_notice(), Some(Notice::GroupAcrossFrames));
    assert!(w.el(inside).groups().is_empty());
}

/// ⭐ Travado (o Miro + oráculo E): o clique selecciona-o mas o arrasto não o move, o laço e o
/// `Ctrl+A` deixam-no de fora, apagar e escrever recusam — sempre com o aviso.
#[test]
fn a_locked_element_cannot_be_moved_deleted_or_written_and_marquee_skips_it() {
    let mut w = world();
    let l = put_rect(&mut w, [0.0, 0.0, 50.0, 50.0]);
    let free = put_rect(&mut w, [100.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [l]);
    assert!(w.cmd(Command::Lock));
    assert!(w.el(l).locked());
    w.click([-500.0, -500.0], NONE);
    w.drag([25.0, 25.0], [25.0, 225.0], CTRL);
    assert_eq!(w.el(l).y, 0.0, "não se moveu");
    assert_eq!(
        *w.ed.selection(),
        BTreeSet::from([l]),
        "mas seleccionou-se (para o destravar)"
    );
    assert_eq!(w.ed.take_notice(), Some(Notice::Locked));
    w.ed.select(&w.doc, [l]);
    w.cmd(Command::Nudge([10.0, 0.0]));
    assert_eq!(w.el(l).x, 0.0, "o empurrão das setas moveu o travado");
    assert!(!w.cmd(Command::EditText));
    assert!(!w.cmd(Command::Delete));
    assert!(w.doc.get(l).is_some());
    w.click([-500.0, -500.0], NONE);
    w.drag([-10.0, -10.0], [200.0, 100.0], NONE);
    assert_eq!(
        *w.ed.selection(),
        BTreeSet::from([free]),
        "o laço salta o travado"
    );
    assert!(w.cmd(Command::SelectAll));
    assert_eq!(*w.ed.selection(), BTreeSet::from([free]), "o Ctrl+A também");
    w.ed.select(&w.doc, [l]);
    assert!(w.cmd(Command::Lock), "o mesmo atalho destrava");
    assert!(!w.el(l).locked());
}

/// ⭐ Oráculo A1: o frame desenhado adopta só o que lhe fica INTEIRO dentro — no MESMO passo.
#[test]
fn a_drawn_frame_adopts_only_what_is_fully_inside_in_one_step() {
    let mut w = world();
    let full = put_rect(&mut w, [50.0, 50.0, 40.0, 40.0]);
    let half = put_rect(&mut w, [280.0, 50.0, 60.0, 40.0]);
    let center_in = put_rect(&mut w, [250.0, 150.0, 80.0, 40.0]);
    let f = frame(&mut w, [0.0, 0.0], [300.0, 300.0]);
    assert_eq!(w.el(full).frame, Some(f));
    assert_eq!(w.el(half).frame, None);
    assert_eq!(
        w.el(center_in).frame,
        None,
        "centro dentro mas a caixa atravessa a borda"
    );
    assert!(w.cmd(Command::Undo));
    assert!(w.doc.get(f).is_none());
    assert_eq!(
        w.el(full).frame,
        None,
        "o desfazer leva o frame e a adopção juntos"
    );
}

/// Oráculo A1b–A1e: uma forma desenhada vive no frame onde o arrasto COMEÇOU.
#[test]
fn a_drawn_shape_lives_where_the_drag_started() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [300.0, 300.0]);
    let inner = frame(&mut w, [20.0, 20.0], [120.0, 120.0]);
    assert_eq!(w.el(inner).frame, Some(f));
    let deep = w.rect([50.0, 50.0, 30.0, 30.0]);
    assert_eq!(
        w.el(deep).frame,
        Some(inner),
        "vive no frame mais INTERIOR debaixo do início"
    );
    let inside = w.rect([280.0, 280.0, 100.0, 100.0]);
    assert_eq!(w.el(inside).frame, Some(f), "começou dentro, acabou fora");
    let outside = w.rect([-50.0, -50.0, 100.0, 100.0]);
    assert_eq!(w.el(outside).frame, None, "começou fora, atravessa a borda");
}

/// ⭐ Oráculo A4/A5: ao largar, quem decide é o PONTEIRO — nem o centro, nem a área.
#[test]
fn the_pointer_at_drop_decides_the_frame() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [300.0, 300.0]);
    let r = put_rect(&mut w, [400.0, 100.0, 100.0, 50.0]);
    // Agarrado pela ponta esquerda: o ponteiro entra, o centro fica fora.
    w.click([-500.0, -500.0], NONE);
    w.drag([405.0, 125.0], [295.0, 125.0], CTRL);
    assert!(w.el(r).center()[0] > 300.0, "o centro ficou fora");
    assert_eq!(w.el(r).frame, Some(f));
    // Agarrado pela ponta direita, a caixa quase toda dentro, o ponteiro fora: sai.
    w.click([-500.0, -500.0], NONE);
    w.drag([385.0, 125.0], [305.0, 125.0], CTRL);
    assert!(w.el(r).center()[0] < 300.0, "o centro ficou dentro");
    assert_eq!(w.el(r).frame, None);
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(r).frame, Some(f), "a casa volta com o desfazer");
}

/// ⭐ Mover um frame leva o que vive dentro dele (A6) — a qualquer profundidade (frames aninhados:
/// a queixa aberta no Miro, ideia 3902) — e o empurrão do teclado também.
#[test]
fn moving_a_frame_carries_everything_inside_at_any_depth() {
    let mut w = world();
    let outer = frame(&mut w, [0.0, 0.0], [600.0, 600.0]);
    let inner = frame(&mut w, [100.0, 100.0], [300.0, 300.0]);
    assert_eq!(
        w.el(inner).frame,
        Some(outer),
        "desenhado dentro, aninha-se"
    );
    let r = w.rect([150.0, 150.0, 40.0, 40.0]);
    assert_eq!(w.el(r).frame, Some(inner));
    w.ed.select(&w.doc, [outer]);
    assert!(w.cmd(Command::Nudge([10.0, 0.0])));
    assert!(near(&xs(&w, &[outer, inner, r]), &[10.0, 110.0, 160.0]));
    // Pela faixa do título (acima do frame): um arrasto leva tudo.
    w.click([-500.0, -500.0], NONE);
    w.drag([300.0, -5.0], [300.0, 95.0], CTRL);
    assert!(near(&ys(&w, &[outer, inner, r]), &[100.0, 200.0, 250.0]));
    assert_eq!(w.el(r).frame, Some(inner), "a casa dos de dentro não muda");
}

/// ⭐ Carregar no VAZIO de dentro de um frame agarra o FRAME, como uma forma (ordem do dono, 08/10:
/// *«transformar como as outras shapes clicando e arrastando dentro deles, desde que num espaço
/// vazio»*) — o que está dentro ganha o clique; e o frame leva o que é dele, TRAVADO incluído
/// (*«a nota com o cadeado não segue o frame»*).
#[test]
fn pressing_the_empty_inside_of_a_frame_grabs_it_and_it_carries_even_the_locked() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [600.0, 600.0]);
    let a = w.rect([300.0, 300.0, 40.0, 40.0]);
    let l = w.rect([100.0, 100.0, 40.0, 40.0]);
    w.ed.select(&w.doc, [l]);
    assert!(w.cmd(Command::Lock));
    w.click([-500.0, -500.0], NONE);
    w.click([320.0, 320.0], NONE);
    assert_eq!(
        *w.ed.selection(),
        BTreeSet::from([a]),
        "o que está dentro ganha o clique"
    );
    w.click([-500.0, -500.0], NONE);
    w.drag([250.0, 250.0], [250.0, 350.0], CTRL);
    assert!(
        near(&ys(&w, &[f, a, l]), &[100.0, 400.0, 200.0]),
        "o frame levou tudo, o travado também"
    );
    assert_eq!(w.el(l).frame, Some(f));
    w.ed.select(&w.doc, [f]);
    assert!(w.cmd(Command::Nudge([10.0, 0.0])));
    assert!(
        near(&xs(&w, &[f, l]), &[10.0, 110.0]),
        "o empurrão também leva o travado"
    );
    assert!(w.cmd(Command::Undo) && w.cmd(Command::Undo));
    assert!(
        near(&ys(&w, &[f, l]), &[0.0, 100.0]),
        "cada gesto, UM passo"
    );
}

/// ⭐ Um frame RODA como as formas, e leva o que vive dentro à volta do mesmo centro; a casa dos
/// de dentro continua a ser ele (a pertença lê a caixa RODADA).
#[test]
fn rotating_a_frame_turns_its_contents_with_it() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [400.0, 200.0]);
    let a = w.rect([320.0, 80.0, 40.0, 40.0]);
    w.ed.select(&w.doc, [f]);
    let knob = crate::Frame::of(w.el(f)).handle(Handle::Rotate, PX, &metrics());
    // Do topo para a direita do centro (200,100): um quarto de volta no sentido do relógio.
    w.drag(knob, [200.0 + 300.0, 100.0], NONE);
    let q = std::f64::consts::FRAC_PI_2;
    assert!(
        (w.el(f).angle - q).abs() < 1e-6,
        "o frame rodou {}",
        w.el(f).angle
    );
    assert!(
        (w.el(a).angle - q).abs() < 1e-6,
        "o de dentro rodou com ele"
    );
    let c = w.el(a).center();
    assert!(
        (c[0] - 200.0).abs() < 1e-6 && (c[1] - 240.0).abs() < 1e-6,
        "{c:?}"
    );
    assert!(w.el(f).box_contains(c), "a caixa rodada contém-no");
    assert_eq!(w.doc.deepest_frame_at(c, &BTreeSet::new()), Some(f));
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(a).angle, 0.0, "UM passo");
}

/// ⭐ Apagar um frame leva tudo o que vive dentro dele (a lei da W5, o Miro), UM passo — menos o
/// travado, que fica solto onde está.
#[test]
fn deleting_a_frame_takes_its_contents_but_not_the_locked() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [600.0, 600.0]);
    let a = w.rect([100.0, 100.0, 40.0, 40.0]);
    let l = w.rect([200.0, 100.0, 40.0, 40.0]);
    w.ed.select(&w.doc, [l]);
    assert!(w.cmd(Command::Lock));
    w.ed.select(&w.doc, [f]);
    assert!(w.cmd(Command::Delete));
    assert!(w.doc.get(f).is_none() && w.doc.get(a).is_none());
    assert!(w.doc.get(l).is_some(), "o travado fica");
    assert!(w.doc.frame_chain(l).is_empty(), "solto");
    assert!(w.cmd(Command::Undo));
    assert!(w.doc.get(f).is_some() && w.doc.get(a).is_some());
}

/// Oráculo A8: encolher um frame solta o filho que fica INTEIRO fora e guarda o que atravessa.
#[test]
fn shrinking_a_frame_releases_only_what_ends_fully_outside() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [400.0, 400.0]);
    let far = w.rect([300.0, 300.0, 50.0, 50.0]);
    let cross = w.rect([150.0, 150.0, 100.0, 50.0]);
    w.ed.select(&w.doc, [f]);
    w.drag([400.0, 400.0], [200.0, 200.0], CTRL);
    assert!(near(&[w.el(f).w, w.el(f).h], &[200.0, 200.0]));
    assert_eq!(w.el(far).frame, None);
    assert_eq!(w.el(cross).frame, Some(f));
    assert!(
        near(&[w.el(far).x], &[300.0]),
        "os filhos não escalam com o frame"
    );
}

/// Copiar um frame copia o conteúdo: as cópias vivem no frame COPIADO, em grupos NOVOS.
#[test]
fn duplicating_a_frame_copies_its_contents_into_the_copy() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [600.0, 600.0]);
    let a = w.rect([100.0, 100.0, 40.0, 40.0]);
    let b = w.rect([200.0, 100.0, 40.0, 40.0]);
    w.ed.select(&w.doc, [a, b]);
    assert!(w.cmd(Command::Group));
    let g = w.el(a).groups()[0];
    w.ed.select(&w.doc, [f]);
    assert!(w.cmd(Command::Duplicate));
    let copy = *w.ed.selection().iter().next().unwrap();
    assert_ne!(copy, f);
    let inside = w.doc.descendants(copy);
    assert_eq!(inside.len(), 2, "as duas cópias vivem na cópia do frame");
    for id in inside {
        assert_eq!(w.el(id).groups().len(), 1);
        assert_ne!(w.el(id).groups()[0], g, "um grupo novo");
    }
}

fn note(w: &mut World, x: f64) -> ElementId {
    let fill = ph2d_board_model::Rgba(ph2d_board_model::STICKY_COLORS[1]);
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(fill), None, fill.readable_ink()),
        text: Default::default(),
    };
    let el = Element::new_shape(
        w.doc.mint_id(),
        w.doc.z_on_top(),
        shape,
        [x, 0.0, 199.0, 199.0],
    );
    let id = el.id;
    BoardOp::Put(el).apply(&mut w.doc);
    id
}

/// ⭐ A etiqueta: entra nas notas que não a têm; se todas a têm, sai de todas; só notas; até 8 (o
/// Miro), com o aviso — cada vez UM passo.
#[test]
fn a_tag_goes_on_the_notes_comes_off_when_all_have_it_and_stops_at_eight() {
    let mut w = world();
    let (a, b) = (note(&mut w, 0.0), note(&mut w, 300.0));
    let r = put_rect(&mut w, [600.0, 0.0, 50.0, 50.0]);
    let red = ph2d_board_model::Rgba([200, 0, 0, 255]);
    w.ed.select(&w.doc, [a]);
    assert!(w.ed.toggle_tag(&mut w.doc, &mut w.h, "x", red));
    w.ed.select(&w.doc, [a, b, r]);
    assert!(w.ed.toggle_tag(&mut w.doc, &mut w.h, "x", red));
    assert_eq!(w.el(b).tags().len(), 1, "entrou na que não a tinha");
    assert!(w.el(r).tags().is_empty(), "uma forma não leva etiquetas");
    assert!(w.ed.toggle_tag(&mut w.doc, &mut w.h, "x", red));
    assert!(
        w.el(a).tags().is_empty() && w.el(b).tags().is_empty(),
        "todas tinham: saiu"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(a).tags().len(), 1, "UM passo");
    w.ed.select(&w.doc, [a]);
    for k in 0..10 {
        w.ed.toggle_tag(&mut w.doc, &mut w.h, &format!("t{k}"), red);
    }
    assert_eq!(w.el(a).tags().len(), ph2d_board_model::MAX_TAGS);
    assert_eq!(w.ed.take_notice(), Some(Notice::TooManyTags));
}

/// ⭐ O link (o «Link to» do Miro): `Ctrl+Alt+K` arma, o clique no destino liga (UM passo); no
/// vazio o modo fecha sem mudar nada; de novo com o link posto, tira-o; destino apagado = aviso.
#[test]
fn a_link_is_armed_then_made_by_a_click_and_a_gone_target_says_so() {
    let mut w = world();
    let a = put_rect(&mut w, [0.0, 0.0, 50.0, 50.0]);
    let b = put_rect(&mut w, [300.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [a]);
    assert!(w.cmd(Command::Link));
    assert!(w.ed.is_linking());
    assert_eq!(w.ed.take_notice(), Some(Notice::LinkPick));
    w.click([-900.0, -900.0], NONE);
    assert!(
        !w.ed.is_linking() && w.el(a).link().is_none(),
        "o vazio fecha sem ligar"
    );
    w.ed.select(&w.doc, [a]);
    w.cmd(Command::Link);
    w.click([325.0, 25.0], NONE);
    assert_eq!(w.el(a).link(), Some(b));
    assert_eq!(w.ed.follow_link(&w.doc, a), Some(b));
    assert_eq!(
        *w.ed.selection(),
        BTreeSet::from([b]),
        "seguir selecciona o destino"
    );
    w.ed.select(&w.doc, [a]);
    assert!(
        w.cmd(Command::Link),
        "com o link posto, o mesmo atalho tira-o"
    );
    assert!(w.el(a).link().is_none());
    assert!(w.cmd(Command::Undo));
    BoardOp::Delete(b).apply(&mut w.doc);
    assert_eq!(w.ed.follow_link(&w.doc, a), None);
    assert_eq!(w.ed.take_notice(), Some(Notice::LinkGone));
    w.ed.select(&w.doc, [a]);
    w.cmd(Command::Link);
    assert!(
        w.cmd(Command::Escape) && !w.ed.is_linking(),
        "Esc fecha o modo"
    );
}

/// Travado não se EDITA (o Miro): um estilo, a ordem ou uma etiqueta sobre uma selecção com um
/// travado mudam só os outros.
#[test]
fn a_style_on_a_selection_skips_the_locked() {
    let mut w = world();
    let l = put_rect(&mut w, [0.0, 0.0, 50.0, 50.0]);
    let f = put_rect(&mut w, [100.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [l]);
    w.cmd(Command::Lock);
    w.ed.select(&w.doc, [l, f]);
    let red = Some(ph2d_board_model::Rgba([200, 0, 0, 255]));
    w.ed.set_style(&mut w.doc, &mut w.h, |s| s.fill = red);
    assert_eq!(w.el(f).style().fill, red);
    assert_ne!(w.el(l).style().fill, red, "o travado mudou de cor");
    let z = w.el(l).z.clone();
    w.cmd(Command::Order(Order::Back));
    assert_eq!(w.el(l).z, z, "o travado mudou de ordem");
}

/// ⭐ Largar no MESMO sítio mas noutro frame (o ponteiro decide a casa) é UM passo de desfazer: a
/// pose não mudou, a casa sim.
#[test]
fn dropping_in_place_but_in_another_frame_is_undoable() {
    let mut w = world();
    let f = frame(&mut w, [0.0, 0.0], [300.0, 300.0]);
    let r = put_rect(&mut w, [250.0, 100.0, 100.0, 50.0]);
    let mut el = w.el(r).clone();
    el.frame = Some(f);
    BoardOp::Put(el).apply(&mut w.doc);
    w.click([-900.0, -900.0], NONE);
    // Agarrado pela ponta de FORA do frame, levado longe e trazido de volta ao mesmo ponto.
    let p = at(330.0, 125.0, CTRL);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, p, VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(600.0, 400.0, CTRL));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, p);
    w.ed.pointer_up(&mut w.doc, &mut w.h, p);
    assert_eq!(w.el(r).x, 250.0, "voltou ao mesmo sítio");
    assert_eq!(w.el(r).frame, None, "o ponteiro largou-o fora do frame");
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(r).frame, Some(f), "o desfazer não devolveu a casa");
}
