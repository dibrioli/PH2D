//! As leis do gesto: as tabelas do Trial (T4, G4) lidas da fixtura, e o que nós decidimos, por
//! propriedade, em caixas não quadradas que separam a esquerda da direita e `w` de `h`.

use ph2d_pixel_core::slice::{create, set_shape};
use ph2d_pixel_core::{NewSprite, Rect, SliceId, SliceShape, Sprite, apply};
use ph2d_pixel_slices::{Gesto, Grab, Lados, aplica, hit, marcas};

const TOL: f64 = 0.5;

fn fixtura() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../ph2d-pixel-io/tests/fixtures/oracle_slices_ferramenta.json"
    ))
    .expect("fixtura")
}

fn rect(v: &serde_json::Value) -> Rect {
    let n = |i: usize| v[i].as_i64().expect("número");
    Rect::new(n(0) as i32, n(1) as i32, n(2) as u32, n(3) as u32)
}

fn par(v: &serde_json::Value) -> (i32, i32) {
    (v[0].as_i64().unwrap() as i32, v[1].as_i64().unwrap() as i32)
}

/// Um sprite 64×64 com uma slice no quadro 0.
fn com_uma(shape: SliceShape) -> (Sprite, SliceId) {
    let mut s = Sprite::new(NewSprite {
        width: 64,
        height: 64,
        ..NewSprite::default()
    })
    .expect("sprite");
    let id = nova(&mut s, shape);
    (s, id)
}

fn nova(s: &mut Sprite, shape: SliceShape) -> SliceId {
    let (op, id) = create(s, 0, Rect::new(0, 0, 1, 1)).expect("cria");
    apply(s, op).expect("aplica");
    let op = set_shape(s, id, 0, shape).expect("muda");
    apply(s, op).expect("aplica");
    id
}

/// Premir no meio do píxel `a`, largar no meio do píxel `b`, com a seleção `sel`.
fn arrasta(s: &mut Sprite, sel: &mut Vec<SliceId>, a: (i32, i32), b: (i32, i32)) {
    let meio = |p: (i32, i32)| (f64::from(p.0) + 0.5, f64::from(p.1) + 0.5);
    let mut g = Gesto::press(s, 0, sel, meio(a), TOL, false);
    g.drag(meio(b));
    if let Some(op) = g.release(s, 0, sel) {
        apply(s, op).expect("aplica");
    }
}

fn forma(s: &Sprite, id: SliceId) -> SliceShape {
    s.slice(id).unwrap().shape_at(0).unwrap()
}

/// **T4**: a tabela inteira do Trial, cada linha com o gesto escrito aqui. A descrição é conferida para
/// o gesto não se desencontrar da linha.
#[test]
fn the_nine_slice_table_of_the_trial_comes_out_cell_by_cell() {
    let f = fixtura();
    let linhas = f["t4_nine_drag_full_nine"].as_array().unwrap();
    let inicio = &f["inputs"]["full"]["nine"];
    let gestos = [
        ("linha esquerda x=12 -> 16", (12, 16), (16, 16)),
        ("linha esquerda x=12 -> 24", (12, 16), (24, 16)),
        ("linha de cima y=12 -> 14", (16, 12), (16, 14)),
        ("linha direita x=20 -> 18", (20, 16), (18, 16)),
        ("linha de baixo y=20 -> 22", (16, 20), (16, 22)),
        ("canto (12,12) -> (14,14)", (12, 12), (14, 14)),
        ("canto (20,20) -> (22,22)", (20, 20), (22, 22)),
        ("interior (16,16) -> (18,18)", (16, 16), (18, 18)),
        ("banda (8,8) -> (10,10)", (8, 8), (10, 10)),
        ("borda esquerda -2", (4, 16), (2, 16)),
        ("canto sup-esq -2,-2", (4, 4), (2, 2)),
        ("canto inf-dir +4,+4", (27, 27), (31, 31)),
        ("borda direita -2", (27, 16), (25, 16)),
        ("borda direita ate w=17", (27, 16), (20, 16)),
        ("borda direita ate w=16", (27, 16), (19, 16)),
        ("borda direita ate w=11", (27, 16), (14, 16)),
        ("borda esquerda +2", (4, 16), (6, 16)),
        ("borda esquerda ate w=17", (4, 16), (11, 16)),
        ("borda esquerda ate w=16", (4, 16), (12, 16)),
        ("canto inf-dir para alem do oposto", (27, 27), (2, 2)),
    ];
    assert_eq!(linhas.len(), gestos.len());
    for (linha, (nome, a, b)) in linhas.iter().zip(gestos) {
        let drag = linha["drag"].as_str().unwrap();
        assert!(drag.contains(nome), "«{drag}» × «{nome}»");
        let (mut s, id) = com_uma(SliceShape {
            bounds: rect(&inicio["bounds"]),
            center: Some(rect(&inicio["center"])),
            pivot: None,
        });
        arrasta(&mut s, &mut vec![id], a, b);
        let k = forma(&s, id);
        assert_eq!(k.bounds, rect(&linha["bounds"]), "{drag}");
        let quer = (!linha["center"].is_null()).then(|| rect(&linha["center"]));
        assert_eq!(k.center, quer, "{drag}");
    }
}

/// **G4**: o pivot não muda ao redimensionar, mesmo ficando fora; puxar pela esquerda anda com ele.
#[test]
fn resizing_never_changes_the_pivot_even_when_it_ends_up_outside() {
    let f = fixtura();
    let g4 = &f["rodada2"]["g4_redimensionar"];
    for (nome, a, b) in [
        ("direita_w8", (31, 16), (15, 16)),
        ("esquerda_4", (8, 16), (12, 16)),
        ("esquerda_16", (8, 16), (24, 16)),
        ("cima_16", (16, 8), (16, 24)),
        ("canto_sup_esq_4", (8, 8), (12, 12)),
    ] {
        let (mut s, id) = com_uma(SliceShape {
            bounds: Rect::new(8, 8, 24, 24),
            center: None,
            pivot: Some((12, 12)),
        });
        arrasta(&mut s, &mut vec![id], a, b);
        let k = forma(&s, id);
        assert_eq!(k.bounds, rect(&g4[nome]["bounds"]), "{nome}");
        assert_eq!(k.pivot, Some(par(&g4[nome]["pivot"])), "{nome}");
    }
}

/// G4, a key herdada: mexer no quadro 3 cria key SÓ no 3; os outros quadros ficam.
#[test]
fn editing_an_inherited_frame_creates_a_key_only_there() {
    let f = fixtura();
    let t = &f["rodada2"]["g4_redimensionar"]["tecla_herdada_q3_direita_menos_4"];
    let shape = |q: &str| SliceShape {
        bounds: rect(&t[q]["bounds"]),
        center: None,
        pivot: Some(par(&t[q]["pivot"])),
    };
    let (mut s, id) = com_uma(shape("0"));
    for (q, frame) in [("2", 2), ("4", 4)] {
        let op = set_shape(&s, id, frame, shape(q)).unwrap();
        apply(&mut s, op).unwrap();
    }
    let mut sel = vec![id];
    let mut g = Gesto::press(&s, 3, &mut sel, (53.5, 40.5), TOL, false);
    g.drag((49.5, 40.5));
    let op = g.release(&mut s, 3, &mut sel).expect("muda");
    apply(&mut s, op).unwrap();
    let c = s.slice(id).unwrap();
    assert_eq!(
        c.keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
        [0, 2, 3, 4]
    );
    assert_eq!(c.shape_at(3), Some(shape("3")));
    for (q, frame) in [("0", 0), ("2", 2), ("4", 4), ("5", 5)] {
        assert_eq!(c.shape_at(frame), Some(shape(q)), "quadro {frame}");
    }
}

/// As 8 alças numa caixa não quadrada: cada lado mexe só nos seus números, e travam em 1.
#[test]
fn each_of_the_eight_handles_moves_only_its_own_sides_on_a_non_square_box() {
    let b = Rect::new(10, 20, 30, 12);
    let s = SliceShape {
        bounds: b,
        center: None,
        pivot: None,
    };
    let (l, r, t, d) = (
        Lados {
            left: true,
            ..Lados::default()
        },
        Lados {
            right: true,
            ..Lados::default()
        },
        Lados {
            top: true,
            ..Lados::default()
        },
        Lados {
            bottom: true,
            ..Lados::default()
        },
    );
    let junta = |a: Lados, z: Lados| Lados {
        left: a.left || z.left,
        right: a.right || z.right,
        top: a.top || z.top,
        bottom: a.bottom || z.bottom,
    };
    for (lados, quer, quer_longe) in [
        (l, Rect::new(13, 20, 27, 12), Rect::new(39, 20, 1, 12)),
        (r, Rect::new(10, 20, 33, 12), Rect::new(10, 20, 33, 12)),
        (t, Rect::new(10, 25, 30, 7), Rect::new(10, 25, 30, 7)),
        (d, Rect::new(10, 20, 30, 17), Rect::new(10, 20, 30, 17)),
        (
            junta(l, t),
            Rect::new(13, 25, 27, 7),
            Rect::new(39, 25, 1, 7),
        ),
        (
            junta(r, t),
            Rect::new(10, 25, 33, 7),
            Rect::new(10, 25, 33, 7),
        ),
        (
            junta(l, d),
            Rect::new(13, 20, 27, 17),
            Rect::new(39, 20, 1, 17),
        ),
        (
            junta(r, d),
            Rect::new(10, 20, 33, 17),
            Rect::new(10, 20, 33, 17),
        ),
    ] {
        assert_eq!(
            aplica(s, Grab::Bounds(lados), 3, 5).bounds,
            quer,
            "{lados:?}"
        );
        assert_eq!(
            aplica(s, Grab::Bounds(lados), if lados.left { 90 } else { 3 }, 5).bounds,
            quer_longe,
            "{lados:?} longe"
        );
    }
    // À esquerda e para baixo: a direita e o fundo travam em 1 (T2).
    assert_eq!(
        aplica(s, Grab::Bounds(junta(r, d)), -90, -90).bounds,
        Rect::new(10, 20, 1, 1)
    );
    // Passar a oposta por cima trava na oposta.
    assert_eq!(
        aplica(s, Grab::Bounds(t), 0, 90).bounds,
        Rect::new(10, 31, 30, 1)
    );
}

/// O centro: `w` e `h` separados, numa caixa não quadrada, a largura a desligar só por largura.
#[test]
fn the_center_absorbs_width_and_height_separately_and_turns_off_when_it_no_longer_fits() {
    let s = SliceShape {
        bounds: Rect::new(10, 20, 30, 12),
        center: Some(Rect::new(3, 2, 9, 4)),
        pivot: None,
    };
    let r = Grab::Bounds(Lados {
        right: true,
        ..Lados::default()
    });
    let d = Grab::Bounds(Lados {
        bottom: true,
        ..Lados::default()
    });
    assert_eq!(aplica(s, r, -8, 0).center, Some(Rect::new(3, 2, 1, 4)));
    assert_eq!(aplica(s, r, -9, 0).center, None);
    assert_eq!(aplica(s, d, 0, -3).center, Some(Rect::new(3, 2, 9, 1)));
    assert_eq!(aplica(s, d, 0, -4).center, None);
    // Uma linha do centro não sai da caixa nem mexe na caixa.
    let cl = Grab::Center(Lados {
        left: true,
        ..Lados::default()
    });
    let cr = Grab::Center(Lados {
        right: true,
        ..Lados::default()
    });
    assert_eq!(aplica(s, cl, -90, 0).center, Some(Rect::new(0, 2, 12, 4)));
    assert_eq!(aplica(s, cr, 90, 0).center, Some(Rect::new(3, 2, 27, 4)));
    assert_eq!(aplica(s, cr, 90, 0).bounds, s.bounds);
}

/// ⭐ Além do Aseprite: arrastar o pivot mexe SÓ no pivot, e ele pode sair da caixa.
#[test]
fn dragging_the_pivot_moves_only_the_pivot_and_it_may_leave_the_box() {
    let shape = SliceShape {
        bounds: Rect::new(36, 4, 24, 16),
        center: Some(Rect::new(4, 4, 8, 8)),
        pivot: Some((12, 8)),
    };
    let (mut s, id) = com_uma(shape);
    arrasta(&mut s, &mut vec![id], (48, 12), (48, 40));
    let k = forma(&s, id);
    assert_eq!(k.pivot, Some((12, 36)), "saiu da caixa por baixo");
    assert_eq!(k.bounds, shape.bounds);
    assert_eq!(k.center, shape.center);
}

/// A prioridade numa selecionada: pivot, centro, caixa, interior. O pivot fora da caixa agarra-se.
#[test]
fn the_hit_priority_is_pivot_then_center_then_bounds_then_interior() {
    let mut s = SliceShape {
        bounds: Rect::new(10, 10, 40, 20),
        center: Some(Rect::new(10, 5, 20, 10)),
        pivot: Some((10, 5)),
    };
    // O pivot cai no canto do centro: ele ganha.
    assert_eq!(hit(&s, (20.5, 15.5), TOL), Some(Grab::Pivot));
    s.pivot = Some((-5, 5));
    assert_eq!(
        hit(&s, (5.5, 15.5), TOL),
        Some(Grab::Pivot),
        "fora da caixa"
    );
    assert!(matches!(hit(&s, (20.5, 15.5), TOL), Some(Grab::Center(_))));
    assert!(matches!(hit(&s, (10.5, 20.5), TOL), Some(Grab::Bounds(_))));
    assert_eq!(hit(&s, (15.5, 12.5), TOL), Some(Grab::Move), "a banda move");
    assert_eq!(hit(&s, (30.5, 20.5), TOL), Some(Grab::Move), "o miolo move");
    assert_eq!(hit(&s, (70.5, 20.5), TOL), None);
}

/// A qualquer zoom: o pivot agarra-se a zoom baixo e o miolo da caixa continua a mover.
#[test]
fn at_low_zoom_the_pivot_is_grabbable_and_the_middle_of_any_box_still_moves() {
    let tol = 6.0;
    let s = SliceShape {
        bounds: Rect::new(10, 10, 24, 16),
        center: None,
        pivot: Some((30, 30)),
    };
    assert_eq!(hit(&s, (44.0, 44.0), tol), Some(Grab::Pivot), "a 5 células");
    for (w, h) in [(1, 1), (2, 3), (5, 2), (24, 16)] {
        let k = SliceShape {
            bounds: Rect::new(10, 10, w, h),
            center: None,
            pivot: None,
        };
        let meio = (10.0 + f64::from(w) / 2.0, 10.0 + f64::from(h) / 2.0);
        assert_eq!(hit(&k, meio, tol), Some(Grab::Move), "{w}×{h}");
    }
}

/// T1: criar recorta e normaliza; a nova não fica selecionada; o clique não cria e limpa.
#[test]
fn creating_clips_normalizes_and_a_click_creates_nothing_and_clears_the_selection() {
    let (mut s, a) = com_uma(SliceShape {
        bounds: Rect::new(0, 0, 4, 4),
        ..SliceShape::default()
    });
    let mut sel = vec![a];
    // Da direita para a esquerda, por baixo do canvas 64×64: o medido era (50,40)→(70,70).
    arrasta(&mut s, &mut sel, (70, 70), (50, 40));
    assert_eq!(s.slices().len(), 2);
    let nova = s.slices()[1].shape_at(0).unwrap().bounds;
    assert_eq!(nova, Rect::new(50, 40, 14, 24));
    assert_eq!(s.slices()[1].name, "Slice 2");
    assert!(sel.is_empty(), "a nova nasce sem seleção");
    sel.push(a);
    arrasta(&mut s, &mut sel, (30, 30), (30, 30));
    assert_eq!(s.slices().len(), 2, "o clique não cria");
    assert!(sel.is_empty(), "o clique em vazio limpa");
    // T1: 8 → 30 dá 22, o píxel de chegada não conta.
    arrasta(&mut s, &mut sel, (8, 8), (30, 30));
    assert_eq!(
        s.slices()[2].shape_at(0).unwrap().bounds,
        Rect::new(8, 8, 22, 22)
    );
}

/// T1 e T2: numa não selecionada, premir seleciona e move; o clique substitui; a marquise toca.
#[test]
fn pressing_an_unselected_slice_moves_it_and_shift_drag_selects_what_it_touches() {
    let (mut s, a) = com_uma(SliceShape {
        bounds: Rect::new(8, 8, 8, 8),
        ..SliceShape::default()
    });
    let b = nova(
        &mut s,
        SliceShape {
            bounds: Rect::new(30, 8, 8, 8),
            ..SliceShape::default()
        },
    );
    let mut sel = vec![b];
    arrasta(&mut s, &mut sel, (11, 11), (15, 15));
    assert_eq!(forma(&s, a).bounds, Rect::new(12, 12, 8, 8));
    assert_eq!(forma(&s, b).bounds, Rect::new(30, 8, 8, 8));
    assert_eq!(sel, [a]);

    // A marquise que só toca a coluna da borda de `b` e não toca `a`.
    let mut g = Gesto::press(&s, 0, &mut sel, (37.5, 30.5), TOL, true);
    g.drag((50.5, 15.5));
    assert!(g.release(&mut s, 0, &mut sel).is_none());
    assert_eq!(sel, [b]);
    // `Shift` sobre uma slice também é a marquise (T4): não a move.
    let antes = forma(&s, b);
    let mut g = Gesto::press(&s, 0, &mut sel, (33.5, 11.5), TOL, true);
    g.drag((35.5, 13.5));
    assert!(g.release(&mut s, 0, &mut sel).is_none());
    assert_eq!(forma(&s, b), antes);
}

/// G3 e §10: várias selecionadas andam juntas e levam a mesma borda, num só passo; clicar numa
/// deixa só ela; um arrasto que volta ao sítio não dá passo nenhum.
#[test]
fn several_selected_move_and_resize_together_in_one_step_and_a_return_trip_gives_no_step() {
    let (mut s, a) = com_uma(SliceShape {
        bounds: Rect::new(8, 8, 8, 8),
        ..SliceShape::default()
    });
    let b = nova(
        &mut s,
        SliceShape {
            bounds: Rect::new(30, 8, 12, 6),
            ..SliceShape::default()
        },
    );
    let mut sel = vec![a, b];
    let meio = (11.5, 11.5);
    let mut g = Gesto::press(&s, 0, &mut sel, meio, TOL, false);
    g.drag((15.5, 13.5));
    assert_eq!(forma(&s, a).bounds.x, 8, "o documento não muda a meio");
    assert_eq!(g.previa().len(), 2);
    let op = g.release(&mut s, 0, &mut sel).expect("muda");
    apply(&mut s, op).unwrap();
    assert_eq!(forma(&s, a).bounds, Rect::new(12, 10, 8, 8));
    assert_eq!(forma(&s, b).bounds, Rect::new(34, 10, 12, 6));
    assert_eq!(sel, [a, b]);

    // A borda direita de `a`, -3: as duas perdem 3 de largura.
    let mut g = Gesto::press(&s, 0, &mut sel, (19.5, 14.5), TOL, false);
    g.drag((16.5, 14.5));
    let op = g.release(&mut s, 0, &mut sel).expect("muda");
    apply(&mut s, op).unwrap();
    assert_eq!(forma(&s, a).bounds, Rect::new(12, 10, 5, 8));
    assert_eq!(forma(&s, b).bounds, Rect::new(34, 10, 9, 6));

    // Ida e volta: nenhum passo, e a seleção não vira clique.
    let mut g = Gesto::press(&s, 0, &mut sel, (14.5, 14.5), TOL, false);
    g.drag((20.5, 20.5));
    g.drag((14.5, 14.5));
    assert!(g.release(&mut s, 0, &mut sel).is_none());
    assert_eq!(sel, [a, b]);

    // Clique sem arrasto numa das duas: fica só ela (T2).
    let g = Gesto::press(&s, 0, &mut sel, (14.5, 14.5), TOL, false);
    assert!(g.release(&mut s, 0, &mut sel).is_none());
    assert_eq!(sel, [a]);
}

/// O que se desenha: a prévia em vez do documento, sem caixa não aparece, sem cor no ficheiro a padrão.
#[test]
fn the_marks_show_the_preview_hide_the_empty_box_and_default_the_color() {
    let (mut s, a) = com_uma(SliceShape {
        bounds: Rect::new(8, 8, 8, 8),
        ..SliceShape::default()
    });
    let b = nova(
        &mut s,
        SliceShape {
            bounds: Rect::new(30, 8, 8, 4),
            ..SliceShape::default()
        },
    );
    let mut sem_cor = s.slices().to_vec();
    sem_cor[1].color = None;
    apply(&mut s, ph2d_pixel_core::Op::Slices(sem_cor)).unwrap();
    let mut sel = vec![a];
    let mut g = Gesto::press(&s, 0, &mut sel, (10.5, 10.5), TOL, false);
    g.drag((12.5, 10.5));
    let m = marcas(&s, 0, Some(&g), &sel);
    assert_eq!(m[0].shape.bounds.x, 10, "a prévia");
    assert!(m[0].selected && !m[1].selected);
    assert_eq!(m[1].color, ph2d_pixel_core::DEFAULT_SLICE_COLOR);
    let op = set_shape(&s, b, 2, SliceShape::default()).unwrap();
    apply(&mut s, op).unwrap();
    let ids: Vec<_> = marcas(&s, 2, None, &sel).iter().map(|k| k.id).collect();
    assert_eq!(ids, [a], "caixa vazia esconde até à seguinte");
}
