//! ⭐ **O Pivot da Animated Sprite no Align** (GEF 29, pedido do dono 09/11): a grade de 25 pontos conta sobre o
//! desenho de TODOS os quadros de TODAS as animações; o «By hand» arrasta a cruz; cada escolha é UMA edição (um
//! Ctrl+Z) e o desenho de cada folha fica no mundo.

use super::super::{Mesa, centre};
use super::desalinhada;
use ph2d_app_sprite_anim::editor::align_pivot::{SIDE, grid_point, point_name};
use ph2d_ecs::{Entity, Transform};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::{AnimFieldEdit, ImageUseEdit, row_cells};
use ph2d_host::Modifiers;

/// O centro no mundo do quad de cada folha (sem rotação nem escala: a posição mais a âncora).
fn quads_no_mundo(m: &Mesa) -> Vec<[f32; 2]> {
    let w = m.sim.world();
    let e = Entity::from_bits(m.e);
    let t = w.get::<Transform>(e).expect("o herói").translation;
    let ppm = m.hero.project.pixels_per_meter;
    (0..2_u8)
        .filter_map(|k| ph2d_render::sheet_geometry(w, e, k, 0))
        .map(|(s, _)| {
            let a = s.resolve_anchor(ppm);
            [t.x + a[0], t.y + a[1]]
        })
        .collect()
}

/// O ecrã do ponto `q` da grade da moldura.
fn na_grade(m: &mut Mesa, q: [u8; 2]) -> (f32, f32) {
    let g = m.doc().layout.align.pivot_grid;
    assert!(g.w > 0.0 && g.h > 0.0, "a grade do Pivot na moldura");
    let passo = g.w / f32::from(SIDE);
    (
        g.x + (f32::from(q[0]) + 0.5) * passo, // LITERAL-PX-OK: o meio da casa
        g.y + (f32::from(q[1]) + 0.5) * passo, // LITERAL-PX-OK: o meio da casa
    )
}

/// ⭐ Os 25 pontos contam sobre todos os quadros de todas as animações: a base da caixa é o pé mais baixo de TODAS.
/// Um clique em «embaixo, meio» é uma edição só; depois dela o Pivot está nesse ponto, as duas folhas ficam no mundo,
/// e trocar de animação não muda o ponto.
/// *Mutações: a caixa só da animação aberta ⇒ a base sobe · o ponto contado de baixo para cima ⇒ cai em cima ·
/// `move_origin` só na folha visível ⇒ a folha 0 sai do lugar · a prévia sem seguir a geometria ⇒ o ponto não acende.*
#[test]
fn os_25_pontos_contam_sobre_todos_os_quadros_e_o_desenho_fica() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    m.align_on();
    m.paint();
    let caixa = m
        .doc()
        .all_frames_box()
        .expect("o desenho de todos os quadros");
    let info = m.doc().snap.info.clone().expect("a biblioteca");
    assert!(info.rows.len() >= 2, "mais de uma animação na fixtura");
    let mut mais_baixo = f32::INFINITY;
    for row in &info.rows {
        for f in row_cells(row, &info.sheets) {
            let Some(p) = m
                .doc()
                .align_point(f, ph2d_app_sprite_anim::editor::align::AlignBy::Feet)
            else {
                continue;
            };
            assert!(
                p[0] >= caixa.0[0] - 1e-5 && p[0] <= caixa.1[0] + 1e-5,
                "{f:?} dentro"
            );
            mais_baixo = mais_baixo.min(p[1]);
        }
    }
    assert!(
        (mais_baixo - caixa.0[1]).abs() < 1e-5,
        "a base é o pé mais baixo de todas"
    );
    let antes = quads_no_mundo(&m);
    assert_eq!(antes.len(), 2, "as duas folhas");
    let alvo = grid_point(caixa, [2, 4]);
    let at = na_grade(&mut m, [2, 4]);
    m.click(at);
    assert_eq!(
        m.sheet_edits(),
        [ImageUseEdit::MovePivot(alvo)],
        "uma edição: um Ctrl+Z"
    );
    assert_eq!(m.doc().pivot_on_grid(), Some([2, 4]), "a prévia segue já");
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([2, 4]), "o mundo concorda");
    for (a, d) in antes.iter().zip(quads_no_mundo(&m)) {
        assert!(
            (a[0] - d[0]).abs() < 1e-5 && (a[1] - d[1]).abs() < 1e-5,
            "{a:?} → {d:?}"
        );
    }
    let outra = (m.doc().selected + 1) % info.rows.len();
    m.doc().selected = outra;
    m.paint();
    assert_eq!(
        m.doc().pivot_on_grid(),
        Some([2, 4]),
        "noutra animação, o mesmo ponto"
    );
    assert_eq!(point_name(Some([2, 4])), "Bottom");
    assert_eq!(point_name(Some([1, 4])), "25% \u{b7} 100%");
    assert_eq!(point_name(None), "Custom");
}

/// O «By hand»: o arrasto na prévia move a CRUZ (a folha não anda) e só o largar pede a edição; perto de um dos 25
/// pontos encaixa nele; longe, cai num pixel inteiro do desenho. Desligado, o arrasto volta a mover a folha.
/// *Mutações: a edição a cada movimento ⇒ várias, vários Ctrl+Z · sem encaixe ⇒ o ponto não acende · sem o pixel
/// inteiro ⇒ o resto não é múltiplo.*
#[test]
fn by_hand_arrasta_a_cruz_e_encaixa_nos_pontos() {
    let mut m = Mesa::new();
    desalinhada(&mut m);
    m.align_on();
    m.paint();
    let b = m.doc().layout.align.by_hand;
    assert!(b.w > 0.0, "o botão By hand na moldura");
    m.click(centre(b));
    assert!(m.doc().align.pivot.by_hand);
    m.paint();
    let caixa = m.doc().all_frames_box().unwrap();
    let v = m.doc().align.view.expect("a prévia");
    let cruz = m.doc().align.painted.pivot.expect("a cruz");
    let destino = v.to_screen(grid_point(caixa, [4, 0]));
    let p = ph2d_host::PointerButton::Primary;
    m.pointer_with(ph2d_host::PointerKind::Down, p, cruz, Modifiers::default());
    m.pointer_with(
        ph2d_host::PointerKind::Move,
        p,
        destino,
        Modifiers::default(),
    );
    assert!(m.sheet_edits().is_empty(), "nada antes de largar");
    m.paint();
    let pintada = m.doc().align.painted.pivot.unwrap();
    assert!(
        (pintada.0 - destino.0).abs() < 0.5 && (pintada.1 - destino.1).abs() < 0.5,
        "a cruz segue o rato"
    );
    m.pointer_with(
        ph2d_host::PointerKind::Up,
        p,
        (destino.0 + 2.0, destino.1 - 1.0),
        Modifiers::default(),
    );
    assert_eq!(
        m.sheet_edits(),
        [ImageUseEdit::MovePivot(grid_point(caixa, [4, 0]))],
        "perto do ponto, encaixa; uma edição"
    );
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), Some([4, 0]));
    // Longe dos pontos: um pixel inteiro do desenho.
    let caixa = m.doc().all_frames_box().unwrap();
    let v = m.doc().align.view.unwrap();
    let folha = m.doc().align_current().unwrap().1.sheet;
    let mpp = m.doc().meters_per_px(folha).unwrap();
    let meio = |a: f32, b: f32| a + (b - a) * 0.37; // LITERAL-PX-OK: um ponto qualquer entre os da grade
    let solto = v.to_screen([meio(caixa.0[0], caixa.1[0]), meio(caixa.0[1], caixa.1[1])]);
    m.drag(solto, solto, Modifiers::default());
    let [ImageUseEdit::MovePivot(q)] = m.sheet_edits()[..] else {
        panic!("uma edição: {:?}", m.sheet_edits());
    };
    for a in 0..2 {
        let n = (q[a] - caixa.0[a]) / mpp;
        assert!((n - n.round()).abs() < 1e-3, "pixel inteiro: {n}");
    }
    m.frame();
    m.paint();
    assert_eq!(m.doc().pivot_on_grid(), None, "fora dos 25: Custom");
    // Desligado, o arrasto volta a mover a folha.
    let b = m.doc().layout.align.by_hand;
    m.click(centre(b));
    let pv = m.doc().layout.align.preview;
    m.drag(
        centre(pv),
        (centre(pv).0 + 30.0, centre(pv).1),
        Modifiers::default(),
    );
    assert!(
        m.sheet_edits()
            .iter()
            .any(|e| matches!(e, ImageUseEdit::MoveSheet(_)))
            && !m
                .sheet_edits()
                .iter()
                .any(|e| matches!(e, ImageUseEdit::MovePivot(_))),
        "{:?}",
        m.sheet_edits()
    );
}

/// O campo Pivot do Inspector numa Animated Sprite abre o editor com o Align ligado; o «Edit animations» de sempre
/// abre-o sem.
/// *Mutação: o pedido do Align esquecido ⇒ abre desligado.*
#[test]
fn o_campo_pivot_abre_o_editor_com_o_align_ligado() {
    let mut m = Mesa::new();
    assert!(!m.doc().align.on);
    let e = m.e;
    m.docs().close(e);
    assert!(ph2d_app_sprite_anim::editor::request_open(
        &mut m.sim,
        e,
        &AnimFieldEdit::OpenEditorAlign
    ));
    m.frame();
    assert!(m.doc().align.on, "aberto com o Align");
}
