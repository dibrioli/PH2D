//! **A SESSÃO molhada: molhar, secar e o que sobrevive a isso.** O botão Wet e o Dry, a secagem que
//! recua das bordas, o segundo traço que continua a sessão em vez de a reiniciar, as junções entre
//! washes, o backrun de água limpa, o rim assinado, e o re-render que reproduz o bake byte a byte.

use super::*;

/// **EDGE-1 (doc 12, W-C): washes que se encostam MOLHADOS fundem — sem contorno duplo.** Dentro
/// da janela de secagem (~8,5 s, DiVerdi) o segundo traço CONTINUA a sessão molhada: os buffers
/// acumulam a UNIÃO e o bake re-renderiza tudo sobre a base da sessão — um wash só, um rim só
/// (o rim interno do primeiro traço DERRETE no re-bake, e o traço novo não desenha rim sobre o
/// vizinho — Curtis §3-4). Depois de seco, o mesmo gesto volta a empilhar rim por cima (glazing)
/// e o mapa seco é DROPADO junto com a sessão (fast path de volta, sem custo ocioso).
#[test]
fn watercolor_touching_wet_washes_merge_without_double_rim() {
    let run = |dry_first: bool| -> f32 {
        let size = 192u32;
        let mut t = white_canvas(size, 8.0);
        t.paint.brush = BrushSpec {
            radius_px: 12.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.12,
            depth: 1.0,
            edge_gain: 2.5,
            edge_spread: 6.0,
            warp: 0.0,
            granulation: 0.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        let stroke_v = |t: &mut PainterTool, x: f32| {
            assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
            let mut y = 30.0f32;
            while y < 160.0 {
                y += 2.0;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x, 160.0], PointerPhase::Up));
        };
        stroke_v(&mut t, 80.0); // wash A (band x ≈ 68..92) — pours moisture at its bake
        assert!(
            !t.paint.canvas_wet.is_empty(),
            "the bake must pour the wash into the persistent wet map"
        );
        if dry_first {
            for _ in 0..140 {
                t.paint_tick(0.5); // 70 s of heartbeat — way past the ~60 s drying window
            }
            assert!(
                t.paint.canvas_wet.is_empty(),
                "a fully-dried wet map must be dropped (composite fast path back)"
            );
        }
        stroke_v(&mut t, 88.0); // wash B overlaps A deeply (union band ≈ 68..100)
        // Junction probes: B's would-be LEFT rim (x 78-81, deep in A's interior) + A's OWN right
        // rim position (x 88-91). Merged (wet): both re-render as union INTERIOR — light. Dried
        // first: B lays a rim over A AND A's baked rim persists under B — both bands dark.
        let mut acc = 0.0f32;
        for x in [78u32, 79, 80, 81, 88, 89, 90, 91] {
            for y in 80..110u32 {
                acc += f32::from(px(&t, size, x, y)[1]);
            }
        }
        acc / (8.0 * 30.0)
    };
    let wet_junction = run(false);
    let dry_junction = run(true);
    assert!(
        wet_junction > dry_junction + 40.0,
        "wet washes must MERGE into one rim-less junction — both the new stroke's rim over the \
         neighbour and the neighbour's old inner rim must be gone (wet G {wet_junction:.1} vs \
         dried-first double-contour G {dry_junction:.1})"
    );
}

/// **EDGE-1 #3 (Enio smoke 2026-07-11) — "Wet the layer" (Rebelle):** o botão **Wet** re-molha o canvas e
/// reabre uma sessão molhada sobre a tinta EXISTENTE, com um rewet FORÇADO — então um traço de água clara
/// (Rewet do pincel = 0) feito depois LEVANTA a tinta seca (clareia rumo ao papel), coisa que NÃO acontece
/// sem apertar Wet (água clara sobre papel seco não reativa nada). Propriedade: mesmo A seco + mesmo B de
/// água, o núcleo de A fica mais claro COM Wet que SEM. DIRETIVA §4 (o forced-rewet é o discriminador).
#[test]
fn watercolor_wet_button_reactivates_dry_paint() {
    let size = 96u32;
    fn dry_wash_a(size: u32) -> PainterTool {
        let mut t = white_canvas(size, 8.0);
        t.paint.brush = BrushSpec {
            radius_px: 16.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.8, 0.1, 0.1], // a solid dark-red wash to reactivate
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 1.5,
            edge_gain: 0.0,
            warp: 0.0,
            granulation: 0.0,
            wet_rewet: 0.0, // the brush itself does NOT rewet — only the Wet button will
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        // Wash A — a vertical band at x = 48.
        assert!(t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Down)));
        let mut y = 20.0f32;
        while y < 76.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([48.0, 76.0], PointerPhase::Up));
        // Dry it FULLY (past the ~10 s window) so the session tears down — A is dry paint now.
        for _ in 0..60 {
            t.paint_tick(0.5);
        }
        assert!(
            t.paint.canvas_wet.is_empty(),
            "A must be fully dry before the test"
        );
        t
    }
    // A clear-water stroke over A (same brush, near-white colour, low fill → the deposit is negligible,
    // any change is the LIFT). `press_wet` toggles the Wet button before painting.
    let clear_water_over_a = |press_wet: bool| -> [u8; 4] {
        let mut t = dry_wash_a(size);
        t.paint.brush.color = [0.98, 0.98, 0.98];
        t.paint.brush.fill = 0.1;
        t.paint.brush.opacity = 0.0; // no body: pure transparent water, so any change is the LIFT only
        t.paint.brush_by_mode.fill(t.paint.brush);
        if press_wet {
            t.wet_canvas_now();
        }
        assert!(t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Down)));
        let mut y = 20.0f32;
        while y < 76.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([48.0, 76.0], PointerPhase::Up));
        px(&t, size, 48, 48)
    };
    let lum = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    let with_wet = clear_water_over_a(true);
    let without_wet = clear_water_over_a(false);
    assert!(
        lum(with_wet) > lum(without_wet) + 30,
        "the Wet button must reactivate the dry wash (a clear-water stroke lifts it lighter): \
         with Wet {with_wet:?} (lum {}) vs without {without_wet:?} (lum {})",
        lum(with_wet),
        lum(without_wet),
    );
}

/// **EDGE-1 #4 (Enio smoke 2026-07-11):** cada traço seca no SEU próprio relógio — um segundo traço
/// (longe do primeiro) NÃO pode re-molhar o wash anterior. `stroke_coverage` é a UNIÃO da sessão; despejar
/// isso na moisture pela rect cumulativa re-molhava TUDO a 255 no bake de cada traço (resetando a secagem
/// dos anteriores). O pour agora usa só a footprint do traço atual. Propriedade: seco parcialmente o A,
/// pinto B longe, e a umidade de A no seu núcleo NÃO sobe (sem o fix ela voltaria a 255). DIRETIVA §4.
#[test]
fn watercolor_second_stroke_does_not_reset_the_first_strokes_drying() {
    let size = 160u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.2,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let stroke_v = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
        let mut y = 30.0f32;
        while y < 130.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 130.0], PointerPhase::Up));
    };
    let wet_at = |t: &PainterTool, x: usize, y: usize| t.paint.canvas_wet[y * size as usize + x];

    stroke_v(&mut t, 40.0); // wash A (band x ≈ 28..52) — pours moisture at its bake
    assert!(
        !t.paint.canvas_wet.is_empty(),
        "A's bake must pour moisture"
    );
    // Dry A partially: ~4 s of heartbeat (rate 25.5 ⇒ ~102 bytes off 255, still comfortably wet).
    for _ in 0..8 {
        t.paint_tick(0.5);
    }
    let a_before = wet_at(&t, 40, 80);
    assert!(
        a_before > 0 && a_before < 220,
        "A must have partially dried before B (got {a_before})"
    );

    stroke_v(&mut t, 120.0); // wash B (band x ≈ 108..132) — FAR from A, no overlap
    let a_after = wet_at(&t, 40, 80);
    let b_fresh = wet_at(&t, 120, 80);
    assert!(
        u32::from(b_fresh) > u32::from(a_after) + 40,
        "B is freshly wet, A stayed drier: B {b_fresh} vs A {a_after}"
    );
    assert!(
        a_after <= a_before,
        "painting B must NOT re-wet A's core (its own drying clock): A was {a_before}, became {a_after}"
    );
}

/// **#4b (Enio smoke 2026-07-11, "retângulo gigante na união"):** um traço cujo BBOX apenas CONTÉM um wash
/// anterior (sem a footprint cobri-lo) NÃO pode re-molhar esse wash. `stroke_coverage` é a UNIÃO; o pour
/// despejava-a sobre o RECT do traço → re-molhava os pixels do vizinho DENTRO do rect a 255 (um retângulo
/// de umidade que o overlay pintava). Fix: o pour restringe à footprint DONA (`owner == cur_o`). Sonda: um
/// pixel de A coberto por A, DENTRO do bbox do diagonal B, mas FORA da footprint de B — não re-molha.
/// **Undo apaga a umidade (Enio smoke 2026-07-11):** desfazer um traço de aquarela tem que limpar o mapa
/// de umidade — o canvas voltou, mas o overlay continuava mostrando o damp do traço desfeito.
#[test]
fn watercolor_undo_clears_the_moisture() {
    let size = 64u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.3, 0.9],
        space_attenuation: false,
        watercolor: true,
        fill: 0.4,
        depth: 1.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([32.0, 20.0], PointerPhase::Down)));
    let mut y = 20.0f32;
    while y < 44.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([32.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([32.0, 44.0], PointerPhase::Up));
    assert!(
        t.paint.canvas_wet.iter().any(|&w| w > 0),
        "the stroke must have poured moisture"
    );
    assert!(t.undo_last(), "the stroke is undoable");
    assert!(
        t.paint.canvas_wet.is_empty(),
        "undo must clear the wet session's moisture map (no stale damp overlay)"
    );
}

#[test]
fn watercolor_overlapping_bbox_does_not_rewet_the_neighbour_wash() {
    let size = 80u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.3, 0.9],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let wet_at = |t: &PainterTool, x: usize, y: usize| t.paint.canvas_wet[y * size as usize + x];
    // A: horizontal band at y = 30 (x ≈ 12..68). Bake, then dry a little.
    assert!(t.on_canvas_pointer(cp([20.0, 30.0], PointerPhase::Down)));
    let mut x = 20.0f32;
    while x < 60.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Up));
    for _ in 0..6 {
        t.paint_tick(0.5);
    }
    let a_probe_before = wet_at(&t, 25, 30); // on A, will fall inside B's bbox but NOT B's footprint
    assert!(
        a_probe_before > 0 && a_probe_before < 230,
        "A's probe must be wet-but-decayed before B ({a_probe_before})"
    );
    // B: DIAGONAL from (20,60) to (60,20) — its bbox (20..60, 20..60) CONTAINS A, but at x=25 B sits at
    // y≈55, so (25,30) is inside B's bbox yet OUTSIDE B's footprint. Same wet session (A still wet).
    assert!(t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Down)));
    let mut s = 0.0f32;
    while s < 40.0 {
        s += 2.0;
        t.on_canvas_pointer(cp([20.0 + s, 60.0 - s], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 20.0], PointerPhase::Up));
    let a_probe_after = wet_at(&t, 25, 30);
    assert!(
        a_probe_after <= a_probe_before,
        "B's bbox merely CONTAINS A here — its footprint doesn't reach (25,30), so it must NOT re-wet it: \
         was {a_probe_before}, became {a_probe_after}"
    );
}

/// **#18 (Enio smoke 2026-07-11):** mudar params de Wash (Body/Concentration/Edge/Opacity) entre traços e
/// cruzar um traço úmido não pode imprimir borda dura na junção — os params por-dono degrauavam na fronteira
/// de posse (Bug #8 lição #4). O campo suavizado (`build_style_field`) espalha os params na fronteira.
#[test]
fn watercolor_param_change_junction_is_soft() {
    let size = 96u32;
    let mut t = white_canvas(size, 8.0);
    // A: vertical band x=48, params X (Body alto, Concentration alta) → renderiza ESCURO.
    t.paint.brush = BrushSpec {
        radius_px: 9.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.3, 0.9],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([48.0, 15.0], PointerPhase::Down)));
    let mut y = 15.0;
    while y < 80.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([48.0, 80.0], PointerPhase::Up));
    // B: horizontal band y=48 crossing A, params Y (Body baixo) → CLARO. Mesma sessão úmida.
    t.paint.brush.fill = 0.12;
    t.paint.brush.depth = 1.0;
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([15.0, 48.0], PointerPhase::Down)));
    let mut x = 15.0;
    while x < 80.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Up));
    // Max |grad| do verde ao longo de x=48, cruzando a fronteira A(escuro)/B(claro) em y~40. Sem o campo
    // suavizado os params degrauam ali (medido 118 bytes/px); com o campo, ~13.
    let g = |yy: u32| f32::from(px(&t, size, 48, yy)[1]);
    let mut maxg = 0.0f32;
    for yy in 33..47u32 {
        maxg = maxg.max((g(yy + 1) - g(yy)).abs());
    }
    assert!(
        maxg < 22.0,
        "a junção com params trocados deve ser SUAVE, não degrau (grad {maxg} bytes/px)"
    );
}

/// **#2 (Enio smoke 2026-07-11):** a umidade é lançada AO VIVO durante o traço — o damp aparece enquanto
/// pinta, não só no mouse-up ("a umidade só aparece no mouse up. isso é muito feio"). Sem SOLTAR, o mapa de
/// umidade já tem que estar populado sobre a região pintada (o pour antes rodava só no bake).
#[test]
fn watercolor_wetness_is_laid_live_during_the_stroke() {
    let size = 96u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([48.0, 30.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([48.0, 50.0], PointerPhase::Move));
    // NO pen-up yet — the moisture must already be present (live pour), not empty until the bake.
    assert!(
        t.paint.canvas_wet.iter().any(|&w| w > 0),
        "the wetness must be laid LIVE during the stroke (not only at pen-up)"
    );
}

/// **#3a/#12b (Enio smoke 2026-07-11):** o papel seca das BORDAS para o CENTRO — a poça molhada recede pelo
/// perímetro, não uniformemente. Pinto uma banda sólida (interior despeja umidade plana), seco PARCIALMENTE,
/// e o núcleo profundo continua mais úmido que a borda (a secagem uniforme esvaziaria os dois juntos).
#[test]
fn watercolor_drying_recedes_from_the_edges() {
    let size = 96u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 18.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // A solid vertical band centred on x = 48 (radius 18 ⇒ x ≈ 30..66).
    assert!(t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Down)));
    let mut y = 20.0f32;
    while y < 76.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([48.0, 76.0], PointerPhase::Up));
    let wet_at = |t: &PainterTool, x: usize| t.paint.canvas_wet[48 * size as usize + x];
    let center0 = wet_at(&t, 48);
    assert!(
        center0 > 200,
        "the band interior pours a high flat moisture ({center0})"
    );
    // Dry partially (the erosion recedes the perimeter faster than the flat interior decay).
    for _ in 0..12 {
        t.paint_tick(0.5);
    }
    let center = wet_at(&t, 48);
    // x = 34 is INTERIOR (coverage ~1 ⇒ it poured the same flat moisture as the centre): under a UNIFORM
    // decay it would equal the centre, but the edges-to-centre recession has eaten into it (front near x=34).
    let edge = wet_at(&t, 34);
    assert!(center > 0, "the deep centre must still be wet ({center})");
    assert!(
        u32::from(edge) + 40 < u32::from(center),
        "the recession must eat an INTERIOR pixel ahead of the deep centre (edge {edge} vs centre {center})"
    );
}

/// **EDGE-1 regressão (Enio smoke 2026-07-09, "traços duplicados"):** a janela de secagem vencendo
/// NO MEIO de um traço aberto não pode duplicar o wash. O teardown antigo derrubava a base da
/// sessão no tick (mapa zerado) mas deixava os buffers da união vivos (o traço aberto é dono
/// deles) — o bake do pen-up caía no fallback per-stroke (que JÁ contém a união assada) e
/// re-renderizava tudo por cima: dupla contagem, o conjunto escurecia de vez. O teardown agora é
/// ATÔMICO e adiado até não haver traço aberto.
#[test]
fn watercolor_session_drying_mid_stroke_does_not_double_bake() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.12,
        depth: 1.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A (vertical band at x = 60), baked + poured into the session.
    assert!(t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
    let a_interior_before = f32::from(px(&t, size, 60, 95)[1]);
    // Stroke C opens while the paper is still wet (session continues), then the drying deadline
    // fires MID-STROKE (one big heartbeat zeroes the map), then C finishes far from A.
    assert!(t.on_canvas_pointer(cp([140.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 90.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([140.0, y], PointerPhase::Move));
    }
    t.paint_tick(70.0); // way past the ~60 s window — the map zeroes with the stroke OPEN
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([140.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([140.0, 160.0], PointerPhase::Up));
    // A's interior (far from C) must be untouched by C's bake — the double-count re-rendered the
    // whole union over its own baked pixels and darkened it hard.
    let a_interior_after = f32::from(px(&t, size, 60, 95)[1]);
    assert!(
        (a_interior_after - a_interior_before).abs() <= 2.0,
        "the drying deadline mid-stroke must not re-bake (duplicate) the neighbour wash \
         (A interior G {a_interior_before:.1} -> {a_interior_after:.1})"
    );
}

/// **Sessão molhada — params POR TRAÇO (doc 13 topo, Enio 2026-07-09):** traço 1 com Concentration
/// (depth) alta + traço 2 com baixa na MESMA sessão — o re-bake da união resolvia os params
/// CORRENTES do brush pro conjunto ("no mouse up o primeiro traço é convertido para 0.3"), e
/// qualquer mudança propagava pelas poças na janela retangular do composite. Com a tabela de
/// estilos + mapa de dono, cada wash mantém o SEU caráter byte-exato; o traço novo usa o dele.
#[test]
fn watercolor_session_keeps_each_strokes_style() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 2.0, // Concentration ALTA no traço 1
        edge_gain: 1.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let stroke_v = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
        let mut y = 30.0f32;
        while y < 160.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 160.0], PointerPhase::Up));
    };
    stroke_v(&mut t, 60.0); // wash A (banda x ≈ 48..72), baked
    let a_probe: Vec<[u8; 4]> = (80..110u32).map(|y| px(&t, size, 60, y)).collect();
    // Concentration BAIXA no traço 2 — mesma sessão molhada (imediato).
    t.paint.brush.depth = 0.6;
    t.paint.brush_by_mode.fill(t.paint.brush);
    stroke_v(&mut t, 140.0); // wash B, longe do probe de A mas na mesma união/sessão
    let a_after: Vec<[u8; 4]> = (80..110u32).map(|y| px(&t, size, 60, y)).collect();
    assert_eq!(
        a_probe, a_after,
        "o wash 1 deve manter SUA Concentration byte-exata após o re-bake da união"
    );
    // E o wash 2 usa a Concentration DELE (bem mais claro que o 1).
    let g = |x: u32| f32::from(px(&t, size, x, 95)[1]);
    assert!(
        g(140) > g(60) + 30.0,
        "o wash 2 rende com a própria Concentration baixa (B G {:.0} vs A G {:.0})",
        g(140),
        g(60)
    );
}

/// **EDGE-2 (doc 12, W-C): backrun/bloom de ÁGUA LIMPA** — o gesto canônico era inalcançável por
/// construção (Dilution 1 → flow 0 → cobertura 0 → `cw ≤ 0` pulava TODO o caminho). Agora a água
/// carregada poura o canal próprio (`stroke_water`) e o composite a trata como superfície viva:
/// sobre um wash assentado, o interior do pool CLAREIA (lift — "whitened wake") e o pigmento
/// empurrado escurece o CONTORNO serrilhado (anel `água − halo`, Curtis §2.2 "severely darkened
/// edges"). Água em papel branco = nada (lift/dissolve/anel ∝ presença de tinta).
#[test]
fn watercolor_clean_water_backrun_blooms_on_wet_wash() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.35,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 6.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.0, // Rewet OFF — a ÁGUA sozinha tem que produzir o bloom
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash vertical (banda x ≈ 46..74), assentado (seca a sessão).
    assert!(t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
    for _ in 0..140 {
        t.paint_tick(0.5);
    }
    assert!(t.paint.canvas_wet.is_empty(), "sessão do wash deve secar");
    let wash_before = f32::from(px(&t, size, 60, 95)[1]);
    // Gota de ÁGUA PURA (Dilution 1) parada dentro do wash + um rabisco em papel branco.
    // Raio 30 ≫ o blur do halo (12 px): o interior fica livre do casco (raw ≈ halo) e o anel
    // forma no contorno — numa gota pequena o casco cobre o centro (bloom todo-anel, físico).
    t.paint.brush.radius_px = 30.0;
    t.paint.brush.wet_dilution = 1.0;
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([60.0, 95.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([61.0, 95.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 95.0], PointerPhase::Up));
    assert!(t.on_canvas_pointer(cp([150.0, 40.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([151.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([150.0, 40.0], PointerPhase::Up));
    // (a) interior do pool clareou (pigmento empurrado — whitened wake).
    let pool_after = f32::from(px(&t, size, 60, 95)[1]);
    assert!(
        pool_after > wash_before + 10.0,
        "água limpa deve CLAREAR o interior do pool (antes G {wash_before:.0} → depois G {pool_after:.0})"
    );
    // (b) o contorno do pool escureceu em algum ponto do anel (raio ~10-20 px, serrilhado).
    let mut ring_min = 255.0f32;
    for r in 22..=40u32 {
        for &(dx, dy) in &[
            (r as i32, 0),
            (-(r as i32), 0),
            (0, r as i32),
            (0, -(r as i32)),
        ] {
            let (x, y) = ((60 + dx) as u32, (95 + dy) as u32);
            if x < size && y < size {
                ring_min = ring_min.min(f32::from(px(&t, size, x, y)[1]));
            }
        }
    }
    assert!(
        ring_min < wash_before - 8.0,
        "o pigmento empurrado deve ESCURECER o contorno do pool (wash G {wash_before:.0}, anel mín G {ring_min:.0})"
    );
    // (c) água em papel branco = invisível.
    let blank = px(&t, size, 150, 40);
    assert_eq!(
        &blank[..3],
        &[255, 255, 255],
        "água pura sobre papel em branco não deposita nada"
    );
}

/// **EDGE-3 (doc 12, W-C): rim ASSINADO com conservação (Curtis §4.3.3)** — o pigmento que
/// escurece a borda MIGROU do interior/franja; o lobo negativo do unsharp (antes clampado fora)
/// EMPALIDECE a franja. Propriedade refutável: com Edge > 0, o rim escurece E a franja fica MAIS
/// CLARA que a mesma franja com Edge = 0 (na fórmula aditiva antiga, Edge > 0 só podia escurecer
/// ou manter QUALQUER pixel — nunca clarear).
#[test]
fn watercolor_signed_rim_pales_the_fringe() {
    let run = |gain: f32| -> PainterTool {
        let size = 160u32;
        let mut t = white_canvas(size, 8.0);
        t.paint.brush = BrushSpec {
            radius_px: 16.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            depth: 2.0,
            edge_gain: gain,
            edge_spread: 6.0,
            warp: 0.0,
            granulation: 0.0,
            opacity: 0.0, // isolate the signed rim from the body/opacity film (its own test)
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
        let mut y = 30.0f32;
        while y < 130.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([80.0, 130.0], PointerPhase::Up));
        t
    };
    let plain = run(0.0);
    let rimmed = run(3.0);
    let size = 160u32;
    let g = |t: &PainterTool, x: u32| -> f32 {
        let mut a = 0.0f32;
        for y in 60..100u32 {
            a += f32::from(px(t, size, x, y)[1]);
        }
        a / 40.0
    };
    // Interior profundo (x = 84): o tom NÃO desloca com Edge (o resíduo cru-vs-endurecido do
    // inner deslocava o wash inteiro — a reclamação literal da auditoria).
    assert!(
        (g(&rimmed, 84) - g(&plain, 84)).abs() <= 2.0,
        "Edge não pode deslocar o tom do interior (plain G {:.0} vs rimmed G {:.0})",
        g(&plain, 84),
        g(&rimmed, 84)
    );
    // Rim (justo dentro da silhueta, x ≈ 90): mais escuro com Edge.
    assert!(
        g(&rimmed, 90) < g(&plain, 90) - 8.0,
        "o rim deve escurecer com Edge (plain G {:.0} vs rimmed G {:.0})",
        g(&plain, 90),
        g(&rimmed, 90)
    );
    // Franja (onde inner > cw, x ≈ 94): mais CLARA com Edge — o lobo negativo (conservação).
    let (fp, fr) = (g(&plain, 94), g(&rimmed, 94));
    assert!(
        fr > fp + 4.0,
        "a franja deve EMPALIDECER com Edge — pigmento migrou pro rim (plain G {fp:.0} vs rimmed G {fr:.0})"
    );
}

/// **EDGE-4 (doc 12, W-C): o rim conta a história do gesto** — a amplitude deixa de ser uniforme:
/// onde o pincel DEMOROU (soak/dwell) o rim fortalece (`gain·(1 + k·soak)`), onde o depósito foi
/// tênue ele esmaece (`×(0.5 + 0.5·alpha)`). Propriedade: segurando o pincel parado num ponto do
/// traço (com Bleed > 0, o que poura dwell), o rim ADJACENTE ao dwell sai mais escuro que o rim
/// do resto do traço — mesma geometria, história diferente.
#[test]
fn watercolor_rim_strengthens_where_the_brush_dwelled() {
    let size = 160u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.15,
        depth: 1.0, // rim em MEIO-TOM: no escuro o Beer–Lambert comprime e o boost some
        edge_gain: 0.8, // abaixo do clamp do edge (≤1) — o boost do dwell precisa de headroom
        edge_spread: 6.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.5, // Bleed on — the dwell pours soak
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
    }
    // PARK at (80, 120) — the heartbeat pours dwell under the nib.
    for _ in 0..30 {
        t.paint_tick(0.1); // 3 s parked
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));
    // Rim column (x ≈ 88, just inside the silhouette): dwell zone (y ≈ 118-124) vs plain zone
    // (y ≈ 55-75, far from both the head and the dwell).
    let rim = |y0: u32, y1: u32| -> f32 {
        let mut a = 0.0f32;
        for y in y0..y1 {
            a += f32::from(px(&t, size, 88, y)[1]);
        }
        a / (y1 - y0) as f32
    };
    let (plain, dwelled) = (rim(55, 75), rim(116, 126));
    assert!(
        dwelled < plain - 6.0,
        "o rim deve FORTALECER onde o pincel demorou (rim plain G {plain:.0} vs dwell G {dwelled:.0})"
    );
}

/// **W-C reprodutibilidade (Enio smoke 2026-07-09, "área retangular clareia a poça vizinha"):**
/// o composite é função PURA do estado da sessão — re-renderizar a janela viva do traço 2 sobre
/// pixels JÁ ASSADOS do traço 1 reproduz o bake byte-exato. Antes: os campos de rewet liam o
/// base per-stroke (que contém a poça 1 assada) → dissolve/pool/mix "re-molhavam" a vizinha só
/// por ela cair na janela; o settle da granulação seguia a flag `commit` do frame; o soak (dwell)
/// zerava a cada pen-down. O probe fica FORA da zona de fusão legítima (EDGE-1 derrete rims que
/// se aproximam dentro do raio do blur).
#[test]
fn watercolor_session_rerender_reproduces_the_bake_byte_exact() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.5, // cobre o vazamento do settle (fonte = Same as Paper, default)
        wet_rewet: 0.5,   // Bleed on — arma o caminho de rewet + o dwell
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A (banda x ≈ 48..72) com DWELL no fim — o soak boosta o rim do bake (EDGE-4).
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    for _ in 0..20 {
        t.paint_tick(0.15); // 3 s parado — poura dwell em (60, 120)
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..140u32 {
            for x in 40..=66u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Traço B na MESMA sessão, perto mas sem encostar (gap ≥ 4 px de cobertura): a janela viva
    // dele cobre a borda direita de A, mas a cobertura não — A não pode mudar UM byte.
    assert!(t.on_canvas_pointer(cp([88.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([88.0, y], PointerPhase::Move));
    }
    assert_eq!(
        probe(&t),
        baked,
        "o re-render VIVO da janela de B não pode alterar a poça assada de A"
    );
    t.on_canvas_pointer(cp([88.0, 120.0], PointerPhase::Up));
    assert_eq!(
        probe(&t),
        baked,
        "o re-bake da união no pen-up de B deve reproduzir A byte-exato"
    );
    // Sanidade: B realmente pintou (o teste não passa por janela vazia).
    assert!(px(&t, size, 88, 80)[1] < 250, "B pintou de verdade");
}

/// **W-C deriva de params (doc 13, "qualquer mudança no brush propaga pelas poças"):** trocar o
/// brush entre traços da sessão (Size/Spread/cor/…) não pode re-renderizar a poça assada com a
/// GEOMETRIA do brush novo — `core_r` (raio do blur do rim) e `spread_thin` agora são por-DONO
/// na tabela de estilos, como os params de aparência (#1); o raio dos campos usa o máximo da
/// sessão (inerte em canvas virgem, onde os campos são zero).
#[test]
fn watercolor_session_brush_changes_do_not_touch_baked_washes() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.5,
        wet_rewet: 0.5,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A (banda x ≈ 48..72), assado.
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..140u32 {
            for x in 40..=72u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Brush RADICALMENTE diferente pro traço B (mesma sessão): size 24, Spread 24, corpo raso,
    // Concentration alta, azul. Nada disso pode tocar os bytes assados de A.
    t.paint.brush.radius_px = 24.0;
    t.paint.brush.edge_spread = 24.0;
    t.paint.brush.fill = 0.1;
    t.paint.brush.depth = 3.0;
    t.paint.brush.edge_gain = 0.3;
    t.paint.brush.wet_rewet = 0.2;
    t.paint.brush.granulation = 0.0;
    t.paint.brush.color = [0.1, 0.2, 0.9];
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([106.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([106.0, y], PointerPhase::Move));
    }
    assert_eq!(
        probe(&t),
        baked,
        "a janela viva do brush trocado não pode re-estilizar/re-blurar a poça assada de A"
    );
    t.on_canvas_pointer(cp([106.0, 120.0], PointerPhase::Up));
    assert_eq!(
        probe(&t),
        baked,
        "o re-bake da união com o brush trocado deve reproduzir A byte-exato"
    );
    assert!(px(&t, size, 106, 80)[0] < 250, "B (azul) pintou de verdade");
}

/// #13 (doc 14, smoke Enio 2026-07-10): mudar o SUBSTRATO (paper kind / Same-as-Paper / grain)
/// entre traços da MESMA sessão não pode re-renderizar a poça assada de A com o substrato NOVO —
/// o sintoma do "aplica a tudo" + retângulos. O substrato precisa ser POR-DONO como a geometria e
/// a aparência (#1). RED até o fix por-dono do #13 (o composite lê paper/gran GLOBAIS do brush vivo).
#[test]
fn watercolor_session_substrate_change_does_not_touch_baked_washes() {
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 24.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        edge_gain: 1.0,
        edge_spread: 24.0,
        warp: 0.0,
        granulation: 0.9, // forte, pra o substrato pesar no bake
        wet_rewet: 0.3,
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::PaperCold;
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A (banda x ≈ 36..84), assado com PaperCold.
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    // Sonda o NÚCLEO de A (x 48..64): dono de A e DENTRO da janela de re-bake de B (dab 72..120
    // padded 26 = 46..146), mas FORA da cobertura de B (disco em x=96, borda esquerda ~72) — então
    // B nunca vira dono ali. Só o substrato global (bug) mudaria esses bytes no commit de B.
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..140u32 {
            for x in 48..=64u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Traço B (mesma sessão, mesma geometria) TROCA o papel para None — a janela larga de B cobre A,
    // e o composite re-texturiza a poça assada de A com o substrato novo (o bug).
    t.paint.brush.paper.kind = TextureKind::None;
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([96.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([96.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([96.0, 120.0], PointerPhase::Up));
    assert_eq!(
        probe(&t),
        baked,
        "trocar o papel re-texturizou a poça assada de A (substrato global — bug #13)"
    );
}

/// **Costura na junção (Enio smoke 2026-07-09, cruz rápida com Dilution — take knobs):** a linha
/// dura seguia a fronteira de ROUBO DE DONO do traço novo dentro da união (a presença union crua
/// do `lift_wash` e o deepen full-strength no gate flipavam em 1 px ali — 29 bytes medidos).
/// Fix zero-custo: `lift_wash` lê a presença BORRADA (`bp_u`, já amostrada pro anel) e o
/// `backrun` escala pela presença da fonte. Mesmos params nos dois traços — a junção tem que
/// derreter (o degrau residual é o taper suave da água).
#[test]
fn watercolor_water_junction_owner_line_is_smooth() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.35,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.3,
        wet_dilution: 0.5,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // VERTICAL primeiro (x = 60), HORIZONTAL por último (y = 95) — a ordem do smoke; a fronteira
    // de dono do horizontal corta o vertical em y ≈ 81.
    assert!(t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
    assert!(t.on_canvas_pointer(cp([20.0, 95.0], PointerPhase::Down)));
    let mut x = 20.0f32;
    while x < 170.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 95.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([170.0, 95.0], PointerPhase::Up));
    let mut max_step = 0.0f32;
    let mut at_y = 0u32;
    for y in 60..95u32 {
        let a = f32::from(px(&t, size, 60, y)[1]);
        let b = f32::from(px(&t, size, 60, y + 1)[1]);
        if (a - b).abs() > max_step {
            max_step = (a - b).abs();
            at_y = y;
        }
    }
    assert!(
        max_step <= 15.0,
        "a junção da cruz deve derreter — degrau máx G {max_step:.0} em y={at_y} \
         (sem o fix a linha de dono degrauzava 29)"
    );
}

/// **O retângulo do preview com Dilution (Enio 2026-07-09, "só com charge 1 e dilution > 0"):**
/// um traço com Dilution rega o PRÓPRIO corpo; a máscara dos campos union excluía o traço VIVO
/// (`owner != cur`) — instável: no bake do traço A o próprio pigmento fica de fora (sem
/// auto-anel), mas no pen-down de B o A vira "estrangeiro" e lift/anel RETROAGEM sobre o wash
/// inteiro dentro da janela viva de B (o retângulo; some no pen-up porque o commit re-assa tudo
/// uniforme). Água agora interage só com tinta SECA (base da sessão); molhado funde pela união.
/// Propriedade: os bytes assados de A não mudam UM byte durante o traço B.
#[test]
fn watercolor_diluted_wash_is_not_retroactively_rewetted_by_next_stroke() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.5,
        wet_rewet: 0.3,
        wet_dilution: 0.6, // o gatilho do smoke (Charge 1 default)
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Wash A assado (banda x ≈ 48..72), com a própria água (Dilution).
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..110u32 {
            for x in 44..=66u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Traço B perto (sem encostar): a janela viva cobre A — A não pode mudar um byte.
    assert!(t.on_canvas_pointer(cp([92.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([92.0, y], PointerPhase::Move));
    }
    assert_eq!(
        probe(&t),
        baked,
        "o wash diluído assado não pode ser re-molhado retroativamente na janela viva de B"
    );
    t.on_canvas_pointer(cp([92.0, 120.0], PointerPhase::Up));
    assert_eq!(probe(&t), baked, "nem no re-bake da união do pen-up de B");
}

/// Regressão do smoke 2026-07-09 (gesto A, doc 12 take 8): o diag `[wet-diag]` mostrou
/// `sess=false` no pen-down IMEDIATAMENTE após o Enio reduzir o slider de Charge — mexer num
/// slider watercolor NÃO pode quebrar a sessão molhada (o traço seguinte vira glaze sobre
/// "seco" e ganha o rim duro by-design na junção: a "borda dura ao reduzir Charge"). O gesto
/// com Charge intacto (2→3) manteve `sess=true`, isolando o slider como o gatilho.
#[test]
fn watercolor_wet_session_survives_charge_slider_change() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 20.0,
        watercolor: true,
        wet_charge: 1.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([60.0, 128.0], PointerPhase::Down)));
    for i in 1..=20 {
        t.on_canvas_pointer(cp([60.0 + i as f32 * 7.0, 128.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([200.0, 128.0], PointerPhase::Up));
    assert!(
        t.wet_session_continues(),
        "controle: a sessão deve estar viva logo após o bake"
    );
    // O caminho REAL do painel (handle_panel_event → route_brush_watercolor_event →
    // set_brush_wet_charge), não o setter puro.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_CHARGE,
        0.4608,
    ));
    let n = (size as usize) * (size as usize);
    assert!(
        t.wet_session_continues(),
        "mexer no slider de Charge quebrou a sessão molhada — guards: wet_rect={} cov={} col={} \
         base={} arc={}",
        t.paint.canvas_wet_rect.is_some(),
        t.paint.stroke_coverage.len() == n,
        t.paint.stroke_color.len() == n * 4,
        t.paint
            .wet_session_base
            .as_ref()
            .is_some_and(|b| b.len() == n * 4),
        t.paint
            .wet_session_canvas
            .as_ref()
            .is_some_and(|c| Arc::ptr_eq(c, &t.canvas_rgba)),
    );
}

/// Diag do take 10 (rode com `--ignored --nocapture`): perfil 1D de luminância pela junção
/// (eixo do traço 2) vs a borda orgânica do próprio wash — quantifica a DUREZA da fronteira
/// do clareamento (bytes/px) pra calibrar a spec de suavidade.
#[test]
#[ignore = "diag exploratório — imprime perfis de transição da junção (take 10)"]
fn watercolor_junction_transition_profile() {
    for (label, wet) in [("chg<1 wet=0", 0.0f32), ("chg<1 wet=1", 1.0f32)] {
        let size = 600u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 49.8,
            color: [1.0, 0.27, 0.27],
            spacing: 0.05,
            watercolor: true,
            fill: 0.120,
            depth: 1.20,
            edge_gain: 0.70,
            edge_spread: 22.8,
            warp: 11.1,
            granulation: 0.30,
            wet_charge: 0.4841,
            wet_dilution: 0.2918,
            wet_pull: 0.22,
            wet_rewet: 0.0, // traço 1 sem rewet (como no smoke)
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        // Traço 1: vertical por x=300.
        assert!(t.on_canvas_pointer(cp([300.0, 80.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([300.0, 80.0 + i as f32 * 11.0], PointerPhase::Move));
            t.on_tick(16.0);
        }
        t.on_canvas_pointer(cp([300.0, 520.0], PointerPhase::Up));
        t.on_tick(16.0);
        // Traço 2: horizontal por y=300, com o Rewet do caso.
        t.paint.brush.wet_rewet = wet;
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([80.0, 300.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([80.0 + i as f32 * 11.0, 300.0], PointerPhase::Move));
            // Traço LENTO real (o soak do smoke: 30k px) — ~8 frames por move.
            for _ in 0..8 {
                t.on_tick(16.0);
            }
        }
        t.on_canvas_pointer(cp([520.0, 300.0], PointerPhase::Up));
        eprintln!(
            "[profile] soak_px={} water={}",
            t.paint.wet_soak.iter().filter(|&&v| v > 0).count(),
            !t.paint.stroke_water.is_empty(),
        );
        let px = &t.canvas_rgba;
        // Luminância média numa faixa 11px de altura centrada em y=300, x de 180 a 420.
        let lum = |x: usize| -> f32 {
            let mut s = 0.0f32;
            for y in 295..306usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        let prof: Vec<f32> = (180..420).map(lum).collect();
        let mut grad_max = 0.0f32;
        let mut grad_at = 0usize;
        for i in 1..prof.len() {
            let g = (prof[i] - prof[i - 1]).abs();
            if g > grad_max {
                grad_max = g;
                grad_at = 180 + i;
            }
        }
        // Baseline orgânico: a borda superior do traço 2 longe da junção (scan vertical em x=150).
        let lumv = |y: usize| -> f32 {
            let mut s = 0.0f32;
            for x in 145..156usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        let vprof: Vec<f32> = (220..300).map(lumv).collect();
        let mut vgrad_max = 0.0f32;
        for i in 1..vprof.len() {
            vgrad_max = vgrad_max.max((vprof[i] - vprof[i - 1]).abs());
        }
        eprintln!(
            "[profile {label}] junção: grad_max={grad_max:.1} bytes/px em x={grad_at} | \
             borda própria: grad_max={vgrad_max:.1} bytes/px | razão={:.2}",
            grad_max / vgrad_max.max(0.01)
        );
        let cells: Vec<String> = prof.iter().step_by(8).map(|v| format!("{v:.0}")).collect();
        eprintln!("[profile {label}] perfil x180..420/8: {}", cells.join(" "));
        // Scan VERTICAL em x=300 (dentro do traço 1): cruza a borda do footprint do traço 2 —
        // a fronteira do lift/rewet DENTRO da tinta velha (os arcos duros da foto do take 10).
        let lumj = |y: usize| -> f32 {
            let mut s = 0.0f32;
            for x in 295..306usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        let jprof: Vec<f32> = (180..420).map(lumj).collect();
        let mut jgrad_max = 0.0f32;
        let mut jgrad_at = 0usize;
        for i in 1..jprof.len() {
            let g = (jprof[i] - jprof[i - 1]).abs();
            if g > jgrad_max {
                jgrad_max = g;
                jgrad_at = 180 + i;
            }
        }
        eprintln!(
            "[profile {label}] lift-boundary (scan vertical x=300): grad_max={jgrad_max:.1}              bytes/px em y={jgrad_at} | razão vs borda própria={:.2}",
            jgrad_max / vgrad_max.max(0.01)
        );
        let jcells: Vec<String> = jprof.iter().step_by(8).map(|v| format!("{v:.0}")).collect();
        eprintln!("[profile {label}] perfil y180..420/8: {}", jcells.join(" "));
        // Sonda dos MAPAS no penhasco (x=300, y=jgrad_at±8): quem degraua ali?
        for y in (jgrad_at.saturating_sub(8))..(jgrad_at + 9) {
            let idx = y * size as usize + 300;
            eprintln!(
                "[maps {label}] y={y} lum={:.0} col_a={} depl={} cov={} own={}",
                lumj(y),
                t.paint.stroke_color.get(idx * 4 + 3).copied().unwrap_or(0),
                t.paint.stroke_deplete.get(idx).copied().unwrap_or(0),
                t.paint.stroke_coverage.get(idx).copied().unwrap_or(0),
                t.paint.wet_styles.owner.get(idx).copied().unwrap_or(0),
            );
        }
        // Buffers do caminho wet no mesmo corte: água do traço 2 + soak (cru) — qual degraua?
        for y in (jgrad_at.saturating_sub(8))..(jgrad_at + 9) {
            let idx = y * size as usize + 300;
            eprintln!(
                "[wetmaps {label}] y={y} water={} soak={}",
                t.paint.stroke_water.get(idx).copied().unwrap_or(0),
                t.paint.wet_soak.get(idx).copied().unwrap_or(0),
            );
        }
    }
}

/// Spec do take 10 (smoke 2026-07-09, rodada 3 + foto): o CLAREAMENTO da junção é o look
/// desejado ("perdeu o efeito de clareamento" — veto ao clamp do take 9), o defeito é só a
/// FRONTEIRA dura dele. Dois portadores corrigidos: (a) o flip raw↔depositado na janela fixa
/// `COL_LO..COL_HI` atravessada em ~1px espacial (→ lerp proporcional `ca8/255`); (b) o `st.wet`
/// BINÁRIO do mapa de dono nos termos wet-driven quando o Rewet difere entre traços da sessão
/// (→ campo borrado `wet_field`). Este gate exige AMBOS: clareia E suave.
#[test]
fn watercolor_junction_lightening_is_soft_and_preserved() {
    for (label, wet2) in [("wet=0", 0.0f32), ("wet=1", 1.0f32)] {
        let size = 600u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 49.8,
            color: [1.0, 0.27, 0.27],
            spacing: 0.05,
            watercolor: true,
            fill: 0.120,
            depth: 1.20,
            edge_gain: 0.70,
            edge_spread: 22.8,
            warp: 11.1,
            granulation: 0.30,
            wet_charge: 0.4841,
            wet_dilution: 0.2918,
            wet_pull: 0.22,
            wet_rewet: 0.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([300.0, 80.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([300.0, 80.0 + i as f32 * 11.0], PointerPhase::Move));
            t.on_tick(16.0);
        }
        t.on_canvas_pointer(cp([300.0, 520.0], PointerPhase::Up));
        t.on_tick(16.0);
        t.paint.brush.wet_rewet = wet2;
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([80.0, 300.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([80.0 + i as f32 * 11.0, 300.0], PointerPhase::Move));
            for _ in 0..8 {
                t.on_tick(16.0);
            }
        }
        t.on_canvas_pointer(cp([520.0, 300.0], PointerPhase::Up));
        let px = &t.canvas_rgba;
        // Scan vertical em x=300 (dentro do traço 1, cruzando a fronteira do traço 2).
        let lum = |y: usize| -> f32 {
            let mut s = 0.0f32;
            for x in 295..306usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        // (a) O clareamento EXISTE: platô da junção > corpo do traço 1.
        let body: f32 = (200..230).map(lum).sum::<f32>() / 30.0;
        let plateau: f32 = (290..311).map(lum).sum::<f32>() / 21.0;
        assert!(
            plateau > body + 2.0,
            "[{label}] o clareamento da junção sumiu (veto do take 9): corpo={body:.1} \
             platô={plateau:.1}"
        );
        // (b) A fronteira é SUAVE: nenhum degrau de 1px acima de 4 bytes no scan interno
        // (pré-fix: 7.5 com wet=0, 11.5 com wet=1).
        let prof: Vec<f32> = (200..400).map(lum).collect();
        let mut grad_max = 0.0f32;
        let mut grad_at = 0usize;
        for i in 1..prof.len() {
            let g = (prof[i] - prof[i - 1]).abs();
            if g > grad_max {
                grad_max = g;
                grad_at = 200 + i;
            }
        }
        assert!(
            grad_max <= 4.0,
            "[{label}] fronteira dura na junção: grad {grad_max:.1} bytes/px em y={grad_at}"
        );
    }
}

/// #11 (doc 13): o slider de Drying Time mapeia SEGUNDOS → taxa de secagem (`255/seg`) e volta,
/// com clamp em `2..60 s`. Canvas-level (não muda por modo de pincel).
#[test]
fn watercolor_dry_time_slider_maps_seconds_to_rate() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 16 * 16 * 4], 16, 16);
    t.set_dry_time_s(10.0);
    assert!((t.dry_time_s() - 10.0).abs() < 0.05, "10 s round-trips");
    assert!(
        (t.paint.dry_rate_per_s - 25.5).abs() < 0.1,
        "10 s ⇒ ~25.5 bytes/s"
    );
    t.set_dry_time_s(60.0);
    assert!(
        (t.paint.dry_rate_per_s - 4.25).abs() < 0.05,
        "60 s ⇒ ~4.25 bytes/s"
    );
    t.set_dry_time_s(0.5); // abaixo do mínimo → clamp em 2 s
    assert!((t.dry_time_s() - 2.0).abs() < 0.05, "clamp inferior 2 s");
    t.set_dry_time_s(999.0); // acima do máximo → clamp em 60 s
    assert!((t.dry_time_s() - 60.0).abs() < 0.05, "clamp superior 60 s");
    // Default = ~10 s (o knob CANVAS_WET_DRY_DEFAULT).
    let fresh = PainterTool::default();
    assert!((fresh.dry_time_s() - 10.0).abs() < 0.05, "default ~10 s");
}

/// #9 (doc 13): o botão Dry encerra a sessão molhada NA HORA — os pixels assados ficam, mas a
/// fusão com traços futuros acaba (canvas_wet zerado, sessão morta).
#[test]
fn watercolor_dry_button_ends_the_wet_session() {
    let size = 64u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 16.0,
        watercolor: true,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down)));
    for i in 1..=12 {
        t.on_canvas_pointer(cp([16.0 + i as f32 * 3.0, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(
        t.wet_session_continues(),
        "precondição: sessão viva após o bake"
    );
    let baked = t.canvas_rgba.clone();
    t.dry_session_now();
    assert!(!t.wet_session_continues(), "Dry encerrou a sessão");
    assert!(t.paint.canvas_wet.is_empty(), "canvas_wet zerado");
    assert!(t.paint.canvas_wet_rect.is_none(), "rect de umidade zerado");
    assert_eq!(t.canvas_rgba, baked, "Dry NÃO toca os pixels assados");
}

/// #10 (doc 13): o botão Wet re-molha o canvas inteiro SEM depositar pigmento (canvas_wet = 255
/// full + rect = canvas todo), sem tocar os pixels.
#[test]
fn watercolor_wet_button_moistens_the_canvas() {
    let size = 32u32;
    let mut t = PainterTool::default();
    let src = vec![200u8; (size * size * 4) as usize];
    t.set_source(src.clone(), size, size);
    t.wet_canvas_now();
    let n = (size * size) as usize;
    assert_eq!(t.paint.canvas_wet.len(), n, "canvas_wet dimensionado");
    assert!(
        t.paint.canvas_wet.iter().all(|&w| w == 255),
        "umidade cheia"
    );
    assert_eq!(
        t.paint.canvas_wet_rect,
        Some((0, 0, size as usize, size as usize)),
        "rect = canvas inteiro"
    );
    assert_eq!(
        t.canvas_rgba.as_slice(),
        src.as_slice(),
        "Wet NÃO deposita pigmento"
    );
}

/// Costura do ROUTE (doc 13 #9-#11): os ids novos da Wetness card despacham pelos setters certos
/// via `route_brush_watercolor_event` — o par do seam.rs do painel (que cobre o forward).
#[test]
fn watercolor_route_dispatches_wetness_controls() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    assert!(t.route_brush_watercolor_event(&PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_DRY_TIME,
        30.0
    )));
    assert!(
        (t.dry_time_s() - 30.0).abs() < 0.05,
        "DRY_TIME → set_dry_time_s"
    );
    assert!(
        t.route_brush_watercolor_event(&PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_WET_NOW))
    );
    assert!(!t.paint.canvas_wet.is_empty(), "WET_NOW → wet_canvas_now");
    assert!(
        t.route_brush_watercolor_event(&PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_DRY_NOW))
    );
    assert!(t.paint.canvas_wet.is_empty(), "DRY_NOW → dry_session_now");
}

/// #12a (doc 14): o accessor `canvas_wet_view` expõe o mapa de umidade + rect para o overlay
/// on-canvas — Some quando molhado, None quando seco. (O véu em si é smoke-only.)
#[test]
fn watercolor_canvas_wet_view_exposes_moisture() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    assert!(t.canvas_wet_view().is_none(), "seco → None");
    t.wet_canvas_now();
    let (bytes, w, h, rect) = t.canvas_wet_view().expect("molhado → Some");
    assert_eq!((w, h), (32, 32));
    assert_eq!(rect, [0, 0, 32, 32], "rect = canvas inteiro");
    assert!(bytes.iter().all(|&b| b == 255), "umidade cheia");
    t.dry_session_now();
    assert!(t.canvas_wet_view().is_none(), "secou → None");
}
