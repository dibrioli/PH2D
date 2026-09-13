//! Os gates do gizmo do colisor da forma (doc 109 §5).

use super::*;

const SEM_GIRO: [f32; 2] = [1.0, 0.0];

fn caixa_em(p: [f32; 2], meia: [f32; 2], eixo: [f32; 2]) -> Peca {
    Peca {
        linha: 0,
        p,
        colisor: Colisor::caixa(meia, eixo),
        size: [1.0, 1.0],
    }
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5
}

fn alca(hs: &[Handle], x: i8, y: i8) -> Handle {
    *hs.iter()
        .find(|h| h.alca == Alca { x, y })
        .unwrap_or_else(|| panic!("sem a alca ({x}, {y})"))
}

/// ⭐ **As alças de uma caixa estão nos cantos e nos lados DELA, giradas com a peça.**
#[test]
fn the_box_handles_sit_on_its_corners_and_edges_turned_with_the_piece() {
    let em_pe = caixa_em([1.0, 1.0], [0.5, 0.25], [0.0, 1.0]);
    let hs = handles(&em_pe);
    assert_eq!(hs.len(), 8);
    // O eixo `x` da caixa aponta para cima: o lado `+x` fica 0,5 acima, o `+y` 0,25 à esquerda.
    assert!(perto(alca(&hs, 1, 0).world, [1.0, 1.5]), "{hs:?}");
    assert!(perto(alca(&hs, 0, 1).world, [0.75, 1.0]), "{hs:?}");
    assert!(perto(alca(&hs, 1, 1).world, [0.75, 1.5]), "{hs:?}");
}

/// ⭐ **As alças de um círculo estão na borda, nos quatro eixos.**
#[test]
fn the_circle_handles_sit_on_its_rim() {
    let peca = Peca {
        linha: 0,
        p: [2.0, 0.0],
        colisor: Colisor::disco(0.3),
        size: [1.0, 1.0],
    };
    let hs = handles(&peca);
    assert_eq!(hs.len(), 4);
    assert!(perto(alca(&hs, 0, 1).world, [2.0, 0.3]) && perto(alca(&hs, -1, 0).world, [1.7, 0.0]));
}

/// ⭐⭐ **A alça que se vê é a alça que se agarra** — e longe de todas não se agarra nada.
#[test]
fn the_handle_you_see_is_the_handle_you_grab() {
    for peca in [
        caixa_em([0.0, 0.0], [0.5, 0.25], [0.6, 0.8]),
        Peca {
            linha: 0,
            p: [0.0, 0.0],
            colisor: Colisor::disco(0.4),
            size: [1.0, 1.0],
        },
    ] {
        let hs = handles(&peca);
        for (i, h) in hs.iter().enumerate() {
            assert_eq!(hit(&hs, h.world, 1e-3), Some(i), "{h:?}");
        }
        assert_eq!(hit(&hs, [5.0, 5.0], 1e-3), None);
    }
}

fn arrasto(x: i8, y: i8, circulo: bool, size: [f32; 2], eixo: [f32; 2]) -> Arrasto {
    Arrasto {
        node: NodeId(1),
        linha: 0,
        alca: Alca { x, y },
        circulo,
        fit: ColliderFit {
            center: [0.0, 0.0],
            half: [1.0, 0.5],
        },
        size,
        eixo,
        ancora: [0.0, 0.0],
        inicio: [1.0, 1.0],
    }
}

/// ⭐⭐ **Arrastar um LADO muda só esse eixo, a partir de onde ele estava.**
///
/// Base de mundo `1 × 0,5 = 0,5` na largura: andar `0,25` para fora dá meia `0,75` ⇒ `1,5`.
#[test]
fn dragging_a_box_edge_resizes_that_axis_from_where_it_was() {
    let a = arrasto(1, 0, false, [0.5, 0.5], SEM_GIRO);
    assert_eq!(edits(&a, [0.25, 7.0]), vec![(param::COLLIDER_WIDTH, 1.5)]);
    // O lado de BAIXO anda para baixo: base `0,5 × 0,5 = 0,25`, `+0,25` ⇒ `2`.
    let b = arrasto(0, -1, false, [0.5, 0.5], SEM_GIRO);
    assert_eq!(edits(&b, [9.0, -0.25]), vec![(param::COLLIDER_HEIGHT, 2.0)]);
}

/// ⭐⭐ **Um CANTO muda os dois, e uma caixa GIRADA lê a mão pelos eixos DELA.**
#[test]
fn a_corner_resizes_both_and_a_turned_box_reads_its_own_axes() {
    let canto = arrasto(1, 1, false, [0.5, 0.5], SEM_GIRO);
    assert_eq!(
        edits(&canto, [0.25, 0.25]),
        vec![(param::COLLIDER_WIDTH, 1.5), (param::COLLIDER_HEIGHT, 2.0)]
    );
    let em_pe = arrasto(1, 0, false, [0.5, 0.5], [0.0, 1.0]);
    let de_lado = edits(&em_pe, [0.25, 0.0]);
    assert!(
        (de_lado[0].1 - 1.0).abs() < 1e-6,
        "para o lado nao muda: {de_lado:?}"
    );
    let para_cima = edits(&em_pe, [0.0, 0.25]);
    assert!(
        (para_cima[0].1 - 1.5).abs() < 1e-6,
        "ao longo do eixo muda: {para_cima:?}"
    );
}

/// ⭐ **Passar o centro dá ZERO**, nunca uma caixa do avesso.
#[test]
fn dragging_past_the_centre_clamps_at_zero() {
    let a = arrasto(1, 0, false, [0.5, 0.5], SEM_GIRO);
    assert_eq!(edits(&a, [-3.0, 0.0]), vec![(param::COLLIDER_WIDTH, 0.0)]);
}

/// ⭐ **A borda do círculo muda o raio** — base `max(1; 0,5) × max(0,2; 0,3) = 0,3`, `+0,3` ⇒ `2`.
#[test]
fn dragging_the_rim_resizes_the_circle() {
    let a = arrasto(0, 1, true, [0.2, -0.3], SEM_GIRO);
    let e = edits(&a, [5.0, 0.3]);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].0, param::COLLIDER_RADIUS);
    assert!((e[0].1 - 2.0).abs() < 1e-5, "{e:?}");
}

/// ⭐⭐ **O pintor emite o contorno de CADA peça e as alças de uma**, e nada sem peças.
#[test]
fn the_gizmo_paints_every_outline_and_the_handles_of_one_piece() {
    use ph2d_vector::VectorScene;
    let v = ColliderGizmoView {
        node: NodeId(1),
        circulo: false,
        fit: ColliderFit {
            center: [0.0, 0.0],
            half: [1.0, 1.0],
        },
        pecas: vec![
            caixa_em([0.0, 0.0], [0.5, 0.5], SEM_GIRO),
            caixa_em([2.0, 0.0], [0.5, 0.5], SEM_GIRO),
        ],
    };
    let segmentos = |v: &ColliderGizmoView| {
        let mut cena = VectorScene::new();
        crate::collider_gizmo_overlay::draw(
            v,
            &ph2d_render::Camera2d::default(),
            ph2d_editor_core::screens::layout::CenterSplit::None,
            ph2d_host::WindowSize {
                width: 1200,
                height: 800,
            },
            &mut cena,
        );
        cena.inner().encoding().n_path_segments
    };
    let vazia = ColliderGizmoView {
        pecas: Vec::new(),
        ..v.clone()
    };
    let uma = ColliderGizmoView {
        pecas: vec![v.pecas[0]],
        ..v.clone()
    };
    assert_eq!(segmentos(&vazia), 0, "sem pecas nao ha tinta");
    let (s1, s2) = (segmentos(&uma), segmentos(&v));
    assert!(s1 > 0, "a peca com alcas pinta");
    assert!(s2 > s1, "a segunda peca tem contorno: {s1} contra {s2}");
}

/// **A SONDA DO CUSTO DA TINTA** — de onde sai o [`MAX_CONTORNOS`]: quanto custa codificar `n`
/// contornos de caixa num quadro.
#[test]
#[ignore = "sonda de custo — corre com a maquina calma"]
fn measure_the_outline_paint_cost() {
    use ph2d_vector::VectorScene;
    for n in [256_usize, 1024, 2048, 4096, 16_384] {
        #[expect(clippy::cast_precision_loss, reason = "coordenadas de sonda")]
        let pecas: Vec<Peca> = (0..n)
            .map(|i| {
                caixa_em(
                    [(i % 128) as f32 * 0.2, (i / 128) as f32 * 0.2],
                    [0.08, 0.08],
                    SEM_GIRO,
                )
            })
            .collect();
        let v = ColliderGizmoView {
            node: NodeId(1),
            circulo: false,
            fit: ColliderFit::default(),
            pecas,
        };
        let mut tempos: Vec<f64> = (0..15)
            .map(|_| {
                let mut cena = VectorScene::new();
                let t = std::time::Instant::now();
                crate::collider_gizmo_overlay::draw(
                    &v,
                    &ph2d_render::Camera2d::default(),
                    ph2d_editor_core::screens::layout::CenterSplit::None,
                    ph2d_host::WindowSize {
                        width: 1200,
                        height: 800,
                    },
                    &mut cena,
                );
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        tempos.sort_by(f64::total_cmp);
        eprintln!(
            "  {n:>6} contornos │ mediana {:.3} ms",
            tempos[tempos.len() / 2]
        );
    }
}

#[cfg(feature = "panel-motion-graph")]
mod na_cena {
    use super::*;

    /// A `=114` montada pelo roteador, a forma pedida seleccionada, as tomadas armadas pela porta
    /// do gizmo, as formas publicadas e UM quadro marchado na CPU.
    fn cena(seleccionar_a_direita: bool) -> (MotionState, NodeId) {
        let mut m = MotionState::new();
        let sinks = crate::motion_demo_legend::monta("114", &mut m.doc, &m.registry).0;
        m.sinks = sinks;
        let formas: Vec<NodeId> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "source.shape")
            .map(|n| n.id)
            .collect();
        let colide = |id: &NodeId| param_value(&m, *id, param::COLLIDE) >= 0.5;
        let direita = *formas.iter().find(|id| colide(id)).expect("a da direita");
        let esquerda = *formas.iter().find(|id| !colide(id)).expect("a da esquerda");
        let alvo = if seleccionar_a_direita {
            direita
        } else {
            esquerda
        };
        ph2d_panel_motion_graph::set_graph_selection(vec![alvo.0]);
        let taps = taps_for(&m);
        m.pump.set_taps(&taps);
        m.pump.clear_tap_fires();
        crate::motion_shape_gen::publish(&mut m, 0.0);
        crate::warp_gizmo_fixtures::marcha_na_cpu(&mut m);
        (m, direita)
    }

    /// ⭐⭐⭐ **O gizmo acha AS 25 peças que a forma seleccionada carimbou**, cada uma com a caixa
    /// dela, e a primeira é a mais próxima do cursor. Os CONTROLOS: fora da tool Motion, e com a
    /// forma de `Collide` desligado seleccionada, não há gizmo.
    #[test]
    fn the_gizmo_finds_every_piece_the_selected_shape_stamped() {
        let _t = crate::warp_gizmo_fixtures::trava();
        let (m, direita) = cena(true);
        assert_eq!(taps_for(&m).len(), 2, "a forma e o sink");
        let cursor = [2.3, -0.6];
        let v = resolve(&m, true, Some(cursor)).expect("o gizmo existe");
        assert_eq!(v.node, direita);
        assert!(!v.circulo, "Box e' o default");
        assert_eq!(v.pecas.len(), 25, "as 25 pecas da taca da direita");
        for q in &v.pecas {
            match q.colisor.forma {
                Forma::Caixa { meia, .. } => {
                    assert!(perto(meia, [0.11, 0.11]), "a caixa do quadrado: {meia:?}");
                }
                Forma::Disco(_) => panic!("Box declara caixas"),
            }
        }
        let d = |q: &Peca| (q.p[0] - cursor[0]).hypot(q.p[1] - cursor[1]);
        assert!(
            v.pecas.iter().all(|q| d(&v.pecas[0]) <= d(q)),
            "as alcas vao para a peca mais proxima do cursor"
        );
        assert!(
            resolve(&m, false, Some(cursor)).is_none(),
            "fora da tool Motion"
        );

        let (sem, _) = cena(false);
        assert!(
            resolve(&sem, true, Some(cursor)).is_none(),
            "a forma sem Collide nao tem gizmo"
        );
    }

    /// ⭐⭐⭐ **Arrastar uma alça escreve o param do CARTÃO, e o arrasto é UM passo de undo.**
    #[test]
    fn a_handle_drag_writes_the_card_param_and_is_one_undo_step() {
        let _t = crate::warp_gizmo_fixtures::trava();
        let (mut m, direita) = cena(true);
        let v = resolve(&m, true, None).expect("o gizmo existe");
        let lado = alca(&handles(&v.pecas[0]), 1, 0);
        publish(Some(v));
        assert!(
            pointer_down(&mut m, lado.world, 1e-3),
            "agarra a alca do lado"
        );
        assert!(!m.history.can_undo(), "agarrar ainda nao e' um passo");
        // A caixa do quadrado tem meia 0,11; andar 0,11 para fora dobra-a.
        assert!(pointer_move(&mut m, [lado.world[0] + 0.11, lado.world[1]]));
        let largura = param_value(&m, direita, param::COLLIDER_WIDTH);
        assert!((largura - 2.0).abs() < 1e-3, "o cartao le {largura}");
        assert!(
            (param_value(&m, direita, param::COLLIDER_HEIGHT) - 1.0).abs() < 1e-6,
            "o lado nao mexe na altura"
        );
        assert!(pointer_up(&mut m), "larga");
        assert!(!pointer_up(&mut m), "e larga UMA vez");
        assert!(m.history.can_undo(), "o arrasto e' um passo");
        m.doc = m.history.undo(&m.doc).expect("desfaz");
        assert!(
            (param_value(&m, direita, param::COLLIDER_WIDTH) - 1.0).abs() < 1e-6,
            "o Ctrl+Z devolve a largura"
        );
        publish(None);
    }
}
