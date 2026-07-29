//! **The point handles draw where the anchors are, and are grabbable there.**
//!
//! Split out of `point.rs` when W-J2b turned the single dot into a list (LOC).

use super::*;
use crate::interaction::HitIndex;

fn view(handles: Vec<PointHandle>) -> PointGizmoView {
    PointGizmoView {
        handles,
        snap_world: None,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 1000.0,
        window_h: 1000.0,
        canvas: Rect::new(0.0, 0.0, 1000.0, 1000.0),
        inert: false,
    }
}

fn a(key: u64, world: [f32; 2]) -> PointHandle {
    PointHandle {
        key,
        kind: PointHandleKind::AnchorA,
        world,
    }
}

fn b(key: u64, world: [f32; 2]) -> PointHandle {
    PointHandle {
        key,
        kind: PointHandleKind::AnchorB,
        world,
    }
}

/// Paint into a fresh index + map + scene and hand all three back.
///
/// `n_paths` é o oráculo de *foi mesmo DESENHADO?* que este repo usa
/// (`progress/tests.rs`, `blend_overlay.rs`) — sem ele um gate de alça mede só
/// o registro de hit, e **desenhar e registrar são dois fatos**: o report do
/// Enio é *"não aparece"*, que é a metade que o `HitIndex` não conhece.
fn paint_counted(v: &PointGizmoView) -> (HitIndex, BTreeMap<NodeId, PointHandle>, u32) {
    let mut scene = VectorScene::new();
    let mut hits = HitIndex::default();
    let mut map = BTreeMap::new();
    paint_point_gizmo(&mut scene, v, Theme::default(), &mut hits, &mut map);
    let drawn = scene.inner().encoding().n_paths;
    (hits, map, drawn)
}

/// Paint into a fresh index + map and hand both back.
fn paint(v: &PointGizmoView) -> (HitIndex, BTreeMap<NodeId, PointHandle>) {
    let (hits, map, _) = paint_counted(v);
    (hits, map)
}

fn screen(v: &PointGizmoView, w: [f32; 2]) -> [f32; 2] {
    world_to_screen_px(
        v.camera_center,
        v.camera_height_world,
        v.window_w,
        v.window_h,
        w,
    )
}

/// **A Down on the dot's screen position hits its handle, and the map says
/// whose it is.**
///
/// The whole point of the gizmo: the anchor must be grabbable on the canvas.
/// The hit is registered at the anchor's PROJECTED position, so it tracks the
/// joint under pan/zoom the same way every other gizmo handle does.
///
/// Mutation-tested: dropping the `hit_index.register` call leaves nothing to
/// hit, and this goes red — the dot would paint but never be draggable.
#[test]
fn the_anchor_dot_is_hittable_where_it_is_drawn() {
    let v = view(vec![a(7, [2.0, 1.0])]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [2.0, 1.0]);
    let id = hits.hit(s[0], s[1]).expect(
        "a Down on the anchor's screen position did not hit the joint-anchor handle — the \
         pivot would be undraggable on the canvas",
    );
    assert_eq!(map.get(&id).copied(), Some(a(7, [2.0, 1.0])));
    // And a point far away misses it (the hit is a handle, not the whole canvas).
    assert_eq!(hits.hit(s[0] + 200.0, s[1] + 200.0), None);
}

/// **The B end is grabbable where it is drawn.** Without its own hit rect the
/// second anchor would paint and never take a drag.
///
/// Mutation-tested: painting only the A pass goes red.
#[test]
fn the_b_handle_is_hittable_where_it_is_drawn() {
    let v = view(vec![a(3, [0.0, 0.0]), b(3, [2.0, 1.0])]);
    let (hits, map) = paint(&v);

    let s = screen(&v, [2.0, 1.0]);
    let hit_b = hits
        .hit(s[0], s[1])
        .expect("the B anchor must be grabbable");
    assert_eq!(map[&hit_b].kind, PointHandleKind::AnchorB);

    let sa = screen(&v, [0.0, 0.0]);
    let hit_a = hits.hit(sa[0], sa[1]).expect("and A at its own position");
    assert_eq!(map[&hit_a].kind, PointHandleKind::AnchorA);
}

/// **A coincident pair is still two handles.** A Pin at rest anchors both
/// bodies at the same world point, so the two marks land on each other — A
/// takes the inner square and B the band outside it.
///
/// This is the gate that fails if the registration order is swapped (B last
/// would swallow A entirely, and the pivot would become undraggable on every
/// Pin and Weld in the scene — i.e. on the common case).
#[test]
fn a_coincident_pair_gives_a_the_centre_and_b_the_band() {
    let v = view(vec![a(11, [1.0, -1.0]), b(11, [1.0, -1.0])]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [1.0, -1.0]);

    let centre = hits.hit(s[0], s[1]).expect("dead centre belongs to A");
    assert_eq!(
        map[&centre].kind,
        PointHandleKind::AnchorA,
        "dead centre belongs to A"
    );

    let band = hits
        .hit(s[0] + JOINT_ANCHOR_RING_PX - 1.0, s[1])
        .expect("the band outside A's square must still reach B");
    assert_eq!(
        map[&band].kind,
        PointHandleKind::AnchorB,
        "the band outside A's square must reach B, or a Pin's B end could never be grabbed"
    );
}

/// **Two joints are four handles with four distinct ids, each answering for
/// itself.** This is what makes "every joint has handles" a feature rather than
/// a way to drag the wrong joint.
///
/// Mutation-tested: dropping `key` from [`point_handle_id`] (so all A handles
/// share one id) makes the map hold ONE entry per side and the first joint's
/// dot resolve to the second joint — the shell would author an anchor on a
/// joint the artist never touched.
#[test]
fn two_joints_are_four_handles_with_four_distinct_ids() {
    let v = view(vec![
        a(101, [-3.0, 0.0]),
        b(101, [-1.0, 0.0]),
        a(202, [1.0, 0.0]),
        b(202, [3.0, 0.0]),
    ]);
    let (hits, map) = paint(&v);
    assert_eq!(map.len(), 4, "four handles must own four distinct hit ids");

    for h in &v.handles {
        let s = screen(&v, h.world);
        let id = hits
            .hit(s[0], s[1])
            .expect("every published handle must be grabbable at its own position");
        assert_eq!(
            map.get(&id).copied(),
            Some(*h),
            "the hit at {:?} resolved to {:?}, not to the handle drawn there — a drag would \
             author the wrong joint's anchor",
            h.world,
            map.get(&id)
        );
    }
}

/// **No handles, nothing registered.** The empty list is never published (the
/// shell hands out `None`), but the painter must not invent a mark for it.
#[test]
fn an_empty_list_paints_nothing() {
    let (hits, map) = paint(&view(vec![]));
    assert_eq!(hits.hit(500.0, 500.0), None);
    assert!(map.is_empty());
}

/// **Every mark is at least as grabbable as it is visible.**
///
/// A dot drawn bigger than the rect that catches it is a dot the artist clicks
/// on and nothing happens — the failure mode of *making the marks bigger*
/// (Enio, 2026-07-25) if only the drawing half moves. Pinned per side, plus the
/// nesting that makes a coincident pair two handles.
///
/// Mutation-tested: leaving A's hit half at the old `HANDLE_SIZE_PX * 0.5` (6)
/// while the dot draws at 9 goes red here.
#[test]
fn the_hit_rects_are_never_smaller_than_the_marks() {
    assert!(
        hit_half_px(PointHandleKind::AnchorA) >= JOINT_ANCHOR_DOT_PX,
        "A's dot is drawn larger than its hit rect — visible and ungrabbable"
    );
    assert!(
        hit_half_px(PointHandleKind::AnchorB) >= JOINT_ANCHOR_RING_PX,
        "B's ring is drawn larger than its hit rect — visible and ungrabbable"
    );
    assert!(
        hit_half_px(PointHandleKind::AnchorB) - hit_half_px(PointHandleKind::AnchorA) >= 4.0,
        "the band between the two squares is where a coincident pair's B end is grabbed; \
         narrower than a few pixels and a Pin's B anchor is unreachable in practice"
    );
    assert!(
        SNAP_CROSS_PX > hit_half_px(PointHandleKind::AnchorB),
        "the snap crosshair must reach past the outermost handle, or the mark that explains \
         the magnet is hidden under the marks it explains"
    );
}

/// **The snap crosshair draws and takes no hit.** It is a readout — a mark that
/// says *this is why the dot stopped* — and a readout that swallows the pointer
/// would steal the drag it is describing.
#[test]
fn the_snap_mark_is_drawn_without_taking_the_pointer() {
    let mut v = view(vec![a(1, [0.0, 0.0])]);
    v.snap_world = Some([3.0, 3.0]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [3.0, 3.0]);
    assert_eq!(
        hits.hit(s[0], s[1]),
        None,
        "the snap crosshair must not register a hit"
    );
    assert_eq!(map.len(), 1, "only the A handle registers");
}

/// **The dot moves with the anchor.** Two anchors project to two different
/// screen positions, and the hit follows — so the handle sits on the joint, not
/// at a fixed screen spot.
#[test]
fn the_hit_follows_the_anchor() {
    for anchor in [[0.0, 0.0], [3.0, -2.0]] {
        let v = view(vec![a(5, anchor)]);
        let (hits, map) = paint(&v);
        let s = screen(&v, anchor);
        let id = hits.hit(s[0], s[1]).expect("hit at the projected anchor");
        assert_eq!(map[&id].world, anchor);
    }
}

/// **Where an anchor and a parameter grip land on each other, the ANCHOR wins.**
///
/// A limit wall can be grabbed anywhere along its tick and a length ring
/// anywhere on its circle; an anchor is a single point with nowhere else to go.
/// So the anchors register LAST (`PAINT_ORDER`) and the backwards walk of
/// `HitIndex::hit` hands them the shared pixel.
///
/// Mutation-tested: moving the anchors to the front of `PAINT_ORDER` makes the
/// grip swallow the dot, and this goes red.
#[test]
fn an_anchor_beats_a_parameter_grip_on_a_shared_pixel() {
    let p = |kind| PointHandle {
        key: 42,
        kind,
        world: [1.0, 1.0],
    };
    let v = view(vec![
        p(PointHandleKind::AnchorA),
        p(PointHandleKind::Length),
        p(PointHandleKind::LimitMin),
    ]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [1.0, 1.0]);
    let id = hits.hit(s[0], s[1]).expect("something is there");
    assert_eq!(
        map[&id].kind,
        PointHandleKind::AnchorA,
        "a parameter grip took the anchor's own pixel — the anchor has nowhere \
         else to be grabbed, the grip has its whole line"
    );
}

/// **Uma vista INERTE desenha e não registra** (W-J4b).
///
/// Enio, smoke da W-J4: com o gesto de desenhar armado, as alças já postas
/// *"devem ficar inacessíveis ao mouse e discretamente semitransparentes para que
/// o usuário não corra o risco de mover os gizmos previamente colocados"*.
///
/// As duas metades num gate só, porque são um fato só: se registrar sem dimming,
/// o artista move um gizmo achando que está desenhando; se dimmar sem parar de
/// registrar, ele o move achando que não pode. ⚠️ O caso `inert: false` é o
/// CONTROLE — sem ele o gate passaria sobre um painter que nunca registra nada.
///
/// Mutação: manter o `hit_index.register` sob `inert` ⇒ a metade "inacessível"
/// falha nomeando a alça que ainda pega o clique.
#[test]
fn an_inert_view_is_drawn_but_registers_nothing() {
    let live = view(vec![a(7, [1.0, 1.0])]);
    let (hits, map) = paint(&live);
    let s = screen(&live, [1.0, 1.0]);
    assert!(
        hits.hit(s[0], s[1]).is_some(),
        "o controle: uma vista viva TEM de pegar o clique na âncora"
    );
    assert_eq!(map.len(), 1, "e mapear a alça de volta");

    let inert = PointGizmoView {
        inert: true,
        ..view(vec![a(7, [1.0, 1.0])])
    };
    let (hits, map) = paint(&inert);
    assert!(
        hits.hit(s[0], s[1]).is_none(),
        "com o gesto armado a alça não pode ser agarrada — o press é do gesto"
    );
    assert!(
        map.is_empty(),
        "e nada fica no mapa: um `id -> handle` órfão responderia um press que \
         outro caminho produzisse"
    );
}

/// **E a marca inerte é MAIS transparente, sem desaparecer.**
///
/// *"Discretamente semitransparente"* é uma faixa, não um valor: opaco demais e
/// nada mudou na tela (o artista não sabe que está fora de alcance); zero e as
/// âncoras existentes somem justamente no gesto em que ele quer VÊ-las para não
/// empilhar um joint em cima de outro.
#[test]
fn the_inert_mark_is_dimmer_but_still_visible() {
    let live = handle_color(false).to_rgba8().a;
    let dim = handle_color(true).to_rgba8().a;
    assert!(
        dim < live,
        "inerte tem de ser mais apagado: {dim} vs {live}"
    );
    assert!(dim > 0, "e não invisível: {dim}");
}

/// **As três alças de uma ROLDANA são DESENHADAS e agarráveis** (W-Pulley W6).
///
/// ⚠️ **Este gate nasceu VERMELHO sobre o produto shipado**, e ele é o report do
/// Enio: *"selecionar Geared Rope Drum não mostra três alças âmbar"*, depois
/// *"nada visível ainda"*. Duas correções reais — o enquadramento da cena e o
/// relógio parado — não o moveram, porque o defeito estava no terceiro estágio.
///
/// A causa era `PAINT_ORDER`, uma lista de kinds **escrita à mão** sobre a qual
/// o laço de pintura iterava: os três kinds de roldana caíam fora do filtro e
/// não eram desenhados **nem** registrados. O braço de `match` que os desenha
/// era código morto, e o compilador não podia denunciá-lo — um `match` precisa
/// ser exaustivo de qualquer forma, então ele *parecia* tratado.
///
/// ⚠️ **Havia gate dos dois lados e nenhum no meio:** `render_loop::point_gizmo`
/// provava que o publicador produz as três alças, e este arquivo provava que o
/// pintor desenha âncoras e grips. Ninguém afirmava que a saída de um chega à
/// entrada do outro — a costura não-testada, outra vez.
///
/// Mutação: voltar a ordem a uma lista sem os kinds de roldana ⇒ vermelho nas
/// **duas** metades (nada desenhado, nada agarrável).
#[test]
fn the_wheel_handles_are_painted_and_hittable() {
    let w = |kind, world| PointHandle {
        key: 9,
        kind,
        world,
    };
    let hs = vec![
        w(PointHandleKind::WheelCentre, [0.0, 0.0]),
        w(PointHandleKind::WheelRim, [2.0, 0.0]),
        w(PointHandleKind::WheelRimOut, [-1.0, 0.0]),
    ];
    let v = view(hs.clone());
    let (hits, map, drawn) = paint_counted(&v);
    assert_eq!(
        drawn,
        hs.len() as u32,
        "cada alça de roldana desenha um anel — o report é 'não aparece', e um \
         gate que só olhasse o hit ficaria verde sobre uma tela vazia"
    );
    for h in &hs {
        let s = screen(&v, h.world);
        let id = hits
            .hit(s[0], s[1])
            .unwrap_or_else(|| panic!("{:?} não pega o clique onde desenha", h.kind));
        assert_eq!(
            map.get(&id).map(|m| m.kind),
            Some(h.kind),
            "o mapa tem de devolver a alça que ESTÁ ali: sem isso o arrasto \
             inteiro de {:?} é inalcançável pelo canvas",
            h.kind
        );
    }
}

/// **A alça da roldana ganha o pixel que divide com uma âncora, e o CENTRO ganha
/// do aro.**
///
/// As duas metades são a mesma decisão de precedência, e as duas têm razão
/// própria. Contra a âncora: alça de roldana só é publicada para a roldana
/// **SELECIONADA**, enquanto toda âncora de joint é publicada sempre — quem
/// acabou de selecionar a roda quer a alça dela. Centro contra aro: eles só se
/// tocam num zoom em que o raio é sub-pixel, e ali *redimensionar* não quer
/// dizer nada enquanto *mover* quer.
///
/// Mutação: dar ao centro um rank menor que o do aro ⇒ o aro engole o centro e
/// a metade de baixo fica vermelha.
#[test]
fn a_wheel_handle_beats_an_anchor_and_its_centre_beats_its_rim() {
    let p = |kind| PointHandle {
        key: 42,
        kind,
        world: [1.0, 1.0],
    };
    let v = view(vec![
        p(PointHandleKind::AnchorA),
        p(PointHandleKind::WheelRim),
        p(PointHandleKind::WheelCentre),
    ]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [1.0, 1.0]);
    let id = hits.hit(s[0], s[1]).expect("something is there");
    assert_eq!(
        map[&id].kind,
        PointHandleKind::WheelCentre,
        "a roldana selecionada perdeu o próprio pixel"
    );
}
