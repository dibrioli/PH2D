use super::*;

/// Uma imagem de 8 × 8 com desenho só em `x ∈ 2..6`, `y ∈ 4..8`: sobra transparente em cima e dos lados.
fn boneco() -> Vec<u8> {
    let mut px = vec![0_u8; 8 * 8 * 4];
    for y in 4..8 {
        for x in 2..6 {
            px[(y * 8 + x) * 4 + 3] = 255;
        }
    }
    px
}

#[test]
fn a_caixa_opaca_ignora_a_sobra_transparente() {
    assert_eq!(caixa_opaca(&boneco(), 8, [0, 0, 8, 8]), Some([2, 4, 6, 8]));
    assert_eq!(caixa_opaca(&boneco(), 8, [0, 0, 8, 4]), None);
}

#[test]
fn numa_folha_a_caixa_e_a_uniao_das_celulas() {
    // Duas células de 4 × 8: a esquerda desenha em x 2..4, a direita em x 0..2 (relativos à célula).
    let f = fracao_opaca(&boneco(), 8, [0, 0, 8, 8], [2, 1]).expect("tem desenho");
    assert_eq!(f, [0.0, 0.5, 1.0, 1.0]);
}

fn mundo_com(s: Sprite, t: Transform) -> (World, Entity) {
    let mut w = World::new();
    let e = w.spawn((s, t)).id();
    (w, e)
}

fn sprite_2x2() -> Sprite {
    Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [2.0, 2.0], [1.0; 4])
}

/// ⭐ A lei: o desenho não anda. O centro do quad antes e depois é o mesmo, com rotação e escala.
#[test]
fn mudar_o_pivo_nao_move_o_desenho() {
    let mut t = Transform {
        translation: Vec2::new(5.0, 7.0),
        ..Default::default()
    };
    t.rotation = 0.7;
    t.scale = Vec2::new(2.0, 3.0);
    let (mut w, e) = mundo_com(sprite_2x2(), t);
    let centro = |w: &World| {
        let s = w.get::<Sprite>(e).unwrap();
        no_pai(*w.get::<Transform>(e).unwrap(), s.resolve_anchor(100.0))
    };
    let antes = centro(&w);
    assert!(pousa(&mut w, e, 100.0, [0.0, -1.0]));
    let depois = centro(&w);
    assert!((antes - depois).length() < 1.0e-4, "{antes:?} → {depois:?}");
    assert_eq!(w.get::<Sprite>(e).unwrap().anchor, [0.0, 1.0]);
    assert_ne!(w.get::<Transform>(e).unwrap().translation, t.translation);
}

/// Controle: sem compensar, o mesmo pivô moveria o desenho; a régua acima sabe ver isso.
#[test]
fn a_regua_ve_o_desenho_andar_quando_a_posicao_nao_compensa() {
    let t = Transform {
        rotation: 0.7,
        ..Default::default()
    };
    let s = sprite_2x2();
    let antes = no_pai(t, s.resolve_anchor(100.0));
    let depois = no_pai(t, [0.0, 1.0]);
    assert!((antes - depois).length() > 0.5);
}

#[test]
fn o_pe_e_o_do_desenho_nao_o_da_borda_da_imagem() {
    let s = sprite_2x2();
    let f = fracao_opaca(&boneco(), 8, [0, 0, 8, 8], [1, 1]).unwrap();
    // Desenho em u 0.25..0.75, v 0.5..1: o pé (meio, baixo) é x 0, y −1; o topo do desenho é y 0.
    assert_eq!(ponto_local(&s, f, [1, 2]), [0.0, -1.0]);
    assert_eq!(ponto_local(&s, f, [1, 0]), [0.0, 0.0]);
    assert_eq!(ponto_local(&s, f, [0, 2]), [-0.5, -1.0]);
}

#[test]
fn a_imagem_espelhada_espelha_o_ponto() {
    let mut s = sprite_2x2();
    s.flip_x = true;
    assert_eq!(
        ponto_local(&s, [0.0, 0.0, 1.0, 1.0], [0, 1]),
        [1.0, 0.0],
        "o lado esquerdo do desenho está à direita do quad"
    );
}

/// Um ficheiro antigo com Centered desligado e Offset: o pivô novo normaliza sem mover o desenho.
#[test]
fn centered_e_offset_antigos_normalizam_sem_mover_o_desenho() {
    let mut s = sprite_2x2();
    s.centered = false;
    s.offset = [10.0, 20.0];
    let (mut w, e) = mundo_com(s, Transform::default());
    let centro = |w: &World| {
        let s = w.get::<Sprite>(e).unwrap();
        no_pai(*w.get::<Transform>(e).unwrap(), s.resolve_anchor(100.0))
    };
    let antes = centro(&w);
    assert!(pousa(&mut w, e, 100.0, [0.5, 0.5]));
    assert!((antes - centro(&w)).length() < 1.0e-4);
    let s = w.get::<Sprite>(e).unwrap();
    assert!(s.centered);
    assert_eq!(s.offset, [0.0, 0.0]);
}

#[test]
fn o_pivo_no_mesmo_ponto_nao_muda_nada() {
    let (mut w, e) = mundo_com(sprite_2x2(), Transform::default());
    assert!(!pousa(&mut w, e, 100.0, [0.0, 0.0]));
}

#[test]
fn mudar_o_pivo_nao_move_os_filhos() {
    let db = AssetDb::default();
    let atlas = BTreeMap::new();
    let mut w = World::new();
    let pai = w
        .spawn((
            Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [2.0, 2.0], [1.0; 4]),
            Transform::default(),
        ))
        .id();
    let local = Transform {
        translation: Vec2::new(3.0, 1.0),
        ..Default::default()
    };
    let filho = w.spawn((local, ph2d_ecs::ChildOf(pai))).id();
    assert!(set_pivot(
        &mut w,
        pai,
        &db,
        &atlas,
        100.0,
        SpritePivot::Px([50.0, -100.0])
    ));
    let t_pai = *w.get::<Transform>(pai).unwrap();
    let t_filho = *w.get::<Transform>(filho).unwrap();
    let no_mundo = Transform::compose(t_pai, t_filho).translation;
    assert!(
        (no_mundo.x - 3.0).abs() < 1e-4 && (no_mundo.y - 1.0).abs() < 1e-4,
        "{no_mundo:?}"
    );
}
