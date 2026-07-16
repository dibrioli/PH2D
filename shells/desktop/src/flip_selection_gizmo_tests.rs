//! Gates da §4.A — o gizmo da SELEÇÃO. Cada 🔴 mata uma mutação provada vermelha na
//! construção (DIRETIVA §3). O par render/input inverso e o funil pose-free do move
//! (em `flip_transform`/`flip_pose_gizmo`) continuam verdes — esta fase não os toca.

use super::*;
use ph2d_ecs::{FlipObjectRef, Name, Transform};
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_vec_scene::Xform;

/// Um objeto (1 camada, chave 0 de arte EXCLUSIVA) com dois traços fechados: um
/// **retângulo SELECIONADO** (x∈[-3,-1], y∈[-1,1] ⇒ centro (-2,0)) e um triângulo NÃO
/// selecionado à direita. Devolve `(doc, sim, map, oid, lid, entity)`.
fn doc_two_shapes() -> (
    FlipDoc,
    SimWorld,
    FlipEntityMap,
    FlipObjectId,
    LayerId,
    ph2d_ecs::Entity,
) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Obj");
    let obj = doc.object_mut(oid).unwrap();
    let l = obj.add_layer("L");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let dr = obj.drawing_mut(d).unwrap();
    dr.strokes.push(closed(
        &[[-3.0, -1.0], [-1.0, -1.0], [-1.0, 1.0], [-3.0, 1.0]],
        true,
    ));
    dr.strokes
        .push(closed(&[[1.5, -1.0], [3.0, -1.0], [2.25, 1.2]], false));
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Obj"), FlipObjectRef(oid.0)))
        .id();
    let mut map = FlipEntityMap::new();
    map.insert(oid, e.to_bits());
    (doc, sim, map, oid, l, e)
}

fn closed(verts: &[[f32; 2]], selected: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &[x, y] in verts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 5.0,
            opacity: 1.0,
            color: Rgba::new(0.9, 0.9, 0.95, 1.0),
        });
    }
    s.closed = true;
    s.selected = selected;
    s
}

fn paused() -> Playhead {
    let mut p = Playhead::new(1.0 / 60.0);
    p.pause();
    p
}

/// Um traço de line-art com os pontos dados (helper local dos gates de ÁREA — o
/// `doc_two_shapes` monta um doc inteiro, e aqui basta o desenho).
fn seg_line(pts: &[(f32, f32)], width: f32) -> FlipStroke {
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

fn bare_drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes = strokes;
    d
}

/// 🔴 **O INTERIOR do gizmo arrasta a seleção** — o 2º achado do smoke do §4.A (Enio):
/// *"se clicar em qualquer lugar dentro do gizmo que não seja sobre ponto ou linha ou fill
/// não funciona. Isso precisa funcionar: qualquer clique na área do gizmo"*.
///
/// Um retângulo VAZADO (sem fill) selecionado: o clique no meio dele erra a tinta e o
/// interior do fill, e antes disso virava **marquee** — que ainda por cima LIMPAVA a
/// seleção. Agora é um `Move` do grupo.
///
/// Mutações que sangram: dropar o arm `(None, false) if in_box`; ou a
/// `grabbable_selection_box` deixar de recusar o que não tem extensão (o 2º assert cai).
#[test]
fn a_click_inside_the_gizmo_box_grabs_the_selection() {
    // Retângulo 0..20 × 0..20 SEM fill, selecionado. O centro (10,10) está a 10 de toda
    // aresta — muito além do MIN_PICK_PX (5), então a tinta é MESMO errada.
    let mut r = seg_line(&[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)], 4.0);
    r.closed = true;
    r.selected = true;
    let mut d = bare_drawing(vec![r]);
    let center = Vec2::new(10.0, 10.0);
    assert_eq!(
        crate::flip_select::stroke_at(&d, center, 1.0, &Xform::IDENTITY),
        None,
        "o fixture tem de ERRAR a tinta no centro (senao o teste passa pelo motivo errado)"
    );
    let in_box = selection_box_contains(&d, center, ph2d_tool_flip::EditDomain::Stroke);
    assert!(in_box, "o centro da selecao tem de estar na area do gizmo");
    assert_eq!(
        crate::flip_select::plan_down(&mut d, None, false, in_box),
        crate::flip_select::Down::Move { collapse_to: None },
        "clicar no vazio DENTRO do gizmo tem de agarrar a selecao"
    );
    assert!(
        d.strokes[0].selected,
        "agarrar o interior nao pode LIMPAR a selecao (era o que o marquee fazia)"
    );
    // 🔴 O par de AUSÊNCIA: FORA da caixa o gesto continua sendo o marquee (que limpa).
    let outside = Vec2::new(100.0, 100.0);
    let out_box = selection_box_contains(&d, outside, ph2d_tool_flip::EditDomain::Stroke);
    assert!(!out_box, "(100,100) esta fora da caixa 0..20");
    assert_eq!(
        crate::flip_select::plan_down(&mut d, None, false, out_box),
        crate::flip_select::Down::Marquee { additive: false },
        "fora da caixa o vazio ainda e marquee"
    );
}

/// 🔴 **O domínio POINT não tem gizmo — trocar o toggle some com ele NA HORA** (Enio,
/// smoke do §4.A: *"se eu seleciono no painel Select:Point, o gizmo do stroke deve sumir
/// imediatamente"*).
///
/// O gizmo é do domínio **Stroke**. No Point o alvo do clique são as **âncoras**, e os
/// handles pousariam em cima delas — a bbox de um retângulo tem as âncoras NAS quinas. É a
/// mesma regra que o ADR-0112 já tomou no Vector (o gizmo da forma só publica no modo
/// Select; em Node ele comeria o clique do nó).
///
/// **A troca de domínio faz BROADCAST** (`selection_to_point_domain`, W8): a MESMA seleção
/// que abria o gizmo no Stroke continua lá, ponto a ponto — a caixa segue com extensão.
/// Então o gizmo só some se a regra olhar o DOMÍNIO, e é isso que este gate prende.
///
/// Mutação que sangra: tirar o teste de domínio da `grabbable_selection_box`.
#[test]
fn the_point_domain_never_opens_the_gizmo() {
    use ph2d_tool_flip::EditDomain;
    let mut r = seg_line(&[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)], 4.0);
    r.closed = true;
    r.selected = true;
    let mut d = bare_drawing(vec![r]);
    // No STROKE a mesma selecao abre o gizmo (senao a recusa no Point nao prova nada).
    assert!(
        grabbable_selection_box(&d, EditDomain::Stroke).is_some(),
        "o fixture tem de abrir gizmo no Stroke"
    );
    // A troca de dominio REAL (o broadcast do W8): todo ponto do traco fica selecionado.
    d.selection_to_point_domain();
    assert!(
        d.strokes[0].all_points_selected(),
        "o broadcast mantem a selecao — e por isso a caixa AINDA teria extensao"
    );
    assert!(
        grabbable_selection_box(&d, EditDomain::Point).is_none(),
        "Select:Point tem de sumir com o gizmo do stroke NA HORA"
    );
    // Sem gizmo nao ha area: o clique no meio volta a ser o gesto do W8.
    assert!(!selection_box_contains(
        &d,
        Vec2::new(10.0, 10.0),
        EditDomain::Point
    ));
}

/// 🔴 **Uma seleção sem EXTENSÃO não abre o gizmo** — *"não se rotaciona ou escalona um
/// único ponto"* (Enio). Os 8 handles empilhariam sobre ele e roubariam o clique que o
/// move. O caso ALCANÇÁVEL no domínio Stroke é um traço de um ponto só (um toque): a bbox
/// dele tem meia-extensão `(0,0)` — o zero exato, sem épsilon inventado.
///
/// Mutação que sangra: tirar o teste de extensão da `grabbable_selection_box`.
#[test]
fn a_selection_without_extent_never_opens_the_gizmo() {
    use ph2d_tool_flip::EditDomain;
    let mut dot = seg_line(&[(7.0, -3.0)], 4.0);
    dot.selected = true;
    let d = bare_drawing(vec![dot]);
    assert!(
        grabbable_selection_box(&d, EditDomain::Stroke).is_none(),
        "um ponto so nao se rotaciona nem se escalona: nao pode abrir gizmo"
    );
    assert!(!selection_box_contains(
        &d,
        Vec2::new(7.0, -3.0),
        EditDomain::Stroke
    ));
    // Dois pontos distintos ja tem extensao ⇒ o gizmo abre.
    let mut two = seg_line(&[(7.0, -3.0), (7.0, 17.0)], 4.0);
    two.selected = true;
    let d2 = bare_drawing(vec![two]);
    let (c, h) =
        grabbable_selection_box(&d2, EditDomain::Stroke).expect("dois pontos tem extensao");
    assert_eq!(c, [7.0, 7.0]);
    assert_eq!(h, [0.0, 10.0]);
}

/// 🔴 **Seed = sample: a caixa do gizmo pousa na SELEÇÃO posada** — o pivô é
/// `objeto ∘ pose` aplicado ao centro dos pontos SELECIONADOS (não os do desenho
/// inteiro), com pose girada/escalada E objeto movido/escalado. Espelho do
/// `the_pose_gizmo_box_lands_on_the_posed_art`.
///
/// Mutação que sangra: `selection_center_half` varrer TODOS os pontos (ignorar
/// `point_selected`) → o centro inclui o triângulo e o pivô descola do retângulo.
#[test]
fn the_selection_gizmo_box_lands_on_the_posed_selection() {
    let (mut doc, mut sim, map, oid, lid, e) = doc_two_shapes();
    // A pose da CHAVE (arte exclusiva pode tê-la — Unlink preserva): girada + escalada.
    let c_local = [-2.0, 0.0]; // o centro do retângulo selecionado
    let trs = TransformSnapshot {
        translation: [40.0, -12.0],
        rotation: std::f32::consts::FRAC_PI_4, // 45°
        scale: [2.0, 1.5],
    };
    let pose = trs_to_pose(trs, c_local);
    doc.object_mut(oid).unwrap().set_frame_pose(lid, 0, pose);
    // E o OBJETO tem pose própria (na identidade o erro se esconde).
    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(6.0, 3.0),
        scale: Vec2::new(3.0, 3.0),
        ..Transform::IDENTITY
    });
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    let ph = paused();
    let v = selection_view(
        &sim,
        &doc,
        &map,
        SelectionViewInputs {
            playhead: &ph,
            active_layer: None,
            last_pointer: (0.0, 0.0),
            domain: ph2d_tool_flip::EditDomain::Stroke,
        },
        &cam,
        ws,
    )
    .expect("arte exclusiva com seleção publica a view");
    // Oráculo: a MESMA cadeia do render, no centro da SELEÇÃO.
    let obj_x = crate::flip_transform::object_xform(&sim, e);
    let posed_c = pose.apply(Vec2::new(c_local[0], c_local[1]));
    let want = obj_x.apply([f64::from(posed_c.x), f64::from(posed_c.y)]);
    assert!(
        (f64::from(v.pivot_world[0]) - want[0]).abs() < 1e-3
            && (f64::from(v.pivot_world[1]) - want[1]).abs() < 1e-3,
        "pivô {:?} != seleção posada {want:?}",
        v.pivot_world
    );
    // O retângulo selecionado é 2×2 (half 1×1); com pose 2×1.5 ⊙ objeto 3: 1·2·3 × 1·1.5·3.
    let half = [
        (v.bbox_max_world[0] - v.bbox_min_world[0]) * 0.5,
        (v.bbox_max_world[1] - v.bbox_min_world[1]) * 0.5,
    ];
    assert!(
        (half[0] - 6.0).abs() < 1e-2 && (half[1] - 4.5).abs() < 1e-2,
        "meia-extensão {half:?} (esperado 6 × 4.5)"
    );
    assert!((v.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
}

/// 🔴 **O snapshot é SÓ os pontos selecionados** (+ os buracos de um traço INTEIRO
/// selecionado) — o resto do desenho não entra e por isso não anda. Um traço com
/// buraco, todo selecionado, leva o buraco; um traço não selecionado não contribui
/// nada.
///
/// Mutação que sangra: `snapshot_selected_points` ignorar `point_selected` (snapshotar
/// tudo) → os 3 vértices do triângulo entram e o gizmo os moveria junto.
#[test]
fn the_snapshot_is_only_the_selected_points() {
    let (mut doc, _sim, _map, oid, lid, _e) = doc_two_shapes();
    // Dá um BURACO ao retângulo (traço 0, o selecionado inteiro).
    let did = crate::flip_select::visible_drawing(&doc, &paused(), Some(lid))
        .map(|(_, _, d)| d)
        .unwrap();
    doc.object_mut(oid)
        .unwrap()
        .drawing_mut(did)
        .unwrap()
        .strokes[0]
        .holes
        .push(vec![
            Vec2::new(-2.4, -0.4),
            Vec2::new(-1.6, -0.4),
            Vec2::new(-2.0, 0.4),
        ]);
    let dr = doc.object(oid).unwrap().drawing(did).unwrap();
    let pts = snapshot_selected_points(dr);
    // 4 vértices do retângulo (todos selecionados) + 3 do buraco = 7; zero do triângulo.
    assert_eq!(pts.len(), 7, "esperado 4 main + 3 buraco, só do traço 0");
    assert!(pts.iter().all(|p| p.si == 0), "nenhum ponto do traço 1");
    assert_eq!(
        pts.iter()
            .filter(|p| matches!(p.ring, Ring::Hole(_)))
            .count(),
        3,
        "os 3 pontos do buraco andam com o traço inteiro selecionado"
    );
}

/// 🔴 **Um Rotate assa uma rotação em torno do CENTRO da seleção** — cada ponto
/// selecionado orbita o centro; o mesmo laço do `flip_selection_gizmo_move` com as
/// funções REAIS (`advance_cursor` + `compute_gizmo_transform` + `art_bake_xform`).
///
/// Mutação que sangra: `art_bake_xform` dropar o `start_inv` (usar identidade) → a
/// rotação passa a ser em torno da ORIGEM da pose e os pontos vão para longe.
#[test]
fn a_rotate_drag_spins_the_selection_about_its_center() {
    // Objeto e pose na identidade ⇒ ART = MUNDO; centro da seleção = c_art.
    let c_art = [-2.0, 0.0];
    let pose = Pose::IDENTITY;
    let start = pose_trs(pose, c_art); // translation = c_art, rot 0, escala 1
    let cam = GizmoCamera {
        center: [0.0, 0.0],
        height_world: 20.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    let to_screen = |w: [f32; 2]| -> (f32, f32) {
        let half_h = cam.height_world * 0.5;
        let half_w = half_h * (cam.window_w / cam.window_h);
        (
            (w[0] + half_w) / (2.0 * half_w) * cam.window_w,
            cam.window_h - (w[1] + half_h) / (2.0 * half_h) * cam.window_h,
        )
    };
    let pivot = start.translation; // = c_art (parent identity)
    let start_cursor = [pivot[0] + 1.0, pivot[1]];
    let mut drag = ph2d_editor::GizmoDragState {
        kind: ph2d_editor::GizmoDragKind::Rotate,
        entity_bits: 1,
        start_screen: to_screen(start_cursor),
        cursor_screen: to_screen(start_cursor),
        start_transform: start,
        pivot_world: pivot,
        start_cursor_world: start_cursor,
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: false,
        target: ph2d_editor::GizmoTarget::FlipSelection,
        parent_world: TransformSnapshot::IDENTITY,
        turns: 0,
    };
    for w in [
        [pivot[0] + 0.707, pivot[1] + 0.707], // 45°
        [pivot[0], pivot[1] + 1.0],           // 90° CCW
    ] {
        drag.advance_cursor(to_screen(w), &cam);
    }
    let new_t = ph2d_editor::compute_gizmo_transform(
        &drag,
        &cam,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    let m = art_bake_xform(pose, start, new_t);
    // Um vértice do retângulo, a +1 em x do centro, aparece a +1 em y depois de 90° CCW.
    let corner = [c_art[0] + 1.0, c_art[1]];
    let out = m.apply([f64::from(corner[0]), f64::from(corner[1])]);
    assert!(
        (out[0] - f64::from(c_art[0])).abs() < 1e-2
            && (out[1] - f64::from(c_art[1] + 1.0)).abs() < 1e-2,
        "o vértice não orbitou 90° em torno do centro: {out:?} (centro {c_art:?})"
    );
    // O CENTRO da seleção é ponto fixo do giro.
    let cc = m.apply([f64::from(c_art[0]), f64::from(c_art[1])]);
    assert!(
        (cc[0] - f64::from(c_art[0])).abs() < 1e-2 && (cc[1] - f64::from(c_art[1])).abs() < 1e-2,
        "o centro transladou ao girar: {cc:?}"
    );
}

/// 🔴 **Arte INSTANCIADA não abre gizmo de seleção** — a instância é da pose gizmo
/// (arte compartilhada não deforma por arrasto). Mutação que sangra: inverter/remover o
/// `is_instanced` do `selection_target`.
#[test]
fn an_instanced_drawing_never_opens_the_selection_gizmo() {
    let (mut doc, sim, map, oid, lid, _e) = doc_two_shapes();
    // Vira a chave 0 numa instância: cria a 12 compartilhando, e a 0 passa a ter users=2.
    assert!(
        doc.object_mut(oid)
            .unwrap()
            .duplicate_frame(lid, 0, 12, ph2d_flip::DupMode::Instance)
    );
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    assert!(
        selection_view(
            &sim,
            &doc,
            &map,
            SelectionViewInputs {
                playhead: &paused(),
                active_layer: Some(lid),
                last_pointer: (0.0, 0.0),
                domain: ph2d_tool_flip::EditDomain::Stroke,
            },
            &cam,
            ws,
        )
        .is_none(),
        "arte compartilhada (users≥2) é da pose gizmo, não da de seleção"
    );
}

/// 🔴 **Sem seleção, sem gizmo** — um desenho de arte exclusiva mas nada selecionado
/// não publica caixa (senão ela enquadraria o vazio e comeria o clique de seleção).
/// Mutação que sangra: `selection_center_half` devolver `Some` sobre zero pontos.
#[test]
fn an_empty_selection_never_opens_the_gizmo() {
    let (mut doc, sim, map, oid, lid, _e) = doc_two_shapes();
    // Desmarca tudo.
    let did = crate::flip_select::visible_drawing(&doc, &paused(), Some(lid))
        .map(|(_, _, d)| d)
        .unwrap();
    doc.object_mut(oid)
        .unwrap()
        .drawing_mut(did)
        .unwrap()
        .clear_selection();
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    assert!(
        selection_view(
            &sim,
            &doc,
            &map,
            SelectionViewInputs {
                playhead: &paused(),
                active_layer: Some(lid),
                last_pointer: (0.0, 0.0),
                domain: ph2d_tool_flip::EditDomain::Stroke,
            },
            &cam,
            ws,
        )
        .is_none(),
        "sem pontos selecionados o gizmo de seleção não abre"
    );
}

/// **Um Translate puro é um deslocamento RÍGIDO** que casa com o funil do move — todo
/// ponto anda pelo mesmo delta de ART, e esse delta é o delta de objeto descido pela
/// parte linear inversa da pose (`object_delta_to_art`, o que o move do Edit já usa).
/// Espelho da invariante "translação pura = identidade byte a byte" da pose.
#[test]
fn a_pure_translate_bake_is_a_rigid_shift_matching_the_move_funnel() {
    // Pose girada (linear ≠ identidade) para o funil de fato importar.
    let c_art = [10.0, -4.0];
    let pose = trs_to_pose(
        TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: std::f32::consts::FRAC_PI_3,
            scale: [1.0, 1.0],
        },
        c_art,
    );
    let start = pose_trs(pose, c_art);
    // Um Translate desloca a translação do TRS por um delta de OBJETO.
    let delta_obj = [3.0, 2.0];
    let new_t = TransformSnapshot {
        translation: [
            start.translation[0] + delta_obj[0],
            start.translation[1] + delta_obj[1],
        ],
        rotation: start.rotation,
        scale: start.scale,
    };
    let m = art_bake_xform(pose, start, new_t);
    // (1) Rígido: dois pontos distintos andam pelo MESMO vetor de ART.
    let p: [f32; 2] = [12.0, -1.0];
    let q: [f32; 2] = [8.0, -9.0];
    let mp = m.apply([f64::from(p[0]), f64::from(p[1])]);
    let mq = m.apply([f64::from(q[0]), f64::from(q[1])]);
    let dp = [mp[0] - f64::from(p[0]), mp[1] - f64::from(p[1])];
    let dq = [mq[0] - f64::from(q[0]), mq[1] - f64::from(q[1])];
    assert!(
        (dp[0] - dq[0]).abs() < 1e-4 && (dp[1] - dq[1]).abs() < 1e-4,
        "o Translate não foi rígido: {dp:?} != {dq:?}"
    );
    // (2) O delta de ART = `object_delta_to_art(pose, delta_obj)` — a MESMA descida do
    // move (o gizmo e o arrasto de canvas convergem no mesmo funil).
    let want =
        crate::flip_transform::object_delta_to_art(pose, Vec2::new(delta_obj[0], delta_obj[1]));
    assert!(
        (dp[0] - f64::from(want.x)).abs() < 1e-3 && (dp[1] - f64::from(want.y)).abs() < 1e-3,
        "o delta de ART {dp:?} != funil do move {want:?}"
    );
}
