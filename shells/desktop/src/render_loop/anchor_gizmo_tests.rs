//! Os testes do gizmo da §12 — irmão de [`super::anchor_gizmo`] por assunto.
//!
//! ⚠️ **O que eles provam é a classe de erro que gizmo tem**, e nenhuma delas é de compilação:
//! a alça que agarra a vizinha · o arrasto que anda ao contrário · o canto que arrasta o canto
//! oposto · a âncora rodada que se move na direção para onde APONTA em vez de para onde o rato
//! foi. Todas passam num código que compila.

use super::*;

fn ppm() -> f32 {
    100.0
}

/// Um sprite deslocado, rodado e escalado — **as três coisas ao mesmo tempo**, de propósito.
/// Um fixture na identidade daria certo com a matemática errada.
fn sprite() -> Transform {
    Transform {
        translation: Vec2::new(3.0, -2.0),
        rotation: std::f32::consts::FRAC_PI_3,
        scale: Vec2::new(2.0, 1.5),
        ..Default::default()
    }
}

fn anchor_at(px: f32, py: f32) -> NamedAnchor {
    let mut a = NamedAnchor::socket("muzzle");
    a.transform.translation = Vec2::new(px / ppm(), py / ppm());
    a
}

/// **Ida e volta**: arrastar o centro para o sítio onde ele já está não o move.
///
/// ⚠️ É o teste de identidade do gizmo, e ele apanha a classe inteira dos erros de base — um eixo
/// trocado, uma escala esquecida, um `ppm` a mais ou a menos falham aqui.
#[test]
fn dragging_the_centre_to_where_it_already_is_moves_nothing() {
    let a = anchor_at(12.0, -7.0);
    let (hs, n) = handles(sprite(), &a, ppm());
    let centre = hs[0].unwrap().world;
    assert_eq!(hs[0].unwrap().kind, AnchorHandleKind::Centre);
    assert_eq!(n, 2, "sem rects, sao duas alcas: centro e rotacao");

    let (edits, k) = drag(AnchorHandleKind::Centre, 0, &a, sprite(), centre, ppm());
    assert_eq!(k, 2, "mover escreve os DOIS eixos");
    match (&edits[0], &edits[1]) {
        (Some(AnchorFieldEdit::Pos(0, 0, x)), Some(AnchorFieldEdit::Pos(0, 1, y))) => {
            assert!((x - 12.0).abs() < 1e-2, "x saiu {x}, esperava 12");
            assert!((y + 7.0).abs() < 1e-2, "y saiu {y}, esperava -7");
        }
        other => panic!("edicoes erradas: {other:?}"),
    }
}

/// ⚠️ **Uma âncora RODADA move-se para onde o rato foi, não para onde ela aponta.**
///
/// O centro vive no espaço do SPRITE; se ele usasse a base da própria âncora, arrastar para a
/// direita uma âncora a 90° movê-la-ia para cima — o erro que se sente como «o gizmo está
/// possuído» e que nenhum teste de compilação vê.
#[test]
fn a_rotated_anchor_still_moves_where_the_pointer_went() {
    let mut a = anchor_at(0.0, 0.0);
    a.transform.rotation = std::f32::consts::FRAC_PI_2; // 90°
    let world = Transform::default();
    // Um alvo a 10 px à direita, no espaço do sprite (que aqui é a identidade).
    let target = Vec2::new(10.0 / ppm(), 0.0);
    let (edits, k) = drag(AnchorHandleKind::Centre, 0, &a, world, target, ppm());
    assert_eq!(k, 2);
    match (&edits[0], &edits[1]) {
        (Some(AnchorFieldEdit::Pos(_, 0, x)), Some(AnchorFieldEdit::Pos(_, 1, y))) => {
            assert!(
                (x - 10.0).abs() < 1e-3,
                "x saiu {x}: a base da ANCORA vazou"
            );
            assert!(y.abs() < 1e-3, "y saiu {y}: devia ficar em zero");
        }
        other => panic!("{other:?}"),
    }
}

/// A alça de rotação devolve o ângulo que o ponteiro faz — em GRAUS, como o campo do painel.
#[test]
fn the_rotate_handle_reports_the_angle_the_pointer_makes() {
    let a = anchor_at(0.0, 0.0);
    let world = Transform::default();
    for (target, want_deg) in [
        (Vec2::new(1.0, 0.0), 0.0_f32),
        (Vec2::new(0.0, 1.0), 90.0),
        (Vec2::new(-1.0, 0.0), 180.0),
    ] {
        let (edits, k) = drag(AnchorHandleKind::Rotate, 3, &a, world, target, ppm());
        assert_eq!(k, 1);
        let Some(AnchorFieldEdit::Rot(row, deg)) = edits[0] else {
            panic!("{:?}", edits[0])
        };
        assert_eq!(row, 3, "a edicao foi para a linha errada");
        assert!(
            (deg - want_deg).abs() < 1e-2 || (deg.abs() - want_deg.abs()).abs() < 1e-2,
            "alvo {target:?} deu {deg}°, esperava {want_deg}°"
        );
    }
}

/// **Em cima do centro não há direção** — e saltar para 0° seria uma edição que o artista não
/// pediu, no meio de um arrasto.
#[test]
fn rotating_from_the_dead_centre_writes_nothing() {
    let a = anchor_at(4.0, 4.0);
    let centre = anchor_world_point(sprite(), &a, [0.0, 0.0], ppm());
    let (_, k) = drag(AnchorHandleKind::Rotate, 0, &a, sprite(), centre, ppm());
    assert_eq!(k, 0, "escreveu um angulo sem haver direcao");
}

/// ⚠️ **O canto OPOSTO fica quieto** — é isso que faz o gesto ser «redimensionar».
///
/// Arrastar o canto `0` (o de `x,y`) para longe tem de mover a origem e o tamanho, e deixar o
/// canto `2` exatamente onde estava. Um erro de `+2 % 4` faz o retângulo saltar.
#[test]
fn dragging_a_corner_keeps_the_opposite_one_still() {
    let mut a = anchor_at(0.0, 0.0);
    a.set_bounds(Some([10.0, 20.0, 30.0, 40.0]));
    let world = Transform::default();
    let before_opposite = corner(a.bounds.unwrap(), 2); // (40, 60)
    assert_eq!(before_opposite, [40.0, 60.0]);

    // Puxa o canto 0 para (4, 8) em px da fonte.
    let target = Vec2::new(4.0 / ppm(), 8.0 / ppm());
    let (edits, k) = drag(AnchorHandleKind::Bounds(0), 1, &a, world, target, ppm());
    assert_eq!(k, 4, "um rect tem quatro componentes");
    let mut rect = [0.0f32; 4];
    for e in edits.iter().flatten() {
        let Some(AnchorFieldEdit::Bounds(row, f, v)) = Some(e.clone()) else {
            panic!("{e:?}")
        };
        assert_eq!(row, 1);
        rect[usize::from(f)] = v;
    }
    assert!((rect[0] - 4.0).abs() < 1e-3, "x saiu {}", rect[0]);
    assert!((rect[1] - 8.0).abs() < 1e-3, "y saiu {}", rect[1]);
    // E o canto oposto continua em (40, 60): x+w e y+h.
    assert!(
        (rect[0] + rect[2] - 40.0).abs() < 1e-3,
        "o canto oposto ANDOU"
    );
    assert!(
        (rect[1] + rect[3] - 60.0).abs() < 1e-3,
        "o canto oposto ANDOU"
    );
}

/// Um retângulo nunca colapsa a zero: abaixo de um pixel da fonte ele deixa de endereçar um
/// texel, e a alça ficaria inagarrável.
#[test]
fn a_dragged_rect_never_collapses_below_one_source_pixel() {
    let mut a = anchor_at(0.0, 0.0);
    a.set_bounds(Some([0.0, 0.0, 20.0, 20.0]));
    let world = Transform::default();
    // Arrasta o canto 0 EXATAMENTE para cima do oposto.
    let target = Vec2::new(20.0 / ppm(), 20.0 / ppm());
    let (edits, _) = drag(AnchorHandleKind::Bounds(0), 0, &a, world, target, ppm());
    for e in edits.iter().flatten() {
        if let AnchorFieldEdit::Bounds(_, f, v) = e
            && *f >= 2
        {
            assert!(*v >= MIN_RECT_PX, "lado {f} colapsou para {v}");
        }
    }
}

/// ⚠️ **Empate resolve-se pela alça MAIS ESPECÍFICA.** Um retângulo com origem em `[0,0]` põe o
/// canto `0` em cima do centro; ganhar o centro deixaria aquele canto inagarrável para sempre.
#[test]
fn a_corner_on_top_of_the_centre_still_wins_the_pick() {
    let mut a = anchor_at(0.0, 0.0);
    a.set_bounds(Some([0.0, 0.0, 20.0, 20.0]));
    let world = Transform::default();
    let (hs, n) = handles(world, &a, ppm());
    assert_eq!(n, 6, "centro + rotacao + quatro cantos");
    let at_origin = anchor_world_point(world, &a, [0.0, 0.0], ppm());
    assert_eq!(
        hit(&hs, n, at_origin, 0.01),
        Some(AnchorHandleKind::Bounds(0)),
        "o centro comeu o canto que estava debaixo dele"
    );
}

/// Longe de tudo não agarra nada — o controlo negativo do `hit`.
#[test]
fn a_pick_far_from_every_handle_grabs_nothing() {
    let a = anchor_at(0.0, 0.0);
    let (hs, n) = handles(sprite(), &a, ppm());
    assert_eq!(hit(&hs, n, Vec2::new(999.0, 999.0), 0.05), None);
}

/// As dez alças aparecem quando a âncora tem os dois retângulos — e **só** então. Uma âncora sem
/// área não pode oferecer cantos de uma área que não existe.
#[test]
fn the_handle_count_follows_what_the_anchor_actually_has() {
    let mut a = anchor_at(1.0, 1.0);
    assert_eq!(handles(sprite(), &a, ppm()).1, 2, "socket puro");
    a.set_bounds(Some([0.0, 0.0, 8.0, 8.0]));
    assert_eq!(handles(sprite(), &a, ppm()).1, 6, "com area");
    a.set_center(Some([1.0, 1.0, 4.0, 4.0]));
    assert_eq!(
        handles(sprite(), &a, ppm()).1,
        MAX_HANDLES,
        "com area e miolo"
    );
}

/// Uma escala degenerada não divide por zero — devolve «nada a fazer» em vez de `NaN`, que
/// envenenaria a pose da âncora e sobreviveria ao save.
#[test]
fn a_degenerate_sprite_scale_writes_nothing_instead_of_nan() {
    let a = anchor_at(0.0, 0.0);
    let flat = Transform {
        scale: Vec2::new(0.0, 1.0),
        ..Default::default()
    };
    let (_, k) = drag(
        AnchorHandleKind::Centre,
        0,
        &a,
        flat,
        Vec2::new(1.0, 1.0),
        ppm(),
    );
    assert_eq!(k, 0, "produziu edicao sobre uma base degenerada");
}

/// ⚠️ **Só a âncora ABERTA é agarrável** — a mesma lei do desenho, e é ela que garante que *a
/// alça pintada é a alça que agarra*.
///
/// Sem isto o canvas agarraria uma alça que ninguém desenhou: o artista veria o cursor prender-se
/// a nada e mover uma âncora que ele não abriu.
#[test]
fn only_the_open_row_can_be_grabbed() {
    let mut list = ph2d_ecs::NamedAnchorList::new();
    let mut a0 = NamedAnchor::socket("muzzle");
    a0.transform.translation = Vec2::new(0.0, 0.0);
    let mut a1 = NamedAnchor::socket("hitbox");
    a1.transform.translation = Vec2::new(1.0, 0.0);
    list.insert(a0).unwrap();
    list.insert(a1.clone()).unwrap();
    let world = Transform::default();

    // O centro da âncora 1, em mundo.
    let p1 = anchor_world_point(world, &a1, [0.0, 0.0], ppm());

    // Com a linha 1 aberta: agarra.
    assert_eq!(
        open_drag(world, &list, Some(1), 7, p1, ppm(), 0.05),
        Some(AnchorDrag {
            entity: 7,
            row: 1,
            kind: AnchorHandleKind::Centre,
        })
    );
    // Com a linha 0 aberta: o MESMO ponto não agarra nada — a alça ali não está desenhada.
    assert_eq!(open_drag(world, &list, Some(0), 7, p1, ppm(), 0.05), None);
    // ⚠️ **Sem linha aberta, nada agarra — e o ponto de prova tem de ser o da PRIMEIRA âncora.**
    //
    // A versão anterior deste assert usava `p1` (o centro da segunda) e passava por acidente: uma
    // mutação que trocasse o `open_row?` por `unwrap_or(0)` continuava a devolver `None`, porque
    // aquele ponto não cai numa alça da linha 0. *Um assert cujo ponto de prova não alcança o
    // defeito mede o silêncio* — a mesma forma do gate de layout que media 55,56 nas duas pontas.
    let p0 = anchor_world_point(world, list.iter().next().unwrap(), [0.0, 0.0], ppm());
    assert_eq!(
        open_drag(world, &list, None, 7, p0, ppm(), 0.05),
        None,
        "agarrou a primeira ancora sem haver linha aberta"
    );
    // E o controlo positivo do mesmo ponto: com a linha 0 aberta, ELE agarra.
    assert!(open_drag(world, &list, Some(0), 7, p0, ppm(), 0.05).is_some());
}

/// Uma linha aberta que já não existe (a âncora foi apagada) não agarra — em vez de agarrar a
/// vizinha que herdou o índice.
#[test]
fn an_open_row_past_the_end_grabs_nothing() {
    let mut list = ph2d_ecs::NamedAnchorList::new();
    list.insert(NamedAnchor::socket("only")).unwrap();
    let world = Transform::default();
    let p = anchor_world_point(world, list.iter().next().unwrap(), [0.0, 0.0], ppm());
    assert_eq!(open_drag(world, &list, Some(5), 1, p, ppm(), 0.05), None);
}

/// O arrasto guarda a ENTIDADE — para não migrar de sprite se a seleção mudar a meio do gesto.
#[test]
fn the_drag_remembers_which_sprite_it_belongs_to() {
    let mut list = ph2d_ecs::NamedAnchorList::new();
    list.insert(NamedAnchor::socket("a")).unwrap();
    let world = Transform::default();
    let p = anchor_world_point(world, list.iter().next().unwrap(), [0.0, 0.0], ppm());
    let d = open_drag(world, &list, Some(0), 0xDEAD, p, ppm(), 0.05).unwrap();
    assert_eq!(d.entity, 0xDEAD);
    assert_eq!(d.row, 0);
}
